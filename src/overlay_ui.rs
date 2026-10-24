use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke};
use crate::model::{parse_hex_color, BattleRecord, FilterMode, OverlayConfig};

#[derive(Debug, Clone)]
pub struct CardDisplayData {
    pub tag: String,
    pub target_display: String,
    pub time_display: String,
    pub meta_display: String,
    pub drops_summary: Vec<String>,
    pub accent_color: Color32,
}

pub fn accent_color_for_battle_type(btype: &str) -> Color32 {
    match btype {
        "fort" => Color32::from_rgb(245, 158, 11),       // Gold / Amber
        "motte" => Color32::from_rgb(239, 68, 68),       // Crimson
        "barbarian" => Color32::from_rgb(59, 130, 246),  // Steel Blue
        "gathering" => Color32::from_rgb(16, 185, 129),  // Emerald
        _ => Color32::from_rgb(148, 163, 184),           // Slate
    }
}

pub fn format_card_data(rec: &BattleRecord) -> CardDisplayData {
    let tag = match rec.battle_type.as_str() {
        "fort" => "[Fort]".to_string(),
        "motte" => "[Motte]".to_string(),
        "barbarian" => "[Barbarian]".to_string(),
        "gathering" => "[Gathering]".to_string(),
        _ => "[Reward]".to_string(),
    };

    let accent_color = accent_color_for_battle_type(&rec.battle_type);

    let target_display = if !rec.target_level.is_empty() && !rec.target.contains(&rec.target_level) {
        format!("{} Lvl {}", rec.target, rec.target_level)
    } else {
        rec.target.clone()
    };

    let raw_time = if !rec.time_vn.is_empty() {
        &rec.time_vn
    } else if !rec.time_utc.is_empty() {
        &rec.time_utc
    } else {
        ""
    };
    let time_display = if let Some((_, t)) = raw_time.split_once(' ') {
        t.chars().take(5).collect()
    } else if raw_time.len() > 5 && raw_time.contains(':') {
        raw_time.chars().take(5).collect()
    } else if !raw_time.is_empty() {
        raw_time.to_string()
    } else {
        format!("{}", rec.time_raw)
    };

    let meta_display = if rec.battle_type == "gathering" {
        format!("Coord: ({}, {}) | Complete", rec.pos_x, rec.pos_y)
    } else {
        format!("Coord: ({}, {}) | {}% Damage", rec.pos_x, rec.pos_y, rec.damage_pct)
    };

    let mut drops_summary = Vec::new();
    if !rec.item_totals.is_empty() {
        let mut items: Vec<(&String, &i64)> = rec.item_totals.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));

        let top_parts: Vec<String> = items
            .iter()
            .take(2)
            .map(|(name, count)| format!("{}: {}", name, crate::model::format_num(*count)))
            .collect();

        if !top_parts.is_empty() {
            drops_summary.push(format!("• {}", top_parts.join(" • ")));
        }
    }

    if drops_summary.is_empty() {
        drops_summary.push("• (No items)".to_string());
    }

    CardDisplayData {
        tag,
        target_display,
        time_display,
        meta_display,
        drops_summary,
        accent_color,
    }
}

pub fn compute_idle_dim_factor(last_arrival: std::time::Instant) -> f32 {
    let elapsed = last_arrival.elapsed().as_secs_f32();
    if elapsed <= 20.0 {
        1.0
    } else if elapsed >= 23.0 {
        0.65
    } else {
        // Linear transition from 1.0 down to 0.65 over 3 seconds
        let progress = (elapsed - 20.0) / 3.0;
        1.0 - (progress * 0.35)
    }
}

pub fn get_latest_filtered_records(records: &[BattleRecord], filter: FilterMode, limit: usize) -> Vec<BattleRecord> {
    let mut filtered: Vec<BattleRecord> = records
        .iter()
        .filter(|r| filter.matches(&r.battle_type))
        .cloned()
        .collect();

    filtered.sort_by_key(|a| std::cmp::Reverse(a.time_raw));
    filtered.truncate(limit);
    filtered
}

pub fn compute_overlay_card_style(
    opacity: f32,
    dim_factor: f32,
) -> (Color32, Stroke, f32) {
    let text_alpha = dim_factor.clamp(0.40, 1.0);
    if opacity <= 0.01 {
        return (Color32::TRANSPARENT, Stroke::NONE, text_alpha);
    }

    let bg_alpha = (opacity * dim_factor).clamp(0.0, 1.0);
    let bg_color = Color32::from_rgba_unmultiplied(15, 23, 42, (bg_alpha * 240.0) as u8);
    let card_stroke = Stroke::new(
        1.0_f32,
        Color32::from_rgba_unmultiplied(51, 65, 85, (bg_alpha * 255.0) as u8),
    );

    (bg_color, card_stroke, text_alpha)
}

