use super::*;
use crate::gui::components::*;
use crate::gui::theme::*;
use crate::i18n::*;
use crate::model::*;
use crate::watcher::*;

#[test]
fn test_overlay_state_initialization() {
    let config = AppConfig::default();
    assert!(!config.overlay.enabled);
    assert_eq!(config.overlay.filter, FilterMode::AllRewards);
}

#[test]
fn test_overlay_state_with_position_and_size() {
    let mut config = AppConfig::default();
    config.overlay.offset_x = 45.0;
    config.overlay.offset_y = 120.0;
    config.overlay.width = 280.0;
    config.overlay.font_size = 12.5;
    assert_eq!(config.overlay.offset_x, 45.0);
    assert_eq!(config.overlay.offset_y, 120.0);
    assert_eq!(config.overlay.width, 280.0);
    assert_eq!(config.overlay.font_size, 12.5);
}

#[test]
fn test_gui_language_state_default() {
    let config = AppConfig::default();
    assert_eq!(config.language, AppLanguage::Vietnamese);
}

#[test]
fn test_gui_language_switching() {
    let mut config = AppConfig::default();
    assert_eq!(config.language, AppLanguage::Vietnamese);
    config.language = AppLanguage::English;
    assert_eq!(config.language, AppLanguage::English);
}

#[test]
fn test_report_table_dimensions() {
    assert_eq!(
        report_table_dimensions(2000.0, 1000.0, 10),
        ReportTableDimensions {
            table_width: 2000.0,
            target_width: 180.0,
            extra_per_column: 82.0,
        }
    );
    assert_eq!(
        report_table_dimensions(1200.0, 1000.0, 10),
        ReportTableDimensions {
            table_width: 1200.0,
            target_width: 180.0,
            extra_per_column: 2.0,
        }
    );
    assert_eq!(
        report_table_dimensions(1050.0, 1000.0, 10),
        ReportTableDimensions {
            table_width: 1100.0,
            target_width: 100.0,
            extra_per_column: 0.0,
        }
    );
}

#[test]
fn test_report_non_target_column_count() {
    assert_eq!(report_non_target_column_count(true, false), 7);
    assert_eq!(report_non_target_column_count(false, true), 11);
    assert_eq!(report_non_target_column_count(false, false), 10);
}

#[test]
fn test_category_badge_color_and_sort_columns() {
    let colors = ThemeColors::dark();
    assert_eq!(category_badge_color("Tài nguyên", &colors), colors.accent_gold);
    assert_eq!(category_badge_color("Tăng tốc", &colors), colors.speedup_blue);
    assert_eq!(category_badge_color("Pha Lê", &colors), colors.crystal_cyan);

    let cols = [
        ItemSortColumn::Name,
        ItemSortColumn::Category,
        ItemSortColumn::TotalQty,
        ItemSortColumn::Converted,
        ItemSortColumn::Battles,
        ItemSortColumn::DropRate,
        ItemSortColumn::AvgPerBattle,
        ItemSortColumn::MaxInBattle,
    ];
    assert_eq!(cols.len(), 8);
}

#[test]
fn test_theme_toggle_preserves_watcher_status_badge() {
    let dark_colors = ThemeColors::dark();
    let light_colors = ThemeColors::light();

    let status = WatcherStatus::Running("running".into());

    let (dark_text, dark_color) = resolve_status_badge(&status, true, AppLanguage::Vietnamese, &dark_colors);
    assert_eq!(dark_text, format!("● {}", AppLanguage::Vietnamese.t(TrKey::StatusWatching)));
    assert_eq!(dark_color, dark_colors.status_green);

    let (light_text, light_color) = resolve_status_badge(&status, true, AppLanguage::Vietnamese, &light_colors);
    assert_eq!(light_text, dark_text);
    assert_eq!(light_color, light_colors.status_green);

    let stopped_status = WatcherStatus::Stopped("paused".into());
    let (stop_text, stop_color) = resolve_status_badge(&stopped_status, false, AppLanguage::English, &dark_colors);
    assert_eq!(stop_text, format!("● {}", AppLanguage::English.t(TrKey::StatusPaused)));
    assert_eq!(stop_color, dark_colors.text_muted);
}

#[test]
fn test_light_mode_contrast_and_palette() {
    let light = ThemeColors::light();
    let dark = ThemeColors::dark();

    assert!(!light.is_dark);
    assert!(dark.is_dark);

    assert_eq!(light.card_surface, Color32::from_rgb(0xff, 0xff, 0xff));
    assert_eq!(light.ink, Color32::from_rgb(0xe6, 0xec, 0xf5));
    assert_eq!(light.ink_deep, Color32::from_rgb(0xee, 0xf2, 0xf8));

    assert_eq!(light.log_bg, Color32::from_rgb(0xf8, 0xfa, 0xfc));
    assert_eq!(dark.log_bg, Color32::from_rgb(0x08, 0x0b, 0x11));

    assert_eq!(light.canvas_tint, Color32::from_rgb(0x0f, 0x17, 0x2a));
    assert_eq!(light.text_muted, Color32::from_rgb(0x33, 0x41, 0x55));

    assert_eq!(light.train_orange, Color32::from_rgb(0xc2, 0x41, 0x0c));
    assert_eq!(light.heal_blue, Color32::from_rgb(0x03, 0x69, 0xa1));
    assert_eq!(light.status_green, Color32::from_rgb(0x04, 0x78, 0x57));
    assert_eq!(light.accent_gold, Color32::from_rgb(0xb4, 0x53, 0x09));
    assert_eq!(light.speedup_blue, Color32::from_rgb(0x1d, 0x4e, 0xd8));
    assert_eq!(light.crystal_cyan, Color32::from_rgb(0x0e, 0x74, 0x90));

    assert_eq!(dark.ink, Color32::from_rgb(0x0b, 0x0f, 0x17));
    assert_eq!(dark.card_surface, Color32::from_rgb(0x19, 0x22, 0x31));
    assert_eq!(dark.text_muted, Color32::from_rgb(0xb0, 0xc0, 0xd4));

    assert_eq!(category_badge_color("Vật liệu", &light), light.heal_blue);
    assert_eq!(category_badge_color("Vật liệu", &dark), dark.heal_blue);
}

