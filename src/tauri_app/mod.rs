pub mod dto;
pub mod commands;
pub mod overlay_mgr;
#[cfg(test)]
pub mod tests;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use crate::model::{AppConfig, CloseBehavior};
use crate::watcher::{RoKMailWatcher, WatcherStatus, WatcherUiEvent};
use self::commands::{TauriAppState, TrayMenuItems};

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

pub fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::SeqCst)
}

pub fn shutdown_app_gracefully(app: &tauri::AppHandle) {
    if SHUTTING_DOWN.swap(true, Ordering::SeqCst) {
        return;
    }

    overlay_mgr::stop_overlay_loop();

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.hide();
    }
    if let Some(ov_win) = app.get_webview_window("overlay") {
        let _ = ov_win.hide();
        let _ = ov_win.close();
    }

    let app_clone = app.clone();
    std::thread::Builder::new()
        .name("rok-graceful-shutdown".to_string())
        .spawn(move || {
            if let Some(state) = app_clone.try_state::<TauriAppState>() {
                state.watcher.flush_vault_if_dirty();
                state.watcher.stop();
                commands::reconcile_sync_worker_config(&state.sync_worker, &state.config);

                let (ack_tx, ack_rx) = std::sync::mpsc::channel();
                if state
                    .sync_worker
                    .tx
                    .send(crate::cloud_sync::SyncCommand::Shutdown(ack_tx))
                    .is_ok()
                {
                    let _ = ack_rx.recv_timeout(std::time::Duration::from_millis(2000));
                }
                commands::reconcile_sync_worker_config(&state.sync_worker, &state.config);
            }
            app_clone.exit(0);
        })
        .unwrap_or_else(|_| {
            app.exit(0);
            unreachable!()
        });
}

fn fallback_tray_icon() -> tauri::image::Image<'static> {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32usize {
        for x in 0..32usize {
            let idx = (y * 32 + x) * 4;
            let border = !(2..30).contains(&x) || !(2..30).contains(&y);
            if border {
                rgba[idx] = 240;
                rgba[idx + 1] = 185;
                rgba[idx + 2] = 11;
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = 24;
                rgba[idx + 1] = 26;
                rgba[idx + 2] = 32;
                rgba[idx + 3] = 255;
            }
        }
    }
    tauri::image::Image::new_owned(rgba, 32, 32)
}

