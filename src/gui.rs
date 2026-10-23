use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::analyzer::parse_date_str;
use crate::model::{
    format_in_millions_vietnamese, format_num, format_speedup_dhm, get_item_converted_display,
    AppConfig, AppTheme, BattleRecord, CloseBehavior, DateFilterPreset, FilterMode,
    SpeedupUnit,
};
use crate::tray::SystemTrayManager;
use crate::watcher::{detect_default_mail_dir, RoKMailWatcher, WatcherStatus, WatcherUiEvent};

// Slate & Charcoal Minimalist Analytics Design System Theme Colors
#[derive(Clone, Copy, Debug)]
pub struct ThemeColors {
    #[allow(dead_code)]
    pub is_dark: bool,
    pub ink: Color32,
    pub ink_deep: Color32,
    pub card_surface: Color32,
    pub sidebar_bg: Color32,
    pub primary: Color32,
    pub primary_pressed: Color32,
    pub brand_mark: Color32,
    pub canvas_tint: Color32,
    pub hairline: Color32,
    pub field_border: Color32,
    pub accent_gold: Color32,
    #[allow(dead_code)]
    pub accent_gold_soft: Color32,
    pub focus_teal: Color32,
    pub status_green: Color32,
    pub text_muted: Color32,
    pub error: Color32,
    pub motte_accent: Color32,
    pub fort_accent: Color32,
    pub barb_accent: Color32,
    pub crystal_cyan: Color32,
    pub speedup_purple: Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            ink: Color32::from_rgb(0x11, 0x13, 0x17),              // #111317 - App deep base ground
            ink_deep: Color32::from_rgb(0x18, 0x1b, 0x20),         // #181b20 - Content panel surface
            card_surface: Color32::from_rgb(0x1e, 0x22, 0x2a),     // #1e222a - Elevated card surface
            sidebar_bg: Color32::from_rgb(0x14, 0x16, 0x1b),       // #14161b - Sleek sidebar
            primary: Color32::from_rgb(0x25, 0x2a, 0x34),          // #252a34 - Action button
            primary_pressed: Color32::from_rgb(0x32, 0x38, 0x46),  // #323846 - Active action
            brand_mark: Color32::from_rgb(0xf4, 0x3f, 0x5e),       // #f43f5e - Vibrant Ruby Rose
            canvas_tint: Color32::from_rgb(0xf1, 0xf5, 0xf9),      // #f1f5f9 - Clean crisp text
            hairline: Color32::from_rgb(0x28, 0x2d, 0x37),         // #282d37 - Subtle rule / border
            field_border: Color32::from_rgb(0x38, 0x3f, 0x4e),     // #383f4e - Input border
            accent_gold: Color32::from_rgb(0xf5, 0x9e, 0x0b),      // #f59e0b - Amber gold for Fort / Gems
            accent_gold_soft: Color32::from_rgb(0xfb, 0xbf, 0x24), // #fbbf24 - Soft amber
            focus_teal: Color32::from_rgb(0x06, 0xb6, 0xd4),       // #06b6d4 - Cyan focus & links
            status_green: Color32::from_rgb(0x10, 0xb9, 0x81),     // #10b981 - Emerald active status
            text_muted: Color32::from_rgb(0x94, 0xa3, 0xb8),       // #94a3b8 - Slate-400 secondary text
            error: Color32::from_rgb(0xef, 0x44, 0x44),            // #ef4444 - Coral error
            motte_accent: Color32::from_rgb(0xf4, 0x3f, 0x5e),     // Rose-500 Motte
            fort_accent: Color32::from_rgb(0xf5, 0x9e, 0x0b),      // Amber-500 Fort
            barb_accent: Color32::from_rgb(0x10, 0xb9, 0x81),      // Emerald-500 Barbarian
            crystal_cyan: Color32::from_rgb(0x06, 0xb6, 0xd4),     // Cyan Crystal
            speedup_purple: Color32::from_rgb(0xa8, 0x55, 0xf7),   // Purple Speedups
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            ink: Color32::from_rgb(0xf8, 0xfa, 0xfc),              // Slate-50 clean light background
            ink_deep: Color32::from_rgb(0xff, 0xff, 0xff),         // Crisp pure white cards
            card_surface: Color32::from_rgb(0xf1, 0xf5, 0xf9),     // Slate-100 card surface
            sidebar_bg: Color32::from_rgb(0xf1, 0xf5, 0xf9),       // Slate-100 sidebar
            primary: Color32::from_rgb(0xe2, 0xe8, 0xf0),          // Light grey button fill
            primary_pressed: Color32::from_rgb(0xcb, 0xd5, 0xe1),  // Hovered button
            brand_mark: Color32::from_rgb(0xe1, 0x1d, 0x48),       // Rose-600
            canvas_tint: Color32::from_rgb(0x0f, 0x17, 0x2a),      // Slate-900 dark text
            hairline: Color32::from_rgb(0xe2, 0xe8, 0xf0),         // Clean slate border
            field_border: Color32::from_rgb(0xcb, 0xd5, 0xe1),     // Input border
            accent_gold: Color32::from_rgb(0xd9, 0x77, 0x06),      // Amber-600 gold
            accent_gold_soft: Color32::from_rgb(0xb4, 0x53, 0x09), // Amber-700
            focus_teal: Color32::from_rgb(0x08, 0x91, 0xb2),       // Cyan-600
            status_green: Color32::from_rgb(0x05, 0x96, 0x69),     // Emerald-600
            text_muted: Color32::from_rgb(0x64, 0x74, 0x8b),       // Slate-500
            error: Color32::from_rgb(0xdc, 0x26, 0x26),            // Red-600
            motte_accent: Color32::from_rgb(0xe1, 0x1d, 0x48),     // Rose Motte
            fort_accent: Color32::from_rgb(0xd9, 0x77, 0x06),      // Amber Fort
            barb_accent: Color32::from_rgb(0x05, 0x96, 0x69),      // Emerald Barbarian
            crystal_cyan: Color32::from_rgb(0x08, 0x91, 0xb2),     // Cyan Crystal
            speedup_purple: Color32::from_rgb(0x93, 0x33, 0xea),   // Purple Speedups
        }
    }
}

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
    NameVi,
    NameEn,
    Category,
    TotalQty,
    Converted,
    Battles,
}

pub struct RoKLootWatcherApp {
    pub watcher: Arc<RoKMailWatcher>,
    pub rx_ui: Receiver<WatcherUiEvent>,
    #[allow(dead_code)]
    pub tray: Option<SystemTrayManager>,

    pub theme: AppTheme,
    status_badge_text: String,
    status_badge_color: Color32,

    mail_path_input: String,
    selected_mode: FilterMode,

    selected_date_preset: DateFilterPreset,
    date_from_input: String,
    date_to_input: String,

    current_tab: usize,
    battle_search: String,
    item_search: String,
    item_category_filter: String,
    log_auto_scroll: bool,

    battle_sort_col: Option<BattleSortColumn>,
    battle_sort_dir: SortDirection,
    item_sort_col: Option<ItemSortColumn>,
    item_sort_dir: SortDirection,

    console_logs: VecDeque<String>,
    selected_battle_detail: Option<BattleRecord>,
    speedup_unit: SpeedupUnit,
    pub close_behavior: CloseBehavior,
    pub autostart: bool,
    pub autostart_minimized: bool,
    pending_start_minimized_frames: u32,
    is_armored: bool,
    _single_instance: Option<crate::single_instance::windows::SingleInstanceGuard>,
}

#[allow(clippy::too_many_arguments)]
fn render_sort_header_cell(
    ui: &mut egui::Ui,
    label: &str,
    col: BattleSortColumn,
    active_col: Option<BattleSortColumn>,
    dir: SortDirection,
    width: f32,
    align: egui::Align,
    color: Color32,
    active_color: Color32,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
    let is_active = active_col == Some(col);
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), Color32::from_white_alpha(15));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let arrow = if is_active {
        match dir {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    } else {
        ""
    };

    let text = format!("{}{}", label, arrow);
    let text_color = if is_active { active_color } else if is_hovered { Color32::WHITE } else { color };

    let text_pos = match align {
        egui::Align::LEFT => egui::pos2(rect.min.x + 2.0, rect.center().y),
        egui::Align::Center => rect.center(),
        egui::Align::RIGHT => egui::pos2(rect.max.x - 2.0, rect.center().y),
    };
    let anchor = match align {
        egui::Align::LEFT => egui::Align2::LEFT_CENTER,
        egui::Align::Center => egui::Align2::CENTER_CENTER,
        egui::Align::RIGHT => egui::Align2::RIGHT_CENTER,
    };
    ui.painter().text(
        text_pos,
        anchor,
        text,
        egui::FontId::proportional(12.0),
        text_color,
    );

    let tooltip = if is_active {
        match dir {
            SortDirection::Descending => format!("{}: Đang giảm dần (nhấp để chuyển tăng dần)", label),
            SortDirection::Ascending => format!("{}: Đang tăng dần (nhấp để chuyển giảm dần)", label),
        }
    } else {
        format!("Nhấp để sắp xếp theo {}", label)
    };
    let clicked = resp.clicked();
    resp.on_hover_text(tooltip);

    clicked
}

#[allow(clippy::too_many_arguments)]
fn render_item_sort_header_cell(
    ui: &mut egui::Ui,
    label: &str,
    col: ItemSortColumn,
    active_col: Option<ItemSortColumn>,
    dir: SortDirection,
    width: f32,
    align: egui::Align,
    color: Color32,
    active_color: Color32,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
    let is_active = active_col == Some(col);
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), Color32::from_white_alpha(15));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let arrow = if is_active {
        match dir {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    } else {
        ""
    };

    let text = format!("{}{}", label, arrow);
    let text_color = if is_active { active_color } else if is_hovered { Color32::WHITE } else { color };

    let text_pos = match align {
        egui::Align::LEFT => egui::pos2(rect.min.x + 2.0, rect.center().y),
        egui::Align::Center => rect.center(),
        egui::Align::RIGHT => egui::pos2(rect.max.x - 2.0, rect.center().y),
    };
    let anchor = match align {
        egui::Align::LEFT => egui::Align2::LEFT_CENTER,
        egui::Align::Center => egui::Align2::CENTER_CENTER,
        egui::Align::RIGHT => egui::Align2::RIGHT_CENTER,
    };
    ui.painter().text(
        text_pos,
        anchor,
        text,
        egui::FontId::proportional(12.0),
        text_color,
    );

    let tooltip = if is_active {
        match dir {
            SortDirection::Descending => format!("{}: Đang giảm dần (nhấp để chuyển tăng dần)", label),
            SortDirection::Ascending => format!("{}: Đang tăng dần (nhấp để chuyển giảm dần)", label),
        }
    } else {
        format!("Nhấp để sắp xếp theo {}", label)
    };
    let clicked = resp.clicked();
    resp.on_hover_text(tooltip);

    clicked
}

impl RoKLootWatcherApp {
    pub fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let win_dir = std::env::var("WINDIR")
            .or_else(|_| std::env::var("SystemRoot"))
            .unwrap_or_else(|_| r"C:\Windows".to_string());
        let fonts_dir = PathBuf::from(win_dir).join("Fonts");

        // 1. Phông chữ Proportional chính (Segoe UI - chuẩn Windows với đầy đủ dấu tiếng Việt)
        let proportional_candidates = [
            ("segoe_ui", fonts_dir.join("segoeui.ttf")),
            ("arial", fonts_dir.join("arial.ttf")),
            ("tahoma", fonts_dir.join("tahoma.ttf")),
        ];

        let mut primary_name = None;
        for (name, path) in proportional_candidates {
            if let Ok(font_bytes) = std::fs::read(&path) {
                fonts.font_data.insert(
                    name.to_owned(),
                    egui::FontData::from_owned(font_bytes),
                );
                primary_name = Some(name.to_owned());
                break;
            }
        }

