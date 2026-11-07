use eframe::egui::{self, Color32, FontId, Frame, Margin, RichText, Rounding, Stroke};
use crate::model::{parse_hex_color, BattleRecord, OverlayConfig, OverlayDisplayMode, PinnedItemKey};

#[derive(Debug, Clone, PartialEq)]
pub struct CachedPinnedChip {
    pub key: PinnedItemKey,
    pub label: String,        // e.g. "GEMS", "BOOKS", "ARROWS", "SPEEDUP"
    pub display_val: String,  // e.g. "1,450", "32", "14h 30m"
    pub text_color: Color32,
}

pub fn format_speedup_compact(minutes: i64) -> String {
    if minutes <= 0 {
        return "0m".to_string();
    }
    let days = minutes / 1440;
    let rem = minutes % 1440;
    let hours = rem / 60;
    let mins = rem % 60;
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if mins > 0 || parts.is_empty() {
        parts.push(format!("{}m", mins));
    }
    parts.join(" ")
}

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
        "fort" => Color32::from_rgb(245, 158, 11),
        "motte" => Color32::from_rgb(239, 68, 68),
        "barbarian" => Color32::from_rgb(59, 130, 246),
        "gathering" => Color32::from_rgb(16, 185, 129),
        _ => Color32::from_rgb(148, 163, 184),
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
        let progress = (elapsed - 20.0) / 3.0;
        1.0 - (progress * 0.35)
    }
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

