use std::fs;
use std::path::Path;
use serde_json::json;

use crate::lilith_stream::decode_mail_file;
use crate::model::{AppLanguage, BattleRecord, Statistics};

fn escape_csv_field(val: &str) -> String {
    val.replace('"', "\"\"")
}

pub fn generate_csv(records: &[BattleRecord], lang: AppLanguage) -> String {
    let header = match lang {
        AppLanguage::Vietnamese => "\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút)\n",
        AppLanguage::English => "\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min)\n",
    };

    let mut w = String::from(header);
    for r in records {
        let btype_label = match (r.battle_type.as_str(), lang) {
            ("fort", AppLanguage::Vietnamese) => "Pháo Đài",
            ("fort", AppLanguage::English) => "Barbarian Fort",
            ("barbarian", AppLanguage::Vietnamese) => "Dã Man",
            ("barbarian", AppLanguage::English) => "Barbarian",
            ("motte", _) => "Motte",
            ("gathering", AppLanguage::Vietnamese) => "Thu Thập",
            ("gathering", AppLanguage::English) => "Gathering",
            (other, _) => other,
        };

        let target_display = r.display_target(lang);
        let cry = r.item_quantity(1, 9);
        let rss = r.total_resource_value();
        let exp = r.total_exp_value();
        let arrow = r.item_quantity(2, 145);

        let safe_time = escape_csv_field(&r.time_vn);
        let safe_btype = escape_csv_field(btype_label);
        let safe_target = escape_csv_field(&target_display);

        w.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},{},{},{},{},{},{},{},{}\n",
            safe_time, safe_btype, safe_target, r.damage_pct, r.plunder_tier,
            cry, rss, exp, arrow,
            r.univ_speedup_minutes(), r.training_speedup_minutes(), r.heal_speedup_minutes()
        ));
    }
    w
}

pub fn export_json(
    records: &[BattleRecord],
    stats: &Statistics,
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = out_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let payload = json!({
        "statistics": stats,
        "battles": records
    });

    let s = serde_json::to_string_pretty(&payload)?;
    let tmp_path = out_path.with_extension("tmp");
    fs::write(&tmp_path, s)?;
    fs::rename(&tmp_path, out_path)?;
    println!("[+] Đã xuất file JSON: {}", out_path.display());
    Ok(())
}

pub fn export_individual_mails_json(
    records: &[BattleRecord],
    mail_dir: &Path,
    out_dir: &Path,
) {
    let mails_json_dir = out_dir.join("filtered_mails_json");
    if let Err(e) = fs::create_dir_all(&mails_json_dir) {
        eprintln!("[-] Không thể tạo thư mục {}: {}", mails_json_dir.display(), e);
        return;
    }

    let binary_vault_dir = out_dir.join("mail_vault");

    for r in records {
        let out_file = mails_json_dir.join(format!("{}.json", r.filename));
        if out_file.is_file() {
            continue;
        }

        let mut src_path = mail_dir.join(&r.filename);
        if !src_path.is_file() {
            src_path = binary_vault_dir.join(&r.filename);
        }

        if src_path.is_file() {
            if let Ok(mut m_full) = decode_mail_file(&src_path) {
                let box_name = m_full.get("box").and_then(|v| v.as_str()).unwrap_or("");
                if box_name != "Report" && box_name != "Archive" {
                    continue;
                }

                if let crate::lilith_stream::LilithValue::Table(ref mut map) = m_full {
                    let mut meta = indexmap::IndexMap::new();
                    meta.insert("damage_pct".into(), crate::lilith_stream::LilithValue::Number(r.damage_pct as f64));
                    meta.insert("plunder_tier".into(), crate::lilith_stream::LilithValue::Number(r.plunder_tier as f64));
                    meta.insert("time_vn".into(), crate::lilith_stream::LilithValue::String(r.time_vn.clone()));
                    map.insert("_stats_metadata".into(), crate::lilith_stream::LilithValue::Table(meta));

                    if let Ok(s) = serde_json::to_string_pretty(&m_full) {
                        let _ = fs::write(&out_file, s);
                    }
                }
            }
        } else {
            if let Ok(s) = serde_json::to_string_pretty(r) {
                let _ = fs::write(&out_file, s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AppLanguage, BattleRecord};
    use std::collections::HashMap;

    #[test]
    fn test_generate_csv_vietnamese_and_english() {
        let rec = BattleRecord {
            filename: "mail1".into(),
            mail_id: "100".into(),
            battle_type: "fort".into(),
            time_raw: 12345,
            time_utc: "".into(),
            time_vn: "2026-10-23 12:00:00".into(),
            target: "Pháo Đài Cấp 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 1,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let csv_vi = generate_csv(std::slice::from_ref(&rec), AppLanguage::Vietnamese);
        assert!(csv_vi.starts_with("\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút)\n"));
        assert!(csv_vi.contains("\"2026-10-23 12:00:00\",\"Pháo Đài\",\"Pháo Đài Cấp 10\",100,1,0,0,0,0,0,0,0\n"));

        let csv_en = generate_csv(&[rec], AppLanguage::English);
        assert!(csv_en.starts_with("\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min)\n"));
        assert!(csv_en.contains("\"2026-10-23 12:00:00\",\"Barbarian Fort\",\"Barbarian Fort Level 10\",100,1,0,0,0,0,0,0,0\n"));
    }

    #[test]
    fn test_generate_csv_escapes_internal_quotes() {
        let rec = BattleRecord {
            filename: "mail_quote".into(),
            mail_id: "101".into(),
            battle_type: "other".into(),
            time_raw: 12345,
            time_utc: "".into(),
            time_vn: "2026-10-23 12:00:00".into(),
            target: "Target \"Special\" 25".into(),
            target_level: "".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let csv = generate_csv(&[rec], AppLanguage::English);
        assert!(csv.contains("\"Target \"\"Special\"\" 25\""));
    }
}
