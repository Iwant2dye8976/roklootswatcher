use std::path::PathBuf;
use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

use crate::i18n::TrKey;
use crate::model::{format_num, AppLanguage, CloseBehavior, FilterMode, OverlayConfig, OverlayDisplayMode, PinnedItemKey};
use crate::watcher::detect_default_mail_dir;
use super::super::components::is_network_offline_error;
use super::super::RoKLootWatcherApp;

impl RoKLootWatcherApp {
    pub(crate) fn render_view_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
                            egui::RichText::new(self.language.t(TrKey::SettingsCardMailTitle))
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
                        egui::RichText::new(self.language.t(TrKey::SettingsCardScanTitle))
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
                            egui::RichText::new(self.language.t(TrKey::SettingsBtnRescanForce))
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

                        // Mode Switcher
                        let mode_lbl = match self.language {
                            AppLanguage::Vietnamese => "Chế độ hiển thị:",
                            AppLanguage::English => "Display Mode:",
                        };
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.label(egui::RichText::new(mode_lbl).strong().color(c.canvas_tint));

                            let modes = [
                                (OverlayDisplayMode::Cards, "Cards"),
                                (OverlayDisplayMode::MiniHud, "Mini HUD"),
                                (OverlayDisplayMode::Combined, "Combined"),
                            ];

                            for (m, label) in modes {
                                let is_selected = self.overlay_config.mode == m;
                                let fill = if is_selected { c.primary_pressed } else { c.primary };
                                let stroke = if is_selected {
                                    Stroke::new(1.0_f32, c.focus_teal)
                                } else {
                                    Stroke::new(1.0_f32, c.hairline)
                                };
                                let col = if is_selected { c.focus_teal } else { c.text_muted };

                                let btn = egui::Button::new(
                                    egui::RichText::new(label)
                                        .font(egui::FontId::proportional(11.0))
                                        .strong()
                                        .color(col),
                                )
                                .fill(fill)
                                .stroke(stroke)
                                .rounding(Rounding::same(5.0));

                                if ui.add_sized([0.0, 26.0], btn).clicked() && !is_selected {
                                    self.overlay_config.mode = m;
                                    self.save_config();
                                    ctx.request_repaint();
                                }
                            }