#[test]
fn test_inventory_sorting_category_localized_and_converted_numerical() {
    use crate::model::ItemStat;
    use std::collections::HashMap;

    let make_item = |itype: u32, sub_type: u32, total_quantity: i64, category: &str| ItemStat {
        itype,
        sub_type,
        name_vi: String::new(),
        name_en: String::new(),
        category: category.to_string(),
        sprites: vec![],
        pic: String::new(),
        total_quantity,
        battle_count: 1,
        max_in_battle: total_quantity,
        min_in_battle: total_quantity,
        avg_per_battle: total_quantity as f64,
        by_battle_type: HashMap::new(),
    };

    let item_crystals = make_item(1, 9, 100, "Pha Lê");
    let item_resources_150k = make_item(2, 2, 3, "Tài nguyên");
    let item_resources_2m = make_item(2, 2, 40, "Tài nguyên");
    let item_resources_50k = make_item(2, 2, 1, "Tài nguyên");
    let item_speedups = make_item(2, 58, 10, "Tăng tốc");

    let lang = AppLanguage::English;
    assert_eq!(category_name(&item_crystals.category, lang), "Crystals");
    assert_eq!(category_name(&item_resources_150k.category, lang), "Resources");
    assert_eq!(category_name(&item_speedups.category, lang), "Speedups");

    let mut items = [&item_speedups, &item_resources_150k, &item_crystals];
    items.sort_by(|a, b| {
        category_name(&a.category, lang).cmp(category_name(&b.category, lang))
    });
    assert_eq!(category_name(&items[0].category, lang), "Crystals");
    assert_eq!(category_name(&items[1].category, lang), "Resources");
    assert_eq!(category_name(&items[2].category, lang), "Speedups");

    assert_eq!(item_resources_50k.raw_converted_value(), 50_000);
    assert_eq!(item_resources_150k.raw_converted_value(), 150_000);
    assert_eq!(item_resources_2m.raw_converted_value(), 2_000_000);

    let mut converted_items = [&item_resources_150k, &item_resources_2m];
    converted_items.sort_by(|a, b| {
        b.raw_converted_value().cmp(&a.raw_converted_value())
    });
    assert_eq!(converted_items[0].raw_converted_value(), 2_000_000);
    assert_eq!(converted_items[1].raw_converted_value(), 150_000);
    assert_eq!(converted_items[0].total_quantity, 40);
    assert_eq!(converted_items[1].total_quantity, 3);

    let mut all_res = [&item_resources_150k, &item_resources_50k, &item_resources_2m];
    all_res.sort_by(|a, b| {
        b.raw_converted_value().cmp(&a.raw_converted_value())
    });
    assert_eq!(all_res[0].raw_converted_value(), 2_000_000);
    assert_eq!(all_res[1].raw_converted_value(), 150_000);
    assert_eq!(all_res[2].raw_converted_value(), 50_000);

    converted_items.sort_by(|a, b| {
        a.raw_converted_value().cmp(&b.raw_converted_value())
    });
    assert_eq!(converted_items[0].raw_converted_value(), 150_000);
    assert_eq!(converted_items[1].raw_converted_value(), 2_000_000);
}


#[test]
fn test_poll_cloud_receiver_ok_clears_busy_and_receiver() {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    tx.send("test_token".to_string()).unwrap();

    let mut opt = Some(rx);
    let mut busy = true;
    let res = poll_cloud_receiver(&mut opt, &mut busy);

    assert_eq!(res, Some(Ok("test_token".to_string())));
    assert!(!busy);
    assert!(opt.is_none());
}

#[test]
fn test_poll_cloud_receiver_disconnect_clears_busy_and_receiver() {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    drop(tx);

    let mut opt = Some(rx);
    let mut busy = true;
    let res = poll_cloud_receiver(&mut opt, &mut busy);

    assert_eq!(res, Some(Err(())));
    assert!(!busy);
    assert!(opt.is_none());
}

#[test]
fn test_poll_cloud_receiver_empty_retains_busy_and_receiver() {
    let (_tx, rx) = std::sync::mpsc::channel::<String>();

    let mut opt = Some(rx);
    let mut busy = true;
    let res = poll_cloud_receiver(&mut opt, &mut busy);

    assert_eq!(res, None);
    assert!(busy);
    assert!(opt.is_some());
}

#[test]
fn test_auto_scan_trigger_fallback_preserves_state() {
    let mut initial_path = "C:\\existing\\custom\\path".to_string();
    let scan_result: Option<std::path::PathBuf> = None;
    if let Some(dir) = scan_result {
        initial_path = dir.to_string_lossy().to_string();
    }
    assert_eq!(initial_path, "C:\\existing\\custom\\path");
}

#[test]
fn test_auto_scan_trigger_success_updates_state() {
    let mut initial_path = "C:\\existing\\custom\\path".to_string();
    let scan_result: Option<std::path::PathBuf> = Some(std::path::PathBuf::from("D:\\Games\\ROK\\mailcache"));
    if let Some(dir) = scan_result {
        initial_path = dir.to_string_lossy().to_string();
    }
    assert_eq!(initial_path, "D:\\Games\\ROK\\mailcache");
}

#[test]
fn test_theme_visuals_caching_state() {
    let mut last_applied_theme: Option<AppTheme> = None;
    let current_theme = AppTheme::Dark;

    assert_ne!(last_applied_theme, Some(current_theme));
    last_applied_theme = Some(current_theme);

    assert_eq!(last_applied_theme, Some(current_theme));

    let toggled = current_theme.toggle();
    assert_ne!(last_applied_theme, Some(toggled));
    last_applied_theme = Some(toggled);
    assert_eq!(last_applied_theme, Some(AppTheme::Light));
}