#[allow(clippy::too_many_arguments)]
pub fn render_mini_hud_ribbon(
    ui: &mut egui::Ui,
    config: &OverlayConfig,
    bg_color: Color32,
    card_stroke: Stroke,
    text_alpha: f32,
    text_color: Color32,
    run_counts: (usize, usize, usize),
    pinned_chips: &[CachedPinnedChip],
) {
    Frame::none()
        .fill(bg_color)
        .rounding(Rounding::same(5.0))
        .stroke(card_stroke)
        .inner_margin(Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(5.0, 0.0);

                let badge_bg_alpha = if config.opacity <= 0.01 {
                    0
                } else {
                    ((0x1f as f32) * config.opacity * text_alpha).clamp(0.0, 255.0) as u8
                };
                let badge_stroke_alpha = if config.opacity <= 0.01 {
                    0
                } else {
                    ((0x40 as f32) * config.opacity * text_alpha).clamp(0.0, 255.0) as u8
                };

                let render_badge = |ui: &mut egui::Ui, label: &str, count: usize, r: u8, g: u8, b: u8| {
                    let pill_stroke = if badge_stroke_alpha == 0 {
                        Stroke::NONE
                    } else {
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(r, g, b, badge_stroke_alpha))
                    };
                    let pill_bg = Color32::from_rgba_unmultiplied(r, g, b, badge_bg_alpha);

                    Frame::none()
                        .fill(pill_bg)
                        .rounding(Rounding::same(3.0))
                        .stroke(pill_stroke)
                        .inner_margin(Margin::symmetric(4.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("{} {}", label, count))
                                    .font(FontId::proportional(10.0))
                                    .strong()
                                    .color(Color32::from_rgb(r, g, b).linear_multiply(text_alpha)),
                            );
                        });
                };

                render_badge(ui, "FORT", run_counts.0, 0xf5, 0x9e, 0x0b);
                render_badge(ui, "MOTTE", run_counts.1, 0xf4, 0x3f, 0x5e);
                render_badge(ui, "BARB", run_counts.2, 0x10, 0xb9, 0x81);

                ui.label(
                    RichText::new("|")
                        .color(Color32::from_rgb(0x47, 0x55, 0x69).linear_multiply(text_alpha))
                        .size(10.0),
                );

                if pinned_chips.is_empty() {
                    ui.label(
                        RichText::new("[No items pinned]")
                            .color(Color32::from_rgb(0x94, 0xa3, 0xb8).linear_multiply(text_alpha))
                            .size(10.0),
                    );
                } else {
                    for chip in pinned_chips {
                        let col = chip.text_color;
                        let pill_stroke = if badge_stroke_alpha == 0 {
                            Stroke::NONE
                        } else {
                            Stroke::new(
                                1.0_f32,
                                Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), badge_stroke_alpha),
                            )
                        };
                        let pill_bg = Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), badge_bg_alpha);

                        Frame::none()
                            .fill(pill_bg)
                            .rounding(Rounding::same(3.0))
                            .stroke(pill_stroke)
                            .inner_margin(Margin::symmetric(4.0, 2.0))
                            .show(ui, |ui| {
                                let mut job = egui::text::LayoutJob::default();
                                job.wrap.max_rows = 1;
                                job.append(
                                    &chip.label,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: FontId::proportional(10.0),
                                        color: chip.text_color.linear_multiply(text_alpha),
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    " ",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: FontId::proportional(10.0),
                                        color: chip.text_color.linear_multiply(text_alpha),
                                        ..Default::default()
                                    },
                                );
                                job.append(
                                    &chip.display_val,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: FontId::proportional(10.0),
                                        color: text_color,
                                        ..Default::default()
                                    },
                                );
                                ui.label(job);
                            });
                    }
                }
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn render_cards_stack(
    ui: &mut egui::Ui,
    records: &[BattleRecord],
    config: &OverlayConfig,
    bg_color: Color32,
    card_stroke: Stroke,
    text_color: Color32,
    muted_text: Color32,
    accent_alpha: f32,
) {
    let fs_title = config.font_size;
    let fs_sub = (config.font_size * 0.88).max(8.0);
    let fs_time = (config.font_size * 0.82).max(7.5);
    let pad_v = (config.font_size * 0.3).clamp(2.0, 6.0);
    let pad_h = (config.font_size * 0.7).clamp(6.0, 12.0);

    ui.spacing_mut().item_spacing = egui::vec2(0.0, 3.0);

    if records.is_empty() {
        Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
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
                ui.set_min_width(ui.available_width());
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
}

pub fn render_overlay_viewport(
    ctx: &egui::Context,
    records: &[BattleRecord],
    config: &OverlayConfig,
    last_mail_time: std::time::Instant,
    run_counts: (usize, usize, usize),
    pinned_chips: &[CachedPinnedChip],
) {
    if config.mode == OverlayDisplayMode::Hidden {
        return;
    }

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

    egui::CentralPanel::default()
        .frame(Frame::none().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            match config.mode {
                OverlayDisplayMode::MiniHud => {
                    render_mini_hud_ribbon(
                        ui,
                        config,
                        bg_color,
                        card_stroke,
                        text_alpha,
                        text_color,
                        run_counts,
                        pinned_chips,
                    );
                }
                OverlayDisplayMode::Cards => {
                    render_cards_stack(
                        ui,
                        records,
                        config,
                        bg_color,
                        card_stroke,
                        text_color,
                        muted_text,
                        accent_alpha,
                    );
                }
                OverlayDisplayMode::Combined => {
                    render_mini_hud_ribbon(
                        ui,
                        config,
                        bg_color,
                        card_stroke,
                        text_alpha,
                        text_color,
                        run_counts,
                        pinned_chips,
                    );
                    ui.add_space(4.0);
                    render_cards_stack(
                        ui,
                        records,
                        config,
                        bg_color,
                        card_stroke,
                        text_color,
                        muted_text,
                        accent_alpha,
                    );
                }
                OverlayDisplayMode::Hidden => {}
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
            ..Default::default()
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
        assert_eq!(compute_idle_dim_factor(now), 1.0);
        let past = now - std::time::Duration::from_secs(30);
        assert!((compute_idle_dim_factor(past) - 0.65).abs() < 0.01);
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
        let (bg_0, _, _) = compute_overlay_card_style(0.0, 1.0);
        assert_eq!(bg_0.a(), 0);

        let (bg_30, stroke_30, _) = compute_overlay_card_style(0.3, 1.0);
        assert!(bg_30.a() > 50 && bg_30.a() < 100, "Alpha at 0.3 should be ~72, got {}", bg_30.a());
        assert!(stroke_30.color.a() > 50 && stroke_30.color.a() < 100);

        let (bg_50, stroke_50, _) = compute_overlay_card_style(0.5, 1.0);
        assert!(bg_50.a() > 100 && bg_50.a() < 140, "Alpha at 0.5 should be ~120, got {}", bg_50.a());
        assert!(stroke_50.color.a() > 100 && stroke_50.color.a() < 140);

        let (bg_100, stroke_100, _) = compute_overlay_card_style(1.0, 1.0);
        assert_eq!(bg_100.a(), 240);
        assert_eq!(stroke_100.color.a(), 255);

        assert!(bg_0.a() < bg_30.a());
        assert!(bg_30.a() < bg_50.a());
        assert!(bg_50.a() < bg_100.a());

        let (bg_dimmed, _, _) = compute_overlay_card_style(1.0, 0.65);
        assert!(bg_dimmed.a() < bg_100.a(), "Dimmed alpha {} should be less than active alpha {}", bg_dimmed.a(), bg_100.a());
    }

    #[test]
    fn test_format_speedup_compact() {
        assert_eq!(format_speedup_compact(0), "0m");
        assert_eq!(format_speedup_compact(-15), "0m");
        assert_eq!(format_speedup_compact(45), "45m");
        assert_eq!(format_speedup_compact(60), "1h");
        assert_eq!(format_speedup_compact(90), "1h 30m");
        assert_eq!(format_speedup_compact(1440), "1d");
        assert_eq!(format_speedup_compact(1500), "1d 1h");
        assert_eq!(format_speedup_compact(1515), "1d 1h 15m");
    }

    #[test]
    fn test_mini_hud_zero_emojis() {
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

        // Ribbon badges
        check_no_emoji("FORT");
        check_no_emoji("MOTTE");
        check_no_emoji("BARB");
        check_no_emoji("|");
        check_no_emoji("[No items pinned]");
        check_no_emoji("•");

        // Standard pinned chip labels
        let sample_labels = ["GEMS", "BOOKS", "ARROWS", "SPEEDUP", "TRAIN", "HEAL"];
        for label in &sample_labels {
            check_no_emoji(label);
        }

        // Formatted chip values
        let sample_vals = [
            format_speedup_compact(0),
            format_speedup_compact(870),
            crate::model::format_num(0),
            crate::model::format_num(1450),
            crate::model::format_num(100_000),
        ];
        for val in &sample_vals {
            check_no_emoji(val);
        }
    }

    #[test]
    fn test_render_overlay_viewport_mode_dispatch() {
        let ctx = egui::Context::default();
        let now = std::time::Instant::now();

        let sample_rec = BattleRecord {
            filename: "test.mail".into(),
            mail_id: "101".into(),
            battle_type: "fort".into(),
            time_raw: 2000,
            time_vn: "14:00".into(),
            target: "Fortress Lvl 10".into(),
            target_level: "10".into(),
            pos_x: 100,
            pos_y: 200,
            damage_pct: 85,
            ..Default::default()
        };
        let records = vec![sample_rec];

        let chips = vec![
            CachedPinnedChip {
                key: PinnedItemKey::CatalogItem {
                    itype: 1,
                    isub: 2,
                    canonical_name: "Gems".into(),
                },
                label: "GEMS".into(),
                display_val: "1,450".into(),
                text_color: Color32::from_rgb(6, 182, 212),
            },
            CachedPinnedChip {
                key: PinnedItemKey::SpeedupUniversal,
                label: "SPEEDUP".into(),
                display_val: "14h 30m".into(),
                text_color: Color32::from_rgb(56, 189, 248),
            },
        ];

        let modes = [
            OverlayDisplayMode::Cards,
            OverlayDisplayMode::MiniHud,
            OverlayDisplayMode::Combined,
            OverlayDisplayMode::Hidden,
        ];

        for mode in modes {
            let mut cfg = OverlayConfig::default();
            cfg.mode = mode;

            // Test with populated records and chips
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                render_overlay_viewport(ctx, &records, &cfg, now, (14, 6, 32), &chips);
            });

            // Test with empty records and empty chips
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                render_overlay_viewport(ctx, &[], &cfg, now, (0, 0, 0), &[]);
            });
        }
    }
}
