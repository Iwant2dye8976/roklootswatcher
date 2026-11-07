use std::path::PathBuf;
use eframe::egui::{self, Color32};

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
    pub speedup_blue: Color32,
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
            ink: Color32::from_rgb(0x0b, 0x0f, 0x17),
            ink_deep: Color32::from_rgb(0x13, 0x1b, 0x29),
            card_surface: Color32::from_rgb(0x19, 0x22, 0x31),
            sidebar_bg: Color32::from_rgb(0x10, 0x16, 0x22),
            primary: Color32::from_rgb(0x24, 0x30, 0x44),
            primary_pressed: Color32::from_rgb(0x31, 0x41, 0x5c),
            brand_mark: Color32::from_rgb(0xfb, 0x71, 0x85),
            canvas_tint: Color32::from_rgb(0xf8, 0xfa, 0xfc),
            hairline: Color32::from_rgb(0x2e, 0x3d, 0x52),
            field_border: Color32::from_rgb(0x43, 0x56, 0x70),
            accent_gold: Color32::from_rgb(0xfb, 0xbf, 0x24),
            accent_gold_soft: Color32::from_rgb(0xfd, 0xe0, 0x47),
            focus_teal: Color32::from_rgb(0x22, 0xd3, 0xee),
            status_green: Color32::from_rgb(0x34, 0xd3, 0x99),
            text_muted: Color32::from_rgb(0xb0, 0xc0, 0xd4),
            error: Color32::from_rgb(0xf8, 0x71, 0x71),
            motte_accent: Color32::from_rgb(0xfb, 0x71, 0x85),
            fort_accent: Color32::from_rgb(0xfb, 0xbf, 0x24),
            barb_accent: Color32::from_rgb(0x34, 0xd3, 0x99),
            crystal_cyan: Color32::from_rgb(0x38, 0xbd, 0xf8),
            speedup_blue: Color32::from_rgb(0x60, 0xa5, 0xfa),
            train_orange: Color32::from_rgb(0xfb, 0x92, 0x3c),
            heal_blue: Color32::from_rgb(0x38, 0xbd, 0xf8),
            log_bg: Color32::from_rgb(0x08, 0x0b, 0x11),
            mat_iron: Color32::from_rgb(0xcb, 0xd5, 0xe1),
            mat_ebony: Color32::from_rgb(0x34, 0xd3, 0x99),
            mat_bone: Color32::from_rgb(0xf4, 0x72, 0xb6),
            rank_silver: Color32::from_rgb(0xe2, 0xe8, 0xf0),
            rank_bronze: Color32::from_rgb(0xf5, 0x9e, 0x0b),
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            ink: Color32::from_rgb(0xe6, 0xec, 0xf5),
            ink_deep: Color32::from_rgb(0xee, 0xf2, 0xf8),
            card_surface: Color32::from_rgb(0xff, 0xff, 0xff),
            sidebar_bg: Color32::from_rgb(0xdc, 0xe5, 0xf0),
            primary: Color32::from_rgb(0xe2, 0xe8, 0xf0),
            primary_pressed: Color32::from_rgb(0xcb, 0xd5, 0xe1),
            brand_mark: Color32::from_rgb(0xbe, 0x12, 0x3c),
            canvas_tint: Color32::from_rgb(0x0f, 0x17, 0x2a),
            hairline: Color32::from_rgb(0xc3, 0xcf, 0xe0),
            field_border: Color32::from_rgb(0x94, 0xa3, 0xb8),
            accent_gold: Color32::from_rgb(0xb4, 0x53, 0x09),
            accent_gold_soft: Color32::from_rgb(0x92, 0x40, 0x0e),
            focus_teal: Color32::from_rgb(0x03, 0x69, 0xa1),
            status_green: Color32::from_rgb(0x04, 0x78, 0x57),
            text_muted: Color32::from_rgb(0x33, 0x41, 0x55),
            error: Color32::from_rgb(0xb9, 0x1c, 0x1c),
            motte_accent: Color32::from_rgb(0xbe, 0x12, 0x3c),
            fort_accent: Color32::from_rgb(0xb4, 0x53, 0x09),
            barb_accent: Color32::from_rgb(0x04, 0x78, 0x57),
            crystal_cyan: Color32::from_rgb(0x0e, 0x74, 0x90),
            speedup_blue: Color32::from_rgb(0x1d, 0x4e, 0xd8),
            train_orange: Color32::from_rgb(0xc2, 0x41, 0x0c),
            heal_blue: Color32::from_rgb(0x03, 0x69, 0xa1),
            log_bg: Color32::from_rgb(0xf8, 0xfa, 0xfc),
            mat_iron: Color32::from_rgb(0x33, 0x41, 0x55),
            mat_ebony: Color32::from_rgb(0x04, 0x78, 0x57),
            mat_bone: Color32::from_rgb(0xbe, 0x18, 0x5d),
            rank_silver: Color32::from_rgb(0x33, 0x41, 0x55),
            rank_bronze: Color32::from_rgb(0x9a, 0x34, 0x12),
        }
    }
}

pub fn category_badge_color(category: &str, c: &ThemeColors) -> Color32 {
    let lower = category.to_lowercase();
    if lower.contains("nguyên") || lower.contains("resource") {
        c.accent_gold
    } else if lower.contains("vật liệu") || lower.contains("material") {
        c.heal_blue
    } else if lower.contains("tốc") || lower.contains("speed") {
        c.speedup_blue
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
