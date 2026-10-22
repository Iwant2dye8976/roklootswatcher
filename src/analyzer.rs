use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::model::{BattleRecord, ItemStat, PlunderTierStat, Statistics};

/// Trả về ngày hiện tại theo múi giờ Việt Nam (UTC+7)
pub fn current_date_vn() -> NaiveDate {
    (Utc::now() + Duration::try_hours(7).unwrap_or_else(|| Duration::hours(7))).date_naive()
}

/// Trích xuất ngày NaiveDate từ bản ghi chiến báo theo giờ Việt Nam
pub fn get_record_date_vn(r: &BattleRecord) -> Option<NaiveDate> {
    if let Some(d) = parse_date_str(&r.time_vn) {
        return Some(d);
    }
    if r.time_raw > 0 {
        let secs = if r.time_raw > 100_000_000_000 {
            r.time_raw / 1_000_000
        } else {
            r.time_raw
        };
        if let Some(dt) = DateTime::from_timestamp(secs, 0) {
            return Some((dt + Duration::try_hours(7).unwrap_or_else(|| Duration::hours(7))).date_naive());
        }
    }
    None
}

/// Parse chuỗi ngày linh hoạt hỗ trợ nhiều định dạng:
/// - YYYY-MM-DD (e.g. "2026-10-22" hoặc "2026-10-22 14:30:00")
/// - YYYY/MM/DD, YYYY.MM.DD
/// - DD/MM/YYYY, DD-MM-YYYY, DD.MM.YYYY
pub fn parse_date_str(d_str: &str) -> Option<NaiveDate> {
    let clean = d_str.trim();
    if clean.is_empty() {
        return None;
    }

    if clean.len() >= 10 {
        let prefix = &clean[..10];
        let b = prefix.as_bytes();

        // Định dạng YYYY[-/.]MM[-/.]DD
        let is_sep_4 = b[4] == b'-' || b[4] == b'/' || b[4] == b'.';
        let is_sep_7 = b[7] == b'-' || b[7] == b'/' || b[7] == b'.';
        if is_sep_4 && is_sep_7 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                prefix[0..4].parse::<i32>(),
                prefix[5..7].parse::<u32>(),
                prefix[8..10].parse::<u32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }

        // Định dạng DD[-/.]MM[-/.]YYYY
        let is_sep_2 = b[2] == b'-' || b[2] == b'/' || b[2] == b'.';
        let is_sep_5 = b[5] == b'-' || b[5] == b'/' || b[5] == b'.';
        if is_sep_2 && is_sep_5 {
            if let (Ok(d), Ok(m), Ok(y)) = (
                prefix[0..2].parse::<u32>(),
                prefix[3..5].parse::<u32>(),
                prefix[6..10].parse::<i32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
    }

    // Các mẫu dự phòng bổ sung
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d/%m/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d-%m-%Y") {
        return Some(d);
    }
    None
}

