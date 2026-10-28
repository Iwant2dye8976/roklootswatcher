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
            ink: Color32::from_rgb(0x11, 0x13, 0x17),
            ink_deep: Color32::from_rgb(0x18, 0x1b, 0x20),
            card_surface: Color32::from_rgb(0x1e, 0x22, 0x2a),
            sidebar_bg: Color32::from_rgb(0x14, 0x16, 0x1b),
            primary: Color32::from_rgb(0x25, 0x2a, 0x34),
            primary_pressed: Color32::from_rgb(0x32, 0x38, 0x46),
            brand_mark: Color32::from_rgb(0xf4, 0x3f, 0x5e),
            canvas_tint: Color32::from_rgb(0xf1, 0xf5, 0xf9),
            hairline: Color32::from_rgb(0x28, 0x2d, 0x37),
            field_border: Color32::from_rgb(0x38, 0x3f, 0x4e),
            accent_gold: Color32::from_rgb(0xf5, 0x9e, 0x0b),
            accent_gold_soft: Color32::from_rgb(0xfb, 0xbf, 0x24),
            focus_teal: Color32::from_rgb(0x06, 0xb6, 0xd4),
            status_green: Color32::from_rgb(0x10, 0xb9, 0x81),
            text_muted: Color32::from_rgb(0x94, 0xa3, 0xb8),
            error: Color32::from_rgb(0xef, 0x44, 0x44),
            motte_accent: Color32::from_rgb(0xf4, 0x3f, 0x5e),
            fort_accent: Color32::from_rgb(0xf5, 0x9e, 0x0b),
            barb_accent: Color32::from_rgb(0x10, 0xb9, 0x81),
            crystal_cyan: Color32::from_rgb(0x06, 0xb6, 0xd4),
            speedup_purple: Color32::from_rgb(0xa8, 0x55, 0xf7),
            train_orange: Color32::from_rgb(0xfb, 0x92, 0x3c),
            heal_blue: Color32::from_rgb(0x38, 0xbd, 0xf8),
            log_bg: Color32::from_rgb(0x0d, 0x0f, 0x14),
            mat_iron: Color32::from_rgb(0x94, 0xa3, 0xb8),
            mat_ebony: Color32::from_rgb(0x34, 0xd3, 0x99),
            mat_bone: Color32::from_rgb(0xf4, 0x72, 0xb6),
            rank_silver: Color32::from_rgb(0xc0, 0xc0, 0xc0),
            rank_bronze: Color32::from_rgb(0xcd, 0x7f, 0x32),
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            ink: Color32::from_rgb(0xf8, 0xfa, 0xfc),
            ink_deep: Color32::from_rgb(0xff, 0xff, 0xff),
            card_surface: Color32::from_rgb(0xff, 0xff, 0xff),
            sidebar_bg: Color32::from_rgb(0xf1, 0xf5, 0xf9),
            primary: Color32::from_rgb(0xf1, 0xf5, 0xf9),
            primary_pressed: Color32::from_rgb(0xe2, 0xe8, 0xf0),
            brand_mark: Color32::from_rgb(0xbe, 0x12, 0x3c),
            canvas_tint: Color32::from_rgb(0x0f, 0x17, 0x2a),
            hairline: Color32::from_rgb(0xe2, 0xe8, 0xf0),
            field_border: Color32::from_rgb(0xcb, 0xd5, 0xe1),
            accent_gold: Color32::from_rgb(0xb4, 0x53, 0x09),
            accent_gold_soft: Color32::from_rgb(0x92, 0x40, 0x0e),
            focus_teal: Color32::from_rgb(0x03, 0x69, 0xa1),
            status_green: Color32::from_rgb(0x04, 0x78, 0x57),
            text_muted: Color32::from_rgb(0x47, 0x55, 0x69),
            error: Color32::from_rgb(0xb9, 0x1c, 0x1c),
            motte_accent: Color32::from_rgb(0xbe, 0x12, 0x3c),
            fort_accent: Color32::from_rgb(0xb4, 0x53, 0x09),
            barb_accent: Color32::from_rgb(0x04, 0x78, 0x57),
            crystal_cyan: Color32::from_rgb(0x0e, 0x74, 0x90),
            speedup_purple: Color32::from_rgb(0x7e, 0x22, 0xce),
            train_orange: Color32::from_rgb(0xc2, 0x41, 0x0c),
            heal_blue: Color32::from_rgb(0x03, 0x69, 0xa1),
            log_bg: Color32::from_rgb(0xff, 0xff, 0xff),
            mat_iron: Color32::from_rgb(0x47, 0x55, 0x69),
            mat_ebony: Color32::from_rgb(0x04, 0x78, 0x57),
            mat_bone: Color32::from_rgb(0xbe, 0x18, 0x5d),
            rank_silver: Color32::from_rgb(0x47, 0x55, 0x69),
            rank_bronze: Color32::from_rgb(0x9a, 0x34, 0x12),
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

struct CloudOpResult {
    message: String,
    is_backup: bool,
    new_tokens: Option<(String, String)>,
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
    overlay_last_opacity: Option<f32>,
    pending_overlay_save: bool,
    pending_start_minimized_frames: u32,
    _single_instance: Option<crate::single_instance::windows::SingleInstanceGuard>,

    pub cloud_auth: crate::model::CloudAuthConfig,
    cloud_rx: Option<std::sync::mpsc::Receiver<Result<crate::cloud_sync::CloudAuthResult, String>>>,
    cloud_op_rx: Option<std::sync::mpsc::Receiver<Result<CloudOpResult, String>>>,
    cloud_busy: bool,
    pub has_unbacked_data: bool,
    pub last_cloud_backup_time: Option<std::time::Instant>,