pub fn render_overlay_viewport(
    ctx: &egui::Context,
    records: &[BattleRecord],
    config: &OverlayConfig,
    last_mail_time: std::time::Instant,
) {
    let dim_factor = compute_idle_dim_factor(last_mail_time);
    let (bg_color, card_stroke, text_alpha) = compute_overlay_card_style(config.opacity, dim_factor);

    let raw_text_color = parse_hex_color(&config.text_color_hex)
        .unwrap_or_else(|| Color32::from_rgb(248, 250, 252));
    let base_text_color = if raw_text_color.r() == 0 && raw_text_color.g() == 0 && raw_text_color.b() == 0 {
        Color32::from_rgb(24, 24, 24)
    } else {
        raw_text_color
    };
    let text_color = base_text_color.linear_multiply(text_alpha);
    let muted_text = base_text_color.linear_multiply(0.70 * text_alpha);
    let accent_alpha = text_alpha;

    let fs_title = config.font_size;
    let fs_sub = (config.font_size * 0.88).max(8.0);
    let fs_time = (config.font_size * 0.82).max(7.5);
    let pad_v = (config.font_size * 0.3).clamp(2.0, 6.0);
    let pad_h = (config.font_size * 0.7).clamp(6.0, 12.0);

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 3.0);

            if records.is_empty() {
                Frame::none()
                    .fill(bg_color)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("[RoK Overlay - No matching reports]")
                                .color(muted_text)
                                .size(fs_sub),
                        );
                    });
                return;
            }

            for rec in records.iter().take(5) {
                let card = format_card_data(rec);

                let card_resp = Frame::none()
                    .fill(bg_color)
                    .rounding(Rounding::same(4.0))
                    .stroke(card_stroke)
                    .inner_margin(Margin {
                        left: pad_h,
                        right: 6.0,
                        top: pad_v,
                        bottom: pad_v,
                    })
                    .show(ui, |ui| {
                        // Line 1: [TAG] Target (left) ... Time (right)
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                            ui.label(
                                egui::RichText::new(&card.tag)
                                    .color(card.accent_color.linear_multiply(accent_alpha))
                                    .size(fs_sub)
                                    .strong(),
                            );
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&card.target_display)
                                        .color(text_color)
                                        .size(fs_title)
                                        .strong(),
                                )
                                .truncate(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(&card.time_display)
                                        .color(muted_text)
                                        .size(fs_time),
                                );
                            });
                        });

                        // Line 2: Coordinates, % Damage, and top loot
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(3.0, 0.0);
                            ui.label(
                                egui::RichText::new(format!("({}, {})", rec.pos_x, rec.pos_y))
                                    .color(muted_text)
                                    .size(fs_sub),
                            );
                            if rec.battle_type != "gathering" {
                                ui.label(
                                    egui::RichText::new(format!("{}%", rec.damage_pct))
                                        .color(muted_text)
                                        .size(fs_sub),
                                );
                            }
                            if let Some(drop) = card.drops_summary.first() {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(drop)
                                            .color(text_color)
                                            .size(fs_sub),
                                    )
                                    .truncate(),
                                );
                            }
                        });
                    });

                // Vertical 3px left accent bar matching battle type
                let r = card_resp.response.rect;
                let accent_rect = egui::Rect::from_min_max(
                    egui::pos2(r.min.x, r.min.y),
                    egui::pos2(r.min.x + 3.0, r.max.y),
                );
                ui.painter().rect_filled(
                    accent_rect,
                    Rounding {
                        nw: 4.0,
                        sw: 4.0,
                        ne: 0.0,
                        se: 0.0,
                    },
                    card.accent_color.linear_multiply(accent_alpha),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_emojis_in_card_formatting() {
        let mut item_totals = std::collections::HashMap::new();
        item_totals.insert("Sách ước nguyện".to_string(), 2);
        item_totals.insert("Đá quý".to_string(), 100);

        let rec = BattleRecord {
            filename: "test.mail".into(),
            mail_id: "1".into(),
            battle_type: "fort".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "13:42".into(),
            target: "Pháo Đài Cấp 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 524,
            pos_y: 681,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals,
            content: String::new(),
        };

        let card = format_card_data(&rec);
        assert_eq!(card.tag, "[Fort]");
        assert!(card.target_display.contains("Pháo Đài Cấp 10"));
        assert_eq!(card.meta_display, "Coord: (524, 681) | 100% Damage");
        
        let check_no_emoji = |s: &str| {
            for ch in s.chars() {
                let u = ch as u32;
                let is_emoji = (0x1F600..=0x1F64F).contains(&u)
                    || (0x1F300..=0x1F5FF).contains(&u)
                    || (0x1F680..=0x1F6FF).contains(&u)
                    || (0x2600..=0x26FF).contains(&u)
                    || (0x2700..=0x27BF).contains(&u)
                    || (0x1F900..=0x1F9FF).contains(&u)
                    || (0x1FA70..=0x1FAFF).contains(&u);
                assert!(!is_emoji, "Found emoji character: '{}' (U+{:X}) in '{}'", ch, u, s);
            }
        };

        check_no_emoji(&card.tag);
        check_no_emoji(&card.target_display);
        check_no_emoji(&card.time_display);
        check_no_emoji(&card.meta_display);
        for line in &card.drops_summary {
            check_no_emoji(line);
        }
    }

    #[test]
    fn test_compute_idle_dim_factor() {
        let now = std::time::Instant::now();
        // Fresh report (under 20s) -> 1.0 full opacity
        assert_eq!(compute_idle_dim_factor(now), 1.0);
        // Old report (> 20s) -> 0.65 dimmed opacity
        let past = now - std::time::Duration::from_secs(30);
        assert!((compute_idle_dim_factor(past) - 0.65).abs() < 0.01);
    }

    #[test]
    fn test_get_latest_filtered_records_limit_5() {
        let mut records = Vec::new();
        for i in 1..=10 {
            records.push(BattleRecord {
                filename: format!("mail_{}", i),
                mail_id: i.to_string(),
                battle_type: if i % 2 == 0 { "fort".into() } else { "motte".into() },
                time_raw: i * 100,
                time_utc: "".into(),
                time_vn: "".into(),
                target: format!("Target {}", i),
                target_level: "1".into(),
                talent_add: 0,
                pos_x: 0,
                pos_y: 0,
                damage_pct: 100,
                plunder_tier: 0,
                items: vec![],
                item_totals: std::collections::HashMap::new(),
                content: String::new(),
            });
        }

        let forts = get_latest_filtered_records(&records, FilterMode::Fort, 5);
        assert_eq!(forts.len(), 5);
        for f in &forts {
            assert_eq!(f.battle_type, "fort");
        }
        assert_eq!(forts[0].time_raw, 1000);
    }

    #[test]
    fn test_card_accent_color_mapping() {
        assert_eq!(accent_color_for_battle_type("fort"), egui::Color32::from_rgb(245, 158, 11));
        assert_eq!(accent_color_for_battle_type("motte"), egui::Color32::from_rgb(239, 68, 68));
        assert_eq!(accent_color_for_battle_type("barbarian"), egui::Color32::from_rgb(59, 130, 246));
        assert_eq!(accent_color_for_battle_type("gathering"), egui::Color32::from_rgb(16, 185, 129));
    }

    #[test]
    fn test_see_through_opacity_zero() {
        let (bg, stroke, text_alpha) = compute_overlay_card_style(0.0, 1.0);
        assert_eq!(bg, Color32::TRANSPARENT, "Background at opacity 0 must be 100% transparent");
        assert_eq!(stroke, Stroke::NONE, "Stroke at opacity 0 must be None");
        assert!(text_alpha >= 0.40, "Text must remain readable at opacity 0");
    }

    #[test]
    fn test_compute_overlay_card_style_transparency_scaling() {
        // Zero opacity is completely transparent
        let (bg_0, _, _) = compute_overlay_card_style(0.0, 1.0);
        assert_eq!(bg_0.a(), 0);

        // 30% opacity has translucent alpha
        let (bg_30, stroke_30, _) = compute_overlay_card_style(0.3, 1.0);
        assert!(bg_30.a() > 50 && bg_30.a() < 100, "Alpha at 0.3 should be ~72, got {}", bg_30.a());
        assert!(stroke_30.color.a() > 50 && stroke_30.color.a() < 100);

        // 50% opacity has medium alpha
        let (bg_50, stroke_50, _) = compute_overlay_card_style(0.5, 1.0);
        assert!(bg_50.a() > 100 && bg_50.a() < 140, "Alpha at 0.5 should be ~120, got {}", bg_50.a());
        assert!(stroke_50.color.a() > 100 && stroke_50.color.a() < 140);

        // 100% opacity is solid dark card
        let (bg_100, stroke_100, _) = compute_overlay_card_style(1.0, 1.0);
        assert_eq!(bg_100.a(), 240);
        assert_eq!(stroke_100.color.a(), 255);

        // Monotonic scaling: 0.0 < 0.3 < 0.5 < 1.0
        assert!(bg_0.a() < bg_30.a());
        assert!(bg_30.a() < bg_50.a());
        assert!(bg_50.a() < bg_100.a());

        // Idle dimming scales alpha downward
        let (bg_dimmed, _, _) = compute_overlay_card_style(1.0, 0.65);
        assert!(bg_dimmed.a() < bg_100.a(), "Dimmed alpha {} should be less than active alpha {}", bg_dimmed.a(), bg_100.a());
    }
}
