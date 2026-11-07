use eframe::egui::{self, Margin, Rounding, Stroke};

use crate::i18n::TrKey;
use crate::model::{
    format_in_millions, format_num, AppLanguage, FilterMode,
};
use super::super::components::{
    render_sort_header_cell, report_non_target_column_count, report_table_dimensions,
};
use super::super::{BattleSortColumn, RoKLootWatcherApp, SortDirection};

impl RoKLootWatcherApp {
    pub(crate) fn render_view_battles(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();

        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.language.t(TrKey::SearchLabel)).strong().color(c.canvas_tint));
                    ui.add_sized([260.0, 22.0], egui::TextEdit::singleline(&mut self.battle_search).hint_text(self.language.t(TrKey::SearchPlaceholder)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if ui.button(self.language.t(TrKey::BtnExportJson)).on_hover_text(self.language.t(TrKey::BtnExportJsonTooltip)).clicked() {
                            self.export_active_json();
                        }
                        if ui.button(self.language.t(TrKey::BtnExportCsv)).on_hover_text(self.language.t(TrKey::BtnExportCsvTooltip)).clicked() {
                            self.export_active_csv();
                        }

                        let cur_cnt = self.watcher.get_active_count();
                        let match_txt = self.language.t(TrKey::FilterMatchingBattles).replace("{}", &format_num(cur_cnt));
                        egui::Frame::none()
                            .fill(c.primary)
                            .stroke(Stroke::new(1.0_f32, c.hairline))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(8.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(match_txt)
                                        .font(egui::FontId::monospace(11.0))
                                        .strong()
                                        .color(c.focus_teal),
                                );
                            });
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
        const COL_BARB_UNIV: f32 = 100.0;
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
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(TABLE_PAD_X, 5.0))
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
                            || r.display_target(cur_lang).to_lowercase().contains(&search_lower)
                            || r.account_id.contains(&search_lower)
                            || format!("player_{}", r.account_id).contains(&search_lower)
                            || r.player_name.to_lowercase().contains(&search_lower);
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
                            c.primary_pressed,
                        );
                        ui.painter().rect_stroke(
                            row_rect,
                            egui::Rounding::same(5.0),
                            egui::Stroke::new(1.0_f32, c.focus_teal),
                        );
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    } else if idx % 2 == 1 {
                        ui.painter().rect_filled(
                            row_rect,
                            egui::Rounding::same(4.0),
                            c.card_surface,
                        );
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
}