        if let Some(name) = primary_name {
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, name);
        }

        // 2. Phông chữ Monospace (Consolas / Lucida Console)
        let mono_candidates = [
            ("consolas", fonts_dir.join("consola.ttf")),
            ("lucida_console", fonts_dir.join("lucon.ttf")),
        ];
        for (name, path) in mono_candidates {
            if let Ok(font_bytes) = std::fs::read(&path) {
                fonts.font_data.insert(
                    name.to_owned(),
                    egui::FontData::from_owned(font_bytes),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, name.to_owned());
                break;
            }
        }

        // 3. Phông chữ biểu tượng hệ thống (Segoe UI Symbol / Emoji nếu có)
        let emoji_path = fonts_dir.join("seguiemj.ttf");
        if let Ok(font_bytes) = std::fs::read(&emoji_path) {
            fonts.font_data.insert(
                "segoe_ui_emoji".to_owned(),
                egui::FontData::from_owned(font_bytes),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("segoe_ui_emoji".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("segoe_ui_emoji".to_owned());
        }

        ctx.set_fonts(fonts);
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

        // Start watcher initial scan
        watcher.start(true);

        if let Some(ref tray_mgr) = tray {
            tray_mgr.setup_event_handlers(cc.egui_ctx.clone(), Arc::clone(&watcher));
        }

        if let Some(ref guard) = single_instance {
            crate::single_instance::windows::start_activation_listener(
                guard,
                cc.egui_ctx.clone(),
                Arc::clone(&watcher),
            );
        }

        let config = AppConfig::load();
        let initial_speedup_unit = config.speedup_unit;
        let initial_close_behavior = config.close_behavior;
        let initial_theme = config.theme;
        let is_armored = crate::armor::is_mailcache_armored(&watcher.get_mail_dir());

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

        if start_minimized {
            crate::tray::hide_main_window();
            watcher.log(
                "INFO",
                "Ứng dụng đã tự động khởi chạy và thu nhỏ xuống Khay hệ thống (System Tray).",
            );
        }

        let theme_colors = match initial_theme {
            AppTheme::Dark => ThemeColors::dark(),
            AppTheme::Light => ThemeColors::light(),
        };

        Self {
            watcher,
            rx_ui,
            tray,

            theme: initial_theme,
            status_badge_text: "● Đang theo dõi".into(),
            status_badge_color: theme_colors.status_green,

            mail_path_input: initial_path,
            selected_mode: initial_mode,

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
            pending_start_minimized_frames: if start_minimized { 5 } else { 0 },
            is_armored,
            _single_instance: single_instance,
        }
    }

    fn save_config(&self) {
        let config = AppConfig {
            close_behavior: self.close_behavior,
            speedup_unit: self.speedup_unit,
            mail_dir: Some(self.watcher.get_mail_dir()),
            mode: Some(self.selected_mode),
            theme: self.theme,
            language: Default::default(),
            autostart: self.autostart,
            autostart_minimized: self.autostart_minimized,
        };
        config.save();
    }

    pub fn quit_app(&self) {
        self.watcher.stop();
        if let Some(ref t) = self.tray {
            t.hide_icon();
        }
        std::process::exit(0);
    }

    fn hide_to_tray(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        crate::tray::hide_main_window();
        self.watcher.log(
            "INFO",
            "Ứng dụng đã thu nhỏ xuống Khay hệ thống (System Tray). Nhấp vào biểu tượng ở khay để mở lại bất cứ lúc nào.",
        );
    }

    fn poll_watcher_events(&mut self, ctx: &egui::Context) -> bool {
        let mut had_events = false;
        let c = self.colors();
        while let Ok(evt) = self.rx_ui.try_recv() {
            had_events = true;
            match evt {
                WatcherUiEvent::Log { level: _, message } => {
                    self.console_logs.push_back(message);
                    if self.console_logs.len() > 1000 {
                        self.console_logs.pop_front();
                    }
                }
                WatcherUiEvent::StatusChanged(status) => match status {
                    WatcherStatus::Running(_text) => {
                        self.status_badge_text = "● Đang theo dõi".into();
                        self.status_badge_color = c.status_green;
                        ctx.request_repaint();
                    }
                    WatcherStatus::Scanning(_text) => {
                        self.status_badge_text = "● Đang quét...".into();
                        self.status_badge_color = c.accent_gold;
                        ctx.request_repaint();
                    }
                    WatcherStatus::Stopped(_text) => {
                        self.status_badge_text = "● Đã tạm dừng".into();
                        self.status_badge_color = c.text_muted;
                        ctx.request_repaint();
                    }
                    WatcherStatus::Error(_text) => {
                        self.status_badge_text = "● Lỗi watcher".into();
                        self.status_badge_color = c.error;
                        ctx.request_repaint();
                    }
                },
                WatcherUiEvent::StatsUpdated { .. } => {
                    ctx.request_repaint();
                }
                WatcherUiEvent::MailDetected { .. } => {
                    ctx.request_repaint();
                }
            }
        }
        had_events
    }

    fn toggle_watcher(&self) {
        if self.watcher.is_watching() {
            self.watcher.stop();
        } else {
            self.watcher.start(false);
        }
    }

    fn apply_date_preset(&mut self, preset: DateFilterPreset) {
        self.selected_date_preset = preset.clone();
        let today_vn = crate::analyzer::current_date_vn();
        match preset {
            DateFilterPreset::AllTime => {
                self.watcher.set_date_filter(None, None, None);
            }
            DateFilterPreset::Today => {
                self.watcher.set_date_filter(Some(today_vn), Some(today_vn), None);
            }
            DateFilterPreset::Yesterday => {
                let yest_vn = today_vn - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));
                self.watcher.set_date_filter(Some(yest_vn), Some(yest_vn), None);
            }
            DateFilterPreset::Last3Days => {
                let start = today_vn - chrono::Duration::try_days(2).unwrap_or_else(|| chrono::Duration::days(2));
                self.watcher.set_date_filter(Some(start), Some(today_vn), None);
            }
            DateFilterPreset::Last7Days => {
                let start = today_vn - chrono::Duration::try_days(6).unwrap_or_else(|| chrono::Duration::days(6));
                self.watcher.set_date_filter(Some(start), Some(today_vn), None);
            }
            DateFilterPreset::Last14Days => {
                let start = today_vn - chrono::Duration::try_days(13).unwrap_or_else(|| chrono::Duration::days(13));
                self.watcher.set_date_filter(Some(start), Some(today_vn), None);
            }
            DateFilterPreset::Last30Days => {
                let start = today_vn - chrono::Duration::try_days(29).unwrap_or_else(|| chrono::Duration::days(29));
                self.watcher.set_date_filter(Some(start), Some(today_vn), None);
            }
            DateFilterPreset::Custom => {
                if self.date_from_input.is_empty() {
                    let start = today_vn - chrono::Duration::try_days(6).unwrap_or_else(|| chrono::Duration::days(6));
                    self.date_from_input = start.to_string();
                }
                if self.date_to_input.is_empty() {
                    self.date_to_input = today_vn.to_string();
                }
                self.apply_custom_date_filter();
            }
        }
    }

    fn apply_custom_date_filter(&mut self) {
        let s = parse_date_str(&self.date_from_input);
        let e = parse_date_str(&self.date_to_input);
        self.selected_date_preset = DateFilterPreset::Custom;
        self.watcher.set_date_filter(s, e, None);
    }
}


