use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use crate::analyzer::{compute_ap_economics, current_date_vn, parse_date_str};
use crate::model::{
    format_num, format_speedup_compact, get_item_conversion, get_item_converted_display_for_lang,
    AppConfig, AppLanguage, AppTheme, BattleRecord, CloseBehavior, ConvertedValue,
    DateFilterPreset, FilterMode, ItemStat, PinnedItemKey, SpeedupUnit, Statistics,
};
use crate::tauri_app::dto::*;
use crate::watcher::RoKMailWatcher;

pub struct TrayMenuItems {
    pub show: tauri::menu::MenuItem<tauri::Wry>,
    pub rescan: tauri::menu::MenuItem<tauri::Wry>,
    pub toggle: tauri::menu::MenuItem<tauri::Wry>,
    pub quit: tauri::menu::MenuItem<tauri::Wry>,
}

pub fn tray_menu_labels(
    lang: AppLanguage,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match lang {
        AppLanguage::Vietnamese => (
            "Mở cửa sổ chính",
            "Quét lại thư",
            "Tạm dừng / Tiếp tục",
            "Thoát ứng dụng",
        ),
        AppLanguage::English => (
            "Open Main Window",
            "Rescan Mail",
            "Pause / Resume",
            "Exit Application",
        ),
    }
}

pub struct TauriAppState {
    pub watcher: Arc<RoKMailWatcher>,
    pub config: Arc<std::sync::Mutex<AppConfig>>,
    pub sync_worker: crate::cloud_sync::SyncWorkerHandle,
    pub cloud_login_busy: Arc<AtomicBool>,
    pub last_mail_arrival: Arc<std::sync::Mutex<std::time::Instant>>,
    pub tray_items: std::sync::Mutex<Option<TrayMenuItems>>,
    pub selected_mode: std::sync::Mutex<FilterMode>,
    pub selected_date_preset: std::sync::Mutex<DateFilterPreset>,
    pub date_from: std::sync::Mutex<String>,
    pub date_to: std::sync::Mutex<String>,
    pub selected_account: std::sync::Mutex<Option<String>>,
    pub speedup_unit: std::sync::Mutex<SpeedupUnit>,
    pub single_instance: std::sync::Mutex<Option<crate::single_instance::windows::SingleInstanceGuard>>,
}

