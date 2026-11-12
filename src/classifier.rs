use std::collections::HashMap;
use std::sync::LazyLock;
use chrono::{DateTime, Duration};
use regex::Regex;

use crate::lilith_stream::LilithValue;
use crate::loot_catalog::LootCatalog;
use crate::model::{BattleRecord, ItemDrop};

static HTML_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
static LEVEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(?:level\s*(\d+)|cấp\s*(?:độ\s*)?(\d+))").unwrap());
static DMG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)%").unwrap());
static PLUNDER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:level\s*(\d+)\s*plunder|tier\s*(\d+)\s*(?:trophies|plunder)|chiến\s*lợi\s*phẩm\s*(?:cấp|bậc)\s*(\d+))").unwrap()
});
static POS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"X:(\d+)\s+Y:(\d+)").unwrap());
static NUMBER_ONLY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)").unwrap());

pub fn clean_text(text: &str) -> String {
    if !text.contains('<') {
        return text.trim().to_string();
    }
    HTML_TAG_REGEX.replace_all(text, "").trim().to_string()
}

pub fn format_timestamp(raw_time: i64) -> (String, String) {
    if raw_time <= 0 {
        return ("N/A".to_string(), "N/A".to_string());
    }

    let secs = if raw_time > 100_000_000_000 {
        raw_time / 1_000_000
    } else {
        raw_time
    };

    let dt_utc = match DateTime::from_timestamp(secs, 0) {
        Some(dt) => dt,
        None => return (raw_time.to_string(), raw_time.to_string()),
    };

    let dt_vn = dt_utc + Duration::try_hours(7).unwrap_or_else(|| Duration::hours(7));

    (
        dt_utc.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        dt_vn.format("%Y-%m-%d %H:%M:%S").to_string(),
    )
}

pub fn extract_account_info(mail: &LilithValue) -> (String, String) {
    let mut account_id = mail
        .get("receiver")
        .and_then(|v| v.as_str())
        .map(|r| r.strip_prefix("player_").unwrap_or(r).trim().to_string())
        .unwrap_or_default();

    let mut player_name = String::new();
    if let Some(body) = mail.get("body") {
        if let Some(content) = body.get("content") {
            if let Some(self_char) = content.get("SelfChar") {
                if let Some(name) = self_char.get("PName").and_then(|v| v.as_str()) {
                    player_name = name.trim().to_string();
                }
                if account_id.is_empty() {
                    if let Some(pid) = self_char.get("PId").and_then(|v| v.as_i64()) {
                        if pid > 0 {
                            account_id = pid.to_string();
                        }
                    }
                }
            }
        }
    }
    (account_id, player_name)
}

pub fn is_barbarian_battle_report(mail: &LilithValue) -> bool {
    let body = match mail.get("body") {
        Some(b) => b,
        None => return false,
    };
    let content = match body.get("content") {
        Some(c) => c,
        None => return false,
    };
    let attacks = match content.get("Attacks").and_then(|a| a.as_table()) {
        Some(a) => a,
        None => return false,
    };

    for (_atk_id, atk) in attacks {
        if atk.get("NpcKillLoot").is_some() {
            return true;
        }
        if let Some(cidt) = atk.get("CIdt") {
            let pid = cidt.get("PId").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let npcbtype = cidt.get("NpcBType").and_then(|v| v.as_f64()).unwrap_or(0.0) as i64;
            let is_fort = cidt.get("IsFort").and_then(|v| v.as_bool()).unwrap_or(false);
            if pid < 0.0 && !is_fort && npcbtype == 1 {
                return true;
            }
        }
    }
    false
}

pub fn get_commander_name(id: u32) -> String {
    crate::model::get_commander_name(id)
}

pub fn is_pvp_battle_report(mail: &LilithValue) -> bool {
    if mail.get("type").and_then(|v| v.as_str()) != Some("Battle") {
        return false;
    }
    if is_barbarian_battle_report(mail) {
        return false;
    }
    let body = match mail.get("body") {
        Some(b) => b,
        None => return false,
    };
    let content = match body.get("content") {
        Some(c) => c,
        None => return false,
    };
    let attacks = match content.get("Attacks").and_then(|a| a.as_table()) {
        Some(a) => a,
        None => return false,
    };

    for (_atk_id, atk) in attacks {
        if let Some(cidt) = atk.get("CIdt") {
            let pid = cidt.get("PId").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if pid > 0.0 {
                return true;
            }
        }
    }
    false
}

pub fn classify_mail(mail: &LilithValue) -> Option<&'static str> {
    let box_name = mail.get("box").and_then(|v| v.as_str()).unwrap_or("");
    if box_name != "Report" && box_name != "Archive" {
        return None;
    }

    if is_barbarian_battle_report(mail) {
        return Some("barbarian");
    }

    if is_pvp_battle_report(mail) {
        return Some("pvp");
    }

    if mail.get("type").and_then(|v| v.as_str()) == Some("Rss") {
        return Some("gathering");
    }

    let body = mail.get("body")?;
    let sub_t = body.get("subType").and_then(|v| v.as_u32()).unwrap_or(0);
    let sub_p = body.get("subParam").and_then(|v| v.as_u32()).unwrap_or(0);

    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .or_else(|| mail.get("title").and_then(|v| v.as_str()))
        .unwrap_or("");
    let sub = body.get("subTitle").and_then(|v| v.as_str()).unwrap_or("");
    let content_raw = body.get("content").and_then(|v| v.as_str()).unwrap_or("");

    let is_canceled = |s: &str| {
        s.contains("canceled") || s.contains("đã bị hủy") || s.contains("Canceled")
    };
    if is_canceled(title) || is_canceled(sub) || is_canceled(content_raw) {
        return None;
    }

    let has_attachments = mail
        .get("attachments")
        .and_then(|a| a.as_table())
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    if sub_t == 11
        && sub_p == 4
        && (has_attachments
            || content_raw.contains("destroyed")
            || content_raw.contains("tiêu diệt")
            || title.contains("destroyed")
            || title.contains("tiêu diệt"))
    {
        return Some("motte");
    }

    if sub_t == 11
        && sub_p == 1
        && (has_attachments
            || content_raw.contains("destroyed")
            || content_raw.contains("phá hủy")
            || title.contains("destroyed")
            || title.contains("phá hủy"))
    {
        return Some("fort");
    }

    let tgt = body
        .get("targetName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = clean_text(content_raw);
    let all_txt = format!("{} {} {} {}", title, sub, content, tgt).to_lowercase();

    if all_txt.contains("canceled") || all_txt.contains("đã bị hủy") {
        return None;
    }

    if all_txt.contains("motte")
        && (has_attachments
            || all_txt.contains("destroyed")
            || all_txt.contains("tiêu diệt"))
    {
        return Some("motte");
    }

    if (all_txt.contains("barbarian fort")
        || (all_txt.contains("pháo đài")
            && (all_txt.contains("dã man") || all_txt.contains("barbarian") || all_txt.contains("barbaria"))))
        && (has_attachments
            || all_txt.contains("destroyed")
            || all_txt.contains("phá hủy"))
    {
        return Some("fort");
    }

    if has_attachments {
        return Some("reward_other");
    }

    None
}

