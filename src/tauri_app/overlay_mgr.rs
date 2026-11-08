use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use crate::model::OverlayConfig;
use crate::overlay_tracker::{
    apply_webview_click_through_styles, calculate_overlay_physical_geometry,
    determine_overlay_visibility_action, find_rok_window, get_rok_window_bounds,
    get_window_dpi_scale, is_rok_active_and_visible_with_overlay, OverlayVisibilityAction,
};
use crate::tauri_app::commands::TauriAppState;

static OVERLAY_RUNNING: AtomicBool = AtomicBool::new(true);

pub fn stop_overlay_loop() {
    OVERLAY_RUNNING.store(false, Ordering::Relaxed);
}

pub fn spawn_overlay_manager(app: AppHandle) {
    OVERLAY_RUNNING.store(true, Ordering::Relaxed);

    std::thread::Builder::new()
        .name("rok-tauri-overlay-tracker".to_string())
        .spawn(move || {
            let mut styled_hwnd: Option<isize> = None;
            let mut last_action: Option<OverlayVisibilityAction> = None;
            let mut last_pos: Option<(i32, i32)> = None;
            let mut last_size: Option<(u32, u32)> = None;

            while OVERLAY_RUNNING.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));

                let state = match app.try_state::<TauriAppState>() {
                    Some(s) => s,
                    None => continue,
                };

                let overlay_cfg: OverlayConfig = state.config.lock().unwrap().overlay.clone();
                let overlay_win = match app.get_webview_window("overlay") {
                    Some(w) => w,
                    None => continue,
                };

                let rok_hwnd = if overlay_cfg.enabled {
                    find_rok_window()
                } else {
                    None
                };

                let rok_found = rok_hwnd.is_some();
                #[cfg(windows)]
                let raw_overlay_hwnd = overlay_win.hwnd().ok().map(|h| h.0 as _);
                #[cfg(not(windows))]
                let raw_overlay_hwnd = None;

                let is_active = rok_hwnd.is_some_and(|h| {
                    is_rok_active_and_visible_with_overlay(h, raw_overlay_hwnd)
                });

                let action = determine_overlay_visibility_action(
                    overlay_cfg.enabled,
                    overlay_cfg.mode,
                    rok_found,
                    is_active,
                );

                match action {
                    OverlayVisibilityAction::Show => {
                        if let Some(hwnd) = rok_hwnd {
                            if let Some(bounds) = get_rok_window_bounds(hwnd) {
                                let rok_scale = get_window_dpi_scale(hwnd);
                                let geom = calculate_overlay_physical_geometry(
                                    &bounds,
                                    &overlay_cfg,
                                    rok_scale,
                                );

                                let size_changed = match last_size {
                                    Some((w, h)) => {
                                        w.abs_diff(geom.width) >= 2 || h.abs_diff(geom.height) >= 2
                                    }
                                    None => true,
                                };

                                if size_changed {
                                    let _ = overlay_win.set_size(tauri::Size::Physical(
                                        tauri::PhysicalSize {
                                            width: geom.width,
                                            height: geom.height,
                                        },
                                    ));
                                    last_size = Some((geom.width, geom.height));
                                }

                                let pos_changed = match last_pos {
                                    Some((x, y)) => {
                                        (x - geom.pos_x).abs() >= 2 || (y - geom.pos_y).abs() >= 2
                                    }
                                    None => true,
                                };

                                if pos_changed {
                                    let _ = overlay_win.set_position(tauri::Position::Physical(
                                        tauri::PhysicalPosition {
                                            x: geom.pos_x,
                                            y: geom.pos_y,
                                        },
                                    ));
                                    last_pos = Some((geom.pos_x, geom.pos_y));
                                }

                                #[cfg(windows)]
                                {
                                    if let Ok(raw_hwnd) = overlay_win.hwnd() {
                                        let h_ptr = raw_hwnd.0;
                                        let h_isize = h_ptr as isize;
                                        if styled_hwnd != Some(h_isize) {
                                            styled_hwnd = Some(h_isize);
                                            let _ = overlay_win.set_ignore_cursor_events(true);
                                            apply_webview_click_through_styles(h_ptr);
                                        }
                                    }
                                }

                                if last_action != Some(OverlayVisibilityAction::Show) {
                                    let _ = overlay_win.set_ignore_cursor_events(true);
                                    let _ = overlay_win.show();
                                    #[cfg(windows)]
                                    {
                                        if let Ok(raw_hwnd) = overlay_win.hwnd() {
                                            let h_ptr = raw_hwnd.0;
                                            apply_webview_click_through_styles(h_ptr);
                                            unsafe {
                                                let fg = windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
                                                let ov = h_ptr as windows_sys::Win32::Foundation::HWND;
                                                if fg == ov
                                                    || windows_sys::Win32::UI::WindowsAndMessaging::IsChild(ov, fg) != 0
                                                {
                                                    windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                                                }
                                            }
                                        }
                                    }
                                    last_action = Some(OverlayVisibilityAction::Show);
                                }
                            }
                        }
                    }
                    OverlayVisibilityAction::Hide => {
                        if last_action != Some(OverlayVisibilityAction::Hide) {
                            let _ = overlay_win.hide();
                            last_action = Some(OverlayVisibilityAction::Hide);
                            styled_hwnd = None;
                        }
                    }
                }
            }
        })
        .expect("Failed to spawn overlay tracker thread");
}
