use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::analyzer::parse_date_str;
use crate::i18n::{category_name, date_preset_label, filter_mode_label, TrKey};
use crate::model::{
    format_in_millions, format_num, get_item_converted_display_for_lang, AppConfig, AppLanguage,
    AppTheme, BattleRecord, CloseBehavior, DateFilterPreset, FilterMode, SpeedupUnit,
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
    pub train_orange: Color32,
    pub heal_blue: Color32,
    pub log_bg: Color32,
    pub mat_iron: Color32,
    pub mat_ebony: Color32,
    pub mat_bone: Color32,
    pub rank_silver: Color32,
    pub rank_bronze: Color32,
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
            train_orange: Color32::from_rgb(0xfb, 0x92, 0x3c),     // Orange-400 Training Speedup
            heal_blue: Color32::from_rgb(0x38, 0xbd, 0xf8),        // Sky-400 Healing Speedup
            log_bg: Color32::from_rgb(0x0d, 0x0f, 0x14),           // Deep terminal black
            mat_iron: Color32::from_rgb(0x94, 0xa3, 0xb8),         // Slate-400 Iron Ore
            mat_ebony: Color32::from_rgb(0x34, 0xd3, 0x99),        // Emerald-400 Ebony
            mat_bone: Color32::from_rgb(0xf4, 0x72, 0xb6),         // Pink-400 Bone
            rank_silver: Color32::from_rgb(0xc0, 0xc0, 0xc0),      // Silver
            rank_bronze: Color32::from_rgb(0xcd, 0x7f, 0x32),      // Bronze
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            ink: Color32::from_rgb(0xf8, 0xfa, 0xfc),              // Slate-50 clean light background
            ink_deep: Color32::from_rgb(0xff, 0xff, 0xff),         // Crisp pure white cards
            card_surface: Color32::from_rgb(0xff, 0xff, 0xff),     // Pure white for elevated cards
            sidebar_bg: Color32::from_rgb(0xf1, 0xf5, 0xf9),       // Slate-100 sidebar
            primary: Color32::from_rgb(0xf1, 0xf5, 0xf9),          // Slate-100 button background
            primary_pressed: Color32::from_rgb(0xe2, 0xe8, 0xf0),  // Slate-200 active button
            brand_mark: Color32::from_rgb(0xbe, 0x12, 0x3c),       // Rose-700 vibrant brand (>5.5:1)
            canvas_tint: Color32::from_rgb(0x0f, 0x17, 0x2a),      // Slate-900 crisp high-contrast text (>16:1)
            hairline: Color32::from_rgb(0xe2, 0xe8, 0xf0),         // Slate-200 clean border
            field_border: Color32::from_rgb(0xcb, 0xd5, 0xe1),     // Slate-300 input border
            accent_gold: Color32::from_rgb(0xb4, 0x53, 0x09),      // Amber-700 rich gold (>5.2:1)
            accent_gold_soft: Color32::from_rgb(0x92, 0x40, 0x0e), // Amber-800 deep amber
            focus_teal: Color32::from_rgb(0x03, 0x69, 0xa1),       // Sky-700 deep cyan/teal (>6.2:1)
            status_green: Color32::from_rgb(0x04, 0x78, 0x57),     // Emerald-700 crisp status (>5.1:1)
            text_muted: Color32::from_rgb(0x47, 0x55, 0x69),       // Slate-600 readable secondary text (>7.5:1)
            error: Color32::from_rgb(0xb9, 0x1c, 0x1c),            // Red-700 authoritative error (>6.1:1)
            motte_accent: Color32::from_rgb(0xbe, 0x12, 0x3c),     // Rose-700 Motte (>5.5:1)
            fort_accent: Color32::from_rgb(0xb4, 0x53, 0x09),      // Amber-700 Fort (>5.2:1)
            barb_accent: Color32::from_rgb(0x04, 0x78, 0x57),      // Emerald-700 Barbarian (>5.1:1)
            crystal_cyan: Color32::from_rgb(0x0e, 0x74, 0x90),     // Cyan-700 Crystal (>5.2:1)
            speedup_purple: Color32::from_rgb(0x7e, 0x22, 0xce),   // Purple-700 Speedups (>5.6:1)
            train_orange: Color32::from_rgb(0xc2, 0x41, 0x0c),     // Orange-700 Training (>5.8:1)
            heal_blue: Color32::from_rgb(0x03, 0x69, 0xa1),        // Sky-700 Healing (>6.2:1)
            log_bg: Color32::from_rgb(0xff, 0xff, 0xff),           // Pure white terminal card
            mat_iron: Color32::from_rgb(0x47, 0x55, 0x69),         // Slate-600 Iron Ore (>7.5:1)
            mat_ebony: Color32::from_rgb(0x04, 0x78, 0x57),        // Emerald-700 Ebony (>5.1:1)
            mat_bone: Color32::from_rgb(0xbe, 0x18, 0x5d),         // Pink-700 Bone (>5.5:1)
            rank_silver: Color32::from_rgb(0x47, 0x55, 0x69),      // Slate-600 Silver rank (>7.5:1)
            rank_bronze: Color32::from_rgb(0x9a, 0x34, 0x12),      // Amber-800 Bronze rank (>6.0:1)
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
    watcher_status: WatcherStatus,

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
    pub overlay_config: crate::model::OverlayConfig,
    pub last_mail_arrival: std::time::Instant,
    overlay_styled_hwnd: Option<isize>,
    pending_start_minimized_frames: u32,
    is_armored: bool,
    pending_armor_check: Option<std::sync::mpsc::Receiver<bool>>,
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
    lang: AppLanguage,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
    let is_active = active_col == Some(col);
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), active_color.linear_multiply(0.12));
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
    let text_color = if is_active || is_hovered { active_color } else { color };

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
        match (dir, lang) {
            (SortDirection::Descending, AppLanguage::Vietnamese) => format!("{}: Đang giảm dần (nhấp để chuyển tăng dần)", label),
            (SortDirection::Descending, AppLanguage::English) => format!("{}: Descending (click to sort ascending)", label),
            (SortDirection::Ascending, AppLanguage::Vietnamese) => format!("{}: Đang tăng dần (nhấp để chuyển giảm dần)", label),
            (SortDirection::Ascending, AppLanguage::English) => format!("{}: Ascending (click to sort descending)", label),
        }
    } else {
        match lang {
            AppLanguage::Vietnamese => format!("Nhấp để sắp xếp theo {}", label),
            AppLanguage::English => format!("Click to sort by {}", label),
        }
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
    lang: AppLanguage,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 22.0), egui::Sense::click());
    let is_active = active_col == Some(col);
    let is_hovered = resp.hovered();

    if is_hovered {
        ui.painter().rect_filled(rect, egui::Rounding::same(3.0), active_color.linear_multiply(0.12));
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
    let text_color = if is_active || is_hovered { active_color } else { color };

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
        match (dir, lang) {
            (SortDirection::Descending, AppLanguage::Vietnamese) => format!("{}: Đang giảm dần (nhấp để chuyển tăng dần)", label),
            (SortDirection::Descending, AppLanguage::English) => format!("{}: Descending (click to sort ascending)", label),
            (SortDirection::Ascending, AppLanguage::Vietnamese) => format!("{}: Đang tăng dần (nhấp để chuyển giảm dần)", label),
            (SortDirection::Ascending, AppLanguage::English) => format!("{}: Ascending (click to sort descending)", label),
        }
    } else {
        match lang {
            AppLanguage::Vietnamese => format!("Nhấp để sắp xếp theo {}", label),
            AppLanguage::English => format!("Click to sort by {}", label),
        }
    };
    let clicked = resp.clicked();
    resp.on_hover_text(tooltip);

    clicked
}

fn category_badge_color(category: &str, c: &ThemeColors) -> Color32 {
    let lower = category.to_lowercase();
    if lower.contains("nguyên") || lower.contains("resource") {
        c.accent_gold
    } else if lower.contains("vật liệu") || lower.contains("material") {
        c.heal_blue
    } else if lower.contains("tốc") || lower.contains("speed") {
        c.speedup_purple
    } else if lower.contains("pha lê") || lower.contains("crystal") {
        c.crystal_cyan
    } else if lower.contains("sách") || lower.contains("tên") || lower.contains("book") || lower.contains("arrow") {
        c.brand_mark
    } else if lower.contains("kinh nghiệm") || lower.contains("exp") {
        c.status_green
    } else {
        c.focus_teal
    }
}

pub fn resolve_status_badge(
    status: &WatcherStatus,
    is_watching: bool,
    language: AppLanguage,
    c: &ThemeColors,
) -> (String, Color32) {
    match status {
        WatcherStatus::Scanning(_) => (
            match language {
                AppLanguage::Vietnamese => "● Đang quét...".to_string(),
                AppLanguage::English => "● Scanning...".to_string(),
            },
            c.accent_gold,
        ),
        WatcherStatus::Error(_) => (
            format!("● {}", language.t(TrKey::StatusError)),
            c.error,
        ),
        WatcherStatus::Stopped(_) => (
            format!("● {}", language.t(TrKey::StatusPaused)),
            c.text_muted,
        ),
        WatcherStatus::Running(_) => {
            if !is_watching {
                (
                    format!("● {}", language.t(TrKey::StatusPaused)),
                    c.text_muted,
                )
            } else {
                (
                    format!("● {}", language.t(TrKey::StatusWatching)),
                    c.status_green,
                )
            }
        }
    }
}