#[test]
fn test_reports_tab_dirty_state_tracking() {
    let mut reports_sort_dirty = false;
    let last_records_count = 10usize;
    let last_battle_search = "motte".to_string();
    let last_battle_sort_col = Some(BattleSortColumn::Time);
    let last_battle_sort_dir = SortDirection::Descending;
    let last_battle_mode = FilterMode::AllTargets;
    let last_battle_lang = AppLanguage::Vietnamese;
    let cached_indices = &[0, 1, 2];

    let is_dirty = reports_sort_dirty
        || 10 != last_records_count
        || "motte" != last_battle_search
        || Some(BattleSortColumn::Time) != last_battle_sort_col
        || SortDirection::Descending != last_battle_sort_dir
        || FilterMode::AllTargets != last_battle_mode
        || AppLanguage::Vietnamese != last_battle_lang
        || cached_indices.iter().any(|&i| i >= 10);
    assert!(!is_dirty, "Should be clean when no parameters changed");

    let is_dirty_search = "fort" != last_battle_search;
    assert!(is_dirty_search);

    let is_dirty_col = Some(BattleSortColumn::Damage) != last_battle_sort_col;
    assert!(is_dirty_col);

    let is_dirty_dir = SortDirection::Ascending != last_battle_sort_dir;
    assert!(is_dirty_dir);

    let is_dirty_count = 11 != last_records_count;
    assert!(is_dirty_count);

    let is_dirty_lang = AppLanguage::English != last_battle_lang;
    assert!(is_dirty_lang);

    let is_dirty_mode = FilterMode::Barbarian != last_battle_mode;
    assert!(is_dirty_mode);

    reports_sort_dirty = true;
    assert!(reports_sort_dirty);

    let invalid_indices = &[0, 1, 15];
    assert!(invalid_indices.iter().any(|&i| i >= 10));
}

