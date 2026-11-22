use std::fs;
use std::path::Path;
use serde_json::json;

use crate::model::{AppLanguage, BattleRecord, Statistics};

fn escape_csv_field(val: &str) -> String {
    val.replace('"', "\"\"")
}

pub fn generate_csv(records: &[BattleRecord], lang: AppLanguage) -> String {
    let header = match lang {
        AppLanguage::Vietnamese => "\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút),Trọng thương,Chiến lực mất\n",
        AppLanguage::English => "\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min),Severely Wounded,Power Lost\n",
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
            "\"{}\",\"{}\",\"{}\",{},{},{},{},{},{},{},{},{},{},{}\n",
            safe_time, safe_btype, safe_target, r.damage_pct, r.plunder_tier,
            cry, rss, exp, arrow,
            r.univ_speedup_minutes(), r.training_speedup_minutes(), r.heal_speedup_minutes(),
            r.severely_wounded_taken, r.power_lost
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
            ..Default::default()
        };

        let csv_vi = generate_csv(std::slice::from_ref(&rec), AppLanguage::Vietnamese);
        assert!(csv_vi.starts_with("\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút),Trọng thương,Chiến lực mất\n"));
        assert!(csv_vi.contains("\"2026-10-23 12:00:00\",\"Pháo Đài\",\"Pháo Đài Cấp 10\",100,1,0,0,0,0,0,0,0,0,0\n"));

        let csv_en = generate_csv(&[rec], AppLanguage::English);
        assert!(csv_en.starts_with("\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min),Severely Wounded,Power Lost\n"));
        assert!(csv_en.contains("\"2026-10-23 12:00:00\",\"Barbarian Fort\",\"Barbarian Fort Level 10\",100,1,0,0,0,0,0,0,0,0,0\n"));
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
            ..Default::default()
        };

        let csv = generate_csv(&[rec], AppLanguage::English);
        assert!(csv.contains("\"Target \"\"Special\"\" 25\""));
    }
}