pub fn format_barbarian_target(levels: &[u32]) -> (String, String) {
    crate::model::format_barbarian_target(levels)
}

pub fn extract_barbarian_battle_details(
    mail: &LilithValue,
    filename: &str,
    catalog: &LootCatalog,
) -> Result<BattleRecord, String> {
    let body = mail.get("body");
    let content = body.and_then(|b| b.get("content"));
    let attacks = content
        .and_then(|c| c.get("Attacks"))
        .and_then(|a| a.as_table());

    let raw_time = mail
        .get("time")
        .and_then(|v| v.as_i64())
        .or_else(|| content.and_then(|c| c.get("Ets")).and_then(|v| v.as_i64()))
        .unwrap_or(0);
    let (time_utc, time_vn) = format_timestamp(raw_time);

    let mail_id = mail
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            mail.get("id")
                .and_then(|v| v.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default()
        });

    let mut items = Vec::new();
    let mut item_totals: HashMap<String, i64> = HashMap::new();
    let mut levels: Vec<u32> = Vec::new();
    let mut pos_x: u32 = 0;
    let mut pos_y: u32 = 0;

    if let Some(att_map) = attacks {
        for (_atk_id, atk) in att_map {
            let cidt = atk.get("CIdt");
            let is_fort = cidt
                .and_then(|c| c.get("IsFort"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let npcbtype = cidt
                .and_then(|c| c.get("NpcBType"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as i64;

            if !is_fort && npcbtype == 1 {
                if let Some(cidt) = cidt {
                    if let Some(hlv) = cidt.get("HLv").and_then(|v| v.as_u32()) {
                        if hlv > 0 && hlv <= 55 {
                            levels.push(hlv);
                        }
                    }
                }
            }

            let is_valid_target = !is_fort && (npcbtype == 1 || atk.get("NpcKillLoot").is_some());
            if is_valid_target || (pos_x == 0 && pos_y == 0) {
                if let Some(cidt) = cidt {
                    if let Some(cpos) = cidt.get("CastlePos") {
                        let x = cpos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let y = cpos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        if x > 0.0 && y > 0.0 {
                            pos_x = (x / 6.0).round() as u32;
                            pos_y = (y / 6.0).round() as u32;
                        }
                    }
                }

                if let Some(pos) = atk.get("Pos") {
                    let x = pos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = pos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if x > 0.0 && y > 0.0 {
                        pos_x = (x / 6.0).round() as u32;
                        pos_y = (y / 6.0).round() as u32;
                    }
                }
            }

            if let Some(loots_map) = atk.get("NpcKillLoot").and_then(|l| l.as_table()) {
                for (_loot_id, item_val) in loots_map {
                    let itype = item_val.get("Type").and_then(|v| v.as_u32()).unwrap_or(0);
                    let isub = item_val.get("SubType").and_then(|v| v.as_u32()).unwrap_or(0);
                    let val = item_val.get("Value").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pic = item_val
                        .get("PIC")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let info = catalog.get_item_info(
                        itype,
                        isub,
                        if pic.is_empty() { None } else { Some(&pic) },
                    );

                    *item_totals.entry(info.name_vi.clone()).or_insert(0) += val;

                    items.push(ItemDrop {
                        itype,
                        sub_type: isub,
                        value: val,
                        pic,
                        name_vi: info.name_vi,
                        name_en: info.name_en,
                        category: info.category,
                        sprites: info.sprites,
                    });
                }
            }
        }
    }

    let (motte_level, target) = format_barbarian_target(&levels);

    let content_desc = target.clone();
    let (account_id, player_name) = extract_account_info(mail);

    Ok(BattleRecord {
        filename: filename.to_string(),
        mail_id,
        battle_type: "barbarian".to_string(),
        time_raw: raw_time,
        time_utc,
        time_vn,
        target,
        target_level: motte_level,
        talent_add: 0,
        pos_x: pos_x as i32,
        pos_y: pos_y as i32,
        damage_pct: 100,
        plunder_tier: 0,
        items,
        item_totals,
        content: content_desc,
        account_id,
        player_name,
        ..Default::default()
    })
}

pub fn extract_gathering_details(
    mail: &LilithValue,
    filename: &str,
) -> Result<BattleRecord, String> {
    let raw_time = mail
        .get("time")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let (time_utc, time_vn) = format_timestamp(raw_time);

    let mail_id = mail
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            mail.get("id")
                .and_then(|v| v.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default()
        });

    let body = mail.get("body").ok_or_else(|| "Missing mail body".to_string())?;
    let content = body.get("content").and_then(|c| c.as_table()).ok_or_else(|| "Missing or invalid body content".to_string())?;

    let res_type = content.get("ResType").and_then(|v| v.as_u32()).unwrap_or(0);
    let level = content.get("Level").and_then(|v| v.as_u32()).unwrap_or(0);
    let res_value = content.get("ResValue").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let talent_add = content.get("talentAdd").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let (res_name, res_name_en, category) = match res_type {
        1 => ("Thực phẩm", "Food", "resource"),
        2 => ("Gỗ", "Wood", "resource"),
        3 => ("Đá", "Stone", "resource"),
        4 => ("Vàng", "Gold", "resource"),
        5 => ("Đá quý", "Gems", "gems"),
        _ => ("Tài nguyên khác", "Other Resources", "resource"),
    };

    let target = if (1..=5).contains(&res_type) {
        if level > 0 {
            format!("Mỏ {} Cấp {}", res_name, level)
        } else {
            format!("Mỏ {}", res_name)
        }
    } else if level > 0 {
        format!("Mỏ Thu Thập Cấp {}", level)
    } else {
        "Mỏ Thu Thập".to_string()
    };

    let base_val = res_value.round() as i64;
    let talent_val = talent_add.round() as i64;

    let mut items = Vec::new();
    let mut item_totals = HashMap::new();

    if base_val > 0 {
        let sprites = match res_type {
            1 => vec!["img_food.png".to_string()],
            2 => vec!["img_wood.png".to_string()],
            3 => vec!["img_stone.png".to_string()],
            4 => vec!["img_gold.png".to_string()],
            5 => vec!["img_gem.png".to_string()],
            9 => vec!["img_Crystal.png".to_string()],
            _ => Vec::new(),
        };
        items.push(ItemDrop {
            itype: 1,
            sub_type: res_type,
            value: base_val,
            pic: String::new(),
            name_vi: res_name.to_string(),
            name_en: res_name_en.to_string(),
            category: category.to_string(),
            sprites,
        });
        item_totals.insert(res_name.to_string(), base_val);
    }

    let (account_id, player_name) = extract_account_info(mail);

    Ok(BattleRecord {
        filename: filename.to_string(),
        mail_id,
        battle_type: "gathering".to_string(),
        time_raw: raw_time,
        time_utc,
        time_vn,
        target,
        target_level: if level > 0 { level.to_string() } else { String::new() },
        talent_add: talent_val,
        pos_x: 0,
        pos_y: 0,
        damage_pct: 0,
        plunder_tier: 0,
        items,
        item_totals,
        content: String::new(),
        account_id,
        player_name,
        ..Default::default()
    })
}

fn lilith_bool_or_truthy(val: Option<&LilithValue>) -> bool {
    match val {
        Some(LilithValue::Bool(b)) => *b,
        Some(LilithValue::Number(n)) => *n > 0.0,
        Some(LilithValue::String(s)) => s.eq_ignore_ascii_case("true") || s == "1",
        _ => false,
    }
}

pub fn extract_pvp_battle_details(
    mail: &LilithValue,
    filename: &str,
) -> Result<BattleRecord, String> {
    let body = mail.get("body").ok_or_else(|| "Missing mail body".to_string())?;
    let content = body.get("content").ok_or_else(|| "Missing body content".to_string())?;

    let raw_time = mail
        .get("time")
        .and_then(|v| v.as_i64())
        .or_else(|| content.get("Ets").and_then(|v| v.as_i64()))
        .unwrap_or(0);
    let (time_utc, time_vn) = format_timestamp(raw_time);

    let mail_id = mail
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            mail.get("id")
                .and_then(|v| v.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default()
        });

    let self_char = content.get("SelfChar");
    let mut primary_commander_id = self_char
        .and_then(|sc| sc.get("HId"))
        .and_then(|v| v.as_u32())
        .unwrap_or(0);
    let mut secondary_commander_id = self_char
        .and_then(|sc| sc.get("HId2"))
        .and_then(|v| v.as_u32())
        .unwrap_or(0);

    // Fallback to STs if SelfChar did not carry HId
    if primary_commander_id == 0 {
        if let Some(sts) = content.get("STs").and_then(|s| s.as_table()) {
            if let Some((_k, st)) = sts.iter().next() {
                primary_commander_id = st.get("HId").and_then(|v| v.as_u32()).unwrap_or(0);
                secondary_commander_id = st.get("HId2").and_then(|v| v.as_u32()).unwrap_or(0);
            }
        }
    }

    let primary_commander_name = get_commander_name(primary_commander_id);
    let secondary_commander_name = get_commander_name(secondary_commander_id);

    let mut primary_is_expertised = self_char
        .and_then(|sc| sc.get("HAw"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let mut secondary_is_expertised = self_char
        .and_then(|sc| sc.get("HAw2"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !primary_is_expertised {
        if let Some(hss) = self_char.and_then(|sc| sc.get("HSS")).and_then(|h| h.as_table()) {
            if hss.len() >= 5 || hss.contains_key("5") {
                primary_is_expertised = true;
            }
        }
    }
    if !secondary_is_expertised {
        if let Some(hss2) = self_char.and_then(|sc| sc.get("HSS2")).and_then(|h| h.as_table()) {
            if hss2.len() >= 5 || hss2.contains_key("5") {
                secondary_is_expertised = true;
            }
        }
    }

    // let sc = content.get("SelfChar");

    let mut is_garrison = lilith_bool_or_truthy(content.get("IsRally"));

    let mut kp_gained = 0.0;
    let mut kp_lost = 0.0;
    let mut dead_dealt: u64 = 0;
    let mut dead_taken: u64 = 0;
    let mut severely_wounded_dealt: u64 = 0;
    let mut severely_wounded_taken: u64 = 0;
    let mut pos_x: i32 = 0;
    let mut pos_y: i32 = 0;
    let mut enemy_names: Vec<String> = Vec::new();
    let mut target_primary_commander_id: u32 = 0;
    let mut target_secondary_commander_id: u32 = 0;
    let mut target_primary_is_expertised = false;
    let mut target_secondary_is_expertised = false;

    if let Some(attacks) = content.get("Attacks").and_then(|a| a.as_table()) {
        for (_atk_id, atk) in attacks {
            let cidt = atk.get("CIdt");
            if let Some(c) = cidt {
                if lilith_bool_or_truthy(c.get("IsRally")) {
                    is_garrison = true;
                }
            }

            let pid = cidt
                .and_then(|c| c.get("PId"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if pid <= 0.0 {
                continue;
            }

            if let Some(cidt) = cidt {
                if let Some(ename) = cidt.get("PName").and_then(|v| v.as_str()) {
                    let trimmed = ename.trim();
                    if !trimmed.is_empty() && !enemy_names.iter().any(|n| n == trimmed) {
                        enemy_names.push(trimmed.to_string());
                    }
                }
                if target_primary_commander_id == 0 {
                    target_primary_commander_id = cidt.get("HId").and_then(|v| v.as_u32()).unwrap_or(0);
                    target_secondary_commander_id = cidt.get("HId2").and_then(|v| v.as_u32()).unwrap_or(0);
                    target_primary_is_expertised = cidt.get("HAw").and_then(|v| v.as_bool()).unwrap_or(false);
                    target_secondary_is_expertised = cidt.get("HAw2").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !target_primary_is_expertised {
                        if let Some(hss) = cidt.get("HSS").and_then(|h| h.as_table()) {
                            if hss.len() >= 5 || hss.contains_key("5") {
                                target_primary_is_expertised = true;
                            }
                        }
                    }
                    if !target_secondary_is_expertised {
                        if let Some(hss2) = cidt.get("HSS2").and_then(|h| h.as_table()) {
                            if hss2.len() >= 5 || hss2.contains_key("5") {
                                target_secondary_is_expertised = true;
                            }
                        }
                    }
                }
            }

            if let Some(dmg) = atk.get("Damage") {
                kp_gained += dmg.get("KillScore").and_then(|v| v.as_f64()).unwrap_or(0.0);
                dead_taken += dmg.get("Death").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u64;
                severely_wounded_taken += dmg.get("BadHurt").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u64;
            }

            if let Some(kill) = atk.get("Kill") {
                kp_lost += kill.get("KillScore").and_then(|v| v.as_f64()).unwrap_or(0.0);
                dead_dealt += kill.get("Death").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u64;
                severely_wounded_dealt += kill.get("BadHurt").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0) as u64;
            }

            if pos_x == 0 && pos_y == 0 {
                if let Some(pos) = atk.get("Pos") {
                    let x = pos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = pos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if x > 0.0 && y > 0.0 {
                        pos_x = (x / 6.0).round() as i32;
                        pos_y = (y / 6.0).round() as i32;
                    }
                }
            }
        }
    }

    let is_self_rally = lilith_bool_or_truthy(content.get("SelfChar").and_then(|sc| sc.get("IsRally")));
    let pvp_combat_type = if is_self_rally {
    "rally".to_string()
    } else if is_garrison {
        "garrison".to_string()
    } else {
        "open-field".to_string()
    };

    let (account_id, player_name) = extract_account_info(mail);
    let target_primary_commander_name = get_commander_name(target_primary_commander_id);
    let target_secondary_commander_name = get_commander_name(target_secondary_commander_id);

    let target_pair_display = if !target_secondary_commander_name.is_empty() {
        format!("{}/{}", target_primary_commander_name, target_secondary_commander_name)
    } else if !target_primary_commander_name.is_empty() {
        target_primary_commander_name.clone()
    } else {
        String::new()
    };

    let target_player_name = if !enemy_names.is_empty() {
        if enemy_names.len() == 1 {
            enemy_names[0].clone()
        } else {
            format!("{} +{}", enemy_names[0], enemy_names.len() - 1)
        }
    } else {
        String::new()
    };

    let target = if !target_pair_display.is_empty() && !target_player_name.is_empty() {
        format!("[{}]-{}", target_pair_display, target_player_name)
    } else if !target_pair_display.is_empty() {
        format!("[{}]", target_pair_display)
    } else if !target_player_name.is_empty() {
        format!("vs {}", target_player_name)
    } else {
        "PvP".to_string()
    };

    Ok(BattleRecord {
        filename: filename.to_string(),
        mail_id,
        battle_type: "pvp".to_string(),
        time_raw: raw_time,
        time_utc,
        time_vn,
        target: target.clone(),
        target_level: pvp_combat_type.clone(),
        talent_add: 0,
        pos_x,
        pos_y,
        damage_pct: 100,
        plunder_tier: 0,
        items: Vec::new(),
        item_totals: HashMap::new(),
        content: target,
        account_id,
        player_name,
        pvp_combat_type,
        primary_commander_id,
        primary_commander_name,
        secondary_commander_id,
        secondary_commander_name,
        target_primary_commander_id,
        target_primary_commander_name,
        target_secondary_commander_id,
        target_secondary_commander_name,
        primary_is_expertised,
        secondary_is_expertised,
        target_primary_is_expertised,
        target_secondary_is_expertised,
        target_player_name,
        kp_gained,
        kp_lost,
        dead_dealt,
        dead_taken,
        severely_wounded_dealt,
        severely_wounded_taken,
    })
}

pub fn extract_motte_details(
    mail: &LilithValue,
    filename: &str,
    catalog: &LootCatalog,
    pre_classified: Option<&str>,
) -> Result<BattleRecord, String> {
    let btype = match pre_classified {
        Some(bt) => bt,
        None => classify_mail(mail).unwrap_or("other"),
    };

    if btype == "barbarian" || is_barbarian_battle_report(mail) {
        return extract_barbarian_battle_details(mail, filename, catalog);
    }

    if btype == "pvp" || is_pvp_battle_report(mail) {
        return extract_pvp_battle_details(mail, filename);
    }

    if btype == "gathering" {
        return extract_gathering_details(mail, filename);
    }

    let body = mail.get("body");
    let content_raw = body
        .and_then(|b| b.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = clean_text(content_raw);
    let mut found_lvl: Option<u32> = None;
    if let Some(caps) = LEVEL_REGEX.captures(&content) {
        if let Some(m) = caps.get(1).or_else(|| caps.get(2)) {
            found_lvl = m.as_str().parse().ok();
        }
    }

    let target_raw = body
        .and_then(|b| b.get("targetName"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if found_lvl.is_none() && !target_raw.is_empty() {
        if let Some(caps) = NUMBER_ONLY_REGEX.captures(target_raw) {
            if let Some(m) = caps.get(1) {
                found_lvl = m.as_str().parse().ok();
            }
        }
    }

    let motte_level_str = match found_lvl {
        Some(lvl) => lvl.to_string(),
        None => target_raw.to_string(),
    };

    let damage_pct = DMG_REGEX
        .captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    let plunder_tier = PLUNDER_REGEX
        .captures(&content)
        .and_then(|c| c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    let mut pos_x = 0;
    let mut pos_y = 0;
    if let Some(caps) = POS_REGEX.captures(&content) {
        if let (Some(x), Some(y)) = (caps.get(1), caps.get(2)) {
            pos_x = x.as_str().parse().unwrap_or(0);
            pos_y = y.as_str().parse().unwrap_or(0);
        }
    }

    let display_target = match btype {
        "motte" => {
            if !motte_level_str.is_empty() {
                format!("Motte {}", motte_level_str)
            } else {
                "Motte".to_string()
            }
        }
        "fort" => {
            if !motte_level_str.is_empty() {
                format!("Pháo Đài Cấp {}", motte_level_str)
            } else {
                "Pháo Đài Dã Man".to_string()
            }
        }
        "barbarian" => {
            if !motte_level_str.is_empty() {
                format!("Dã Man Cấp {}", motte_level_str)
            } else {
                body.and_then(|b| b.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Dã Man")
                    .to_string()
            }
        }
        _ => body
            .and_then(|b| b.get("title"))
            .and_then(|v| v.as_str())
            .or_else(|| mail.get("title").and_then(|v| v.as_str()))
            .unwrap_or("Phần thưởng")
            .to_string(),
    };

    let raw_time = mail.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
    let (time_utc, time_vn) = format_timestamp(raw_time);

    let mail_id = mail
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            mail.get("id")
                .and_then(|v| v.as_i64())
                .map(|i| i.to_string())
                .unwrap_or_default()
        });

    let mut items = Vec::new();
    let mut item_totals: HashMap<String, i64> = HashMap::new();

    if let Some(att_map) = mail.get("attachments").and_then(|a| a.as_table()) {
        for (_att_id, att_val) in att_map {
            if let Some(loots_map) = att_val.get("loot").and_then(|l| l.as_table()) {
                for (_loot_id, item_val) in loots_map {
                    let itype = item_val.get("Type").and_then(|v| v.as_u32()).unwrap_or(0);
                    let isub = item_val.get("SubType").and_then(|v| v.as_u32()).unwrap_or(0);
                    let val = item_val.get("Value").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pic = item_val
                        .get("PIC")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let info = catalog.get_item_info(itype, isub, Some(&pic));

                    *item_totals.entry(info.name_vi.clone()).or_insert(0) += val;

                    items.push(ItemDrop {
                        itype,
                        sub_type: isub,
                        value: val,
                        pic,
                        name_vi: info.name_vi,
                        name_en: info.name_en,
                        category: info.category,
                        sprites: info.sprites,
                    });
                }
            }
        }
    }

    let (account_id, player_name) = extract_account_info(mail);

    Ok(BattleRecord {
        filename: filename.to_string(),
        mail_id,
        battle_type: btype.to_string(),
        time_raw: raw_time,
        time_utc,
        time_vn,
        target: display_target,
        target_level: motte_level_str,
        talent_add: 0,
        pos_x,
        pos_y,
        damage_pct,
        plunder_tier,
        items,
        item_totals,
        content,
        account_id,
        player_name,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_clean_text() {
        assert_eq!(clean_text("Hello World"), "Hello World");
        assert_eq!(clean_text("<color=#FF0000>Thắng lợi</color>"), "Thắng lợi");
        assert_eq!(clean_text("   <b>X: 500 Y: 600</b>   "), "X: 500 Y: 600");
    }

    #[test]
    fn test_classify_motte_fast_path() {
        let mut body = IndexMap::new();
        body.insert("subType".to_string(), LilithValue::U8(11));
        body.insert("subParam".to_string(), LilithValue::U8(4));
        body.insert("title".to_string(), LilithValue::String("Tiêu diệt Motte".to_string()));

        let mut mail_map = IndexMap::new();
        mail_map.insert("box".to_string(), LilithValue::String("Report".to_string()));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mut att = IndexMap::new();
        att.insert("item1".to_string(), LilithValue::Null);
        mail_map.insert("attachments".to_string(), LilithValue::Table(att));

        let mail = LilithValue::Table(mail_map);
        assert_eq!(classify_mail(&mail), Some("motte"));
    }

    #[test]
    fn test_classify_fort_fast_path() {
        let mut body = IndexMap::new();
        body.insert("subType".to_string(), LilithValue::U8(11));
        body.insert("subParam".to_string(), LilithValue::U8(1));
        body.insert("title".to_string(), LilithValue::String("Phá hủy Pháo Đài".to_string()));

        let mut mail_map = IndexMap::new();
        mail_map.insert("box".to_string(), LilithValue::String("Report".to_string()));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mut att = IndexMap::new();
        att.insert("item1".to_string(), LilithValue::Null);
        mail_map.insert("attachments".to_string(), LilithValue::Table(att));

        let mail = LilithValue::Table(mail_map);
        assert_eq!(classify_mail(&mail), Some("fort"));
    }

    #[test]
    fn test_classify_reject_non_report_box() {
        let mut body = IndexMap::new();
        body.insert("subType".to_string(), LilithValue::U8(11));
        body.insert("subParam".to_string(), LilithValue::U8(4));
        body.insert("title".to_string(), LilithValue::String("Tiêu diệt Motte".to_string()));

        let mut mail_map = IndexMap::new();
        mail_map.insert("box".to_string(), LilithValue::String("System".to_string()));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mut att = IndexMap::new();
        att.insert("item1".to_string(), LilithValue::Null);
        mail_map.insert("attachments".to_string(), LilithValue::Table(att));

        let mail = LilithValue::Table(mail_map);
        assert_eq!(classify_mail(&mail), None);
    }

    #[test]
    fn test_classify_canceled_rally() {
        let mut body = IndexMap::new();
        body.insert("subType".to_string(), LilithValue::U8(11));
        body.insert("subParam".to_string(), LilithValue::U8(4));
        body.insert("title".to_string(), LilithValue::String("Tập kích đã bị hủy".to_string()));

        let mut mail_map = IndexMap::new();
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mail = LilithValue::Table(mail_map);
        assert_eq!(classify_mail(&mail), None);
    }

    #[test]
    fn test_classify_and_extract_persistent_mail_95837234179264017425() {
        use crate::lilith_stream::decode_mail_file;
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.95837234179264017425");
        let mail = decode_mail_file(&path).expect("Target barbarian mail should decode");

        let btype = classify_mail(&mail);
        assert_eq!(btype, Some("barbarian"), "Persistent.Mail.95837234179264017425 must be classified as barbarian");

        let catalog = LootCatalog::load_from_candidates(None);
        let rec = extract_motte_details(&mail, "Persistent.Mail.95837234179264017425", &catalog, btype).expect("Should extract details");

        assert_eq!(rec.battle_type, "barbarian");
        assert_eq!(rec.target, "Dã Man Cấp 50");
        assert_eq!(rec.target_level, "50");
        assert_eq!(rec.pos_x, 781);
        assert_eq!(rec.pos_y, 525);
        assert_eq!(rec.damage_pct, 100);
        assert_eq!(rec.plunder_tier, 0);
        assert_eq!(rec.items.len(), 4, "Must extract 4 loot items");
        assert!(rec.item_totals.contains_key("Pha Lê"), "Should contain Pha Lê");
        assert_eq!(rec.item_totals.get("Pha Lê"), Some(&688));
        assert!(rec.item_totals.contains_key("Sách Kiến Thức Cấp 3"), "Should contain Sách EXP");
        assert_eq!(rec.item_totals.get("Sách Kiến Thức Cấp 3"), Some(&10));
    }

    #[test]
    fn test_pass_first_occupation_not_classified_as_barbarian() {
        use crate::lilith_stream::decode_mail_file;
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.57774831178214367624");
        if let Ok(mail) = decode_mail_file(&path) {
            let btype = classify_mail(&mail);
            assert_ne!(btype, Some("barbarian"), "Pass First Occupation must NOT be classified as barbarian");
            assert_eq!(btype, Some("reward_other"));
        }
    }

    #[test]
    fn test_chained_barbarian_battle_extraction() {
        use crate::lilith_stream::decode_mail_file;
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.79355196177398328125");
        if let Ok(mail) = decode_mail_file(&path) {
            let box_name = mail.get("box").and_then(|v| v.as_str());
            let mail_type = mail.get("type").and_then(|v| v.as_str());
            println!("BOX: {:?}, TYPE: {:?}", box_name, mail_type);
            let is_bb = is_barbarian_battle_report(&mail);
            println!("is_barbarian_battle_report: {}", is_bb);
            let btype = classify_mail(&mail);
            assert_eq!(btype, Some("barbarian"));

            let catalog = LootCatalog::load_from_candidates(None);
            let rec = extract_motte_details(&mail, "Persistent.Mail.79355196177398328125", &catalog, btype).expect("Should extract chained details");
            assert_eq!(rec.target, "Dã Man Cấp 47-52 (x27)");
            assert_eq!(rec.target_level, "47-52");
            assert!(!rec.items.is_empty());
        }
    }

    #[test]
    fn test_mixed_barbarian_and_fort_extraction() {
        use crate::lilith_stream::decode_mail_file;
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.95953682179264410425");
        if let Ok(mail) = decode_mail_file(&path) {
            let btype = classify_mail(&mail);
            assert_eq!(btype, Some("barbarian"));

            let catalog = LootCatalog::load_from_candidates(None);
            let rec = extract_motte_details(&mail, "Persistent.Mail.95953682179264410425", &catalog, btype)
                .expect("Should extract mixed fort barbarian battle");
            assert_eq!(rec.target, "Dã Man Cấp 52");
            assert_eq!(rec.target_level, "52");
            assert_eq!(rec.pos_x, 772);
            assert_eq!(rec.pos_y, 531);
            assert_eq!(rec.item_totals.get("Pha Lê"), Some(&688));
            assert_eq!(rec.item_totals.get("Sách Kiến Thức Cấp 3"), Some(&10));
        }
    }

    #[test]
    fn test_extract_motte_details_delegates_to_gathering() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.1002220217856803764");
        let mail = crate::lilith_stream::decode_mail_file(&path).expect("Decode mail");
        let catalog = LootCatalog::load_from_candidates(None);
        let rec = extract_motte_details(&mail, "Persistent.Mail.1002220217856803764", &catalog, None)
            .expect("Should extract gathering via extract_motte_details");
        assert_eq!(rec.battle_type, "gathering");
        assert_eq!(rec.target, "Mỏ Đá Cấp 6");
    }

    #[test]
    fn test_classify_and_extract_gathering_mail() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.1002220217856803764");
        let mail = crate::lilith_stream::decode_mail_file(&path).expect("Must decode sample RSS mail");
        assert_eq!(classify_mail(&mail), Some("gathering"));

        let rec = extract_gathering_details(&mail, "Persistent.Mail.1002220217856803764")
            .expect("Must extract gathering details");
        assert_eq!(rec.battle_type, "gathering");
        assert_eq!(rec.target, "Mỏ Đá Cấp 6");
        assert_eq!(rec.target_level, "6");
        assert_eq!(rec.pos_x, 0);
        assert_eq!(rec.pos_y, 0);
        assert_eq!(rec.talent_add, 4071);
        assert_eq!(rec.items.len(), 1);
        assert_eq!(rec.items[0].itype, 1);
        assert_eq!(rec.items[0].sub_type, 3);
        assert_eq!(rec.items[0].name_vi, "Đá");
        assert_eq!(rec.items[0].name_en, "Stone");
        assert_eq!(rec.items[0].value, 23462);
        assert_eq!(rec.item_totals.get("Đá"), Some(&23462));
    }

    #[test]
    fn test_extract_gathering_unknown_res_type_fallback() {
        use indexmap::IndexMap;
        let mut content = IndexMap::new();
        content.insert("ResType".to_string(), LilithValue::U8(99));
        content.insert("Level".to_string(), LilithValue::U8(4));
        content.insert("ResValue".to_string(), LilithValue::Number(50000.0));
        content.insert("talentAdd".to_string(), LilithValue::Number(500.0));

        let mut body = IndexMap::new();
        body.insert("content".to_string(), LilithValue::Table(content));

        let mut mail_map = IndexMap::new();
        mail_map.insert("box".to_string(), LilithValue::String("Report".to_string()));
        mail_map.insert("type".to_string(), LilithValue::String("Rss".to_string()));
        mail_map.insert("time".to_string(), LilithValue::Number(1700000000.0));
        mail_map.insert("id".to_string(), LilithValue::String("999".to_string()));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mail = LilithValue::Table(mail_map);
        let rec = extract_gathering_details(&mail, "mock.mail").expect("Should extract with fallback");
        assert_eq!(rec.target, "Mỏ Thu Thập Cấp 4");
        assert_eq!(rec.talent_add, 500);
        assert_eq!(rec.items[0].name_vi, "Tài nguyên khác");
        assert_eq!(rec.items[0].name_en, "Other Resources");
        assert_eq!(rec.items[0].value, 50000);
    }

    #[test]
    fn test_extract_account_info() {
        use indexmap::IndexMap;
        let mut self_char = IndexMap::new();
        self_char.insert("PName".to_string(), LilithValue::String("GovernorOne".to_string()));
        self_char.insert("PId".to_string(), LilithValue::Number(12345678.0));

        let mut content = IndexMap::new();
        content.insert("SelfChar".to_string(), LilithValue::Table(self_char));

        let mut body = IndexMap::new();
        body.insert("content".to_string(), LilithValue::Table(content));

        let mut mail_map = IndexMap::new();
        mail_map.insert("receiver".to_string(), LilithValue::String("player_12345678".to_string()));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mail = LilithValue::Table(mail_map);
        let (acc_id, p_name) = extract_account_info(&mail);
        assert_eq!(acc_id, "12345678");
        assert_eq!(p_name, "GovernorOne");
    }

    #[test]
    fn test_classify_and_extract_pvp_battle_report() {
        use indexmap::IndexMap;
        let mut self_char = IndexMap::new();
        self_char.insert("PName".to_string(), LilithValue::String("GovernorOne".to_string()));
        self_char.insert("PId".to_string(), LilithValue::Number(139825469.0));
        self_char.insert("HId".to_string(), LilithValue::Number(198.0));
        self_char.insert("HId2".to_string(), LilithValue::Number(140.0));

        let mut cidt = IndexMap::new();
        cidt.insert("PId".to_string(), LilithValue::Number(5752412.0));
        cidt.insert("PName".to_string(), LilithValue::String("EnemyGov".to_string()));
        cidt.insert("HId".to_string(), LilithValue::Number(179.0));
        cidt.insert("HId2".to_string(), LilithValue::Number(6.0));

        let mut dmg = IndexMap::new();
        dmg.insert("KillScore".to_string(), LilithValue::Number(250_000.0));
        dmg.insert("Death".to_string(), LilithValue::Number(500.0));
        dmg.insert("BadHurt".to_string(), LilithValue::Number(12000.0));

        let mut kill = IndexMap::new();
        kill.insert("KillScore".to_string(), LilithValue::Number(100_000.0));
        kill.insert("Death".to_string(), LilithValue::Number(100.0));
        kill.insert("BadHurt".to_string(), LilithValue::Number(4900.0));

        let mut pos = IndexMap::new();
        pos.insert("X".to_string(), LilithValue::Number(3600.0));
        pos.insert("Y".to_string(), LilithValue::Number(3000.0));

        let mut atk = IndexMap::new();
        atk.insert("CIdt".to_string(), LilithValue::Table(cidt));
        atk.insert("Damage".to_string(), LilithValue::Table(dmg));
        atk.insert("Kill".to_string(), LilithValue::Table(kill));
        atk.insert("Pos".to_string(), LilithValue::Table(pos));

        let mut attacks = IndexMap::new();
        attacks.insert("1".to_string(), LilithValue::Table(atk));

        let mut content = IndexMap::new();
        content.insert("SelfChar".to_string(), LilithValue::Table(self_char));
        content.insert("Attacks".to_string(), LilithValue::Table(attacks));

        let mut body = IndexMap::new();
        body.insert("content".to_string(), LilithValue::Table(content));

        let mut mail_map = IndexMap::new();
        mail_map.insert("box".to_string(), LilithValue::String("Report".to_string()));
        mail_map.insert("type".to_string(), LilithValue::String("Battle".to_string()));
        mail_map.insert("id".to_string(), LilithValue::String("pvp_1001".to_string()));
        mail_map.insert("time".to_string(), LilithValue::Number(1700000000000000.0));
        mail_map.insert("body".to_string(), LilithValue::Table(body));

        let mail = LilithValue::Table(mail_map);
        assert_eq!(classify_mail(&mail), Some("pvp"));

        let catalog = LootCatalog::empty();
        let rec = extract_motte_details(&mail, "Persistent.Mail.pvp_1001", &catalog, None)
            .expect("Must extract PvP battle record");

        assert_eq!(rec.battle_type, "pvp");
        assert_eq!(rec.pvp_combat_type, "open-field");
        assert_eq!(rec.primary_commander_id, 198);
        assert_eq!(rec.primary_commander_name, "Ragnar Lodbrok (P)");
        assert_eq!(rec.secondary_commander_id, 140);
        assert_eq!(rec.secondary_commander_name, "Scipio Africanus (P)");
        assert_eq!(rec.target_primary_commander_id, 179);
        assert_eq!(rec.target_primary_commander_name, "Zhuge Liang");
        assert_eq!(rec.target_secondary_commander_id, 6);
        assert_eq!(rec.target_secondary_commander_name, "Yi Seong-Gye");
        assert_eq!(rec.kp_gained, 250_000.0);
        assert_eq!(rec.kp_lost, 100_000.0);
        assert_eq!(rec.dead_dealt, 100);
        assert_eq!(rec.dead_taken, 500);
        assert_eq!(rec.severely_wounded_dealt, 4900);
        assert_eq!(rec.severely_wounded_taken, 12000);
        assert_eq!(rec.pos_x, 600);
        assert_eq!(rec.pos_y, 500);
        assert_eq!(rec.target, "[Zhuge Liang/Yi Seong-Gye]-EnemyGov");
        assert_eq!(rec.target_player_name, "EnemyGov");
    }

    #[test]
    fn test_classify_and_extract_pvp_rally_and_garrison() {
        use indexmap::IndexMap;

        // 1. Self Rally report via IsRally: true inside SelfChar
        let mut self_char_rally = IndexMap::new();
        self_char_rally.insert("PName".to_string(), LilithValue::String("RallyLeader".to_string()));
        self_char_rally.insert("IsRally".to_string(), LilithValue::Bool(true));
        self_char_rally.insert("HId".to_string(), LilithValue::Number(186.0));

        let mut content_rally = IndexMap::new();
        content_rally.insert("SelfChar".to_string(), LilithValue::Table(self_char_rally));
        content_rally.insert("Attacks".to_string(), LilithValue::Table(IndexMap::new()));

        let mut body_rally = IndexMap::new();
        body_rally.insert("content".to_string(), LilithValue::Table(content_rally));

        let mut mail_rally = IndexMap::new();
        mail_rally.insert("type".to_string(), LilithValue::String("Battle".to_string()));
        mail_rally.insert("id".to_string(), LilithValue::String("rally_01".to_string()));
        mail_rally.insert("body".to_string(), LilithValue::Table(body_rally));

        let rec_rally = extract_pvp_battle_details(&LilithValue::Table(mail_rally), "Persistent.Mail.rally_01")
            .expect("Must parse rally report");
        assert_eq!(rec_rally.pvp_combat_type, "rally");

        // 2. Garrison report via IsRally outside SelfChar
        let mut self_char_def = IndexMap::new();
        self_char_def.insert("PName".to_string(), LilithValue::String("Defender".to_string()));
        self_char_def.insert("HId".to_string(), LilithValue::Number(3.0));

        let mut enemy_cidt = IndexMap::new();
        enemy_cidt.insert("PName".to_string(), LilithValue::String("EnemyRallyLeader".to_string()));
        enemy_cidt.insert("PId".to_string(), LilithValue::Number(99999.0));

        let mut atk_def = IndexMap::new();
        atk_def.insert("CIdt".to_string(), LilithValue::Table(enemy_cidt));

        let mut attacks_def = IndexMap::new();
        attacks_def.insert("1".to_string(), LilithValue::Table(atk_def));

        let mut content_def = IndexMap::new();
        content_def.insert("SelfChar".to_string(), LilithValue::Table(self_char_def));
        content_def.insert("IsRally".to_string(), LilithValue::Bool(true));
        content_def.insert("Attacks".to_string(), LilithValue::Table(attacks_def));

        let mut body_def = IndexMap::new();
        body_def.insert("content".to_string(), LilithValue::Table(content_def));

        let mut mail_def = IndexMap::new();
        mail_def.insert("type".to_string(), LilithValue::String("Battle".to_string()));
        mail_def.insert("id".to_string(), LilithValue::String("def_01".to_string()));
        mail_def.insert("body".to_string(), LilithValue::Table(body_def));

        let rec_def = extract_pvp_battle_details(&LilithValue::Table(mail_def), "Persistent.Mail.def_01")
            .expect("Must parse garrison defense report");
        assert_eq!(rec_def.pvp_combat_type, "garrison");

        // 3. Open-field report: no IsRally anywhere
        let mut self_char_field = IndexMap::new();
        self_char_field.insert("PName".to_string(), LilithValue::String("FieldPlayer".to_string()));
        self_char_field.insert("HId".to_string(), LilithValue::Number(1.0));

        let mut enemy_field = IndexMap::new();
        enemy_field.insert("PName".to_string(), LilithValue::String("EnemyField".to_string()));
        enemy_field.insert("PId".to_string(), LilithValue::Number(88888.0));

        let mut atk_field = IndexMap::new();
        atk_field.insert("CIdt".to_string(), LilithValue::Table(enemy_field));

        let mut attacks_field = IndexMap::new();
        attacks_field.insert("1".to_string(), LilithValue::Table(atk_field));

        let mut content_field = IndexMap::new();
        content_field.insert("SelfChar".to_string(), LilithValue::Table(self_char_field));
        content_field.insert("Attacks".to_string(), LilithValue::Table(attacks_field));

        let mut body_field = IndexMap::new();
        body_field.insert("content".to_string(), LilithValue::Table(content_field));

        let mut mail_field = IndexMap::new();
        mail_field.insert("type".to_string(), LilithValue::String("Battle".to_string()));
        mail_field.insert("id".to_string(), LilithValue::String("field_01".to_string()));
        mail_field.insert("body".to_string(), LilithValue::Table(body_field));

        let rec_field = extract_pvp_battle_details(&LilithValue::Table(mail_field), "Persistent.Mail.field_01")
            .expect("Must parse field report");
        assert_eq!(rec_field.pvp_combat_type, "open-field");
    }

    #[test]
    fn test_commanders_yaml_lookup() {
        assert_eq!(get_commander_name(0), "");
        assert_eq!(get_commander_name(1), "Julius Caesar");
        assert_eq!(get_commander_name(3), "Sun Tzu");
        assert_eq!(get_commander_name(37), "Lohar");
        assert_eq!(get_commander_name(99), "Æthelflæd");
        assert_eq!(get_commander_name(140), "Scipio Africanus (P)");
        assert_eq!(get_commander_name(148), "Joan of Arc (P)");
        assert_eq!(get_commander_name(185), "Liu Che");
        assert_eq!(get_commander_name(186), "Ashurbanipal");
        assert_eq!(get_commander_name(198), "Ragnar Lodbrok (P)");
        assert_eq!(get_commander_name(506), "Shapur I");
        assert_eq!(get_commander_name(509), "Qin Shi Huang");
        assert_eq!(get_commander_name(540), "Arthur Pendragon");
        assert_eq!(get_commander_name(545), "Bai Qi");
        assert_eq!(get_commander_name(575), "Achilles");
        assert_eq!(get_commander_name(595), "Sun Tzu (P)");
        assert_eq!(get_commander_name(618), "Pelagius (P)");
    }
}