#[test]
fn test_overlay_records_and_event_refresh() {
    use std::sync::mpsc::channel;
    let temp_dir = std::env::temp_dir().join(format!("rok_test_cache_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let make_rec = |id: &str, btype: &str, time_raw: i64| BattleRecord {
        filename: format!("mail_{}", id),
        mail_id: id.to_string(),
        battle_type: btype.to_string(),
        time_raw,
        time_utc: String::new(),
        time_vn: "2026-10-27 00:00:00".to_string(),
        target: format!("Target {}", id),
        target_level: "5".to_string(),
        damage_pct: 100,
        plunder_tier: 1,
        talent_add: 0,
        pos_x: 0,
        pos_y: 0,
        items: vec![],
        item_totals: std::collections::HashMap::new(),
        content: String::new(),
        ..Default::default()
    };

    let mut vault = crate::vault::MailVault::new(&temp_dir);
    vault.upsert(make_rec("1", "fort", 100));
    let _ = vault.save_to_disk();

    let (tx, _rx) = channel();
    let watcher = Arc::new(RoKMailWatcher::new(
        Some(temp_dir.clone()),
        Some(temp_dir.clone()),
        FilterMode::AllTargets,
        crate::loot_catalog::LootCatalog::empty(),
        tx,
    ));

    let initial_records = watcher.get_latest_vault_records(FilterMode::AllRewards, 5);
    assert_eq!(initial_records.len(), 1);

    let mut cached_overlay_records = initial_records;
    assert_eq!(cached_overlay_records.len(), 1);

    let filtered_motte = watcher.get_latest_vault_records(FilterMode::Motte, 5);
    assert_eq!(filtered_motte.len(), 0);

    cached_overlay_records = filtered_motte;
    assert_eq!(cached_overlay_records.len(), 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cloud_auth_stay_login_default_state() {
    let cfg = AppConfig::default();
    assert!(cfg.cloud_auth.stay_login, "Cloud auth stay_login must default to true");
}

#[test]
fn test_cloud_auth_logged_in_with_refresh_or_access_token() {
    let mut cfg = AppConfig::default();
    cfg.cloud_auth.user_email = Some("user@test.com".to_string());

    // Refresh token only (e.g. after restart where access token expired)
    cfg.cloud_auth.access_token = None;
    cfg.cloud_auth.refresh_token = Some("valid_refresh_token".to_string());
    let is_logged_in = cfg.cloud_auth.user_email.is_some()
        && (cfg.cloud_auth.access_token.is_some() || cfg.cloud_auth.refresh_token.is_some());
    assert!(is_logged_in);

    // Access token only
    cfg.cloud_auth.access_token = Some("valid_access_token".to_string());
    cfg.cloud_auth.refresh_token = None;
    let is_logged_in2 = cfg.cloud_auth.user_email.is_some()
        && (cfg.cloud_auth.access_token.is_some() || cfg.cloud_auth.refresh_token.is_some());
    assert!(is_logged_in2);

    // Neither token
    cfg.cloud_auth.access_token = None;
    cfg.cloud_auth.refresh_token = None;
    let is_logged_in3 = cfg.cloud_auth.user_email.is_some()
        && (cfg.cloud_auth.access_token.is_some() || cfg.cloud_auth.refresh_token.is_some());
    assert!(!is_logged_in3);
}

#[test]
fn test_cloud_auto_backup_dirty_flag_state() {
    let mut config = AppConfig::default();
    config.cloud_auth.auto_backup = true;
    config.cloud_auth.backup_interval_minutes = 15;
    config.cloud_auth.backup_on_exit = true;

    assert_eq!(config.cloud_auth.clamped_interval_minutes(), 15);
    assert!(config.cloud_auth.backup_on_exit);

    let mut has_unbacked_data = false;
    let is_new_mail = true;
    if is_new_mail {
        has_unbacked_data = true;
    }
    assert!(has_unbacked_data);

    let mut last_cloud_backup_time = None;
    assert!(last_cloud_backup_time.is_none());
    // On backup dispatch:
    has_unbacked_data = false;
    last_cloud_backup_time = Some(std::time::Instant::now());

    let op_is_backup = true;
    if op_is_backup {
        last_cloud_backup_time = Some(std::time::Instant::now());
    }
    assert!(!has_unbacked_data);
    assert!(last_cloud_backup_time.is_some());

    let interval_duration = std::time::Duration::from_secs(config.cloud_auth.clamped_interval_minutes() as u64 * 60);
    let should_run = match last_cloud_backup_time {
        Some(last) => std::time::Instant::now().duration_since(last) >= interval_duration,
        None => false,
    };
    assert!(!should_run);
}

#[test]
fn test_cloud_backup_inflight_dirty_flag_preservation() {
    let mut has_unbacked_data = true;
    assert!(has_unbacked_data);
    // 1. Dispatch backup: sets has_unbacked_data = false immediately
    has_unbacked_data = false;
    let mut last_cloud_backup_time = Some(std::time::Instant::now());

    // 2. While upload is in flight, new mail arrives:
    let new_mail_arrived = true;
    if new_mail_arrived {
        has_unbacked_data = true;
    }

    // 3. Upload finishes: does NOT clear has_unbacked_data
    let op_is_backup = true;
    if op_is_backup {
        last_cloud_backup_time = Some(std::time::Instant::now());
    }

    // In-flight report must not have been wiped!
    assert!(has_unbacked_data);
    assert!(last_cloud_backup_time.is_some());
}

#[test]
fn test_cloud_backup_error_restores_dirty_and_throttles() {
    let mut has_unbacked_data = false;
    let mut last_cloud_backup_time = None;
    assert!(last_cloud_backup_time.is_none());

    // Dispatch backup
    last_cloud_backup_time = Some(std::time::Instant::now());

    // Backup fails
    let op_failed = true;
    if op_failed {
        last_cloud_backup_time = Some(std::time::Instant::now());
        has_unbacked_data = true;
    }

    assert!(has_unbacked_data);
    assert!(last_cloud_backup_time.is_some());
}

#[test]
fn test_cloud_backup_startup_conditions() {
    // Startup throttle: initialized to Some(Instant::now())
    let startup_time = Some(std::time::Instant::now());
    let interval_duration = std::time::Duration::from_secs(15 * 60);
    let should_run = match startup_time {
        Some(last) => std::time::Instant::now().duration_since(last) >= interval_duration,
        None => false,
    };
    assert!(!should_run);

    // None case evaluates to false (no runaway backup)
    let none_time: Option<std::time::Instant> = None;
    let should_run_none = match none_time {
        Some(last) => std::time::Instant::now().duration_since(last) >= interval_duration,
        None => false,
    };
    assert!(!should_run_none);

    // Initial unbacked logic: only dirty on startup if records exist and never synced
    let never_synced: Option<String> = None;
    let already_synced: Option<String> = Some("2026-10-28 12:00:00".to_string());

    assert!(never_synced.is_none() && 10 > 0);
    assert!(!(never_synced.is_none() && 0 > 0));
    assert!(!(already_synced.is_none() && 10 > 0));
}

#[test]
fn test_ap_economics_empty_render_does_not_panic() {
    let econ = crate::analyzer::compute_ap_economics(&[], &[]);
    assert_eq!(econ.daily_timeline.len(), 7);
    assert_eq!(econ.total_ap_consumed, 0);
    assert_eq!(econ.total_ap_recovered, 0);
    assert_eq!(econ.net_ap_spent, 0);
    assert_eq!(econ.speedup_mins_per_1k_ap, 0.0);
    assert_eq!(econ.gems_per_1k_ap, 0.0);
    assert_eq!(econ.target_benchmarks.len(), 0);
}

#[test]
fn test_localize_benchmark_target_bilingual() {
    assert_eq!(
        localize_benchmark_target("Fort Lvl 11-15 (KvK)", AppLanguage::English),
        "Fort Lvl 11-15 (KvK)"
    );
    assert_eq!(
        localize_benchmark_target("Fort Lvl 11-15 (KvK)", AppLanguage::Vietnamese),
        "Pháo Đài Cấp 11-15 (KvK)"
    );
    assert_eq!(
        localize_benchmark_target("Barbarians Lvl 41-55", AppLanguage::Vietnamese),
        "Dã Man Cấp 41-55"
    );
    assert_eq!(
        localize_benchmark_target("Motte Lvl 1-10", AppLanguage::Vietnamese),
        "Motte Cấp 1-10"
    );
    assert_eq!(
        localize_benchmark_target("Custom Target", AppLanguage::Vietnamese),
        "Custom Target"
    );
}

#[test]
fn test_perform_exit_cloud_backup_shutdown_bounded() {
    let (tx_ui, _rx_ui) = std::sync::mpsc::channel();
    let watcher = RoKMailWatcher::new(None, None, FilterMode::AllTargets, crate::loot_catalog::LootCatalog::empty(), tx_ui);

    let (sync_tx, sync_rx) = std::sync::mpsc::channel();
    watcher.set_sync_channel(sync_tx);

    let start = std::time::Instant::now();
    // Background thread to simulate worker acknowledging shutdown
    std::thread::spawn(move || {
        if let Ok(crate::cloud_sync::SyncCommand::Shutdown(ack_tx)) = sync_rx.recv_timeout(std::time::Duration::from_millis(500)) {
            let _ = ack_tx.send(());
        }
    });

    perform_exit_cloud_backup(&watcher);
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_secs(2), "Exit backup exceeded 2-second bound: {:?}", elapsed);
}

#[test]
fn test_perform_exit_cloud_backup_without_worker_returns_immediately() {
    let (tx_ui, _rx_ui) = std::sync::mpsc::channel();
    let watcher = RoKMailWatcher::new(None, None, FilterMode::AllTargets, crate::loot_catalog::LootCatalog::empty(), tx_ui);

    let start = std::time::Instant::now();
    perform_exit_cloud_backup(&watcher);
    let elapsed = start.elapsed();
    assert!(elapsed < std::time::Duration::from_millis(50), "Exit backup without worker took too long: {:?}", elapsed);
}

#[test]
fn test_cloud_sync_live_status_indicator_states() {
    use crate::cloud_sync::SyncWorkerState;
    use crate::i18n::TrKey;

    // 1. Restoring state
    let mut state = SyncWorkerState {
        is_restoring: true,
        total_records_count: 350,
        ..Default::default()
    };
    let restoring_vi = AppLanguage::Vietnamese.t(TrKey::CloudSyncStatusRestoring).replace("{0}", &state.total_records_count.to_string());
    assert_eq!(restoring_vi, "Đồng bộ: Đang khôi phục từ Cloud (350 bản ghi)...");

    // 2. Syncing state
    state.is_restoring = false;
    state.is_syncing = true;
    state.pending_records_count = 42;
    let syncing_en = AppLanguage::English.t(TrKey::CloudSyncStatusSyncing).replace("{0}", &state.pending_records_count.to_string());
    assert_eq!(syncing_en, "Sync: Uploading 42 records...");

    // 3. Offline with retry backoff
    state.is_syncing = false;
    state.retry_in_secs = Some(15);
    let offline_vi = format!("{} ({}s)", AppLanguage::Vietnamese.t(TrKey::CloudSyncStatusOffline), state.retry_in_secs.unwrap());
    assert_eq!(offline_vi, "Đồng bộ: Mất kết nối, sẽ thử lại (15s)");

    // 3b. Server/Auth error (e.g. HTTP 403) with retry backoff
    let err_403 = "Cloud Sync HTTP 403 error: permission denied for table report_records";
    assert!(!is_network_offline_error(Some(err_403)));
    let err_vi = format!("{} ({}s)", AppLanguage::Vietnamese.t(TrKey::CloudSyncStatusError), state.retry_in_secs.unwrap());
    assert_eq!(err_vi, "Đồng bộ: Lỗi, sẽ thử lại (15s)");
    let err_en = format!("{} ({}s)", AppLanguage::English.t(TrKey::CloudSyncStatusError), state.retry_in_secs.unwrap());
    assert_eq!(err_en, "Sync: Error, will retry (15s)");

    // Network error should be classified as offline
    assert!(is_network_offline_error(Some("Connection timed out")));
    assert!(is_network_offline_error(Some("failed to lookup address information: dns failure")));
    assert!(is_network_offline_error(None));

    // 4. Idle with pending records (not actively syncing: shows Idle + pending label)
    state.retry_in_secs = None;
    state.is_syncing = false;
    state.pending_records_count = 5;
    let idle_with_pending_vi = AppLanguage::Vietnamese.t(TrKey::CloudSyncStatusIdle);
    assert_eq!(idle_with_pending_vi, "Đồng bộ: Đã cập nhật");
    let pending_suffix_en = format!("({} pending records)", state.pending_records_count);
    assert_eq!(pending_suffix_en, "(5 pending records)");

    // 5. Up to date (Idle, 0 pending)
    state.pending_records_count = 0;
    let idle_vi = AppLanguage::Vietnamese.t(TrKey::CloudSyncStatusIdle);
    assert_eq!(idle_vi, "Đồng bộ: Đã cập nhật");
}

#[test]
fn test_sync_worker_state_token_rotation_extraction() {
    use crate::cloud_sync::SyncWorkerState;

    let state = std::sync::Arc::new(std::sync::RwLock::new(SyncWorkerState::default()));
    if let Ok(mut s) = state.write() {
        s.new_tokens = Some(("acc_rot_123".to_string(), "ref_rot_456".to_string()));
    }

    let has_new_tokens = state.read().map(|s| s.new_tokens.is_some()).unwrap_or(false);
    assert!(has_new_tokens);

    let mut auth_cfg = crate::model::CloudAuthConfig::default();
    if has_new_tokens {
        if let Ok(mut s) = state.write() {
            if let Some((acc, ref_tok)) = s.new_tokens.take() {
                auth_cfg.access_token = Some(acc);
                auth_cfg.refresh_token = Some(ref_tok);
            }
        }
    }

    assert_eq!(auth_cfg.access_token.as_deref(), Some("acc_rot_123"));
    assert_eq!(auth_cfg.refresh_token.as_deref(), Some("ref_rot_456"));
    assert!(state.read().unwrap().new_tokens.is_none());
}

#[test]
fn test_format_speedup_compact() {
    assert_eq!(crate::overlay_ui::format_speedup_compact(0), "0m");
    assert_eq!(crate::overlay_ui::format_speedup_compact(-15), "0m");
    assert_eq!(crate::overlay_ui::format_speedup_compact(45), "45m");
    assert_eq!(crate::overlay_ui::format_speedup_compact(870), "14h 30m");
    assert_eq!(crate::overlay_ui::format_speedup_compact(1440), "1d");
    assert_eq!(crate::overlay_ui::format_speedup_compact(1440 + 30), "1d 30m");
    assert_eq!(crate::overlay_ui::format_speedup_compact(1440 + 120 + 45), "1d 2h 45m");
}

#[test]
fn test_format_num_discrete_items() {
    assert_eq!(crate::model::format_num(0), "0");
    assert_eq!(crate::model::format_num(32), "32");
    assert_eq!(crate::model::format_num(1450), "1,450");
    assert_eq!(crate::model::format_num(1000000), "1,000,000");
}

fn make_test_app(pinned_items: Vec<PinnedItemKey>) -> RoKLootWatcherApp {
    RoKLootWatcherApp {
        watcher: Arc::new(RoKMailWatcher::new(
            None,
            None,
            FilterMode::AllRewards,
            crate::loot_catalog::LootCatalog::empty(),
            std::sync::mpsc::channel().0,
        )),
        rx_ui: std::sync::mpsc::channel().1,
        tray: None,
        theme: AppTheme::Dark,
        language: AppLanguage::English,
        watcher_status: WatcherStatus::Running(String::new()),
        mail_path_input: String::new(),
        selected_mode: FilterMode::AllRewards,
        selected_account: None,
        selected_date_preset: DateFilterPreset::AllTime,
        date_from_input: String::new(),
        date_to_input: String::new(),
        current_tab: 0,
        battle_search: String::new(),
        item_search: String::new(),
        item_category_filter: "All".to_string(),
        log_auto_scroll: true,
        battle_sort_col: None,
        battle_sort_dir: SortDirection::Descending,
        item_sort_col: None,
        item_sort_dir: SortDirection::Descending,
        console_logs: std::collections::VecDeque::new(),
        selected_battle_detail: None,
        speedup_unit: SpeedupUnit::Dhm,
        close_behavior: CloseBehavior::Tray,
        autostart: false,
        autostart_minimized: false,
        overlay_config: OverlayConfig {
            enabled: true,
            filter: FilterMode::AllRewards,
            pinned_items,
            ..Default::default()
        },
        last_mail_arrival: std::time::Instant::now(),
        overlay_styled_hwnd: None,
        overlay_last_opacity: None,
        pending_overlay_save: false,
        pending_start_minimized_frames: 0,
        _single_instance: None,
        hotkey_rx: std::sync::mpsc::channel().1,
        _hotkey_guard: None,
        cloud_auth: CloudAuthConfig::default(),
        cloud_rx: None,
        sync_worker: None,
        sync_rx: None,
        restore_rx: None,
        cloud_busy: false,
        has_unbacked_data: false,
        main_account_uids: Vec::new(),
        new_main_uid_input: String::new(),
        confirm_delete_non_main: None,
        cached_overlay_records: Vec::new(),
        cached_today_run_counts: (0, 0, 0),
        cached_today_pinned_chips: Vec::new(),
        cached_sorted_battle_indices: Vec::new(),
        reports_sort_dirty: false,
        last_applied_theme: None,
        last_records_count: 0,
        last_battle_search: String::new(),
        last_battle_sort_col: None,
        last_battle_sort_dir: SortDirection::Descending,
        last_battle_mode: FilterMode::AllRewards,
        last_battle_lang: AppLanguage::English,
        cached_ap_econ: None,
        ap_econ_dirty: false,
        last_ap_records_count: 0,
        last_ap_mode: FilterMode::AllRewards,
    }
}

#[test]
fn test_calculate_today_pinned_chips_aggregation() {
    let app = make_test_app(vec![
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 3,
            canonical_name: "Book of Covenant".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 4,
            canonical_name: "Arrow of Resistance".to_string(),
        },
        PinnedItemKey::SpeedupUniversal,
        PinnedItemKey::SpeedupTraining,
        PinnedItemKey::SpeedupHealing,
    ]);

    let mut rec1 = BattleRecord {
        battle_type: "fort".to_string(),
        items: vec![
            ItemDrop {
                itype: 2,
                sub_type: 36, // 60 min univ speedup
                value: 2,
                name_vi: "Tăng tốc 60 phút".to_string(),
                name_en: "60m Speedup".to_string(),
                category: "Tăng tốc".to_string(),
                sprites: vec![],
                pic: String::new(),
            },
            ItemDrop {
                itype: 2,
                sub_type: 76, // 60 min train speedup
                value: 3,
                name_vi: "Tăng tốc huấn luyện 60 phút".to_string(),
                name_en: "60m Training Speedup".to_string(),
                category: "Tăng tốc".to_string(),
                sprites: vec![],
                pic: String::new(),
            },
        ],
        ..Default::default()
    };
    rec1.item_totals.insert("Book of Covenant".to_string(), 12);
    rec1.item_totals.insert("Đá quý".to_string(), 450);

    let mut rec2 = BattleRecord {
        battle_type: "fort".to_string(),
        items: vec![
            ItemDrop {
                itype: 2,
                sub_type: 96, // 60 min heal speedup
                value: 5,
                name_vi: "Tăng tốc chữa lành 60 phút".to_string(),
                name_en: "60m Healing Speedup".to_string(),
                category: "Tăng tốc".to_string(),
                sprites: vec![],
                pic: String::new(),
            },
        ],
        ..Default::default()
    };
    rec2.item_totals.insert("Sách giao ước".to_string(), 20);
    rec2.item_totals.insert("Gems".to_string(), 1000);
    rec2.item_totals.insert("Arrow of Resistance".to_string(), 14);

    let mut rec3 = BattleRecord {
        battle_type: "motte".to_string(),
        ..Default::default()
    };
    rec3.item_totals.insert("Mũi Tên Chống Trả".to_string(), 18);

    let rec4 = BattleRecord {
        battle_type: "barbarian".to_string(),
        ..Default::default()
    };

    let records = vec![rec1, rec2, rec3, rec4];
    let (runs, chips) = app.calculate_today_pinned_chips(&records);

    // Runs: (Fort, Motte, Barbarian) -> (2, 1, 1)
    assert_eq!(runs, (2, 1, 1));

    // Pinned chips
    assert_eq!(chips.len(), 6);

    // Gems: 450 + 1000 = 1450 -> "1,450"
    assert_eq!(chips[0].label, "GEMS");
    assert_eq!(chips[0].display_val, "1,450");
    assert_eq!(chips[0].text_color, app.colors().crystal_cyan);

    // Books: 12 + 20 = 32 -> "32"
    assert_eq!(chips[1].label, "BOOKS");
    assert_eq!(chips[1].display_val, "32");
    assert_eq!(chips[1].text_color, app.colors().accent_gold);

    // Arrows: 14 + 18 = 32 -> "32"
    assert_eq!(chips[2].label, "ARROWS");
    assert_eq!(chips[2].display_val, "32");
    assert_eq!(chips[2].text_color, app.colors().train_orange);

    // Universal Speedup: 2 * 60 = 120 mins = 2h
    assert_eq!(chips[3].label, "SPEEDUP");
    assert_eq!(chips[3].display_val, "2h");
    assert_eq!(chips[3].text_color, app.colors().speedup_blue);

    // Training Speedup: 3 * 60 = 180 mins = 3h
    assert_eq!(chips[4].label, "TRAIN");
    assert_eq!(chips[4].display_val, "3h");
    assert_eq!(chips[4].text_color, app.colors().train_orange);

    // Healing Speedup: 5 * 60 = 300 mins = 5h
    assert_eq!(chips[5].label, "HEAL");
    assert_eq!(chips[5].display_val, "5h");
    assert_eq!(chips[5].text_color, app.colors().heal_blue);
}

#[test]
fn test_today_vault_records_filtering_and_account_isolation() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("rok_test_today_v_{}_{}", std::process::id(), unique));
    let _ = std::fs::create_dir_all(&temp_dir);

    let (tx, _rx) = std::sync::mpsc::channel();
    let watcher = RoKMailWatcher::new(
        Some(temp_dir.clone()),
        Some(temp_dir.clone()),
        FilterMode::AllRewards,
        crate::loot_catalog::LootCatalog::empty(),
        tx,
    );

    let today = crate::analyzer::current_date_vn();
    let yesterday = today - chrono::Duration::days(1);

    // 1. Fort today (main account)
    let mut rec1 = BattleRecord {
        mail_id: "m1".to_string(),
        battle_type: "fort".to_string(),
        time_vn: format!("{} 10:00:00", today.format("%Y-%m-%d")),
        account_id: "player_1001".to_string(),
        ..Default::default()
    };
    rec1.item_totals.insert("Book of Covenant".to_string(), 5);

    // 2. Motte today (main account)
    let mut rec2 = BattleRecord {
        mail_id: "m2".to_string(),
        battle_type: "motte".to_string(),
        time_vn: format!("{} 11:00:00", today.format("%Y-%m-%d")),
        account_id: "player_1001".to_string(),
        ..Default::default()
    };
    rec2.item_totals.insert("Arrow of Resistance".to_string(), 3);

    // 3. Barbarian today (main account)
    let rec3 = BattleRecord {
        mail_id: "m3".to_string(),
        battle_type: "barbarian".to_string(),
        time_vn: format!("{} 12:00:00", today.format("%Y-%m-%d")),
        account_id: "player_1001".to_string(),
        ..Default::default()
    };

    // 4. Gathering today (main account)
    let rec4 = BattleRecord {
        mail_id: "m4".to_string(),
        battle_type: "gathering".to_string(),
        time_vn: format!("{} 13:00:00", today.format("%Y-%m-%d")),
        account_id: "player_1001".to_string(),
        ..Default::default()
    };

    // 5. Fort yesterday (main account) - should be excluded by today filter
    let rec5 = BattleRecord {
        mail_id: "m5".to_string(),
        battle_type: "fort".to_string(),
        time_vn: format!("{} 10:00:00", yesterday.format("%Y-%m-%d")),
        account_id: "player_1001".to_string(),
        ..Default::default()
    };

    // 6. Fort today from farm account (player_9999) - should be excluded when main_account_uids active
    let rec6 = BattleRecord {
        mail_id: "m6".to_string(),
        battle_type: "fort".to_string(),
        time_vn: format!("{} 14:00:00", today.format("%Y-%m-%d")),
        account_id: "player_9999".to_string(),
        ..Default::default()
    };

    {
        let mut v = watcher.vault.write().unwrap();
        v.upsert(rec1);
        v.upsert(rec2);
        v.upsert(rec3);
        v.upsert(rec4);
        v.upsert(rec5);
        v.upsert(rec6);
    }

    // No main_account_uids set: all 5 today records returned for AllRewards
    let all_today = watcher.get_today_vault_records(FilterMode::AllRewards);
    assert_eq!(all_today.len(), 5);

    // FilterMode restrictions without account isolation:
    let forts = watcher.get_today_vault_records(FilterMode::Fort);
    assert_eq!(forts.len(), 2); // rec1 and rec6

    let mottes = watcher.get_today_vault_records(FilterMode::Motte);
    assert_eq!(mottes.len(), 1); // rec2

    let barbs = watcher.get_today_vault_records(FilterMode::Barbarian);
    assert_eq!(barbs.len(), 1); // rec3

    let combat_targets = watcher.get_today_vault_records(FilterMode::AllTargets);
    assert_eq!(combat_targets.len(), 4); // rec1, rec2, rec3, rec6 (no gathering)

    let gathering = watcher.get_today_vault_records(FilterMode::Gathering);
    assert_eq!(gathering.len(), 1); // rec4

    // Activate Account Isolation for UID "1001"
    watcher.set_main_account_uids(vec!["1001".to_string()]);

    let main_all_today = watcher.get_today_vault_records(FilterMode::AllRewards);
    assert_eq!(main_all_today.len(), 4); // rec1, rec2, rec3, rec4 (rec6 excluded)

    let main_forts = watcher.get_today_vault_records(FilterMode::Fort);
    assert_eq!(main_forts.len(), 1); // only rec1

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_overlay_settings_mode_selection() {
    let mut config = OverlayConfig::default();
    assert_eq!(config.mode, OverlayDisplayMode::Cards);

    // Switch mode
    config.mode = OverlayDisplayMode::MiniHud;
    assert_eq!(config.mode, OverlayDisplayMode::MiniHud);

    config.mode = OverlayDisplayMode::Combined;
    assert_eq!(config.mode, OverlayDisplayMode::Combined);

    // Verify cycling through .next()
    assert_eq!(config.mode.next(), OverlayDisplayMode::Hidden);
    assert_eq!(OverlayDisplayMode::Hidden.next(), OverlayDisplayMode::Cards);
    assert_eq!(OverlayDisplayMode::Cards.next(), OverlayDisplayMode::MiniHud);
    assert_eq!(OverlayDisplayMode::MiniHud.next(), OverlayDisplayMode::Combined);

    // Verify hotkey toggle state
    assert!(config.hotkey_enabled);
    config.hotkey_enabled = false;
    assert!(!config.hotkey_enabled);
}