                            if self.overlay_config.mode == OverlayDisplayMode::Hidden {
                                let hidden_hint = match self.language {
                                    AppLanguage::Vietnamese => "(Đang ẩn)",
                                    AppLanguage::English => "(Hidden)",
                                };
                                ui.label(egui::RichText::new(hidden_hint).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                            }
                        });

                        ui.add_space(8.0);

                        // Global Hotkey Toggle
                        let hotkey_lbl = match self.language {
                            AppLanguage::Vietnamese => "Phím tắt (Ctrl + Shift + H)",
                            AppLanguage::English => "Global Hotkey (Ctrl + Shift + H)",
                        };
                        let prev_hotkey = self.overlay_config.hotkey_enabled;
                        ui.checkbox(
                            &mut self.overlay_config.hotkey_enabled,
                            egui::RichText::new(hotkey_lbl).strong().color(c.canvas_tint),
                        );
                        if self.overlay_config.hotkey_enabled != prev_hotkey {
                            self.save_config();
                            ctx.request_repaint();
                        }

                        let hotkey_hint = match self.language {
                            AppLanguage::Vietnamese => "Nhấn Ctrl + Shift + H trong game để chuyển nhanh giữa các chế độ (Cards -> Mini HUD -> Combined -> Ẩn).",
                            AppLanguage::English => "Press Ctrl + Shift + H in-game to cycle overlay modes (Cards -> Mini HUD -> Combined -> Hidden).",
                        };
                        ui.label(egui::RichText::new(hotkey_hint).font(egui::FontId::proportional(10.0)).color(c.text_muted));

                        ui.add_space(10.0);

                        // Pinned Items Manager
                        let pinned_lbl = match self.language {
                            AppLanguage::Vietnamese => "Mục theo dõi nhanh (Tối đa 5 mục):",
                            AppLanguage::English => "Pinned Counters (Max 5 items):",
                        };
                        ui.label(egui::RichText::new(pinned_lbl).strong().color(c.canvas_tint));
                        ui.add_space(4.0);

                        let mut remove_idx: Option<usize> = None;
                        if self.overlay_config.pinned_items.is_empty() {
                            let empty_hint = match self.language {
                                AppLanguage::Vietnamese => "(Chưa ghim mục nào)",
                                AppLanguage::English => "(No items pinned)",
                            };
                            ui.label(egui::RichText::new(empty_hint).font(egui::FontId::proportional(10.5)).color(c.text_muted));
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                                for (idx, item) in self.overlay_config.pinned_items.iter().enumerate() {
                                    let col = pinned_item_color(item, &c);
                                    let name = item.localized_name(self.language);

                                    egui::Frame::none()
                                        .fill(c.primary)
                                        .stroke(Stroke::new(1.0_f32, c.hairline))
                                        .rounding(Rounding::same(4.0))
                                        .inner_margin(Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x = 4.0;
                                                ui.label(
                                                    egui::RichText::new(name)
                                                        .font(egui::FontId::proportional(10.5))
                                                        .strong()
                                                        .color(col),
                                                );
                                                let btn_del = egui::Button::new(
                                                    egui::RichText::new("x")
                                                        .font(egui::FontId::monospace(10.0))
                                                        .strong()
                                                        .color(c.text_muted),
                                                )
                                                .fill(Color32::TRANSPARENT)
                                                .stroke(Stroke::NONE)
                                                .min_size(egui::vec2(10.0, 10.0));

                                                if ui.add(btn_del).clicked() {
                                                    remove_idx = Some(idx);
                                                }
                                            });
                                        });
                                }
                            });
                        }

                        if let Some(idx) = remove_idx {
                            self.overlay_config.remove_pinned_item(idx);
                            self.refresh_cached_overlay_records();
                            self.save_config();
                            ctx.request_repaint();
                        }

                        if self.overlay_config.pinned_items.len() < OverlayConfig::MAX_PINNED_ITEMS {
                            ui.add_space(6.0);

                            // Presets row
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                let preset_btn = |text: &str| {
                                    egui::Button::new(
                                        egui::RichText::new(text)
                                            .font(egui::FontId::proportional(10.5))
                                            .strong()
                                            .color(c.canvas_tint),
                                    )
                                    .fill(c.primary)
                                    .stroke(Stroke::new(1.0_f32, c.hairline))
                                    .rounding(Rounding::same(4.0))
                                };

                                if ui.add_sized([0.0, 24.0], preset_btn("[+ Castle Push]")).clicked() {
                                    self.overlay_config.apply_pinned_preset(&preset_castle_push());
                                    self.refresh_cached_overlay_records();
                                    self.save_config();
                                    ctx.request_repaint();
                                }

                                if ui.add_sized([0.0, 24.0], preset_btn("[+ War & Training]")).clicked() {
                                    self.overlay_config.apply_pinned_preset(&preset_war_and_training());
                                    self.refresh_cached_overlay_records();
                                    self.save_config();
                                    ctx.request_repaint();
                                }

                                if ui.add_sized([0.0, 24.0], preset_btn("[+ Gathering & Heals]")).clicked() {
                                    self.overlay_config.apply_pinned_preset(&preset_gathering_and_heals());
                                    self.refresh_cached_overlay_records();
                                    self.save_config();
                                    ctx.request_repaint();
                                }
                            });

                            ui.add_space(6.0);

                            // Add Item Dropdown
                            let add_lbl = match self.language {
                                AppLanguage::Vietnamese => "Thêm mục:",
                                AppLanguage::English => "Add Item:",
                            };
                            let mut selected_to_add: Option<PinnedItemKey> = None;
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(add_lbl).color(c.text_muted));
                                egui::ComboBox::from_id_salt("overlay_add_pinned_item_select")
                                    .selected_text(match self.language {
                                        AppLanguage::Vietnamese => "+ Chọn mục để thêm...",
                                        AppLanguage::English => "+ Select item to add...",
                                    })
                                    .show_ui(ui, |ui| {
                                        for opt in available_pinned_item_options() {
                                            let is_pinned = self.overlay_config.pinned_items.contains(&opt);
                                            let opt_name = opt.localized_name(self.language);
                                            let label = if is_pinned {
                                                let suffix = match self.language {
                                                    AppLanguage::Vietnamese => "(Đã ghim)",
                                                    AppLanguage::English => "(Pinned)",
                                                };
                                                format!("{} {}", opt_name, suffix)
                                            } else {
                                                opt_name.to_string()
                                            };
                                            let color = if is_pinned {
                                                c.text_muted
                                            } else {
                                                pinned_item_color(&opt, &c)
                                            };
                                            if ui.add_enabled(!is_pinned, egui::SelectableLabel::new(false, egui::RichText::new(label).color(color))).clicked() {
                                                selected_to_add = Some(opt);
                                            }
                                        }
                                    });
                            });

                            if let Some(item) = selected_to_add {
                                if self.overlay_config.add_pinned_item(item) {
                                    self.refresh_cached_overlay_records();
                                    self.save_config();
                                    ctx.request_repaint();
                                }
                            }
                        }

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
                        egui::RichText::new(self.language.t(TrKey::SettingsCardSystemTitle))
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
                    let title = match self.language {
                        AppLanguage::Vietnamese => "NHÂN VẬT CHÍNH (MAIN CHARACTERS)",
                        AppLanguage::English => "MAIN CHARACTERS (UID FILTER)",
                    };
                    ui.label(
                        egui::RichText::new(title)
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.crystal_cyan),
                    );
                    ui.add_space(4.0);

                    let desc = match self.language {
                        AppLanguage::Vietnamese => "Chọn nhân vật UID chính để theo dõi. Thư của các UID khác sẽ tự động bị bỏ qua.",
                        AppLanguage::English => "Select main character UID(s) to track. Reports from other UIDs will be automatically ignored.",
                    };
                    ui.label(
                        egui::RichText::new(desc)
                            .font(egui::FontId::proportional(10.5))
                            .color(c.text_muted),
                    );
                    ui.add_space(8.0);

                    let all_detected = self.watcher.get_all_detected_accounts();
                    let mut changed = false;

                    if all_detected.is_empty() && self.main_account_uids.is_empty() {
                        let no_acc_msg = match self.language {
                            AppLanguage::Vietnamese => "Chưa phát hiện nhân vật nào trong dữ liệu thư.",
                            AppLanguage::English => "No characters detected in mail data yet.",
                        };
                        ui.label(egui::RichText::new(no_acc_msg).font(egui::FontId::proportional(11.0)).color(c.text_muted));
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(130.0)
                            .show(ui, |ui| {
                                for (acc_id, label) in &all_detected {
                                    let clean_id = acc_id.strip_prefix("player_").unwrap_or(acc_id).trim();
                                    let mut is_checked = self.main_account_uids.iter().any(|u| {
                                        u.strip_prefix("player_").unwrap_or(u).trim() == clean_id
                                    });
                                    if ui.checkbox(&mut is_checked, egui::RichText::new(label).color(c.canvas_tint)).changed() {
                                        if is_checked {
                                            if !self.main_account_uids.iter().any(|u| u.strip_prefix("player_").unwrap_or(u).trim() == clean_id) {
                                                self.main_account_uids.push(clean_id.to_string());
                                            }
                                        } else {
                                            self.main_account_uids.retain(|u| u.strip_prefix("player_").unwrap_or(u).trim() != clean_id);
                                        }
                                        changed = true;
                                    }
                                }
                            });
                    }

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let add_lbl = match self.language {
                            AppLanguage::Vietnamese => "Thêm UID:",
                            AppLanguage::English => "Add UID:",
                        };
                        ui.label(egui::RichText::new(add_lbl).font(egui::FontId::proportional(11.0)).color(c.text_muted));
                        ui.add_sized([120.0, 24.0], egui::TextEdit::singleline(&mut self.new_main_uid_input).hint_text("VD: 122980539"));
                        let btn_add = egui::Button::new(
                            egui::RichText::new("+ Thêm")
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(c.focus_teal)
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, c.focus_teal.linear_multiply(0.4)))
                        .rounding(Rounding::same(4.0));

                        if ui.add_sized([0.0, 24.0], btn_add).clicked() {
                            let uid = self.new_main_uid_input.strip_prefix("player_").unwrap_or(&self.new_main_uid_input).trim().to_string();
                            if !uid.is_empty() && !self.main_account_uids.contains(&uid) {
                                self.main_account_uids.push(uid);
                                self.new_main_uid_input.clear();
                                changed = true;
                            }
                        }
                    });

                    if changed {
                        self.watcher.set_main_account_uids(self.main_account_uids.clone());
                        self.save_config();
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let non_main_count = self.watcher.count_non_main_records();
                    ui.horizontal(|ui| {
                        let has_mains = !self.main_account_uids.is_empty();
                        let btn_del_text = match self.language {
                            AppLanguage::Vietnamese => format!("Xóa báo cáo UID phụ ({})", non_main_count),
                            AppLanguage::English => format!("Delete Non-Main Reports ({})", non_main_count),
                        };

                        let btn_del = egui::Button::new(
                            egui::RichText::new(btn_del_text)
                                .font(egui::FontId::proportional(11.0))
                                .strong()
                                .color(if non_main_count > 0 && has_mains { c.error } else { c.text_muted })
                        )
                        .fill(c.primary)
                        .stroke(Stroke::new(1.0_f32, if non_main_count > 0 && has_mains { c.error.linear_multiply(0.5) } else { c.hairline }))
                        .rounding(Rounding::same(5.0));

                        let resp = ui.add_sized([0.0, 26.0], btn_del);
                        let resp = if !has_mains {
                            resp.on_hover_text(match self.language {
                                AppLanguage::Vietnamese => "Vui lòng chọn ít nhất 1 nhân vật chính trước khi dọn dẹp.",
                                AppLanguage::English => "Please select at least 1 main character before cleaning up.",
                            })
                        } else if non_main_count == 0 {
                            resp.on_hover_text(match self.language {
                                AppLanguage::Vietnamese => "Không có báo cáo nào thuộc UID phụ trong dữ liệu.",
                                AppLanguage::English => "No secondary UID reports in database.",
                            })
                        } else {
                            resp
                        };

                        if has_mains && non_main_count > 0 && resp.clicked() {
                            self.confirm_delete_non_main = Some(non_main_count);
                        }
                    });
                });

                ui.add_space(12.0);

                card_frame.show(ui, |ui| {
                    let vault_total = self.watcher.get_vault_total_count();
                    ui.label(
                        egui::RichText::new(self.language.t(TrKey::SettingsCardVaultTitle))
                            .font(egui::FontId::proportional(12.5))
                            .strong()
                            .color(c.accent_gold),
                    );
                    ui.add_space(4.0);

                    let secured_str = self.language.t(TrKey::SettingsVaultSecured).replace("{}", &format_num(vault_total));
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(secured_str)
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
                            let out_dir = self.watcher.get_out_dir();
                            let export_path = out_dir.join("rok_history_vault_export.json");
                            if let Ok(json) = self.watcher.export_vault_json() {
                                let _ = std::fs::write(&export_path, &json);
                                let _ = open::that(&export_path);
                            }
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

                    let is_logged_in = self.cloud_auth.user_email.is_some()
                        && (self.cloud_auth.access_token.is_some() || self.cloud_auth.refresh_token.is_some());

                    let (worker_syncing, worker_restoring, pending_count, total_count, last_sync_time, last_err, retry_secs) = {
                        if let Some(ref worker) = self.sync_worker {
                            if let Ok(s) = worker.state.read() {
                                (
                                    s.is_syncing,
                                    s.is_restoring,
                                    s.pending_records_count,
                                    s.total_records_count,
                                    s.last_sync_time.clone(),
                                    s.last_error.clone(),
                                    s.retry_in_secs,
                                )
                            } else {
                                (false, false, 0, 0, None, None, None)
                            }
                        } else {
                            (false, false, 0, 0, None, None, None)
                        }
                    };

                    let (status_text, is_logged_in) = match (
                        &self.cloud_auth.user_email,
                        is_logged_in,
                    ) {
                        (Some(email), true) => {
                            let sync_info = last_sync_time
                                .as_deref()
                                .or(self.cloud_auth.last_synced.as_deref())
                                .unwrap_or("Chưa đồng bộ / Never");
                            let txt = match self.language {
                                AppLanguage::Vietnamese => {
                                    format!("● Tài khoản: {} | Lần đồng bộ gần nhất: {}", email, sync_info)
                                }
                                AppLanguage::English => {
                                    format!("● Account: {} | Last sync: {}", email, sync_info)
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

                    if is_logged_in {
                        ui.add_space(3.0);
                        let is_busy_restoring = worker_restoring || self.restore_rx.is_some();
                        let is_busy_syncing = worker_syncing || self.sync_rx.is_some();
                        let (live_text, live_col, show_spinner) = if is_busy_restoring {
                            (
                                self.language.t(TrKey::CloudSyncStatusRestoring).replace("{0}", &total_count.to_string()),
                                c.accent_gold,
                                true,
                            )
                        } else if is_busy_syncing {
                            (
                                self.language.t(TrKey::CloudSyncStatusSyncing).replace("{0}", &pending_count.to_string()),
                                c.crystal_cyan,
                                true,
                            )
                        } else if retry_secs.is_some() || (last_err.is_some() && pending_count > 0) {
                            let is_offline = is_network_offline_error(last_err.as_deref());
                            let base_key = if is_offline {
                                TrKey::CloudSyncStatusOffline
                            } else {
                                TrKey::CloudSyncStatusError
                            };
                            let msg = if let Some(secs) = retry_secs {
                                format!("{} ({}s)", self.language.t(base_key), secs)
                            } else {
                                self.language.t(base_key).to_string()
                            };
                            (msg, c.error, false)
                        } else {
                            (
                                self.language.t(TrKey::CloudSyncStatusIdle).to_string(),
                                c.status_green,
                                false,
                            )
                        };

                        ui.horizontal(|ui| {
                            if show_spinner {
                                ui.spinner();
                            }
                            let resp = ui.label(
                                egui::RichText::new(&live_text)
                                    .font(egui::FontId::proportional(11.0))
                                    .color(live_col)
                                    .strong(),
                            );
                            if let Some(ref err) = last_err {
                                if !is_busy_syncing && !is_busy_restoring {
                                    resp.on_hover_text(err);
                                }
                            }
                            if pending_count > 0 && !is_busy_syncing && !is_busy_restoring {
                                let unsynced_txt = match self.language {
                                    AppLanguage::Vietnamese => format!("({} bản ghi chờ đồng bộ)", pending_count),
                                    AppLanguage::English => format!("({} pending records)", pending_count),
                                };
                                ui.label(
                                    egui::RichText::new(unsynced_txt)
                                        .font(egui::FontId::proportional(10.5))
                                        .color(c.text_muted),
                                );
                            }
                        });
                    }

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        if self.cloud_busy && self.cloud_rx.is_some() {
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
                            let btn_sync_txt = match self.language {
                                AppLanguage::Vietnamese => "Đồng Bộ Ngay",
                                AppLanguage::English => "Sync Now",
                            };
                            let btn_sync = egui::Button::new(
                                egui::RichText::new(btn_sync_txt)
                                    .font(egui::FontId::proportional(11.0))
                                    .strong()
                                    .color(c.status_green),
                            )
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.status_green.linear_multiply(0.4)))
                            .rounding(Rounding::same(5.0));

                            let is_busy = worker_syncing || worker_restoring || self.cloud_busy || self.sync_rx.is_some();
                            if ui.add_sized([0.0, 28.0], btn_sync).clicked() && !is_busy {
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

                            if ui.add_sized([0.0, 28.0], btn_restore).clicked() && !is_busy {
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

                            if ui.add_sized([0.0, 28.0], btn_logout).clicked() && !worker_restoring && !self.cloud_busy {
                                self.cloud_auth.access_token = None;
                                self.cloud_auth.refresh_token = None;
                                self.cloud_auth.user_id = None;
                                self.cloud_auth.user_email = None;
                                self.save_config();
                                if let Some(ref worker) = self.sync_worker {
                                    let _ = worker.tx.send(crate::cloud_sync::SyncCommand::UpdateConfig(self.cloud_auth.clone()));
                                }
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
                        if let Some(ref worker) = self.sync_worker {
                            let _ = worker.tx.send(crate::cloud_sync::SyncCommand::UpdateConfig(self.cloud_auth.clone()));
                        }
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
                            if let Some(ref worker) = self.sync_worker {
                                let _ = worker.tx.send(crate::cloud_sync::SyncCommand::UpdateConfig(self.cloud_auth.clone()));
                            }
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
                            egui::RichText::new(self.language.t(TrKey::LogsTitle))
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
}

impl OverlayConfig {
    pub const MAX_PINNED_ITEMS: usize = 5;

    pub fn add_pinned_item(&mut self, item: PinnedItemKey) -> bool {
        if self.pinned_items.len() >= Self::MAX_PINNED_ITEMS {
            return false;
        }
        if self.pinned_items.contains(&item) {
            return false;
        }
        self.pinned_items.push(item);
        true
    }

    pub fn remove_pinned_item(&mut self, index: usize) -> Option<PinnedItemKey> {
        if index < self.pinned_items.len() {
            Some(self.pinned_items.remove(index))
        } else {
            None
        }
    }

    pub fn apply_pinned_preset(&mut self, preset_items: &[PinnedItemKey]) {
        for item in preset_items {
            if self.pinned_items.len() >= Self::MAX_PINNED_ITEMS {
                break;
            }
            if !self.pinned_items.contains(item) {
                self.pinned_items.push(item.clone());
            }
        }
    }
}

impl PinnedItemKey {
    pub fn canonical_name(&self) -> &str {
        match self {
            Self::CatalogItem { canonical_name, .. } => canonical_name.as_str(),
            Self::SpeedupUniversal => "Universal Speedup",
            Self::SpeedupTraining => "Training Speedup",
            Self::SpeedupHealing => "Healing Speedup",
        }
    }

    pub fn localized_name(&self, lang: AppLanguage) -> &str {
        match self {
            Self::SpeedupUniversal => match lang {
                AppLanguage::Vietnamese => "Tăng tốc Đa Năng",
                AppLanguage::English => "Universal Speedup",
            },
            Self::SpeedupTraining => match lang {
                AppLanguage::Vietnamese => "Tăng tốc Huấn Luyện",
                AppLanguage::English => "Training Speedup",
            },
            Self::SpeedupHealing => match lang {
                AppLanguage::Vietnamese => "Tăng tốc Chữa Lành",
                AppLanguage::English => "Healing Speedup",
            },
            Self::CatalogItem { canonical_name, .. } => match canonical_name.as_str() {
                "Gems" => match lang {
                    AppLanguage::Vietnamese => "Đá quý",
                    AppLanguage::English => "Gems",
                },
                "Book of Covenant" => match lang {
                    AppLanguage::Vietnamese => "Sách giao ước",
                    AppLanguage::English => "Book of Covenant",
                },
                "Arrow of Resistance" => match lang {
                    AppLanguage::Vietnamese => "Mũi tên kháng cự",
                    AppLanguage::English => "Arrow of Resistance",
                },
                "Gold Key" => match lang {
                    AppLanguage::Vietnamese => "Chìa khóa vàng",
                    AppLanguage::English => "Gold Key",
                },
                "Silver Key" => match lang {
                    AppLanguage::Vietnamese => "Chìa khóa bạc",
                    AppLanguage::English => "Silver Key",
                },
                "Iron Ore" => match lang {
                    AppLanguage::Vietnamese => "Quặng sắt",
                    AppLanguage::English => "Iron Ore",
                },
                "Ebony" => match lang {
                    AppLanguage::Vietnamese => "Gỗ mun",
                    AppLanguage::English => "Ebony",
                },
                "Animal Bone" => match lang {
                    AppLanguage::Vietnamese => "Xương động vật",
                    AppLanguage::English => "Animal Bone",
                },
                "Leather" => match lang {
                    AppLanguage::Vietnamese => "Da lông",
                    AppLanguage::English => "Leather",
                },
                other => other,
            },
        }
    }
}

pub fn preset_castle_push() -> Vec<PinnedItemKey> {
    vec![
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 3,
            canonical_name: "Book of Covenant".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 4,
            canonical_name: "Arrow of Resistance".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        },
    ]
}

pub fn preset_war_and_training() -> Vec<PinnedItemKey> {
    vec![
        PinnedItemKey::SpeedupUniversal,
        PinnedItemKey::SpeedupTraining,
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        },
    ]
}

pub fn preset_gathering_and_heals() -> Vec<PinnedItemKey> {
    vec![
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        },
        PinnedItemKey::SpeedupHealing,
        PinnedItemKey::SpeedupUniversal,
    ]
}

pub fn available_pinned_item_options() -> Vec<PinnedItemKey> {
    vec![
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 3,
            canonical_name: "Book of Covenant".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 4,
            canonical_name: "Arrow of Resistance".to_string(),
        },
        PinnedItemKey::SpeedupUniversal,
        PinnedItemKey::SpeedupTraining,
        PinnedItemKey::SpeedupHealing,
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 48,
            canonical_name: "Gold Key".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 47,
            canonical_name: "Silver Key".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 413,
            canonical_name: "Iron Ore".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 417,
            canonical_name: "Ebony".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 414,
            canonical_name: "Animal Bone".to_string(),
        },
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 412,
            canonical_name: "Leather".to_string(),
        },
    ]
}

pub fn pinned_item_color(item: &PinnedItemKey, c: &crate::gui::theme::ThemeColors) -> Color32 {
    match item {
        PinnedItemKey::SpeedupUniversal => c.speedup_blue,
        PinnedItemKey::SpeedupTraining => c.train_orange,
        PinnedItemKey::SpeedupHealing => c.heal_blue,
        PinnedItemKey::CatalogItem { canonical_name, .. } => match canonical_name.as_str() {
            "Gems" => c.crystal_cyan,
            "Book of Covenant" | "Gold Key" => c.accent_gold,
            "Arrow of Resistance" => c.train_orange,
            "Silver Key" => c.rank_silver,
            "Iron Ore" => c.mat_iron,
            "Ebony" => c.mat_ebony,
            "Animal Bone" => c.mat_bone,
            "Leather" => c.rank_bronze,
            _ => c.canvas_tint,
        },
    }
}
