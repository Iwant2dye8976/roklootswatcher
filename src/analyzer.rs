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
        entry.sum_heal_spd += r.heal_speedup_minutes();
        entry.sum_univ_spd += r.univ_speedup_minutes();
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

/// Calculates Action Point (AP) cost per battle based on target level tiers:
/// - Barbarian Forts: Lvl 1-10 = 150 AP | Lvl 11-15 = 300 AP
/// - Motte: Lvl 1-10 = 150 AP | Lvl 11-15 = 300 AP
/// - Barbarians: Lvl 1-40 = 40 AP | Lvl 41-55 = 80 AP
/// - Chained reports (AoE) count as 1 march cost
/// - Gathering / other = 0 AP
pub fn calculate_ap_cost(battle_type: &str, target_level: &str) -> i64 {
    let lvl: u32 = target_level
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse::<u32>().ok())
        .max()
        .unwrap_or(0);

    match battle_type {
        "fort" => {
            if lvl <= 10 { 150 } else { 300 }
        }
        "motte" => {
            if lvl <= 10 { 150 } else { 300 }
        }
        "barbarian" => {
            if lvl <= 40 { 40 } else { 80 }
        }
        _ => 0,
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DailyApSpeedupStat {
    pub date_label: String,
    pub speedup_hours: f64,
    pub ap_consumed: i64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TargetRoiBenchmark {
    pub target_name: String,
    pub unit_ap_cost: i64,
    pub battle_count: usize,
    pub avg_speedup_mins: f64,
    pub speedup_mins_per_1k_ap: f64,
    pub covenant_drop_rate: f64,
    pub avg_gems: f64,
    pub tier_badge: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApEconomics {
    pub total_ap_consumed: i64,
    pub total_ap_recovered: i64,
    pub net_ap_spent: i64,
    pub rebate_pct: f64,
    pub speedup_mins_per_1k_ap: f64,
    pub gems_per_1k_ap: f64,
    pub daily_timeline: Vec<DailyApSpeedupStat>,
    pub target_benchmarks: Vec<TargetRoiBenchmark>,
}

/// Computes AP Economics and Drop Rate Analytics across active filtered records
/// and a rolling 7-day timeline from historical vault records.
pub fn compute_ap_economics(active_records: &[BattleRecord], vault_records: &[BattleRecord]) -> ApEconomics {
    let mut total_ap_consumed = 0i64;
    let mut total_ap_recovered = 0i64;
    let mut total_speedup_mins = 0i64;
    let mut total_gems = 0i64;

    struct TargetAccumulator {
        name: &'static str,
        unit_ap: i64,
        count: usize,
        speedup_mins: i64,
        gems: i64,
        covenant_battles: usize,
    }

    let mut target_groups = [
        TargetAccumulator { name: "Fort Lvl 11-15 (KvK)", unit_ap: 300, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
        TargetAccumulator { name: "Fort Lvl 1-10", unit_ap: 150, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
        TargetAccumulator { name: "Motte Lvl 11-15", unit_ap: 300, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
        TargetAccumulator { name: "Motte Lvl 1-10", unit_ap: 150, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
        TargetAccumulator { name: "Barbarians Lvl 41-55", unit_ap: 80, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
        TargetAccumulator { name: "Barbarians Lvl 1-40", unit_ap: 40, count: 0, speedup_mins: 0, gems: 0, covenant_battles: 0 },
    ];

    for r in active_records {
        if r.battle_type == "gathering" {
            continue;
        }
        let ap_cost = calculate_ap_cost(&r.battle_type, &r.target_level);
        total_ap_consumed += ap_cost;
        total_ap_recovered += r.ap_breakdown().total_ap_points();

        let spd = r.total_speedup_minutes();
        total_speedup_mins += spd;

        let gems = r.compute_gems();
        total_gems += gems;

        let has_covenant = r.items.iter().any(|it| it.itype == 2 && it.sub_type == 109);

        let lvl: u32 = r.target_level
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse::<u32>().ok())
            .max()
            .unwrap_or(0);

        let group_idx = match r.battle_type.as_str() {
            "fort" if lvl > 10 => Some(0),
            "fort" => Some(1),
            "motte" if lvl > 10 => Some(2),
            "motte" => Some(3),
            "barbarian" if lvl > 40 => Some(4),
            "barbarian" => Some(5),
            _ => None,
        };

        if let Some(idx) = group_idx {
            target_groups[idx].count += 1;
            target_groups[idx].speedup_mins += spd;
            target_groups[idx].gems += gems;
            if has_covenant {
                target_groups[idx].covenant_battles += 1;
            }
        }
    }

    let net_ap_spent = total_ap_consumed - total_ap_recovered;
    let rebate_pct = if total_ap_consumed > 0 {
        (total_ap_recovered as f64 / total_ap_consumed as f64) * 100.0
    } else {
        0.0
    };

    let speedup_mins_per_1k_ap = if total_ap_consumed > 0 {
        total_speedup_mins as f64 / (total_ap_consumed as f64 / 1000.0)
    } else {
        0.0
    };

    let gems_per_1k_ap = if total_ap_consumed > 0 {
        total_gems as f64 / (total_ap_consumed as f64 / 1000.0)
    } else {
        0.0
    };

    // Rolling 7 calendar days timeline (today down to 6 days ago) in a single pass
    let today = current_date_vn();
    let min_date = today - Duration::days(6);
    let mut day_stats = [(0i64, 0i64); 7]; // (speedup_mins, ap_consumed)

    for r in vault_records {
        if r.battle_type == "gathering" {
            continue;
        }
        if let Some(d) = get_record_date_vn(r) {
            if d >= min_date && d <= today {
                let day_idx = (d - min_date).num_days() as usize;
                if day_idx < 7 {
                    day_stats[day_idx].0 += r.total_speedup_minutes();
                    day_stats[day_idx].1 += calculate_ap_cost(&r.battle_type, &r.target_level);
                }
            }
        }
    }

    let mut daily_timeline = Vec::with_capacity(7);
    for (idx, stat) in day_stats.iter().enumerate() {
        let target_date = min_date + Duration::days(idx as i64);
        let date_label = if idx == 6 {
            "Today".to_string()
        } else {
            target_date.format("%d/%m").to_string()
        };
        daily_timeline.push(DailyApSpeedupStat {
            date_label,
            speedup_hours: stat.0 as f64 / 60.0,
            ap_consumed: stat.1,
        });
    }

    // Benchmark Matrix (only non-empty targets, ranked by yield)
    let mut target_benchmarks: Vec<TargetRoiBenchmark> = target_groups
        .into_iter()
        .filter(|g| g.count > 0)
        .map(|g| {
            let avg_speedup = g.speedup_mins as f64 / g.count as f64;
            let group_ap = g.count as i64 * g.unit_ap;
            let spd_per_1k = if group_ap > 0 {
                g.speedup_mins as f64 / (group_ap as f64 / 1000.0)
            } else {
                0.0
            };
            let covenant_rate = (g.covenant_battles as f64 / g.count as f64) * 100.0;
            let avg_gems = g.gems as f64 / g.count as f64;

            let tier_badge = if spd_per_1k >= 1000.0 {
                "TIER S+"
            } else if spd_per_1k >= 800.0 {
                "TIER S"
            } else if spd_per_1k >= 500.0 {
                "TIER A"
            } else {
                "TIER B"
            };

            TargetRoiBenchmark {
                target_name: g.name.to_string(),
                unit_ap_cost: g.unit_ap,
                battle_count: g.count,
                avg_speedup_mins: avg_speedup,
                speedup_mins_per_1k_ap: spd_per_1k,
                covenant_drop_rate: covenant_rate,
                avg_gems,
                tier_badge,
            }
        })
        .collect();

    target_benchmarks.sort_by(|a, b| b.speedup_mins_per_1k_ap.partial_cmp(&a.speedup_mins_per_1k_ap).unwrap_or(std::cmp::Ordering::Equal));

    ApEconomics {
        total_ap_consumed,
        total_ap_recovered,
        net_ap_spent,
        rebate_pct,
        speedup_mins_per_1k_ap,
        gems_per_1k_ap,
        daily_timeline,
        target_benchmarks,
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
        assert_eq!(t1.avg_heal_spd, 50);
        assert_eq!(t1.avg_univ_spd, 100);
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
        assert_eq!(t2.avg_heal_spd, 150);
        assert_eq!(t2.avg_univ_spd, 200);
        assert_eq!(t2.starlight, 10);

        let crystal_stat = stats.item_stats.get("1_9").expect("Crystals stat must exist");
        assert_eq!(crystal_stat.battle_count, 3);
        assert_eq!(crystal_stat.total_quantity, 800);
        assert_eq!(crystal_stat.min_in_battle, 100);
        assert_eq!(crystal_stat.max_in_battle, 500);
    }

    #[test]
    fn test_calculate_ap_cost_tiers() {
        assert_eq!(calculate_ap_cost("fort", "5"), 150);
        assert_eq!(calculate_ap_cost("fort", "10"), 150);
        assert_eq!(calculate_ap_cost("fort", "11"), 300);
        assert_eq!(calculate_ap_cost("fort", "15"), 300);
        assert_eq!(calculate_ap_cost("motte", "5"), 150);
        assert_eq!(calculate_ap_cost("motte", "12"), 300);
        assert_eq!(calculate_ap_cost("barbarian", "25"), 40);
        assert_eq!(calculate_ap_cost("barbarian", "40"), 40);
        assert_eq!(calculate_ap_cost("barbarian", "41"), 80);
        assert_eq!(calculate_ap_cost("barbarian", "47-52"), 80);
        assert_eq!(calculate_ap_cost("gathering", "5"), 0);
    }

    #[test]
    fn test_ap_economics_zero_ap_safety() {
        let stats = compute_ap_economics(&[], &[]);
        assert_eq!(stats.total_ap_consumed, 0);
        assert_eq!(stats.speedup_mins_per_1k_ap, 0.0);
        assert_eq!(stats.gems_per_1k_ap, 0.0);
        assert_eq!(stats.daily_timeline.len(), 7);
        assert_eq!(stats.target_benchmarks.len(), 0);
    }

    #[test]
    fn test_ap_economics_full_aggregation() {
        use crate::model::ItemDrop;

        let mk_item = |itype: u32, sub_type: u32, value: i64| ItemDrop {
            itype,
            sub_type,
            value,
            pic: "".into(),
            name_vi: "".into(),
            name_en: "".into(),
            category: "".into(),
            sprites: vec![],
        };

        let make_battle = |btype: &str, lvl: &str, spd_5m: i64, ap_50: i64, covenant: i64, gems_10: i64| {
            let mut items = Vec::new();
            if spd_5m > 0 {
                items.push(mk_item(2, 58, spd_5m)); // 5m universal
            }
            if ap_50 > 0 {
                items.push(mk_item(2, 182, ap_50)); // 50 AP pot
            }
            if covenant > 0 {
                items.push(mk_item(2, 109, covenant)); // Book of Covenant
            }
            if gems_10 > 0 {
                items.push(mk_item(2, 26, gems_10)); // 10 gems
            }
            BattleRecord {
                filename: "mail1".into(),
                mail_id: "1".into(),
                battle_type: btype.into(),
                time_raw: 1792720000,
                time_utc: "".into(),
                time_vn: "2026-10-28 10:00:00".into(),
                target: format!("{} Lvl {}", btype, lvl),
                target_level: lvl.into(),
                talent_add: 0,
                pos_x: 0,
                pos_y: 0,
                damage_pct: 100,
                plunder_tier: 0,
                items,
                item_totals: HashMap::new(),
                content: "".into(),
            }
        };

        // 2 x Fort 5 (150 AP each = 300 AP), 20x 5m speedups (100m total), 2x 50 AP pots (100 AP recovered), 1 book, 5x 10 gems (50 gems)
        let r1 = make_battle("fort", "5", 10, 1, 1, 3);
        let r2 = make_battle("fort", "5", 10, 1, 0, 2);

        // 1 x Barb 25 (40 AP), 2x 5m speedups (10m total), 0 AP pot, 0 book, 1x 10 gems (10 gems)
        let r3 = make_battle("barbarian", "25", 2, 0, 0, 1);

        let records = vec![r1, r2, r3];
        let econ = compute_ap_economics(&records, &records);

        // Gross AP: 150 + 150 + 40 = 340 AP
        assert_eq!(econ.total_ap_consumed, 340);
        // AP Recovered: 2 x 50 AP = 100 AP
        assert_eq!(econ.total_ap_recovered, 100);
        assert_eq!(econ.net_ap_spent, 240);
        // Rebate: (100 / 340) * 100 = 29.41%
        assert!((econ.rebate_pct - 29.41).abs() < 0.1);

        // Total speedup: 100m + 10m = 110m.
        // Speedup per 1k AP: 110 / 0.340 = 323.5m
        assert!((econ.speedup_mins_per_1k_ap - 323.5).abs() < 0.2);

        // Total gems: 50 + 10 = 60 gems.
        // Gems per 1k AP: 60 / 0.340 = 176.47
        assert!((econ.gems_per_1k_ap - 176.47).abs() < 0.2);

        // Target benchmarks: 2 rows (Fort Lvl 1-10, Barbarians Lvl 1-40)
        assert_eq!(econ.target_benchmarks.len(), 2);
        let fort_bm = econ.target_benchmarks.iter().find(|b| b.target_name == "Fort Lvl 1-10").unwrap();
        assert_eq!(fort_bm.battle_count, 2);
        assert_eq!(fort_bm.unit_ap_cost, 150);
        assert_eq!(fort_bm.avg_speedup_mins, 50.0);
        assert_eq!(fort_bm.covenant_drop_rate, 50.0); // 1 out of 2 battles
    }

    #[test]
    fn test_ap_economics_ignores_gathering_gems() {
        use crate::model::ItemDrop;

        let gathering_record = BattleRecord {
            filename: "mail_gather".into(),
            mail_id: "gather_1".into(),
            battle_type: "gathering".into(),
            time_raw: 1792720000,
            time_utc: "".into(),
            time_vn: "2026-10-28 10:00:00".into(),
            target: "Mỏ Đá Quý Cấp 2".into(),
            target_level: "2".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![
                ItemDrop {
                    itype: 1, // gathering gem node
                    sub_type: 5,
                    value: 10,
                    pic: "".into(),
                    name_vi: "".into(),
                    name_en: "".into(),
                    category: "".into(),
                    sprites: vec![],
                },
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let econ = compute_ap_economics(&[gathering_record], &[]);
        assert_eq!(econ.total_ap_consumed, 0);
        assert_eq!(econ.gems_per_1k_ap, 0.0, "Gathering gems must not count in AP economics");
        assert_eq!(econ.target_benchmarks.len(), 0);
    }

    #[test]
    fn test_ap_economics_timeline_today_label() {
        let econ = compute_ap_economics(&[], &[]);
        assert_eq!(econ.daily_timeline.len(), 7);
        assert_eq!(econ.daily_timeline[6].date_label, "Today");
        for i in 0..6 {
            assert_ne!(econ.daily_timeline[i].date_label, "Today");
            assert!(econ.daily_timeline[i].date_label.contains('/'));
        }
    }
}