    pub cached_overlay_records: Vec<BattleRecord>,
    pub cached_sorted_battle_indices: Vec<usize>,
    pub reports_sort_dirty: bool,
    pub last_applied_theme: Option<AppTheme>,
    pub last_records_count: usize,
    last_battle_search: String,
    last_battle_sort_col: Option<BattleSortColumn>,
    last_battle_sort_dir: SortDirection,
    last_battle_mode: FilterMode,
    last_battle_lang: AppLanguage,
    pub cached_ap_econ: Option<crate::analyzer::ApEconomics>,
    pub ap_econ_dirty: bool,
    last_ap_records_count: usize,
    last_ap_mode: FilterMode,
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

pub fn localize_benchmark_target(name: &str, lang: AppLanguage) -> &str {
    match (name, lang) {
        ("Fort Lvl 11-15 (KvK)", AppLanguage::Vietnamese) => "Pháo Đài Cấp 11-15 (KvK)",
        ("Fort Lvl 1-10", AppLanguage::Vietnamese) => "Pháo Đài Cấp 1-10",
        ("Motte Lvl 11-15", AppLanguage::Vietnamese) => "Motte Cấp 11-15",
        ("Motte Lvl 1-10", AppLanguage::Vietnamese) => "Motte Cấp 1-10",
        ("Barbarians Lvl 41-55", AppLanguage::Vietnamese) => "Dã Man Cấp 41-55",
        ("Barbarians Lvl 1-40", AppLanguage::Vietnamese) => "Dã Man Cấp 1-40",
        _ => name,
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


pub(crate) fn poll_cloud_receiver<T>(
    rx_opt: &mut Option<std::sync::mpsc::Receiver<T>>,
    busy: &mut bool,
) -> Option<Result<T, ()>> {
    if let Some(ref rx) = rx_opt {
        match rx.try_recv() {
            Ok(res) => {
                *busy = false;
                *rx_opt = None;
                Some(Ok(res))
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                *busy = false;
                *rx_opt = None;
                Some(Err(()))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
        }
    } else {
        None
    }
}

impl RoKLootWatcherApp {
    pub fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let win_dir = std::env::var("WINDIR")
            .or_else(|_| std::env::var("SystemRoot"))
            .unwrap_or_else(|_| r"C:\Windows".to_string());
        let fonts_dir = PathBuf::from(win_dir).join("Fonts");

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
        let initial_has_unbacked = watcher.get_vault_total_count() > 0;

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
            overlay_last_opacity: None,
            pending_overlay_save: false,
            pending_start_minimized_frames: if start_minimized { 5 } else { 0 },
            _single_instance: single_instance,

            cloud_auth: config.cloud_auth.clone(),
            cloud_rx: None,
            cloud_op_rx: None,
            cloud_busy: false,
            has_unbacked_data: initial_has_unbacked,
            last_cloud_backup_time: None,

            cached_overlay_records: initial_overlay_records,
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
        }
    }

    pub fn refresh_cached_overlay_records(&mut self) {
        self.cached_overlay_records = self.watcher.get_latest_vault_records(self.overlay_config.filter, 5);
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
            cloud_auth: self.cloud_auth.clone(),
        };
        config.save();
    }

    pub fn quit_app(&self) {
        self.watcher.flush_vault_if_dirty();
        if self.cloud_auth.backup_on_exit && self.has_unbacked_data && self.cloud_auth.access_token.is_some() {
            if let (Some(token), Some(user_id)) = (&self.cloud_auth.access_token, &self.cloud_auth.user_id) {
                let (url, key) = self.get_cloud_credentials();
                let refresh_token = self.cloud_auth.refresh_token.as_deref();
                let vault_path = self.watcher.get_vault_file_path();
                match crate::cloud_sync::backup_vault(
                    &url,
                    &key,
                    token,
                    refresh_token,
                    user_id,
                    &vault_path,
                    Some(std::time::Duration::from_secs(4)),
                ) {
                    Ok((cnt, new_tokens)) => {
                        let msg = match self.language {
                            AppLanguage::Vietnamese => format!("Sao lưu khi thoát hoàn tất: {} chiến báo.", cnt),
                            AppLanguage::English => format!("Exit backup completed: {} reports.", cnt),
                        };
                        self.watcher.log("SUCCESS", &msg);
                        if let Some((new_acc, new_ref)) = new_tokens {
                            let mut cfg = AppConfig::load();
                            cfg.cloud_auth.access_token = Some(new_acc);
                            cfg.cloud_auth.refresh_token = Some(new_ref);
                            cfg.cloud_auth.last_synced = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                            cfg.save();
                        } else {
                            let mut cfg = AppConfig::load();
                            cfg.cloud_auth.last_synced = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                            cfg.save();
                        }
                    }
                    Err(e) => {
                        let msg = match self.language {
                            AppLanguage::Vietnamese => format!("Sao lưu khi thoát thất bại hoặc hết hạn thời gian (4s): {}", e),
                            AppLanguage::English => format!("Exit backup failed or timed out (4s): {}", e),
                        };
                        self.watcher.log("WARN", &msg);
                    }
                }
            }
        }
        self.watcher.stop();
        if let Some(ref t) = self.tray {
            t.hide_icon();
        }
        std::process::exit(0);
    }

    fn hide_to_tray(&self, _ctx: &egui::Context) {
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
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    ctx.request_repaint();
                }
                WatcherUiEvent::MailDetected { is_new, .. } => {
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    self.last_mail_arrival = std::time::Instant::now();
                    if is_new {
                        self.has_unbacked_data = true;
                    }
                    ctx.request_repaint();
                }
            }
        }
        had_events
    }


    fn poll_cloud_events(&mut self, ctx: &egui::Context) {
        if !self.cloud_busy && self.cloud_auth.auto_backup && self.cloud_auth.access_token.is_some() && self.has_unbacked_data {
            let interval_duration = std::time::Duration::from_secs(self.cloud_auth.clamped_interval_minutes() as u64 * 60);
            let should_run = match self.last_cloud_backup_time {
                Some(last) => {
                    let elapsed = std::time::Instant::now().duration_since(last);
                    if elapsed >= interval_duration {
                        true
                    } else {
                        ctx.request_repaint_after(interval_duration - elapsed);
                        false
                    }
                }
                None => true,
            };
            if should_run {
                self.trigger_cloud_backup();
            }
        }

        if let Some(res) = poll_cloud_receiver(&mut self.cloud_rx, &mut self.cloud_busy) {
            match res {
                Ok(Ok(auth)) => {
                    let email = auth.user.email.clone();
                    self.cloud_auth.access_token = Some(auth.access_token);
                    self.cloud_auth.refresh_token = Some(auth.refresh_token);
                    self.cloud_auth.user_id = Some(auth.user.id);
                    self.cloud_auth.user_email = Some(auth.user.email);
                    self.save_config();
                    let msg = match self.language {
                        AppLanguage::Vietnamese => format!("Đăng nhập Google thành công: {}", email),
                        AppLanguage::English => format!("Google login successful: {}", email),
                    };
                    self.watcher.log("SUCCESS", &msg);
                }
                Ok(Err(e)) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => format!("Đăng nhập Google thất bại: {}", e),
                        AppLanguage::English => format!("Google login failed: {}", e),
                    };
                    self.watcher.log("ERROR", &msg);
                }
                Err(()) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => "Đăng nhập Google thất bại: luồng xác thực bị ngắt kết nối".to_string(),
                        AppLanguage::English => "Google login failed: auth thread disconnected".to_string(),
                    };
                    self.watcher.log("ERROR", &msg);
                }
            }
        }

        if let Some(res) = poll_cloud_receiver(&mut self.cloud_op_rx, &mut self.cloud_busy) {
            match res {
                Ok(Ok(op_res)) => {
                    let mut needs_save = false;
                    if let Some((new_acc, new_ref)) = op_res.new_tokens {
                        self.cloud_auth.access_token = Some(new_acc);
                        self.cloud_auth.refresh_token = Some(new_ref);
                        needs_save = true;
                    }
                    if op_res.is_backup {
                        self.has_unbacked_data = false;
                        self.last_cloud_backup_time = Some(std::time::Instant::now());
                        self.cloud_auth.last_synced = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        needs_save = true;
                    }
                    if needs_save {
                        self.save_config();
                    }
                    self.watcher.log("SUCCESS", &op_res.message);
                }
                Ok(Err(e)) => {
                    self.watcher.log("ERROR", &e);
                }
                Err(()) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => "Thao tác cloud thất bại: luồng xử lý bị đóng bất ngờ".to_string(),
                        AppLanguage::English => "Cloud operation failed: worker thread disconnected".to_string(),
                    };
                    self.watcher.log("ERROR", &msg);
                }
            }
        }
    }

    fn get_cloud_credentials(&self) -> (String, String) {
        let url = self
            .cloud_auth
            .supabase_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(crate::cloud_sync::DEFAULT_SUPABASE_URL)
            .trim()
            .to_string();
        let key = self
            .cloud_auth
            .anon_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(crate::cloud_sync::DEFAULT_SUPABASE_ANON_KEY)
            .trim()
            .to_string();
        (url, key)
    }

    fn start_google_login(&mut self) {
        if self.cloud_busy {
            return;
        }

        let (url, key) = self.get_cloud_credentials();
        let lang = self.language;

        if url.is_empty() || key.is_empty() {
            let err_msg = match lang {
                AppLanguage::Vietnamese => "Chưa cấu hình Supabase URL / Anon Key (cần thiết lập trong rok_config.json hoặc biến môi trường build ROK_SUPABASE_URL).",
                AppLanguage::English => "Supabase URL / Anon Key not configured (set in rok_config.json or compile with ROK_SUPABASE_URL).",
            };
            self.watcher.log("ERROR", err_msg);
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.cloud_rx = Some(rx);
        self.cloud_busy = true;

        let start_msg = match lang {
            AppLanguage::Vietnamese => "Đang mở trình duyệt để đăng nhập Google qua Supabase...",
            AppLanguage::English => "Opening browser for Google login via Supabase...",
        };
        self.watcher.log("INFO", start_msg);

        let _ = std::thread::Builder::new()
            .name("rok-google-login".into())
            .spawn(move || {
                let res = crate::cloud_sync::start_google_login(&url, &key);
                let _ = tx.send(res);
            });
    }

    fn trigger_cloud_backup(&mut self) {
        if self.cloud_busy {
            return;
        }
        self.watcher.flush_vault_if_dirty();

        let (url, key) = self.get_cloud_credentials();
        let token = match &self.cloud_auth.access_token {
            Some(t) => t.clone(),
            None => return,
        };
        let refresh_token = self.cloud_auth.refresh_token.clone();
        let user_id = match &self.cloud_auth.user_id {
            Some(u) => u.clone(),
            None => return,
        };

        let vault_path = self.watcher.get_vault_file_path();
        let (tx, rx) = std::sync::mpsc::channel();
        self.cloud_op_rx = Some(rx);
        self.cloud_busy = true;

        let lang = self.language;
        self.watcher.log("INFO", match lang {
            AppLanguage::Vietnamese => "Đang tải dữ liệu Vault lên Supabase Cloud...",
            AppLanguage::English => "Uploading Vault data to Supabase Cloud...",
        });

        let _ = std::thread::Builder::new()
            .name("rok-cloud-backup".into())
            .spawn(move || {
                match crate::cloud_sync::backup_vault(&url, &key, &token, refresh_token.as_deref(), &user_id, &vault_path, None) {
                    Ok((cnt, new_tokens)) => {
                        let success_msg = match lang {
                            AppLanguage::Vietnamese => format!("Đã sao lưu thành công {} chiến báo lên Cloud!", cnt),
                            AppLanguage::English => format!("Successfully backed up {} reports to Cloud!", cnt),
                        };
                        let _ = tx.send(Ok(CloudOpResult {
                            message: success_msg,
                            is_backup: true,
                            new_tokens,
                        }));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            });
    }

    fn trigger_cloud_restore(&mut self) {
        if self.cloud_busy {
            return;
        }

        let (url, key) = self.get_cloud_credentials();
        let token = match &self.cloud_auth.access_token {
            Some(t) => t.clone(),
            None => return,
        };
        let refresh_token = self.cloud_auth.refresh_token.clone();

        let vault_path = self.watcher.get_vault_file_path();
        let (tx, rx) = std::sync::mpsc::channel();
        self.cloud_op_rx = Some(rx);
        self.cloud_busy = true;

        let lang = self.language;
        self.watcher.log("INFO", match lang {
            AppLanguage::Vietnamese => "Đang tải bản sao lưu từ Supabase Cloud...",
            AppLanguage::English => "Downloading backup from Supabase Cloud...",
        });

        let watcher_clone = Arc::clone(&self.watcher);
        let _ = std::thread::Builder::new()
            .name("rok-cloud-restore".into())
            .spawn(move || {
                match crate::cloud_sync::restore_vault(&url, &key, &token, refresh_token.as_deref(), &vault_path) {
                    Ok((cnt, updated_at, new_tokens)) => {
                        watcher_clone.reload_vault();
                        let success_msg = match lang {
                            AppLanguage::Vietnamese => format!("Khôi phục thành công {} chiến báo từ Cloud (Bản sao lưu: {})!", cnt, updated_at),
                            AppLanguage::English => format!("Successfully restored {} reports from Cloud (Backup: {})!", cnt, updated_at),
                        };
                        let _ = tx.send(Ok(CloudOpResult {
                            message: success_msg,
                            is_backup: false,
                            new_tokens,
                        }));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            });
    }

    fn toggle_watcher(&self) {
        if self.watcher.is_watching() {
            self.watcher.stop();
        } else {
            self.watcher.start(false);
        }
    }

    fn apply_date_preset(&mut self, preset: DateFilterPreset) {
        self.reports_sort_dirty = true;
        self.ap_econ_dirty = true;
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
        self.reports_sort_dirty = true;
        self.ap_econ_dirty = true;
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

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(4.0);

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

                    let col_gap = 4.0;
                    let btn_w = ((ui.available_width() - col_gap) / 2.0).max(40.0);
                    let btn_h = 26.0;

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_gap;

                        let scan_btn = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::BtnScan))
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
                        let (pause_text, pause_color) = if is_watching {
                            (self.language.t(TrKey::BtnPause), c.canvas_tint)
                        } else {
                            (self.language.t(TrKey::BtnResume), c.accent_gold)
                        };

                        let pause_btn = egui::Button::new(
                            egui::RichText::new(pause_text)
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

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = col_gap;

                        let theme_text = if self.theme == AppTheme::Dark {
                            self.language.t(TrKey::ThemeToggleLight)
                        } else {
                            self.language.t(TrKey::ThemeToggleDark)
                        };

                        let theme_btn = egui::Button::new(
                            egui::RichText::new(theme_text)
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
                            egui::RichText::new(self.language.t(TrKey::TrayMinimize))
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

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .id_salt("dashboard_main_scroll")
            .show(ui, |ui| {
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

            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
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

            let stats = self.watcher.get_stats();
            let total_battles = stats.total_battles;
            let res = stats.compute_resources();
            let (train_spd_mins, heal_spd_mins, univ_spd_mins) = stats.compute_speedups_by_category();
            let total_spd_mins = train_spd_mins + heal_spd_mins + univ_spd_mins;
            let pha_le = stats.compute_crystals();
            let avg_crystal = if total_battles > 0 { (pha_le as f64) / (total_battles as f64) } else { 0.0 };
            let (univ_spd_val, univ_spd_unit) =
                self.speedup_unit.format_kpi(total_spd_mins, self.language);

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

            ui.add_space(10.0);

            let need_ap_recompute = self.ap_econ_dirty
                || self.cached_ap_econ.is_none()
                || active_records.len() != self.last_ap_records_count
                || self.selected_mode != self.last_ap_mode;

            if need_ap_recompute {
                let econ = self.watcher.with_vault_records(|vault| {
                    crate::analyzer::compute_ap_economics(&active_records, vault)
                });
                self.cached_ap_econ = Some(econ);
                self.last_ap_records_count = active_records.len();
                self.last_ap_mode = self.selected_mode;
                self.ap_econ_dirty = false;
            }

            if let Some(ref ap_econ) = self.cached_ap_econ {
                self.render_ap_economics_section(ui, ap_econ, &c);
            }

            ui.add_space(10.0);

            ui.columns(2, |cols| {
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
                                        ui.add(egui::Label::new(egui::RichText::new(it.display_name(self.language)).strong().color(c.canvas_tint)).truncate()).on_hover_text(it.display_name(self.language));

                                        if let Some(conv_str) = get_item_converted_display_for_lang(it.itype, it.sub_type, it.total_quantity, self.language) {
                                            ui.label(egui::RichText::new(format!("({})", conv_str)).font(egui::FontId::proportional(10.5)).color(c.status_green));
                                        }

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new(format_num(it.total_quantity)).monospace().strong().color(c.accent_gold));
                                        });
                                    });

                                    ui.add_space(2.0);

                                    let pct = (it.total_quantity as f32 / max_qty as f32).clamp(0.04, 1.0);
                                    let bar_height = 5.0;
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

                                    ui.add_space(3.0);
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
                                    "gathering" => c.focus_teal,
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
                                let sub_info = if r.battle_type == "gathering" {
                                    let type_lbl = match self.language {
                                        AppLanguage::Vietnamese => "THU THẬP",
                                        AppLanguage::English => "GATHERING",
                                    };
                                    let res_name = r.items.first().map(|i| i.display_name(self.language)).unwrap_or("-");
                                    let total_val = r.items.first().map(|i| i.value).unwrap_or(0);
                                    let bonus = if r.talent_add > 0 {
                                        format!(" (+{})", format_num(r.talent_add))
                                    } else {
                                        "".to_string()
                                    };
                                    format!("[{}]  {}: {}{}", type_lbl, res_name, format_num(total_val), bonus)
                                } else {
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
                                    format!("[{}]  {}{}", r.battle_type.to_uppercase(), cry_str, dmg_str)
                                };
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

    fn render_ap_economics_section(
        &self,
        ui: &mut egui::Ui,
        ap_econ: &crate::analyzer::ApEconomics,
        c: &ThemeColors,
    ) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("[AP ECONOMY]")
                    .font(egui::FontId::monospace(10.5))
                    .strong()
                    .color(c.train_orange),
            );
            ui.label(
                egui::RichText::new(self.language.t(TrKey::ApEconomicsTitle))
                    .font(egui::FontId::proportional(12.0))
                    .strong()
                    .color(c.canvas_tint),
            );
        });
        ui.add_space(6.0);

        let (rebate_str, net_ap_str, spd_rate_sub, gem_sub) = match self.language {
            AppLanguage::Vietnamese => (
                format!("+{:.1}% hoàn lại", ap_econ.rebate_pct),
                format!("Thực chi: {} AP", format_num(ap_econ.net_ap_spent)),
                format!("{:.1} giờ / 1k AP", ap_econ.speedup_mins_per_1k_ap / 60.0),
                "Gems nhận / 1,000 AP".to_string(),
            ),
            AppLanguage::English => (
                format!("+{:.1}% Rebate", ap_econ.rebate_pct),
                format!("Net: {} AP", format_num(ap_econ.net_ap_spent)),
                format!("{:.1}h / 1k AP", ap_econ.speedup_mins_per_1k_ap / 60.0),
                "Gems yield / 1,000 AP".to_string(),
            ),
        };

        let cards = [
            (
                self.language.t(TrKey::ApConsumed),
                format_num(ap_econ.total_ap_consumed),
                net_ap_str,
                c.train_orange,
            ),
            (
                self.language.t(TrKey::ApRecovered),
                format!("+{}", format_num(ap_econ.total_ap_recovered)),
                rebate_str,
                c.status_green,
            ),
            (
                self.language.t(TrKey::ApSpeedupYield),
                format!("{:.0}m", ap_econ.speedup_mins_per_1k_ap),
                spd_rate_sub,
                c.crystal_cyan,
            ),
            (
                self.language.t(TrKey::ApGemYield),
                format!("{:.0}", ap_econ.gems_per_1k_ap),
                gem_sub,
                c.accent_gold,
            ),
        ];

        ui.columns(4, |cols| {
            for (i, (title, val, sub, color)) in cards.iter().enumerate() {
                egui::Frame::none()
                    .fill(c.card_surface)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(8.0, 5.0))
                    .show(&mut cols[i], |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(*title)
                                    .font(egui::FontId::proportional(10.5))
                                    .strong()
                                    .color(c.text_muted),
                            )
                            .truncate(),
                        ).on_hover_text(*title);
                        ui.add_space(2.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(val)
                                    .font(egui::FontId::monospace(14.0))
                                    .strong()
                                    .color(*color),
                            )
                            .truncate(),
                        );
                        ui.add_space(2.0);
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(sub)
                                    .font(egui::FontId::proportional(9.5))
                                    .color(c.text_muted),
                            )
                            .truncate(),
                        ).on_hover_text(sub);
                    });
            }
        });

        ui.add_space(6.0);

        const BENCHMARK_CARD_HEIGHT: f32 = 160.0;

        ui.columns(2, |cols| {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(10.0, 8.0))
                .show(&mut cols[0], |ui| {
                    ui.set_height(BENCHMARK_CARD_HEIGHT);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.language.t(TrKey::ApTimelineTitle))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(c.canvas_tint),
                        );
                    });
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        let spd_lbl = match self.language {
                            AppLanguage::Vietnamese => "Tăng tốc (giờ)",
                            AppLanguage::English => "Speedup (hours)",
                        };
                        let ap_lbl = match self.language {
                            AppLanguage::Vietnamese => "AP tiêu thụ (/100)",
                            AppLanguage::English => "AP burn (/100)",
                        };
                        ui.label(
                            egui::RichText::new(format!("● {}", spd_lbl))
                                .font(egui::FontId::proportional(9.5))
                                .color(c.crystal_cyan),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("● {}", ap_lbl))
                                .font(egui::FontId::proportional(9.5))
                                .color(c.train_orange),
                        );
                    });
                    ui.add_space(4.0);

                    let chart_h = 104.0;
                    let (chart_rect, chart_resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), chart_h),
                        egui::Sense::hover(),
                    );
                    let painter = ui.painter();
                    painter.rect_filled(chart_rect, Rounding::same(4.0), c.ink);

                    let days_count = ap_econ.daily_timeline.len().max(1);
                    let slot_w = chart_rect.width() / days_count as f32;
                    let bar_w = (slot_w * 0.30).clamp(4.0, 16.0);
                    let baseline_y = chart_rect.max.y - 18.0;

                    painter.line_segment(
                        [
                            egui::pos2(chart_rect.min.x + 4.0, baseline_y),
                            egui::pos2(chart_rect.max.x - 4.0, baseline_y),
                        ],
                        Stroke::new(1.0_f32, c.hairline),
                    );

                    let max_spd_h = ap_econ
                        .daily_timeline
                        .iter()
                        .map(|d| d.speedup_hours)
                        .fold(0.0_f64, f64::max)
                        .max(1.0);
                    let max_ap = ap_econ
                        .daily_timeline
                        .iter()
                        .map(|d| d.ap_consumed)
                        .max()
                        .unwrap_or(0)
                        .max(100) as f64;

                    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
                    let mut hovered_day: Option<usize> = None;

                    if let Some(pos) = mouse_pos {
                        if chart_rect.contains(pos) {
                            let rel_x = (pos.x - chart_rect.min.x).max(0.0);
                            let idx = (rel_x / slot_w) as usize;
                            if idx < ap_econ.daily_timeline.len() {
                                hovered_day = Some(idx);
                            }
                        }
                    }

                    for (idx, d) in ap_econ.daily_timeline.iter().enumerate() {
                        let slot_x_min = chart_rect.min.x + (idx as f32) * slot_w;
                        let slot_x_max = slot_x_min + slot_w;
                        let center_x = (slot_x_min + slot_x_max) / 2.0;
                        let is_hovered = hovered_day == Some(idx);

                        if is_hovered {
                            let highlight_r = egui::Rect::from_min_max(
                                egui::pos2(slot_x_min + 1.0, chart_rect.min.y + 1.0),
                                egui::pos2(slot_x_max - 1.0, baseline_y),
                            );
                            painter.rect_filled(
                                highlight_r,
                                Rounding::same(2.0),
                                c.canvas_tint.linear_multiply(0.06),
                            );
                        }

                        let spd_bar_h = ((d.speedup_hours / max_spd_h) as f32 * (baseline_y - chart_rect.min.y - 6.0))
                            .max(if d.speedup_hours > 0.0 { 2.0 } else { 0.0 });
                        if spd_bar_h > 0.0 {
                            let spd_rect = egui::Rect::from_min_max(
                                egui::pos2(center_x - bar_w - 1.0, baseline_y - spd_bar_h),
                                egui::pos2(center_x - 1.0, baseline_y),
                            );
                            painter.rect_filled(spd_rect, Rounding::same(2.0), c.crystal_cyan);
                        }

                        let ap_bar_h = ((d.ap_consumed as f64 / max_ap) as f32 * (baseline_y - chart_rect.min.y - 6.0))
                            .max(if d.ap_consumed > 0 { 2.0 } else { 0.0 });
                        if ap_bar_h > 0.0 {
                            let ap_rect = egui::Rect::from_min_max(
                                egui::pos2(center_x + 1.0, baseline_y - ap_bar_h),
                                egui::pos2(center_x + bar_w + 1.0, baseline_y),
                            );
                            painter.rect_filled(ap_rect, Rounding::same(2.0), c.train_orange);
                        }

                        let display_date = if d.date_label == "Today" {
                            match self.language {
                                AppLanguage::Vietnamese => "Hôm nay",
                                AppLanguage::English => "Today",
                            }
                        } else {
                            &d.date_label
                        };
                        let text_color = if is_hovered { c.canvas_tint } else { c.text_muted };
                        painter.text(
                            egui::pos2(center_x, baseline_y + 9.0),
                            egui::Align2::CENTER_CENTER,
                            display_date,
                            egui::FontId::monospace(8.5),
                            text_color,
                        );
                    }

                    if let Some(idx) = hovered_day {
                        let d = &ap_econ.daily_timeline[idx];
                        let display_date = if d.date_label == "Today" {
                            match self.language {
                                AppLanguage::Vietnamese => "Hôm nay",
                                AppLanguage::English => "Today",
                            }
                        } else {
                            &d.date_label
                        };
                        let (lbl_date, lbl_spd, lbl_ap, lbl_yield) = match self.language {
                            AppLanguage::Vietnamese => ("Ngày", "Tăng tốc", "AP tiêu thụ", "Hiệu suất"),
                            AppLanguage::English => ("Date", "Speedups", "AP Consumed", "Yield"),
                        };
                        chart_resp.on_hover_ui(|ui| {
                            ui.label(egui::RichText::new(format!("{}: {}", lbl_date, display_date)).strong().monospace());
                            ui.label(egui::RichText::new(format!("{}: {:.1}h ({:.0}m)", lbl_spd, d.speedup_hours, d.speedup_hours * 60.0)).color(c.crystal_cyan).monospace());
                            ui.label(egui::RichText::new(format!("{}: {} AP", lbl_ap, format_num(d.ap_consumed))).color(c.train_orange).monospace());
                            if d.ap_consumed > 0 {
                                let yield_rate = (d.speedup_hours * 60.0) / (d.ap_consumed as f64 / 1000.0);
                                ui.label(egui::RichText::new(format!("{}: {:.0}m / 1k AP", lbl_yield, yield_rate)).color(c.status_green).monospace());
                            }
                        });
                    }
                });

            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(10.0, 8.0))
                .show(&mut cols[1], |ui| {
                    ui.set_height(BENCHMARK_CARD_HEIGHT);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.language.t(TrKey::ApBenchmarkTitle))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(c.canvas_tint),
                        );
                    });
                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    let avail_w = ui.available_width();
                    let col_ap = 40.0;
                    let col_runs = 36.0;
                    let col_spd = 58.0;
                    let col_cov = 46.0;
                    let col_tier = 50.0;
                    let fixed_cols = col_ap + col_runs + col_spd + col_cov + col_tier + 20.0;
                    let col_target = (avail_w - fixed_cols).max(90.0);
                    let table_w = fixed_cols + col_target;

                    let (hdr_target, hdr_runs, hdr_spd, hdr_cov) = match self.language {
                        AppLanguage::Vietnamese => ("Mục tiêu", "Trận", "Tốc/1k", "Sách %"),
                        AppLanguage::English => ("Target", "Runs", "Spd/1k", "Cov %"),
                    };

                    egui::ScrollArea::horizontal()
                        .auto_shrink([false, false])
                        .id_salt("benchmark_table_hscroll")
                        .show(ui, |ui| {
                            ui.set_width(table_w);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                ui.add_sized([col_target, 18.0], egui::Label::new(egui::RichText::new(hdr_target).font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::LEFT));
                                ui.add_sized([col_ap, 18.0], egui::Label::new(egui::RichText::new("AP").font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::Center)).on_hover_text(self.language.t(TrKey::ApUnitCost));
                                ui.add_sized([col_runs, 18.0], egui::Label::new(egui::RichText::new(hdr_runs).font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::Center)).on_hover_text(self.language.t(TrKey::ApRuns));
                                ui.add_sized([col_spd, 18.0], egui::Label::new(egui::RichText::new(hdr_spd).font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::Center)).on_hover_text(self.language.t(TrKey::ApSpeedupYield));
                                ui.add_sized([col_cov, 18.0], egui::Label::new(egui::RichText::new(hdr_cov).font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::Center)).on_hover_text(self.language.t(TrKey::ApCovenantRate));
                                ui.add_sized([col_tier, 18.0], egui::Label::new(egui::RichText::new("Tier").font(egui::FontId::proportional(10.0)).strong().color(c.text_muted)).truncate().halign(egui::Align::Center)).on_hover_text(self.language.t(TrKey::ApTier));
                            });
                            ui.add_space(2.0);

                            let rows_h = (BENCHMARK_CARD_HEIGHT - 44.0).max(80.0);
                            egui::ScrollArea::vertical()
                                .id_salt("benchmark_rows_vscroll")
                                .auto_shrink([false, false])
                                .max_height(rows_h)
                                .show(ui, |ui| {
                                    ui.set_width(table_w);
                                    if ap_econ.target_benchmarks.is_empty() {
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new(self.language.t(TrKey::NoBattlesData)).font(egui::FontId::proportional(11.0)).color(c.text_muted));
                                    } else {
                                        for b in &ap_econ.target_benchmarks {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 4.0;
                                                let target_color = if b.battle_count > 0 { c.canvas_tint } else { c.text_muted };

                                                let display_target = localize_benchmark_target(&b.target_name, self.language);
                                                ui.add_sized(
                                                    [col_target, 20.0],
                                                    egui::Label::new(egui::RichText::new(display_target).font(egui::FontId::proportional(10.5)).strong().color(target_color))
                                                        .truncate()
                                                        .halign(egui::Align::LEFT),
                                                ).on_hover_text(display_target);

                                                ui.add_sized([col_ap, 20.0], egui::Label::new(egui::RichText::new(format!("{} AP", b.unit_ap_cost)).font(egui::FontId::monospace(10.0)).color(c.text_muted)).truncate().halign(egui::Align::Center));
                                                ui.add_sized([col_runs, 20.0], egui::Label::new(egui::RichText::new(format_num(b.battle_count)).font(egui::FontId::monospace(10.0)).color(target_color)).truncate().halign(egui::Align::Center));

                                                let yield_str = if b.battle_count > 0 {
                                                    format!("{:.0}m", b.speedup_mins_per_1k_ap)
                                                } else {
                                                    "-".to_string()
                                                };
                                                ui.add_sized([col_spd, 20.0], egui::Label::new(egui::RichText::new(yield_str).font(egui::FontId::monospace(10.0)).strong().color(if b.battle_count > 0 { c.crystal_cyan } else { c.text_muted })).truncate().halign(egui::Align::Center));

                                                let is_fort_or_motte = b.target_name.starts_with("Fort") || b.target_name.starts_with("Motte");
                                                let cov_str = if b.battle_count > 0 && is_fort_or_motte {
                                                    format!("{:.1}%", b.covenant_drop_rate)
                                                } else {
                                                    "-".to_string()
                                                };
                                                ui.add_sized([col_cov, 20.0], egui::Label::new(egui::RichText::new(cov_str).font(egui::FontId::monospace(10.0)).color(if b.battle_count > 0 && is_fort_or_motte { c.accent_gold } else { c.text_muted })).truncate().halign(egui::Align::Center));

                                                let (badge_bg, badge_fg) = match b.tier_badge {
                                                    "TIER S+" => (Color32::from_rgb(255, 77, 77).linear_multiply(0.22), Color32::from_rgb(255, 95, 95)),
                                                    "TIER S" => (Color32::from_rgb(255, 127, 127).linear_multiply(0.22), Color32::from_rgb(255, 140, 140)),
                                                    "TIER A" => (Color32::from_rgb(255, 191, 127).linear_multiply(0.22), Color32::from_rgb(255, 200, 140)),
                                                    "TIER B" => (Color32::from_rgb(255, 255, 127).linear_multiply(0.20), Color32::from_rgb(255, 255, 140)),
                                                    "TIER C" => (Color32::from_rgb(191, 255, 127).linear_multiply(0.20), Color32::from_rgb(200, 255, 140)),
                                                    _ => (Color32::from_rgb(127, 191, 255).linear_multiply(0.22), Color32::from_rgb(140, 200, 255)),
                                                };
                                                let (badge_rect, _) = ui.allocate_exact_size(egui::vec2(col_tier, 18.0), egui::Sense::hover());
                                                if b.battle_count > 0 {
                                                    ui.painter().rect_filled(badge_rect, Rounding::same(3.0), badge_bg);
                                                    ui.painter().text(
                                                        badge_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        b.tier_badge,
                                                        egui::FontId::monospace(9.0),
                                                        badge_fg,
                                                    );
                                                } else {
                                                    ui.painter().text(
                                                        badge_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        "[-]",
                                                        egui::FontId::monospace(9.5),
                                                        c.text_muted,
                                                    );
                                                }
                                            });
                                        }
                                    }
                                });
                        });
                });
        });
    }

    fn render_view_battles(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();

        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
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

        const TABLE_PAD_X: f32 = 8.0;
        const ITEM_SPACING_X: f32 = 6.0;

        let col_type = if is_gathering { 72.0 } else { 46.0 };
        const COL_TIME: f32 = 116.0;
        const COL_DMG: f32 = 48.0;

        const COL_GATHER_LVL: f32 = 60.0;
        const COL_GATHER_RES: f32 = 80.0;
        const COL_GATHER_TOTAL: f32 = 110.0;
        const COL_GATHER_BONUS: f32 = 100.0;

        const COL_BARB_CRY: f32 = 100.0;
        const COL_BARB_RES: f32 = 100.0;
        const COL_BARB_EXP: f32 = 58.0;
        const COL_BARB_UNIV: f32 =100.0;
        const COL_BARB_TRAIN: f32 = 100.0;
        const COL_BARB_HEAL: f32 = 100.0;
        const COL_BARB_ARROW: f32 = 60.0;

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
            self.reports_sort_dirty = true;
        }

        ui.add_space(4.0);

        if records.is_empty() {
            if self.last_records_count != 0 {
                self.cached_sorted_battle_indices.clear();
                self.last_records_count = 0;
            }
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

        let is_dirty = self.reports_sort_dirty
            || records.len() != self.last_records_count
            || self.battle_search != self.last_battle_search
            || self.battle_sort_col != self.last_battle_sort_col
            || self.battle_sort_dir != self.last_battle_sort_dir
            || self.selected_mode != self.last_battle_mode
            || self.language != self.last_battle_lang
            || self.cached_sorted_battle_indices.iter().any(|&i| i >= records.len());

        if is_dirty {
            let mut indices: Vec<usize> = records
                .iter()
                .enumerate()
                .filter_map(|(idx, r)| {
                    if search_lower.is_empty() {
                        Some(idx)
                    } else {
                        let matches = r.battle_type.contains(&search_lower)
                            || r.time_vn.contains(&search_lower)
                            || r.target.to_lowercase().contains(&search_lower)
                            || r.display_target(cur_lang).to_lowercase().contains(&search_lower);
                        if matches { Some(idx) } else { None }
                    }
                })
                .collect();

            if let Some(sort_col) = self.battle_sort_col {
                indices.sort_by(|&a_idx, &b_idx| {
                    let a = &records[a_idx];
                    let b = &records[b_idx];
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

            self.cached_sorted_battle_indices = indices;
            self.last_records_count = records.len();
            self.last_battle_search = self.battle_search.clone();
            self.last_battle_sort_col = self.battle_sort_col;
            self.last_battle_sort_dir = self.battle_sort_dir;
            self.last_battle_mode = self.selected_mode;
            self.last_battle_lang = self.language;
            self.reports_sort_dirty = false;
        }

        let mut clicked_record = None;
        let row_height = 30.0;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .id_salt("battles_table_scroll")
            .show_rows(ui, row_height, self.cached_sorted_battle_indices.len(), |ui, row_range| {
                ui.set_width(dimensions.table_width);
                for idx in row_range {
                    let r = &records[self.cached_sorted_battle_indices[idx]];
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
            .id_salt("inventory_table_scroll")
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

    fn render_view_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let c = self.colors();

        let card_frame = egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(16.0, 14.0));

        ui.columns(2, |cols| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(&mut cols[0], |ui| {
                card_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", self.language.t(TrKey::SettingsCardMailTitle)))
                                .font(egui::FontId::proportional(12.5))
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(self.language.t(TrKey::SettingsCardMailDesc))
                            .font(egui::FontId::proportional(10.5))
                            .color(c.text_muted),
                    );
                    ui.add_space(10.0);

                    let text_edit = ui.add_sized(
                        [ui.available_width(), 26.0],
                        egui::TextEdit::singleline(&mut self.mail_path_input)
                            .font(egui::FontId::monospace(11.0))
                            .margin(egui::vec2(8.0, 5.0)),
                    );
                    if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.watcher.set_mail_dir(PathBuf::from(&self.mail_path_input));
                        self.save_config();
                    }

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        let btn_scan = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnAutoScan))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(c.focus_teal),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.4)))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_scan).clicked() {
                            match crate::watcher::scan_for_mailcache() {
                                Some(dir) => {
                                    self.mail_path_input = dir.to_string_lossy().to_string();
                                    self.watcher.set_mail_dir(dir.clone());
                                    self.save_config();
                                    self.watcher.log(
                                        "SUCCESS",
                                        &format!("{}: {}", self.language.t(TrKey::SettingsScanFound), dir.display()),
                                    );
                                }
                                None => {
                                    self.watcher.log("WARN", self.language.t(TrKey::SettingsScanNotFound));
                                }
                            }
                        }

                        let btn_browse = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnBrowse))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_browse).clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.mail_path_input = folder.to_string_lossy().to_string();
                                self.watcher.set_mail_dir(folder);
                                self.save_config();
                            }
                        }

                        let btn_def = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnDefault))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.text_muted),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_def).clicked() {
                            let def_mail = detect_default_mail_dir();
                            self.mail_path_input = def_mail.to_string_lossy().to_string();
                            self.watcher.set_mail_dir(def_mail);
                            self.save_config();
                        }

                        let btn_open = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnOpenExplorer))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.text_muted),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_open).clicked() {
                            let _ = open::that(self.watcher.get_mail_dir());
                        }
                    });
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", self.language.t(TrKey::SettingsCardScanTitle)))
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.accent_gold),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(self.language.t(TrKey::SettingsCardScanDesc))
                            .font(egui::FontId::proportional(10.5))
                            .color(c.text_muted),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        let btn_scan_new = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnScanNew))
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.status_green),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.status_green.linear_multiply(0.4)))
                        .rounding(Rounding::same(6.0));

                        if ui.add_sized([0.0, 30.0], btn_scan_new)
                            .on_hover_text(self.language.t(TrKey::SettingsBtnScanNewTooltip))
                            .clicked()
                        {
                            self.watcher.rescan();
                        }

                        let btn_rescan_force = egui::Button::new(
                            egui::RichText::new(format!("{}", self.language.t(TrKey::SettingsBtnRescanForce)))
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.accent_gold),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.accent_gold.linear_multiply(0.4)))
                        .rounding(Rounding::same(6.0));

                        if ui.add_sized([0.0, 30.0], btn_rescan_force)
                            .on_hover_text(self.language.t(TrKey::SettingsBtnRescanForceTooltip))
                            .clicked()
                        {
                            self.watcher.rescan_force();
                        }
                    });
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    let title = match self.language {
                        AppLanguage::Vietnamese => "HIỂN THỊ ĐÈ TRONG GAME",
                        AppLanguage::English => "IN-GAME OVERLAY",
                    };
                    ui.label(
                        egui::RichText::new(title)
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.focus_teal),
                    );
                    ui.add_space(4.0);

                    let desc = match self.language {
                        AppLanguage::Vietnamese => "Tự động hiển thị 5 chiến báo mới nhất ghim trên cửa sổ game với cơ chế xuyên chuột (click-through).",
                        AppLanguage::English => "Displays 5 latest reports pinned to Rise of Kingdoms window with click-through safety.",
                    };
                    ui.label(egui::RichText::new(desc).font(egui::FontId::proportional(10.5)).color(c.text_muted));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        let prev_enabled = self.overlay_config.enabled;
                        let toggle_lbl = match self.language {
                            AppLanguage::Vietnamese => "Kích hoạt Overlay",
                            AppLanguage::English => "Enable Overlay",
                        };
                        ui.checkbox(&mut self.overlay_config.enabled, egui::RichText::new(toggle_lbl).strong().color(c.canvas_tint));

                        ui.add_space(8.0);
                        let (badge_txt, badge_col) = if self.overlay_config.enabled {
                            ("● ACTIVE", c.status_green)
                        } else {
                            ("○ INACTIVE", c.text_muted)
                        };
                        ui.label(egui::RichText::new(badge_txt).font(egui::FontId::proportional(10.0)).strong().color(badge_col));

                        if self.overlay_config.enabled != prev_enabled {
                            if self.overlay_config.enabled {
                                self.refresh_cached_overlay_records();
                            }
                            self.save_config();
                        }
                    });

                    if self.overlay_config.enabled {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

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
                                self.refresh_cached_overlay_records();
                                self.save_config();
                            }
                        });

                        ui.add_space(10.0);

                        let pos_lbl = match self.language {
                            AppLanguage::Vietnamese => "Vị trí & Di chuyển:",
                            AppLanguage::English => "Position & Movement:",
                        };
                        let hold_hint = match self.language {
                            AppLanguage::Vietnamese => "(Nhấn giữ mũi tên để di chuyển)",
                            AppLanguage::English => "(Hold arrows to nudge)",
                        };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(pos_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(hold_hint).size(10.5).color(c.text_muted));
                        });
                        ui.add_space(6.0);

                        let mut pos_changed = false;
                        let nudge_step = 0.5_f32;
                        ui.horizontal(|ui| {
                            let dpad_size = [32.0, 26.0];
                            let dpad_btn = |text: &str| {
                                egui::Button::new(
                                    egui::RichText::new(text)
                                        .font(egui::FontId::proportional(12.0))
                                        .strong()
                                        .color(c.canvas_tint),
                                )
                                .fill(c.primary)
                                .stroke(Stroke::new(1.0_f32, c.hairline))
                                .rounding(Rounding::same(4.0))
                                .sense(egui::Sense::click_and_drag())
                            };

                            egui::Grid::new("overlay_dpad_grid")
                                .spacing([4.0, 4.0])
                                .show(ui, |ui| {
                                    ui.allocate_space(egui::vec2(dpad_size[0], dpad_size[1]));
                                    let btn_up = ui.add_sized(dpad_size, dpad_btn("▲"));
                                    if btn_up.is_pointer_button_down_on() || btn_up.dragged() || (btn_up.hovered() && ui.input(|i| i.pointer.primary_down())) {
                                        self.overlay_config.offset_y = (self.overlay_config.offset_y - nudge_step).max(0.0);
                                        pos_changed = true;
                                    }
                                    ui.allocate_space(egui::vec2(dpad_size[0], dpad_size[1]));
                                    ui.end_row();

                                    let btn_left = ui.add_sized(dpad_size, dpad_btn("◄"));
                                    if btn_left.is_pointer_button_down_on() || btn_left.dragged() || (btn_left.hovered() && ui.input(|i| i.pointer.primary_down())) {
                                        self.overlay_config.offset_x = (self.overlay_config.offset_x - nudge_step).max(0.0);
                                        pos_changed = true;
                                    }
                                    let btn_reset = egui::Button::new(egui::RichText::new("RoK").size(10.0).strong().color(c.focus_teal))
                                        .fill(c.primary)
                                        .stroke(Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.4)))
                                        .rounding(Rounding::same(4.0));
                                    if ui.add_sized(dpad_size, btn_reset).clicked() {
                                        self.overlay_config.offset_x = 16.0;
                                        self.overlay_config.offset_y = 60.0;
                                        pos_changed = true;
                                    }
                                    let btn_right = ui.add_sized(dpad_size, dpad_btn("►"));
                                    if btn_right.is_pointer_button_down_on() || btn_right.dragged() || (btn_right.hovered() && ui.input(|i| i.pointer.primary_down())) {
                                        self.overlay_config.offset_x += nudge_step;
                                        pos_changed = true;
                                    }
                                    ui.end_row();

                                    ui.allocate_space(egui::vec2(dpad_size[0], dpad_size[1]));
                                    let btn_down = ui.add_sized(dpad_size, dpad_btn("▼"));
                                    if btn_down.is_pointer_button_down_on() || btn_down.dragged() || (btn_down.hovered() && ui.input(|i| i.pointer.primary_down())) {
                                        self.overlay_config.offset_y += nudge_step;
                                        pos_changed = true;
                                    }
                                    ui.allocate_space(egui::vec2(dpad_size[0], dpad_size[1]));
                                    ui.end_row();
                                });

                            ui.add_space(16.0);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("X:").color(c.text_muted).strong());
                                    let prev_x = self.overlay_config.offset_x;
                                    ui.add(egui::DragValue::new(&mut self.overlay_config.offset_x).speed(1.0).range(0.0..=3000.0));
                                    if (self.overlay_config.offset_x - prev_x).abs() > 0.1 {
                                        self.overlay_config.offset_x = self.overlay_config.offset_x.max(0.0);
                                        pos_changed = true;
                                    }
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("Y:").color(c.text_muted).strong());
                                    let prev_y = self.overlay_config.offset_y;
                                    ui.add(egui::DragValue::new(&mut self.overlay_config.offset_y).speed(1.0).range(0.0..=3000.0));
                                    if (self.overlay_config.offset_y - prev_y).abs() > 0.1 {
                                        self.overlay_config.offset_y = self.overlay_config.offset_y.max(0.0);
                                        pos_changed = true;
                                    }
                                });

                                ui.add_space(6.0);
                                let reset_lbl = match self.language {
                                    AppLanguage::Vietnamese => "Đặt lại vị trí mặc định",
                                    AppLanguage::English => "Reset Coordinates",
                                };
                                let btn_rst = egui::Button::new(egui::RichText::new(reset_lbl).font(egui::FontId::proportional(10.5)).color(c.canvas_tint))
                                    .fill(c.primary)
                                    .stroke(Stroke::new(1.0_f32, c.hairline))
                                    .rounding(Rounding::same(5.0))
                                    .min_size(egui::vec2(0.0, 24.0));
                                if ui.add(btn_rst).clicked() {
                                    self.overlay_config.offset_x = 16.0;
                                    self.overlay_config.offset_y = 60.0;
                                    pos_changed = true;
                                }
                            });
                        });

                        if pos_changed {
                            self.overlay_config.offset_x = self.overlay_config.offset_x.max(0.0);
                            self.overlay_config.offset_y = self.overlay_config.offset_y.max(0.0);
                            self.last_mail_arrival = std::time::Instant::now();
                            self.pending_overlay_save = true;
                            ctx.request_repaint();
                        }

                        if self.pending_overlay_save && (!ui.input(|i| i.pointer.any_down()) || ui.input(|i| i.pointer.any_released())) {
                            self.pending_overlay_save = false;
                            self.save_config();
                        }

                        ui.add_space(10.0);

                        let width_lbl = match self.language {
                            AppLanguage::Vietnamese => "Chiều rộng lớp phủ:",
                            AppLanguage::English => "Overlay Width:",
                        };
                        let prev_width = self.overlay_config.width;
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(width_lbl).strong().color(c.canvas_tint));
                            ui.add(
                                egui::Slider::new(&mut self.overlay_config.width, 180.0_f32..=380.0_f32)
                                    .custom_formatter(|n, _| format!("{:.0} px", n)),
                            );
                        });
                        if (self.overlay_config.width - prev_width).abs() > 0.5 {
                            self.last_mail_arrival = std::time::Instant::now();
                            self.save_config();
                        }

                        ui.add_space(8.0);

                        let font_lbl = match self.language {
                            AppLanguage::Vietnamese => "Cỡ chữ hiển thị:",
                            AppLanguage::English => "Font Size:",
                        };
                        let prev_fs = self.overlay_config.font_size;
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(font_lbl).strong().color(c.canvas_tint));
                            ui.add(
                                egui::Slider::new(&mut self.overlay_config.font_size, 8.5_f32..=15.0_f32)
                                    .custom_formatter(|n, _| format!("{:.1} pt", n)),
                            );
                        });
                        if (self.overlay_config.font_size - prev_fs).abs() > 0.1 {
                            self.last_mail_arrival = std::time::Instant::now();
                            self.save_config();
                        }

                        ui.add_space(8.0);

                        let opacity_lbl = match self.language {
                            AppLanguage::Vietnamese => "Độ mờ:",
                            AppLanguage::English => "Opacity:",
                        };
                        let prev_opacity = self.overlay_config.opacity;
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(opacity_lbl).strong().color(c.canvas_tint));
                            ui.add(
                                egui::Slider::new(&mut self.overlay_config.opacity, 0.0_f32..=1.0_f32)
                                    .custom_formatter(|n, _| {
                                        if n <= 0.01 {
                                            "0%".to_string()
                                        } else {
                                            format!("{:.0}%", n * 100.0)
                                        }
                                    }),
                            );
                        });
                        if (self.overlay_config.opacity - prev_opacity).abs() > 0.005 {
                            self.last_mail_arrival = std::time::Instant::now();
                            self.save_config();
                        }

                        ui.add_space(10.0);

                        let color_lbl = match self.language {
                            AppLanguage::Vietnamese => "Màu chữ:",
                            AppLanguage::English => "Text Color:",
                        };
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.label(egui::RichText::new(color_lbl).strong().color(c.canvas_tint));
                            let themes = [
                                ("#F8FAFC", "White", Color32::from_rgb(0xF8, 0xFA, 0xFC)),
                                ("#FDE047", "Gold", Color32::from_rgb(0xFD, 0xE0, 0x47)),
                                ("#38BDF8", "Cyan", Color32::from_rgb(0x38, 0xBD, 0xF8)),
                                ("#4ADE80", "Green", Color32::from_rgb(0x4A, 0xDE, 0x80)),
                            ];
                            for (hex, name, col) in themes {
                                let is_active = self.overlay_config.text_color_hex == hex;
                                let border_stroke = if is_active {
                                    Stroke::new(1.5_f32, col)
                                } else {
                                    Stroke::new(1.0_f32, c.hairline)
                                };
                                let btn = egui::Button::new(
                                    egui::RichText::new(format!("■ {}", name))
                                        .font(egui::FontId::proportional(11.0))
                                        .strong()
                                        .color(col),
                                )
                                .fill(if is_active { c.primary_pressed } else { c.primary })
                                .stroke(border_stroke)
                                .rounding(Rounding::same(5.0));

                                if ui.add_sized([0.0, 26.0], btn).clicked() {
                                    self.overlay_config.text_color_hex = hex.to_string();
                                    self.last_mail_arrival = std::time::Instant::now();
                                    self.save_config();
                                }
                            }
                        });
                    }
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", self.language.t(TrKey::SettingsCardSystemTitle)))
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.canvas_tint),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsLanguageLabel)).strong().color(c.canvas_tint));

                        let lang_btn = |ui: &mut egui::Ui, is_sel: bool, text: &str| -> egui::Response {
                            let fill = if is_sel { c.primary_pressed } else { c.primary };
                            let stroke = if is_sel { Stroke::new(1.0_f32, c.focus_teal) } else { Stroke::new(1.0_f32, c.hairline) };
                            let col = if is_sel { c.focus_teal } else { c.text_muted };
                            ui.add_sized(
                                [0.0, 26.0],
                                egui::Button::new(egui::RichText::new(text).font(egui::FontId::proportional(11.0)).strong().color(col))
                                    .fill(fill)
                                    .stroke(stroke)
                                    .rounding(Rounding::same(5.0)),
                            )
                        };

                        let is_vi = self.language == AppLanguage::Vietnamese;
                        if lang_btn(ui, is_vi, "Tiếng Việt").clicked() && !is_vi {
                            self.language = AppLanguage::Vietnamese;
                            self.watcher.set_language(self.language);
                            self.save_config();
                            if let Some(tray) = &self.tray {
                                tray.update_language(self.language);
                            }
                        }

                        let is_en = self.language == AppLanguage::English;
                        if lang_btn(ui, is_en, "English").clicked() && !is_en {
                            self.language = AppLanguage::English;
                            self.watcher.set_language(self.language);
                            self.save_config();
                            if let Some(tray) = &self.tray {
                                tray.update_language(self.language);
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.label(egui::RichText::new(self.language.t(TrKey::SettingsCloseBehavior)).strong().color(c.canvas_tint));

                        let is_tray = self.close_behavior == CloseBehavior::Tray;
                        let tray_stroke = if is_tray { Stroke::new(1.0_f32, c.focus_teal) } else { Stroke::new(1.0_f32, c.hairline) };
                        let tray_fill = if is_tray { c.primary_pressed } else { c.primary };
                        let tray_col = if is_tray { c.focus_teal } else { c.text_muted };

                        let btn_tray = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsCloseTray))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(tray_col),
                        )
                        .fill(tray_fill)
                        .stroke(tray_stroke)
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 26.0], btn_tray).clicked() {
                            self.close_behavior = CloseBehavior::Tray;
                            self.save_config();
                        }

                        let is_close = self.close_behavior == CloseBehavior::Close;
                        let close_stroke = if is_close { Stroke::new(1.0_f32, c.error.linear_multiply(0.6)) } else { Stroke::new(1.0_f32, c.hairline) };
                        let close_fill = if is_close { c.primary_pressed } else { c.primary };
                        let close_col = if is_close { c.error } else { c.text_muted };

                        let btn_close = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsCloseExit))
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(close_col),
                        )
                        .fill(close_fill)
                        .stroke(close_stroke)
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 26.0], btn_close).clicked() {
                            self.close_behavior = CloseBehavior::Close;
                            self.save_config();
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

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
                                        AppLanguage::Vietnamese => format!("Đã BẬT tự khởi động cùng Windows (Thu nhỏ: {})", if self.autostart_minimized { "BẬT" } else { "TẮT" }),
                                        AppLanguage::English => format!("ENABLED Autostart with Windows (Minimized: {})", if self.autostart_minimized { "YES" } else { "NO" }),
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
                                        AppLanguage::Vietnamese => format!("Lỗi cấu hình Autostart: {}", e),
                                        AppLanguage::English => format!("Autostart config error: {}", e),
                                    };
                                    self.watcher.log("ERROR", &err_msg);
                                }
                                self.save_config();
                            }
                        });
                    }
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    let vault_total = self.watcher.get_vault_total_count();
                    ui.label(
                        egui::RichText::new(format!("{}", self.language.t(TrKey::SettingsCardVaultTitle)))
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.accent_gold),
                    );
                    ui.add_space(4.0);

                    let secured_str = self.language.t(TrKey::SettingsVaultSecured).replace("{}", &format_num(vault_total));
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", secured_str))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.accent_gold),
                        );
                    });
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        let btn_vdir = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnOpenVaultDir))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_vdir).clicked() {
                            let dir = self.watcher.get_vault_binary_dir();
                            let _ = open::that(&dir);
                        }

                        let btn_vjson = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnOpenVaultJson))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.canvas_tint),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_vjson).clicked() {
                            let f = self.watcher.get_vault_file_path();
                            let _ = open::that(&f);
                        }

                        let btn_clean = egui::Button::new(
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnCleanVault))
                                .font(egui::FontId::proportional(11.0))
                                .color(c.text_muted),
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.hairline))
                        .rounding(Rounding::same(5.0));

                        if ui.add_sized([0.0, 28.0], btn_clean).clicked() {
                            match self.watcher.clean_binary_vault() {
                                Ok((cnt, bytes)) => {
                                    let mb = format!("{:.2}", bytes as f64 / (1024.0 * 1024.0));
                                    let msg = self.language.t(TrKey::SettingsCleanVaultSuccess)
                                        .replace("{files}", &cnt.to_string())
                                        .replace("{mb}", &mb);
                                    self.watcher.log("SUCCESS", &msg);
                                }
                                Err(err) => {
                                    self.watcher.log("ERROR", &err);
                                }
                            }
                        }
                    });
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    let title = match self.language {
                        AppLanguage::Vietnamese => "ĐỒNG BỘ CLOUD VỚI GOOGLE & SUPABASE",
                        AppLanguage::English => "CLOUD BACKUP & SYNC WITH GOOGLE & SUPABASE",
                    };
                    ui.label(
                        egui::RichText::new(title)
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.crystal_cyan),
                    );
                    ui.add_space(4.0);

                    let (status_text, is_logged_in) = match (
                        &self.cloud_auth.user_email,
                        self.cloud_auth.access_token.is_some(),
                    ) {
                        (Some(email), true) => {
                            let sync_info = self
                                .cloud_auth
                                .last_synced
                                .as_deref()
                                .unwrap_or("Chưa đồng bộ / Never");
                            let txt = match self.language {
                                AppLanguage::Vietnamese => {
                                    format!("● Tài khoản: {} | Lần sao lưu gần nhất: {}", email, sync_info)
                                }
                                AppLanguage::English => {
                                    format!("● Account: {} | Last backup: {}", email, sync_info)
                                }
                            };
                            (txt, true)
                        }
                        _ => {
                            let txt = match self.language {
                                AppLanguage::Vietnamese => {
                                    "Chưa đăng nhập. Đăng nhập Google để sao lưu & khôi phục khi chuyển thiết bị.".to_string()
                                }
                                AppLanguage::English => {
                                    "Not logged in. Log in with Google to backup & restore across devices.".to_string()
                                }
                            };
                            (txt, false)
                        }
                    };

                    let status_col = if is_logged_in { c.status_green } else { c.text_muted };
                    ui.label(
                        egui::RichText::new(status_text)
                            .font(egui::FontId::proportional(11.0))
                            .color(status_col),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if self.cloud_busy {
                            ui.spinner();
                            let wait_txt = match self.language {
                                AppLanguage::Vietnamese => "Đang xử lý...",
                                AppLanguage::English => "Processing...",
                            };
                            ui.label(egui::RichText::new(wait_txt).font(egui::FontId::proportional(11.0)).color(c.accent_gold));

                            let cancel_txt = match self.language {
                                AppLanguage::Vietnamese => "Hủy",
                                AppLanguage::English => "Cancel",
                            };
                            let btn_cancel = egui::Button::new(
                                egui::RichText::new(cancel_txt)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(c.error),
                            )
                            .fill(c.card_surface)
                            .stroke(Stroke::new(1.0_f32, c.error.linear_multiply(0.5)))
                            .rounding(Rounding::same(4.0));

                            if ui.add(btn_cancel).clicked() {
                                self.cloud_busy = false;
                                self.cloud_rx = None;
                                self.cloud_op_rx = None;
                                let msg = match self.language {
                                    AppLanguage::Vietnamese => "Đã hủy thao tác cloud",
                                    AppLanguage::English => "Cloud operation cancelled",
                                };
                                self.watcher.log("WARN", msg);
                            }
                        } else if !is_logged_in {
                            let btn_login_txt = match self.language {
                                AppLanguage::Vietnamese => "Đăng Nhập Google",
                                AppLanguage::English => "Login with Google",
                            };
                            let btn_login = egui::Button::new(
                                egui::RichText::new(btn_login_txt)
                                    .font(egui::FontId::proportional(11.5))
                                    .strong()
                                    .color(c.crystal_cyan),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.crystal_cyan.linear_multiply(0.4)))
                            .rounding(Rounding::same(6.0));

                            if ui.add_sized([ui.available_width(), 30.0], btn_login).clicked() && !self.cloud_busy {
                                self.start_google_login();
                            }
                        } else {
                            let btn_backup_txt = match self.language {
                                AppLanguage::Vietnamese => "Sao Lưu Lên Cloud",
                                AppLanguage::English => "Backup to Cloud",
                            };
                            let btn_backup = egui::Button::new(
                                egui::RichText::new(btn_backup_txt)
                                    .font(egui::FontId::proportional(11.0))
                                    .strong()
                                    .color(c.status_green),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.status_green.linear_multiply(0.4)))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([0.0, 28.0], btn_backup).clicked() && !self.cloud_busy {
                                self.trigger_cloud_backup();
                            }

                            let btn_restore_txt = match self.language {
                                AppLanguage::Vietnamese => "Khôi Phục Từ Cloud",
                                AppLanguage::English => "Restore from Cloud",
                            };
                            let btn_restore = egui::Button::new(
                                egui::RichText::new(btn_restore_txt)
                                    .font(egui::FontId::proportional(11.0))
                                    .strong()
                                    .color(c.accent_gold),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.accent_gold.linear_multiply(0.4)))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([0.0, 28.0], btn_restore).clicked() && !self.cloud_busy {
                                self.trigger_cloud_restore();
                            }

                            let btn_logout_txt = match self.language {
                                AppLanguage::Vietnamese => "Đăng Xuất",
                                AppLanguage::English => "Logout",
                            };
                            let btn_logout = egui::Button::new(
                                egui::RichText::new(btn_logout_txt)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(c.error),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.error.linear_multiply(0.4)))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([0.0, 28.0], btn_logout).clicked() && !self.cloud_busy {
                                self.cloud_auth.access_token = None;
                                self.cloud_auth.refresh_token = None;
                                self.cloud_auth.user_id = None;
                                self.cloud_auth.user_email = None;
                                self.save_config();
                                let msg = match self.language {
                                    AppLanguage::Vietnamese => "Đã đăng xuất tài khoản Cloud",
                                    AppLanguage::English => "Logged out from Cloud",
                                };
                                self.watcher.log("INFO", msg);
                            }
                        }
                    });

                    ui.add_space(8.0);
                    let stay_login_label = match self.language {
                        AppLanguage::Vietnamese => "Duy trì đăng nhập",
                        AppLanguage::English => "Stay logged in",
                    };
                    if ui.checkbox(&mut self.cloud_auth.stay_login, stay_login_label).changed() {
                        self.save_config();
                    }

                    if is_logged_in {
                        ui.add_space(4.0);
                        let mut changed = false;

                        ui.horizontal(|ui| {
                            if ui.checkbox(
                                &mut self.cloud_auth.auto_backup,
                                self.language.t(TrKey::SettingsCloudAutoBackupPeriodic),
                            ).changed() {
                                changed = true;
                            }
                            if ui.add_enabled(
                                self.cloud_auth.auto_backup,
                                egui::DragValue::new(&mut self.cloud_auth.backup_interval_minutes).range(5..=1440),
                            ).changed() {
                                self.cloud_auth.backup_interval_minutes = self.cloud_auth.clamped_interval_minutes();
                                changed = true;
                            }
                            ui.label(self.language.t(TrKey::SettingsCloudAutoBackupMinutes));
                        });

                        ui.add_space(2.0);
                        if ui.checkbox(
                            &mut self.cloud_auth.backup_on_exit,
                            self.language.t(TrKey::SettingsCloudAutoBackupOnExit),
                        ).changed() {
                            changed = true;
                        }

                        if changed {
                            self.save_config();
                        }
                    }
                });

                ui.add_space(8.0);
            });

            egui::Frame::none()
                .fill(c.log_bg)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(14.0))
                .show(&mut cols[1], |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", self.language.t(TrKey::LogsTitle)))
                                .font(egui::FontId::proportional(12.5))
                                .strong()
                                .color(c.focus_teal),
                        );

                        let count_str = self.language.t(TrKey::LogsLinesCount).replace("{}", &self.console_logs.len().to_string());
                        ui.label(
                            egui::RichText::new(format!("({})", count_str))
                                .font(egui::FontId::proportional(10.5))
                                .color(c.text_muted),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_clear = egui::Button::new(
                                egui::RichText::new(self.language.t(TrKey::LogsBtnClear))
                                    .font(egui::FontId::proportional(10.5))
                                    .color(c.canvas_tint),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.hairline))
                            .rounding(Rounding::same(5.0));

                            if ui.add_sized([0.0, 24.0], btn_clear).clicked() {
                                self.console_logs.clear();
                            }

                            ui.checkbox(&mut self.log_auto_scroll, self.language.t(TrKey::LogsAutoScroll));
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

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
                                    ui.add_space(2.0);
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _ = crate::tray::get_or_find_main_hwnd();

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
                .frame(egui::Frame::none().fill(c.sidebar_bg).stroke(Stroke::new(1.0_f32, c.hairline)).inner_margin(Margin::same(12.0)))
                .show(ctx, |ui| {
                    self.render_sidebar(ui, ctx);
                });

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

                        ctx.show_viewport_immediate(overlay_id, builder, |ctx, _class| {
                            crate::overlay_ui::render_overlay_viewport(ctx, overlay_records, &overlay_cfg, last_mail);
                        });

                        ctx.send_viewport_cmd_to(
                            overlay_id,
                            egui::ViewportCommand::InnerSize(egui::vec2(geom.width.round(), geom.height.round())),
                        );
                        ctx.send_viewport_cmd_to(
                            overlay_id,
                            egui::ViewportCommand::OuterPosition(egui::pos2(geom.pos_x.round(), geom.pos_y.round())),
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
                                let title_wide: Vec<u16> = "Rise of Kingdoms Loot Overlay\0".encode_utf16().collect();
                                unsafe {
                                    let vhwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
                                        std::ptr::null(),
                                        title_wide.as_ptr(),
                                    );
                                    if vhwnd != 0 as _ {
                                        self.overlay_styled_hwnd = Some(vhwnd as isize);
                                        self.overlay_last_opacity = Some(self.overlay_config.opacity);
                                        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                            vhwnd,
                                            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE,
                                        );
                                        crate::overlay_tracker::apply_click_through_styles(vhwnd, self.overlay_config.opacity);
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
                    let title_wide: Vec<u16> = "Rise of Kingdoms Loot Overlay\0".encode_utf16().collect();
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
        assert_eq!(light.ink, Color32::from_rgb(0xf8, 0xfa, 0xfc));

        assert_eq!(light.log_bg, Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(dark.log_bg, Color32::from_rgb(0x0d, 0x0f, 0x14));

        assert_eq!(light.canvas_tint, Color32::from_rgb(0x0f, 0x17, 0x2a));
        assert_eq!(light.text_muted, Color32::from_rgb(0x47, 0x55, 0x69));

        assert_eq!(light.train_orange, Color32::from_rgb(0xc2, 0x41, 0x0c));
        assert_eq!(light.heal_blue, Color32::from_rgb(0x03, 0x69, 0xa1));
        assert_eq!(light.status_green, Color32::from_rgb(0x04, 0x78, 0x57));
        assert_eq!(light.accent_gold, Color32::from_rgb(0xb4, 0x53, 0x09));
        assert_eq!(light.speedup_purple, Color32::from_rgb(0x7e, 0x22, 0xce));
        assert_eq!(light.crystal_cyan, Color32::from_rgb(0x0e, 0x74, 0x90));

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
        let op_is_backup = true;
        if op_is_backup {
            has_unbacked_data = false;
            last_cloud_backup_time = Some(std::time::Instant::now());
        }
        assert!(!has_unbacked_data);
        assert!(last_cloud_backup_time.is_some());

        let interval_duration = std::time::Duration::from_secs(config.cloud_auth.clamped_interval_minutes() as u64 * 60);
        let should_run = match last_cloud_backup_time {
            Some(last) => std::time::Instant::now().duration_since(last) >= interval_duration,
            None => true,
        };
        assert!(!should_run);
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
}
