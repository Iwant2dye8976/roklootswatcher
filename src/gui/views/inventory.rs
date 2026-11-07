use eframe::egui::{self, Margin, Rounding, Stroke};

use crate::i18n::{category_name, TrKey};
use crate::model::{format_num, get_item_converted_display_for_lang, AppLanguage};
use super::super::components::render_item_sort_header_cell;
use super::super::theme::category_badge_color;
use super::super::{ItemSortColumn, RoKLootWatcherApp, SortDirection};

impl RoKLootWatcherApp {
    pub(crate) fn render_view_inventory(&mut self, ui: &mut egui::Ui) {
        let c = self.colors();
        let stats = self.watcher.get_stats();
        let total_battles = stats.total_battles.max(1);

        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.language.t(TrKey::InvSearchLabel)).strong().color(c.canvas_tint));
                    ui.add_sized([220.0, 22.0], egui::TextEdit::singleline(&mut self.item_search).hint_text(self.language.t(TrKey::InvSearchPlaceholder)));

                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(self.language.t(TrKey::InvCategoryLabel)).color(c.text_muted));
                    let categories = ["Tất cả", "Tài Nguyên", "Vật Liệu", "Tăng Tốc", "Pha Lê", "Sách & Tên", "Khác"];
                    for cat in categories {
                        let is_active = self.item_category_filter == cat;
                        let cat_display = category_name(cat, self.language);
                        if ui.selectable_label(is_active, cat_display).clicked() {
                            self.item_category_filter = cat.to_string();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cnt_txt = self.language.t(TrKey::InvItemCount).replace("{}", &format_num(stats.item_stats.len()));
                        ui.label(
                            egui::RichText::new(cnt_txt)
                                .strong()
                                .color(c.focus_teal),
                        );
                    });
                });
            });

        ui.add_space(8.0);

        let res = stats.compute_resources();
        if res.total_rss_val() > 0 {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.language.t(TrKey::InvRssSummaryTitle))
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.accent_gold),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let total_lbl = match self.language {
                                AppLanguage::Vietnamese => "Tổng cộng",
                                AppLanguage::English => "Total",
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}: {:.2}M RSS ({})",
                                    total_lbl,
                                    res.total_rss_val() as f64 / 1_000_000.0,
                                    format_num(res.total_rss_val())
                                ))
                                .font(egui::FontId::proportional(12.5))
                                .color(c.status_green)
                                .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    let pack_mult = match self.language {
                        AppLanguage::Vietnamese => "gói",
                        AppLanguage::English => "packs",
                    };
                    let total_packs_desc = match self.language {
                        AppLanguage::Vietnamese => "Số lượng gói",
                        AppLanguage::English => "Total packs",
                    };
                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvFood)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.food_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.accent_gold));
                            let pack_label = if res.food_packs * 1_000 == res.food_val {
                                format!("{} {} (x1.000)", format_num(res.food_packs), pack_mult)
                            } else if res.food_packs == res.food_val {
                                format_num(res.food_val)
                            } else {
                                format!("{} {}", format_num(res.food_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvWood)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.wood_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.status_green));
                            let pack_label = if res.wood_packs * 1_000 == res.wood_val {
                                format!("{} {} (x1.000)", format_num(res.wood_packs), pack_mult)
                            } else if res.wood_packs == res.wood_val {
                                format_num(res.wood_val)
                            } else {
                                format!("{} {}", format_num(res.wood_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvStone)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.stone_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.heal_blue));
                            let pack_label = if res.stone_packs * 750 == res.stone_val {
                                format!("{} {} (x750)", format_num(res.stone_packs), pack_mult)
                            } else if res.stone_packs == res.stone_val {
                                format_num(res.stone_val)
                            } else {
                                format!("{} {}", format_num(res.stone_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvGold)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{:.2}M", res.gold_val as f64 / 1_000_000.0)).font(egui::FontId::proportional(15.0)).strong().color(c.accent_gold));
                            let pack_label = if res.gold_packs * 500 == res.gold_val {
                                format!("{} {} (x500)", format_num(res.gold_packs), pack_mult)
                            } else if res.gold_packs == res.gold_val {
                                format_num(res.gold_val)
                            } else {
                                format!("{} {}", format_num(res.gold_packs), pack_mult)
                            };
                            ui.label(egui::RichText::new(pack_label).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new(self.language.t(TrKey::InvChoicePacks)).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(res.choice_packs), pack_mult)).font(egui::FontId::proportional(15.0)).strong().color(c.focus_teal));
                            ui.label(egui::RichText::new(total_packs_desc).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                    });
                });

            ui.add_space(8.0);
        }

        let mats = stats.compute_materials();
        let ap = stats.compute_ap();
        if mats.total_pieces() > 0 || ap.total_bottles() > 0 {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mat_summary_title = match self.language {
                            AppLanguage::Vietnamese => "TỔNG HỢP VẬT LIỆU TRANG BỊ & ĐIỂM HÀNH ĐỘNG (AP):",
                            AppLanguage::English => "MATERIALS & ACTION POINTS (AP) SUMMARY:",
                        };
                        ui.label(
                            egui::RichText::new(mat_summary_title)
                                .font(egui::FontId::proportional(11.5))
                                .strong()
                                .color(c.focus_teal),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mat_total_str = match self.language {
                                AppLanguage::Vietnamese => format!(
                                    "Tổng: {} BT | {} NC => {} quy đổi | +{} AP",
                                    format_num(mats.total_normal()),
                                    format_num(mats.total_advanced()),
                                    format_num(mats.total_converted_normal()),
                                    format_num(ap.total_ap_points())
                                ),
                                AppLanguage::English => format!(
                                    "Total: {} Norm | {} Adv => {} conv | +{} AP",
                                    format_num(mats.total_normal()),
                                    format_num(mats.total_advanced()),
                                    format_num(mats.total_converted_normal()),
                                    format_num(ap.total_ap_points())
                                ),
                            };
                            ui.label(
                                egui::RichText::new(mat_total_str)
                                    .font(egui::FontId::proportional(12.5))
                                    .color(c.status_green)
                                    .strong(),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    let (conv_lbl, norm_lbl, adv_lbl) = match self.language {
                        AppLanguage::Vietnamese => ("quy đổi", "BT", "NC"),
                        AppLanguage::English => ("conv", "Norm", "Adv"),
                    };
                    let (leather_lbl, iron_lbl, ebony_lbl, bone_lbl, ap_lbl) = match self.language {
                        AppLanguage::Vietnamese => ("Da (Leather)", "Quặng Sắt (Iron)", "Gỗ Mun (Ebony)", "Xương (Bone)", "Điểm Hành Động"),
                        AppLanguage::English => ("Leather", "Iron Ore", "Ebony Wood", "Animal Bone", "Action Points"),
                    };
                    let pots_lbl = match self.language {
                        AppLanguage::Vietnamese => "bình",
                        AppLanguage::English => "pots",
                    };
                    let total_pots_lbl = match self.language {
                        AppLanguage::Vietnamese => "Tổng",
                        AppLanguage::English => "Total",
                    };
                    ui.columns(5, |cols| {
                        cols[0].vertical(|ui| {
                            ui.label(egui::RichText::new(leather_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.leather_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.train_orange));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.leather_normal), adv_lbl, format_num(mats.leather_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[1].vertical(|ui| {
                            ui.label(egui::RichText::new(iron_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.iron_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_iron));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.iron_normal), adv_lbl, format_num(mats.iron_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[2].vertical(|ui| {
                            ui.label(egui::RichText::new(ebony_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.ebony_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_ebony));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.ebony_normal), adv_lbl, format_num(mats.ebony_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[3].vertical(|ui| {
                            ui.label(egui::RichText::new(bone_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("{} {}", format_num(mats.bone_total_converted()), conv_lbl)).font(egui::FontId::proportional(14.0)).strong().color(c.mat_bone));
                            ui.label(egui::RichText::new(format!("{}: {} | {}: {}", norm_lbl, format_num(mats.bone_normal), adv_lbl, format_num(mats.bone_advanced))).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                        cols[4].vertical(|ui| {
                            ui.label(egui::RichText::new(ap_lbl).strong().color(c.canvas_tint));
                            ui.label(egui::RichText::new(format!("+{} AP", format_num(ap.total_ap_points()))).font(egui::FontId::proportional(14.0)).strong().color(c.status_green));
                            ui.label(egui::RichText::new(format!("{} {} {}", total_pots_lbl, format_num(ap.total_bottles()), pots_lbl)).font(egui::FontId::proportional(10.0)).color(c.text_muted));
                        });
                    });
                });

            ui.add_space(8.0);
        }

        let mut sorted_items: Vec<_> = stats.item_stats.values().collect();
        sorted_items.sort_by_key(|a| std::cmp::Reverse(a.total_quantity));

        let search_lower = self.item_search.trim().to_lowercase();
        let cat_filter = self.item_category_filter.clone();

        let mut filtered_items: Vec<_> = sorted_items
            .into_iter()
            .filter(|it| {
                let matches_search = if search_lower.is_empty() {
                    true
                } else {
                    it.name_vi.to_lowercase().contains(&search_lower)
                        || it.name_en.to_lowercase().contains(&search_lower)
                        || it.category.to_lowercase().contains(&search_lower)
                };

                let matches_cat = match cat_filter.as_str() {
                    "Tài Nguyên" => {
                        it.category == "Tài nguyên"
                            || it.category.to_lowercase().contains("nguyên")
                            || it.category.to_lowercase().contains("resource")
                            || it.name_vi.contains("Đá Quý")
                            || it.name_vi.contains("thực phẩm")
                            || it.name_vi.contains("khối gỗ")
                            || it.name_vi.contains("khối đá")
                            || it.name_vi.contains("đồng vàng")
                    }
                    "Vật Liệu" => {
                        it.category == "Vật liệu"
                            || it.category.to_lowercase().contains("vật liệu")
                            || it.category.to_lowercase().contains("material")
                            || it.name_vi.starts_with("Da")
                            || it.name_vi.starts_with("Quặng sắt")
                            || it.name_vi.starts_with("Gỗ mun")
                            || it.name_vi.starts_with("Xương")
                    }
                    "Tăng Tốc" => {
                        it.category == "Tăng tốc"
                            || it.category.to_lowercase().contains("tốc")
                            || it.category.to_lowercase().contains("speed")
                            || it.name_vi.contains("Tăng tốc")
                    }
                    "Pha Lê" => {
                        it.category == "Pha Lê"
                            || it.name_vi.contains("Pha Lê")
                            || it.name_en.contains("Crystal")
                    }
                    "Sách & Tên" => {
                        it.name_vi.contains("Sách")
                            || it.name_vi.contains("Tên")
                            || it.name_vi.contains("Mũi Tên")
                            || it.name_vi.contains("Tome")
                    }
                    "Khác" => {
                        let c_low = it.category.to_lowercase();
                        c_low != "tài nguyên" && c_low != "vật liệu" && c_low != "tăng tốc" && c_low != "pha lê"
                    }
                    _ => true,
                };

                matches_search && matches_cat
            })
            .collect();

        if let Some(col) = self.item_sort_col {
            filtered_items.sort_by(|a, b| {
                let ordering = match col {
                    ItemSortColumn::Name => a
                        .display_name(self.language)
                        .cmp(b.display_name(self.language)),
                    ItemSortColumn::Category => category_name(&a.category, self.language)
                        .cmp(category_name(&b.category, self.language)),
                    ItemSortColumn::TotalQty => a.total_quantity.cmp(&b.total_quantity),
                    ItemSortColumn::Converted => a.raw_converted_value().cmp(&b.raw_converted_value()),
                    ItemSortColumn::Battles => a.battle_count.cmp(&b.battle_count),
                    ItemSortColumn::DropRate => a.battle_count.cmp(&b.battle_count),
                    ItemSortColumn::AvgPerBattle => a.avg_per_battle.partial_cmp(&b.avg_per_battle).unwrap_or(std::cmp::Ordering::Equal),
                    ItemSortColumn::MaxInBattle => a.max_in_battle.cmp(&b.max_in_battle),
                };
                match self.item_sort_dir {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                }
            });
        }

        const INV_PAD_X: f32 = 8.0;
        const INV_SPACING_X: f32 = 6.0;

        const COL_CAT: f32 = 88.0;
        const COL_TOTAL: f32 = 78.0;
        const COL_CONV: f32 = 98.0;
        const COL_BATTLES: f32 = 58.0;
        const COL_RATE: f32 = 66.0;
        const COL_AVG: f32 = 62.0;
        const COL_MAX: f32 = 62.0;

        let avail_w = ui.available_width();
        let inv_fixed_w = COL_CAT
            + COL_TOTAL
            + COL_CONV
            + COL_BATTLES
            + COL_RATE
            + COL_AVG
            + COL_MAX
            + (INV_PAD_X * 2.0)
            + (INV_SPACING_X * 7.0);
        let min_name_w = 190.0_f32;
        let inv_table_width = avail_w.max(inv_fixed_w + min_name_w);
        let names_w = inv_table_width - inv_fixed_w;

        let mut toggle_item_sort_col = None;

        egui::Frame::none()
            .fill(c.card_surface)
            .stroke(Stroke::new(1.0_f32, c.hairline))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(INV_PAD_X, 5.0))
            .show(ui, |ui| {
                ui.set_width(inv_table_width - (INV_PAD_X * 2.0));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = INV_SPACING_X;
                    ui.set_min_height(24.0);

                    if render_item_sort_header_cell(
                        ui,
                        self.language.t(TrKey::InvColName),
                        ItemSortColumn::Name,
                        self.item_sort_col,
                        self.item_sort_dir,
                        names_w,
                        egui::Align::LEFT,
                        c.text_muted,
                        c.focus_teal,
                        self.language,
                    ) {
                        toggle_item_sort_col = Some(ItemSortColumn::Name);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColCategory), ItemSortColumn::Category, self.item_sort_col, self.item_sort_dir, COL_CAT, egui::Align::Center, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Category);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColTotalQty), ItemSortColumn::TotalQty, self.item_sort_col, self.item_sort_dir, COL_TOTAL, egui::Align::RIGHT, c.accent_gold, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::TotalQty);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColConverted), ItemSortColumn::Converted, self.item_sort_col, self.item_sort_dir, COL_CONV, egui::Align::RIGHT, c.status_green, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Converted);
                    }
                    if render_item_sort_header_cell(ui, self.language.t(TrKey::InvColBattles), ItemSortColumn::Battles, self.item_sort_col, self.item_sort_dir, COL_BATTLES, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::Battles);
                    }

                    let (rate_hdr, avg_hdr, max_hdr) = match self.language {
                        AppLanguage::Vietnamese => ("Tỷ Lệ Rơi", "TB / Trận", "Max / Trận"),
                        AppLanguage::English => ("Drop Rate", "Avg / Battle", "Max / Battle"),
                    };
                    if render_item_sort_header_cell(ui, rate_hdr, ItemSortColumn::DropRate, self.item_sort_col, self.item_sort_dir, COL_RATE, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::DropRate);
                    }
                    if render_item_sort_header_cell(ui, avg_hdr, ItemSortColumn::AvgPerBattle, self.item_sort_col, self.item_sort_dir, COL_AVG, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::AvgPerBattle);
                    }
                    if render_item_sort_header_cell(ui, max_hdr, ItemSortColumn::MaxInBattle, self.item_sort_col, self.item_sort_dir, COL_MAX, egui::Align::RIGHT, c.text_muted, c.focus_teal, self.language) {
                        toggle_item_sort_col = Some(ItemSortColumn::MaxInBattle);
                    }
                });
            });

        if let Some(col) = toggle_item_sort_col {
            if self.item_sort_col == Some(col) {
                self.item_sort_dir = match self.item_sort_dir {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            } else {
                self.item_sort_col = Some(col);
                self.item_sort_dir = SortDirection::Descending;
            }
        }

        ui.add_space(4.0);

        if filtered_items.is_empty() {
            egui::Frame::none()
                .fill(c.card_surface)
                .stroke(Stroke::new(1.0_f32, c.hairline))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(24.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(16.0);
                        let empty_title = match self.language {
                            AppLanguage::Vietnamese => "Không Tìm Thấy Vật Phẩm",
                            AppLanguage::English => "No Items Found",
                        };
                        ui.label(
                            egui::RichText::new(empty_title)
                                .font(egui::FontId::proportional(14.0))
                                .strong()
                                .color(c.canvas_tint),
                        );
                        ui.add_space(4.0);
                        let empty_desc = if !self.item_search.trim().is_empty() {
                            match self.language {
                                AppLanguage::Vietnamese => "Không có vật phẩm nào khớp với từ khóa tìm kiếm.",
                                AppLanguage::English => "No items matched your search query.",
                            }
                        } else {
                            match self.language {
                                AppLanguage::Vietnamese => "Chưa có dữ liệu vật phẩm nào được ghi nhận trong phiên này.",
                                AppLanguage::English => "No inventory items recorded yet.",
                            }
                        };
                        ui.label(
                            egui::RichText::new(empty_desc)
                                .font(egui::FontId::proportional(11.5))
                                .color(c.text_muted),
                        );
                        ui.add_space(16.0);
                    });
                });
            return;
        }

        let row_height = 29.0_f32;

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .id_salt("inventory_table_scroll")
            .show_rows(ui, row_height, filtered_items.len(), |ui, row_range| {
                ui.set_width(inv_table_width);
                for idx in row_range {
                    let it = filtered_items[idx];
                    let (row_rect, row_resp) = ui.allocate_exact_size(
                        egui::vec2(inv_table_width, 27.0),
                        egui::Sense::hover(),
                    );

                    let is_hovered = row_resp.hovered();
                    if is_hovered {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::same(4.0),
                            c.primary_pressed,
                        );
                        ui.painter().rect_stroke(
                            row_rect,
                            Rounding::same(4.0),
                            Stroke::new(1.0_f32, c.focus_teal),
                        );
                    } else if idx % 2 == 1 {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::same(4.0),
                            c.card_surface,
                        );
                    }

                    row_resp.on_hover_text(it.display_name(self.language));

                    let inner_rect = egui::Rect::from_min_max(
                        egui::pos2(row_rect.min.x + INV_PAD_X, row_rect.min.y),
                        egui::pos2(row_rect.max.x - INV_PAD_X, row_rect.max.y),
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = INV_SPACING_X;

                            ui.add_sized(
                                [names_w, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(it.display_name(self.language))
                                        .strong()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::LEFT),
                            );

                            let cat_color = category_badge_color(&it.category, &c);
                            egui::Frame::none()
                                .fill(cat_color.linear_multiply(0.12))
                                .stroke(Stroke::new(1.0_f32, cat_color.linear_multiply(0.35)))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(4.0, 1.0))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        [COL_CAT - 8.0, 18.0],
                                        egui::Label::new(
                                            egui::RichText::new(category_name(&it.category, self.language))
                                                .font(egui::FontId::proportional(10.5))
                                                .strong()
                                                .color(cat_color),
                                        )
                                        .truncate()
                                        .halign(egui::Align::Center),
                                    );
                                });

                            ui.add_sized(
                                [COL_TOTAL, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format_num(it.total_quantity))
                                        .monospace()
                                        .strong()
                                        .color(c.accent_gold),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );

                            let conv_str = get_item_converted_display_for_lang(it.itype, it.sub_type, it.total_quantity, self.language).unwrap_or_else(|| "-".to_string());
                            let conv_color = if conv_str != "-" { c.status_green } else { c.text_muted };
                            ui.add_sized(
                                [COL_CONV, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(conv_str)
                                        .monospace()
                                        .strong()
                                        .color(conv_color),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );

                            let drop_rate = (it.battle_count as f64 / total_battles as f64) * 100.0;
                            ui.add_sized(
                                [COL_BATTLES, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{}", it.battle_count))
                                        .monospace()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_RATE, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:.1}%", drop_rate))
                                        .monospace()
                                        .strong()
                                        .color(c.focus_teal),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_AVG, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:.1}", it.avg_per_battle))
                                        .monospace()
                                        .color(c.text_muted),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                            ui.add_sized(
                                [COL_MAX, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(format_num(it.max_in_battle))
                                        .monospace()
                                        .color(c.canvas_tint),
                                )
                                .truncate()
                                .halign(egui::Align::RIGHT),
                            );
                        });
                    });

                    ui.add_space(2.0);
                }
            });
    }
}
