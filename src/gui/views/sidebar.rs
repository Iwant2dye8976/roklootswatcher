use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::i18n::{date_preset_label, filter_mode_label, TrKey};
use crate::model::{format_num, AppLanguage, AppTheme, DateFilterPreset, FilterMode, SpeedupUnit};
use crate::watcher::WatcherStatus;
use super::super::components::resolve_status_badge;
use super::super::RoKLootWatcherApp;

impl RoKLootWatcherApp {
    pub(crate) fn render_sidebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let c = self.colors();

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("RISE OF KINGDOMS")
                        .font(egui::FontId::proportional(9.5))
                        .strong()
                        .color(c.brand_mark),
                );
                ui.add_space(1.0);
                ui.label(
                    egui::RichText::new("Loot Watcher")
                        .font(egui::FontId::proportional(15.0))
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
                            egui::RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                                .font(egui::FontId::monospace(9.5))
                                .strong()
                                .color(c.text_muted),
                        );
                    });
            });
        });

        ui.add_space(8.0);
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
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), egui::Sense::click());
            let is_hovered = resp.hovered();

            if is_hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let bg_fill = if is_active {
                c.primary_pressed
            } else if is_hovered {
                c.primary
            } else {
                Color32::TRANSPARENT
            };

            ui.painter().rect_filled(rect, Rounding::same(6.0), bg_fill);

            if is_active {
                let bar_rect = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height()));
                ui.painter().rect_filled(bar_rect, Rounding::same(2.0), c.focus_teal);
            }

            let text_pos = egui::pos2(rect.min.x + 12.0, rect.center().y);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(12.5),
                if is_active || is_hovered { c.canvas_tint } else { c.text_muted },
            );

            let clicked = resp.clicked();
            resp.on_hover_text(desc);

            if clicked {
                self.current_tab = idx;
            }

            ui.add_space(4.0);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(4.0);

            egui::Frame::none()
                .fill(c.card_surface)
                .rounding(Rounding::same(6.0))
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
                                .font(egui::FontId::proportional(10.0))
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
                                            .font(egui::FontId::monospace(10.0))
                                            .strong()
                                            .color(c.canvas_tint),
                                    );
                                })
                                .response
                                .on_hover_text(&vault_txt);
                        });
                    });

                    ui.add_space(8.0);

                    let col_gap = 6.0;
                    let btn_w = ((ui.available_width() - col_gap) / 2.0).max(36.0);
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
                        .rounding(Rounding::same(4.0));

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
                        .rounding(Rounding::same(4.0));

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

                    ui.add_space(6.0);

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
                        .rounding(Rounding::same(4.0));

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
                        .rounding(Rounding::same(4.0));

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

    pub(crate) fn render_global_filter_bar(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(self.language.t(TrKey::FilterReportType))
                    .font(egui::FontId::proportional(11.5))
                    .strong()
                    .color(c.canvas_tint),
            );
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
                self.reports_sort_dirty = true;
                self.ap_econ_dirty = true;
                self.save_config();
            }

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(self.language.t(TrKey::FilterDateRange))
                    .font(egui::FontId::proportional(11.5))
                    .strong()
                    .color(c.canvas_tint),
            );
            let prev_preset = self.selected_date_preset.clone();
            egui::ComboBox::from_id_salt("global_date_combo")
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
                ui.add_sized([78.0, 20.0], egui::TextEdit::singleline(&mut self.date_from_input));
                ui.label(egui::RichText::new(self.language.t(TrKey::FilterTo)).color(c.text_muted));
                ui.add_sized([78.0, 20.0], egui::TextEdit::singleline(&mut self.date_to_input));
                if ui.button(self.language.t(TrKey::FilterApply)).clicked() {
                    self.apply_custom_date_filter();
                }
            }

            let accounts = self.watcher.get_accounts();
            if !accounts.is_empty() {
                ui.add_space(8.0);
                let acc_lbl_tag = match self.language {
                    AppLanguage::Vietnamese => "Tài khoản:",
                    AppLanguage::English => "Account:",
                };
                ui.label(
                    egui::RichText::new(acc_lbl_tag)
                        .font(egui::FontId::proportional(11.5))
                        .strong()
                        .color(c.canvas_tint),
                );

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
                egui::ComboBox::from_id_salt("global_acc_combo")
                    .selected_text(selected_text)
                    .width(145.0)
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
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                }
            }

            let cur_battles = self.watcher.get_stats().total_battles;
            let battles_txt = self.language.t(TrKey::FilterRecordedBattles).replace("{}", &format_num(cur_battles));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::none()
                    .fill(c.primary)
                    .stroke(Stroke::new(1.0_f32, c.hairline))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(battles_txt)
                                .font(egui::FontId::monospace(11.0))
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
            });
        });

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(self.language.t(TrKey::SpeedupUnitChange))
                    .font(egui::FontId::proportional(11.5))
                    .strong()
                    .color(c.canvas_tint),
            );
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
                ui.add_space(8.0);
                let equiv_text = if self.speedup_unit == SpeedupUnit::Dhm {
                    match self.language {
                        AppLanguage::Vietnamese => format!("= {:.1} giờ ({} phút)", spd_total as f64 / 60.0, format_num(spd_total)),
                        AppLanguage::English => format!("= {:.1} hours ({} minutes)", spd_total as f64 / 60.0, format_num(spd_total)),
                    }
                } else {
                    format!(
                        "= {}",
                        SpeedupUnit::Dhm.format_kpi(spd_total, self.language).0
                    )
                };
                ui.label(
                    egui::RichText::new(equiv_text)
                        .font(egui::FontId::monospace(11.0))
                        .strong()
                        .color(c.status_green),
                );
            }
        });
    }
}