impl TauriAppState {
    pub fn idle_secs(&self) -> u64 {
        self.last_mail_arrival
            .lock()
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn selected_account_clone(&self) -> Option<String> {
        self.selected_account
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

pub fn reconcile_sync_worker_config(
    sync_worker: &crate::cloud_sync::SyncWorkerHandle,
    config_mutex: &std::sync::Mutex<AppConfig>,
) {
    let mut rotated_tokens: Option<(String, String)> = None;
    let mut worker_last_sync: Option<String> = None;

    if let Ok(mut s) = sync_worker.state.write() {
        rotated_tokens = s.new_tokens.take();
        worker_last_sync = s.last_sync_time.clone();
    }

    if rotated_tokens.is_some() || worker_last_sync.is_some() {
        if let Ok(mut cfg) = config_mutex.lock() {
            let mut changed = false;
            if let Some((acc, ref_tok)) = rotated_tokens {
                cfg.cloud_auth.access_token = Some(acc);
                cfg.cloud_auth.refresh_token = Some(ref_tok);
                changed = true;
            }
            if let Some(ts) = worker_last_sync {
                if cfg.cloud_auth.last_synced.as_deref() != Some(ts.as_str()) {
                    cfg.cloud_auth.last_synced = Some(ts);
                    changed = true;
                }
            }
            if changed {
                cfg.save();
            }
        }
    }
}

pub fn get_cloud_status_from_parts(
    watcher: &RoKMailWatcher,
    cfg: &AppConfig,
    sync_worker: &crate::cloud_sync::SyncWorkerHandle,
    login_in_progress: bool,
) -> CloudStatusDto {
    let auth = &cfg.cloud_auth;
    let logged_in = auth.is_logged_in();
    let email = auth.user_email.clone();

    let (is_syncing, is_restoring, worker_last_sync, sync_error) = sync_worker
        .state
        .read()
        .map(|s| {
            (
                s.is_syncing,
                s.is_restoring,
                s.last_sync_time.clone(),
                s.last_error.clone(),
            )
        })
        .unwrap_or((false, false, None, None));

    let last_sync_time = worker_last_sync.or_else(|| auth.last_synced.clone());
    let pending = watcher
        .vault
        .read()
        .ok()
        .and_then(|v| v.get_sync_counts().ok())
        .map(|(_, unsynced)| unsynced)
        .unwrap_or(0);

    CloudStatusDto {
        logged_in,
        email,
        last_sync_time,
        pending_count: pending,
        is_syncing,
        is_restoring,
        sync_error,
        auto_sync: auth.auto_backup,
        backup_interval_minutes: auth.clamped_interval_minutes(),
        backup_on_exit: auth.backup_on_exit,
        stay_login: auth.stay_login,
        login_in_progress,
    }
}

pub fn get_cloud_status_for_state(state: &TauriAppState) -> CloudStatusDto {
    reconcile_sync_worker_config(&state.sync_worker, &state.config);
    let cfg = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let login_busy = state.cloud_login_busy.load(Ordering::SeqCst);
    get_cloud_status_from_parts(&state.watcher, &cfg, &state.sync_worker, login_busy)
}

pub fn resolve_date_filter(
    preset: DateFilterPreset,
    date_from: &str,
    date_to: &str,
) -> (
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
    Option<u32>,
    String,
    String,
) {
    match preset {
        DateFilterPreset::Yesterday => {
            let yest = current_date_vn() - chrono::Duration::days(1);
            (Some(yest), Some(yest), None, String::new(), String::new())
        }
        DateFilterPreset::Custom => {
            let clean_from = date_from.trim();
            let clean_to = date_to.trim();
            let s = if !clean_from.is_empty() {
                parse_date_str(clean_from)
            } else {
                None
            };
            let e = if !clean_to.is_empty() {
                parse_date_str(clean_to)
            } else {
                None
            };
            (s, e, None, clean_from.to_string(), clean_to.to_string())
        }
        other => (None, None, other.to_days(), String::new(), String::new()),
    }
}

pub fn sum_pinned_catalog_item(
    rec: &BattleRecord,
    canonical_name: &str,
    itype: u32,
    isub: u32,
) -> i64 {
    match canonical_name {
        "Gems" => {
            if !rec.items.is_empty() {
                let converted = rec.compute_gems();
                let extra_named: i64 = rec
                    .items
                    .iter()
                    .filter(|it| {
                        matches!(get_item_conversion(it.itype, it.sub_type), ConvertedValue::Other)
                            && (it.name_en.eq_ignore_ascii_case("Gems")
                                || it.name_vi.eq_ignore_ascii_case("Đá quý")
                                || it.name_vi == "Đá Quý")
                    })
                    .map(|it| it.value)
                    .sum();
                converted + extra_named
            } else {
                rec.item_totals
                    .iter()
                    .filter(|(k, _)| {
                        k.eq_ignore_ascii_case("Gems")
                            || k.as_str() == "Đá quý"
                            || k.as_str() == "Đá Quý"
                    })
                    .map(|(_, &v)| v)
                    .sum()
            }
        }
        "Crystals" => rec.crystal_quantity(),
        "Book of Covenant" => {
            if !rec.items.is_empty() {
                rec.items
                    .iter()
                    .filter(|it| {
                        (it.itype == 2 && it.sub_type == 109)
                            || it.name_en.eq_ignore_ascii_case("Book of Covenant")
                            || it.name_vi.contains("Giao Ước")
                            || it.name_vi.contains("giao ước")
                            || it.name_vi.contains("Ước Nguyện")
                            || it.name_vi.contains("ước nguyện")
                    })
                    .map(|it| it.value)
                    .sum()
            } else {
                rec.item_totals
                    .iter()
                    .filter(|(k, _)| {
                        k.eq_ignore_ascii_case("Book of Covenant")
                            || k.contains("Giao Ước")
                            || k.contains("giao ước")
                            || k.contains("Ước Nguyện")
                            || k.contains("ước nguyện")
                    })
                    .map(|(_, &v)| v)
                    .sum()
            }
        }
        "Arrow of Resistance" => {
            if !rec.items.is_empty() {
                rec.items
                    .iter()
                    .filter(|it| {
                        (it.itype == 2 && it.sub_type == 145)
                            || it.name_en.eq_ignore_ascii_case("Arrow of Resistance")
                            || it.name_vi.contains("Kháng Cự")
                            || it.name_vi.contains("kháng cự")
                            || it.name_vi.contains("Chống Trả")
                            || it.name_vi.contains("chống trả")
                    })
                    .map(|it| it.value)
                    .sum()
            } else {
                rec.item_totals
                    .iter()
                    .filter(|(k, _)| {
                        k.eq_ignore_ascii_case("Arrow of Resistance")
                            || k.contains("Kháng Cự")
                            || k.contains("kháng cự")
                            || k.contains("Chống Trả")
                            || k.contains("chống trả")
                    })
                    .map(|(_, &v)| v)
                    .sum()
            }
        }
        _ => {
            if !rec.items.is_empty() {
                rec.items
                    .iter()
                    .filter(|it| {
                        (it.itype == itype && it.sub_type == isub)
                            || it.name_en.eq_ignore_ascii_case(canonical_name)
                            || it.name_vi.eq_ignore_ascii_case(canonical_name)
                    })
                    .map(|it| it.value)
                    .sum()
            } else {
                rec.item_totals
                    .iter()
                    .filter(|(k, _)| k.eq_ignore_ascii_case(canonical_name))
                    .map(|(_, &v)| v)
                    .sum()
            }
        }
    }
}

fn matches_account_filter(
    rec: &BattleRecord,
    main_uids: &[String],
    selected_account: Option<&str>,
) -> bool {
    let clean_rec = rec
        .account_id
        .strip_prefix("player_")
        .unwrap_or(&rec.account_id)
        .trim();
    if !main_uids.is_empty() {
        let is_main = !clean_rec.is_empty()
            && main_uids.iter().any(|u| {
                let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                clean_u == clean_rec
            });
        if !is_main {
            return false;
        }
    }
    if let Some(sel) = selected_account {
        let clean_sel = sel.strip_prefix("player_").unwrap_or(sel).trim();
        if !clean_sel.is_empty() {
            return clean_rec == clean_sel;
        }
    }
    true
}

pub fn build_pinned_chips(
    records: &[BattleRecord],
    pinned_items: &[PinnedItemKey],
    accent_color_hex: &str,
) -> Vec<PinnedChipDto> {
    let mut chips = Vec::with_capacity(pinned_items.len());
    for item_key in pinned_items {
        match item_key {
            PinnedItemKey::SpeedupUniversal => {
                let total_mins: i64 = records.iter().map(|r| r.univ_speedup_minutes()).sum();
                chips.push(PinnedChipDto {
                    id: "SpeedupUniversal".to_string(),
                    label: "SPEEDUP".to_string(),
                    display_val: format_speedup_compact(total_mins),
                    color_hex: "#38bdf8".to_string(),
                });
            }
            PinnedItemKey::SpeedupTraining => {
                let total_mins: i64 = records.iter().map(|r| r.training_speedup_minutes()).sum();
                chips.push(PinnedChipDto {
                    id: "SpeedupTraining".to_string(),
                    label: "TRAIN".to_string(),
                    display_val: format_speedup_compact(total_mins),
                    color_hex: "#fb923c".to_string(),
                });
            }
            PinnedItemKey::SpeedupHealing => {
                let total_mins: i64 = records.iter().map(|r| r.heal_speedup_minutes()).sum();
                chips.push(PinnedChipDto {
                    id: "SpeedupHealing".to_string(),
                    label: "HEAL".to_string(),
                    display_val: format_speedup_compact(total_mins),
                    color_hex: "#34d399".to_string(),
                });
            }
            PinnedItemKey::CatalogItem {
                itype,
                isub,
                canonical_name,
            } => {
                let total: i64 = records
                    .iter()
                    .map(|r| sum_pinned_catalog_item(r, canonical_name, *itype, *isub))
                    .sum();
                let label = match canonical_name.as_str() {
                    "Gems" => "GEMS".to_string(),
                    "Crystals" => "CRYSTALS".to_string(),
                    "Book of Covenant" => "BOOKS".to_string(),
                    "Arrow of Resistance" => "ARROWS".to_string(),
                    other => other.to_uppercase().chars().take(8).collect(),
                };
                let color_hex = match canonical_name.as_str() {
                    "Gems" => "#22d3ee".to_string(),
                    "Crystals" => "#38bdf8".to_string(),
                    "Book of Covenant" => "#fbbf24".to_string(),
                    "Arrow of Resistance" => "#f87171".to_string(),
                    _ => accent_color_hex.to_string(),
                };
                chips.push(PinnedChipDto {
                    id: format!("Catalog:{}:{}:{}", itype, isub, canonical_name),
                    label,
                    display_val: format_num(total),
                    color_hex,
                });
            }
        }
    }
    chips
}

pub fn get_overlay_payload_internal(
    watcher: &RoKMailWatcher,
    cfg: &AppConfig,
    selected_account: Option<&str>,
    idle_secs: u64,
) -> OverlayPayloadDto {
    let overlay_cfg = cfg.overlay.clone();
    let main_uids = watcher.get_main_account_uids();

    let today_records: Vec<BattleRecord> = watcher
        .get_today_vault_records(overlay_cfg.filter)
        .into_iter()
        .filter(|r| matches_account_filter(r, &main_uids, selected_account))
        .collect();

    let recent_source: Vec<BattleRecord> = if !today_records.is_empty() {
        today_records.clone()
    } else {
        watcher
            .get_latest_vault_records(overlay_cfg.filter, 50)
            .into_iter()
            .filter(|r| matches_account_filter(r, &main_uids, selected_account))
            .collect()
    };

    let recent = crate::analyzer::get_latest_records(&recent_source, 5);
    let lang = cfg.language;

    let mut motte = 0;
    let mut fort = 0;
    let mut barb = 0;
    let mut gather = 0;

    for r in today_records.iter() {
        match r.battle_type.as_str() {
            "motte" => motte += 1,
            "fort" => fort += 1,
            "barbarian" => barb += 1,
            "gathering" => gather += 1,
            _ => {}
        }
    }

    let last_mail_timestamp = recent.first().map(|r| r.time_vn.clone());
    let records_dto: Vec<BattleRecordDto> = recent
        .iter()
        .map(|r| BattleRecordDto::from_record(r, lang))
        .collect();

    let pinned_chips = build_pinned_chips(
        &today_records,
        &overlay_cfg.pinned_items,
        &overlay_cfg.accent_color_hex,
    );
    let rok_detected = crate::overlay_tracker::find_rok_window().is_some();

    OverlayPayloadDto {
        records: records_dto,
        config: overlay_cfg,
        pinned_chips,
        today_runs: TodayRunsDto {
            motte,
            fort,
            barb,
            gather,
            total: motte + fort + barb + gather,
        },
        last_mail_timestamp,
        rok_window_detected: rok_detected,
        idle_secs,
    }
}

pub fn query_battles_from_slice(
    records: &[BattleRecord],
    lang: AppLanguage,
    search: &str,
    sort_col: Option<&str>,
    sort_dir: Option<&str>,
) -> Vec<BattleRecordDto> {
    let q = search.trim().to_lowercase();
    let mut filtered: Vec<&BattleRecord> = if q.is_empty() {
        records.iter().collect()
    } else {
        records
            .iter()
            .filter(|r| {
                r.target.to_lowercase().contains(&q)
                    || r.display_target(lang).to_lowercase().contains(&q)
                    || r.filename.to_lowercase().contains(&q)
                    || r.mail_id.to_lowercase().contains(&q)
                    || r.account_id.to_lowercase().contains(&q)
                    || r.player_name.to_lowercase().contains(&q)
                    || r.battle_type.to_lowercase().contains(&q)
                    || r.time_vn.to_lowercase().contains(&q)
            })
            .collect()
    };

    let is_asc = sort_dir == Some("asc");
    match sort_col {
        Some("time") => filtered.sort_by(|a, b| {
            if is_asc {
                a.time_raw.cmp(&b.time_raw)
            } else {
                b.time_raw.cmp(&a.time_raw)
            }
        }),
        Some("type") => filtered.sort_by(|a, b| {
            if is_asc {
                a.battle_type.cmp(&b.battle_type)
            } else {
                b.battle_type.cmp(&a.battle_type)
            }
        }),
        Some("target") => filtered.sort_by(|a, b| {
            let ta = a.display_target(lang);
            let tb = b.display_target(lang);
            if is_asc {
                ta.cmp(&tb)
            } else {
                tb.cmp(&ta)
            }
        }),
        Some("damage") => filtered.sort_by(|a, b| {
            if is_asc {
                a.damage_pct.cmp(&b.damage_pct)
            } else {
                b.damage_pct.cmp(&a.damage_pct)
            }
        }),
        Some("tier") => filtered.sort_by(|a, b| {
            if is_asc {
                a.plunder_tier.cmp(&b.plunder_tier)
            } else {
                b.plunder_tier.cmp(&a.plunder_tier)
            }
        }),
        Some("bonus") => filtered.sort_by(|a, b| {
            if is_asc {
                a.talent_add.cmp(&b.talent_add)
            } else {
                b.talent_add.cmp(&a.talent_add)
            }
        }),
        Some("crystals") => filtered.sort_by(|a, b| {
            let ca = a.crystal_quantity();
            let cb = b.crystal_quantity();
            if is_asc {
                ca.cmp(&cb)
            } else {
                cb.cmp(&ca)
            }
        }),
        Some("speedups") => filtered.sort_by(|a, b| {
            let sa = a.univ_speedup_minutes();
            let sb = b.univ_speedup_minutes();
            if is_asc {
                sa.cmp(&sb)
            } else {
                sb.cmp(&sa)
            }
        }),
        Some("train_spd") => filtered.sort_by(|a, b| {
            let sa = a.training_speedup_minutes();
            let sb = b.training_speedup_minutes();
            if is_asc {
                sa.cmp(&sb)
            } else {
                sb.cmp(&sa)
            }
        }),
        Some("heal_spd") => filtered.sort_by(|a, b| {
            let sa = a.heal_speedup_minutes();
            let sb = b.heal_speedup_minutes();
            if is_asc {
                sa.cmp(&sb)
            } else {
                sb.cmp(&sa)
            }
        }),
        Some("resources") => filtered.sort_by(|a, b| {
            let ra = a.total_resource_value();
            let rb = b.total_resource_value();
            if is_asc {
                ra.cmp(&rb)
            } else {
                rb.cmp(&ra)
            }
        }),
        Some("tome_exp") => filtered.sort_by(|a, b| {
            let ea = a.total_exp_value();
            let eb = b.total_exp_value();
            if is_asc {
                ea.cmp(&eb)
            } else {
                eb.cmp(&ea)
            }
        }),
        Some("books") => filtered.sort_by(|a, b| {
            let ba = a.item_quantity(2, 109);
            let bb = b.item_quantity(2, 109);
            if is_asc {
                ba.cmp(&bb)
            } else {
                bb.cmp(&ba)
            }
        }),
        Some("arrows") => filtered.sort_by(|a, b| {
            let aa = a.item_quantity(2, 145);
            let ab = b.item_quantity(2, 145);
            if is_asc {
                aa.cmp(&ab)
            } else {
                ab.cmp(&aa)
            }
        }),
        Some("gems") => filtered.sort_by(|a, b| {
            let ga = a.compute_gems();
            let gb = b.compute_gems();
            if is_asc {
                ga.cmp(&gb)
            } else {
                gb.cmp(&ga)
            }
        }),
        _ => filtered.sort_by_key(|a| std::cmp::Reverse(a.time_raw)),
    }

    filtered
        .into_iter()
        .map(|r| BattleRecordDto::from_record(r, lang))
        .collect()
}

pub fn classify_inventory_tab(it: &ItemStat) -> &'static str {
    match get_item_conversion(it.itype, it.sub_type) {
        ConvertedValue::Resource { .. } | ConvertedValue::Gem(_) => "Resources",
        ConvertedValue::EquipmentMaterial { .. } => "Materials",
        ConvertedValue::Speedup { .. } => "Speedups",
        ConvertedValue::Crystal => "Crystals",
        ConvertedValue::BookOfCovenant
        | ConvertedValue::ArrowOfResistance
        | ConvertedValue::Exp(_) => "Books & Arrows",
        ConvertedValue::Starlight | ConvertedValue::ApRecovery(_) | ConvertedValue::AllianceChest => {
            "Other"
        }
        ConvertedValue::Other => {
            let c_low = it.category.to_lowercase();
            let n_vi = it.name_vi.as_str();
            let n_en = it.name_en.as_str();
            if c_low.contains("pha lê")
                || c_low.contains("crystal")
                || n_vi.contains("Pha Lê")
                || n_en.contains("Crystal")
            {
                "Crystals"
            } else if c_low.contains("tăng tốc") || c_low.contains("speedup") {
                "Speedups"
            } else if c_low.contains("vật liệu") || c_low.contains("material") {
                "Materials"
            } else if c_low.contains("tài nguyên")
                || c_low.contains("resource")
                || n_vi.contains("Đá Quý")
                || n_vi.contains("Đá quý")
            {
                "Resources"
            } else if n_vi.contains("Giao Ước")
                || n_vi.contains("giao ước")
                || n_vi.contains("Ước Nguyện")
                || n_vi.contains("ước nguyện")
                || n_vi.contains("Kháng Cự")
                || n_vi.contains("kháng cự")
                || n_vi.contains("Chống Trả")
                || n_vi.contains("chống trả")
                || n_vi.contains("Tri Thức")
                || n_vi.contains("Kiến Thức")
                || n_en.contains("Covenant")
                || n_en.contains("Resistance")
                || n_en.contains("Knowledge")
            {
                "Books & Arrows"
            } else {
                "Other"
            }
        }
    }
}

pub fn query_inventory_from_stats(
    stats: &Statistics,
    lang: AppLanguage,
    search: &str,
    category: &str,
    sort_col: Option<&str>,
    sort_dir: Option<&str>,
) -> serde_json::Value {
    let res = stats.compute_resources();
    let mats = stats.compute_materials();
    let ap = stats.compute_ap();

    let mut items: Vec<&ItemStat> = stats.item_stats.values().collect();

    if category != "Tất cả" && category != "All" && !category.trim().is_empty() {
        let target_tab = match category {
            "Tài Nguyên" | "Resources" => Some("Resources"),
            "Vật Liệu" | "Materials" => Some("Materials"),
            "Tăng Tốc" | "Speedups" => Some("Speedups"),
            "Pha Lê" | "Crystals" => Some("Crystals"),
            "Sách & Tên" | "Books & Arrows" => Some("Books & Arrows"),
            "Khác" | "Other" => Some("Other"),
            _ => None,
        };
        if let Some(tab) = target_tab {
            items.retain(|it| classify_inventory_tab(it) == tab);
        }
    }

    if !search.trim().is_empty() {
        let q = search.trim().to_lowercase();
        items.retain(|it| {
            it.name_vi.to_lowercase().contains(&q)
                || it.name_en.to_lowercase().contains(&q)
                || it.display_name(lang).to_lowercase().contains(&q)
        });
    }

    let total_battles_f = stats.total_battles.max(1) as f64;
    let mut items_dto: Vec<ItemStatDto> = items
        .into_iter()
        .map(|it| ItemStatDto {
            itype: it.itype,
            sub_type: it.sub_type,
            name_vi: it.name_vi.clone(),
            name_en: it.name_en.clone(),
            display_name: it.display_name(lang).to_string(),
            category: it.category.clone(),
            total_quantity: it.total_quantity,
            converted_value: get_item_converted_display_for_lang(
                it.itype,
                it.sub_type,
                it.total_quantity,
                lang,
            ),
            battle_count: it.battle_count,
            drop_rate: (it.battle_count as f64 / total_battles_f) * 100.0,
            avg_per_battle: it.avg_per_battle,
            max_in_battle: it.max_in_battle,
            by_battle_type: it.by_battle_type.clone().into_iter().collect(),
        })
        .collect();

    let is_asc = sort_dir == Some("asc");
    match sort_col {
        Some("name") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.display_name.cmp(&b.display_name)
            } else {
                b.display_name.cmp(&a.display_name)
            }
        }),
        Some("category") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.category.cmp(&b.category)
            } else {
                b.category.cmp(&a.category)
            }
        }),
        Some("qty") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.total_quantity.cmp(&b.total_quantity)
            } else {
                b.total_quantity.cmp(&a.total_quantity)
            }
        }),
        Some("battles") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.battle_count.cmp(&b.battle_count)
            } else {
                b.battle_count.cmp(&a.battle_count)
            }
        }),
        Some("drop_rate") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.drop_rate
                    .partial_cmp(&b.drop_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                b.drop_rate
                    .partial_cmp(&a.drop_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        Some("avg") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.avg_per_battle
                    .partial_cmp(&b.avg_per_battle)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                b.avg_per_battle
                    .partial_cmp(&a.avg_per_battle)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }),
        Some("max") => items_dto.sort_by(|a, b| {
            if is_asc {
                a.max_in_battle.cmp(&b.max_in_battle)
            } else {
                b.max_in_battle.cmp(&a.max_in_battle)
            }
        }),
        _ => items_dto.sort_by_key(|a| std::cmp::Reverse(a.total_quantity)),
    }

    let gem_packs: i64 = stats
        .item_stats
        .values()
        .filter(|it| {
            matches!(
                get_item_conversion(it.itype, it.sub_type),
                ConvertedValue::Gem(_)
            )
        })
        .map(|it| it.total_quantity)
        .sum();

    let res_dto = ResourceSummaryDto {
        total_val: res.total_rss_val(),
        food_val: res.food_val,
        wood_val: res.wood_val,
        stone_val: res.stone_val,
        gold_val: res.gold_val,
        gem_val: stats.compute_gems(),
        food_packs: res.food_packs,
        wood_packs: res.wood_packs,
        stone_packs: res.stone_packs,
        gold_packs: res.gold_packs,
        gem_packs,
        choice_packs: res.choice_packs,
    };

    let mats_dto = MaterialsSummaryDto {
        leather_total: mats.leather_total_converted(),
        iron_total: mats.iron_total_converted(),
        ebony_total: mats.ebony_total_converted(),
        bone_total: mats.bone_total_converted(),
        total_normal: mats.total_normal(),
        total_advanced: mats.total_advanced(),
        total_converted: mats.total_converted_normal(),
    };

    let ap_dto = ApBottlesSummaryDto {
        ap_50: ap.ap_50,
        ap_100: ap.ap_100,
        total_bottles: ap.total_bottles(),
        total_points: ap.total_ap_points(),
    };

    serde_json::json!({
        "items": items_dto,
        "resources": res_dto,
        "materials": mats_dto,
        "apBottles": ap_dto,
    })
}