pub fn filter_records_by_date<'a>(
    records: impl IntoIterator<Item = &'a BattleRecord>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    days: Option<u32>,
) -> Vec<BattleRecord> {
    let (s_date, e_date) = if let Some(d) = days {
        if d > 0 {
            let today = current_date_vn();
            let start = today - Duration::try_days((d - 1) as i64).unwrap_or_else(|| Duration::days((d - 1) as i64));
            (Some(start), Some(today))
        } else {
            (start_date, end_date)
        }
    } else {
        match (start_date, end_date) {
            (Some(s), Some(e)) => {
                if s > e {
                    (Some(e), Some(s)) // Tự động đảo chiều nếu người dùng nhập ngược
                } else {
                    (Some(s), Some(e))
                }
            }
            (s, e) => (s, e),
        }
    };

    records
        .into_iter()
        .filter(|r| {
            if let Some(d) = get_record_date_vn(r) {
                if let Some(s) = s_date {
                    if d < s {
                        return false;
                    }
                }
                if let Some(e) = e_date {
                    if d > e {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

pub fn generate_statistics(records: &[BattleRecord]) -> Statistics {
    if records.is_empty() {
        return Statistics::default();
    }

    let total_battles = records.len();
    let total_dmg: i32 = records.iter().map(|r| r.damage_pct).sum();
    let avg_dmg = (total_dmg as f64) / (total_battles as f64);
    let min_dmg = records.iter().map(|r| r.damage_pct).min().unwrap_or(0);
    let max_dmg = records.iter().map(|r| r.damage_pct).max().unwrap_or(0);

    // Aggregates by item
    let mut item_stats: HashMap<String, ItemStat> = HashMap::new();

    for r in records {
        for it in &r.items {
            let key = format!("{}_{}", it.itype, it.sub_type);
            let entry = item_stats.entry(key).or_insert_with(|| {
                let cat = crate::loot_catalog::LootCatalog::determine_category(it.itype, it.sub_type);
                ItemStat {
                    itype: it.itype,
                    sub_type: it.sub_type,
                    name_vi: it.name_vi.clone(),
                    name_en: it.name_en.clone(),
                    category: cat.to_string(),
                    sprites: it.sprites.clone(),
                    pic: it.pic.clone(),
                    total_quantity: 0,
                    battle_count: 0,
                    max_in_battle: it.value,
                    min_in_battle: it.value,
                    avg_per_battle: 0.0,
                }
            });

            entry.total_quantity += it.value;
            entry.battle_count += 1;
            entry.max_in_battle = entry.max_in_battle.max(it.value);
            entry.min_in_battle = entry.min_in_battle.min(it.value);
        }
    }

    for stat in item_stats.values_mut() {
        stat.avg_per_battle =
            ((stat.total_quantity as f64) / (total_battles as f64) * 10.0).round() / 10.0;
    }

    // Breakdown by Plunder Tier
    #[allow(dead_code)]
    struct TierAccum {
        tier: i32,
        count: usize,
        dmgs: Vec<i32>,
        crystals: Vec<i64>,
        res_packs: Vec<i64>,
        tomes: Vec<i64>,
        heal_spd: Vec<i64>,
        univ_spd: Vec<i64>,
        starlight: i64,
    }

    let mut tier_map: HashMap<i32, TierAccum> = HashMap::new();

    for r in records {
        let entry = tier_map.entry(r.plunder_tier).or_insert_with(|| TierAccum {
            tier: r.plunder_tier,
            count: 0,
            dmgs: Vec::new(),
            crystals: Vec::new(),
            res_packs: Vec::new(),
            tomes: Vec::new(),
            heal_spd: Vec::new(),
            univ_spd: Vec::new(),
            starlight: 0,
        });

        entry.count += 1;
        entry.dmgs.push(r.damage_pct);
        entry.crystals.push(r.item_quantity(1, 9));
        entry.res_packs.push(r.total_resources_breakdown().choice_packs);
        entry.tomes.push(r.item_quantity(2, 130));
        entry.heal_spd.push(r.item_quantity(2, 92));
        entry.univ_spd.push(r.item_quantity(2, 58));
        entry.starlight += r.item_quantity(2, 9998);
    }

    fn calc_avg_i64(vals: &[i64]) -> i64 {
        if vals.is_empty() {
            0
        } else {
            (vals.iter().sum::<i64>() as f64 / vals.len() as f64).round() as i64
        }
    }

    let mut plunder_breakdown = HashMap::new();
    for (tier, accum) in tier_map {
        let min_d = accum.dmgs.iter().copied().min().unwrap_or(0);
        let max_d = accum.dmgs.iter().copied().max().unwrap_or(0);
        let avg_d = if !accum.dmgs.is_empty() {
            ((accum.dmgs.iter().sum::<i32>() as f64) / (accum.dmgs.len() as f64) * 10.0).round() / 10.0
        } else {
            0.0
        };

        let avg_c = calc_avg_i64(&accum.crystals);
        let avg_r = calc_avg_i64(&accum.res_packs);
        let avg_t = calc_avg_i64(&accum.tomes);
        let avg_h = calc_avg_i64(&accum.heal_spd);
        let avg_u = calc_avg_i64(&accum.univ_spd);

        plunder_breakdown.insert(
            tier,
            PlunderTierStat {
                tier,
                count: accum.count,
                min_dmg: min_d,
                max_dmg: max_d,
                avg_dmg: avg_d,
                avg_crystal: avg_c,
                avg_res_packs: avg_r,
                avg_tomes: avg_t,
                avg_heal_spd: avg_h,
                avg_univ_spd: avg_u,
                starlight: accum.starlight,
            },
        );
    }

    Statistics {
        total_battles,
        avg_damage_pct: (avg_dmg * 10.0).round() / 10.0,
        min_damage_pct: min_dmg,
        max_damage_pct: max_dmg,
        item_stats,
        plunder_breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_str() {
        assert_eq!(
            parse_date_str("2026-10-21 14:30:00"),
            Some(NaiveDate::from_ymd_opt(2026, 10, 21).unwrap())
        );
        assert_eq!(
            parse_date_str("2026-05-01"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("2026/10/22"),
            Some(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap())
        );
        assert_eq!(
            parse_date_str("22/10/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap())
        );
        assert_eq!(
            parse_date_str("22-10-2026"),
            Some(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap())
        );
        assert_eq!(
            parse_date_str("22.10.2026"),
            Some(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap())
        );
        assert_eq!(parse_date_str(""), None);
        assert_eq!(parse_date_str("invalid-date"), None);
    }

    #[test]
    fn test_filter_records_by_date() {
        let make_rec = |time_vn: &str, time_raw: i64| BattleRecord {
            filename: "".into(),
            mail_id: "".into(),
            battle_type: "barbarian".into(),
            time_raw,
            time_utc: "".into(),
            time_vn: time_vn.into(),
            target: "".into(),
            motte_level: "".into(),
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let recs = vec![
            make_rec("2026-10-18 10:00:00", 0),
            make_rec("2026-10-20 12:00:00", 0),
            make_rec("2026-10-22 08:00:00", 0),
        ];

        let d18 = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
        let d20 = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
        let d22 = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();

        // Exact day
        let f20 = filter_records_by_date(&recs, Some(d20), Some(d20), None);
        assert_eq!(f20.len(), 1);
        assert_eq!(f20[0].time_vn, "2026-10-20 12:00:00");

        // Range
        let f18_20 = filter_records_by_date(&recs, Some(d18), Some(d20), None);
        assert_eq!(f18_20.len(), 2);

        // Auto swap reversed dates (d22 down to d20)
        let f_swap = filter_records_by_date(&recs, Some(d22), Some(d20), None);
        assert_eq!(f_swap.len(), 2);

        let f_iter = filter_records_by_date(
            recs.iter().filter(|r| r.time_vn.starts_with("2026-10-20")),
            Some(d18),
            Some(d22),
            None,
        );
        assert_eq!(f_iter.len(), 1);
        assert_eq!(f_iter[0].time_vn, "2026-10-20 12:00:00");
    }
}