impl RoKLootWatcherApp {
    fn export_active_csv(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("rok_battles.csv")
            .add_filter("CSV File", &["csv"])
            .save_file()
        {
            let records = self.watcher.get_active_records();
            let mut w = String::from("\u{FEFF}Time,BattleType,Target,DamagePct,PlunderTier,Crystals,TotalRSS,Exp,Arrows,UnivSpeedupMin,TrainSpeedupMin,HealSpeedupMin\n");
            for r in records.iter() {
                let cry = r.item_quantity(1, 9);
                let rss = r.total_resource_value();
                let exp = r.total_exp_value();
                let arrow = r.item_quantity(2, 145);
                w.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",{},{},{},{},{},{},{},{},{}\n",
                    r.time_vn, r.battle_type, r.target, r.damage_pct, r.plunder_tier,
                    cry, rss, exp, arrow,
                    r.univ_speedup_minutes(), r.training_speedup_minutes(), r.heal_speedup_minutes()
                ));
            }
            if let Err(e) = std::fs::write(&path, w) {
                self.watcher.log("ERROR", &format!("Không thể lưu file CSV: {}", e));
            } else {
                self.watcher.log("SUCCESS", &format!("Đã xuất {} trận đánh ra file CSV: {}", records.len(), path.display()));
            }
        }
    }

    fn export_active_json(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("rok_battles.json")
            .add_filter("JSON File", &["json"])
            .save_file()
        {
            let records = self.watcher.get_active_records();
            let stats = self.watcher.get_stats();
            if let Err(e) = crate::exporters::export_json(&records, &stats, &path) {
                self.watcher.log("ERROR", &format!("Không thể lưu file JSON: {}", e));
            } else {
                self.watcher.log("SUCCESS", &format!("Đã xuất {} trận đánh ra file JSON: {}", records.len(), path.display()));
            }
        }
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let c = self.colors();

        // 1. App Logo & Brand Header
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("RISE OF KINGDOMS")
                        .font(egui::FontId::proportional(10.0))
                        .strong()
                        .color(c.brand_mark),
                );
                ui.add_space(1.0);
                ui.label(
                    egui::RichText::new("Loot Watcher")
                        .font(egui::FontId::proportional(16.0))
                        .strong()
                        .color(c.canvas_tint),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::none()
                    .fill(c.primary)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("v2.0")
                                .font(egui::FontId::proportional(10.0))
                                .strong()
                                .color(c.text_muted),
                        );
                    });
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(8.0);

        // 2. Navigation Items
        let nav_items = [
            (0, "Tổng quan", "Dashboard KPI & phân bổ"),
            (1, "Trận đánh", "Chiến báo & bảng chi tiết"),
            (2, "Kho vật phẩm", "Thống kê & quy đổi kho"),
            (3, "Cài đặt & Log", "Cấu hình, quét & nhật ký"),
        ];

        for (idx, title, desc) in nav_items {
            let is_active = self.current_tab == idx;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
            let is_hovered = resp.hovered();

            let bg_fill = if is_active {
                c.primary_pressed
            } else if is_hovered {
                c.primary
            } else {
                Color32::TRANSPARENT
            };

            ui.painter().rect_filled(rect, Rounding::same(6.0), bg_fill);

            if is_active {
                let bar_rect = egui::Rect::from_min_size(rect.min, egui::vec2(3.5, rect.height()));
                ui.painter().rect_filled(bar_rect, Rounding::same(2.0), c.focus_teal);
            }

            let text_pos = egui::pos2(rect.min.x + 14.0, rect.min.y + 13.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(13.0),
                if is_active || is_hovered { c.canvas_tint } else { c.text_muted },
            );

            let desc_pos = egui::pos2(rect.min.x + 14.0, rect.min.y + 29.0);
            ui.painter().text(
                desc_pos,
                egui::Align2::LEFT_CENTER,
                desc,
                egui::FontId::proportional(9.5),
                c.text_muted.linear_multiply(0.85),
            );

            if resp.clicked() {
                self.current_tab = idx;
            }

            ui.add_space(4.0);
        }

        // 3. Bottom Widget: Watcher Status & Utility Controls
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(4.0);

            // Utility buttons row
            ui.horizontal(|ui| {
                let theme_label = if self.theme == AppTheme::Dark { "Sáng" } else { "Tối" };
                if ui.button(theme_label).on_hover_text("Chuyển đổi giao diện Sáng / Tối").clicked() {
                    self.theme = self.theme.toggle();
                    self.save_config();
                }

                if ui.button("Khay").on_hover_text("Thu nhỏ cửa sổ xuống khay hệ thống (System Tray)").clicked() {
                    self.hide_to_tray(ctx);
                }
            });

            ui.add_space(8.0);

            // Mini Watcher Status Card
            egui::Frame::none()
                .fill(c.ink)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&self.status_badge_text)
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(self.status_badge_color),
                        );
                    });

                    ui.add_space(2.0);
                    let vault_count = self.watcher.get_vault_total_count();
                    ui.label(
                        egui::RichText::new(format!("Kho Vault: {} trận", format_num(vault_count)))
                            .font(egui::FontId::proportional(10.0))
                            .color(c.text_muted),
                    );

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Quét").on_hover_text("Quét gia tăng các file mail mới (~0ms)").clicked() {
                            self.watcher.rescan();
                        }
                        let toggle_txt = if self.watcher.is_watching() { "Dừng" } else { "Bật" };
                        if ui.button(toggle_txt).clicked() {
                            self.toggle_watcher();
                        }
                    });
                });

            ui.add_space(8.0);
            ui.separator();
        });
    }

    fn render_view_dashboard(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // Dashboard Header
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Tổng Quan Hoạt Động & Chiến Lợi Phẩm")
                        .font(egui::FontId::proportional(18.0))
                        .strong()
                        .color(c.canvas_tint),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Theo dõi và phân tích dữ liệu phần thưởng thời gian thực từ game Rise of Kingdoms")
                        .font(egui::FontId::proportional(12.0))
                        .color(c.text_muted),
                );
            });

            ui.add_space(10.0);

            // Dashboard Inline Filter Toolbar: Report Type, Speedup Unit & Date Filter
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    // Row 1: Loại báo cáo & Thời gian
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Loại báo cáo:").strong().color(c.canvas_tint));
                        let prev_mode = self.selected_mode;
                        ui.selectable_value(&mut self.selected_mode, FilterMode::AllTargets, "Tất cả");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::Motte, "Motte");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::Fort, "Pháo Đài");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::Barbarian, "Dã Man");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::AllForts, "Cả 2 Pháo Đài");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::AllRewards, "Tất cả quà");
                        ui.selectable_value(&mut self.selected_mode, FilterMode::Gathering, "Thu Thập");

                        if self.selected_mode != prev_mode {
                            self.watcher.set_mode(self.selected_mode);
                            self.save_config();
                        }

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new("Thời gian:").strong().color(c.canvas_tint));
                        let prev_preset = self.selected_date_preset.clone();
                        egui::ComboBox::from_id_salt("dash_date_combo")
                            .selected_text(self.selected_date_preset.label())
                            .width(135.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::AllTime, DateFilterPreset::AllTime.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Today, DateFilterPreset::Today.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Yesterday, DateFilterPreset::Yesterday.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last3Days, DateFilterPreset::Last3Days.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last7Days, DateFilterPreset::Last7Days.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last14Days, DateFilterPreset::Last14Days.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last30Days, DateFilterPreset::Last30Days.label());
                                ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Custom, DateFilterPreset::Custom.label());
                            });
                        if self.selected_date_preset != prev_preset {
                            let p = self.selected_date_preset.clone();
                            self.apply_date_preset(p);
                        }

                        if self.selected_date_preset == DateFilterPreset::Custom {
                            ui.label(egui::RichText::new("Từ:").color(c.text_muted));
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_from_input));
                            ui.label(egui::RichText::new("Đến:").color(c.text_muted));
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_to_input));
                            if ui.button("Lọc").clicked() {
                                self.apply_custom_date_filter();
                            }
                        }

                        let cur_battles = self.watcher.get_stats().total_battles;
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{} trận ghi nhận", format_num(cur_battles)))
                                    .strong()
                                    .color(c.focus_teal),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Row 2: Chuyển đổi đơn vị Tăng tốc (Phút / Giờ / Ngày / DHM)
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Đổi đơn vị tăng tốc:").strong().color(c.canvas_tint));
                        let prev_unit = self.speedup_unit;
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Minutes, "Phút");
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Hours, "Giờ");
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Days, "Ngày");
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Dhm, "Ngày/Giờ/Phút (DHM)");
                        if self.speedup_unit != prev_unit {
                            self.save_config();
                        }

                        let cur_stats = self.watcher.get_stats();
                        let (tr, he, un) = cur_stats.compute_speedups_by_category();
                        let spd_total = tr + he + un;
                        if spd_total > 0 {
                            ui.add_space(10.0);
                            let equiv_text = if self.speedup_unit == SpeedupUnit::Dhm {
                                format!("≈ {:.1} giờ ({} phút)", spd_total as f64 / 60.0, format_num(spd_total))
                            } else {
                                format!("≈ {}", format_speedup_dhm(spd_total))
                            };
                            ui.label(
                                egui::RichText::new(equiv_text)
                                    .font(egui::FontId::proportional(11.5))
                                    .strong()
                                    .color(c.status_green),
                            );
                        }
                    });
                });

            ui.add_space(12.0);

            // Compute statistics with guaranteed up-to-date filter mode and speedup unit
            let stats = self.watcher.get_stats();
            let total_battles = stats.total_battles;
            let res = stats.compute_resources();
            let (train_spd_mins, heal_spd_mins, univ_spd_mins) = stats.compute_speedups_by_category();
            let total_spd_mins = train_spd_mins + heal_spd_mins + univ_spd_mins;
            let pha_le = stats.compute_crystals();
            let avg_crystal = if total_battles > 0 { (pha_le as f64) / (total_battles as f64) } else { 0.0 };
            let (univ_spd_val, univ_spd_unit) = self.speedup_unit.format_kpi(total_spd_mins);

            // 4 Primary KPI Cards
            let kpi_cards = [
                (
                    "TỔNG SỐ TRẬN ĐÁNH",
                    format!("{} trận", format_num(total_battles)),
                    format!("Bộ lọc: {}", self.selected_mode.display_label()),
                    c.canvas_tint,
                ),
                (
                    "PHA LÊ THU HOẠCH",
                    format!("{} viên", format_num(pha_le)),
                    format!("Trung bình: {:.1} pha lê / trận", avg_crystal),
                    c.crystal_cyan,
                ),
                (
                    "TỔNG TĂNG TỐC",
                    if self.speedup_unit == SpeedupUnit::Dhm {
                        univ_spd_val.clone()
                    } else {
                        format!("{} {}", univ_spd_val, univ_spd_unit)
                    },
                    format!(
                        "Chung: {} | Luyện: {} | Hồi: {}",
                        self.speedup_unit.format_duration(univ_spd_mins as f64),
                        self.speedup_unit.format_duration(train_spd_mins as f64),
                        self.speedup_unit.format_duration(heal_spd_mins as f64)
                    ),
                    c.speedup_purple,
                ),
                (
                    "TỔNG TÀI NGUYÊN (RSS)",
                    format_in_millions_vietnamese(res.total_rss_val()),
                    format!("{:.2}M tài nguyên các loại", res.total_rss_val() as f64 / 1_000_000.0),
                    c.accent_gold,
                ),
            ];

            ui.columns(4, |cols| {
                for (i, (title, val, sub, color)) in kpi_cards.iter().enumerate() {
                    egui::Frame::none()
                        .fill(c.card_surface)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::same(12.0))
                        .show(&mut cols[i], |ui| {
                            ui.label(egui::RichText::new(*title).font(egui::FontId::proportional(11.0)).strong().color(c.text_muted));
                            ui.add_space(4.0);
                            let font = if val.len() > 14 {
                                egui::FontId::proportional(14.5)
                            } else {
                                egui::FontId::monospace(18.0)
                            };
                            ui.label(egui::RichText::new(val).font(font).strong().color(*color));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(sub).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                }
            });

            ui.add_space(14.0);

            // Distribution Stacked Bar
            let active_records = self.watcher.get_active_records();
            let mut motte_cnt = 0usize;
            let mut fort_cnt = 0usize;
            let mut barb_cnt = 0usize;
            for r in active_records.iter() {
                match r.battle_type.as_str() {
                    "motte" => motte_cnt += 1,
                    "fort" => fort_cnt += 1,
                    "barbarian" => barb_cnt += 1,
                    _ => {}
                }
            }
            let total_counted = (motte_cnt + fort_cnt + barb_cnt).max(1);
            let motte_pct = (motte_cnt as f32 / total_counted as f32) * 100.0;
            let fort_pct = (fort_cnt as f32 / total_counted as f32) * 100.0;
            let barb_pct = (barb_cnt as f32 / total_counted as f32) * 100.0;

            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("TỶ LỆ PHÂN BỔ LOẠI TRẬN ĐÁNH").font(egui::FontId::proportional(11.5)).strong().color(c.canvas_tint));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!("Tổng: {} trận", format_num(active_records.len()))).color(c.text_muted).font(egui::FontId::proportional(11.0)));
                        });
                    });

                    ui.add_space(8.0);

                    let bar_width = ui.available_width();
                    let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, 10.0), egui::Sense::hover());
                    ui.painter().rect_filled(bar_rect, Rounding::same(5.0), c.primary);

                    let motte_w = bar_width * (motte_pct / 100.0);
                    let fort_w = bar_width * (fort_pct / 100.0);
                    let barb_w = bar_width * (barb_pct / 100.0);

                    let mut cur_x = bar_rect.min.x;
                    if motte_w > 0.5 {
                        let r = egui::Rect::from_min_size(egui::pos2(cur_x, bar_rect.min.y), egui::vec2(motte_w, 10.0));
                        ui.painter().rect_filled(r, Rounding::same(3.0), c.motte_accent);
                        cur_x += motte_w;
                    }
                    if fort_w > 0.5 {
                        let r = egui::Rect::from_min_size(egui::pos2(cur_x, bar_rect.min.y), egui::vec2(fort_w, 10.0));
                        ui.painter().rect_filled(r, Rounding::same(3.0), c.fort_accent);
                        cur_x += fort_w;
                    }
                    if barb_w > 0.5 {
                        let r = egui::Rect::from_min_size(egui::pos2(cur_x, bar_rect.min.y), egui::vec2(barb_w, 10.0));
                        ui.painter().rect_filled(r, Rounding::same(3.0), c.barb_accent);
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("● Motte Destroyed: {} ({:.1}%)", motte_cnt, motte_pct)).color(c.motte_accent).font(egui::FontId::proportional(11.0)));
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("● Pháo Đài Dã Man: {} ({:.1}%)", fort_cnt, fort_pct)).color(c.fort_accent).font(egui::FontId::proportional(11.0)));
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("● Dã Man Thường: {} ({:.1}%)", barb_cnt, barb_pct)).color(c.barb_accent).font(egui::FontId::proportional(11.0)));
                    });
                });

            ui.add_space(14.0);

            // 2-Column Grid: Top 10 High-value Items (Left) + 5 Latest Battles (Right)
            ui.columns(2, |cols| {
                // Col 0: Top 10 Items
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(12.0))
                    .show(&mut cols[0], |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("TOP 10 VẬT PHẨM THU HOẠCH NHIỀU NHẤT").font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Xem tất cả →").clicked() {
                                    self.current_tab = 2;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let mut sorted_items: Vec<_> = stats.item_stats.values().collect();
                        sorted_items.sort_by_key(|a| std::cmp::Reverse(a.total_quantity));
                        let top_items: Vec<_> = sorted_items.into_iter().take(10).collect();
                        let max_qty = top_items.first().map(|it| it.total_quantity).unwrap_or(1).max(1);

                        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                            if top_items.is_empty() {
                                ui.label(egui::RichText::new("Chưa có dữ liệu vật phẩm.").color(c.text_muted));
                            } else {
                                for (idx, it) in top_items.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let rank_color = match idx {
                                            0 => c.accent_gold,
                                            1 => Color32::from_rgb(0xc0, 0xc0, 0xc0),
                                            2 => Color32::from_rgb(0xcd, 0x7f, 0x32),
                                            _ => c.text_muted,
                                        };
                                        ui.label(egui::RichText::new(format!("#{}", idx + 1)).strong().color(rank_color).monospace());
                                        ui.label(egui::RichText::new(&it.name_vi).strong().color(c.canvas_tint));

                                        if let Some(conv_str) = get_item_converted_display(it.itype, it.sub_type, it.total_quantity) {
                                            ui.label(egui::RichText::new(format!("({})", conv_str)).font(egui::FontId::proportional(10.5)).color(c.status_green));
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new(format_num(it.total_quantity)).monospace().strong().color(c.accent_gold));
                                        });
                                    });

                                    let pct = (it.total_quantity as f32 / max_qty as f32).clamp(0.02, 1.0);
                                    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
                                    ui.painter().rect_filled(r, Rounding::same(2.0), c.primary);
                                    let filled_r = egui::Rect::from_min_size(r.min, egui::vec2(r.width() * pct, 4.0));
                                    ui.painter().rect_filled(filled_r, Rounding::same(2.0), c.focus_teal);

                                    ui.add_space(4.0);
                                }
                            }
                        });
                    });

                // Col 1: 5 Latest Battles (Realtime Feed)
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(12.0))
                    .show(&mut cols[1], |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("5 CHIẾN BÁO MỚI NHẤT (REALTIME)").font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Xem bảng trận →").clicked() {
                                    self.current_tab = 1;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let latest_battles = crate::analyzer::get_latest_records(&active_records, 5);

                        if latest_battles.is_empty() {
                            ui.label(egui::RichText::new("Chưa có chiến báo nào được ghi nhận.").color(c.text_muted));
                        } else {
                            for r in latest_battles {
                                let (card_rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), egui::Sense::click());
                                let is_hovered = resp.hovered();

                                ui.painter().rect_filled(
                                    card_rect,
                                    Rounding::same(5.0),
                                    if is_hovered { c.primary_pressed } else { c.ink },
                                );
                                ui.painter().rect_stroke(
                                    card_rect,
                                    Rounding::same(5.0),
                                    Stroke::new(1.0_f32, if is_hovered { c.focus_teal } else { c.hairline }),
                                );

                                let badge_color = match r.battle_type.as_str() {
                                    "motte" => c.motte_accent,
                                    "fort" => c.fort_accent,
                                    _ => c.barb_accent,
                                };

                                let t_pos = egui::pos2(card_rect.min.x + 8.0, card_rect.min.y + 14.0);
                                ui.painter().text(
                                    t_pos,
                                    egui::Align2::LEFT_CENTER,
                                    &r.target,
                                    egui::FontId::proportional(12.5),
                                    c.canvas_tint,
                                );

                                let time_pos = egui::pos2(card_rect.max.x - 8.0, card_rect.min.y + 14.0);
                                ui.painter().text(
                                    time_pos,
                                    egui::Align2::RIGHT_CENTER,
                                    &r.time_vn,
                                    egui::FontId::proportional(11.0),
                                    c.text_muted,
                                );

                                let d_pos = egui::pos2(card_rect.min.x + 8.0, card_rect.min.y + 36.0);
                                let cry = r.item_quantity(1, 9);
                                let cry_str = if cry > 0 { format!("+{} Pha Lê  |  ", format_num(cry)) } else { "".to_string() };
                                let dmg_str = format!("ST: {}%", r.damage_pct);
                                let sub_info = format!("[{}]  {}{}", r.battle_type.to_uppercase(), cry_str, dmg_str);
                                ui.painter().text(
                                    d_pos,
                                    egui::Align2::LEFT_CENTER,
                                    sub_info,
                                    egui::FontId::proportional(11.0),
                                    badge_color,
                                );

                                let action_pos = egui::pos2(card_rect.max.x - 8.0, card_rect.min.y + 36.0);
                                ui.painter().text(
                                    action_pos,
                                    egui::Align2::RIGHT_CENTER,
                                    "Chi tiết →",
                                    egui::FontId::proportional(11.0),
                                    if is_hovered { c.focus_teal } else { c.text_muted },
                                );

                                if resp.clicked() {
                                    self.selected_battle_detail = Some(r.clone());
                                }

                                ui.add_space(4.0);
                            }
                        }
                    });
            });
        });
    }

    fn render_view_battles(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();

        // 1. Inline Toolbar
        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                // Row 1: Target Mode & Date Filter
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Loại báo cáo:").strong().color(c.canvas_tint));
                    let prev_mode = self.selected_mode;
                    ui.selectable_value(&mut self.selected_mode, FilterMode::AllTargets, "Tất cả");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::Motte, "Motte");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::Fort, "Pháo Đài");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::Barbarian, "Dã Man");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::AllForts, "Cả 2 Pháo Đài");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::AllRewards, "Tất cả quà");
                    ui.selectable_value(&mut self.selected_mode, FilterMode::Gathering, "Thu Thập");

                    if self.selected_mode != prev_mode {
                        self.watcher.set_mode(self.selected_mode);
                        self.save_config();
                    }

                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Thời gian:").strong().color(c.canvas_tint));
                    let prev_preset = self.selected_date_preset.clone();
                    egui::ComboBox::from_id_salt("battles_date_combo")
                        .selected_text(self.selected_date_preset.label())
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::AllTime, DateFilterPreset::AllTime.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Today, DateFilterPreset::Today.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Yesterday, DateFilterPreset::Yesterday.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last3Days, DateFilterPreset::Last3Days.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last7Days, DateFilterPreset::Last7Days.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last14Days, DateFilterPreset::Last14Days.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Last30Days, DateFilterPreset::Last30Days.label());
                            ui.selectable_value(&mut self.selected_date_preset, DateFilterPreset::Custom, DateFilterPreset::Custom.label());
                        });

                    if self.selected_date_preset != prev_preset {
                        let p = self.selected_date_preset.clone();
                        self.apply_date_preset(p);
                    }

                    if self.selected_date_preset == DateFilterPreset::Custom {
                        ui.label(egui::RichText::new("Từ:").color(c.text_muted));
                        ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_from_input));
                        ui.label(egui::RichText::new("Đến:").color(c.text_muted));
                        ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_to_input));
                        if ui.button("Lọc").clicked() {
                            self.apply_custom_date_filter();
                        }
                    }

                    let cur_cnt = self.watcher.get_active_count();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} trận phù hợp", cur_cnt))
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
                });

                ui.add_space(6.0);

                // Row 2: Search, Speedup Unit & Export Actions
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Tìm kiếm:").strong().color(c.canvas_tint));
                    ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut self.battle_search).hint_text("Tên quái, ngày giờ..."));

                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Tăng tốc:").color(c.text_muted));
                    let prev_unit = self.speedup_unit;
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Minutes, "Phút");
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Hours, "Giờ");
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Days, "Ngày");
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Dhm, "DHM");
                    if self.speedup_unit != prev_unit {
                        self.save_config();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Xuất JSON").on_hover_text("Xuất dữ liệu trận đánh ra file JSON").clicked() {
                            self.export_active_json();
                        }
                        if ui.button("Xuất CSV").on_hover_text("Xuất bảng trận đánh ra file CSV Excel").clicked() {
                            self.export_active_csv();
                        }
                    });
                });
            });

        ui.add_space(8.0);

        let records = self.watcher.get_active_records();
        let is_barb = self.selected_mode == FilterMode::Barbarian;
        let is_gathering = self.selected_mode == FilterMode::Gathering;

        // Precise Responsive Layout Metrics
        const TABLE_PAD_X: f32 = 8.0;
        const ITEM_SPACING_X: f32 = 6.0;

        let col_type = if is_gathering { 72.0 } else { 46.0 };
        const COL_TIME: f32 = 116.0;
        const COL_DMG: f32 = 48.0;

        // Gathering specific columns
        const COL_GATHER_LVL: f32 = 60.0;
        const COL_GATHER_RES: f32 = 80.0;
        const COL_GATHER_TOTAL: f32 = 110.0;
        const COL_GATHER_BONUS: f32 = 100.0;

        // Barbarian specific columns (NOW INCLUDES PHA LÊ!)
        const COL_BARB_CRY: f32 = 56.0;
        const COL_BARB_RES: f32 = 60.0;
        const COL_BARB_EXP: f32 = 58.0;
        const COL_BARB_UNIV: f32 = 56.0;
        const COL_BARB_TRAIN: f32 = 56.0;
        const COL_BARB_HEAL: f32 = 56.0;
        const COL_BARB_ARROW: f32 = 46.0;

        // Motte / Fort columns
        const COL_TIER: f32 = 48.0;
        const COL_CRY: f32 = 56.0;
        const COL_PACKS: f32 = 58.0;
        const COL_TOME: f32 = 52.0;
        const COL_UNIV_SPD: f32 = 56.0;
        const COL_HEAL_SPD: f32 = 56.0;

        const COL_VIEW: f32 = 56.0;

        let avail_w = ui.available_width();
        let fixed_w = if is_gathering {
            COL_TIME + col_type + COL_GATHER_LVL + COL_GATHER_RES + COL_GATHER_TOTAL + COL_GATHER_BONUS + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * 7.0)
        } else if is_barb {
            COL_TIME + col_type + COL_DMG + COL_BARB_CRY + COL_BARB_RES + COL_BARB_EXP + COL_BARB_UNIV + COL_BARB_TRAIN + COL_BARB_HEAL + COL_BARB_ARROW + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * 11.0)
        } else {
            COL_TIME + col_type + COL_DMG + COL_TIER + COL_CRY + COL_PACKS + COL_TOME + COL_UNIV_SPD + COL_HEAL_SPD + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * 10.0)
        };

        let min_target = 180.0_f32;
        let min_table_width = fixed_w + min_target;
        let table_width = avail_w.max(min_table_width);
        let col_target = table_width - fixed_w;

        // Table Header
        let mut toggle_sort_col = None;

        egui::Frame::none()
            .fill(c.card_surface)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(TABLE_PAD_X, 4.0))
            .show(ui, |ui| {
                ui.set_width(table_width - (TABLE_PAD_X * 2.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = ITEM_SPACING_X;
                    ui.set_min_height(26.0);

                    if render_sort_header_cell(ui, "Thời Gian (VN)", BattleSortColumn::Time, self.battle_sort_col, self.battle_sort_dir, COL_TIME, egui::Align::LEFT, c.text_muted, c.focus_teal) {
                        toggle_sort_col = Some(BattleSortColumn::Time);
                    }
                    if render_sort_header_cell(ui, "Loại", BattleSortColumn::Type, self.battle_sort_col, self.battle_sort_dir, col_type, egui::Align::Center, c.text_muted, c.focus_teal) {
                        toggle_sort_col = Some(BattleSortColumn::Type);
                    }
                    if render_sort_header_cell(ui, "Mục Tiêu", BattleSortColumn::Target, self.battle_sort_col, self.battle_sort_dir, col_target, egui::Align::LEFT, c.text_muted, c.focus_teal) {
                        toggle_sort_col = Some(BattleSortColumn::Target);
                    }
                    if !is_gathering {
                        if render_sort_header_cell(ui, "Sát Thương", BattleSortColumn::Damage, self.battle_sort_col, self.battle_sort_dir, COL_DMG, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Damage);
                        }
                    }

                    if is_gathering {
                        if render_sort_header_cell(ui, "Cấp Mỏ", BattleSortColumn::Level, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_LVL, egui::Align::Center, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Level);
                        }
                        if render_sort_header_cell(ui, "Tài Nguyên", BattleSortColumn::Target, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_RES, egui::Align::LEFT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Target);
                        }
                        if render_sort_header_cell(ui, "Sản Lượng", BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_TOTAL, egui::Align::RIGHT, c.accent_gold, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, "Điểm Cộng", BattleSortColumn::Damage, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_BONUS, egui::Align::RIGHT, c.status_green, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Damage);
                        }
                    } else if is_barb {
                        if render_sort_header_cell(ui, "Pha Lê", BattleSortColumn::Crystal, self.battle_sort_col, self.battle_sort_dir, COL_BARB_CRY, egui::Align::RIGHT, c.crystal_cyan, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Crystal);
                        }
                        if render_sort_header_cell(ui, "Tổng TN", BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_BARB_RES, egui::Align::RIGHT, c.accent_gold, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, "EXP Tướng", BattleSortColumn::TomeExp, self.battle_sort_col, self.battle_sort_dir, COL_BARB_EXP, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::TomeExp);
                        }
                        if render_sort_header_cell(ui, "TT Đa Năng", BattleSortColumn::UnivSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_UNIV, egui::Align::RIGHT, c.status_green, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::UnivSpeedup);
                        }
                        if render_sort_header_cell(ui, "TT H.Luyện", BattleSortColumn::TrainSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_TRAIN, egui::Align::RIGHT, Color32::from_rgb(0xfb, 0x92, 0x3c), c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::TrainSpeedup);
                        }
                        if render_sort_header_cell(ui, "TT Chữa Lành", BattleSortColumn::HealSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_HEAL, egui::Align::RIGHT, Color32::from_rgb(0x38, 0xbd, 0xf8), c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::HealSpeedup);
                        }
                        if render_sort_header_cell(ui, "Mũi Tên", BattleSortColumn::Arrow, self.battle_sort_col, self.battle_sort_dir, COL_BARB_ARROW, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Arrow);
                        }
                    } else {
                        if render_sort_header_cell(ui, "Plunder", BattleSortColumn::Tier, self.battle_sort_col, self.battle_sort_dir, COL_TIER, egui::Align::Center, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Tier);
                        }
                        if render_sort_header_cell(ui, "Pha Lê", BattleSortColumn::Crystal, self.battle_sort_col, self.battle_sort_dir, COL_CRY, egui::Align::RIGHT, c.crystal_cyan, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Crystal);
                        }
                        if render_sort_header_cell(ui, "Gói TN", BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_PACKS, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, "Sách EXP", BattleSortColumn::TomeExp, self.battle_sort_col, self.battle_sort_dir, COL_TOME, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::TomeExp);
                        }
                        if render_sort_header_cell(ui, "TT Đa Năng", BattleSortColumn::UnivSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_UNIV_SPD, egui::Align::RIGHT, c.status_green, c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::UnivSpeedup);
                        }
                        if render_sort_header_cell(ui, "TT Hồi Máu", BattleSortColumn::HealSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_HEAL_SPD, egui::Align::RIGHT, Color32::from_rgb(0x38, 0xbd, 0xf8), c.focus_teal) {
                            toggle_sort_col = Some(BattleSortColumn::HealSpeedup);
                        }
                    }

                    ui.add_sized(
                        [COL_VIEW, 20.0],
                        egui::Label::new(egui::RichText::new("Chi Tiết").strong().color(c.focus_teal)).halign(egui::Align::Center),
                    );
                });
            });

        if let Some(col) = toggle_sort_col {
            if self.battle_sort_col == Some(col) {
                self.battle_sort_dir = match self.battle_sort_dir {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            } else {
                self.battle_sort_col = Some(col);
                self.battle_sort_dir = SortDirection::Descending;
            }
        }

        ui.add_space(4.0);

        if records.is_empty() {
            ui.add_space(30.0);
            if self.watcher.is_scanning() {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("[Đang quét] Đang quét và giải mã dữ liệu mailcache... Vui lòng đợi trong giây lát.")
                            .font(egui::FontId::proportional(13.0))
                            .color(c.accent_gold),
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Không tìm thấy trận đánh nào phù hợp với bộ lọc hiện tại.")
                            .font(egui::FontId::proportional(14.0))
                            .strong()
                            .color(c.canvas_tint),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("[Gợi ý] Thử đổi 'Thời gian' sang 'Từ trước đến nay' hoặc đổi 'Mục tiêu' sang 'Tất cả' để hiển thị toàn bộ lịch sử.")
                            .font(egui::FontId::proportional(12.0))
                            .color(c.focus_teal),
                    );
                });
            }
            return;
        }

        let search_lower = self.battle_search.to_lowercase();
        let mut sorted_records: Vec<&BattleRecord> = records
            .iter()
            .filter(|r| {
                if search_lower.is_empty() {
                    true
                } else {
                    let hay = format!("{} {} {}", r.target, r.time_vn, r.battle_type).to_lowercase();
                    hay.contains(&search_lower)
                }
            })
            .collect();

        if let Some(sort_col) = self.battle_sort_col {
            sorted_records.sort_by(|a, b| {
                let ordering = match sort_col {
                    BattleSortColumn::Time => a.time_raw.cmp(&b.time_raw),
                    BattleSortColumn::Type => a.battle_type.cmp(&b.battle_type),
                    BattleSortColumn::Target => a.target.cmp(&b.target),
                    BattleSortColumn::Damage => {
                        if is_gathering {
                            a.talent_add.cmp(&b.talent_add)
                        } else {
                            a.damage_pct.cmp(&b.damage_pct)
                        }
                    },
                    BattleSortColumn::Tier => a.plunder_tier.cmp(&b.plunder_tier),
                    BattleSortColumn::Crystal => a.crystal_quantity().cmp(&b.crystal_quantity()),
                    BattleSortColumn::Level => a.target_level.cmp(&b.target_level),
                    BattleSortColumn::Resource => {
                        if is_gathering {
                            let a_val = a.items.first().map(|i| i.value).unwrap_or(0);
                            let b_val = b.items.first().map(|i| i.value).unwrap_or(0);
                            a_val.cmp(&b_val)
                        } else if is_barb {
                            a.total_resource_value().cmp(&b.total_resource_value())
                        } else {
                            a.total_resources_breakdown().choice_packs.cmp(&b.total_resources_breakdown().choice_packs)
                        }
                    }
                    BattleSortColumn::TomeExp => {
                        if is_barb {
                            a.total_exp_value().cmp(&b.total_exp_value())
                        } else {
                            a.item_quantity(2, 130).cmp(&b.item_quantity(2, 130))
                        }
                    }
                    BattleSortColumn::Arrow => a.item_quantity(2, 145).cmp(&b.item_quantity(2, 145)),
                    BattleSortColumn::UnivSpeedup => a.univ_speedup_minutes().cmp(&b.univ_speedup_minutes()),
                    BattleSortColumn::TrainSpeedup => a.training_speedup_minutes().cmp(&b.training_speedup_minutes()),
                    BattleSortColumn::HealSpeedup => a.heal_speedup_minutes().cmp(&b.heal_speedup_minutes()),
                };
                match self.battle_sort_dir {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                }
            });
        }

        let mut clicked_record = None;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(table_width);
                for r in sorted_records {
                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(table_width, 28.0),
                        egui::Sense::click(),
                    );

                    let is_hovered = row_resp.hovered();
                    if is_hovered {
                        ui.painter().rect_filled(
                            row_rect,
                            egui::Rounding::same(5.0),
                            c.focus_teal.linear_multiply(0.12),
                        );
                        ui.painter().rect_stroke(
                            row_rect,
                            egui::Rounding::same(5.0),
                            egui::Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.40)),
                        );
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if row_resp.clicked() {
                        clicked_record = Some(r.clone());
                    }

                    let inner_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x + TABLE_PAD_X, row_rect.min.y),
                        egui::pos2(row_rect.max.x - TABLE_PAD_X, row_rect.max.y),
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = ITEM_SPACING_X;

                            ui.add_sized([COL_TIME, 24.0], egui::Label::new(egui::RichText::new(&r.time_vn).monospace().font(egui::FontId::monospace(11.0))).halign(egui::Align::LEFT));

                            let (type_text, type_badge_color) = match r.battle_type.as_str() {
                                "motte" => ("MOTTE", c.motte_accent),
                                "fort" => ("FORT", c.fort_accent),
                                "gathering" => ("THU THẬP", c.focus_teal),
                                _ => ("DÃ MAN", c.barb_accent),
                            };
                            ui.add_sized([col_type, 24.0], egui::Label::new(egui::RichText::new(type_text).font(egui::FontId::proportional(10.5)).color(type_badge_color).strong()).halign(egui::Align::Center));

                            let target_btn = egui::Button::new(
                                egui::RichText::new(&r.target).strong().color(c.canvas_tint)
                            )
                            .frame(false)
                            .truncate();
                            if ui.add_sized([col_target, 24.0], target_btn).clicked() {
                                clicked_record = Some(r.clone());
                            }

                            if !is_gathering {
                                ui.add_sized([COL_DMG, 24.0], egui::Label::new(egui::RichText::new(format!("{}%", r.damage_pct)).monospace().color(c.focus_teal).strong()).halign(egui::Align::RIGHT));
                            }

                            if is_gathering {
                                let lvl_str = if !r.target_level.is_empty() { format!("Cấp {}", r.target_level) } else { "-".to_string() };
                                ui.add_sized([COL_GATHER_LVL, 24.0], egui::Label::new(egui::RichText::new(lvl_str).strong().color(c.focus_teal)).halign(egui::Align::Center));

                                let res_name = r.items.first().map(|i| i.name_vi.as_str()).unwrap_or("-");
                                ui.add_sized([COL_GATHER_RES, 24.0], egui::Label::new(egui::RichText::new(res_name).strong().color(c.canvas_tint)).halign(egui::Align::LEFT));

                                let total_val = r.items.first().map(|i| i.value).unwrap_or(0);
                                ui.add_sized([COL_GATHER_TOTAL, 24.0], egui::Label::new(egui::RichText::new(format_num(total_val)).monospace().strong().color(c.accent_gold)).halign(egui::Align::RIGHT));

                                let bonus_str = if r.talent_add > 0 { format!("+{}", format_num(r.talent_add)) } else { "-".to_string() };
                                ui.add_sized([COL_GATHER_BONUS, 24.0], egui::Label::new(egui::RichText::new(bonus_str).monospace().color(c.status_green)).halign(egui::Align::RIGHT));
                            } else if is_barb {
                                let cry = r.crystal_quantity();
                                let (cry_str, cry_color) = if cry > 0 {
                                    (format_num(cry), c.crystal_cyan)
                                } else {
                                    ("-".to_string(), c.text_muted)
                                };
                                ui.add_sized([COL_BARB_CRY, 24.0], egui::Label::new(egui::RichText::new(cry_str).monospace().color(cry_color).strong()).halign(egui::Align::RIGHT));

                                let res_breakdown = r.total_resources_breakdown();
                                let res_val = res_breakdown.total_rss_val();
                                let res_str = format_in_millions_vietnamese(res_val);
                                ui.add_sized([COL_BARB_RES, 24.0], egui::Label::new(egui::RichText::new(res_str).monospace().color(c.accent_gold).strong()).halign(egui::Align::RIGHT));

                                let exp_val = r.total_exp_value();
                                ui.add_sized([COL_BARB_EXP, 24.0], egui::Label::new(egui::RichText::new(format_in_millions_vietnamese(exp_val)).monospace()).halign(egui::Align::RIGHT));

                                let univ_spd_min = r.univ_speedup_minutes();
                                let univ_spd_str = self.speedup_unit.format_duration(univ_spd_min as f64);
                                ui.add_sized([COL_BARB_UNIV, 24.0], egui::Label::new(egui::RichText::new(univ_spd_str).monospace().color(c.status_green)).halign(egui::Align::RIGHT));

                                let train_spd_min = r.training_speedup_minutes();
                                let train_spd_str = self.speedup_unit.format_duration(train_spd_min as f64);
                                ui.add_sized([COL_BARB_TRAIN, 24.0], egui::Label::new(egui::RichText::new(train_spd_str).monospace().color(Color32::from_rgb(0xfb, 0x92, 0x3c))).halign(egui::Align::RIGHT));

                                let heal_spd_min = r.heal_speedup_minutes();
                                let heal_spd_str = self.speedup_unit.format_duration(heal_spd_min as f64);
                                ui.add_sized([COL_BARB_HEAL, 24.0], egui::Label::new(egui::RichText::new(heal_spd_str).monospace().color(Color32::from_rgb(0x38, 0xbd, 0xf8))).halign(egui::Align::RIGHT));

                                let arrows = r.item_quantity(2, 145);
                                ui.add_sized([COL_BARB_ARROW, 24.0], egui::Label::new(egui::RichText::new(format_num(arrows)).monospace()).halign(egui::Align::RIGHT));
                            } else {
                                let tier_str = if r.plunder_tier > 0 {
                                    format!("Tier {}", r.plunder_tier)
                                } else {
                                    "-".to_string()
                                };
                                ui.add_sized([COL_TIER, 24.0], egui::Label::new(egui::RichText::new(tier_str).color(c.accent_gold).strong()).halign(egui::Align::Center));

                                let cry = r.crystal_quantity();
                                let cry_str = if cry > 0 { format_num(cry) } else { "-".to_string() };
                                ui.add_sized([COL_CRY, 24.0], egui::Label::new(egui::RichText::new(cry_str).monospace().color(c.crystal_cyan).strong()).halign(egui::Align::RIGHT));

                                let packs = if r.battle_type == "gathering" {
                                    r.items.first().map(|i| i.value).unwrap_or(0)
                                } else {
                                    r.total_resources_breakdown().choice_packs
                                };
                                let pack_label = if r.battle_type == "gathering" {
                                    format_num(packs)
                                } else {
                                    format!("{} gói", format_num(packs))
                                };
                                ui.add_sized([COL_PACKS, 24.0], egui::Label::new(egui::RichText::new(pack_label).monospace().color(c.accent_gold)).halign(egui::Align::RIGHT));

                                let tome = r.item_quantity(2, 130);
                                ui.add_sized([COL_TOME, 24.0], egui::Label::new(egui::RichText::new(format_num(tome)).monospace()).halign(egui::Align::RIGHT));

                                let u_spd_min = r.univ_speedup_minutes();
                                let u_spd_str = self.speedup_unit.format_duration(u_spd_min as f64);
                                ui.add_sized([COL_UNIV_SPD, 24.0], egui::Label::new(egui::RichText::new(u_spd_str).monospace().color(c.status_green)).halign(egui::Align::RIGHT));

                                let h_spd_min = r.heal_speedup_minutes();
                                let h_spd_str = self.speedup_unit.format_duration(h_spd_min as f64);
                                ui.add_sized([COL_HEAL_SPD, 24.0], egui::Label::new(egui::RichText::new(h_spd_str).monospace().color(Color32::from_rgb(0x38, 0xbd, 0xf8))).halign(egui::Align::RIGHT));
                            }

                            let view_btn = egui::Button::new(
                                egui::RichText::new("Chi tiết")
                                    .font(egui::FontId::proportional(11.0))
                                    .strong()
                                    .color(c.focus_teal),
                            )
                            .fill(c.focus_teal.linear_multiply(0.12))
                            .stroke(egui::Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.45)))
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add_sized([COL_VIEW, 22.0], view_btn).clicked() {
                                clicked_record = Some(r.clone());
                            }
                        });
                    });

                    ui.add_space(2.0);
                }
            });

        if let Some(rec) = clicked_record {
            self.selected_battle_detail = Some(rec);
        }
    }

    fn render_view_inventory(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();
        let stats = self.watcher.get_stats();
        let total_battles = stats.total_battles.max(1);

        // 1. Inventory Toolbar
        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Tìm vật phẩm:").strong().color(c.canvas_tint));
                    ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut self.item_search).hint_text("Tên tiếng Việt, Anh..."));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Danh mục:").color(c.text_muted));
                    let categories = ["Tất cả", "Tài Nguyên", "Vật Liệu", "Tăng Tốc", "Pha Lê", "Sách & Tên", "Khác"];
                    for cat in categories {
                        let is_active = self.item_category_filter == cat;
                        if ui.selectable_label(is_active, cat).clicked() {
                            self.item_category_filter = cat.to_string();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} loại vật phẩm", stats.item_stats.len()))
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
                });
            });

        ui.add_space(8.0);

        // 2. Resource breakdown banner if resources exist
        let res = stats.compute_resources();
        if res.total_rss_val() > 0 {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("TỔNG HỢP TÀI NGUYÊN (QUY ĐỔI SỐ LƯỢNG GÓI x GIÁ TRỊ GÓI -> TRIỆU / M):")
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.accent_gold),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Tổng cộng: {:.2}M RSS ({})",
                                    res.total_rss_val() as f64 / 1_000_000.0,
                                    format_num(res.total_rss_val())
                                ))
                                .font(egui::FontId::proportional(12.5))
                                .color(c.status_green)
                                .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new("Thực phẩm (Ngô)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.food_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.accent_gold));
                            ui.label(egui::RichText::new(format!("{} gói (x1.000)", format_num(res.food_packs))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new("Gỗ (Wood)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.wood_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.status_green));
                            ui.label(egui::RichText::new(format!("{} gói (x1.000)", format_num(res.wood_packs))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new("Đá (Stone)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.stone_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(Color32::from_rgb(0x38, 0xbd, 0xf8)));
                            ui.label(egui::RichText::new(format!("{} gói (x750)", format_num(res.stone_packs))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new("Vàng (Gold)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.gold_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(Color32::from_rgb(0xf5, 0x9e, 0x0b)));
                            ui.label(egui::RichText::new(format!("{} gói (x500)", format_num(res.gold_packs))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new("Gói C Tùy Chọn").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} gói", format_num(res.choice_packs))).font(egui::FontId::proportional(15.0)).strong().color(c.focus_teal));
                            ui.label(egui::RichText::new("Số lượng gói").font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                    });
                });

            ui.add_space(8.0);
        }

        // 3. Material & AP breakdown banner if materials or AP exist
        let mats = stats.compute_materials();
        let ap = stats.compute_ap();
        if mats.total_pieces() > 0 || ap.total_bottles() > 0 {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("TỔNG HỢP VẬT LIỆU TRANG BỊ & ĐIỂM HÀNH ĐỘNG (AP):")
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.focus_teal),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Tổng: {} BT | {} NC => {} quy đổi | +{} AP",
                                    format_num(mats.total_normal()),
                                    format_num(mats.total_advanced()),
                                    format_num(mats.total_converted_normal()),
                                    format_num(ap.total_ap_points())
                                ))
                                .font(egui::FontId::proportional(12.5))
                                .color(c.status_green)
                                .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new("Da (Leather)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} quy đổi", format_num(mats.leather_total_converted()))).font(egui::FontId::proportional(14.0)).strong().color(Color32::from_rgb(0xfb, 0x92, 0x3c)));
                            ui.label(egui::RichText::new(format!("BT: {} | NC: {}", format_num(mats.leather_normal), format_num(mats.leather_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new("Quặng Sắt (Iron)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} quy đổi", format_num(mats.iron_total_converted()))).font(egui::FontId::proportional(14.0)).strong().color(Color32::from_rgb(0x94, 0xa3, 0xb8)));
                            ui.label(egui::RichText::new(format!("BT: {} | NC: {}", format_num(mats.iron_normal), format_num(mats.iron_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new("Gỗ Mun (Ebony)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} quy đổi", format_num(mats.ebony_total_converted()))).font(egui::FontId::proportional(14.0)).strong().color(Color32::from_rgb(0x34, 0xd3, 0x99)));
                            ui.label(egui::RichText::new(format!("BT: {} | NC: {}", format_num(mats.ebony_normal), format_num(mats.ebony_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new("Xương (Bone)").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} quy đổi", format_num(mats.bone_total_converted()))).font(egui::FontId::proportional(14.0)).strong().color(Color32::from_rgb(0xf4, 0x72, 0xb6)));
                            ui.label(egui::RichText::new(format!("BT: {} | NC: {}", format_num(mats.bone_normal), format_num(mats.bone_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new("Điểm Hành Động").strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("+{} AP", format_num(ap.total_ap_points()))).font(egui::FontId::proportional(14.0)).strong().color(c.status_green));
                            ui.label(egui::RichText::new(format!("Tổng {} bình", format_num(ap.total_bottles()))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                    });
                });

            ui.add_space(8.0);
        }

        let mut sorted_items: Vec<_> = stats.item_stats.values().collect();
        sorted_items.sort_by_key(|a| std::cmp::Reverse(a.total_quantity));

        let search_lower = self.item_search.trim().to_lowercase();
        let cat_filter = self.item_category_filter.clone();

        let mut filtered_items: Vec<_> = sorted_items
            .into_iter()
            .filter(|it| {
                let matches_search = if search_lower.is_empty() {
                    true
                } else {
                    it.name_vi.to_lowercase().contains(&search_lower)
                        || it.name_en.to_lowercase().contains(&search_lower)
                        || it.category.to_lowercase().contains(&search_lower)
                };

                let matches_cat = match cat_filter.as_str() {
                    "Tài Nguyên" => {
                        it.category == "Tài nguyên"
                            || it.category.to_lowercase().contains("nguyên")
                            || it.category.to_lowercase().contains("resource")
                            || it.name_vi.contains("Đá Quý")
                            || it.name_vi.contains("thực phẩm")
                            || it.name_vi.contains("khối gỗ")
                            || it.name_vi.contains("khối đá")
                            || it.name_vi.contains("đồng vàng")
                    }
                    "Vật Liệu" => {
                        it.category == "Vật liệu"
                            || it.category.to_lowercase().contains("vật liệu")
                            || it.category.to_lowercase().contains("material")
                            || it.name_vi.starts_with("Da")
                            || it.name_vi.starts_with("Quặng sắt")
                            || it.name_vi.starts_with("Gỗ mun")
                            || it.name_vi.starts_with("Xương")
                    }
                    "Tăng Tốc" => {
                        it.category == "Tăng tốc"
                            || it.category.to_lowercase().contains("tốc")
                            || it.category.to_lowercase().contains("speed")
                            || it.name_vi.contains("Tăng tốc")
                    }
                    "Pha Lê" => {
                        it.category == "Pha Lê"
                            || it.name_vi.contains("Pha Lê")
                            || it.name_en.contains("Crystal")
                    }
                    "Sách & Tên" => {
                        it.name_vi.contains("Sách")
                            || it.name_vi.contains("Tên")
                            || it.name_vi.contains("Mũi Tên")
                            || it.name_vi.contains("Tome")
                    }
                    "Khác" => {
                        let c_low = it.category.to_lowercase();
                        c_low != "tài nguyên" && c_low != "vật liệu" && c_low != "tăng tốc" && c_low != "pha lê"
                    }
                    _ => true,
                };

                matches_search && matches_cat
            })
            .collect();

        if let Some(col) = self.item_sort_col {
            filtered_items.sort_by(|a, b| {
                let ordering = match col {
                    ItemSortColumn::NameVi => a.name_vi.cmp(&b.name_vi),
                    ItemSortColumn::NameEn => a.name_en.cmp(&b.name_en),
                    ItemSortColumn::Category => a.category.cmp(&b.category),
                    ItemSortColumn::TotalQty => a.total_quantity.cmp(&b.total_quantity),
                    ItemSortColumn::Converted => {
                        let conv_a = get_item_converted_display(a.itype, a.sub_type, a.total_quantity).unwrap_or_default();
                        let conv_b = get_item_converted_display(b.itype, b.sub_type, b.total_quantity).unwrap_or_default();
                        conv_a.cmp(&conv_b)
                    }
                    ItemSortColumn::Battles => a.battle_count.cmp(&b.battle_count),
                };
                match self.item_sort_dir {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                }
            });
        }

        // Inventory Table Responsive Metrics
        // Inventory Table Responsive Metrics
        const INV_PAD_X: f32 = 8.0;
        const INV_SPACING_X: f32 = 6.0;

        const COL_CAT: f32 = 78.0;
        const COL_TOTAL: f32 = 74.0;
        const COL_CONV: f32 = 100.0;
        const COL_BATTLES: f32 = 52.0;
        const COL_RATE: f32 = 56.0;
        const COL_AVG: f32 = 52.0;
        const COL_MAX: f32 = 52.0;

        let avail_w = ui.available_width();
        // 9 items total, 8 gaps
        let inv_fixed_w = COL_CAT + COL_TOTAL + COL_CONV + COL_BATTLES + COL_RATE + COL_AVG + COL_MAX + (INV_PAD_X * 2.0) + (INV_SPACING_X * 8.0);
        let min_names_w = 340.0_f32; // 190.0 for VI + 150.0 for EN
        let inv_table_width = avail_w.max(inv_fixed_w + min_names_w);
        let names_w = inv_table_width - inv_fixed_w;
        let col_name_vi = (names_w * 0.55).round();
        let col_name_en = names_w - col_name_vi;

        let mut toggle_item_sort_col = None;

        egui::Frame::none()
            .fill(c.card_surface)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(INV_PAD_X, 4.0))
            .show(ui, |ui| {
                ui.set_width(inv_table_width - (INV_PAD_X * 2.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = INV_SPACING_X;
                    ui.set_min_height(26.0);

                    if render_item_sort_header_cell(ui, "Tên Vật Phẩm (VI)", ItemSortColumn::NameVi, self.item_sort_col, self.item_sort_dir, col_name_vi, egui::Align::LEFT, c.text_muted, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::NameVi);
                    }
                    if render_item_sort_header_cell(ui, "Tên Tiếng Anh (EN)", ItemSortColumn::NameEn, self.item_sort_col, self.item_sort_dir, col_name_en, egui::Align::LEFT, c.text_muted, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::NameEn);
                    }
                    if render_item_sort_header_cell(ui, "Danh Mục", ItemSortColumn::Category, self.item_sort_col, self.item_sort_dir, COL_CAT, egui::Align::Center, c.text_muted, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::Category);
                    }
                    if render_item_sort_header_cell(ui, "Tổng Số Lượng", ItemSortColumn::TotalQty, self.item_sort_col, self.item_sort_dir, COL_TOTAL, egui::Align::RIGHT, c.accent_gold, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::TotalQty);
                    }
                    if render_item_sort_header_cell(ui, "Quy Đổi / Thực", ItemSortColumn::Converted, self.item_sort_col, self.item_sort_dir, COL_CONV, egui::Align::RIGHT, c.status_green, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::Converted);
                    }
                    if render_item_sort_header_cell(ui, "Số Trận", ItemSortColumn::Battles, self.item_sort_col, self.item_sort_dir, COL_BATTLES, egui::Align::RIGHT, c.text_muted, c.focus_teal) {
                        toggle_item_sort_col = Some(ItemSortColumn::Battles);
                    }

                    ui.add_sized([COL_RATE, 20.0], egui::Label::new(egui::RichText::new("Tỷ Lệ Rơi").strong().color(c.text_muted)).halign(egui::Align::RIGHT));
                    ui.add_sized([COL_AVG, 20.0], egui::Label::new(egui::RichText::new("TB / Trận").strong().color(c.text_muted)).halign(egui::Align::RIGHT));
                    ui.add_sized([COL_MAX, 20.0], egui::Label::new(egui::RichText::new("Max / Trận").strong().color(c.text_muted)).halign(egui::Align::RIGHT));
                });
            });

        if let Some(col) = toggle_item_sort_col {
            if self.item_sort_col == Some(col) {
                self.item_sort_dir = match self.item_sort_dir {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            } else {
                self.item_sort_col = Some(col);
                self.item_sort_dir = SortDirection::Descending;
            }
        }

        ui.add_space(4.0);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(inv_table_width);
                for it in filtered_items {
                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(inv_table_width, 26.0),
                        egui::Sense::hover(),
                    );

                    if row_resp.hovered() {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::same(4.0),
                            c.focus_teal.linear_multiply(0.10),
                        );
                    }

                    let inner_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x + INV_PAD_X, row_rect.min.y),
                        egui::pos2(row_rect.max.x - INV_PAD_X, row_rect.max.y),
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = INV_SPACING_X;

                            ui.add_sized([col_name_vi, 22.0], egui::Label::new(egui::RichText::new(&it.name_vi).strong().color(c.canvas_tint)).truncate().halign(egui::Align::LEFT));
                            ui.add_sized([col_name_en, 22.0], egui::Label::new(egui::RichText::new(&it.name_en).color(c.text_muted)).truncate().halign(egui::Align::LEFT));

                            egui::Frame::none()
                                .fill(c.primary)
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(4.0, 1.0))
                                .show(ui, |ui| {
                                    ui.add_sized([COL_CAT - 10.0, 18.0], egui::Label::new(egui::RichText::new(&it.category).font(egui::FontId::proportional(10.0)).color(c.focus_teal)).truncate().halign(egui::Align::Center));
                                });

                            ui.add_sized([COL_TOTAL, 22.0], egui::Label::new(egui::RichText::new(format_num(it.total_quantity)).monospace().strong().color(c.accent_gold)).truncate().halign(egui::Align::RIGHT));

                            let conv_str = get_item_converted_display(it.itype, it.sub_type, it.total_quantity).unwrap_or_else(|| "-".to_string());
                            let conv_color = if conv_str != "-" { c.status_green } else { c.text_muted };
                            ui.add_sized([COL_CONV, 22.0], egui::Label::new(egui::RichText::new(conv_str).monospace().strong().color(conv_color)).truncate().halign(egui::Align::RIGHT));

                            let drop_rate = (it.battle_count as f64 / total_battles as f64) * 100.0;
                            ui.add_sized([COL_BATTLES, 22.0], egui::Label::new(egui::RichText::new(format!("{}", it.battle_count)).monospace()).truncate().halign(egui::Align::RIGHT));
                            ui.add_sized([COL_RATE, 22.0], egui::Label::new(egui::RichText::new(format!("{:.1}%", drop_rate)).monospace().strong()).truncate().halign(egui::Align::RIGHT));
                            ui.add_sized([COL_AVG, 22.0], egui::Label::new(egui::RichText::new(format!("{:.1}", it.avg_per_battle)).monospace()).truncate().halign(egui::Align::RIGHT));
                            ui.add_sized([COL_MAX, 22.0], egui::Label::new(egui::RichText::new(format_num(it.max_in_battle)).monospace()).truncate().halign(egui::Align::RIGHT));
                        });
                    });

                    ui.add_space(2.0);
                }
            });
    }

    fn render_view_settings(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let c = self.colors();

        ui.columns(2, |cols| {
            // Left Column: Configuration & Controls
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(&mut cols[0], |ui| {
                // Card 1: Mail Folder
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("THƯ MỤC MAILCACHE GAME ROK").font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Thư mục chứa các file nhị phân báo cáo chiến đấu từ bản cài đặt Rise of Kingdoms PC.").font(egui::FontId::proportional(10.5)).color(c.text_muted));
                        ui.add_space(8.0);

                        let text_edit = ui.add_sized(
                            [ui.available_width(), 24.0],
                            egui::TextEdit::singleline(&mut self.mail_path_input),
                        );
                        if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.watcher.set_mail_dir(PathBuf::from(&self.mail_path_input));
                            self.save_config();
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Duyệt thư mục...").clicked() {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    self.mail_path_input = folder.to_string_lossy().to_string();
                                    self.watcher.set_mail_dir(folder);
                                    self.save_config();
                                }
                            }
                            if ui.button("Mặc định").clicked() {
                                let def_mail = detect_default_mail_dir();
                                self.mail_path_input = def_mail.to_string_lossy().to_string();
                                self.watcher.set_mail_dir(def_mail);
                                self.save_config();
                            }
                            if ui.button("Mở trong Explorer").clicked() {
                                let _ = open::that(self.watcher.get_mail_dir());
                            }
                        });
                    });

                ui.add_space(10.0);

                // Card 2: Scan Actions
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("THAO TÁC QUÉT DỮ LIỆU").font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Quét các file mail mới phát sinh hoặc tái phân tích toàn bộ kho lịch sử.").font(egui::FontId::proportional(10.5)).color(c.text_muted));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("Quét Thư Mới (Gia Tăng)").strong()).on_hover_text("Chỉ quét và giải mã các file mới thêm vào thư mục mailcache (~0ms)").clicked() {
                                self.watcher.rescan();
                            }
                            if ui.button(egui::RichText::new("Quét Lại Từ Đầu").color(c.accent_gold).strong()).on_hover_text("Tái phân tích lại toàn bộ các file thư từ đầu theo batch 50").clicked() {
                                self.watcher.rescan_force();
                            }
                        });
                    });

                ui.add_space(10.0);

                // Card 3: App Behavior & Protection
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("TÙY CHỌN HỆ THỐNG").font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Hành vi nút Đóng X:").strong().color(c.canvas_tint));
                            let prev_close = self.close_behavior;
                            ui.selectable_value(&mut self.close_behavior, CloseBehavior::Tray, "Ẩn vào Khay");
                            ui.selectable_value(&mut self.close_behavior, CloseBehavior::Close, "Thoát Ứng Dụng");
                            if self.close_behavior != prev_close {
                                self.save_config();
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Autostart setting
                        ui.horizontal(|ui| {
                            let mut autostart = self.autostart;
                            if ui.checkbox(&mut autostart, egui::RichText::new("Khởi động cùng Windows (Autostart)").strong().color(c.canvas_tint))
                                .on_hover_text("Tự động mở ROK Loot Watcher khi máy tính khởi động")
                                .changed()
                            {
                                self.autostart = autostart;
                                if let Err(e) = crate::autostart::set_autostart(self.autostart, self.autostart_minimized) {
                                    self.watcher.log("ERROR", &format!("Lỗi cài đặt Autostart: {}", e));
                                } else {
                                    let msg = if self.autostart {
                                        format!("Đã BẬT tự khởi động cùng Windows (Thu nhỏ vào khay: {})", if self.autostart_minimized { "BẬT" } else { "TẮT" })
                                    } else {
                                        "Đã TẮT tự khởi động cùng Windows".to_string()
                                    };
                                    self.watcher.log("SUCCESS", &msg);
                                }
                                self.save_config();
                            }
                        });

                        // Tùy chọn thu nhỏ xuống khay icon ở trong
                        if self.autostart {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                let mut start_min = self.autostart_minimized;
                                let min_color = if self.autostart_minimized { c.accent_gold } else { c.canvas_tint };
                                if ui.checkbox(&mut start_min, egui::RichText::new("↳ Thu nhỏ xuống Khay hệ thống (Tray Icon) khi khởi động").color(min_color))
                                    .on_hover_text("Khi khởi động cùng Windows, ứng dụng sẽ chạy ngầm dưới khay hệ thống mà không mở cửa sổ to gây vướng víu")
                                    .changed()
                                {
                                    self.autostart_minimized = start_min;
                                    if let Err(e) = crate::autostart::set_autostart(self.autostart, self.autostart_minimized) {
                                        self.watcher.log("ERROR", &format!("Lỗi cập nhật cấu hình Autostart: {}", e));
                                    } else {
                                        let msg = if self.autostart_minimized {
                                            "Đã BẬT tùy chọn: Thu nhỏ xuống khay icon khi khởi động cùng Windows"
                                        } else {
                                            "Đã TẮT tùy chọn: Thu nhỏ xuống khay icon khi khởi động cùng Windows (sẽ hiện cửa sổ)"
                                        };
                                        self.watcher.log("INFO", msg);
                                    }
                                    self.save_config();
                                }
                            });
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Deny-Delete Mailcache Armor
                        let armor_label = if self.is_armored {
                            "[Khóa Chống Xóa: ĐANG BẬT]"
                        } else {
                            "[Khóa Chống Xóa: ĐANG TẮT]"
                        };
                        let armor_color = if self.is_armored { c.status_green } else { c.text_muted };

                        if ui.button(egui::RichText::new(armor_label).color(armor_color).strong())
                            .on_hover_text("Ngăn game tự động xóa thư trong mailcache khi mở xem trong game (Yêu cầu Run as Administrator)")
                            .clicked()
                        {
                            let mail_dir = self.watcher.get_mail_dir();
                            if self.is_armored {
                                match crate::armor::disable_mailcache_armor(&mail_dir) {
                                    Ok(msg) => {
                                        self.watcher.log("SUCCESS", &msg);
                                        self.is_armored = false;
                                    }
                                    Err(err) => {
                                        self.watcher.log("ERROR", &err);
                                    }
                                }
                            } else {
                                match crate::armor::enable_mailcache_armor(&mail_dir) {
                                    Ok(msg) => {
                                        self.watcher.log("SUCCESS", &msg);
                                        self.is_armored = true;
                                    }
                                    Err(err) => {
                                        self.watcher.log("ERROR", &err);
                                    }
                                }
                            }
                        }
                    });

                ui.add_space(10.0);

                // Card 4: Persistent Vault
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        let vault_total = self.watcher.get_vault_total_count();
                        ui.label(egui::RichText::new("KHO LƯU TRỮ VĨNH VIỄN (VAULT)").font(egui::FontId::proportional(12.0)).strong().color(c.accent_gold));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(format!("Hiện đã bảo toàn an toàn: {} trận đánh", format_num(vault_total))).font(egui::FontId::proportional(11.0)).color(c.canvas_tint));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button("Mở Thư Mục mail_vault").clicked() {
                                let dir = self.watcher.get_vault_binary_dir();
                                let _ = open::that(&dir);
                            }
                            if ui.button("Mở File JSON Vault").clicked() {
                                let f = self.watcher.get_vault_file_path();
                                let _ = open::that(&f);
                            }
                        });
                    });
            });

            // Right Column: Terminal Log Viewer
            egui::Frame::none()
                .fill(Color32::from_rgb(0x0d, 0x0f, 0x14))
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(&mut cols[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("NHẬT KÝ HỆ THỐNG").font(egui::FontId::proportional(12.0)).strong().color(c.focus_teal));
                        ui.label(egui::RichText::new(format!("({} dòng)", self.console_logs.len())).font(egui::FontId::proportional(10.5)).color(c.text_muted));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Xóa log").clicked() {
                                self.console_logs.clear();
                            }
                            ui.checkbox(&mut self.log_auto_scroll, "Tự cuộn");
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .stick_to_bottom(self.log_auto_scroll)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.console_logs.is_empty() {
                                ui.label(egui::RichText::new("Chưa có sự kiện nào trong phiên làm việc này.").color(c.text_muted));
                            } else {
                                for line in &self.console_logs {
                                    let text_color = if line.contains("[SUCCESS]") {
                                        c.status_green
                                    } else if line.contains("[WARN]") {
                                        c.accent_gold
                                    } else if line.contains("[ERROR]") {
                                        c.error
                                    } else if line.contains("[INFO]") {
                                        c.focus_teal
                                    } else {
                                        c.canvas_tint
                                    };

                                    ui.label(
                                        egui::RichText::new(line)
                                            .font(egui::FontId::monospace(10.5))
                                            .color(text_color),
                                    );
                                }
                            }
                        });
                });
        });
    }

    fn render_side_inspector(&mut self, ui: &mut egui::Ui, record: &BattleRecord, close_inspector: &mut bool) {
        let c = self.colors();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("CHI TIẾT CHIẾN BÁO")
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(c.focus_teal),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(" Đóng ").on_hover_text("Đóng bảng chi tiết (hoặc bấm Esc)").clicked() {
                    *close_inspector = true;
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // Target Name & Type
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&record.target)
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(c.canvas_tint),
                    );

                    ui.add_space(6.0);
                    let badge_color = match record.battle_type.as_str() {
                        "motte" => c.motte_accent,
                        "fort" => c.fort_accent,
                        "gathering" => c.focus_teal,
                        _ => c.barb_accent,
                    };

                    let badge_label = if record.battle_type == "gathering" {
                        "THU THẬP".to_string()
                    } else {
                        record.battle_type.to_uppercase()
                    };

                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(badge_color.linear_multiply(0.2))
                            .rounding(Rounding::same(4.0))
                            .stroke(Stroke::new(1.0_f32, badge_color))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(badge_label).strong().color(badge_color));
                            });

                        if record.plunder_tier > 0 {
                            egui::Frame::none()
                                .fill(c.accent_gold.linear_multiply(0.2))
                                .rounding(Rounding::same(4.0))
                                .stroke(Stroke::new(1.0_f32, c.accent_gold))
                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("Plunder Tier {}", record.plunder_tier)).strong().color(c.accent_gold));
                                });
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("Thời gian: {}", record.time_vn)).font(egui::FontId::proportional(11.5)).color(c.text_muted));
                    if record.battle_type == "gathering" {
                        let lvl_display = if !record.target_level.is_empty() {
                            format!("Cấp {}", record.target_level)
                        } else {
                            "-".to_string()
                        };
                        ui.label(egui::RichText::new(format!("Cấp mỏ tài nguyên: {}", lvl_display)).font(egui::FontId::proportional(11.5)).strong().color(c.focus_teal));
                        if record.talent_add > 0 {
                            ui.label(egui::RichText::new(format!("Thưởng tài năng / công nghệ: +{}", format_num(record.talent_add))).font(egui::FontId::proportional(11.5)).color(c.status_green));
                        }
                    } else {
                        ui.label(egui::RichText::new(format!("Sát thương gây ra: {}%", record.damage_pct)).font(egui::FontId::proportional(11.5)).strong().color(c.focus_teal));
                    }
                });

            ui.add_space(10.0);

            // Summary Badges
            let res = record.total_resources_breakdown();
            if res.total_rss_val() > 0 || res.choice_packs > 0 {
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("TÀI NGUYÊN THU ĐƯỢC").font(egui::FontId::proportional(11.0)).strong().color(c.accent_gold));
                        ui.add_space(4.0);
                        let choice_str = if res.choice_packs > 0 { format!(" | Gói C: {} gói", format_num(res.choice_packs)) } else { String::new() };
                        ui.label(
                            egui::RichText::new(format!(
                                "Tổng: {:.2}M (Ngô: {:.2}M, Gỗ: {:.2}M, Đá: {:.2}M, Vàng: {:.2}M){}",
                                res.total_rss_val() as f64 / 1_000_000.0,
                                res.food_val as f64 / 1_000_000.0,
                                res.wood_val as f64 / 1_000_000.0,
                                res.stone_val as f64 / 1_000_000.0,
                                res.gold_val as f64 / 1_000_000.0,
                                choice_str
                            ))
                            .font(egui::FontId::proportional(11.0))
                            .color(c.canvas_tint),
                        );
                    });
                ui.add_space(8.0);
            }

            let u_min = record.univ_speedup_minutes();
            let t_min = record.training_speedup_minutes();
            let h_min = record.heal_speedup_minutes();
            if u_min > 0 || t_min > 0 || h_min > 0 {
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("TĂNG TỐC THU ĐƯỢC").font(egui::FontId::proportional(11.0)).strong().color(c.status_green));
                        ui.add_space(4.0);
                        let mut spd_parts = Vec::new();
                        if u_min > 0 { spd_parts.push(format!("Đa năng: {}", self.speedup_unit.format_duration(u_min as f64))); }
                        if t_min > 0 { spd_parts.push(format!("Đào tạo: {}", self.speedup_unit.format_duration(t_min as f64))); }
                        if h_min > 0 { spd_parts.push(format!("Hồi phục: {}", self.speedup_unit.format_duration(h_min as f64))); }
                        ui.label(egui::RichText::new(spd_parts.join("  |  ")).font(egui::FontId::proportional(11.0)).color(c.canvas_tint));
                    });
                ui.add_space(8.0);
            }

            let mats = record.materials_breakdown();
            let ap = record.ap_breakdown();
            if mats.total_pieces() > 0 || ap.total_bottles() > 0 {
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("VẬT LIỆU TRANG BỊ & ĐIỂM HÀNH ĐỘNG (AP)").font(egui::FontId::proportional(11.0)).strong().color(c.focus_teal));
                        ui.add_space(4.0);
                        let mut parts = Vec::new();
                        if mats.leather_total_converted() > 0 {
                            parts.push(format!("Da: {} BT", mats.leather_total_converted()));
                        }
                        if mats.iron_total_converted() > 0 {
                            parts.push(format!("Sắt: {} BT", mats.iron_total_converted()));
                        }
                        if mats.ebony_total_converted() > 0 {
                            parts.push(format!("Gỗ: {} BT", mats.ebony_total_converted()));
                        }
                        if mats.bone_total_converted() > 0 {
                            parts.push(format!("Xương: {} BT", mats.bone_total_converted()));
                        }
                        if ap.total_ap_points() > 0 {
                            parts.push(format!("+{} AP ({} bình)", ap.total_ap_points(), ap.total_bottles()));
                        }
                        ui.label(egui::RichText::new(parts.join("  |  ")).font(egui::FontId::proportional(11.0)).color(c.canvas_tint));
                    });
                ui.add_space(8.0);
            }

            // Items List
            ui.label(egui::RichText::new(format!("DANH SÁCH VẬT PHẨM ({} món)", record.items.len())).font(egui::FontId::proportional(11.5)).strong().color(c.canvas_tint));
            ui.add_space(6.0);

            if record.items.is_empty() {
                ui.label(egui::RichText::new("Không có vật phẩm nào được ghi nhận trong chiến báo này.").color(c.text_muted));
            } else {
                for it in &record.items {
                    egui::Frame::none()
                        .fill(c.card_surface)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(&it.name_vi).strong().color(c.canvas_tint));
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("[{}]", it.category)).font(egui::FontId::proportional(10.0)).color(c.focus_teal));
                                        if let Some(conv_str) = get_item_converted_display(it.itype, it.sub_type, it.value) {
                                            ui.label(egui::RichText::new(format!("(= {})", conv_str)).font(egui::FontId::proportional(10.0)).color(c.status_green));
                                        }
                                    });
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(format!("+{}", format_num(it.value))).font(egui::FontId::monospace(14.0)).strong().color(c.accent_gold));
                                });
                            });
                        });

                    ui.add_space(4.0);
                }
            }
        });
    }
}