fn exe_base_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn write_atomic_str(path: &std::path::Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp_name = format!(
        "{}.tmp",
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "export".to_string())
    );
    let tmp_path = path.with_file_name(tmp_name);
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp_path).map_err(|e| e.to_string())?;
        f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    if std::fs::rename(&tmp_path, path).is_err() {
        let _ = std::fs::remove_file(path);
        std::fs::rename(&tmp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            e.to_string()
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_initial_state(state: State<'_, TauriAppState>) -> Result<AppSnapshotDto, String> {
    reconcile_sync_worker_config(&state.sync_worker, &state.config);
    let cfg = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let is_watching = state.watcher.is_watching();
    let is_scanning = state.watcher.is_scanning();
    let vault_count = state.watcher.get_vault_total_count();

    let available_accounts: Vec<AccountSummaryDto> = state
        .watcher
        .get_accounts()
        .into_iter()
        .map(|(uid, name)| AccountSummaryDto { uid, name })
        .collect();

    let detected_accounts: Vec<AccountSummaryDto> = state
        .watcher
        .get_all_detected_accounts()
        .into_iter()
        .map(|(uid, name)| AccountSummaryDto { uid, name })
        .collect();

    let non_main_records_count = state.watcher.count_non_main_records();
    let cloud_status = get_cloud_status_for_state(&state);
    let (autostart_enabled, autostart_minimized_reg) = crate::autostart::get_autostart_status();
    let autostart_minimized = if autostart_enabled {
        autostart_minimized_reg
    } else {
        cfg.autostart_minimized
    };

    let mail_dir_str = state.watcher.get_mail_dir().to_string_lossy().to_string();

    let status_text = if is_scanning {
        match cfg.language {
            AppLanguage::Vietnamese => "Đang quét...".to_string(),
            AppLanguage::English => "Scanning...".to_string(),
        }
    } else if is_watching {
        match cfg.language {
            AppLanguage::Vietnamese => "Đang theo dõi".to_string(),
            AppLanguage::English => "Watching".to_string(),
        }
    } else {
        match cfg.language {
            AppLanguage::Vietnamese => "Đã tạm dừng".to_string(),
            AppLanguage::English => "Paused".to_string(),
        }
    };

    Ok(AppSnapshotDto {
        version: env!("CARGO_PKG_VERSION").to_string(),
        mail_dir: mail_dir_str,
        mode: *state.selected_mode.lock().unwrap_or_else(|e| e.into_inner()),
        date_preset: *state
            .selected_date_preset
            .lock()
            .unwrap_or_else(|e| e.into_inner()),
        date_from: state
            .date_from
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
        date_to: state
            .date_to
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone(),
        account_uid: state.selected_account_clone(),
        available_accounts,
        detected_accounts,
        non_main_records_count,
        main_account_uids: cfg.main_account_uids.clone(),
        ignore_non_main_reports: cfg.ignore_non_main_reports,
        theme: cfg.theme,
        language: cfg.language,
        speedup_unit: *state.speedup_unit.lock().unwrap_or_else(|e| e.into_inner()),
        close_behavior: cfg.close_behavior,
        is_watching,
        is_scanning,
        watcher_status_text: status_text,
        vault_count,
        auto_scan: cfg.auto_scan,
        autostart: autostart_enabled,
        autostart_minimized,
        overlay_config: cfg.overlay.clone(),
        cloud_status,
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_filter_state(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    mode: FilterMode,
    date_preset: DateFilterPreset,
    date_from: String,
    date_to: String,
    account_uid: Option<String>,
    speedup_unit: Option<SpeedupUnit>,
) -> Result<DashboardDto, String> {
    let (start_date, end_date, days, clean_from, clean_to) =
        resolve_date_filter(date_preset, &date_from, &date_to);

    let normalized_acc = account_uid
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    {
        *state.selected_mode.lock().unwrap_or_else(|e| e.into_inner()) = mode;
        *state
            .selected_date_preset
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = date_preset;
        *state.date_from.lock().unwrap_or_else(|e| e.into_inner()) = clean_from;
        *state.date_to.lock().unwrap_or_else(|e| e.into_inner()) = clean_to;
        *state
            .selected_account
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = normalized_acc.clone();
        if let Some(su) = speedup_unit {
            *state.speedup_unit.lock().unwrap_or_else(|e| e.into_inner()) = su;
        }

        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.mode = Some(mode);
        if let Some(su) = speedup_unit {
            cfg.speedup_unit = su;
        }
        cfg.save();
    }

    state.watcher.set_mode(mode);
    state.watcher.set_account_filter(normalized_acc.clone());
    state.watcher.set_date_filter(start_date, end_date, days);

    let cfg_snap = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    sync_settings_to_cloud_async(&cfg_snap);
    let overlay_payload = get_overlay_payload_internal(
        &state.watcher,
        &cfg_snap,
        normalized_acc.as_deref(),
        state.idle_secs(),
    );
    let _ = app.emit("overlay://updated", overlay_payload);

    get_dashboard_data(state)
}

pub(crate) fn sync_settings_to_cloud_async(cfg: &crate::model::AppConfig) {
    if !cfg.cloud_auth.is_logged_in() {
        return;
    }
    if let Some(tok) = cfg.cloud_auth.access_token.clone() {
        let (url, key) = cfg.cloud_auth.get_credentials();
        let settings = cfg.extract_cloud_settings();
        std::thread::spawn(move || {
            let _ = crate::cloud_sync::upload_user_settings(
                &url,
                &key,
                &tok,
                &settings,
                Some(std::time::Duration::from_secs(10)),
            );
        });
    }
}

#[tauri::command]
pub fn set_preferences(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    theme: AppTheme,
    language: AppLanguage,
    close_behavior: CloseBehavior,
) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.theme = theme;
        cfg.language = language;
        cfg.close_behavior = close_behavior;
        cfg.save();
    }
    state.watcher.set_language(language);

    if let Ok(guard) = state.tray_items.lock() {
        if let Some(ref items) = *guard {
            let (show_lbl, rescan_lbl, toggle_lbl, quit_lbl) = tray_menu_labels(language);
            let _ = items.show.set_text(show_lbl);
            let _ = items.rescan.set_text(rescan_lbl);
            let _ = items.toggle.set_text(toggle_lbl);
            let _ = items.quit.set_text(quit_lbl);
        }
    }

    let cfg_snap = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    sync_settings_to_cloud_async(&cfg_snap);
    sync_settings_to_cloud_async(&cfg_snap);
    let sel_acc = state.selected_account_clone();
    let overlay_payload = get_overlay_payload_internal(
        &state.watcher,
        &cfg_snap,
        sel_acc.as_deref(),
        state.idle_secs(),
    );
    let _ = app.emit("overlay://updated", overlay_payload);
    Ok(())
}

#[tauri::command]
pub fn get_dashboard_data(state: State<'_, TauriAppState>) -> Result<DashboardDto, String> {
    let stats = state.watcher.get_stats();
    let active_records = state.watcher.get_active_records();
    let lang = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .language;

    let mut motte_count = 0usize;
    let mut fort_count = 0usize;
    let mut barb_count = 0usize;
    let mut gathering_count = 0usize;

    for r in active_records.iter() {
        match r.battle_type.as_str() {
            "motte" => motte_count += 1,
            "fort" => fort_count += 1,
            "barbarian" => barb_count += 1,
            "gathering" => gathering_count += 1,
            _ => {}
        }
    }

    let ap_econ = state
        .watcher
        .with_vault_records(|vault| compute_ap_economics(&active_records, vault));

    let (train_spd, heal_spd, univ_spd) = stats.compute_speedups_by_category();
    let total_spd = train_spd + heal_spd + univ_spd;
    let crystals = stats.compute_crystals();
    let avg_crystals = if stats.total_battles > 0 {
        crystals as f64 / stats.total_battles as f64
    } else {
        0.0
    };

    let res = stats.compute_resources();

    let mut sorted_items: Vec<_> = stats.item_stats.values().collect();
    sorted_items.sort_by_key(|a| std::cmp::Reverse(a.total_quantity));

    let total_battles_f = stats.total_battles.max(1) as f64;
    let top_items = sorted_items
        .into_iter()
        .take(15)
        .map(|it| ItemStatDto {
            itype: it.itype,
            sub_type: it.sub_type,
            name_vi: it.name_vi.clone(),
            name_en: it.name_en.clone(),
            display_name: it.display_name(lang).to_string(),
            category: it.category.clone(),
            total_quantity: it.total_quantity,
            converted_value: get_item_converted_display_for_lang(
                it.itype,
                it.sub_type,
                it.total_quantity,
                lang,
            ),
            battle_count: it.battle_count,
            drop_rate: (it.battle_count as f64 / total_battles_f) * 100.0,
            avg_per_battle: it.avg_per_battle,
            max_in_battle: it.max_in_battle,
            by_battle_type: it.by_battle_type.clone().into_iter().collect(),
        })
        .collect();

    let latest_raw = crate::analyzer::get_latest_records(&active_records, 5);
    let latest_battles = latest_raw
        .iter()
        .map(|r| BattleRecordDto::from_record(r, lang))
        .collect();

    Ok(DashboardDto {
        total_battles: stats.total_battles,
        total_crystals: crystals,
        avg_crystals,
        speedup_total_mins: total_spd,
        speedup_univ_mins: univ_spd,
        speedup_train_mins: train_spd,
        speedup_heal_mins: heal_spd,
        total_resources: res.total_rss_val(),
        motte_count,
        fort_count,
        barb_count,
        gathering_count,
        ap_economics: ap_econ,
        top_items,
        latest_battles,
    })
}

#[tauri::command]
pub fn query_battles(
    state: State<'_, TauriAppState>,
    search: String,
    sort_col: Option<String>,
    sort_dir: Option<String>,
) -> Result<Vec<BattleRecordDto>, String> {
    let active_records = state.watcher.get_active_records();
    let lang = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .language;

    Ok(query_battles_from_slice(
        &active_records,
        lang,
        &search,
        sort_col.as_deref(),
        sort_dir.as_deref(),
    ))
}

#[tauri::command]
pub fn query_inventory(
    state: State<'_, TauriAppState>,
    search: String,
    category: String,
    sort_col: Option<String>,
    sort_dir: Option<String>,
) -> Result<serde_json::Value, String> {
    let stats = state.watcher.get_stats();
    let lang = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .language;

    Ok(query_inventory_from_stats(
        &stats,
        lang,
        &search,
        &category,
        sort_col.as_deref(),
        sort_dir.as_deref(),
    ))
}

#[tauri::command]
pub fn trigger_rescan(state: State<'_, TauriAppState>) -> Result<(), String> {
    state.watcher.rescan();
    Ok(())
}

#[tauri::command]
pub fn trigger_force_rescan(state: State<'_, TauriAppState>) -> Result<(), String> {
    state.watcher.rescan_force();
    Ok(())
}

#[tauri::command]
pub fn set_auto_scan(state: State<'_, TauriAppState>, enabled: bool) -> Result<bool, String> {
    let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.auto_scan = enabled;
    cfg.save();
    Ok(enabled)
}

#[tauri::command]
pub fn toggle_watcher_pause(state: State<'_, TauriAppState>) -> Result<bool, String> {
    if state.watcher.is_watching() {
        state.watcher.stop();
        Ok(false)
    } else {
        state.watcher.start(false);
        Ok(true)
    }
}

#[tauri::command]
pub fn pick_mail_directory() -> Result<Option<String>, String> {
    let dialog = rfd::FileDialog::new().pick_folder();
    Ok(dialog.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn set_mail_directory(state: State<'_, TauriAppState>, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Mail directory path cannot be empty".to_string());
    }
    let p = std::path::PathBuf::from(trimmed);
    if !p.is_dir() {
        return Err(format!("Directory does not exist: {}", p.display()));
    }
    state.watcher.set_mail_dir(p.clone());
    let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.mail_dir = Some(p);
    cfg.save();
    Ok(())
}

#[tauri::command]
pub fn reset_default_mail_directory(state: State<'_, TauriAppState>) -> Result<String, String> {
    let def = crate::watcher::detect_default_mail_dir();
    let def_str = def.to_string_lossy().to_string();
    state.watcher.set_mail_dir(def.clone());
    let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.mail_dir = Some(def);
    cfg.save();
    Ok(def_str)
}

#[tauri::command]
pub fn open_directory(state: State<'_, TauriAppState>, kind: String) -> Result<(), String> {
    let target_dir = match kind.as_str() {
        "mailcache" => state.watcher.get_mail_dir(),
        "vault" => {
            let vdir = state.watcher.get_vault_binary_dir();
            let _ = std::fs::create_dir_all(&vdir);
            vdir
        }
        _ => exe_base_dir(),
    };
    if !target_dir.exists() {
        return Err(format!("Directory does not exist: {}", target_dir.display()));
    }
    open::that(&target_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_main_accounts(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    uids: Vec<String>,
) -> Result<(), String> {
    let cleaned: Vec<String> = uids
        .into_iter()
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .collect();
    state.watcher.set_main_account_uids(cleaned.clone());
    let cfg_snap = {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.main_account_uids = cleaned;
        cfg.save();
        cfg.clone()
    };
    sync_settings_to_cloud_async(&cfg_snap);
    let _ = app.emit("watcher://updated", ());
    Ok(())
}

#[tauri::command]
pub fn set_ignore_non_main_reports(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    enabled: bool,
) -> Result<bool, String> {
    state.watcher.set_ignore_non_main_reports(enabled);
    let cfg_snap = {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.ignore_non_main_reports = enabled;
        cfg.save();
        cfg.clone()
    };
    sync_settings_to_cloud_async(&cfg_snap);
    let _ = app.emit("watcher://updated", ());
    Ok(enabled)
}

#[tauri::command]
pub fn delete_secondary_records(
    app: AppHandle,
    state: State<'_, TauriAppState>,
) -> Result<usize, String> {
    let deleted = state.watcher.delete_non_main_records();
    let _ = app.emit("watcher://updated", ());
    Ok(deleted)
}

#[tauri::command]
pub fn export_active_data(
    state: State<'_, TauriAppState>,
    format: String,
) -> Result<Option<String>, String> {
    let records = state.watcher.get_active_records();
    let lang = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .language;
    let out_dir = exe_base_dir();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

    if format == "json" {
        let default_name = format!("rok_battles_{}.json", timestamp);
        let picked = rfd::FileDialog::new()
            .set_directory(&out_dir)
            .set_file_name(&default_name)
            .add_filter("JSON", &["json"])
            .save_file();
        let Some(file_path) = picked else {
            return Ok(None);
        };
        let stats = state.watcher.get_stats();
        crate::exporters::export_json(&records, &stats, &file_path)
            .map_err(|e| e.to_string())?;
        Ok(Some(file_path.to_string_lossy().to_string()))
    } else {
        let default_name = format!("rok_battles_{}.csv", timestamp);
        let picked = rfd::FileDialog::new()
            .set_directory(&out_dir)
            .set_file_name(&default_name)
            .add_filter("CSV", &["csv"])
            .save_file();
        let Some(file_path) = picked else {
            return Ok(None);
        };
        let csv_content = crate::exporters::generate_csv(&records, lang);
        write_atomic_str(&file_path, &csv_content)?;
        Ok(Some(file_path.to_string_lossy().to_string()))
    }
}

#[tauri::command]
pub fn get_clipboard_summary(state: State<'_, TauriAppState>) -> Result<String, String> {
    let stats = state.watcher.get_stats();
    let active_records = state.watcher.get_active_records();
    let cfg = state.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let lang = cfg.language;
    let su = *state.speedup_unit.lock().unwrap_or_else(|e| e.into_inner());
    let date_label = state.watcher.get_date_range_label();

    let (train_spd, heal_spd, univ_spd) = stats.compute_speedups_by_category();
    let total_spd = train_spd + heal_spd + univ_spd;
    let crystals = stats.compute_crystals();
    let gems = stats.compute_gems();
    let books = stats.compute_books_of_covenant();
    let arrows = stats.compute_arrows_of_resistance();
    let res = stats.compute_resources();
    let ap_econ = state
        .watcher
        .with_vault_records(|vault| compute_ap_economics(&active_records, vault));

    let summary = match lang {
        AppLanguage::Vietnamese => format!(
            "=== RoK Loot Watcher ({}) ===\n\
             Tổng số trận: {}\n\
             Pha lê: {} | Đá quý: {}\n\
             Sách giao ước: {} | Mũi tên kháng cự: {}\n\
             Tăng tốc tổng: {} (Chung: {} | Lính: {} | Trị: {})\n\
             Tài nguyên quy đổi: {} (Ngô: {} | Gỗ: {} | Đá: {} | Vàng: {})\n\
             Kinh tế AP: Tiêu hao ~{} AP | Hồi lại {} AP (Ròng: {} AP)",
            date_label,
            format_num(stats.total_battles as i64),
            format_num(crystals),
            format_num(gems),
            format_num(books),
            format_num(arrows),
            su.format_duration_for_language(total_spd as f64, lang),
            su.format_duration_for_language(univ_spd as f64, lang),
            su.format_duration_for_language(train_spd as f64, lang),
            su.format_duration_for_language(heal_spd as f64, lang),
            format_num(res.total_rss_val()),
            format_num(res.food_val),
            format_num(res.wood_val),
            format_num(res.stone_val),
            format_num(res.gold_val),
            format_num(ap_econ.total_ap_consumed),
            format_num(ap_econ.total_ap_recovered),
            format_num(ap_econ.net_ap_spent),
        ),
        AppLanguage::English => format!(
            "=== RoK Loot Watcher ({}) ===\n\
             Total Battles: {}\n\
             Crystals: {} | Gems: {}\n\
             Books of Covenant: {} | Arrows of Resistance: {}\n\
             Total Speedups: {} (Univ: {} | Train: {} | Heal: {})\n\
             Converted Resources: {} (Food: {} | Wood: {} | Stone: {} | Gold: {})\n\
             AP Economics: Est. Spent ~{} AP | Returned {} AP (Net: {} AP)",
            date_label,
            format_num(stats.total_battles as i64),
            format_num(crystals),
            format_num(gems),
            format_num(books),
            format_num(arrows),
            su.format_duration_for_language(total_spd as f64, lang),
            su.format_duration_for_language(univ_spd as f64, lang),
            su.format_duration_for_language(train_spd as f64, lang),
            su.format_duration_for_language(heal_spd as f64, lang),
            format_num(res.total_rss_val()),
            format_num(res.food_val),
            format_num(res.wood_val),
            format_num(res.stone_val),
            format_num(res.gold_val),
            format_num(ap_econ.total_ap_consumed),
            format_num(ap_econ.total_ap_recovered),
            format_num(ap_econ.net_ap_spent),
        ),
    };

    Ok(summary)
}

#[tauri::command]
pub fn export_vault_backup(state: State<'_, TauriAppState>) -> Result<Option<String>, String> {
    let out_dir = exe_base_dir();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let default_name = format!("rok_vault_backup_{}.json", timestamp);
    let picked = rfd::FileDialog::new()
        .set_directory(&out_dir)
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .save_file();
    let Some(file_path) = picked else {
        return Ok(None);
    };
    let json_data = state.watcher.export_vault_json()?;
    write_atomic_str(&file_path, &json_data)?;
    Ok(Some(file_path.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn import_vault_backup(
    app: AppHandle,
    state: State<'_, TauriAppState>,
) -> Result<Option<usize>, String> {
    let out_dir = exe_base_dir();
    let picked = rfd::FileDialog::new()
        .set_directory(&out_dir)
        .add_filter("JSON", &["json"])
        .pick_file();
    let Some(file_path) = picked else {
        return Ok(None);
    };
    let raw_json = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let imported = state.watcher.import_vault_json(&raw_json)?;
    state.watcher.recompute_filtered_records_and_stats();
    state
        .watcher
        .send_sync_command(crate::cloud_sync::SyncCommand::TriggerDelta);

    let _ = app.emit("watcher://updated", ());
    let cfg_snap = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    sync_settings_to_cloud_async(&cfg_snap);
    let sel_acc = state.selected_account_clone();
    let payload = get_overlay_payload_internal(
        &state.watcher,
        &cfg_snap,
        sel_acc.as_deref(),
        state.idle_secs(),
    );
    let _ = app.emit("overlay://updated", payload);
    Ok(Some(imported))
}

#[tauri::command]
pub fn clean_binary_vault(state: State<'_, TauriAppState>) -> Result<serde_json::Value, String> {
    let (deleted_files, freed_bytes) = state.watcher.clean_binary_vault()?;
    Ok(serde_json::json!({
        "deleted_files": deleted_files,
        "freed_bytes": freed_bytes,
    }))
}

#[tauri::command]
pub fn get_overlay_state(state: State<'_, TauriAppState>) -> Result<OverlayPayloadDto, String> {
    let cfg = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let sel_acc = state.selected_account_clone();
    Ok(get_overlay_payload_internal(
        &state.watcher,
        &cfg,
        sel_acc.as_deref(),
        state.idle_secs(),
    ))
}

#[tauri::command]
pub fn update_overlay_config(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    config: OverlayConfigUpdateDto,
) -> Result<(), String> {
    {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(enabled) = config.enabled {
            cfg.overlay.enabled = enabled;
        }
        if let Some(mode) = config.mode {
            cfg.overlay.mode = mode;
        }
        if let Some(filter) = config.filter {
            cfg.overlay.filter = filter;
        }
        if let Some(opacity) = config.opacity {
            cfg.overlay.opacity = opacity.clamp(0.1, 1.0);
        }
        if let Some(text_hex) = config.text_color_hex {
            if !text_hex.trim().is_empty() {
                cfg.overlay.text_color_hex = text_hex;
            }
        }
        if let Some(accent_hex) = config.accent_color_hex {
            if !accent_hex.trim().is_empty() {
                cfg.overlay.accent_color_hex = accent_hex;
            }
        }
        if let Some(ox) = config.offset_x {
            cfg.overlay.offset_x = ox.max(0.0);
        }
        if let Some(oy) = config.offset_y {
            cfg.overlay.offset_y = oy.max(0.0);
        }
        if let Some(w) = config.width {
            cfg.overlay.width = w.clamp(180.0, 520.0);
        }
        if let Some(fs) = config.font_size {
            cfg.overlay.font_size = fs.clamp(8.5, 16.0);
        }
        if let Some(pinned) = config.pinned_items {
            cfg.overlay.pinned_items = pinned;
        }
        if let Some(hk) = config.hotkey_enabled {
            cfg.overlay.hotkey_enabled = hk;
        }
        cfg.save();
    }

    let cfg_snap = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    sync_settings_to_cloud_async(&cfg_snap);
    let sel_acc = state.selected_account_clone();
    let payload = get_overlay_payload_internal(
        &state.watcher,
        &cfg_snap,
        sel_acc.as_deref(),
        state.idle_secs(),
    );
    let _ = app.emit("overlay://updated", payload);
    Ok(())
}

#[tauri::command]
pub fn get_cloud_status(state: State<'_, TauriAppState>) -> Result<CloudStatusDto, String> {
    Ok(get_cloud_status_for_state(&state))
}

#[tauri::command]
pub fn start_google_login(app: AppHandle, state: State<'_, TauriAppState>) -> Result<(), String> {
    let (url, key) = {
        let cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.cloud_auth.get_credentials()
    };
    if url.is_empty() || key.is_empty() {
        return Err("Supabase URL or Anon Key not configured".to_string());
    }
    if state.cloud_login_busy.swap(true, Ordering::SeqCst) {
        return Err("Google login is already in progress".to_string());
    }

    let _ = app.emit("cloud://updated", get_cloud_status_for_state(&state));

    let watcher = Arc::clone(&state.watcher);
    let config = Arc::clone(&state.config);
    let sync_state = Arc::clone(&state.sync_worker.state);
    let sync_tx = state.sync_worker.tx.clone();
    let login_busy = Arc::clone(&state.cloud_login_busy);
    let app_handle = app.clone();

    std::thread::spawn(move || {
        watcher.log("INFO", "Initiating Google OAuth login flow in browser...");
        let outcome = crate::cloud_sync::start_google_login(&url, &key);
        login_busy.store(false, Ordering::SeqCst);

        let dummy_handle = crate::cloud_sync::SyncWorkerHandle {
            tx: sync_tx.clone(),
            state: Arc::clone(&sync_state),
        };

        match outcome {
            Ok(auth_res) => {
                let cfg_clone = {
                    let mut cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                    cfg.cloud_auth.access_token = Some(auth_res.access_token.clone());
                    cfg.cloud_auth.refresh_token = Some(auth_res.refresh_token.clone());
                    cfg.cloud_auth.user_id = Some(auth_res.user.id.clone());
                    cfg.cloud_auth.user_email = Some(auth_res.user.email.clone());
                    cfg.save();
                    cfg.clone()
                };

                if let Some(ref cloud_settings) = auth_res.user.settings {
                    let mut settings_changed = false;
                    {
                        let mut cfg = config.lock().unwrap_or_else(|e| e.into_inner());
                        if cfg.apply_cloud_settings(cloud_settings) {
                            watcher.set_main_account_uids(cfg.main_account_uids.clone());
                            watcher.set_ignore_non_main_reports(cfg.ignore_non_main_reports);
                            cfg.save();
                            settings_changed = true;
                        }
                    }
                    if settings_changed {
                        watcher.log("SUCCESS", "Restored cloud user settings");
                        let _ = app_handle.emit("watcher://updated", ());
                    }
                } else {
                    let (u, k) = cfg_clone.cloud_auth.get_credentials();
                    if let Some(ref tok) = cfg_clone.cloud_auth.access_token {
                        let settings = cfg_clone.extract_cloud_settings();
                        let _ = crate::cloud_sync::upload_user_settings(
                            &u,
                            &k,
                            tok,
                            &settings,
                            Some(std::time::Duration::from_secs(10)),
                        );
                    }
                }
                if let Ok(mut s) = sync_state.write() {
                    s.last_error = None;
                }
                let _ = sync_tx.send(crate::cloud_sync::SyncCommand::UpdateConfig(
                    cfg_clone.cloud_auth.clone(),
                ));
                let _ = sync_tx.send(crate::cloud_sync::SyncCommand::SyncNow(None));
                watcher.log("SUCCESS", "Google OAuth login successful!");
                let _ = app_handle.emit(
                    "cloud://updated",
                    get_cloud_status_from_parts(&watcher, &cfg_clone, &dummy_handle, false),
                );
            }
            Err(e) => {
                let err_msg = format!("Google OAuth login failed: {}", e);
                if let Ok(mut s) = sync_state.write() {
                    s.last_error = Some(err_msg.clone());
                }
                watcher.log("ERROR", &err_msg);
                let cfg_clone = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
                let _ = app_handle.emit(
                    "cloud://updated",
                    get_cloud_status_from_parts(&watcher, &cfg_clone, &dummy_handle, false),
                );
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn cloud_logout(app: AppHandle, state: State<'_, TauriAppState>) -> Result<(), String> {
    let auth_clone = {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        cfg.cloud_auth.logout();
        cfg.save();
        cfg.cloud_auth.clone()
    };
    state
        .watcher
        .send_sync_command(crate::cloud_sync::SyncCommand::UpdateConfig(auth_clone));
    let _ = app.emit("cloud://updated", get_cloud_status_for_state(&state));
    Ok(())
}

#[tauri::command]
pub fn trigger_cloud_push(app: AppHandle, state: State<'_, TauriAppState>) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    if !state
        .watcher
        .send_sync_command(crate::cloud_sync::SyncCommand::SyncNow(Some(tx)))
    {
        return Err("Cloud sync channel unavailable".to_string());
    }

    let _ = app.emit("cloud://updated", get_cloud_status_for_state(&state));

    let watcher = Arc::clone(&state.watcher);
    let config = Arc::clone(&state.config);
    let sync_state = Arc::clone(&state.sync_worker.state);
    let sync_tx = state.sync_worker.tx.clone();
    let login_busy = Arc::clone(&state.cloud_login_busy);
    let app_clone = app.clone();

    std::thread::spawn(move || {
        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Ok(uploaded)) => {
                watcher.log(
                    "SUCCESS",
                    &format!("Cloud backup complete ({} records uploaded)", uploaded),
                );
            }
            Ok(Err(e)) => {
                watcher.log("ERROR", &format!("Cloud backup failed: {}", e));
            }
            Err(e) => {
                watcher.log("ERROR", &format!("Cloud backup timeout: {}", e));
            }
        }
        let handle = crate::cloud_sync::SyncWorkerHandle {
            tx: sync_tx,
            state: sync_state,
        };
        reconcile_sync_worker_config(&handle, &config);
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Some(ref tok) = cfg.cloud_auth.access_token {
            let (u, k) = cfg.cloud_auth.get_credentials();
            let settings = cfg.extract_cloud_settings();
            let _ = crate::cloud_sync::upload_user_settings(
                &u,
                &k,
                tok,
                &settings,
                Some(std::time::Duration::from_secs(10)),
            );
        }
        let _ = app_clone.emit(
            "cloud://updated",
            get_cloud_status_from_parts(&watcher, &cfg, &handle, login_busy.load(Ordering::SeqCst)),
        );
    });

    Ok(())
}

#[tauri::command]
pub fn trigger_cloud_pull(app: AppHandle, state: State<'_, TauriAppState>) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    if !state
        .watcher
        .send_sync_command(crate::cloud_sync::SyncCommand::Restore(tx))
    {
        return Err("Cloud sync channel unavailable".to_string());
    }

    let _ = app.emit("cloud://updated", get_cloud_status_for_state(&state));

    let watcher = Arc::clone(&state.watcher);
    let config = Arc::clone(&state.config);
    let sync_state = Arc::clone(&state.sync_worker.state);
    let sync_tx = state.sync_worker.tx.clone();
    let login_busy = Arc::clone(&state.cloud_login_busy);
    let sel_acc = state.selected_account_clone();
    let idle = state.idle_secs();
    let app_clone = app.clone();

    std::thread::spawn(move || {
        match rx.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok(Ok(cnt)) => {
                watcher.reload_vault();
                watcher.log("SUCCESS", &format!("Cloud pull restored {} records", cnt));
                let _ = app_clone.emit("watcher://updated", ());
                let cfg_snap = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
                let overlay_payload = get_overlay_payload_internal(
                    &watcher,
                    &cfg_snap,
                    sel_acc.as_deref(),
                    idle,
                );
                let _ = app_clone.emit("overlay://updated", overlay_payload);
            }
            Ok(Err(e)) => {
                watcher.log("ERROR", &format!("Cloud pull error: {}", e));
            }
            Err(e) => {
                watcher.log("ERROR", &format!("Cloud pull timeout: {}", e));
            }
        }
        let handle = crate::cloud_sync::SyncWorkerHandle {
            tx: sync_tx,
            state: sync_state,
        };
        reconcile_sync_worker_config(&handle, &config);
        let cfg = config.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Some(ref tok) = cfg.cloud_auth.access_token {
            let (u, k) = cfg.cloud_auth.get_credentials();
            if let Ok(Some(cloud_settings)) = crate::cloud_sync::fetch_user_settings(
                &u,
                &k,
                tok,
                Some(std::time::Duration::from_secs(10)),
            ) {
                let mut changed = false;
                {
                    let mut cfg_guard = config.lock().unwrap_or_else(|e| e.into_inner());
                    if cfg_guard.apply_cloud_settings(&cloud_settings) {
                        watcher.set_main_account_uids(cfg_guard.main_account_uids.clone());
                        watcher.set_ignore_non_main_reports(cfg_guard.ignore_non_main_reports);
                        cfg_guard.save();
                        changed = true;
                    }
                }
                if changed {
                    watcher.log("SUCCESS", "Restored user settings from Cloud");
                    let _ = app_clone.emit("watcher://updated", ());
                }
            }
        }
        let _ = app_clone.emit(
            "cloud://updated",
            get_cloud_status_from_parts(&watcher, &cfg, &handle, login_busy.load(Ordering::SeqCst)),
        );
    });
    Ok(())
}

#[tauri::command]
pub fn set_cloud_auto_sync(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    enabled: bool,
) -> Result<(), String> {
    set_cloud_config(app, state, Some(enabled), None, None, None)
}

#[tauri::command]
pub fn set_cloud_config(
    app: AppHandle,
    state: State<'_, TauriAppState>,
    auto_backup: Option<bool>,
    backup_interval_minutes: Option<u32>,
    backup_on_exit: Option<bool>,
    stay_login: Option<bool>,
) -> Result<(), String> {
    let auth_clone = {
        let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ab) = auto_backup {
            cfg.cloud_auth.auto_backup = ab;
        }
        if let Some(interval) = backup_interval_minutes {
            cfg.cloud_auth.backup_interval_minutes = interval.clamp(5, 1440);
        }
        if let Some(boe) = backup_on_exit {
            cfg.cloud_auth.backup_on_exit = boe;
        }
        if let Some(sl) = stay_login {
            cfg.cloud_auth.stay_login = sl;
        }
        cfg.save();
        cfg.cloud_auth.clone()
    };
    state
        .watcher
        .send_sync_command(crate::cloud_sync::SyncCommand::UpdateConfig(auth_clone));
    let _ = app.emit("cloud://updated", get_cloud_status_for_state(&state));
    Ok(())
}

#[tauri::command]
pub fn set_autostart(
    state: State<'_, TauriAppState>,
    enabled: bool,
    minimized: Option<bool>,
) -> Result<bool, String> {
    let min_val = {
        let cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        minimized.unwrap_or(cfg.autostart_minimized)
    };
    crate::autostart::set_autostart(enabled, min_val).map_err(|e| e.to_string())?;
    let mut cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.autostart = enabled;
    cfg.autostart_minimized = min_val;
    cfg.save();
    Ok(enabled)
}

#[tauri::command]
pub fn window_minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn window_minimize(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.minimize();
    }
    Ok(())
}

#[tauri::command]
pub fn window_maximize(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_maximized().unwrap_or(false) {
            let _ = win.unmaximize();
        } else {
            let _ = win.maximize();
        }
    }
    Ok(())
}

#[tauri::command]
pub fn window_close(app: AppHandle, state: State<'_, TauriAppState>) -> Result<(), String> {
    let close_behavior = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .close_behavior;
    match close_behavior {
        CloseBehavior::Tray => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }
        }
        CloseBehavior::Close => {
            crate::tauri_app::shutdown_app_gracefully(&app);
        }
    }
    Ok(())
}