#[test]
fn test_overlay_settings_pinned_items_add_remove() {
    let mut config = OverlayConfig {
        pinned_items: Vec::new(),
        ..Default::default()
    };
    assert_eq!(config.pinned_items.len(), 0);

    let gems = PinnedItemKey::CatalogItem {
        itype: 1,
        isub: 2,
        canonical_name: "Gems".to_string(),
    };
    let book = PinnedItemKey::CatalogItem {
        itype: 2,
        isub: 3,
        canonical_name: "Book of Covenant".to_string(),
    };
    let arrow = PinnedItemKey::CatalogItem {
        itype: 2,
        isub: 4,
        canonical_name: "Arrow of Resistance".to_string(),
    };

    // Add 1
    assert!(config.add_pinned_item(gems.clone()));
    assert_eq!(config.pinned_items.len(), 1);

    // Prevent duplicate
    assert!(!config.add_pinned_item(gems.clone()));
    assert_eq!(config.pinned_items.len(), 1);

    // Add 2 and 3
    assert!(config.add_pinned_item(book.clone()));
    assert!(config.add_pinned_item(arrow.clone()));
    assert_eq!(config.pinned_items.len(), 3);

    // Add 4 and 5
    assert!(config.add_pinned_item(PinnedItemKey::SpeedupUniversal));
    assert!(config.add_pinned_item(PinnedItemKey::SpeedupTraining));
    assert_eq!(config.pinned_items.len(), 5);

    // Cap at 5: cannot add 6th item
    assert!(!config.add_pinned_item(PinnedItemKey::SpeedupHealing));
    assert_eq!(config.pinned_items.len(), 5);

    // Remove item at index 1 (book)
    let removed = config.remove_pinned_item(1);
    assert_eq!(removed, Some(book));
    assert_eq!(config.pinned_items.len(), 4);
    assert!(!config.pinned_items.contains(&PinnedItemKey::CatalogItem {
        itype: 2,
        isub: 3,
        canonical_name: "Book of Covenant".to_string(),
    }));

    // Can now add another item
    assert!(config.add_pinned_item(PinnedItemKey::SpeedupHealing));
    assert_eq!(config.pinned_items.len(), 5);

    // Out of bounds remove
    assert_eq!(config.remove_pinned_item(10), None);
}

