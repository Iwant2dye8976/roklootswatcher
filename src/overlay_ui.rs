use eframe::egui::Color32;
use crate::model::{BattleRecord, FilterMode};

#[derive(Debug, Clone)]
pub struct CardDisplayData {
    pub tag: String,
    pub target_display: String,
    pub time_display: String,
    pub meta_display: String,
    pub drops_summary: Vec<String>,
    pub accent_color: Color32,
}

pub fn accent_color_for_battle_type(btype: &str) -> Color32 {
    match btype {
        "fort" => Color32::from_rgb(245, 158, 11),       // Gold / Amber
        "motte" => Color32::from_rgb(239, 68, 68),       // Crimson
        "barbarian" => Color32::from_rgb(59, 130, 246),  // Steel Blue
        "gathering" => Color32::from_rgb(16, 185, 129),  // Emerald
        _ => Color32::from_rgb(148, 163, 184),           // Slate
    }
}

pub fn format_card_data(rec: &BattleRecord) -> CardDisplayData {
    let tag = match rec.battle_type.as_str() {
        "fort" => "[Fort]".to_string(),
        "motte" => "[Motte]".to_string(),
        "barbarian" => "[Barbarian]".to_string(),
        "gathering" => "[Gathering]".to_string(),
        _ => "[Reward]".to_string(),
    };

    let accent_color = accent_color_for_battle_type(&rec.battle_type);

    let target_display = if !rec.target_level.is_empty() && !rec.target.contains(&rec.target_level) {
        format!("{} Lvl {}", rec.target, rec.target_level)
    } else {
        rec.target.clone()
    };

    let time_display = if !rec.time_vn.is_empty() {
        rec.time_vn.clone()
    } else if !rec.time_utc.is_empty() {
        rec.time_utc.clone()
    } else {
        format!("{}", rec.time_raw)
    };

    let meta_display = if rec.battle_type == "gathering" {
        format!("Coord: ({}, {}) | Complete", rec.pos_x, rec.pos_y)
    } else {
        format!("Coord: ({}, {}) | {}% Damage", rec.pos_x, rec.pos_y, rec.damage_pct)
    };

    let mut drops_summary = Vec::new();
    if !rec.item_totals.is_empty() {
        // Collect and sort items for display
        let mut items: Vec<(&String, &i64)> = rec.item_totals.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));

        let mut current_line = String::new();
        for (idx, (name, count)) in items.iter().take(6).enumerate() {
            let part = format!("{}: {}", name, count);
            if current_line.is_empty() {
                current_line = format!("• {}", part);
            } else if current_line.len() + part.len() < 45 {
                current_line.push_str(" | ");
                current_line.push_str(&part);
            } else {
                drops_summary.push(current_line);
                current_line = format!("• {}", part);
            }
            if idx == 5 && !current_line.is_empty() {
                drops_summary.push(current_line.clone());
                current_line.clear();
            }
        }
        if !current_line.is_empty() && !drops_summary.contains(&current_line) {
            drops_summary.push(current_line);
        }
    }

    if drops_summary.is_empty() {
        drops_summary.push("• (No major items recorded)".to_string());
    }

    CardDisplayData {
        tag,
        target_display,
        time_display,
        meta_display,
        drops_summary,
        accent_color,
    }
}

pub fn compute_idle_dim_factor(last_arrival: std::time::Instant) -> f32 {
    let elapsed = last_arrival.elapsed().as_secs_f32();
    if elapsed <= 20.0 {
        1.0
    } else if elapsed >= 23.0 {
        0.65
    } else {
        // Linear transition from 1.0 down to 0.65 over 3 seconds
        let progress = (elapsed - 20.0) / 3.0;
        1.0 - (progress * 0.35)
    }
}

pub fn get_latest_filtered_records(records: &[BattleRecord], filter: FilterMode, limit: usize) -> Vec<BattleRecord> {
    let mut filtered: Vec<BattleRecord> = records
        .iter()
        .filter(|r| filter.matches(&r.battle_type))
        .cloned()
        .collect();

    filtered.sort_by(|a, b| b.time_raw.cmp(&a.time_raw));
    filtered.truncate(limit);
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_emojis_in_card_formatting() {
        let mut item_totals = std::collections::HashMap::new();
        item_totals.insert("Sách ước nguyện".to_string(), 2);
        item_totals.insert("Đá quý".to_string(), 100);

        let rec = BattleRecord {
            filename: "test.mail".into(),
            mail_id: "1".into(),
            battle_type: "fort".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "13:42".into(),
            target: "Pháo Đài Cấp 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 524,
            pos_y: 681,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals,
            content: String::new(),
        };

        let card = format_card_data(&rec);
        assert_eq!(card.tag, "[Fort]");
        assert!(card.target_display.contains("Pháo Đài Cấp 10"));
        assert_eq!(card.meta_display, "Coord: (524, 681) | 100% Damage");
        
        let check_no_emoji = |s: &str| {
            for ch in s.chars() {
                let u = ch as u32;
                let is_emoji = (0x1F600..=0x1F64F).contains(&u)
                    || (0x1F300..=0x1F5FF).contains(&u)
                    || (0x1F680..=0x1F6FF).contains(&u)
                    || (0x2600..=0x26FF).contains(&u)
                    || (0x2700..=0x27BF).contains(&u)
                    || (0x1F900..=0x1F9FF).contains(&u)
                    || (0x1FA70..=0x1FAFF).contains(&u);
                assert!(!is_emoji, "Found emoji character: '{}' (U+{:X}) in '{}'", ch, u, s);
            }
        };

        check_no_emoji(&card.tag);
        check_no_emoji(&card.target_display);
        check_no_emoji(&card.time_display);
        check_no_emoji(&card.meta_display);
        for line in &card.drops_summary {
            check_no_emoji(line);
        }
    }

    #[test]
    fn test_compute_idle_dim_factor() {
        let now = std::time::Instant::now();
        // Fresh report (under 20s) -> 1.0 full opacity
        assert_eq!(compute_idle_dim_factor(now), 1.0);
        // Old report (> 20s) -> 0.65 dimmed opacity
        let past = now - std::time::Duration::from_secs(30);
        assert!((compute_idle_dim_factor(past) - 0.65).abs() < 0.01);
    }

    #[test]
    fn test_get_latest_filtered_records_limit_5() {
        let mut records = Vec::new();
        for i in 1..=10 {
            records.push(BattleRecord {
                filename: format!("mail_{}", i),
                mail_id: i.to_string(),
                battle_type: if i % 2 == 0 { "fort".into() } else { "motte".into() },
                time_raw: i * 100,
                time_utc: "".into(),
                time_vn: "".into(),
                target: format!("Target {}", i),
                target_level: "1".into(),
                talent_add: 0,
                pos_x: 0,
                pos_y: 0,
                damage_pct: 100,
                plunder_tier: 0,
                items: vec![],
                item_totals: std::collections::HashMap::new(),
                content: String::new(),
            });
        }

        let forts = get_latest_filtered_records(&records, FilterMode::Fort, 5);
        assert_eq!(forts.len(), 5);
        for f in &forts {
            assert_eq!(f.battle_type, "fort");
        }
        assert_eq!(forts[0].time_raw, 1000);
    }
}
