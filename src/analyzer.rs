use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::model::{BattleRecord, ItemDrop, ItemStat, PlunderTierStat, Statistics};

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

fn parse_single_date_token(token: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = token.split(['-', '/', '.']).collect();
    if parts.len() == 3 {
        // YYYY[-/.]M[-/.]D (hỗ trợ cả 1 hoặc 2 chữ số cho tháng và ngày)
        if parts[0].len() == 4 && (1..=2).contains(&parts[1].len()) && (1..=2).contains(&parts[2].len()) {
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
        // D[-/.]M[-/.]YYYY (hỗ trợ cả 1 hoặc 2 chữ số cho ngày và tháng)
        if parts[2].len() == 4 && (1..=2).contains(&parts[0].len()) && (1..=2).contains(&parts[1].len()) {
            if let (Ok(d), Ok(m), Ok(y)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<i32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
    }
    None
}

/// Parse chuỗi ngày linh hoạt hỗ trợ nhiều định dạng:
/// - YYYY-MM-DD, YYYY-M-D (e.g. "2026-10-22", "2026-5-1", "2026-10-22 14:30:00")
/// - YYYY/MM/DD, YYYY/M/D, YYYY.MM.DD, YYYY.M.D
/// - DD/MM/YYYY, D/M/YYYY, DD-MM-YYYY, D-M-YYYY, DD.MM.YYYY, D.M.YYYY
pub fn parse_date_str(d_str: &str) -> Option<NaiveDate> {
    let clean = d_str.trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Thử trực tiếp toàn bộ chuỗi
    if let Some(d) = parse_single_date_token(clean) {
        return Some(d);
    }

    // 2. Thử từ đầu tiên nếu chuỗi chứa giờ (vd: "2026-5-1 14:30:00")
    if let Some(first_word) = clean.split_whitespace().next() {
        if first_word != clean {
            if let Some(d) = parse_single_date_token(first_word) {
                return Some(d);
            }
        }
    }

    // 3. Fallback Chrono parse
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d/%m/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d-%m-%Y") {
        return Some(d);
    }

    // 4. Tìm kiếm từ ngày nhúng trong chuỗi (ví dụ: "Ngày 1/5/2026")
    for word in clean.split_whitespace() {
        if word != clean {
            if let Some(d) = parse_single_date_token(word) {
                return Some(d);
            }
        }
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

/// Trả về N bản ghi mới nhất theo thứ tự giảm dần của thời gian (mới nhất đứng đầu).
pub fn get_latest_records<'a>(records: &'a [BattleRecord], limit: usize) -> Vec<&'a BattleRecord> {
    let mut sorted: Vec<&'a BattleRecord> = records.iter().collect();
    sorted.sort_by(|a, b| {
        b.time_raw.cmp(&a.time_raw).then_with(|| b.time_vn.cmp(&a.time_vn))
    });
    sorted.truncate(limit);
    sorted
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
    let mut battle_items: HashMap<(u32, u32), (&ItemDrop, i64)> = HashMap::new();

    for r in records {
        // Gom nhóm vật phẩm trong cùng 1 trận theo (itype, sub_type)
        // để chiến báo đánh dã man chuỗi (chained barbarian) không làm thổi phồng battle_count
        // và tính đúng tổng max_in_battle của trận đó.
        for it in &r.items {
            battle_items
                .entry((it.itype, it.sub_type))
                .and_modify(|(_, sum)| *sum += it.value)
                .or_insert((it, it.value));
        }

        for (&(itype, sub_type), &(sample_it, battle_qty)) in &battle_items {
            let key = format!("{}_{}", itype, sub_type);
            let entry = item_stats.entry(key).or_insert_with(|| {
                let cat = crate::loot_catalog::LootCatalog::determine_category(itype, sub_type);
                ItemStat {
                    itype,
                    sub_type,
                    name_vi: sample_it.name_vi.clone(),
                    name_en: sample_it.name_en.clone(),
                    category: cat.to_string(),
                    sprites: sample_it.sprites.clone(),
                    pic: sample_it.pic.clone(),
                    total_quantity: 0,
                    battle_count: 0,
                    max_in_battle: 0,
                    min_in_battle: i64::MAX,
                    avg_per_battle: 0.0,
                    by_battle_type: HashMap::new(),
                }
            });

            entry.total_quantity += battle_qty;
            entry.battle_count += 1;
            entry.max_in_battle = entry.max_in_battle.max(battle_qty);
            entry.min_in_battle = entry.min_in_battle.min(battle_qty);

            let btype = if r.battle_type.is_empty() { "other".to_string() } else { r.battle_type.clone() };
            *entry.by_battle_type.entry(btype).or_insert(0) += battle_qty;
        }

        battle_items.clear();
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
        min_dmg: i32,
        max_dmg: i32,
        sum_dmg: i32,
        sum_crystals: i64,
        sum_res_packs: i64,
        sum_tomes: i64,
        sum_heal_spd: i64,
        sum_univ_spd: i64,
        starlight: i64,
    }

    let mut tier_map: HashMap<i32, TierAccum> = HashMap::new();

    for r in records {
        let entry = tier_map.entry(r.plunder_tier).or_insert_with(|| TierAccum {
            tier: r.plunder_tier,
            count: 0,
            min_dmg: i32::MAX,
            max_dmg: i32::MIN,
            sum_dmg: 0,
            sum_crystals: 0,
            sum_res_packs: 0,
            sum_tomes: 0,
            sum_heal_spd: 0,
            sum_univ_spd: 0,
            starlight: 0,
        });

        entry.count += 1;
        entry.min_dmg = entry.min_dmg.min(r.damage_pct);
        entry.max_dmg = entry.max_dmg.max(r.damage_pct);
        entry.sum_dmg += r.damage_pct;
        entry.sum_crystals += r.item_quantity(1, 9);
        entry.sum_res_packs += r.total_resources_breakdown().choice_packs;
        entry.sum_tomes += r.item_quantity(2, 130);
        entry.sum_heal_spd += r.item_quantity(2, 92);
        entry.sum_univ_spd += r.item_quantity(2, 58);
        entry.starlight += r.item_quantity(2, 9998);
    }

    let mut plunder_breakdown = HashMap::new();
    for (tier, accum) in tier_map {
        let count_f = if accum.count > 0 { accum.count as f64 } else { 1.0 };
        let min_d = if accum.count > 0 { accum.min_dmg } else { 0 };
        let max_d = if accum.count > 0 { accum.max_dmg } else { 0 };
        let avg_d = if accum.count > 0 {
            ((accum.sum_dmg as f64 / count_f) * 10.0).round() / 10.0
        } else {
            0.0
        };

        let avg_c = (accum.sum_crystals as f64 / count_f).round() as i64;
        let avg_r = (accum.sum_res_packs as f64 / count_f).round() as i64;
        let avg_t = (accum.sum_tomes as f64 / count_f).round() as i64;
        let avg_h = (accum.sum_heal_spd as f64 / count_f).round() as i64;
        let avg_u = (accum.sum_univ_spd as f64 / count_f).round() as i64;

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
        assert_eq!(
            parse_date_str("2026-5-1"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("1/5/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("1-5-2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("2026.5.1"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("1.5.2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("2026-5-1 14:30:00"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(
            parse_date_str("Ngày 1/5/2026"),
            Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        );
        assert_eq!(parse_date_str(""), None);
        assert_eq!(parse_date_str("invalid-date"), None);
    }

    #[test]
    fn test_parse_date_str_with_multibyte_utf8() {
        // String where byte index 10 falls in the middle of a multibyte UTF-8 char:
        // "123456789" (9 bytes) + "ư" (0xC6 0xB0 = 2 bytes: byte 9 and 10)
        let s = "123456789ư";
        assert_eq!(parse_date_str(s), None);

        // Vietnamese prefix with valid date afterwards
        assert_eq!(parse_date_str("Ngày 22/10/2026"), Some(NaiveDate::from_ymd_opt(2026, 10, 22).unwrap()));
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

    #[test]
    fn test_get_latest_records_excludes_old_records() {
        let make_rec = |mail_id: &str, time_vn: &str, time_raw: i64| BattleRecord {
            filename: "".into(),
            mail_id: mail_id.into(),
            battle_type: "motte".into(),
            time_raw,
            time_utc: "".into(),
            time_vn: time_vn.into(),
            target: format!("Target {}", mail_id),
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

        // Danh sách lưu theo thứ tự tăng dần thời gian (cũ nhất ở đầu, giống trong Vault)
        let recs = vec![
            make_rec("old_2022", "2022-08-21 07:03:04", 1661040184654146),
            make_rec("old_2025", "2025-01-30 02:24:20", 1738178660507109),
            make_rec("new_1", "2026-10-23 09:00:00", 1792720000000000),
            make_rec("new_2", "2026-10-23 10:00:00", 1792723600000000),
            make_rec("new_3", "2026-10-23 10:30:00", 1792725400000000),
            make_rec("new_4", "2026-10-23 11:00:00", 1792727200000000),
            make_rec("new_5", "2026-10-23 11:30:00", 1792729000000000),
        ];

        let latest = get_latest_records(&recs, 5);
        assert_eq!(latest.len(), 5);

        // Phải được sắp xếp theo thời gian mới nhất đứng đầu
        assert_eq!(latest[0].mail_id, "new_5");
        assert_eq!(latest[1].mail_id, "new_4");
        assert_eq!(latest[2].mail_id, "new_3");
        assert_eq!(latest[3].mail_id, "new_2");
        assert_eq!(latest[4].mail_id, "new_1");

        // Các bản ghi cũ từ 2022 và 2025 tuyệt đối không được xuất hiện
        assert!(latest.iter().all(|r| r.mail_id != "old_2022" && r.mail_id != "old_2025"));
    }

    #[test]
    fn test_chained_barbarian_item_aggregation_no_drop_rate_inflation() {
        let mut items = Vec::new();
        // Giả lập 4 dã man trong 1 chiến báo cùng rớt Gems (1, 5)
        for _ in 0..4 {
            items.push(ItemDrop {
                itype: 1,
                sub_type: 5,
                value: 10,
                pic: "".into(),
                name_vi: "Gems".into(),
                name_en: "Gems".into(),
                category: "Gems".into(),
                sprites: vec![],
            });
        }

        let record = BattleRecord {
            filename: "chained.mail".into(),
            mail_id: "123".into(),
            battle_type: "barbarian".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:00:00".into(),
            target: "Dã Man Cấp 25".into(),
            target_level: "25".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 0,
            items,
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let stats = generate_statistics(&[record]);
        let gem_stat = stats.item_stats.get("1_5").expect("Must have Gems stats");

        // 1 battle only, so battle_count MUST be 1 (not 4)
        assert_eq!(gem_stat.battle_count, 1, "Battle count must be 1 per battle even if chained kills drop same item");
        // Total quantity is 4 * 10 = 40
        assert_eq!(gem_stat.total_quantity, 40);
        // Max in battle must be the sum for this battle = 40
        assert_eq!(gem_stat.max_in_battle, 40);
        // Min in battle is also 40
        assert_eq!(gem_stat.min_in_battle, 40);
    }

    #[test]
    fn test_filter_records_by_rolling_days() {
        let today = current_date_vn();
        let yest = today - Duration::days(1);
        let make_rec = |d: NaiveDate| BattleRecord {
            filename: "".into(),
            mail_id: "".into(),
            battle_type: "barbarian".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: format!("{} 12:00:00", d),
            target: "".into(),
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

        let rec_today = make_rec(today);
        let rec_yest = make_rec(yest);
        let recs = vec![rec_today.clone(), rec_yest.clone()];

        // 1 rolling day = Today only
        let filtered_today = filter_records_by_date(&recs, None, None, Some(1));
        assert_eq!(filtered_today.len(), 1);
        assert_eq!(filtered_today[0].time_vn, rec_today.time_vn);

        // 2 rolling days = Today and Yesterday
        let filtered_2days = filter_records_by_date(&recs, None, None, Some(2));
        assert_eq!(filtered_2days.len(), 2);
    }

    #[test]
    fn test_item_stat_by_battle_type_distribution() {
        let rec_motte = BattleRecord {
            filename: "motte.mail".into(),
            mail_id: "m1".into(),
            battle_type: "motte".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:00:00".into(),
            target: "Motte 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 1,
            items: vec![ItemDrop {
                itype: 1,
                sub_type: 9,
                value: 9000,
                pic: "".into(),
                name_vi: "Pha Lê".into(),
                name_en: "Crystals".into(),
                category: "Pha Lê".into(),
                sprites: vec![],
            }],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let rec_barb = BattleRecord {
            filename: "barb.mail".into(),
            mail_id: "b1".into(),
            battle_type: "barbarian".into(),
            time_raw: 1001,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:05:00".into(),
            target: "Dã Man Cấp 25".into(),
            target_level: "25".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![ItemDrop {
                itype: 1,
                sub_type: 9,
                value: 11000,
                pic: "".into(),
                name_vi: "Pha Lê".into(),
                name_en: "Crystals".into(),
                category: "Pha Lê".into(),
                sprites: vec![],
            }],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let stats = generate_statistics(&[rec_motte, rec_barb]);
        let crystal_stat = stats.item_stats.get("1_9").expect("Crystal stat must exist");
        assert_eq!(crystal_stat.total_quantity, 20000);
        assert_eq!(crystal_stat.by_battle_type.get("motte"), Some(&9000));
        assert_eq!(crystal_stat.by_battle_type.get("barbarian"), Some(&11000));
    }

    #[test]
    fn test_plunder_tier_aggregation_and_battle_items_reuse() {
        let make_item = |itype: u32, sub_type: u32, value: i64| ItemDrop {
            itype,
            sub_type,
            value,
            pic: "".into(),
            name_vi: format!("Item {}_{}", itype, sub_type),
            name_en: format!("Item {}_{}", itype, sub_type),
            category: "Other".into(),
            sprites: vec![],
        };

        let rec1 = BattleRecord {
            filename: "rec1.mail".into(),
            mail_id: "r1".into(),
            battle_type: "barbarian".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:00:00".into(),
            target: "Fort".into(),
            target_level: "5".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 20,
            plunder_tier: 1,
            items: vec![
                make_item(1, 9, 100),    // crystals
                make_item(2, 7005, 4),   // choice pack
                make_item(2, 130, 10),   // tomes
                make_item(2, 92, 5),     // heal speedup
                make_item(2, 58, 15),    // univ speedup
                make_item(2, 9998, 2),   // starlight
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let rec2 = BattleRecord {
            filename: "rec2.mail".into(),
            mail_id: "r2".into(),
            battle_type: "barbarian".into(),
            time_raw: 1001,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:05:00".into(),
            target: "Fort".into(),
            target_level: "5".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 40,
            plunder_tier: 1,
            items: vec![
                make_item(1, 9, 200),
                make_item(2, 7005, 6),
                make_item(2, 130, 20),
                make_item(2, 92, 15),
                make_item(2, 58, 25),
                make_item(2, 9998, 3),
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let rec3 = BattleRecord {
            filename: "rec3.mail".into(),
            mail_id: "r3".into(),
            battle_type: "barbarian".into(),
            time_raw: 1002,
            time_utc: "".into(),
            time_vn: "2026-10-24 10:10:00".into(),
            target: "Fort".into(),
            target_level: "6".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 60,
            plunder_tier: 2,
            items: vec![
                make_item(1, 9, 500),
                make_item(2, 7005, 10),
                make_item(2, 130, 50),
                make_item(2, 92, 30),
                make_item(2, 58, 40),
                make_item(2, 9998, 10),
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let stats = generate_statistics(&[rec1, rec2, rec3]);

        assert_eq!(stats.total_battles, 3);
        assert_eq!(stats.min_damage_pct, 20);
        assert_eq!(stats.max_damage_pct, 60);
        assert_eq!(stats.avg_damage_pct, 40.0);

        let t1 = stats.plunder_breakdown.get(&1).expect("Tier 1 must exist");
        assert_eq!(t1.tier, 1);
        assert_eq!(t1.count, 2);
        assert_eq!(t1.min_dmg, 20);
        assert_eq!(t1.max_dmg, 40);
        assert_eq!(t1.avg_dmg, 30.0);
        assert_eq!(t1.avg_crystal, 150);
        assert_eq!(t1.avg_res_packs, 5);
        assert_eq!(t1.avg_tomes, 15);
        assert_eq!(t1.avg_heal_spd, 10);
        assert_eq!(t1.avg_univ_spd, 20);
        assert_eq!(t1.starlight, 5);

        let t2 = stats.plunder_breakdown.get(&2).expect("Tier 2 must exist");
        assert_eq!(t2.tier, 2);
        assert_eq!(t2.count, 1);
        assert_eq!(t2.min_dmg, 60);
        assert_eq!(t2.max_dmg, 60);
        assert_eq!(t2.avg_dmg, 60.0);
        assert_eq!(t2.avg_crystal, 500);
        assert_eq!(t2.avg_res_packs, 10);
        assert_eq!(t2.avg_tomes, 50);
        assert_eq!(t2.avg_heal_spd, 30);
        assert_eq!(t2.avg_univ_spd, 40);
        assert_eq!(t2.starlight, 10);

        let crystal_stat = stats.item_stats.get("1_9").expect("Crystals stat must exist");
        assert_eq!(crystal_stat.battle_count, 3);
        assert_eq!(crystal_stat.total_quantity, 800);
        assert_eq!(crystal_stat.min_in_battle, 100);
        assert_eq!(crystal_stat.max_in_battle, 500);
    }
}