pub fn run(
    watcher: Arc<RoKMailWatcher>,
    rx_ui: Receiver<WatcherUiEvent>,
    start_minimized: bool,
    single_instance: Option<crate::single_instance::windows::SingleInstanceGuard>,
) -> Result<(), Box<dyn std::error::Error>> {
    SHUTTING_DOWN.store(false, Ordering::SeqCst);

    let config = AppConfig::load();
    let initial_mode = config.mode.unwrap_or(crate::model::FilterMode::AllTargets);
    let initial_date_preset = crate::model::DateFilterPreset::Last7Days;
    let initial_speedup_unit = config.speedup_unit;
    let initial_lang = config.language;
    let auto_scan_on_launch = config.auto_scan;

    watcher.set_language(initial_lang);
    watcher.set_date_filter(None, None, initial_date_preset.to_days());
    watcher.set_main_account_uids(config.main_account_uids.clone());
    watcher.set_ignore_non_main_reports(config.ignore_non_main_reports);

    // Spawn Cloud Sync Worker
    let vault_dir = watcher
        .get_vault_file_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let sync_vault = Arc::new(crate::vault::MailVault::new(&vault_dir));
    let sync_worker = crate::cloud_sync::spawn_sync_worker(sync_vault, config.cloud_auth.clone());
    watcher.set_sync_channel(sync_worker.tx.clone());

    let state = TauriAppState {
        watcher: Arc::clone(&watcher),
        config: Arc::new(std::sync::Mutex::new(config.clone())),
        sync_worker,
        cloud_login_busy: Arc::new(AtomicBool::new(false)),
        cloud_login_cancel: Arc::new(AtomicBool::new(false)),
        last_mail_arrival: Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
        tray_items: std::sync::Mutex::new(None),
        selected_mode: std::sync::Mutex::new(initial_mode),
        selected_date_preset: std::sync::Mutex::new(initial_date_preset),
        date_from: std::sync::Mutex::new(String::new()),
        date_to: std::sync::Mutex::new(String::new()),
        selected_account: std::sync::Mutex::new(None),
        speedup_unit: std::sync::Mutex::new(initial_speedup_unit),
        single_instance: std::sync::Mutex::new(single_instance),
    };

    let watcher_tray = Arc::clone(&watcher);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_initial_state,
            commands::set_filter_state,
            commands::set_preferences,
            commands::get_dashboard_data,
            commands::query_battles,
            commands::query_inventory,
            commands::trigger_rescan,
            commands::trigger_force_rescan,
            commands::set_auto_scan,
            commands::toggle_watcher_pause,
            commands::pick_mail_directory,
            commands::set_mail_directory,
            commands::reset_default_mail_directory,
            commands::open_directory,
            commands::set_main_accounts,
            commands::set_ignore_non_main_reports,
            commands::delete_secondary_records,
            commands::export_active_data,
            commands::get_clipboard_summary,
            commands::export_vault_backup,
            commands::import_vault_backup,
            commands::clean_binary_vault,
            commands::get_overlay_state,
            commands::update_overlay_config,
            commands::get_cloud_status,
            commands::start_google_login,
            commands::cancel_google_login,
            commands::cloud_logout,
            commands::trigger_cloud_push,
            commands::trigger_cloud_pull,
            commands::set_cloud_auto_sync,
            commands::set_cloud_config,
            commands::set_autostart,
            commands::window_minimize_to_tray,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Setup localized System Tray
            let (show_lbl, rescan_lbl, toggle_lbl, quit_lbl) =
                commands::tray_menu_labels(initial_lang);
            let show_i = MenuItem::with_id(app, "show", show_lbl, true, None::<&str>)?;
            let rescan_i = MenuItem::with_id(app, "rescan", rescan_lbl, true, None::<&str>)?;
            let toggle_i = MenuItem::with_id(app, "toggle", toggle_lbl, true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", quit_lbl, true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &rescan_i, &toggle_i, &quit_i])?;

            {
                let st = app.state::<TauriAppState>();
                if let Ok(mut tray_guard) = st.tray_items.lock() {
                    *tray_guard = Some(TrayMenuItems {
                        show: show_i,
                        rescan: rescan_i,
                        toggle: toggle_i,
                        quit: quit_i,
                    });
                };
            }

            let handle_clone = handle.clone();
            let watcher_tray_clone = Arc::clone(&watcher_tray);

            let icon = match app.default_window_icon() {
                Some(ic) => ic.to_owned(),
                None => fallback_tray_icon(),
            };

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Rise of Kingdoms - Loot Watcher")
                .menu(&menu)
                .on_menu_event(move |_app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = handle_clone.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "rescan" => {
                        watcher_tray_clone.rescan();
                    }
                    "toggle" => {
                        if watcher_tray_clone.is_watching() {
                            watcher_tray_clone.stop();
                        } else {
                            watcher_tray_clone.start(false);
                        }
                    }
                    "quit" => {
                        shutdown_app_gracefully(&handle_clone);
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Start single instance activation listener
            #[cfg(windows)]
            {
                let st_si = app.state::<TauriAppState>();
                if let Ok(guard_lock) = st_si.single_instance.lock() {
                    if let Some(ref guard) = *guard_lock {
                        let handle_act = handle.clone();
                        crate::single_instance::windows::start_activation_listener(
                            guard,
                            move || {
                                if let Some(w) = handle_act.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.unminimize();
                                    let _ = w.set_focus();
                                }
                            },
                            Arc::clone(&watcher_tray),
                        );
                    }
                };
            }

            // Start global overlay hotkey listener (Ctrl+Shift+H)
            let (hotkey_tx, hotkey_rx) = std::sync::mpsc::channel::<()>();
            let _hotkey_guard = crate::overlay_tracker::start_overlay_hotkey_listener(hotkey_tx);
            let app_hotkey = handle.clone();
            let state_hotkey = app.state::<TauriAppState>();
            let config_hotkey = Arc::clone(&state_hotkey.config);
            let watcher_hotkey = Arc::clone(&state_hotkey.watcher);
            std::thread::Builder::new()
                .name("rok-overlay-hotkey-listener".to_string())
                .spawn(move || {
                    let _guard = _hotkey_guard;
                    while hotkey_rx.recv().is_ok() {
                        let next_mode_opt = {
                            let mut cfg = config_hotkey.lock().unwrap_or_else(|e| e.into_inner());
                            if !cfg.overlay.enabled || !cfg.overlay.hotkey_enabled {
                                None
                            } else {
                                let nm = cfg.overlay.mode.next();
                                cfg.overlay.mode = nm;
                                cfg.save();
                                Some(nm)
                            }
                        };
                        let Some(next_mode) = next_mode_opt else {
                            continue;
                        };
                        watcher_hotkey.log(
                            "INFO",
                            &format!("Hotkey triggered: Overlay mode set to {:?}", next_mode),
                        );
                        if let Some(st) = app_hotkey.try_state::<TauriAppState>() {
                            let cfg_snap = st
                                .config
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .clone();
                            let sel_acc = st.selected_account_clone();
                            let payload = commands::get_overlay_payload_internal(
                                &watcher_hotkey,
                                &cfg_snap,
                                sel_acc.as_deref(),
                                st.idle_secs(),
                            );
                            let _ = app_hotkey.emit("overlay://updated", payload);
                        }
                    }
                })?;

            // Background thread forwarding watcher events to Tauri webviews
            let app_events = handle.clone();
            let state_events = app.state::<TauriAppState>();
            let config_events = Arc::clone(&state_events.config);
            let watcher_events = Arc::clone(&state_events.watcher);
            let last_mail_events = Arc::clone(&state_events.last_mail_arrival);

            std::thread::Builder::new()
                .name("rok-tauri-event-bridge".to_string())
                .spawn(move || {
                    while let Ok(evt) = rx_ui.recv() {
                        match evt {
                            WatcherUiEvent::Log { message, .. } => {
                                let _ = app_events.emit("watcher://log", message);
                            }
                            WatcherUiEvent::StatusChanged(status) => {
                                let is_scanning = matches!(status, WatcherStatus::Scanning(_));
                                let is_watching = match &status {
                                    WatcherStatus::Running(_) => true,
                                    WatcherStatus::Scanning(_) => watcher_events.is_watching(),
                                    WatcherStatus::Stopped(_) | WatcherStatus::Error(_) => false,
                                };
                                let text = match status {
                                    WatcherStatus::Running(s)
                                    | WatcherStatus::Scanning(s)
                                    | WatcherStatus::Stopped(s)
                                    | WatcherStatus::Error(s) => s,
                                };
                                let _ = app_events.emit(
                                    "watcher://status",
                                    serde_json::json!({
                                        "status": text,
                                        "is_watching": is_watching,
                                        "is_scanning": is_scanning,
                                    }),
                                );
                            }
                            WatcherUiEvent::StatsUpdated { .. } => {
                                let _ = app_events.emit("watcher://updated", ());
                                let cfg_snap = config_events
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .clone();
                                let (sel_acc, idle) = app_events
                                    .try_state::<TauriAppState>()
                                    .map(|st| (st.selected_account_clone(), st.idle_secs()))
                                    .unwrap_or((None, 0));
                                let overlay_payload = commands::get_overlay_payload_internal(
                                    &watcher_events,
                                    &cfg_snap,
                                    sel_acc.as_deref(),
                                    idle,
                                );
                                let _ = app_events.emit("overlay://updated", overlay_payload);
                            }
                            WatcherUiEvent::MailDetected {
                                target,
                                damage_pct,
                                tier,
                                loot_summary,
                                ..
                            } => {
                                if let Ok(mut ts) = last_mail_events.lock() {
                                    *ts = std::time::Instant::now();
                                }
                                let _ = app_events.emit("watcher://updated", ());
                                let stamp = chrono::Local::now().format("%H:%M:%S");
                                let _ = app_events.emit(
                                    "watcher://log",
                                    format!(
                                        "[{}] [SUCCESS] Drop: {} ({}% DMG, T{}) => {}",
                                        stamp, target, damage_pct, tier, loot_summary
                                    ),
                                );
                                let cfg_snap = config_events
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .clone();
                                let sel_acc = app_events
                                    .try_state::<TauriAppState>()
                                    .and_then(|st| st.selected_account_clone());
                                let overlay_payload = commands::get_overlay_payload_internal(
                                    &watcher_events,
                                    &cfg_snap,
                                    sel_acc.as_deref(),
                                    0,
                                );
                                let _ = app_events.emit("overlay://updated", overlay_payload);
                            }
                        }
                    }
                })?;

            // Background cloud-sync token persistence & live status notifier
            let app_cloud_mon = handle.clone();
            std::thread::Builder::new()
                .name("rok-cloud-state-monitor".to_string())
                .spawn(move || {
                    type CloudMonitorSig = (bool, bool, Option<String>, Option<String>, usize);
                    let mut prev_sig: Option<CloudMonitorSig> = None;
                    while !is_shutting_down() {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        if is_shutting_down() {
                            break;
                        }
                        if let Some(st) = app_cloud_mon.try_state::<TauriAppState>() {
                            let status = commands::get_cloud_status_for_state(&st);
                            let sig = (
                                status.is_syncing,
                                status.is_restoring,
                                status.last_sync_time.clone(),
                                status.sync_error.clone(),
                                status.pending_count,
                            );
                            if prev_sig.as_ref() != Some(&sig) {
                                prev_sig = Some(sig);
                                let _ = app_cloud_mon.emit("cloud://updated", status);
                            }
                        }
                    }
                })?;

            // Spawn in-game overlay tracker loop
            overlay_mgr::spawn_overlay_manager(handle.clone());

            // Handle start minimized
            if start_minimized {
                if let Some(w) = handle.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            // Start watcher & initial vault scan if auto_scan is enabled
            if auto_scan_on_launch {
                watcher_tray.start(true);
            }

            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let app = window.app_handle();
                    let close_behavior = app
                        .try_state::<TauriAppState>()
                        .map(|s| {
                            s.config
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .close_behavior
                        })
                        .unwrap_or(CloseBehavior::Tray);

                    api.prevent_close();
                    match close_behavior {
                        CloseBehavior::Tray => {
                            let _ = window.hide();
                        }
                        CloseBehavior::Close => {
                            shutdown_app_gracefully(app);
                        }
                    }
                };
            }
        })
        .run(tauri::generate_context!())?;

    Ok(())
}
