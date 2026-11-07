use eframe::egui::{self, Color32};

use crate::i18n::TrKey;
use crate::model::AppLanguage;
use crate::watcher::WatcherStatus;
use super::theme::ThemeColors;
use super::{BattleSortColumn, ItemSortColumn, SortDirection};

#[allow(clippy::too_many_arguments)]
pub fn render_sort_header_cell(
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
pub fn render_item_sort_header_cell(
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

#[derive(Debug, PartialEq)]
pub struct ReportTableDimensions {
    pub table_width: f32,
    pub target_width: f32,
    pub extra_per_column: f32,
}

pub fn report_table_dimensions(
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

pub fn report_non_target_column_count(is_gathering: bool, is_barb: bool) -> usize {
    if is_gathering {
        7
    } else if is_barb {
        11
    } else {
        10
    }
}

pub(crate) fn is_network_offline_error(err: Option<&str>) -> bool {
    let Some(e) = err else {
        return true;
    };
    let lower = e.to_ascii_lowercase();
    if lower.contains("http")
        || lower.contains("403")
        || lower.contains("401")
        || lower.contains("404")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("permission")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
    {
        return false;
    }
    lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("dns")
        || lower.contains("connection refused")
        || lower.contains("network")
        || lower.contains("unreachable")
        || lower.contains("offline")
        || lower.contains("mất kết nối")
}