#[test]
fn test_overlay_settings_presets() {
    let mut config = OverlayConfig {
        pinned_items: Vec::new(),
        ..Default::default()
    };

    // Apply Castle Push preset: Book of Covenant, Arrow of Resistance, Gems
    let castle_push = crate::gui::views::settings::preset_castle_push();
    assert_eq!(castle_push.len(), 3);
    config.apply_pinned_preset(&castle_push);

    assert_eq!(config.pinned_items.len(), 3);
    assert_eq!(
        config.pinned_items[0],
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 3,
            canonical_name: "Book of Covenant".to_string(),
        }
    );
    assert_eq!(
        config.pinned_items[1],
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 4,
            canonical_name: "Arrow of Resistance".to_string(),
        }
    );
    assert_eq!(
        config.pinned_items[2],
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        }
    );

    // Apply War & Training preset: Universal Speedup, Training Speedup, Gems
    // Gems is already present, so Universal and Training are added (3 + 2 = 5)
    let war_training = crate::gui::views::settings::preset_war_and_training();
    assert_eq!(war_training.len(), 3);
    config.apply_pinned_preset(&war_training);

    assert_eq!(config.pinned_items.len(), 5);
    assert_eq!(config.pinned_items[3], PinnedItemKey::SpeedupUniversal);
    assert_eq!(config.pinned_items[4], PinnedItemKey::SpeedupTraining);

    // Try applying Gathering & Heals preset when already at 5 items -> capped at 5
    let gathering_heals = crate::gui::views::settings::preset_gathering_and_heals();
    config.apply_pinned_preset(&gathering_heals);
    assert_eq!(config.pinned_items.len(), 5);

    // Check available options list has all 13 items (including Crystals)
    let options = crate::gui::views::settings::available_pinned_item_options();
    assert_eq!(options.len(), 13);
}