impl eframe::App for RoKLootWatcherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Cache native HWND if not yet done
        let _ = crate::tray::get_or_find_main_hwnd();

        // Check if an external restore was requested (tray icon clicked, second instance opened, etc.)
        if crate::tray::check_and_clear_restore_request() {
            self.pending_start_minimized_frames = 0;
        }

        // Handle initial autostart minimization
        if self.pending_start_minimized_frames > 0 {
            self.pending_start_minimized_frames -= 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            crate::tray::hide_main_window();
        }

        // 1. If exit was requested from tray menu or background signal, terminate cleanly
        if crate::tray::is_quit_requested() {
            self.quit_app();
        }

        // 2. Intercept window close request based on user setting
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

        // 3. Escape key closes side inspector if open
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_battle_detail = None;
        }

        let had_events = self.poll_watcher_events(ctx);

        // Slate & Charcoal Minimalist Visuals
        let c = self.colors();
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
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, if self.theme == AppTheme::Dark { Color32::WHITE } else { Color32::BLACK });
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

        if had_events || self.watcher.is_scanning() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        // Left Sidebar Navigation
        egui::SidePanel::left("left_sidebar_panel")
            .exact_width(220.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(c.sidebar_bg).stroke(Stroke::new(1.0_f32, c.hairline)).inner_margin(Margin::same(12.0)))
            .show(ctx, |ui| {
                self.render_sidebar(ui, ctx);
            });

        // Slide-over Side Inspector Panel (if battle detail is selected)
        let mut close_inspector = false;
        let inspector_record = self.selected_battle_detail.clone();
        if let Some(ref record) = inspector_record {
            egui::SidePanel::right("inspector_side_panel")
                .exact_width(380.0)
                .resizable(false)
                .frame(egui::Frame::none().fill(c.ink_deep).stroke(Stroke::new(1.0_f32, c.hairline)).inner_margin(Margin::same(16.0)))
                .show(ctx, |ui| {
                    self.render_side_inspector(ui, record, &mut close_inspector);
                });
        }
        if close_inspector {
            self.selected_battle_detail = None;
        }

        // Central Content Area
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
    }
}