pub(crate) fn poll_armor_check_channel(
    pending_armor_check: &mut Option<std::sync::mpsc::Receiver<bool>>,
    is_armored: &mut bool,
) {
    if let Some(ref rx) = pending_armor_check {
        match rx.try_recv() {
            Ok(armored) => {
                *is_armored = armored;
                *pending_armor_check = None;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                *pending_armor_check = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }
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

        let config = AppConfig::load();
        let initial_speedup_unit = config.speedup_unit;
        let initial_close_behavior = config.close_behavior;
        let initial_theme = config.theme;
        let initial_language = config.language;

        // Kiểm tra quyền khóa ghi mailcache ở luồng nền để tránh block main thread (icacls) khi khởi động
        let (armor_tx, armor_rx) = std::sync::mpsc::channel();
        let mail_dir_clone = watcher.get_mail_dir();
        let _ = std::thread::Builder::new()
            .name("rok-armor-check".into())
            .spawn(move || {
                let armored = crate::armor::is_mailcache_armored(&mail_dir_clone);
                let _ = armor_tx.send(armored);
            });

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

        Self {
            watcher,
            rx_ui,
            tray,

            theme: initial_theme,
            language: initial_language,
            watcher_status: WatcherStatus::Running(String::new()),

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
            overlay_config: config.overlay.clone(),
            last_mail_arrival: std::time::Instant::now(),
            overlay_styled_hwnd: None,
            pending_start_minimized_frames: if start_minimized { 5 } else { 0 },
            is_armored: false,
            pending_armor_check: Some(armor_rx),
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
            language: self.language,
            autostart: self.autostart,
            autostart_minimized: self.autostart_minimized,
            overlay: self.overlay_config.clone(),
        };
        config.save();
    }

    pub fn quit_app(&self) {
        self.watcher.flush_vault_if_dirty();
        self.watcher.stop();
        if let Some(ref t) = self.tray {
            t.hide_icon();
        }
        std::process::exit(0);
    }

    fn hide_to_tray(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        crate::tray::hide_main_window();
        let msg = match self.language {
            AppLanguage::Vietnamese => "Ứng dụng đã thu nhỏ xuống Khay hệ thống (System Tray). Nhấp vào biểu tượng ở khay để mở lại bất cứ lúc nào.",
            AppLanguage::English => "Application minimized to System Tray. Click the tray icon to restore at any time.",
        };
        self.watcher.log("INFO", msg);
    }

    fn poll_watcher_events(&mut self, ctx: &egui::Context) -> bool {
        let mut had_events = false;
        while let Ok(evt) = self.rx_ui.try_recv() {
            had_events = true;
            match evt {
                WatcherUiEvent::Log { level: _, message } => {
                    self.console_logs.push_back(message);
                    if self.console_logs.len() > 1000 {
                        self.console_logs.pop_front();
                    }
                }
                WatcherUiEvent::StatusChanged(status) => {
                    self.watcher_status = status;
                    ctx.request_repaint();
                }
                WatcherUiEvent::StatsUpdated { .. } => {
                    ctx.request_repaint();
                }
                WatcherUiEvent::MailDetected { .. } => {
                    self.last_mail_arrival = std::time::Instant::now();
                    ctx.request_repaint();
                }
            }
        }
        had_events
    }

    fn poll_armor_check(&mut self) {
        poll_armor_check_channel(&mut self.pending_armor_check, &mut self.is_armored);
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
                self.watcher.set_date_filter(None, None, Some(1));
            }
            DateFilterPreset::Yesterday => {
                let yest_vn = today_vn - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));
                self.watcher.set_date_filter(Some(yest_vn), Some(yest_vn), None);
            }
            DateFilterPreset::Last3Days => {
                self.watcher.set_date_filter(None, None, Some(3));
            }
            DateFilterPreset::Last7Days => {
                self.watcher.set_date_filter(None, None, Some(7));
            }
            DateFilterPreset::Last14Days => {
                self.watcher.set_date_filter(None, None, Some(14));
            }
            DateFilterPreset::Last30Days => {
                self.watcher.set_date_filter(None, None, Some(30));
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
            let w = crate::exporters::generate_csv(&records, self.language);
            if let Err(e) = std::fs::write(&path, w) {
                let err_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Không thể lưu file CSV: {}", e),
                    AppLanguage::English => format!("Failed to save CSV file: {}", e),
                };
                self.watcher.log("ERROR", &err_msg);
            } else {
                let ok_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Đã xuất {} trận đánh ra file CSV: {}", records.len(), path.display()),
                    AppLanguage::English => format!("Exported {} battle records to CSV: {}", records.len(), path.display()),
                };
                self.watcher.log("SUCCESS", &ok_msg);
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
                let err_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Không thể lưu file JSON: {}", e),
                    AppLanguage::English => format!("Failed to save JSON file: {}", e),
                };
                self.watcher.log("ERROR", &err_msg);
            } else {
                let ok_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Đã xuất {} trận đánh ra file JSON: {}", records.len(), path.display()),
                    AppLanguage::English => format!("Exported {} battle records to JSON: {}", records.len(), path.display()),
                };
                self.watcher.log("SUCCESS", &ok_msg);
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
            (0, self.language.t(TrKey::NavDashboardTitle), self.language.t(TrKey::NavDashboardDesc)),
            (1, self.language.t(TrKey::NavBattlesTitle), self.language.t(TrKey::NavBattlesDesc)),
            (2, self.language.t(TrKey::NavInventoryTitle), self.language.t(TrKey::NavInventoryDesc)),
            (3, self.language.t(TrKey::NavSettingsTitle), self.language.t(TrKey::NavSettingsDesc)),
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
                c.text_muted,
            );

            if resp.clicked() {
                self.current_tab = idx;
            }

            ui.add_space(4.0);
        }

        // 3. Bottom Widget: Watcher Status & Unified Control Dock
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(4.0);

            // Elevated Control Dock Card
            egui::Frame::none()
                .fill(c.card_surface)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    let (badge_text, badge_color) = resolve_status_badge(
                        &self.watcher_status,
                        self.watcher.is_watching(),
                        self.language,
                        &c,
                    );

                    let vault_count = self.watcher.get_vault_total_count();
                    let vault_txt = self.language.t(TrKey::StatusVaultCount).replace("{}", &format_num(vault_count));

                    // Header row: Live status badge (left) & Vault counter (right)
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&badge_text)
                                .font(egui::FontId::proportional(10.5))
                                .strong()
                                .color(badge_color),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            egui::Frame::none()
                                .fill(c.primary)
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(5.0, 1.5))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(format_num(vault_count))
                                            .font(egui::FontId::proportional(10.0))
                                            .strong()
                                            .color(c.canvas_tint),
                                    );
                                })
                                .response
                                .on_hover_text(&vault_txt);
                        });
                    });

                    ui.add_space(6.0);

                    // 2x2 Control Grid
                    let col_gap = 4.0;
                    let btn_w = ((ui.available_width() - col_gap) / 2.0).max(40.0);
                    let btn_h = 26.0;

                    // Row 1: Scan & Pause/Resume
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_gap;

                        let scan_btn = egui::Button::new(
                            egui::RichText::new(format!("🔄 {}", self.language.t(TrKey::BtnScan)))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([btn_w, btn_h], scan_btn)
                            .on_hover_text(self.language.t(TrKey::BtnScanTooltip))
                            .clicked()
                        {
                            self.watcher.rescan();
                        }

                        let is_watching = self.watcher.is_watching();
                        let (pause_icon, pause_text, pause_color) = if is_watching {
                            ("⏸", self.language.t(TrKey::BtnPause), c.canvas_tint)
                        } else {
                            ("▶", self.language.t(TrKey::BtnResume), c.accent_gold)
                        };

                        let pause_btn = egui::Button::new(
                            egui::RichText::new(format!("{} {}", pause_icon, pause_text))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(pause_color),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        let pause_tooltip = if is_watching {
                            match self.language {
                                AppLanguage::Vietnamese => "Tạm dừng tự động quét mail",
                                AppLanguage::English => "Pause automatic mail watching",
                            }
                        } else {
                            match self.language {
                                AppLanguage::Vietnamese => "Tiếp tục tự động quét mail",
                                AppLanguage::English => "Resume automatic mail watching",
                            }
                        };

                        if ui.add_sized([btn_w, btn_h], pause_btn)
                            .on_hover_text(pause_tooltip)
                            .clicked()
                        {
                            self.toggle_watcher();
                            if self.watcher.is_watching() {
                                self.watcher_status = WatcherStatus::Running(String::new());
                            } else {
                                self.watcher_status = WatcherStatus::Stopped(String::new());
                            }
                        }
                    });

                    ui.add_space(4.0);

                    // Row 2: Light/Dark Theme & Minimize to Tray
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_gap;

                        let (theme_icon, theme_text) = if self.theme == AppTheme::Dark {
                            ("☀", self.language.t(TrKey::ThemeToggleLight))
                        } else {
                            ("🌙", self.language.t(TrKey::ThemeToggleDark))
                        };

                        let theme_btn = egui::Button::new(
                            egui::RichText::new(format!("{} {}", theme_icon, theme_text))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([btn_w, btn_h], theme_btn)
                            .on_hover_text(self.language.t(TrKey::ThemeToggleTooltip))
                            .clicked()
                        {
                            self.theme = self.theme.toggle();
                            self.save_config();
                        }

                        let tray_btn = egui::Button::new(
                            egui::RichText::new(format!("📥 {}", self.language.t(TrKey::TrayMinimize)))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([btn_w, btn_h], tray_btn)
                            .on_hover_text(self.language.t(TrKey::TrayMinimizeTooltip))
                            .clicked()
                        {
                            self.hide_to_tray(ctx);
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
                    egui::RichText::new(self.language.t(TrKey::DashboardTitle))
                        .font(egui::FontId::proportional(18.0))
                        .strong()
                        .color(c.canvas_tint),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(self.language.t(TrKey::DashboardSubtitle))
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::FilterReportType)).strong().color(c.canvas_tint));
                        let prev_mode = self.selected_mode;
                        let modes = [
                            (FilterMode::AllTargets, match self.language { AppLanguage::Vietnamese => "Tất cả", AppLanguage::English => "All" }),
                            (FilterMode::Motte, "Motte"),
                            (FilterMode::Fort, match self.language { AppLanguage::Vietnamese => "Pháo Đài", AppLanguage::English => "Forts" }),
                            (FilterMode::Barbarian, match self.language { AppLanguage::Vietnamese => "Dã Man", AppLanguage::English => "Barbarians" }),
                            (FilterMode::AllForts, match self.language { AppLanguage::Vietnamese => "Cả 2 Pháo Đài", AppLanguage::English => "Both Forts" }),
                            (FilterMode::AllRewards, match self.language { AppLanguage::Vietnamese => "Tất cả quà", AppLanguage::English => "All Loot" }),
                            (FilterMode::Gathering, match self.language { AppLanguage::Vietnamese => "Thu Thập", AppLanguage::English => "Gathering" }),
                        ];
                        for (mode, label) in modes {
                            ui.selectable_value(&mut self.selected_mode, mode, label)
                                .on_hover_text(filter_mode_label(&mode, self.language));
                        }

                        if self.selected_mode != prev_mode {
                            self.watcher.set_mode(self.selected_mode);
                            self.save_config();
                        }

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new(self.language.t(TrKey::FilterDateRange)).strong().color(c.canvas_tint));
                        let prev_preset = self.selected_date_preset.clone();
                        egui::ComboBox::from_id_salt("dash_date_combo")
                            .selected_text(date_preset_label(&self.selected_date_preset, self.language))
                            .width(135.0)
                            .show_ui(ui, |ui| {
                                let presets = [
                                    DateFilterPreset::AllTime,
                                    DateFilterPreset::Today,
                                    DateFilterPreset::Yesterday,
                                    DateFilterPreset::Last3Days,
                                    DateFilterPreset::Last7Days,
                                    DateFilterPreset::Last14Days,
                                    DateFilterPreset::Last30Days,
                                    DateFilterPreset::Custom,
                                ];
                                for p in presets {
                                    let lbl = date_preset_label(&p, self.language);
                                    ui.selectable_value(&mut self.selected_date_preset, p, lbl);
                                }
                            });
                        if self.selected_date_preset != prev_preset {
                            let p = self.selected_date_preset.clone();
                            self.apply_date_preset(p);
                        }

                        if self.selected_date_preset == DateFilterPreset::Custom {
                            ui.label(egui::RichText::new(self.language.t(TrKey::FilterFrom)).color(c.text_muted));
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_from_input));
                            ui.label(egui::RichText::new(self.language.t(TrKey::FilterTo)).color(c.text_muted));
                            ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_to_input));
                            if ui.button(self.language.t(TrKey::FilterApply)).clicked() {
                                self.apply_custom_date_filter();
                            }
                        }

                        let cur_battles = self.watcher.get_stats().total_battles;
                        let battles_txt = self.language.t(TrKey::FilterRecordedBattles).replace("{}", &format_num(cur_battles));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(battles_txt)
                                    .strong()
                                    .color(c.focus_teal),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Row 2: Chuyển đổi đơn vị Tăng tốc (Phút / Giờ / Ngày / DHM)
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(self.language.t(TrKey::SpeedupUnitChange)).strong().color(c.canvas_tint));
                        let prev_unit = self.speedup_unit;
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Minutes, self.language.t(TrKey::SpeedupUnitMinutes));
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Hours, self.language.t(TrKey::SpeedupUnitHours));
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Days, self.language.t(TrKey::SpeedupUnitDays));
                        ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Dhm, self.language.t(TrKey::SpeedupUnitDhm));
                        if self.speedup_unit != prev_unit {
                            self.save_config();
                        }

                        let cur_stats = self.watcher.get_stats();
                        let (tr, he, un) = cur_stats.compute_speedups_by_category();
                        let spd_total = tr + he + un;
                        if spd_total > 0 {
                            ui.add_space(10.0);
                            let equiv_text = if self.speedup_unit == SpeedupUnit::Dhm {
                                match self.language {
                                    AppLanguage::Vietnamese => format!("≈ {:.1} giờ ({} phút)", spd_total as f64 / 60.0, format_num(spd_total)),
                                    AppLanguage::English => format!("≈ {:.1} hours ({} minutes)", spd_total as f64 / 60.0, format_num(spd_total)),
                                }
                            } else {
                                format!(
                                    "≈ {}",
                                    SpeedupUnit::Dhm.format_kpi(spd_total, self.language).0
                                )
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
            let (univ_spd_val, univ_spd_unit) =
                self.speedup_unit.format_kpi(total_spd_mins, self.language);

            // 4 Primary KPI Cards
            let filter_prefix = match self.language {
                AppLanguage::Vietnamese => "Bộ lọc",
                AppLanguage::English => "Filter",
            };
            let kpi_cards = [
                (
                    self.language.t(TrKey::CardTotalBattles),
                    format!("{} {}", format_num(total_battles), self.language.t(TrKey::CardBattlesUnit)),
                    format!("{}: {}", filter_prefix, filter_mode_label(&self.selected_mode, self.language)),
                    c.canvas_tint,
                ),
                (
                    self.language.t(TrKey::CardTotalCrystals),
                    format!("{} {}", format_num(pha_le), self.language.t(TrKey::CardCrystalsUnit)),
                    self.language.t(TrKey::CardAvgCrystals).replace("{:.1}", &format!("{:.1}", avg_crystal)),
                    c.crystal_cyan,
                ),
                (
                    self.language.t(TrKey::CardTotalSpeedups),
                    if self.speedup_unit == SpeedupUnit::Dhm {
                        univ_spd_val.clone()
                    } else {
                        format!("{} {}", univ_spd_val, univ_spd_unit)
                    },
                    self.language.t(TrKey::CardSpeedupBreakdown)
                        .replacen(
                            "{}",
                            &self
                                .speedup_unit
                                .format_duration_for_language(univ_spd_mins as f64, self.language),
                            1,
                        )
                        .replacen(
                            "{}",
                            &self
                                .speedup_unit
                                .format_duration_for_language(train_spd_mins as f64, self.language),
                            1,
                        )
                        .replacen(
                            "{}",
                            &self
                                .speedup_unit
                                .format_duration_for_language(heal_spd_mins as f64, self.language),
                            1,
                        ),
                    c.speedup_purple,
                ),
                (
                    self.language.t(TrKey::CardTotalResources),
                    format_in_millions(res.total_rss_val(), self.language),
                    self.language.t(TrKey::CardRssBreakdown).replace("{:.2}", &format!("{:.2}", res.total_rss_val() as f64 / 1_000_000.0)),
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::DistributionTitle)).font(egui::FontId::proportional(11.5)).strong().color(c.canvas_tint));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let total_txt = self.language.t(TrKey::DistributionTotal).replace("{}", &format_num(active_records.len()));
                            ui.label(egui::RichText::new(total_txt).color(c.text_muted).font(egui::FontId::proportional(11.0)));
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
                    let fort_lbl = match self.language {
                        AppLanguage::Vietnamese => "Pháo Đài Dã Man",
                        AppLanguage::English => "Barbarian Forts",
                    };
                    let barb_lbl = match self.language {
                        AppLanguage::Vietnamese => "Dã Man Thường",
                        AppLanguage::English => "Normal Barbarians",
                    };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("● Motte Destroyed: {} ({:.1}%)", motte_cnt, motte_pct)).color(c.motte_accent).font(egui::FontId::proportional(11.0)));
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("● {}: {} ({:.1}%)", fort_lbl, fort_cnt, fort_pct)).color(c.fort_accent).font(egui::FontId::proportional(11.0)));
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new(format!("● {}: {} ({:.1}%)", barb_lbl, barb_cnt, barb_pct)).color(c.barb_accent).font(egui::FontId::proportional(11.0)));
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
                            ui.label(egui::RichText::new(self.language.t(TrKey::TopItemsTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(self.language.t(TrKey::BtnViewAll)).clicked() {
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

                        egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                            if top_items.is_empty() {
                                ui.label(egui::RichText::new(self.language.t(TrKey::NoItemsData)).color(c.text_muted));
                            } else {
                                for (idx, it) in top_items.iter().enumerate() {
                                    let mut type_breakdown: Vec<(&String, &i64)> = it.by_battle_type.iter().filter(|(_, q)| **q > 0).collect();
                                    type_breakdown.sort_by(|a, b| b.1.cmp(a.1));

                                    let item_resp = ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let rank_color = match idx {
                                                0 => c.accent_gold,
                                                1 => c.rank_silver,
                                                2 => c.rank_bronze,
                                                _ => c.text_muted,
                                            };
                                            ui.label(egui::RichText::new(format!("#{}", idx + 1)).strong().color(rank_color).monospace());
                                            ui.label(egui::RichText::new(it.display_name(self.language)).strong().color(c.canvas_tint));

                                            if let Some(conv_str) = get_item_converted_display_for_lang(it.itype, it.sub_type, it.total_quantity, self.language) {
                                                ui.label(egui::RichText::new(format!("({})", conv_str)).font(egui::FontId::proportional(10.5)).color(c.status_green));
                                            }

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(egui::RichText::new(format_num(it.total_quantity)).monospace().strong().color(c.accent_gold));
                                            });
                                        });

                                        ui.add_space(2.0);

                                        // Segmented progress bar showing source distribution
                                        let pct = (it.total_quantity as f32 / max_qty as f32).clamp(0.04, 1.0);
                                        let bar_height = 6.0;
                                        let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), bar_height), egui::Sense::hover());
                                        ui.painter().rect_filled(bar_rect, Rounding::same(3.0), c.primary);

                                        let total_bar_w = (bar_rect.width() * pct).max(bar_height);

                                        if type_breakdown.is_empty() {
                                            let filled_r = egui::Rect::from_min_size(bar_rect.min, egui::vec2(total_bar_w, bar_height));
                                            ui.painter().rect_filled(filled_r, Rounding::same(3.0), c.focus_teal);
                                        } else if type_breakdown.len() == 1 {
                                            let (btype, _) = type_breakdown[0];
                                            let color = match btype.as_str() {
                                                "motte" => c.motte_accent,
                                                "fort" => c.fort_accent,
                                                "barbarian" => c.barb_accent,
                                                "gathering" => c.focus_teal,
                                                _ => c.focus_teal,
                                            };
                                            let filled_r = egui::Rect::from_min_size(bar_rect.min, egui::vec2(total_bar_w, bar_height));
                                            ui.painter().rect_filled(filled_r, Rounding::same(3.0), color);
                                        } else {
                                            let mut cur_x = bar_rect.min.x;
                                            let total_val = it.total_quantity.max(1) as f32;
                                            let count = type_breakdown.len();

                                            for (i, (btype, qty)) in type_breakdown.iter().enumerate() {
                                                let seg_ratio = (**qty as f32) / total_val;
                                                let seg_w = (total_bar_w * seg_ratio).max(1.0);
                                                let color = match btype.as_str() {
                                                    "motte" => c.motte_accent,
                                                    "fort" => c.fort_accent,
                                                    "barbarian" => c.barb_accent,
                                                    "gathering" => c.focus_teal,
                                                    _ => c.focus_teal,
                                                };

                                                let seg_rect = egui::Rect::from_min_max(
                                                    egui::pos2(cur_x, bar_rect.min.y),
                                                    egui::pos2((cur_x + seg_w).min(bar_rect.min.x + total_bar_w), bar_rect.max.y),
                                                );

                                                let rounding = if i == 0 {
                                                    Rounding { nw: 3.0, sw: 3.0, ne: 0.0, se: 0.0 }
                                                } else if i == count - 1 {
                                                    Rounding { nw: 0.0, sw: 0.0, ne: 3.0, se: 3.0 }
                                                } else {
                                                    Rounding::ZERO
                                                };

                                                ui.painter().rect_filled(seg_rect, rounding, color);
                                                cur_x += seg_w;
                                                if cur_x >= bar_rect.min.x + total_bar_w {
                                                    break;
                                                }
                                            }
                                        }

                                        // Micro distribution breakdown badges under bar
                                        if !type_breakdown.is_empty() {
                                            ui.add_space(2.0);
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 8.0;
                                                let total_f = it.total_quantity.max(1) as f64;
                                                for (btype, qty) in type_breakdown.iter().take(3) {
                                                    let ratio = (**qty as f64 / total_f) * 100.0;
                                                    let (type_name, color) = match (btype.as_str(), self.language) {
                                                        ("motte", _) => ("Motte", c.motte_accent),
                                                        ("barbarian", AppLanguage::Vietnamese) => ("Dã Man", c.barb_accent),
                                                        ("barbarian", AppLanguage::English) => ("Barbarian", c.barb_accent),
                                                        ("fort", AppLanguage::Vietnamese) => ("Pháo Đài", c.fort_accent),
                                                        ("fort", AppLanguage::English) => ("Fort", c.fort_accent),
                                                        ("gathering", AppLanguage::Vietnamese) => ("Thu Thập", c.focus_teal),
                                                        ("gathering", AppLanguage::English) => ("Gathering", c.focus_teal),
                                                        _ => (match self.language { AppLanguage::Vietnamese => "Khác", AppLanguage::English => "Other" }, c.text_muted),
                                                    };

                                                    ui.label(egui::RichText::new("●").font(egui::FontId::proportional(8.0)).color(color));
                                                    ui.label(
                                                        egui::RichText::new(format!("{}: {:.0}%", type_name, ratio))
                                                            .font(egui::FontId::proportional(10.5))
                                                            .color(c.text_muted),
                                                    );
                                                }
                                            });
                                        }

                                        ui.add_space(6.0);
                                    }).response;

                                    item_resp.on_hover_ui(|ui| {
                                        ui.set_max_width(280.0);
                                        ui.label(egui::RichText::new(it.display_name(self.language)).strong().color(c.canvas_tint));
                                        let total_lbl = match self.language {
                                            AppLanguage::Vietnamese => format!("Tổng cộng: {} (100%)", format_num(it.total_quantity)),
                                            AppLanguage::English => format!("Total: {} (100%)", format_num(it.total_quantity)),
                                        };
                                        ui.label(egui::RichText::new(total_lbl).font(egui::FontId::proportional(11.0)).color(c.accent_gold));
                                        ui.separator();

                                        let total_f = it.total_quantity.max(1) as f64;
                                        for (btype, qty) in &type_breakdown {
                                            let ratio = (**qty as f64 / total_f) * 100.0;
                                            let (type_name, color) = match (btype.as_str(), self.language) {
                                                ("motte", _) => ("Motte", c.motte_accent),
                                                ("barbarian", AppLanguage::Vietnamese) => ("Dã Man", c.barb_accent),
                                                ("barbarian", AppLanguage::English) => ("Barbarian", c.barb_accent),
                                                ("fort", AppLanguage::Vietnamese) => ("Pháo Đài", c.fort_accent),
                                                ("fort", AppLanguage::English) => ("Fort", c.fort_accent),
                                                ("gathering", AppLanguage::Vietnamese) => ("Thu Thập", c.focus_teal),
                                                ("gathering", AppLanguage::English) => ("Gathering", c.focus_teal),
                                                _ => (match self.language { AppLanguage::Vietnamese => "Khác", AppLanguage::English => "Other" }, c.text_muted),
                                            };

                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("●").color(color));
                                                ui.label(egui::RichText::new(type_name).strong().color(c.canvas_tint));
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(egui::RichText::new(format!("{} ({:.1}%)", format_num(**qty), ratio)).monospace().color(color));
                                                });
                                            });
                                        }
                                    });
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
                            ui.label(egui::RichText::new(self.language.t(TrKey::LatestBattlesTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(self.language.t(TrKey::BtnViewBattles)).clicked() {
                                    self.current_tab = 1;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let latest_battles = crate::analyzer::get_latest_records(&active_records, 5);

                        if latest_battles.is_empty() {
                            ui.label(egui::RichText::new(self.language.t(TrKey::NoBattlesData)).color(c.text_muted));
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
                                    r.display_target(self.language),
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
                                let cry_label = match self.language {
                                    AppLanguage::Vietnamese => "Pha Lê",
                                    AppLanguage::English => "Crystals",
                                };
                                let cry_str = if cry > 0 { format!("+{} {}  |  ", format_num(cry), cry_label) } else { "".to_string() };
                                let dmg_label = match self.language {
                                    AppLanguage::Vietnamese => "ST",
                                    AppLanguage::English => "DMG",
                                };
                                let dmg_str = format!("{}: {}%", dmg_label, r.damage_pct);
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
                                    self.language.t(TrKey::BtnDetail),
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
                    ui.label(egui::RichText::new(self.language.t(TrKey::FilterReportType)).strong().color(c.canvas_tint));
                    let prev_mode = self.selected_mode;
                    let modes = [
                        (FilterMode::AllTargets, match self.language { AppLanguage::Vietnamese => "Tất cả", AppLanguage::English => "All" }),
                        (FilterMode::Motte, "Motte"),
                        (FilterMode::Fort, match self.language { AppLanguage::Vietnamese => "Pháo Đài", AppLanguage::English => "Forts" }),
                        (FilterMode::Barbarian, match self.language { AppLanguage::Vietnamese => "Man Rợ", AppLanguage::English => "Barbarians" }),
                        (FilterMode::AllForts, match self.language { AppLanguage::Vietnamese => "Tất cả pháo đài", AppLanguage::English => "All Forts" }),
                        (FilterMode::AllRewards, match self.language { AppLanguage::Vietnamese => "Tất cả quà", AppLanguage::English => "All Loot" }),
                        (FilterMode::Gathering, match self.language { AppLanguage::Vietnamese => "Thu Thập", AppLanguage::English => "Gathering" }),
                    ];
                    for (mode, label) in modes {
                        ui.selectable_value(&mut self.selected_mode, mode, label)
                            .on_hover_text(filter_mode_label(&mode, self.language));
                    }

                    if self.selected_mode != prev_mode {
                        self.watcher.set_mode(self.selected_mode);
                        self.save_config();
                    }

                    ui.add_space(12.0);
                    ui.label(egui::RichText::new(self.language.t(TrKey::FilterDateRange)).strong().color(c.canvas_tint));
                    let prev_preset = self.selected_date_preset.clone();
                    egui::ComboBox::from_id_salt("battles_date_combo")
                        .selected_text(date_preset_label(&self.selected_date_preset, self.language))
                        .width(130.0)
                        .show_ui(ui, |ui| {
                            let presets = [
                                DateFilterPreset::AllTime,
                                DateFilterPreset::Today,
                                DateFilterPreset::Yesterday,
                                DateFilterPreset::Last3Days,
                                DateFilterPreset::Last7Days,
                                DateFilterPreset::Last14Days,
                                DateFilterPreset::Last30Days,
                                DateFilterPreset::Custom,
                            ];
                            for p in presets {
                                let lbl = date_preset_label(&p, self.language);
                                ui.selectable_value(&mut self.selected_date_preset, p, lbl);
                            }
                        });

                    if self.selected_date_preset != prev_preset {
                        let p = self.selected_date_preset.clone();
                        self.apply_date_preset(p);
                    }

                    if self.selected_date_preset == DateFilterPreset::Custom {
                        ui.label(egui::RichText::new(self.language.t(TrKey::FilterFrom)).color(c.text_muted));
                        ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_from_input));
                        ui.label(egui::RichText::new(self.language.t(TrKey::FilterTo)).color(c.text_muted));
                        ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.date_to_input));
                        if ui.button(self.language.t(TrKey::FilterApply)).clicked() {
                            self.apply_custom_date_filter();
                        }
                    }

                    let cur_cnt = self.watcher.get_active_count();
                    let match_txt = self.language.t(TrKey::FilterMatchingBattles).replace("{}", &format_num(cur_cnt));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(match_txt)
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
                });

                ui.add_space(6.0);

                // Row 2: Search, Speedup Unit & Export Actions
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.language.t(TrKey::SearchLabel)).strong().color(c.canvas_tint));
                    ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut self.battle_search).hint_text(self.language.t(TrKey::SearchPlaceholder)));

                    ui.add_space(12.0);
                    let spd_lbl = match self.language {
                        AppLanguage::Vietnamese => "Tăng tốc:",
                        AppLanguage::English => "Speedups:",
                    };
                    ui.label(egui::RichText::new(spd_lbl).color(c.text_muted));
                    let prev_unit = self.speedup_unit;
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Minutes, self.language.t(TrKey::SpeedupUnitMinutes));
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Hours, self.language.t(TrKey::SpeedupUnitHours));
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Days, self.language.t(TrKey::SpeedupUnitDays));
                    ui.selectable_value(&mut self.speedup_unit, SpeedupUnit::Dhm, self.language.t(TrKey::SpeedupUnitDhm));
                    if self.speedup_unit != prev_unit {
                        self.save_config();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.language.t(TrKey::BtnExportJson)).on_hover_text(self.language.t(TrKey::BtnExportJsonTooltip)).clicked() {
                            self.export_active_json();
                        }
                        if ui.button(self.language.t(TrKey::BtnExportCsv)).on_hover_text(self.language.t(TrKey::BtnExportCsvTooltip)).clicked() {
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
        const COL_BARB_CRY: f32 = 100.0;
        const COL_BARB_RES: f32 = 100.0;
        const COL_BARB_EXP: f32 = 58.0;
        const COL_BARB_UNIV: f32 =100.0;
        const COL_BARB_TRAIN: f32 = 100.0;
        const COL_BARB_HEAL: f32 = 100.0;
        const COL_BARB_ARROW: f32 = 60.0;

        // Motte / Fort columns
        const COL_TIER: f32 = 48.0;
        const COL_CRY: f32 = 100.0;
        const COL_PACKS: f32 = 70.0;
        const COL_TOME: f32 = 70.0;
        const COL_UNIV_SPD: f32 = 100.0;
        const COL_HEAL_SPD: f32 = 100.0;

        const COL_VIEW: f32 = 120.0;

        let avail_w = ui.available_width();
        let non_target_column_count = report_non_target_column_count(is_gathering, is_barb);
        let fixed_w = if is_gathering {
            COL_TIME + col_type + COL_GATHER_LVL + COL_GATHER_RES + COL_GATHER_TOTAL + COL_GATHER_BONUS + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * non_target_column_count as f32)
        } else if is_barb {
            COL_TIME + col_type + COL_DMG + COL_BARB_CRY + COL_BARB_RES + COL_BARB_EXP + COL_BARB_UNIV + COL_BARB_TRAIN + COL_BARB_HEAL + COL_BARB_ARROW + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * non_target_column_count as f32)
        } else {
            COL_TIME + col_type + COL_DMG + COL_TIER + COL_CRY + COL_PACKS + COL_TOME + COL_UNIV_SPD + COL_HEAL_SPD + COL_VIEW + (TABLE_PAD_X * 2.0) + (ITEM_SPACING_X * non_target_column_count as f32)
        };

        let dimensions = report_table_dimensions(avail_w, fixed_w, non_target_column_count);
        let col_target = dimensions.target_width;
        let extra_per_column = dimensions.extra_per_column;

        // Table Header
        let mut toggle_sort_col = None;

        egui::Frame::none()
            .fill(c.card_surface)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(TABLE_PAD_X, 4.0))
            .show(ui, |ui| {
                ui.set_width(dimensions.table_width - (TABLE_PAD_X * 2.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = ITEM_SPACING_X;
                    ui.set_min_height(26.0);

                    if render_sort_header_cell(ui, self.language.t(TrKey::ColTime), BattleSortColumn::Time, self.battle_sort_col, self.battle_sort_dir, COL_TIME + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_sort_col = Some(BattleSortColumn::Time);
                    }
                    if render_sort_header_cell(ui, self.language.t(TrKey::ColType), BattleSortColumn::Type, self.battle_sort_col, self.battle_sort_dir, col_type + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_sort_col = Some(BattleSortColumn::Type);
                    }
                    if render_sort_header_cell(ui, self.language.t(TrKey::ColTarget), BattleSortColumn::Target, self.battle_sort_col, self.battle_sort_dir, col_target, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_sort_col = Some(BattleSortColumn::Target);
                    }
                    if !is_gathering
                        && render_sort_header_cell(ui, self.language.t(TrKey::ColDamage), BattleSortColumn::Damage, self.battle_sort_col, self.battle_sort_dir, COL_DMG + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_sort_col = Some(BattleSortColumn::Damage);
                    }

                    if is_gathering {
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColGatherLvl), BattleSortColumn::Level, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_LVL + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Level);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColGatherRes), BattleSortColumn::Target, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_RES + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Target);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColGatherTotal), BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_TOTAL + extra_per_column, egui::Align::Center, c.accent_gold, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColGatherBonus), BattleSortColumn::Damage, self.battle_sort_col, self.battle_sort_dir, COL_GATHER_BONUS + extra_per_column, egui::Align::Center, c.status_green, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Damage);
                        }
                    } else if is_barb {
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColCrystal), BattleSortColumn::Crystal, self.battle_sort_col, self.battle_sort_dir, COL_BARB_CRY + extra_per_column, egui::Align::Center, c.crystal_cyan, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Crystal);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColRss), BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_BARB_RES + extra_per_column, egui::Align::Center, c.accent_gold, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColExp), BattleSortColumn::TomeExp, self.battle_sort_col, self.battle_sort_dir, COL_BARB_EXP + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::TomeExp);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColUnivSpd), BattleSortColumn::UnivSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_UNIV + extra_per_column, egui::Align::Center, c.status_green, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::UnivSpeedup);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColTrainSpd), BattleSortColumn::TrainSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_TRAIN + extra_per_column, egui::Align::Center, c.train_orange, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::TrainSpeedup);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColHealSpd), BattleSortColumn::HealSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_BARB_HEAL + extra_per_column, egui::Align::Center, c.heal_blue, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::HealSpeedup);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColArrow), BattleSortColumn::Arrow, self.battle_sort_col, self.battle_sort_dir, COL_BARB_ARROW + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Arrow);
                        }
                    } else {
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColTier), BattleSortColumn::Tier, self.battle_sort_col, self.battle_sort_dir, COL_TIER + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Tier);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColCrystal), BattleSortColumn::Crystal, self.battle_sort_col, self.battle_sort_dir, COL_CRY + extra_per_column, egui::Align::Center, c.crystal_cyan, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Crystal);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColPacks), BattleSortColumn::Resource, self.battle_sort_col, self.battle_sort_dir, COL_PACKS + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::Resource);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColTome), BattleSortColumn::TomeExp, self.battle_sort_col, self.battle_sort_dir, COL_TOME + extra_per_column, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::TomeExp);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColUnivSpd), BattleSortColumn::UnivSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_UNIV_SPD + extra_per_column, egui::Align::Center, c.status_green, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::UnivSpeedup);
                        }
                        if render_sort_header_cell(ui, self.language.t(TrKey::ColHealSpd), BattleSortColumn::HealSpeedup, self.battle_sort_col, self.battle_sort_dir, COL_HEAL_SPD + extra_per_column, egui::Align::Center, c.heal_blue, c.focus_teal, self.language) {
                            toggle_sort_col = Some(BattleSortColumn::HealSpeedup);
                        }
                    }

                    ui.add_sized(
                        [COL_VIEW + extra_per_column, 20.0],
                        egui::Label::new(egui::RichText::new(self.language.t(TrKey::ColActions)).strong().color(c.focus_teal)).halign(egui::Align::Center),
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
                    let scan_msg = match self.language {
                        AppLanguage::Vietnamese => "[Đang quét] Đang quét và giải mã dữ liệu mailcache... Vui lòng đợi trong giây lát.",
                        AppLanguage::English => "[Scanning] Scanning and decoding mailcache data... Please wait a moment.",
                    };
                    ui.label(
                        egui::RichText::new(scan_msg)
                            .font(egui::FontId::proportional(13.0))
                            .color(c.accent_gold),
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    let no_rec_msg = match self.language {
                        AppLanguage::Vietnamese => "Không tìm thấy trận đánh nào phù hợp với bộ lọc hiện tại.",
                        AppLanguage::English => "No battles found matching the current filter.",
                    };
                    let hint_msg = match self.language {
                        AppLanguage::Vietnamese => "[Gợi ý] Thử đổi 'Thời gian' sang 'Từ trước đến nay' hoặc đổi 'Mục tiêu' sang 'Tất cả' để hiển thị toàn bộ lịch sử.",
                        AppLanguage::English => "[Tip] Try changing 'Date Range' to 'All Time' or 'Report Type' to 'All Targets' to view full history.",
                    };
                    ui.label(
                        egui::RichText::new(no_rec_msg)
                            .font(egui::FontId::proportional(14.0))
                            .strong()
                            .color(c.canvas_tint),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(hint_msg)
                            .font(egui::FontId::proportional(12.0))
                            .color(c.focus_teal),
                    );
                });
            }
            return;
        }

        let search_lower = self.battle_search.to_lowercase();
        let cur_lang = self.language;
        let mut sorted_records: Vec<&BattleRecord> = records
            .iter()
            .filter(|r| {
                if search_lower.is_empty() {
                    true
                } else {
                    r.battle_type.contains(&search_lower)
                        || r.time_vn.contains(&search_lower)
                        || r.target.to_lowercase().contains(&search_lower)
                        || r.display_target(cur_lang).to_lowercase().contains(&search_lower)
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
        let row_height = 30.0;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, sorted_records.len(), |ui, row_range| {
                ui.set_width(dimensions.table_width);
                for idx in row_range {
                    let r = sorted_records[idx];
                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(dimensions.table_width, 28.0),
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

                            ui.add_sized([COL_TIME + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(&r.time_vn).monospace().font(egui::FontId::monospace(11.0))).halign(egui::Align::Center));

                            let (type_text, type_badge_color) = match r.battle_type.as_str() {
                                "motte" => ("MOTTE", c.motte_accent),
                                "fort" => ("FORT", c.fort_accent),
                                "gathering" => (match self.language { AppLanguage::Vietnamese => "THU THẬP", AppLanguage::English => "GATHERING" }, c.focus_teal),
                                _ => (match self.language { AppLanguage::Vietnamese => "DÃ MAN", AppLanguage::English => "BARBARIAN" }, c.barb_accent),
                            };
                            ui.add_sized([col_type + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(type_text).font(egui::FontId::proportional(10.5)).color(type_badge_color).strong()).halign(egui::Align::Center));

                            let target_btn = egui::Button::new(
                                egui::RichText::new(r.display_target(self.language)).strong().color(c.canvas_tint)
                            )
                            .frame(false)
                            .truncate();
                            if ui.add_sized([dimensions.target_width, 24.0], target_btn).clicked() {
                                clicked_record = Some(r.clone());
                            }

                            if !is_gathering {
                                ui.add_sized([COL_DMG + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(format!("{}%", r.damage_pct)).monospace().color(c.focus_teal).strong()).halign(egui::Align::Center));
                            }

                            if is_gathering {
                                let lvl_str = if !r.target_level.is_empty() {
                                    match self.language {
                                        AppLanguage::Vietnamese => format!("Cấp {}", r.target_level),
                                        AppLanguage::English => format!("Lvl {}", r.target_level),
                                    }
                                } else {
                                    "-".to_string()
                                };
                                ui.add_sized([COL_GATHER_LVL + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(lvl_str).strong().color(c.focus_teal)).halign(egui::Align::Center));

                                let res_name = r.items.first().map(|i| i.display_name(self.language)).unwrap_or("-");
                                ui.add_sized([COL_GATHER_RES + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(res_name).strong().color(c.canvas_tint)).halign(egui::Align::Center));

                                let total_val = r.items.first().map(|i| i.value).unwrap_or(0);
                                ui.add_sized([COL_GATHER_TOTAL + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(format_num(total_val)).monospace().strong().color(c.accent_gold)).halign(egui::Align::Center));

                                let bonus_str = if r.talent_add > 0 { format!("+{}", format_num(r.talent_add)) } else { "-".to_string() };
                                ui.add_sized([COL_GATHER_BONUS + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(bonus_str).monospace().color(c.status_green)).halign(egui::Align::Center));
                            } else if is_barb {
                                let cry = r.crystal_quantity();
                                let (cry_str, cry_color) = if cry > 0 {
                                    (format_num(cry), c.crystal_cyan)
                                } else {
                                    ("-".to_string(), c.text_muted)
                                };
                                ui.add_sized([COL_BARB_CRY + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(cry_str).monospace().color(cry_color).strong()).halign(egui::Align::Center));

                                let res_breakdown = r.total_resources_breakdown();
                                let res_val = res_breakdown.total_rss_val();
                                let res_str = format_in_millions(res_val, self.language);
                                ui.add_sized([COL_BARB_RES + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(res_str).monospace().color(c.accent_gold).strong()).halign(egui::Align::Center));

                                let exp_val = r.total_exp_value();
                                ui.add_sized([COL_BARB_EXP + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(format_in_millions(exp_val, self.language)).monospace()).halign(egui::Align::Center));

                                let univ_spd_min = r.univ_speedup_minutes();
                                let univ_spd_str = self.speedup_unit.format_duration_for_language(univ_spd_min as f64, self.language);
                                ui.add_sized([COL_BARB_UNIV + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(univ_spd_str).monospace().color(c.status_green)).halign(egui::Align::Center));

                                let train_spd_min = r.training_speedup_minutes();
                                let train_spd_str = self.speedup_unit.format_duration_for_language(train_spd_min as f64, self.language);
                                ui.add_sized([COL_BARB_TRAIN + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(train_spd_str).monospace().color(c.train_orange)).halign(egui::Align::Center));

                                let heal_spd_min = r.heal_speedup_minutes();
                                let heal_spd_str = self.speedup_unit.format_duration_for_language(heal_spd_min as f64, self.language);
                                ui.add_sized([COL_BARB_HEAL + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(heal_spd_str).monospace().color(c.heal_blue)).halign(egui::Align::Center));

                                let arrows = r.item_quantity(2, 145);
                                ui.add_sized([COL_BARB_ARROW + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(format_num(arrows)).monospace()).halign(egui::Align::Center));
                            } else {
                                let tier_str = if r.plunder_tier > 0 {
                                    format!("Tier {}", r.plunder_tier)
                                } else {
                                    "-".to_string()
                                };
                                ui.add_sized([COL_TIER + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(tier_str).color(c.accent_gold).strong()).halign(egui::Align::Center));

                                let cry = r.crystal_quantity();
                                let cry_str = if cry > 0 { format_num(cry) } else { "-".to_string() };
                                ui.add_sized([COL_CRY + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(cry_str).monospace().color(c.crystal_cyan).strong()).halign(egui::Align::Center));

                                let packs = if r.battle_type == "gathering" {
                                    r.items.first().map(|i| i.value).unwrap_or(0)
                                } else {
                                    r.total_resources_breakdown().choice_packs
                                };
                                let pack_label = if r.battle_type == "gathering" {
                                    format_num(packs)
                                } else {
                                    match self.language {
                                        AppLanguage::Vietnamese => format!("{} gói", format_num(packs)),
                                        AppLanguage::English => format!("{} packs", format_num(packs)),
                                    }
                                };
                                ui.add_sized([COL_PACKS + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(pack_label).monospace().color(c.accent_gold)).halign(egui::Align::Center));

                                let tome = r.item_quantity(2, 130);
                                ui.add_sized([COL_TOME + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(format_num(tome)).monospace()).halign(egui::Align::Center));

                                let u_spd_min = r.univ_speedup_minutes();
                                let u_spd_str = self.speedup_unit.format_duration_for_language(u_spd_min as f64, self.language);
                                ui.add_sized([COL_UNIV_SPD + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(u_spd_str).monospace().color(c.status_green)).halign(egui::Align::Center));

                                let h_spd_min = r.heal_speedup_minutes();
                                let h_spd_str = self.speedup_unit.format_duration_for_language(h_spd_min as f64, self.language);
                                ui.add_sized([COL_HEAL_SPD + extra_per_column, 24.0], egui::Label::new(egui::RichText::new(h_spd_str).monospace().color(c.heal_blue)).halign(egui::Align::Center));
                            }

                            let view_btn = egui::Button::new(
                                egui::RichText::new(self.language.t(TrKey::BtnDetail))
                                    .font(egui::FontId::proportional(11.0))
                                    .strong()
                                    .color(c.focus_teal),
                            )
                            .fill(c.focus_teal.linear_multiply(0.12))
                            .stroke(egui::Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.45)))
                            .rounding(egui::Rounding::same(4.0));

                            if ui.add_sized([COL_VIEW + extra_per_column, 22.0], view_btn).clicked() {
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
                    ui.label(egui::RichText::new(self.language.t(TrKey::InvSearchLabel)).strong().color(c.canvas_tint));
                    ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut self.item_search).hint_text(self.language.t(TrKey::InvSearchPlaceholder)));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(self.language.t(TrKey::InvCategoryLabel)).color(c.text_muted));
                    let categories = ["Tất cả", "Tài Nguyên", "Vật Liệu", "Tăng Tốc", "Pha Lê", "Sách & Tên", "Khác"];
                    for cat in categories {
                        let is_active = self.item_category_filter == cat;
                        let cat_display = category_name(cat, self.language);
                        if ui.selectable_label(is_active, cat_display).clicked() {
                            self.item_category_filter = cat.to_string();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cnt_txt = self.language.t(TrKey::InvItemCount).replace("{}", &format_num(stats.item_stats.len()));
                        ui.label(
                            egui::RichText::new(cnt_txt)
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
                            egui::RichText::new(self.language.t(TrKey::InvRssSummaryTitle))
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.accent_gold),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let total_lbl = match self.language {
                                AppLanguage::Vietnamese => "Tổng cộng",
                                AppLanguage::English => "Total",
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}: {:.2}M RSS ({})",
                                    total_lbl,
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

                    let pack_mult = match self.language {
                        AppLanguage::Vietnamese => "gói",
                        AppLanguage::English => "packs",
                    };
                    let total_packs_desc = match self.language {
                        AppLanguage::Vietnamese => "Số lượng gói",
                        AppLanguage::English => "Total packs",
                    };
                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvFood)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.food_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.accent_gold));
                            let pack_label = if res.food_packs * 1_000 == res.food_val {
                                format!("{} {} (x1.000)", format_num(res.food_packs), pack_mult)
                            } else if res.food_packs == res.food_val {
                                format_num(res.food_val)
                            } else {
                                format!("{} {}", format_num(res.food_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvWood)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.wood_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.status_green));
                            let pack_label = if res.wood_packs * 1_000 == res.wood_val {
                                format!("{} {} (x1.000)", format_num(res.wood_packs), pack_mult)
                            } else if res.wood_packs == res.wood_val {
                                format_num(res.wood_val)
                            } else {
                                format!("{} {}", format_num(res.wood_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvStone)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.stone_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.heal_blue));
                            let pack_label = if res.stone_packs * 750 == res.stone_val {
                                format!("{} {} (x750)", format_num(res.stone_packs), pack_mult)
                            } else if res.stone_packs == res.stone_val {
                                format_num(res.stone_val)
                            } else {
                                format!("{} {}", format_num(res.stone_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvGold)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.gold_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.accent_gold));
                            let pack_label = if res.gold_packs * 500 == res.gold_val {
                                format!("{} {} (x500)", format_num(res.gold_packs), pack_mult)
                            } else if res.gold_packs == res.gold_val {
                                format_num(res.gold_val)
                            } else {
                                format!("{} {}", format_num(res.gold_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvChoicePacks)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(res.choice_packs), pack_mult)).font(egui::FontId::proportional(15.0)).strong().color(c.focus_teal));
                            ui.label(egui::RichText::new(total_packs_desc).font(egui::FontId::proportional(10.0)).color(c.text_muted));
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
                        let mat_summary_title = match self.language {
                            AppLanguage::Vietnamese => "TỔNG HỢP VẬT LIỆU TRANG BỊ & ĐIỂM HÀNH ĐỘNG (AP):",
                            AppLanguage::English => "MATERIALS & ACTION POINTS (AP) SUMMARY:",
                        };
                        ui.label(
                            egui::RichText::new(mat_summary_title)
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.focus_teal),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mat_total_str = match self.language {
                                AppLanguage::Vietnamese => format!(
                                    "Tổng: {} BT | {} NC => {} quy đổi | +{} AP",
                                    format_num(mats.total_normal()),
                                    format_num(mats.total_advanced()),
                                    format_num(mats.total_converted_normal()),
                                    format_num(ap.total_ap_points())
                                ),
                                AppLanguage::English => format!(
                                    "Total: {} Norm | {} Adv => {} conv | +{} AP",
                                    format_num(mats.total_normal()),
                                    format_num(mats.total_advanced()),
                                    format_num(mats.total_converted_normal()),
                                    format_num(ap.total_ap_points())
                                ),
                            };
                            ui.label(
                                egui::RichText::new(mat_total_str)
                                    .font(egui::FontId::proportional(12.5))
                                    .color(c.status_green)
                                    .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    let (conv_lbl, norm_lbl, adv_lbl) = match self.language {
                        AppLanguage::Vietnamese => ("quy đổi", "BT", "NC"),
                        AppLanguage::English => ("conv", "Norm", "Adv"),
                    };
                    let (leather_lbl, iron_lbl, ebony_lbl, bone_lbl, ap_lbl) = match self.language {
                        AppLanguage::Vietnamese => ("Da (Leather)", "Quặng Sắt (Iron)", "Gỗ Mun (Ebony)", "Xương (Bone)", "Điểm Hành Động"),
                        AppLanguage::English => ("Leather", "Iron Ore", "Ebony Wood", "Animal Bone", "Action Points"),
                    };
                    let pots_lbl = match self.language {
                        AppLanguage::Vietnamese => "bình",
                        AppLanguage::English => "pots",
                    };
                    let total_pots_lbl = match self.language {
                        AppLanguage::Vietnamese => "Tổng",
                        AppLanguage::English => "Total",
                    };
                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new(leather_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.leather_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.train_orange));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.leather_normal), adv_lbl, format_num(mats.leather_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new(iron_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.iron_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_iron));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.iron_normal), adv_lbl, format_num(mats.iron_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new(ebony_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.ebony_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_ebony));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.ebony_normal), adv_lbl, format_num(mats.ebony_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new(bone_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.bone_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_bone));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.bone_normal), adv_lbl, format_num(mats.bone_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new(ap_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("+{} AP", format_num(ap.total_ap_points()))).font(egui::FontId::proportional(14.0)).strong().color(c.status_green));
                            ui.label(egui::RichText::new(format!("{} {} {}", total_pots_lbl, format_num(ap.total_bottles()), pots_lbl)).font(egui::FontId::proportional(10.0)).color(c.text_muted));
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
                    ItemSortColumn::Name => a
                        .display_name(self.language)
                        .cmp(b.display_name(self.language)),
                    ItemSortColumn::Category => category_name(&a.category, self.language)
                        .cmp(category_name(&b.category, self.language)),
                    ItemSortColumn::TotalQty => a.total_quantity.cmp(&b.total_quantity),
                    ItemSortColumn::Converted => a.raw_converted_value().cmp(&b.raw_converted_value()),
                    ItemSortColumn::Battles => a.battle_count.cmp(&b.battle_count),
                    ItemSortColumn::DropRate => a.battle_count.cmp(&b.battle_count),
                    ItemSortColumn::AvgPerBattle => a.avg_per_battle.partial_cmp(&b.avg_per_battle).unwrap_or(std::cmp::Ordering::Equal),
                    ItemSortColumn::MaxInBattle => a.max_in_battle.cmp(&b.max_in_battle),
                };
                match self.item_sort_dir {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                }
            });
        }

        // Inventory Table Responsive Metrics
        const INV_PAD_X: f32 = 8.0;
        const INV_SPACING_X: f32 = 6.0;

        const COL_CAT: f32 = 88.0;
        const COL_TOTAL: f32 = 78.0;
        const COL_CONV: f32 = 98.0;
        const COL_BATTLES: f32 = 58.0;
        const COL_RATE: f32 = 66.0;
        const COL_AVG: f32 = 62.0;
        const COL_MAX: f32 = 62.0;

        let avail_w = ui.available_width();
        // 8 items total, 7 gaps
        let inv_fixed_w = COL_CAT
            + COL_TOTAL
            + COL_CONV
            + COL_BATTLES
            + COL_RATE
            + COL_AVG
            + COL_MAX
            + (INV_PAD_X * 2.0)
            + (INV_SPACING_X * 7.0);
        let min_name_w = 190.0_f32;
        let inv_table_width = avail_w.max(inv_fixed_w + min_name_w);
        let names_w = inv_table_width - inv_fixed_w;

        let mut toggle_item_sort_col = None;

        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(INV_PAD_X, 5.0))
            .show(ui, |ui| {
                ui.set_width(inv_table_width - (INV_PAD_X * 2.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = INV_SPACING_X;
                    ui.set_min_height(24.0);

                    if render_item_sort_header_cell(
                        ui,
                        self.language.t(TrKey::InvColName),
                        ItemSortColumn::Name,
                        self.item_sort_col,
                        self.item_sort_dir,
                        names_w,
                        egui::Align::LEFT,
                        c.text_muted,
                        c.focus_teal,
                        self.language,
                    ) {
                        toggle_item_sort_col = Some(ItemSortColumn::Name);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColCategory), ItemSortColumn::Category, self.item_sort_col, self.item_sort_dir, COL_CAT, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Category);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColTotalQty), ItemSortColumn::TotalQty, self.item_sort_col, self.item_sort_dir, COL_TOTAL, egui::Align::RIGHT, c.accent_gold, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::TotalQty);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColConverted), ItemSortColumn::Converted, self.item_sort_col, self.item_sort_dir, COL_CONV, egui::Align::RIGHT, c.status_green, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Converted);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColBattles), ItemSortColumn::Battles, self.item_sort_col, self.item_sort_dir, COL_BATTLES, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Battles);
                    }

                    let (rate_hdr, avg_hdr, max_hdr) = match self.language {
                        AppLanguage::Vietnamese => ("Tỷ Lệ Rơi", "TB / Trận", "Max / Trận"),
                        AppLanguage::English => ("Drop Rate", "Avg / Battle", "Max / Battle"),
                    };
                    if render_item_sort_header_cell(ui, rate_hdr, ItemSortColumn::DropRate, self.item_sort_col, self.item_sort_dir, COL_RATE, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::DropRate);
                    }
                    if render_item_sort_header_cell(ui, avg_hdr, ItemSortColumn::AvgPerBattle, self.item_sort_col, self.item_sort_dir, COL_AVG, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::AvgPerBattle);
                    }
                    if render_item_sort_header_cell(ui, max_hdr, ItemSortColumn::MaxInBattle, self.item_sort_col, self.item_sort_dir, COL_MAX, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::MaxInBattle);
                    }
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

        if filtered_items.is_empty() {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(24.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(16.0);
                        let empty_title = match self.language {
                            AppLanguage::Vietnamese => "Không Tìm Thấy Vật Phẩm",
                            AppLanguage::English => "No Items Found",
                        };
                        ui.label(
                            egui::RichText::new(empty_title)
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(c.canvas_tint),
                        );
                        ui.add_space(4.0);
                        let empty_desc = if !self.item_search.trim().is_empty() {
                            match self.language {
                                AppLanguage::Vietnamese => "Không có vật phẩm nào khớp với từ khóa tìm kiếm.",
                                AppLanguage::English => "No items matched your search query.",
                            }
                        } else {
                            match self.language {
                                AppLanguage::Vietnamese => "Chưa có dữ liệu vật phẩm nào được ghi nhận trong phiên này.",
                                AppLanguage::English => "No inventory items recorded yet.",
                            }
                        };
                        ui.label(
                            egui::RichText::new(empty_desc)
                                .font(egui::FontId::proportional(11.5))
                                .color(c.text_muted),
                        );
                        ui.add_space(16.0);
                    });
                });
            return;
        }

        let row_height = 29.0_f32;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show_rows(ui, row_height, filtered_items.len(), |ui, row_range| {
                ui.set_width(inv_table_width);
                for idx in row_range {
                    let it = filtered_items[idx];
                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(inv_table_width, 27.0),
                        egui::Sense::hover(),
                    );

                    let is_hovered = row_resp.hovered();
                    if is_hovered {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::same(4.0),
                            c.focus_teal.linear_multiply(0.12),
                        );
                        ui.painter().rect_stroke(
                            row_rect,
                            Rounding::same(4.0),
                            Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.40)),
                        );
                    } else if idx % 2 == 1 {
                        // Subtle alternating row zebra stripe for scanning wide tables
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::same(4.0),
                            c.hairline.linear_multiply(0.12),
                        );
                    }

                    row_resp.on_hover_text(it.display_name(self.language));

                    let inner_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x + INV_PAD_X, row_rect.min.y),
                        egui::pos2(row_rect.max.x - INV_PAD_X, row_rect.max.y),
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = INV_SPACING_X;

                            ui.add_sized(
                                [names_w, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(it.display_name(self.language))
                                        .strong()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::LEFT),
                            );

                            let cat_color = category_badge_color(&it.category, &c);
                            egui::Frame::none()
                                .fill(cat_color.linear_multiply(0.12))
                                .stroke(Stroke::new(1.0_f32, cat_color.linear_multiply(0.35)))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(4.0, 1.0))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        [COL_CAT - 8.0, 18.0],
                                        egui::Label::new(
                                            egui::RichText::new(category_name(&it.category, self.language))
                                                .font(egui::FontId::proportional(10.5))
                                                .strong()
                                                .color(cat_color),
                                        )
                                        .truncate()
                                        .halign(egui::Align::Center),
                                    );
                                });

                            ui.add_sized(
                                [COL_TOTAL, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format_num(it.total_quantity))
                                        .monospace()
                                        .strong()
                                        .color(c.accent_gold),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );

                            let conv_str = get_item_converted_display_for_lang(it.itype, it.sub_type, it.total_quantity, self.language).unwrap_or_else(|| "-".to_string());
                            let conv_color = if conv_str != "-" { c.status_green } else { c.text_muted };
                            ui.add_sized(
                                [COL_CONV, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(conv_str)
                                        .monospace()
                                        .strong()
                                        .color(conv_color),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );

                            let drop_rate = (it.battle_count as f64 / total_battles as f64) * 100.0;
                            ui.add_sized(
                                [COL_BATTLES, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{}", it.battle_count))
                                        .monospace()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_RATE, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:.1}%", drop_rate))
                                        .monospace()
                                        .strong()
                                        .color(c.focus_teal),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_AVG, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:.1}", it.avg_per_battle))
                                        .monospace()
                                        .color(c.text_muted),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_MAX, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format_num(it.max_in_battle))
                                        .monospace()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardMailTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardMailDesc)).font(egui::FontId::proportional(10.5)).color(c.text_muted));
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
                            if ui.button(self.language.t(TrKey::SettingsBtnBrowse)).clicked() {
                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                    self.mail_path_input = folder.to_string_lossy().to_string();
                                    self.watcher.set_mail_dir(folder);
                                    self.save_config();
                                }
                            }
                            if ui.button(self.language.t(TrKey::SettingsBtnDefault)).clicked() {
                                let def_mail = detect_default_mail_dir();
                                self.mail_path_input = def_mail.to_string_lossy().to_string();
                                self.watcher.set_mail_dir(def_mail);
                                self.save_config();
                            }
                            if ui.button(self.language.t(TrKey::SettingsBtnOpenExplorer)).clicked() {
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardScanTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardScanDesc)).font(egui::FontId::proportional(10.5)).color(c.text_muted));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new(self.language.t(TrKey::SettingsBtnScanNew)).strong()).on_hover_text(self.language.t(TrKey::SettingsBtnScanNewTooltip)).clicked() {
                                self.watcher.rescan();
                            }
                            if ui.button(egui::RichText::new(self.language.t(TrKey::SettingsBtnRescanForce)).color(c.accent_gold).strong()).on_hover_text(self.language.t(TrKey::SettingsBtnRescanForceTooltip)).clicked() {
                                self.watcher.rescan_force();
                            }
                        });
                    });

                ui.add_space(10.0);

                // Card: In-Game Overlay (Rise of Kingdoms)
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        let title = match self.language {
                            AppLanguage::Vietnamese => "HIỂN THỊ ĐÈ TRONG GAME (RISE OF KINGDOMS OVERLAY)",
                            AppLanguage::English => "IN-GAME OVERLAY (RISE OF KINGDOMS)",
                        };
                        ui.label(
                            egui::RichText::new(title)
                                .font(egui::FontId::proportional(12.0))
                                .strong()
                                .color(c.focus_teal),
                        );
                        ui.add_space(4.0);

                        let desc = match self.language {
                            AppLanguage::Vietnamese => "Tự động hiển thị 5 báo cáo mới nhất ghim vào góc trái cửa sổ game Rise of Kingdoms với tính năng xuyên chuột (click-through).",
                            AppLanguage::English => "Automatically displays 5 latest reports pinned to the left edge of Rise of Kingdoms window with click-through safety.",
                        };
                        ui.label(egui::RichText::new(desc).font(egui::FontId::proportional(10.5)).color(c.text_muted));
                        ui.add_space(8.0);

                        // Toggle
                        let prev_enabled = self.overlay_config.enabled;
                        let toggle_lbl = match self.language {
                            AppLanguage::Vietnamese => "Bật hiển thị đè (Enable Overlay)",
                            AppLanguage::English => "Enable In-Game Overlay",
                        };
                        ui.checkbox(&mut self.overlay_config.enabled, egui::RichText::new(toggle_lbl).strong().color(c.canvas_tint));
                        if self.overlay_config.enabled != prev_enabled {
                            self.save_config();
                        }

                        if self.overlay_config.enabled {
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // Filter selection
                            let filter_lbl = match self.language {
                                AppLanguage::Vietnamese => "Loại báo cáo:",
                                AppLanguage::English => "Report Filter:",
                            };
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(filter_lbl).strong().color(c.canvas_tint));
                                let prev_filter = self.overlay_config.filter;
                                egui::ComboBox::from_id_salt("overlay_filter_select")
                                    .selected_text(match self.overlay_config.filter {
                                        FilterMode::AllRewards => if self.language == AppLanguage::Vietnamese { "Tất cả phần thưởng" } else { "All Rewards" },
                                        FilterMode::AllTargets => if self.language == AppLanguage::Vietnamese { "Tất cả mục tiêu" } else { "All Targets" },
                                        FilterMode::Motte => "Motte Only",
                                        FilterMode::Fort => if self.language == AppLanguage::Vietnamese { "Pháo đài" } else { "Forts Only" },
                                        FilterMode::Barbarian => if self.language == AppLanguage::Vietnamese { "Dã man" } else { "Barbarians Only" },
                                        FilterMode::Gathering => if self.language == AppLanguage::Vietnamese { "Thu thập" } else { "Gathering Only" },
                                        _ => "Filtered",
                                    })
                                    .show_ui(ui, |ui| {
                                        let all_lbl = if self.language == AppLanguage::Vietnamese { "Tất cả phần thưởng" } else { "All Rewards" };
                                        ui.selectable_value(&mut self.overlay_config.filter, FilterMode::AllRewards, all_lbl);
                                        let motte_lbl = "Motte Only";
                                        ui.selectable_value(&mut self.overlay_config.filter, FilterMode::Motte, motte_lbl);
                                        let fort_lbl = if self.language == AppLanguage::Vietnamese { "Pháo đài" } else { "Forts Only" };
                                        ui.selectable_value(&mut self.overlay_config.filter, FilterMode::Fort, fort_lbl);
                                        let barb_lbl = if self.language == AppLanguage::Vietnamese { "Dã man" } else { "Barbarians Only" };
                                        ui.selectable_value(&mut self.overlay_config.filter, FilterMode::Barbarian, barb_lbl);
                                        let rss_lbl = if self.language == AppLanguage::Vietnamese { "Thu thập" } else { "Gathering Only" };
                                        ui.selectable_value(&mut self.overlay_config.filter, FilterMode::Gathering, rss_lbl);
                                    });
                                if self.overlay_config.filter != prev_filter {
                                    self.save_config();
                                }
                            });

                            ui.add_space(8.0);

                            // Opacity slider
                            let opacity_lbl = match self.language {
                                AppLanguage::Vietnamese => "Độ mờ lớp phủ:",
                                AppLanguage::English => "Overlay Opacity:",
                            };
                            let prev_opacity = self.overlay_config.opacity;
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(opacity_lbl).strong().color(c.canvas_tint));
                                ui.add(
                                    egui::Slider::new(&mut self.overlay_config.opacity, 0.20_f32..=1.0_f32)
                                        .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
                                );
                            });
                            if (self.overlay_config.opacity - prev_opacity).abs() > 0.005 {
                                self.last_mail_arrival = std::time::Instant::now();
                                self.save_config();
                            }

                            ui.add_space(8.0);

                            // Color theme swatches
                            let color_lbl = match self.language {
                                AppLanguage::Vietnamese => "Màu chữ:",
                                AppLanguage::English => "Text Color:",
                            };
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(color_lbl).strong().color(c.canvas_tint));
                                let themes = [
                                    ("#F8FAFC", "White", Color32::from_rgb(0xF8, 0xFA, 0xFC)),
                                    ("#FDE047", "Gold", Color32::from_rgb(0xFD, 0xE0, 0x47)),
                                    ("#38BDF8", "Cyan", Color32::from_rgb(0x38, 0xBD, 0xF8)),
                                    ("#4ADE80", "Green", Color32::from_rgb(0x4A, 0xDE, 0x80)),
                                ];
                                for (hex, name, col) in themes {
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("[ {} ]", name))
                                            .color(col)
                                            .strong(),
                                    );
                                    if ui.add(btn).clicked() {
                                        self.overlay_config.text_color_hex = hex.to_string();
                                        self.last_mail_arrival = std::time::Instant::now();
                                        self.save_config();
                                    }
                                }
                            });
                        }
                    });

                ui.add_space(10.0);

                // Card 3: App Behavior & Protection
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardSystemTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.canvas_tint));
                        ui.add_space(8.0);

                        // Language Selector
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::SettingsLanguageLabel)).strong().color(c.canvas_tint));
                            let prev_lang = self.language;
                            ui.selectable_value(&mut self.language, AppLanguage::Vietnamese, "Tiếng Việt");
                            ui.selectable_value(&mut self.language, AppLanguage::English, "English");
                            if self.language != prev_lang {
                                self.watcher.set_language(self.language);
                                self.save_config();
                                if let Some(tray) = &self.tray {
                                    tray.update_language(self.language);
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCloseBehavior)).strong().color(c.canvas_tint));
                            let prev_close = self.close_behavior;
                            ui.selectable_value(&mut self.close_behavior, CloseBehavior::Tray, self.language.t(TrKey::SettingsCloseTray));
                            ui.selectable_value(&mut self.close_behavior, CloseBehavior::Close, self.language.t(TrKey::SettingsCloseExit));
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
                            if ui.checkbox(&mut autostart, egui::RichText::new(self.language.t(TrKey::SettingsAutostart)).strong().color(c.canvas_tint))
                                .on_hover_text(self.language.t(TrKey::SettingsAutostartTooltip))
                                .changed()
                            {
                                self.autostart = autostart;
                                if let Err(e) = crate::autostart::set_autostart(self.autostart, self.autostart_minimized) {
                                    let err_msg = match self.language {
                                        AppLanguage::Vietnamese => format!("Lỗi cài đặt Autostart: {}", e),
                                        AppLanguage::English => format!("Autostart setting error: {}", e),
                                    };
                                    self.watcher.log("ERROR", &err_msg);
                                } else {
                                    let msg = if self.autostart {
                                        match self.language {
                                            AppLanguage::Vietnamese => format!("Đã BẬT tự khởi động cùng Windows (Thu nhỏ vào khay: {})", if self.autostart_minimized { "BẬT" } else { "TẮT" }),
                                            AppLanguage::English => format!("ENABLED Autostart with Windows (Start minimized: {})", if self.autostart_minimized { "YES" } else { "NO" }),
                                        }
                                    } else {
                                        match self.language {
                                            AppLanguage::Vietnamese => "Đã TẮT tự khởi động cùng Windows".to_string(),
                                            AppLanguage::English => "DISABLED Autostart with Windows".to_string(),
                                        }
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
                                if ui.checkbox(&mut start_min, egui::RichText::new(self.language.t(TrKey::SettingsAutostartMinimized)).color(min_color))
                                    .on_hover_text(self.language.t(TrKey::SettingsAutostartMinimizedTooltip))
                                    .changed()
                                {
                                    self.autostart_minimized = start_min;
                                    if let Err(e) = crate::autostart::set_autostart(self.autostart, self.autostart_minimized) {
                                        let err_msg = match self.language {
                                            AppLanguage::Vietnamese => format!("Lỗi cập nhật cấu hình Autostart: {}", e),
                                            AppLanguage::English => format!("Autostart config update error: {}", e),
                                        };
                                        self.watcher.log("ERROR", &err_msg);
                                    } else {
                                        let msg = if self.autostart_minimized {
                                            match self.language {
                                                AppLanguage::Vietnamese => "Đã BẬT tùy chọn: Thu nhỏ xuống khay icon khi khởi động cùng Windows",
                                                AppLanguage::English => "ENABLED: Start minimized to system tray on Windows startup",
                                            }
                                        } else {
                                            match self.language {
                                                AppLanguage::Vietnamese => "Đã TẮT tùy chọn: Thu nhỏ xuống khay icon khi khởi động cùng Windows (sẽ hiện cửa sổ)",
                                                AppLanguage::English => "DISABLED: Start minimized to system tray on Windows startup (will show window)",
                                            }
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
                            self.language.t(TrKey::SettingsArmorOn)
                        } else {
                            self.language.t(TrKey::SettingsArmorOff)
                        };
                        let armor_color = if self.is_armored { c.status_green } else { c.text_muted };

                        if ui.button(egui::RichText::new(armor_label).color(armor_color).strong())
                            .on_hover_text(self.language.t(TrKey::SettingsArmorTooltip))
                            .clicked()
                        {
                            let mail_dir = self.watcher.get_mail_dir();
                            if self.is_armored {
                                match crate::armor::disable_mailcache_armor(&mail_dir, self.language) {
                                    Ok(msg) => {
                                        self.watcher.log("SUCCESS", &msg);
                                        self.is_armored = false;
                                    }
                                    Err(err) => {
                                        self.watcher.log("ERROR", &err);
                                    }
                                }
                            } else {
                                match crate::armor::enable_mailcache_armor(&mail_dir, self.language) {
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCardVaultTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.accent_gold));
                        ui.add_space(4.0);
                        let secured_str = self.language.t(TrKey::SettingsVaultSecured).replace("{}", &format_num(vault_total));
                        ui.label(egui::RichText::new(secured_str).font(egui::FontId::proportional(11.0)).color(c.canvas_tint));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui.button(self.language.t(TrKey::SettingsBtnOpenVaultDir)).clicked() {
                                let dir = self.watcher.get_vault_binary_dir();
                                let _ = open::that(&dir);
                            }
                            if ui.button(self.language.t(TrKey::SettingsBtnOpenVaultJson)).clicked() {
                                let f = self.watcher.get_vault_file_path();
                                let _ = open::that(&f);
                            }
                        });
                    });
            });

            // Right Column: Terminal Log Viewer
            egui::Frame::none()
                .fill(c.log_bg)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(&mut cols[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(self.language.t(TrKey::LogsTitle)).font(egui::FontId::proportional(12.0)).strong().color(c.focus_teal));
                        let count_str = self.language.t(TrKey::LogsLinesCount).replace("{}", &self.console_logs.len().to_string());
                        ui.label(egui::RichText::new(count_str).font(egui::FontId::proportional(10.5)).color(c.text_muted));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.language.t(TrKey::LogsBtnClear)).clicked() {
                                self.console_logs.clear();
                            }
                            ui.checkbox(&mut self.log_auto_scroll, self.language.t(TrKey::LogsAutoScroll));
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
                                ui.label(egui::RichText::new(self.language.t(TrKey::LogsEmpty)).color(c.text_muted));
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
                egui::RichText::new(self.language.t(TrKey::InspectorTitle))
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(c.focus_teal),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(self.language.t(TrKey::InspectorClose)).on_hover_text(self.language.t(TrKey::InspectorCloseTooltip)).clicked() {
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
                        egui::RichText::new(record.display_target(self.language))
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

                    let badge_label = match record.battle_type.as_str() {
                        "gathering" => match self.language {
                            AppLanguage::Vietnamese => "THU THẬP",
                            AppLanguage::English => "GATHERING",
                        },
                        "motte" => match self.language {
                            AppLanguage::Vietnamese => "MOTTE",
                            AppLanguage::English => "MOTTE",
                        },
                        "fort" => match self.language {
                            AppLanguage::Vietnamese => "PHÁO ĐÀI",
                            AppLanguage::English => "FORT",
                        },
                        _ => match self.language {
                            AppLanguage::Vietnamese => "DÃ MAN",
                            AppLanguage::English => "BARBARIAN",
                        },
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
                    let time_label = self.language.t(TrKey::InspectorTime).replace("{}", &record.time_vn);
                    ui.label(egui::RichText::new(time_label).font(egui::FontId::proportional(11.5)).color(c.text_muted));
                    if record.battle_type == "gathering" {
                        let lvl_display = if !record.target_level.is_empty() {
                            match self.language {
                                AppLanguage::Vietnamese => format!("Cấp {}", record.target_level),
                                AppLanguage::English => format!("Level {}", record.target_level),
                            }
                        } else {
                            "-".to_string()
                        };
                        let lvl_label = self.language.t(TrKey::InspectorTileLevel).replace("{}", &lvl_display);
                        ui.label(egui::RichText::new(lvl_label).font(egui::FontId::proportional(11.5)).strong().color(c.focus_teal));
                        if record.talent_add > 0 {
                            let bonus_label = self.language.t(TrKey::InspectorTalentAdd).replace("{}", &format_num(record.talent_add));
                            ui.label(egui::RichText::new(bonus_label).font(egui::FontId::proportional(11.5)).color(c.status_green));
                        }
                    } else {
                        let dmg_label = self.language.t(TrKey::InspectorDamage).replace("{}", &record.damage_pct.to_string());
                        ui.label(egui::RichText::new(dmg_label).font(egui::FontId::proportional(11.5)).strong().color(c.focus_teal));
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::InspectorRssGained)).font(egui::FontId::proportional(11.0)).strong().color(c.accent_gold));
                        ui.add_space(4.0);
                        let choice_str = if res.choice_packs > 0 {
                            match self.language {
                                AppLanguage::Vietnamese => format!(" | Gói C: {} gói", format_num(res.choice_packs)),
                                AppLanguage::English => format!(" | Choice Chests: {}", format_num(res.choice_packs)),
                            }
                        } else {
                            String::new()
                        };
                        let (lbl_total, lbl_food, lbl_wood, lbl_stone, lbl_gold) = match self.language {
                            AppLanguage::Vietnamese => ("Tổng", "Ngô", "Gỗ", "Đá", "Vàng"),
                            AppLanguage::English => ("Total", "Food", "Wood", "Stone", "Gold"),
                        };
                        ui.label(
                            egui::RichText::new(format!(
                                "{}: {:.2}M ({}: {:.2}M, {}: {:.2}M, {}: {:.2}M, {}: {:.2}M){}",
                                lbl_total,
                                res.total_rss_val() as f64 / 1_000_000.0,
                                lbl_food,
                                res.food_val as f64 / 1_000_000.0,
                                lbl_wood,
                                res.wood_val as f64 / 1_000_000.0,
                                lbl_stone,
                                res.stone_val as f64 / 1_000_000.0,
                                lbl_gold,
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::InspectorSpeedupGained)).font(egui::FontId::proportional(11.0)).strong().color(c.status_green));
                        ui.add_space(4.0);
                        let (lbl_univ, lbl_train, lbl_heal) = match self.language {
                            AppLanguage::Vietnamese => ("Đa năng", "Đào tạo", "Hồi phục"),
                            AppLanguage::English => ("Univ", "Training", "Healing"),
                        };
                        let mut spd_parts = Vec::new();
                        if u_min > 0 { spd_parts.push(format!("{}: {}", lbl_univ, self.speedup_unit.format_duration_for_language(u_min as f64, self.language))); }
                        if t_min > 0 { spd_parts.push(format!("{}: {}", lbl_train, self.speedup_unit.format_duration_for_language(t_min as f64, self.language))); }
                        if h_min > 0 { spd_parts.push(format!("{}: {}", lbl_heal, self.speedup_unit.format_duration_for_language(h_min as f64, self.language))); }
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
                        ui.label(egui::RichText::new(self.language.t(TrKey::InspectorMaterialsAp)).font(egui::FontId::proportional(11.0)).strong().color(c.focus_teal));
                        ui.add_space(4.0);
                        let mut parts = Vec::new();
                        let unit_suffix = match self.language {
                            AppLanguage::Vietnamese => "BT",
                            AppLanguage::English => "Norm",
                        };
                        if mats.leather_total_converted() > 0 {
                            parts.push(format!("{}: {} {}", self.language.t(TrKey::InspectorLeather), mats.leather_total_converted(), unit_suffix));
                        }
                        if mats.iron_total_converted() > 0 {
                            parts.push(format!("{}: {} {}", self.language.t(TrKey::InspectorIron), mats.iron_total_converted(), unit_suffix));
                        }
                        if mats.ebony_total_converted() > 0 {
                            parts.push(format!("{}: {} {}", self.language.t(TrKey::InspectorEbony), mats.ebony_total_converted(), unit_suffix));
                        }
                        if mats.bone_total_converted() > 0 {
                            parts.push(format!("{}: {} {}", self.language.t(TrKey::InspectorBone), mats.bone_total_converted(), unit_suffix));
                        }
                        if ap.total_ap_points() > 0 {
                            let bottles_label = match self.language {
                                AppLanguage::Vietnamese => format!("{} bình", ap.total_bottles()),
                                AppLanguage::English => format!("{} pots", ap.total_bottles()),
                            };
                            parts.push(format!("+{} AP ({})", ap.total_ap_points(), bottles_label));
                        }
                        ui.label(egui::RichText::new(parts.join("  |  ")).font(egui::FontId::proportional(11.0)).color(c.canvas_tint));
                    });
                ui.add_space(8.0);
            }

            // Items List
            let items_list_label = self.language.t(TrKey::InspectorItemsList).replace("{}", &record.items.len().to_string());
            ui.label(egui::RichText::new(items_list_label).font(egui::FontId::proportional(11.5)).strong().color(c.canvas_tint));
            ui.add_space(6.0);

            if record.items.is_empty() {
                ui.label(egui::RichText::new(self.language.t(TrKey::InspectorNoItems)).color(c.text_muted));
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
                                    ui.label(egui::RichText::new(it.display_name(self.language)).strong().color(c.canvas_tint));
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("[{}]", category_name(&it.category, self.language))).font(egui::FontId::proportional(10.0)).color(c.focus_teal));
                                        if let Some(conv_str) = get_item_converted_display_for_lang(it.itype, it.sub_type, it.value, self.language) {
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

        // Nhận kết quả kiểm tra trạng thái armor từ luồng nền nếu có
        self.poll_armor_check();

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

        // In-game overlay multi-viewport handling
        if self.overlay_config.enabled {
            let mut is_active_rendered = false;

            if let Some(hwnd) = crate::overlay_tracker::find_rok_window() {
                if crate::overlay_tracker::is_rok_active_and_visible(hwnd) {
                    if let Some(bounds) = crate::overlay_tracker::get_rok_window_bounds(hwnd) {
                        is_active_rendered = true;
                        let geom = crate::overlay_tracker::calculate_overlay_geometry(&bounds, &self.overlay_config, ctx.pixels_per_point());
                        let overlay_id = egui::ViewportId::from_hash_of("rok_overlay_viewport");
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

                        let filtered = self.watcher.get_latest_vault_records(
                            self.overlay_config.filter,
                            5,
                        );
                        let overlay_cfg = self.overlay_config.clone();
                        let last_mail = self.last_mail_arrival;

                        #[cfg(windows)]
                        if self.overlay_styled_hwnd.is_none() {
                            let title_wide: Vec<u16> = "Rise of Kingdoms Loot Overlay\0".encode_utf16().collect();
                            unsafe {
                                let vhwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                                    std::ptr::null(),
                                    title_wide.as_ptr(),
                                );
                                if vhwnd != 0 as _ {
                                    crate::overlay_tracker::apply_click_through_styles(vhwnd);
                                    self.overlay_styled_hwnd = Some(vhwnd as isize);
                                }
                            }
                        }

                        ctx.show_viewport_immediate(overlay_id, builder, move |ctx, _class| {
                            crate::overlay_ui::render_overlay_viewport(ctx, &filtered, &overlay_cfg, last_mail);
                        });
                    }
                }
            }

            let delay_ms = if is_active_rendered { 100 } else { 500 };
            ctx.request_repaint_after(std::time::Duration::from_millis(delay_ms));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReportTableDimensions {
    table_width: f32,
    target_width: f32,
    extra_per_column: f32,
}

fn report_table_dimensions(
    available_width: f32,
    fixed_width: f32,
    non_target_column_count: usize,
) -> ReportTableDimensions {
    const MIN_TARGET_WIDTH: f32 = 100.0;
    const MAX_TARGET_WIDTH: f32 = 180.0;

    let target_width = (available_width - fixed_width).clamp(MIN_TARGET_WIDTH, MAX_TARGET_WIDTH);
    let extra_per_column = if non_target_column_count == 0 {
        0.0
    } else {
        ((available_width - fixed_width - target_width).max(0.0)) / non_target_column_count as f32
    };

    ReportTableDimensions {
        table_width: fixed_width + target_width + extra_per_column * non_target_column_count as f32,
        target_width,
        extra_per_column,
    }
}

fn report_non_target_column_count(is_gathering: bool, is_barb: bool) -> usize {
    if is_gathering {
        7
    } else if is_barb {
        11
    } else {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppLanguage;

    #[test]
    fn test_overlay_state_initialization() {
        let config = AppConfig::default();
        assert!(!config.overlay.enabled);
        assert_eq!(config.overlay.filter, FilterMode::AllRewards);
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
        assert_eq!(category_badge_color("Tăng tốc", &colors), colors.speedup_purple);
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

        // In Dark theme, Watching status produces green badge and "● ĐANG THEO DÕI"
        let (dark_text, dark_color) = resolve_status_badge(&status, true, AppLanguage::Vietnamese, &dark_colors);
        assert_eq!(dark_text, format!("● {}", AppLanguage::Vietnamese.t(TrKey::StatusWatching)));
        assert_eq!(dark_color, dark_colors.status_green);

        // When toggling to Light theme, status text MUST stay the exact same (not flip to Paused!)
        let (light_text, light_color) = resolve_status_badge(&status, true, AppLanguage::Vietnamese, &light_colors);
        assert_eq!(light_text, dark_text);
        assert_eq!(light_color, light_colors.status_green);

        // Stopped status
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

        // Light mode card surface is pure white for elevation over Slate-50 background
        assert_eq!(light.card_surface, Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(light.ink, Color32::from_rgb(0xf8, 0xfa, 0xfc));

        // Terminal Log viewer background in Light mode is pure white, not black
        assert_eq!(light.log_bg, Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(dark.log_bg, Color32::from_rgb(0x0d, 0x0f, 0x14));

        // Text colors: canvas_tint is dark Slate-900 for >16:1 contrast ratio
        assert_eq!(light.canvas_tint, Color32::from_rgb(0x0f, 0x17, 0x2a));
        // text_muted is Slate-600 for >7.5:1 contrast ratio against white cards
        assert_eq!(light.text_muted, Color32::from_rgb(0x47, 0x55, 0x69));

        // Accent colors: light mode uses deepened shades (700-level) for WCAG AA compliance
        assert_eq!(light.train_orange, Color32::from_rgb(0xc2, 0x41, 0x0c));
        assert_eq!(light.heal_blue, Color32::from_rgb(0x03, 0x69, 0xa1));
        assert_eq!(light.status_green, Color32::from_rgb(0x04, 0x78, 0x57));
        assert_eq!(light.accent_gold, Color32::from_rgb(0xb4, 0x53, 0x09));
        assert_eq!(light.speedup_purple, Color32::from_rgb(0x7e, 0x22, 0xce));
        assert_eq!(light.crystal_cyan, Color32::from_rgb(0x0e, 0x74, 0x90));

        // Material badges
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

        // Item 1: Crystals (itype 1, sub 9, category "Pha Lê")
        let item_crystals = make_item(1, 9, 100, "Pha Lê");
        // Item 2: Resources - 150k Food (itype 2, sub 2, qty 3 -> 3 * 50k = 150k, category "Tài nguyên")
        let item_resources_150k = make_item(2, 2, 3, "Tài nguyên");
        // Item 3: Resources - 2.00M Food (itype 2, sub 2, qty 40 -> 40 * 50k = 2,000,000, category "Tài nguyên")
        let item_resources_2m = make_item(2, 2, 40, "Tài nguyên");
        // Item 4: Resources - 50k Food (itype 2, sub 2, qty 1 -> 50,000, category "Tài nguyên")
        let item_resources_50k = make_item(2, 2, 1, "Tài nguyên");
        // Item 5: Speedups (itype 2, sub 58, qty 10 -> 10 * 5m = 50m, category "Tăng tốc")
        let item_speedups = make_item(2, 58, 10, "Tăng tốc");

        // Verify category translations in English:
        // "Pha Lê" -> "Crystals"
        // "Tài nguyên" -> "Resources"
        // "Tăng tốc" -> "Speedups"
        let lang = AppLanguage::English;
        assert_eq!(category_name(&item_crystals.category, lang), "Crystals");
        assert_eq!(category_name(&item_resources_150k.category, lang), "Resources");
        assert_eq!(category_name(&item_speedups.category, lang), "Speedups");

        // Verify English category sorting orders "Crystals" before "Resources" and "Speedups"
        let mut items = vec![&item_speedups, &item_resources_150k, &item_crystals];
        items.sort_by(|a, b| {
            category_name(&a.category, lang).cmp(category_name(&b.category, lang))
        });
        assert_eq!(category_name(&items[0].category, lang), "Crystals");
        assert_eq!(category_name(&items[1].category, lang), "Resources");
        assert_eq!(category_name(&items[2].category, lang), "Speedups");

        // Verify converted numerical sorting: 2,000,000 Food sorts above 150,000 Food (descending)
        assert_eq!(item_resources_50k.raw_converted_value(), 50_000);
        assert_eq!(item_resources_150k.raw_converted_value(), 150_000);
        assert_eq!(item_resources_2m.raw_converted_value(), 2_000_000);

        let mut converted_items = vec![&item_resources_150k, &item_resources_2m];
        // Descending sort by raw_converted_value: 2,000,000 Food sorts above 150,000 Food
        converted_items.sort_by(|a, b| {
            b.raw_converted_value().cmp(&a.raw_converted_value())
        });
        assert_eq!(converted_items[0].raw_converted_value(), 2_000_000);
        assert_eq!(converted_items[1].raw_converted_value(), 150_000);
        assert_eq!(converted_items[0].total_quantity, 40);
        assert_eq!(converted_items[1].total_quantity, 3);

        // All 3 items descending: 2,000,000 > 150,000 > 50,000
        let mut all_res = vec![&item_resources_150k, &item_resources_50k, &item_resources_2m];
        all_res.sort_by(|a, b| {
            b.raw_converted_value().cmp(&a.raw_converted_value())
        });
        assert_eq!(all_res[0].raw_converted_value(), 2_000_000);
        assert_eq!(all_res[1].raw_converted_value(), 150_000);
        assert_eq!(all_res[2].raw_converted_value(), 50_000);

        // Ascending sort by raw_converted_value
        converted_items.sort_by(|a, b| {
            a.raw_converted_value().cmp(&b.raw_converted_value())
        });
        assert_eq!(converted_items[0].raw_converted_value(), 150_000);
        assert_eq!(converted_items[1].raw_converted_value(), 2_000_000);
    }

    #[test]
    fn test_pending_armor_check_disconnect_clears_receiver() {
        let (tx, rx) = std::sync::mpsc::channel::<bool>();
        drop(tx); // sender drops, causing Disconnected

        let mut pending = Some(rx);
        let mut armored = false;
        poll_armor_check_channel(&mut pending, &mut armored);
        assert!(pending.is_none());
        assert!(!armored);
    }

    #[test]
    fn test_pending_armor_check_ok_sets_armored_and_clears_receiver() {
        let (tx, rx) = std::sync::mpsc::channel::<bool>();
        tx.send(true).unwrap();

        let mut pending = Some(rx);
        let mut armored = false;
        poll_armor_check_channel(&mut pending, &mut armored);
        assert!(pending.is_none());
        assert!(armored);
    }

    #[test]
    fn test_pending_armor_check_empty_leaves_receiver_pending() {
        let (_tx, rx) = std::sync::mpsc::channel::<bool>();

        let mut pending = Some(rx);
        let mut armored = false;
        poll_armor_check_channel(&mut pending, &mut armored);
        assert!(pending.is_some());
        assert!(!armored);
    }
}
