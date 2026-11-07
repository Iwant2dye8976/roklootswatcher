use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::i18n::TrKey;
use crate::model::{format_num, AppLanguage, AppTheme};
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
                            egui::RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
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

                    ui.add_space(8.0);

                    let col_gap = 8.0;
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

                    ui.add_space(8.0);

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
}
