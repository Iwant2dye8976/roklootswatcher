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

    // In Rise of Kingdoms mail caches, timestamps > 100_000_000_000 (10^11) represent
    // microsecond precision timestamps, so convert them to seconds by dividing by 1_000_000.
    // Standard unix epoch timestamps in seconds for modern dates are ~1.7 * 10^9.
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

pub fn classify_mail(mail: &LilithValue) -> Option<&'static str> {
    // Only classify and process mails from "Report" or "Archive" boxes
    let box_name = mail.get("box").and_then(|v| v.as_str()).unwrap_or("");
    if box_name != "Report" && box_name != "Archive" {
        return None;
    }

    // 1. Detect Barbarian Battle Reports (type: "Battle")
    if is_barbarian_battle_report(mail) {
        return Some("barbarian");
    }

    // 2. Detect Gathering Reports (type: "Rss")
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

    // Quick check for canceled rally without regex or lowercase allocations
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

    // Fast Path 1: Motte Destroyed (subType 11, subParam 4)
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

    // Fast Path 2: Barbarian Fort Destroyed (subType 11, subParam 1)
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

    // Fallback Path: only allocate and clean text if fast paths didn't match
    let tgt = body
        .get("targetName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content = clean_text(content_raw);
    let all_txt = format!("{} {} {} {}", title, sub, content, tgt).to_lowercase();

    // Exclude canceled rallies
    if all_txt.contains("canceled") || all_txt.contains("đã bị hủy") {
        return None;
    }

    // Motte Destroyed fallback
    if all_txt.contains("motte")
        && (has_attachments
            || all_txt.contains("destroyed")
            || all_txt.contains("tiêu diệt"))
    {
        return Some("motte");
    }

    // Barbarian Fort Destroyed fallback
    if (all_txt.contains("barbarian fort")
        || (all_txt.contains("pháo đài")
            && (all_txt.contains("dã man") || all_txt.contains("barbarian") || all_txt.contains("barbaria"))))
        && (has_attachments
            || all_txt.contains("destroyed")
            || all_txt.contains("phá hủy"))
    {
        return Some("fort");
    }

    // Any other report with attachments (e.g. Pass First Occupation, Holy Sites, Events)
    if has_attachments {
        return Some("reward_other");
    }

    None
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

            // In Rise of Kingdoms, barbarians never exceed level 55 and are never forts (npcbtype == 1).
            // Barbarian forts (is_fort = true, npcbtype = 2, level 40-60) must NOT be counted as barbarians.
            if !is_fort && npcbtype == 1 {
                if let Some(cidt) = cidt {
                    if let Some(hlv) = cidt.get("HLv").and_then(|v| v.as_u32()) {
                        if hlv > 0 && hlv <= 55 {
                            levels.push(hlv);
                        }
                    }
                }
            }

            // Coordinates from valid barbarian target or loot attack
            let is_valid_target = !is_fort && (npcbtype == 1 || atk.get("NpcKillLoot").is_some());
            if is_valid_target || (pos_x == 0 && pos_y == 0) {
                // Coordinates fallback from CastlePos
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

                // Coordinates from Pos (tile coords = pos / 6)
                if let Some(pos) = atk.get("Pos") {
                    let x = pos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = pos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if x > 0.0 && y > 0.0 {
                        pos_x = (x / 6.0).round() as u32;
                        pos_y = (y / 6.0).round() as u32;
                    }
                }
            }

            // Loot
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

    // Target label and level string
    let (motte_level, target) = if !levels.is_empty() {
        let min_lvl = *levels.iter().min().unwrap();
        let max_lvl = *levels.iter().max().unwrap();
        let count = levels.len();

        if count == 1 {
            (min_lvl.to_string(), format!("Dã Man Cấp {}", min_lvl))
        } else if min_lvl == max_lvl {
            (min_lvl.to_string(), format!("Dã Man Cấp {} (x{})", min_lvl, count))
        } else {
            (
                format!("{}-{}", min_lvl, max_lvl),
                format!("Dã Man Cấp {}-{} (x{})", min_lvl, max_lvl, count),
            )
        }
    } else {
        ("".to_string(), "Dã Man".to_string())
    };

    let content_desc = target.clone();

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

    let (res_name, category) = match res_type {
        1 => ("Thực phẩm", "resource"),
        2 => ("Gỗ", "resource"),
        3 => ("Đá", "resource"),
        4 => ("Vàng", "resource"),
        5 => ("Đá quý", "gems"),
        _ => ("Tài nguyên khác", "resource"),
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
        items.push(ItemDrop {
            itype: 1,
            sub_type: res_type,
            value: base_val,
            pic: String::new(),
            name_vi: res_name.to_string(),
            name_en: res_name.to_string(),
            category: category.to_string(),
            sprites: Vec::new(),
        });
        item_totals.insert(res_name.to_string(), base_val);
    }

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

    // Damage %
    let damage_pct = DMG_REGEX
        .captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    // Plunder tier
    let plunder_tier = PLUNDER_REGEX
        .captures(&content)
        .and_then(|c| c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    // Position
    let mut pos_x = 0;
    let mut pos_y = 0;
    if let Some(caps) = POS_REGEX.captures(&content) {
        if let (Some(x), Some(y)) = (caps.get(1), caps.get(2)) {
            pos_x = x.as_str().parse().unwrap_or(0);
            pos_y = y.as_str().parse().unwrap_or(0);
        }
    }

    // Display target label
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

    // Loot extraction
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
            // Must NOT contain Fort Level 60! Barbarian is strictly level 52.
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
        assert_eq!(rec.items[0].value, 50000);
    }
}
