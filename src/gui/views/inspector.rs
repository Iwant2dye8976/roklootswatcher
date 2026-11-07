use eframe::egui::{self, Margin, Rounding, Stroke};

use crate::i18n::{category_name, TrKey};
use crate::model::{format_num, get_item_converted_display_for_lang, AppLanguage, BattleRecord};
use super::super::RoKLootWatcherApp;

impl RoKLootWatcherApp {
    pub(crate) fn render_side_inspector(&mut self, ui: &mut egui::Ui, record: &BattleRecord, close_inspector: &mut bool) {
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
