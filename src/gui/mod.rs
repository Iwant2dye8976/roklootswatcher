use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

pub mod theme;
pub mod components;
pub mod lifecycle;
pub mod views;
#[cfg(test)]
mod tests;

pub use theme::ThemeColors;
pub use lifecycle::perform_exit_cloud_backup;

use crate::model::{
    AppConfig, AppLanguage, AppTheme, BattleRecord, CloseBehavior, DateFilterPreset, FilterMode,
    SpeedupUnit,
};
use crate::tray::SystemTrayManager;
use crate::watcher::{RoKMailWatcher, WatcherStatus, WatcherUiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleSortColumn {
    Time,
    Type,
    Target,
    Level,
    Damage,
    Tier,
    Crystal,
    Resource,
    TomeExp,
    Arrow,
    UnivSpeedup,
    TrainSpeedup,
    HealSpeedup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSortColumn {
    Name,
    Category,
    TotalQty,
    Converted,
    Battles,
    DropRate,
    AvgPerBattle,
    MaxInBattle,
}

pub struct RoKLootWatcherApp {
    pub watcher: Arc<RoKMailWatcher>,
    pub rx_ui: Receiver<WatcherUiEvent>,
    #[allow(dead_code)]
    pub tray: Option<SystemTrayManager>,

    pub theme: AppTheme,
    pub language: AppLanguage,
    pub(crate) watcher_status: WatcherStatus,

    pub(crate) mail_path_input: String,
    pub(crate) selected_mode: FilterMode,
    pub(crate) selected_account: Option<String>,

    pub(crate) selected_date_preset: DateFilterPreset,
    pub(crate) date_from_input: String,
    pub(crate) date_to_input: String,

    pub(crate) current_tab: usize,
    pub(crate) battle_search: String,
    pub(crate) item_search: String,
    pub(crate) item_category_filter: String,
    pub(crate) log_auto_scroll: bool,

    pub(crate) battle_sort_col: Option<BattleSortColumn>,
    pub(crate) battle_sort_dir: SortDirection,
    pub(crate) item_sort_col: Option<ItemSortColumn>,
    pub(crate) item_sort_dir: SortDirection,

    pub(crate) console_logs: VecDeque<String>,
    pub(crate) selected_battle_detail: Option<BattleRecord>,
    pub(crate) speedup_unit: SpeedupUnit,
    pub close_behavior: CloseBehavior,
    pub autostart: bool,
    pub autostart_minimized: bool,
    pub overlay_config: crate::model::OverlayConfig,
    pub last_mail_arrival: std::time::Instant,
    pub(crate) overlay_styled_hwnd: Option<isize>,
    pub(crate) overlay_last_opacity: Option<f32>,
    pub(crate) pending_overlay_save: bool,
    pub(crate) pending_start_minimized_frames: u32,
    pub(crate) _single_instance: Option<crate::single_instance::windows::SingleInstanceGuard>,
    pub(crate) hotkey_rx: std::sync::mpsc::Receiver<()>,
    pub(crate) _hotkey_guard: Option<crate::overlay_tracker::OverlayHotkeyGuard>,

    pub cloud_auth: crate::model::CloudAuthConfig,
    pub(crate) cloud_rx: Option<std::sync::mpsc::Receiver<Result<crate::cloud_sync::CloudAuthResult, String>>>,
    pub sync_worker: Option<crate::cloud_sync::SyncWorkerHandle>,
    pub(crate) sync_rx: Option<std::sync::mpsc::Receiver<Result<usize, String>>>,
    pub(crate) restore_rx: Option<std::sync::mpsc::Receiver<Result<usize, String>>>,
    pub(crate) cloud_busy: bool,
    pub has_unbacked_data: bool,
    pub main_account_uids: Vec<String>,
    pub new_main_uid_input: String,
    pub confirm_delete_non_main: Option<usize>,

    pub cached_overlay_records: Vec<BattleRecord>,
    pub cached_today_run_counts: (usize, usize, usize),
    pub cached_today_pinned_chips: Vec<crate::overlay_ui::CachedPinnedChip>,
    pub cached_sorted_battle_indices: Vec<usize>,
    pub reports_sort_dirty: bool,
    pub last_applied_theme: Option<AppTheme>,
    pub last_records_count: usize,
    pub(crate) last_battle_search: String,
    pub(crate) last_battle_sort_col: Option<BattleSortColumn>,
    pub(crate) last_battle_sort_dir: SortDirection,
    pub(crate) last_battle_mode: FilterMode,
    pub(crate) last_battle_lang: AppLanguage,
    pub cached_ap_econ: Option<crate::analyzer::ApEconomics>,
    pub ap_econ_dirty: bool,
    pub(crate) last_ap_records_count: usize,
    pub(crate) last_ap_mode: FilterMode,
}

impl RoKLootWatcherApp {
    pub fn configure_fonts(ctx: &egui::Context) {
        theme::configure_fonts(ctx);
    }

    pub fn colors(&self) -> ThemeColors {
        match self.theme {
            AppTheme::Dark => ThemeColors::dark(),
            AppTheme::Light => ThemeColors::light(),
        }
    }

    pub fn new(
        cc: &eframe::CreationContext<'_>,
        watcher: Arc<RoKMailWatcher>,
        rx_ui: Receiver<WatcherUiEvent>,
        tray: Option<SystemTrayManager>,
        single_instance: Option<crate::single_instance::windows::SingleInstanceGuard>,
        start_minimized: bool,
    ) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        let initial_path = watcher.get_mail_dir().to_string_lossy().to_string();
        let initial_mode = watcher.get_mode();

        watcher.start(true);

        let config = AppConfig::load();
        let initial_speedup_unit = config.speedup_unit;
        let initial_close_behavior = config.close_behavior;
        let initial_theme = config.theme;
        let initial_language = config.language;

        if let Some(ref tray_mgr) = tray {
            tray_mgr.setup_event_handlers(cc.egui_ctx.clone(), Arc::clone(&watcher));
            tray_mgr.update_language(initial_language);
        }

        if let Some(ref guard) = single_instance {
            crate::single_instance::windows::start_activation_listener(
                guard,
                cc.egui_ctx.clone(),
                Arc::clone(&watcher),
            );
        }

        let (reg_autostart, reg_minimized) = crate::autostart::get_autostart_status();
        let initial_autostart = if reg_autostart { true } else { config.autostart };
        let initial_autostart_minimized = if reg_autostart {
            reg_minimized || config.autostart_minimized
        } else {
            config.autostart_minimized
        };

        if initial_autostart {
            let _ = crate::autostart::set_autostart(initial_autostart, initial_autostart_minimized);
        }

        watcher.set_language(initial_language);

        if start_minimized {
            crate::tray::hide_main_window();
            let min_msg = match initial_language {
                AppLanguage::Vietnamese => "Ứng dụng đã tự động khởi chạy và thu nhỏ xuống Khay hệ thống (System Tray).",
                AppLanguage::English => "Application launched and minimized to System Tray.",
            };
            watcher.log("INFO", min_msg);
        }

        let initial_overlay_records = watcher.get_latest_vault_records(config.overlay.filter, 5);
        let initial_has_unbacked = config.cloud_auth.last_synced.is_none() && watcher.get_vault_total_count() > 0;
        watcher.set_cloud_unbacked(initial_has_unbacked);

        let vault_dir = watcher
            .get_vault_file_path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        let sync_vault = Arc::new(crate::vault::MailVault::new(&vault_dir));
        let sync_worker = crate::cloud_sync::spawn_sync_worker(sync_vault, config.cloud_auth.clone());
        watcher.set_sync_channel(sync_worker.tx.clone());

        if !config.main_account_uids.is_empty() {
            watcher.set_main_account_uids(config.main_account_uids.clone());
        }

        let (hotkey_tx, hotkey_rx) = std::sync::mpsc::channel();
        let _hotkey_guard = crate::overlay_tracker::start_overlay_hotkey_listener(
            cc.egui_ctx.clone(),
            hotkey_tx,
        );

        let mut app = Self {
            watcher,
            rx_ui,
            tray,

            theme: initial_theme,
            language: initial_language,
            watcher_status: WatcherStatus::Running(String::new()),

            mail_path_input: initial_path,
            selected_mode: initial_mode,
            selected_account: None,

            selected_date_preset: DateFilterPreset::AllTime,
            date_from_input: String::new(),
            date_to_input: String::new(),

            current_tab: 0,
            battle_search: String::new(),
            item_search: String::new(),
            item_category_filter: "Tất cả".to_string(),
            log_auto_scroll: true,

            battle_sort_col: Some(BattleSortColumn::Time),
            battle_sort_dir: SortDirection::Descending,
            item_sort_col: Some(ItemSortColumn::TotalQty),
            item_sort_dir: SortDirection::Descending,

            console_logs: VecDeque::new(),
            selected_battle_detail: None,
            speedup_unit: initial_speedup_unit,
            close_behavior: initial_close_behavior,
            autostart: initial_autostart,
            autostart_minimized: initial_autostart_minimized,
            overlay_config: config.overlay.clone(),
            last_mail_arrival: std::time::Instant::now(),
            overlay_styled_hwnd: None,
            overlay_last_opacity: None,
            pending_overlay_save: false,
            pending_start_minimized_frames: if start_minimized { 5 } else { 0 },
            _single_instance: single_instance,
            hotkey_rx,
            _hotkey_guard,

            cloud_auth: config.cloud_auth.clone(),
            cloud_rx: None,
            sync_worker: Some(sync_worker),
            sync_rx: None,
            restore_rx: None,
            cloud_busy: false,
            has_unbacked_data: initial_has_unbacked,
            main_account_uids: config.main_account_uids.clone(),
            new_main_uid_input: String::new(),
            confirm_delete_non_main: None,

            cached_overlay_records: initial_overlay_records,
            cached_today_run_counts: (0, 0, 0),
            cached_today_pinned_chips: Vec::new(),
            cached_sorted_battle_indices: Vec::new(),
            reports_sort_dirty: true,
            last_applied_theme: None,
            last_records_count: 0,
            last_battle_search: String::new(),
            last_battle_sort_col: Some(BattleSortColumn::Time),
            last_battle_sort_dir: SortDirection::Descending,
            last_battle_mode: initial_mode,
            last_battle_lang: initial_language,
            cached_ap_econ: None,
            ap_econ_dirty: true,
            last_ap_records_count: 0,
            last_ap_mode: initial_mode,
        };

        app.refresh_cached_overlay_records();
        app
    }
}

impl eframe::App for RoKLootWatcherApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _ = crate::tray::get_or_find_main_hwnd();

        while let Ok(()) = self.hotkey_rx.try_recv() {
            if self.overlay_config.hotkey_enabled {
                self.overlay_config.mode = self.overlay_config.mode.next();
                self.save_config();
                ctx.request_repaint();
            }
        }

        self.poll_cloud_events(ctx);

        if crate::tray::check_and_clear_restore_request() {
            self.pending_start_minimized_frames = 0;
        }

        if self.pending_start_minimized_frames > 0 {
            self.pending_start_minimized_frames -= 1;
            crate::tray::hide_main_window();
        }

        if crate::tray::is_quit_requested() {
            self.quit_app();
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            if crate::tray::is_quit_requested() {
                self.quit_app();
            }

            match self.close_behavior {
                CloseBehavior::Tray => {
                    if self.tray.is_some() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        self.hide_to_tray(ctx);
                    } else {
                        self.quit_app();
                    }
                }
                CloseBehavior::Close => {
                    self.quit_app();
                }
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_battle_detail = None;
        }

        let had_events = self.poll_watcher_events(ctx);

        let c = self.colors();
        if self.last_applied_theme != Some(self.theme) {
            self.last_applied_theme = Some(self.theme);
            let mut visuals = if self.theme == AppTheme::Dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            visuals.override_text_color = Some(c.canvas_tint);
            visuals.window_fill = c.ink_deep;
            visuals.panel_fill = c.ink;
            visuals.faint_bg_color = c.ink_deep;
            visuals.extreme_bg_color = if self.theme == AppTheme::Dark {
                c.ink
            } else {
                Color32::from_rgb(0xeb, 0xee, 0xf2)
            };

            visuals.window_shadow = egui::epaint::Shadow::NONE;
            visuals.popup_shadow = egui::epaint::Shadow::NONE;

            visuals.widgets.inactive.rounding = Rounding::same(4.0);
            visuals.widgets.inactive.bg_fill = c.primary;
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, c.canvas_tint);
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, c.field_border);

            visuals.widgets.hovered.rounding = Rounding::same(4.0);
            visuals.widgets.hovered.bg_fill = c.primary_pressed;
            visuals.widgets.hovered.fg_stroke = Stroke::new(
                1.0_f32,
                if self.theme == AppTheme::Dark {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                },
            );
            visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, c.focus_teal);

            visuals.widgets.active.rounding = Rounding::same(4.0);
            visuals.widgets.active.bg_fill = c.primary_pressed;
            visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, c.focus_teal);
            visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, c.focus_teal);

            visuals.widgets.open.rounding = Rounding::same(4.0);
            visuals.widgets.open.bg_fill = c.primary_pressed;

            visuals.window_rounding = Rounding::same(8.0);
            visuals.window_stroke = Stroke::new(1.0_f32, c.hairline);

            visuals.selection.bg_fill = c.primary_pressed;
            visuals.selection.stroke = Stroke::new(1.0_f32, c.focus_teal);

            ctx.set_visuals(visuals);
        }

        if had_events || self.watcher.is_scanning() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        if !crate::tray::is_hidden_to_tray() {
            egui::SidePanel::left("left_sidebar_panel")
                .exact_width(220.0)
                .resizable(false)
                .frame(
                    egui::Frame::none()
                        .fill(c.sidebar_bg)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .inner_margin(Margin::same(12.0)),
                )
                .show(ctx, |ui| {
                    self.render_sidebar(ui, ctx);
                });

            let mut close_inspector = false;
            let inspector_record = self.selected_battle_detail.clone();
            if let Some(ref record) = inspector_record {
                egui::SidePanel::right("inspector_side_panel")
                    .exact_width(380.0)
                    .resizable(false)
                    .frame(
                        egui::Frame::none()
                            .fill(c.ink_deep)
                            .stroke(Stroke::new(1.0_f32, c.hairline))
                            .inner_margin(Margin::same(16.0)),
                    )
                    .show(ctx, |ui| {
                        self.render_side_inspector(ui, record, &mut close_inspector);
                    });
            }
            if close_inspector {
                self.selected_battle_detail = None;
            }

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(c.ink).inner_margin(Margin::same(16.0)))
                .show(ctx, |ui| {
                    match self.current_tab {
                        0 => self.render_view_dashboard(ui),
                        1 => self.render_view_battles(ui),
                        2 => self.render_view_inventory(ui),
                        3 => self.render_view_settings(ui, ctx),
                        _ => self.render_view_dashboard(ui),
                    }
                });

            if let Some(count) = self.confirm_delete_non_main {
                let title = match self.language {
                    AppLanguage::Vietnamese => "Xác nhận xóa báo cáo UID phụ",
                    AppLanguage::English => "Confirm Delete Secondary Reports",
                };
                let mut close_modal = false;
                let mut do_delete = false;

                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("delete_modal_backdrop"),
                ));
                painter.rect_filled(ctx.screen_rect(), 0.0, egui::Color32::from_black_alpha(150));

                egui::Window::new(title)
                    .order(egui::Order::Foreground)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_max_width(380.0);
                        ui.add_space(4.0);

                        let warn_text = match self.language {
                            AppLanguage::Vietnamese => format!(
                                "Bạn có chắc chắn muốn xóa vĩnh viễn {} báo cáo của các nhân vật phụ khỏi cơ sở dữ liệu?\n\nThao tác này sẽ dọn dẹp sạch dữ liệu và không thể hoàn tác.",
                                count
                            ),
                            AppLanguage::English => format!(
                                "Are you sure you want to permanently delete {} secondary character reports from the database?\n\nThis will clean up the database and cannot be undone.",
                                count
                            ),
                        };

                        ui.label(
                            egui::RichText::new(warn_text)
                                .font(egui::FontId::proportional(12.0))
                                .color(c.canvas_tint),
                        );

                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;

                            let cancel_btn = egui::Button::new(
                                egui::RichText::new(match self.language {
                                    AppLanguage::Vietnamese => "Hủy bỏ",
                                    AppLanguage::English => "Cancel",
                                })
                                .font(egui::FontId::proportional(11.5))
                                .color(c.canvas_tint),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.hairline))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([90.0, 28.0], cancel_btn).clicked() {
                                close_modal = true;
                            }

                            let delete_btn = egui::Button::new(
                                egui::RichText::new(match self.language {
                                    AppLanguage::Vietnamese => "Xác nhận xóa",
                                    AppLanguage::English => "Confirm Delete",
                                })
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.error),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.error.linear_multiply(0.7)))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([120.0, 28.0], delete_btn).clicked() {
                                do_delete = true;
                                close_modal = true;
                            }
                        });
                    });

                if do_delete {
                    let deleted = self.watcher.delete_non_main_records();
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    let msg = match self.language {
                        AppLanguage::Vietnamese => format!(
                            "Đã xóa vĩnh viễn {} báo cáo của UID phụ khỏi cơ sở dữ liệu.",
                            deleted
                        ),
                        AppLanguage::English => format!(
                            "Permanently deleted {} secondary UID reports from database.",
                            deleted
                        ),
                    };
                    self.watcher.log("SUCCESS", &msg);
                }
                if close_modal {
                    self.confirm_delete_non_main = None;
                }
            }
        }

        let overlay_id = egui::ViewportId::from_hash_of("rok_overlay_viewport");
        let rok_hwnd = if self.overlay_config.enabled {
            crate::overlay_tracker::find_rok_window()
        } else {
            None
        };
        let rok_found = rok_hwnd.is_some();
        let is_active = rok_hwnd.is_some_and(crate::overlay_tracker::is_rok_active_and_visible);
        let action = crate::overlay_tracker::determine_overlay_visibility_action(
            self.overlay_config.enabled,
            self.overlay_config.mode,
            rok_found,
            is_active,
        );

        match action {
            crate::overlay_tracker::OverlayVisibilityAction::Show => {
                if let Some(hwnd) = rok_hwnd {
                    if let Some(bounds) = crate::overlay_tracker::get_rok_window_bounds(hwnd) {
                        let geom = crate::overlay_tracker::calculate_overlay_geometry(
                            &bounds,
                            &self.overlay_config,
                            ctx.pixels_per_point(),
                        );
                        let builder = egui::ViewportBuilder::default()
                            .with_title("Rise of Kingdoms Loot Overlay")
                            .with_decorations(false)
                            .with_transparent(true)
                            .with_always_on_top()
                            .with_mouse_passthrough(true)
                            .with_active(false)
                            .with_position([geom.pos_x.round(), geom.pos_y.round()])
                            .with_inner_size([geom.width.round(), geom.height.round()])
                            .with_visible(true);

                        let overlay_records = &self.cached_overlay_records;
                        let overlay_cfg = self.overlay_config.clone();
                        let last_mail = self.last_mail_arrival;
                        let run_counts = self.cached_today_run_counts;
                        let pinned_chips = &self.cached_today_pinned_chips;

                        ctx.show_viewport_immediate(overlay_id, builder, |ctx, _class| {
                            crate::overlay_ui::render_overlay_viewport(
                                ctx,
                                overlay_records,
                                &overlay_cfg,
                                last_mail,
                                run_counts,
                                pinned_chips,
                            );
                        });

                        ctx.send_viewport_cmd_to(
                            overlay_id,
                            egui::ViewportCommand::InnerSize(egui::vec2(
                                geom.width.round(),
                                geom.height.round(),
                            )),
                        );
                        ctx.send_viewport_cmd_to(
                            overlay_id,
                            egui::ViewportCommand::OuterPosition(egui::pos2(
                                geom.pos_x.round(),
                                geom.pos_y.round(),
                            )),
                        );
                        ctx.send_viewport_cmd_to(
                            overlay_id,
                            egui::ViewportCommand::Visible(true),
                        );

                        #[cfg(windows)]
                        {
                            let is_valid_hwnd = self.overlay_styled_hwnd.is_some_and(|h| unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(h as _) != 0
                            });
                            let needs_styling = !is_valid_hwnd
                                || self.overlay_last_opacity != Some(self.overlay_config.opacity);
                            if needs_styling {
                                let title_wide: Vec<u16> =
                                    "Rise of Kingdoms Loot Overlay\0".encode_utf16().collect();
                                unsafe {
                                    let vhwnd =
                                        windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                                            std::ptr::null(),
                                            title_wide.as_ptr(),
                                        );
                                    if vhwnd != 0 as _ {
                                        self.overlay_styled_hwnd = Some(vhwnd as isize);
                                        self.overlay_last_opacity =
                                            Some(self.overlay_config.opacity);
                                        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                            vhwnd,
                                            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE,
                                        );
                                        crate::overlay_tracker::apply_click_through_styles(
                                            vhwnd,
                                            self.overlay_config.opacity,
                                        );
                                    } else {
                                        self.overlay_styled_hwnd = None;
                                        self.overlay_last_opacity = None;
                                    }
                                }
                            }
                        }
                    }
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            crate::overlay_tracker::OverlayVisibilityAction::Hide => {
                ctx.send_viewport_cmd_to(overlay_id, egui::ViewportCommand::Visible(false));
                self.overlay_styled_hwnd = None;
                self.overlay_last_opacity = None;
                #[cfg(windows)]
                {
                    let title_wide: Vec<u16> =
                        "Rise of Kingdoms Loot Overlay\0".encode_utf16().collect();
                    unsafe {
                        let vhwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                            std::ptr::null(),
                            title_wide.as_ptr(),
                        );
                        if vhwnd != 0 as _ {
                            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                vhwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE,
                            );
                        }
                    }
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(500));
            }
        }
    }
}
