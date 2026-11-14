use std::collections::HashMap;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{BattleRecord, ItemDrop, ItemStat, PlunderTierStat, Statistics, TroopCombatStats};

pub fn current_date_vn() -> NaiveDate {
    (Utc::now() + Duration::try_hours(7).unwrap_or_else(|| Duration::hours(7))).date_naive()
}

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
        // VN timezone is UTC+7 (25,200 seconds)
        if let Some(dt) = DateTime::from_timestamp(secs + 25_200, 0) {
            return Some(dt.date_naive());
        }
    }
    None
}

fn parse_single_date_token(token: &str) -> Option<NaiveDate> {
    let mut parts = [""; 4];
    let mut count = 0;
    for part in token.split(['-', '/', '.']) {
        if count < 4 {
            parts[count] = part;
            count += 1;
        } else {
            return None;
        }
    }
    if count == 3 {
        let p0 = parts[0];
        let p1 = parts[1];
        let p2 = parts[2];
        if p0.len() == 4 && (1..=2).contains(&p1.len()) && (1..=2).contains(&p2.len()) {
            if let (Ok(y), Ok(m), Ok(d)) = (
                p0.parse::<i32>(),
                p1.parse::<u32>(),
                p2.parse::<u32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
        if p2.len() == 4 && (1..=2).contains(&p0.len()) && (1..=2).contains(&p1.len()) {
            if let (Ok(d), Ok(m), Ok(y)) = (
                p0.parse::<u32>(),
                p1.parse::<u32>(),
                p2.parse::<i32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
    }
    None
}

pub fn parse_date_str(d_str: &str) -> Option<NaiveDate> {
    let clean = d_str.trim();
    if clean.is_empty() {
        return None;
    }

    // Fast path: standard YYYY-MM-DD (e.g. "2026-11-14 12:00:00" or "2026-11-14")
    if clean.len() >= 10 {
        let b = clean.as_bytes();
        if b[4] == b'-' && b[7] == b'-' {
            if let (Ok(y), Ok(m), Ok(d)) = (
                std::str::from_utf8(&b[0..4]).unwrap_or("").parse::<i32>(),
                std::str::from_utf8(&b[5..7]).unwrap_or("").parse::<u32>(),
                std::str::from_utf8(&b[8..10]).unwrap_or("").parse::<u32>(),
            ) {
                if let Some(date) = NaiveDate::from_ymd_opt(y, m, d) {
                    return Some(date);
                }
            }
        }
    }

    if let Some(d) = parse_single_date_token(clean) {
        return Some(d);
    }

    if let Some(first_word) = clean.split_whitespace().next() {
        if first_word != clean {
            if let Some(d) = parse_single_date_token(first_word) {
                return Some(d);
            }
        }
    }

    if let Ok(d) = NaiveDate::parse_from_str(clean, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d/%m/%Y") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(clean, "%d-%m-%Y") {
        return Some(d);
    }

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
                    (Some(e), Some(s))
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

    let mut item_stats: HashMap<(u32, u32), ItemStat> = HashMap::new();
    let mut battle_items: HashMap<(u32, u32), (&ItemDrop, i64)> = HashMap::new();

    for r in records {
        for it in &r.items {
            battle_items
                .entry((it.itype, it.sub_type))
                .and_modify(|(_, sum)| *sum += it.value)
                .or_insert((it, it.value));
        }

        for (&(itype, sub_type), &(sample_it, battle_qty)) in &battle_items {
            let entry = item_stats.entry((itype, sub_type)).or_insert_with(|| {
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

            let btype = if r.battle_type.is_empty() { "other" } else { r.battle_type.as_str() };
            if let Some(val) = entry.by_battle_type.get_mut(btype) {
                *val += battle_qty;
            } else {
                entry.by_battle_type.insert(btype.to_string(), battle_qty);
            }
        }

        battle_items.clear();
    }

    for stat in item_stats.values_mut() {
        stat.avg_per_battle =
            ((stat.total_quantity as f64) / (total_battles as f64) * 10.0).round() / 10.0;
    }

    let final_item_stats: HashMap<String, ItemStat> = item_stats
        .into_iter()
        .map(|((itype, sub_type), stat)| (format!("{}_{}", itype, sub_type), stat))
        .collect();

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
        item_stats: final_item_stats,
        plunder_breakdown,
    }
}

#[inline]
pub fn calculate_ap_cost_from_lvl(battle_type: &str, lvl: u32) -> i64 {
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

#[allow(dead_code)]
pub fn calculate_ap_cost(battle_type: &str, target_level: &str) -> i64 {
    let lvl: u32 = if let Ok(n) = target_level.trim().parse::<u32>() {
        n
    } else {
        target_level
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
    };

    calculate_ap_cost_from_lvl(battle_type, lvl)
}

#[inline]
pub fn parse_target_level_from_str(target_level: &str, target: &str) -> u32 {
    if let Ok(n) = target_level.trim().parse::<u32>() {
        return n;
    }
    let parsed = target_level
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    if parsed > 0 {
        parsed
    } else {
        target
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
    }
}

pub fn calculate_record_ap_cost(r: &BattleRecord) -> i64 {
    let lvl = parse_target_level_from_str(&r.target_level, &r.target);
    calculate_ap_cost_from_lvl(&r.battle_type, lvl)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DailyApSpeedupStat {
    pub date_label: String,
    pub speedup_hours: f64,
    pub ap_consumed: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TargetRoiBenchmark {
    pub target_name: String,
    pub unit_ap_cost: i64,
    pub battle_count: usize,
    pub avg_speedup_mins: f64,
    pub speedup_mins_per_1k_ap: f64,
    pub covenant_drop_rate: f64,
    pub avg_gems: f64,
    pub tier_badge: String,
    #[serde(default)]
    pub battle_type: String,
    #[serde(default)]
    pub level: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

pub fn compute_ap_economics(active_records: &[BattleRecord], vault_records: &[BattleRecord]) -> ApEconomics {
    let mut total_ap_consumed = 0i64;
    let mut total_ap_recovered = 0i64;
    let mut total_speedup_mins = 0i64;
    let mut total_gems = 0i64;

    struct TargetAccumulator {
        battle_type: String,
        level: u32,
        name: String,
        unit_ap: i64,
        count: usize,
        speedup_mins: i64,
        gems: i64,
        covenant_battles: usize,
    }

    let mut target_map: HashMap<(String, u32), TargetAccumulator> = HashMap::new();

    for r in active_records {
        if r.battle_type == "gathering" {
            continue;
        }

        let lvl = parse_target_level_from_str(&r.target_level, &r.target);
        let ap_cost = calculate_ap_cost_from_lvl(&r.battle_type, lvl);
        total_ap_consumed += ap_cost;
        total_ap_recovered += r.ap_breakdown().total_ap_points();

        let spd = r.total_speedup_minutes();
        total_speedup_mins += spd;

        let gems = r.compute_gems();
        total_gems += gems;

        let has_covenant = r.items.iter().any(|it| it.itype == 2 && it.sub_type == 109);

        let key = (r.battle_type.clone(), lvl);
        let entry = target_map.entry(key).or_insert_with(|| {
            let name = match r.battle_type.as_str() {
                "fort" if lvl > 10 => format!("Fort Lvl {} (KvK)", lvl),
                "fort" if lvl > 0 => format!("Fort Lvl {}", lvl),
                "fort" => "Fort".to_string(),
                "motte" if lvl > 0 => format!("Motte Lvl {}", lvl),
                "motte" => "Motte".to_string(),
                "barbarian" if lvl > 0 => format!("Barbarians Lvl {}", lvl),
                "barbarian" => "Barbarians".to_string(),
                _ => {
                    if lvl > 0 {
                        format!("{} Lvl {}", r.battle_type, lvl)
                    } else if !r.target.is_empty() {
                        r.target.clone()
                    } else {
                        r.battle_type.clone()
                    }
                }
            };
            TargetAccumulator {
                battle_type: r.battle_type.clone(),
                level: lvl,
                name,
                unit_ap: ap_cost,
                count: 0,
                speedup_mins: 0,
                gems: 0,
                covenant_battles: 0,
            }
        });

        entry.count += 1;
        entry.speedup_mins += spd;
        entry.gems += gems;
        if has_covenant {
            entry.covenant_battles += 1;
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

    let today = current_date_vn();
    let min_date = today - Duration::days(6);
    let min_epoch_sec = min_date.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc().timestamp() - 7 * 3600);
    let mut day_stats = [(0i64, 0i64); 7];

    for r in vault_records {
        if r.battle_type == "gathering" {
            continue;
        }
        if let Some(min_sec) = min_epoch_sec {
            if r.time_raw > 0 {
                let r_sec = if r.time_raw > 100_000_000_000 { r.time_raw / 1_000_000 } else { r.time_raw };
                if r_sec < min_sec {
                    continue;
                }
            }
        }
        if let Some(d) = get_record_date_vn(r) {
            if d >= min_date && d <= today {
                let day_idx = (d - min_date).num_days() as usize;
                if day_idx < 7 {
                    day_stats[day_idx].0 += r.total_speedup_minutes();
                    day_stats[day_idx].1 += calculate_record_ap_cost(r);
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

    let mut target_benchmarks: Vec<TargetRoiBenchmark> = target_map
        .into_values()
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
            } else if spd_per_1k >= 700.0 {
                "TIER S"
            } else if spd_per_1k >= 400.0 {
                "TIER A"
            } else {
                "TIER B"
            };

            TargetRoiBenchmark {
                target_name: g.name,
                unit_ap_cost: g.unit_ap,
                battle_count: g.count,
                avg_speedup_mins: avg_speedup,
                speedup_mins_per_1k_ap: spd_per_1k,
                covenant_drop_rate: covenant_rate,
                avg_gems,
                tier_badge: tier_badge.to_string(),
                battle_type: g.battle_type,
                level: g.level,
            }
        })
        .collect();

    target_benchmarks.sort_by(|a, b| {
        b.speedup_mins_per_1k_ap
            .partial_cmp(&a.speedup_mins_per_1k_ap)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.battle_count.cmp(&a.battle_count))
            .then_with(|| b.level.cmp(&a.level))
            .then_with(|| a.target_name.cmp(&b.target_name))
    });

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

pub fn aggregate_troop_combat_stats(records: &[BattleRecord]) -> Vec<TroopCombatStats> {
    let mut map: HashMap<(u32, u32, String, String, String), TroopCombatStats> = HashMap::new();

    for rec in records {
        if rec.battle_type != "pvp" {
            continue;
        }
        let primary_name = if rec.primary_commander_id > 0 {
            crate::classifier::get_commander_name(rec.primary_commander_id)
        } else {
            rec.primary_commander_name.clone()
        };
        let secondary_name = if rec.secondary_commander_id > 0 {
            crate::classifier::get_commander_name(rec.secondary_commander_id)
        } else {
            rec.secondary_commander_name.clone()
        };
        let combat_type = match rec.pvp_combat_type.as_str() {
            "rally" => "rally".to_string(),
            "garrison" => "garrison".to_string(),
            _ => "open-field".to_string(),
        };
        let key = (
            rec.primary_commander_id,
            rec.secondary_commander_id,
            primary_name.clone(),
            secondary_name.clone(),
            combat_type.clone(),
        );
        let entry = map.entry(key).or_insert_with(|| TroopCombatStats {
            primary_commander_id: rec.primary_commander_id,
            primary_commander_name: primary_name,
            secondary_commander_id: rec.secondary_commander_id,
            secondary_commander_name: secondary_name,
            primary_is_expertised: rec.primary_is_expertised,
            secondary_is_expertised: rec.secondary_is_expertised,
            combat_type,
            ..Default::default()
        });
        if rec.primary_is_expertised {
            entry.primary_is_expertised = true;
        }
        if rec.secondary_is_expertised {
            entry.secondary_is_expertised = true;
        }
        entry.kp_gained += rec.kp_gained;
        entry.kp_lost += rec.kp_lost;
        entry.dead_dealt += rec.dead_dealt;
        entry.dead_taken += rec.dead_taken;
        entry.severely_wounded_dealt += rec.severely_wounded_dealt;
        entry.severely_wounded_taken += rec.severely_wounded_taken;
        entry.battle_count += 1;
    }

    let mut list: Vec<TroopCombatStats> = map.into_values().collect();
    list.sort_by(|a, b| {
        b.kills_dealt()
            .cmp(&a.kills_dealt())
            .then_with(|| {
                b.kp_gained
                    .partial_cmp(&a.kp_gained)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| b.battle_count.cmp(&a.battle_count))
    });
    list
}

pub fn estimate_t4_t5_breakdown(total_kills: u64, kp: f64) -> (u64, u64) {
    if total_kills == 0 || kp <= 0.0 {
        return (0, 0);
    }
    let k_f = total_kills as f64;
    let t5_raw = (kp - 10.0 * k_f) / 10.0;
    let t5 = t5_raw.clamp(0.0, k_f).round() as u64;
    let t4 = total_kills.saturating_sub(t5);
    (t4, t5)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PvpTimelinePoint {
    pub label: String,
    pub timestamp: i64,
    pub kills_dealt: u64,
    pub kills_taken: u64,
    pub kills_t4_dealt: u64,
    pub kills_t5_dealt: u64,
    pub kills_t4_taken: u64,
    pub kills_t5_taken: u64,
    pub kp_gained: f64,
    pub kp_lost: f64,
    pub trade_ratio_pct: f64,
    pub battle_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PvpDashboardStats {
    pub total_battles: usize,
    pub kills_dealt: u64,
    pub kills_taken: u64,
    pub kills_t4_dealt: u64,
    pub kills_t5_dealt: u64,
    pub kills_t4_taken: u64,
    pub kills_t5_taken: u64,
    pub kp_gained: f64,
    pub kp_lost: f64,
    pub trade_ratio_pct: f64,
    pub top_march: Option<TroopCombatStats>,
    pub timeline: Vec<PvpTimelinePoint>,
    pub is_hourly: bool,
}

pub fn record_hour_vn(r: &BattleRecord) -> Option<u32> {
    if r.time_vn.len() >= 13 {
        let parts: Vec<&str> = r.time_vn.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(hour_str) = parts[1].split(':').next() {
                if let Ok(h) = hour_str.parse::<u32>() {
                    if h < 24 {
                        return Some(h);
                    }
                }
            }
        }
    }
    if r.time_raw > 0 {
        let secs = if r.time_raw > 100_000_000_000 {
            r.time_raw / 1_000_000
        } else {
            r.time_raw
        };
        DateTime::from_timestamp(secs, 0).map(|dt| {
            let vn_dt = dt + Duration::try_hours(7).unwrap_or_else(|| Duration::hours(7));
            vn_dt.hour()
        })
    } else {
        None
    }
}

pub fn compute_pvp_dashboard_analytics(records: &[BattleRecord], is_single_day: bool) -> PvpDashboardStats {
    let pvp_records: Vec<&BattleRecord> = records
        .iter()
        .filter(|r| r.battle_type == "pvp")
        .collect();

    if pvp_records.is_empty() {
        return PvpDashboardStats {
            is_hourly: is_single_day,
            ..Default::default()
        };
    }

    let total_battles = pvp_records.len();
    let mut total_dealt = 0u64;
    let mut total_taken = 0u64;
    let mut total_kp_gained = 0.0f64;
    let mut total_kp_lost = 0.0f64;

    for r in &pvp_records {
        total_dealt += r.kills_dealt();
        total_taken += r.kills_taken();
        total_kp_gained += r.kp_gained;
        total_kp_lost += r.kp_lost;
    }

    let (kills_t4_dealt, kills_t5_dealt) = estimate_t4_t5_breakdown(total_dealt, total_kp_gained);
    let (kills_t4_taken, kills_t5_taken) = estimate_t4_t5_breakdown(total_taken, total_kp_lost);

    let overall_trade_ratio_pct = if total_taken == 0 {
        if total_kp_lost > 0.0 {
            ((total_kp_gained / total_kp_lost) * 100.0 * 10.0).round() / 10.0
        } else if total_dealt > 0 || total_kp_gained > 0.0 {
            100.0
        } else {
            0.0
        }
    } else {
        ((total_dealt as f64 / total_taken as f64) * 100.0 * 10.0).round() / 10.0
    };

    let pvp_cloned: Vec<BattleRecord> = pvp_records.iter().map(|&r| r.clone()).collect();
    let troop_trades = aggregate_troop_combat_stats(&pvp_cloned);
    let top_march = troop_trades.into_iter().next();

    let mut timeline: Vec<PvpTimelinePoint> = Vec::new();

    if is_single_day {
        struct HourBucket {
            min_ts: i64,
            kills_dealt: u64,
            kills_taken: u64,
            kp_gained: f64,
            kp_lost: f64,
            battle_count: usize,
        }
        let mut hour_map: HashMap<u32, HourBucket> = HashMap::new();

        for r in &pvp_records {
            let h = record_hour_vn(r).unwrap_or(0);
            let ts = if r.time_raw > 100_000_000_000 {
                r.time_raw / 1_000_000
            } else {
                r.time_raw
            };
            let entry = hour_map.entry(h).or_insert_with(|| HourBucket {
                min_ts: ts,
                kills_dealt: 0,
                kills_taken: 0,
                kp_gained: 0.0,
                kp_lost: 0.0,
                battle_count: 0,
            });
            entry.kills_dealt += r.kills_dealt();
            entry.kills_taken += r.kills_taken();
            entry.kp_gained += r.kp_gained;
            entry.kp_lost += r.kp_lost;
            entry.battle_count += 1;
            if ts > 0 && (entry.min_ts == 0 || ts < entry.min_ts) {
                entry.min_ts = ts;
            }
        }

        let hours: Vec<u32> = (0..24).collect();

        for h in hours {
            if let Some(b) = hour_map.get(&h) {
                let (t4_d, t5_d) = estimate_t4_t5_breakdown(b.kills_dealt, b.kp_gained);
                let (t4_t, t5_t) = estimate_t4_t5_breakdown(b.kills_taken, b.kp_lost);
                let ratio = if b.kills_taken == 0 {
                    if b.kp_lost > 0.0 {
                        ((b.kp_gained / b.kp_lost) * 100.0 * 10.0).round() / 10.0
                    } else if b.kills_dealt > 0 || b.kp_gained > 0.0 {
                        100.0
                    } else {
                        0.0
                    }
                } else {
                    ((b.kills_dealt as f64 / b.kills_taken as f64) * 100.0 * 10.0).round() / 10.0
                };
                timeline.push(PvpTimelinePoint {
                    label: format!("{:02}:00", h),
                    timestamp: b.min_ts,
                    kills_dealt: b.kills_dealt,
                    kills_taken: b.kills_taken,
                    kills_t4_dealt: t4_d,
                    kills_t5_dealt: t5_d,
                    kills_t4_taken: t4_t,
                    kills_t5_taken: t5_t,
                    kp_gained: b.kp_gained,
                    kp_lost: b.kp_lost,
                    trade_ratio_pct: ratio,
                    battle_count: b.battle_count,
                });
            } else {
                timeline.push(PvpTimelinePoint {
                    label: format!("{:02}:00", h),
                    ..Default::default()
                });
            }
        }
    } else {
        struct DayBucket {
            min_ts: i64,
            kills_dealt: u64,
            kills_taken: u64,
            kp_gained: f64,
            kp_lost: f64,
            battle_count: usize,
        }
        let mut day_map: HashMap<NaiveDate, DayBucket> = HashMap::new();

        for r in &pvp_records {
            let d = get_record_date_vn(r).unwrap_or_else(current_date_vn);
            let ts = if r.time_raw > 100_000_000_000 {
                r.time_raw / 1_000_000
            } else {
                r.time_raw
            };
            let entry = day_map.entry(d).or_insert_with(|| DayBucket {
                min_ts: ts,
                kills_dealt: 0,
                kills_taken: 0,
                kp_gained: 0.0,
                kp_lost: 0.0,
                battle_count: 0,
            });
            entry.kills_dealt += r.kills_dealt();
            entry.kills_taken += r.kills_taken();
            entry.kp_gained += r.kp_gained;
            entry.kp_lost += r.kp_lost;
            entry.battle_count += 1;
            if ts > 0 && (entry.min_ts == 0 || ts < entry.min_ts) {
                entry.min_ts = ts;
            }
        }

        let mut dates: Vec<NaiveDate> = day_map.keys().copied().collect();
        dates.sort_unstable();

        if let (Some(first_date), Some(last_date)) = (dates.first().copied(), dates.last().copied()) {
            let day_count = (last_date - first_date).num_days();
            for day_offset in 0..=day_count {
                let d = first_date + Duration::days(day_offset);
                if let Some(b) = day_map.get(&d) {
                    let (t4_d, t5_d) = estimate_t4_t5_breakdown(b.kills_dealt, b.kp_gained);
                    let (t4_t, t5_t) = estimate_t4_t5_breakdown(b.kills_taken, b.kp_lost);
                    let ratio = if b.kills_taken == 0 {
                        if b.kp_lost > 0.0 {
                            ((b.kp_gained / b.kp_lost) * 100.0 * 10.0).round() / 10.0
                        } else if b.kills_dealt > 0 || b.kp_gained > 0.0 {
                            100.0
                        } else {
                            0.0
                        }
                    } else {
                        ((b.kills_dealt as f64 / b.kills_taken as f64) * 100.0 * 10.0).round()
                            / 10.0
                    };
                    timeline.push(PvpTimelinePoint {
                        label: format!("{:02}/{:02}", d.month(), d.day()),
                        timestamp: b.min_ts,
                        kills_dealt: b.kills_dealt,
                        kills_taken: b.kills_taken,
                        kills_t4_dealt: t4_d,
                        kills_t5_dealt: t5_d,
                        kills_t4_taken: t4_t,
                        kills_t5_taken: t5_t,
                        kp_gained: b.kp_gained,
                        kp_lost: b.kp_lost,
                        trade_ratio_pct: ratio,
                        battle_count: b.battle_count,
                    });
                } else {
                    timeline.push(PvpTimelinePoint {
                        label: format!("{:02}/{:02}", d.month(), d.day()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    PvpDashboardStats {
        total_battles,
        kills_dealt: total_dealt,
        kills_taken: total_taken,
        kills_t4_dealt,
        kills_t5_dealt,
        kills_t4_taken,
        kills_t5_taken,
        kp_gained: total_kp_gained,
        kp_lost: total_kp_lost,
        trade_ratio_pct: overall_trade_ratio_pct,
        top_march,
        timeline,
        is_hourly: is_single_day,
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
        let s = "123456789ư";
        assert_eq!(parse_date_str(s), None);

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
            ..Default::default()
        };

        let recs = vec![
            make_rec("2026-10-18 10:00:00", 0),
            make_rec("2026-10-20 12:00:00", 0),
            make_rec("2026-10-22 08:00:00", 0),
        ];

        let d18 = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
        let d20 = NaiveDate::from_ymd_opt(2026, 10, 20).unwrap();
        let d22 = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();

        let f20 = filter_records_by_date(&recs, Some(d20), Some(d20), None);
        assert_eq!(f20.len(), 1);
        assert_eq!(f20[0].time_vn, "2026-10-20 12:00:00");

        let f18_20 = filter_records_by_date(&recs, Some(d18), Some(d20), None);
        assert_eq!(f18_20.len(), 2);

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
            ..Default::default()
        };

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

        assert_eq!(latest[0].mail_id, "new_5");
        assert_eq!(latest[1].mail_id, "new_4");
        assert_eq!(latest[2].mail_id, "new_3");
        assert_eq!(latest[3].mail_id, "new_2");
        assert_eq!(latest[4].mail_id, "new_1");

        assert!(latest.iter().all(|r| r.mail_id != "old_2022" && r.mail_id != "old_2025"));
    }

    #[test]
    fn test_chained_barbarian_item_aggregation_no_drop_rate_inflation() {
        let mut items = Vec::new();
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
            ..Default::default()
        };

        let stats = generate_statistics(&[record]);
        let gem_stat = stats.item_stats.get("1_5").expect("Must have Gems stats");

        assert_eq!(gem_stat.battle_count, 1, "Battle count must be 1 per battle even if chained kills drop same item");
        assert_eq!(gem_stat.total_quantity, 40);
        assert_eq!(gem_stat.max_in_battle, 40);
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
            ..Default::default()
        };

        let rec_today = make_rec(today);
        let rec_yest = make_rec(yest);
        let recs = vec![rec_today.clone(), rec_yest.clone()];

        let filtered_today = filter_records_by_date(&recs, None, None, Some(1));
        assert_eq!(filtered_today.len(), 1);
        assert_eq!(filtered_today[0].time_vn, rec_today.time_vn);

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
            ..Default::default()
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
            ..Default::default()
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
                make_item(1, 9, 100),
                make_item(2, 7005, 4),
                make_item(2, 130, 10),
                make_item(2, 92, 5),
                make_item(2, 58, 15),
                make_item(2, 9998, 2),
            ],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
                items.push(mk_item(2, 58, spd_5m));
            }
            if ap_50 > 0 {
                items.push(mk_item(2, 182, ap_50));
            }
            if covenant > 0 {
                items.push(mk_item(2, 109, covenant));
            }
            if gems_10 > 0 {
                items.push(mk_item(2, 26, gems_10));
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
            ..Default::default()
            }
        };

        let r1 = make_battle("fort", "5", 10, 1, 1, 3);
        let r2 = make_battle("fort", "5", 10, 1, 0, 2);

        let r3 = make_battle("barbarian", "25", 2, 0, 0, 1);

        let records = vec![r1, r2, r3];
        let econ = compute_ap_economics(&records, &records);

        assert_eq!(econ.total_ap_consumed, 340);
        assert_eq!(econ.total_ap_recovered, 100);
        assert_eq!(econ.net_ap_spent, 240);
        assert!((econ.rebate_pct - 29.41).abs() < 0.1);

        assert!((econ.speedup_mins_per_1k_ap - 323.5).abs() < 0.2);

        assert!((econ.gems_per_1k_ap - 176.47).abs() < 0.2);

        assert_eq!(econ.target_benchmarks.len(), 2);
        let fort_bm = econ.target_benchmarks.iter().find(|b| b.target_name == "Fort Lvl 5").unwrap();
        assert_eq!(fort_bm.battle_count, 2);
        assert_eq!(fort_bm.unit_ap_cost, 150);
        assert_eq!(fort_bm.avg_speedup_mins, 50.0);
        assert_eq!(fort_bm.covenant_drop_rate, 50.0);
        assert_eq!(fort_bm.battle_type, "fort");
        assert_eq!(fort_bm.level, 5);
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
                    itype: 1,
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
            ..Default::default()
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

    #[test]
    fn test_individual_level_target_benchmarks() {
        let m10 = BattleRecord {
            battle_type: "motte".into(),
            target_level: "10".into(),
            ..Default::default()
        };
        let m15 = BattleRecord {
            battle_type: "motte".into(),
            target_level: "15".into(),
            ..Default::default()
        };
        let f9 = BattleRecord {
            battle_type: "fort".into(),
            target_level: "9".into(),
            ..Default::default()
        };
        let b50 = BattleRecord {
            battle_type: "barbarian".into(),
            target_level: "50".into(),
            ..Default::default()
        };

        let records = vec![m10, m15, f9, b50];
        let econ = compute_ap_economics(&records, &records);

        assert_eq!(econ.target_benchmarks.len(), 4);
        assert!(econ.target_benchmarks.iter().any(|b| b.target_name == "Motte Lvl 10" && b.level == 10 && b.unit_ap_cost == 150));
        assert!(econ.target_benchmarks.iter().any(|b| b.target_name == "Motte Lvl 15" && b.level == 15 && b.unit_ap_cost == 300));
        assert!(econ.target_benchmarks.iter().any(|b| b.target_name == "Fort Lvl 9" && b.level == 9 && b.unit_ap_cost == 150));
        assert!(econ.target_benchmarks.iter().any(|b| b.target_name == "Barbarians Lvl 50" && b.level == 50 && b.unit_ap_cost == 80));
    }

    #[test]
    fn test_calculate_record_ap_cost_fallback_to_target() {
        let r1 = BattleRecord {
            battle_type: "motte".into(),
            target: "Motte Lvl 15".into(),
            target_level: "".into(),
            ..Default::default()
        };
        assert_eq!(calculate_record_ap_cost(&r1), 300);

        let r2 = BattleRecord {
            battle_type: "fort".into(),
            target: "Pháo đài man rợ cấp 12".into(),
            target_level: "".into(),
            ..Default::default()
        };
        assert_eq!(calculate_record_ap_cost(&r2), 300);

        let econ = compute_ap_economics(&[r1.clone()], &[r1]);
        assert_eq!(econ.total_ap_consumed, 300);
        assert_eq!(econ.target_benchmarks[0].unit_ap_cost, 300);
        assert_eq!(econ.target_benchmarks[0].level, 15);
    }

    #[test]
    fn test_aggregate_troop_combat_stats() {
        let r1 = BattleRecord {
            battle_type: "pvp".into(),
            primary_commander_id: 198,
            primary_commander_name: "Liu Che".into(),
            secondary_commander_id: 140,
            secondary_commander_name: "Joan Prime".into(),
            kp_gained: 200_000.0,
            kp_lost: 100_000.0,
            dead_dealt: 500,
            dead_taken: 100,
            ..Default::default()
        };
        let r2 = BattleRecord {
            battle_type: "pvp".into(),
            primary_commander_id: 198,
            primary_commander_name: "Liu Che".into(),
            secondary_commander_id: 140,
            secondary_commander_name: "Joan Prime".into(),
            kp_gained: 100_000.0,
            kp_lost: 50_000.0,
            dead_dealt: 200,
            dead_taken: 50,
            ..Default::default()
        };
        let r3 = BattleRecord {
            battle_type: "pvp".into(),
            primary_commander_id: 595,
            primary_commander_name: "Achilles".into(),
            secondary_commander_id: 545,
            secondary_commander_name: "Arthur".into(),
            kp_gained: 500_000.0,
            kp_lost: 400_000.0,
            dead_dealt: 2000,
            dead_taken: 1600,
            ..Default::default()
        };
        let r_barb = BattleRecord {
            battle_type: "barbarian".into(),
            kp_gained: 999_999.0,
            ..Default::default()
        };

        let stats = aggregate_troop_combat_stats(&[r1, r2, r3, r_barb]);
        assert_eq!(stats.len(), 2);

        // First should be 595 / 545 -> Sun Tzu (P) / Bai Qi (2000 kills dealt)
        assert_eq!(stats[0].display_pair_name(), "Sun Tzu (P) / Bai Qi");
        assert_eq!(stats[0].kills_dealt(), 2000);
        assert_eq!(stats[0].kills_taken(), 1600);
        assert_eq!(stats[0].kp_gained, 500_000.0);
        assert_eq!(stats[0].kp_lost, 400_000.0);
        assert_eq!(stats[0].trade_ratio_pct(), 125.0);
        assert_eq!(stats[0].battle_count, 1);

        // Second should be 198 / 140 -> Ragnar Lodbrok (P) / Scipio Africanus (P) (700 kills dealt across 2 battles)
        assert_eq!(stats[1].display_pair_name(), "Ragnar Lodbrok (P) / Scipio Africanus (P)");
        assert_eq!(stats[1].kills_dealt(), 700);
        assert_eq!(stats[1].kills_taken(), 150);
        assert_eq!(stats[1].kp_gained, 300_000.0);
        assert_eq!(stats[1].kp_lost, 150_000.0);
        assert_eq!(stats[1].trade_ratio_pct(), 466.7);
        assert_eq!(stats[1].dead_dealt, 700);
        assert_eq!(stats[1].dead_taken, 150);
        assert_eq!(stats[1].battle_count, 2);
    }

    #[test]
    fn test_estimate_t4_t5_breakdown() {
        // Zero kills
        assert_eq!(estimate_t4_t5_breakdown(0, 0.0), (0, 0));
        assert_eq!(estimate_t4_t5_breakdown(100, 0.0), (0, 0));

        // All T4: 1000 kills * 10 KP = 10,000 KP
        let (t4, t5) = estimate_t4_t5_breakdown(1000, 10_000.0);
        assert_eq!(t4, 1000);
        assert_eq!(t5, 0);

        // All T5: 1000 kills * 20 KP = 20,000 KP
        let (t4, t5) = estimate_t4_t5_breakdown(1000, 20_000.0);
        assert_eq!(t4, 0);
        assert_eq!(t5, 1000);

        // 50/50 mix: 500 T4 (5,000 KP) + 500 T5 (10,000 KP) = 15,000 KP
        let (t4, t5) = estimate_t4_t5_breakdown(1000, 15_000.0);
        assert_eq!(t4, 500);
        assert_eq!(t5, 500);

        // Clamping lower bound (T1-T3 kills or low KP)
        let (t4, t5) = estimate_t4_t5_breakdown(1000, 5_000.0);
        assert_eq!(t4, 1000);
        assert_eq!(t5, 0);

        // Clamping upper bound (super high KP)
        let (t4, t5) = estimate_t4_t5_breakdown(1000, 25_000.0);
        assert_eq!(t4, 0);
        assert_eq!(t5, 1000);
    }

    #[test]
    fn test_compute_pvp_dashboard_analytics_hourly_and_daily() {
        let r1 = BattleRecord {
            battle_type: "pvp".into(),
            time_vn: "2026-11-13 09:15:00".into(),
            time_raw: 1794561300000000,
            primary_commander_id: 198,
            primary_commander_name: "Liu Che".into(),
            secondary_commander_id: 140,
            secondary_commander_name: "Joan Prime".into(),
            kp_gained: 150_000.0,
            kp_lost: 100_000.0,
            dead_dealt: 500,
            severely_wounded_dealt: 9500, // 10,000 kills dealt (150k KP -> 5000 T4, 5000 T5)
            dead_taken: 200,
            severely_wounded_taken: 4800, // 5,000 kills taken
            ..Default::default()
        };

        let r2 = BattleRecord {
            battle_type: "pvp".into(),
            time_vn: "2026-11-13 09:45:00".into(),
            time_raw: 1794563100000000,
            primary_commander_id: 198,
            primary_commander_name: "Liu Che".into(),
            secondary_commander_id: 140,
            secondary_commander_name: "Joan Prime".into(),
            kp_gained: 200_000.0,
            kp_lost: 50_000.0,
            dead_dealt: 1000,
            severely_wounded_dealt: 9000, // 10,000 kills dealt
            dead_taken: 100,
            severely_wounded_taken: 1900, // 2,000 kills taken
            ..Default::default()
        };

        let r3 = BattleRecord {
            battle_type: "pvp".into(),
            time_vn: "2026-11-13 14:20:00".into(),
            time_raw: 1794579600000000,
            primary_commander_id: 595,
            primary_commander_name: "Sun Tzu (P)".into(),
            secondary_commander_id: 545,
            secondary_commander_name: "Bai Qi".into(),
            kp_gained: 50_000.0,
            kp_lost: 25_000.0,
            dead_dealt: 500,
            severely_wounded_dealt: 2000, // 2500 kills dealt
            dead_taken: 100,
            severely_wounded_taken: 900,  // 1000 kills taken
            ..Default::default()
        };

        let r_pve = BattleRecord {
            battle_type: "barbarian".into(),
            time_vn: "2026-11-13 09:30:00".into(),
            ..Default::default()
        };
        let r4 = BattleRecord {
            battle_type: "pvp".into(),
            time_vn: "2026-11-15 09:00:00".into(),
            ..Default::default()
        };

        // Test hourly single-day analytics
        let hourly = compute_pvp_dashboard_analytics(&[r1.clone(), r2.clone(), r3.clone(), r_pve.clone()], true);
        assert_eq!(hourly.total_battles, 3);
        assert_eq!(hourly.kills_dealt, 22_500);
        assert_eq!(hourly.kills_taken, 8_000);
        assert_eq!(hourly.kp_gained, 400_000.0);
        assert_eq!(hourly.kp_lost, 175_000.0);
        assert!(hourly.is_hourly);
        assert!(hourly.top_march.is_some());
        assert_eq!(hourly.timeline.len(), 24);
        assert_eq!(hourly.timeline[0].label, "00:00");
        assert_eq!(hourly.timeline[9].label, "09:00");
        assert_eq!(hourly.timeline[9].battle_count, 2);
        assert_eq!(hourly.timeline[9].kills_dealt, 20_000);
        assert_eq!(hourly.timeline[10].battle_count, 0);
        assert_eq!(hourly.timeline[10].kills_dealt, 0);
        assert_eq!(hourly.timeline[14].label, "14:00");
        assert_eq!(hourly.timeline[14].battle_count, 1);
        assert_eq!(hourly.timeline[14].kills_dealt, 2_500);

        // Test daily multi-day analytics
        let daily = compute_pvp_dashboard_analytics(&[r1, r2, r3, r4, r_pve], false);
        assert_eq!(daily.total_battles, 4);
        assert!(!daily.is_hourly);
        assert_eq!(daily.timeline.len(), 3);
        assert_eq!(daily.timeline[0].battle_count, 3);
        assert_eq!(daily.timeline[0].kills_dealt, 22_500);
        assert_eq!(daily.timeline[1].label, "11/14");
        assert_eq!(daily.timeline[1].battle_count, 0);
        assert_eq!(daily.timeline[1].kills_dealt, 0);
        assert_eq!(daily.timeline[2].label, "11/15");
        assert_eq!(daily.timeline[2].battle_count, 1);
    }
}