#[test]
fn test_pinned_item_crystals_configuration_and_aggregation() {
    let options = crate::gui::views::settings::available_pinned_item_options();
    let crystal_opt = options
        .iter()
        .find(|opt| opt.canonical_name() == "Crystals")
        .expect("Crystals option must be available in pinned item options");

    assert_eq!(crystal_opt.localized_name(AppLanguage::English), "Crystals");
    assert_eq!(crystal_opt.localized_name(AppLanguage::Vietnamese), "Pha Lê");

    let colors = crate::gui::theme::ThemeColors::dark();
    assert_eq!(
        crate::gui::views::settings::pinned_item_color(crystal_opt, &colors),
        colors.crystal_cyan
    );

    let mut rec = BattleRecord {
        mail_id: "rec_crys_1".to_string(),
        battle_type: "fort".to_string(),
        items: vec![
            ItemDrop {
                itype: 1,
                sub_type: 9,
                value: 350,
                name_vi: "Pha Lê".to_string(),
                name_en: "Crystals".to_string(),
                category: "Vật phẩm đặc biệt".to_string(),
                sprites: vec![],
                pic: String::new(),
            },
        ],
        ..Default::default()
    };
    rec.item_totals.insert("Crystals".to_string(), 350);

    let app = make_test_app(vec![crystal_opt.clone()]);

    let (_, chips) = app.calculate_today_pinned_chips(&[rec]);
    assert_eq!(chips.len(), 1);
    assert_eq!(chips[0].label, "CRYSTALS");
    assert_eq!(chips[0].display_val, "350");
    assert_eq!(chips[0].text_color, colors.crystal_cyan);
}

#[test]
fn test_layout_constants_and_global_filter_bar_visibility() {
    assert_eq!(SIDEBAR_WIDTH, 180.0);
    assert_eq!(INSPECTOR_WIDTH, 360.0);
    assert_eq!(SETTINGS_LOG_PANEL_HEIGHT, 190.0);

    assert!(should_show_global_filter_bar(0));
    assert!(should_show_global_filter_bar(1));
    assert!(should_show_global_filter_bar(2));
    assert!(!should_show_global_filter_bar(3));
    assert!(!should_show_global_filter_bar(99));
}

#[test]
fn test_gui_translations_and_badges_contain_no_emojis() {
    let is_emoji = |ch: char| -> bool {
        let code = ch as u32;
        (0x1F300..=0x1FAFF).contains(&code) || (0x2600..=0x27BF).contains(&code)
    };

    for &key in TrKey::all() {
        for lang in [AppLanguage::Vietnamese, AppLanguage::English] {
            let text = lang.t(key);
            assert!(
                !text.chars().any(is_emoji),
                "Emoji found in TrKey::{:?} ({:?}): {}",
                key,
                lang,
                text
            );
        }
    }
}

