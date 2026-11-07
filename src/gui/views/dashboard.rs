use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::i18n::{date_preset_label, filter_mode_label, TrKey};
use crate::model::{
    format_in_millions, format_num, get_item_converted_display_for_lang, AppLanguage,
    DateFilterPreset, FilterMode, SpeedupUnit,
};
use super::super::components::localize_benchmark_target;
use super::super::theme::ThemeColors;
use super::super::RoKLootWatcherApp;

impl RoKLootWatcherApp {
    pub(crate) fn render_view_dashboard(&mut self, ui: &mut egui::Ui) {
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

                        let accounts = self.watcher.get_accounts();
                        if !accounts.is_empty() {
                            ui.add_space(10.0);
                            let acc_lbl_tag = match self.language {
                                AppLanguage::Vietnamese => "Tài khoản:",
                                AppLanguage::English => "Account:",
                            };
                            ui.label(egui::RichText::new(acc_lbl_tag).strong().color(c.canvas_tint));

                            let selected_text = match &self.selected_account {
                                None => match self.language {
                                    AppLanguage::Vietnamese => "Tất cả tài khoản".to_string(),
                                    AppLanguage::English => "All Accounts".to_string(),
                                },
                                Some(acc) => accounts
                                    .iter()
                                    .find(|(id, _)| id == acc)
                                    .map(|(_, l)| l.clone())
                                    .unwrap_or_else(|| acc.clone()),
                            };

                            let prev_acc = self.selected_account.clone();
                            egui::ComboBox::from_id_salt("dash_acc_combo")
                                .selected_text(selected_text)
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    let all_txt = match self.language {
                                        AppLanguage::Vietnamese => "Tất cả tài khoản",
                                        AppLanguage::English => "All Accounts",
                                    };
                                    ui.selectable_value(&mut self.selected_account, None, all_txt);
                                    for (acc_id, label) in &accounts {
                                        ui.selectable_value(&mut self.selected_account, Some(acc_id.clone()), label);
                                    }
                                });

                            if self.selected_account != prev_acc {
                                self.watcher.set_account_filter(self.selected_account.clone());
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
                    c.speedup_blue,
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

    pub(crate) fn render_ap_economics_section(
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
                                .color(c.speedup_blue),
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
                            painter.rect_filled(spd_rect, Rounding::same(2.0), c.speedup_blue);
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
                            ui.label(egui::RichText::new(format!("{}: {:.1}h ({:.0}m)", lbl_spd, d.speedup_hours, d.speedup_hours * 60.0)).color(c.speedup_blue).monospace());
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
}
