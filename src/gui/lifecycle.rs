use eframe::egui;

use crate::analyzer::parse_date_str;
use crate::model::{AppConfig, AppLanguage, BattleRecord, DateFilterPreset};
use crate::watcher::{RoKMailWatcher, WatcherUiEvent};
use super::components::poll_cloud_receiver;
use super::RoKLootWatcherApp;

pub fn perform_exit_cloud_backup(watcher: &RoKMailWatcher) {
    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    if watcher.send_sync_command(crate::cloud_sync::SyncCommand::Shutdown(ack_tx)) {
        let _ = ack_rx.recv_timeout(std::time::Duration::from_secs(2));
    }
}

fn sum_item_in_record(rec: &BattleRecord, canonical_name: &str, itype: u32, isub: u32) -> i64 {
    let mut total: i64 = 0;

    for (k, v) in &rec.item_totals {
        let is_match = match canonical_name {
            "Gems" => {
                k.eq_ignore_ascii_case("Gems") || k.eq_ignore_ascii_case("Đá quý")
            }
            "Crystals" => {
                k.eq_ignore_ascii_case("Crystals")
                    || k.eq_ignore_ascii_case("Crystal")
                    || k.eq_ignore_ascii_case("Pha Lê")
                    || k.eq_ignore_ascii_case("Pha lê")
            }
            "Book of Covenant" => {
                k.eq_ignore_ascii_case("Book of Covenant")
                    || k.eq_ignore_ascii_case("Sách giao ước")
                    || k.eq_ignore_ascii_case("Sách Giao Ước")
                    || k.eq_ignore_ascii_case("Sách ước nguyện")
                    || k.eq_ignore_ascii_case("Sách Ước Nguyện")
            }
            "Arrow of Resistance" => {
                k.eq_ignore_ascii_case("Arrow of Resistance")
                    || k.eq_ignore_ascii_case("Mũi tên chống trả")
                    || k.eq_ignore_ascii_case("Mũi Tên Chống Trả")
                    || k.eq_ignore_ascii_case("Mũi tên kháng cự")
                    || k.eq_ignore_ascii_case("Mũi Tên Kháng Cự")
            }
            _ => k.eq_ignore_ascii_case(canonical_name),
        };
        if is_match {
            total += v;
        }
    }

    if total > 0 {
        return total;
    }

    if !rec.items.is_empty() {
        let items_sum: i64 = rec
            .items
            .iter()
            .filter_map(|it| {
                let is_match = match canonical_name {
                    "Gems" => {
                        (it.itype == 1 && (it.sub_type == 5 || it.sub_type == isub))
                            || it.name_vi.eq_ignore_ascii_case("Đá quý")
                            || it.name_en.eq_ignore_ascii_case("Gems")
                    }
                    "Crystals" => {
                        (it.itype == 1 && (it.sub_type == 9 || it.sub_type == isub))
                            || it.name_vi.eq_ignore_ascii_case("Pha Lê")
                            || it.name_vi.eq_ignore_ascii_case("Pha lê")
                            || it.name_en.eq_ignore_ascii_case("Crystals")
                            || it.name_en.eq_ignore_ascii_case("Crystal")
                    }
                    "Book of Covenant" => {
                        (it.itype == 2 && (it.sub_type == 109 || it.sub_type == isub))
                            || it.name_vi.eq_ignore_ascii_case("Sách giao ước")
                            || it.name_vi.eq_ignore_ascii_case("Sách ước nguyện")
                            || it.name_en.eq_ignore_ascii_case("Book of Covenant")
                    }
                    "Arrow of Resistance" => {
                        (it.itype == 2 && (it.sub_type == 145 || it.sub_type == isub))
                            || it.name_vi.eq_ignore_ascii_case("Mũi tên chống trả")
                            || it.name_en.eq_ignore_ascii_case("Arrow of Resistance")
                    }
                    _ => {
                        (it.itype == itype && it.sub_type == isub)
                            || it.name_vi.eq_ignore_ascii_case(canonical_name)
                            || it.name_en.eq_ignore_ascii_case(canonical_name)
                    }
                };
                if is_match {
                    Some(it.value)
                } else {
                    None
                }
            })
            .sum();

        if items_sum > 0 {
            return items_sum;
        }
    }

    if canonical_name == "Gems" {
        let gems = rec.compute_gems();
        if gems > 0 {
            return gems;
        }
    }

    if canonical_name == "Crystals" {
        let crystals = rec.item_quantity(1, 9);
        if crystals > 0 {
            return crystals;
        }
    }

    0
}

impl RoKLootWatcherApp {
    pub fn calculate_today_pinned_chips(
        &self,
        records: &[BattleRecord],
    ) -> ((usize, usize, usize), Vec<crate::overlay_ui::CachedPinnedChip>) {
        let mut fort_count = 0;
        let mut motte_count = 0;
        let mut barb_count = 0;

        for rec in records {
            match rec.battle_type.as_str() {
                "fort" => fort_count += 1,
                "motte" => motte_count += 1,
                "barbarian" => barb_count += 1,
                _ => {}
            }
        }

        let c = self.colors();
        let mut chips = Vec::with_capacity(self.overlay_config.pinned_items.len());

        for item_key in &self.overlay_config.pinned_items {
            match item_key {
                crate::model::PinnedItemKey::SpeedupUniversal => {
                    let total_mins: i64 = records.iter().map(|r| r.univ_speedup_minutes()).sum();
                    chips.push(crate::overlay_ui::CachedPinnedChip {
                        key: item_key.clone(),
                        label: "SPEEDUP".to_string(),
                        display_val: crate::overlay_ui::format_speedup_compact(total_mins),
                        text_color: c.speedup_blue,
                    });
                }
                crate::model::PinnedItemKey::SpeedupTraining => {
                    let total_mins: i64 = records.iter().map(|r| r.training_speedup_minutes()).sum();
                    chips.push(crate::overlay_ui::CachedPinnedChip {
                        key: item_key.clone(),
                        label: "TRAIN".to_string(),
                        display_val: crate::overlay_ui::format_speedup_compact(total_mins),
                        text_color: c.train_orange,
                    });
                }
                crate::model::PinnedItemKey::SpeedupHealing => {
                    let total_mins: i64 = records.iter().map(|r| r.heal_speedup_minutes()).sum();
                    chips.push(crate::overlay_ui::CachedPinnedChip {
                        key: item_key.clone(),
                        label: "HEAL".to_string(),
                        display_val: crate::overlay_ui::format_speedup_compact(total_mins),
                        text_color: c.heal_blue,
                    });
                }
                crate::model::PinnedItemKey::CatalogItem { canonical_name, itype, isub } => {
                    let total: i64 = records
                        .iter()
                        .map(|r| sum_item_in_record(r, canonical_name, *itype, *isub))
                        .sum();

                    let label = match canonical_name.as_str() {
                        "Gems" => "GEMS".to_string(),
                        "Crystals" => "CRYSTALS".to_string(),
                        "Book of Covenant" => "BOOKS".to_string(),
                        "Arrow of Resistance" => "ARROWS".to_string(),
                        _ => {
                            let upper = canonical_name.to_uppercase();
                            upper.chars().take(8).collect()
                        }
                    };

                    let text_color = match canonical_name.as_str() {
                        "Gems" | "Crystals" => c.crystal_cyan,
                        "Book of Covenant" => c.accent_gold,
                        "Arrow of Resistance" => c.train_orange,
                        _ => c.canvas_tint,
                    };

                    chips.push(crate::overlay_ui::CachedPinnedChip {
                        key: item_key.clone(),
                        label,
                        display_val: crate::model::format_num(total),
                        text_color,
                    });
                }
            }
        }

        ((fort_count, motte_count, barb_count), chips)
    }

    pub fn refresh_cached_overlay_records(&mut self) {
        self.cached_overlay_records = self.watcher.get_latest_vault_records(self.overlay_config.filter, 5);
        let today_recs = self.watcher.get_today_vault_records(self.overlay_config.filter);
        let (run_counts, chips) = self.calculate_today_pinned_chips(&today_recs);
        self.cached_today_run_counts = run_counts;
        self.cached_today_pinned_chips = chips;
    }

    pub(crate) fn save_config(&self) {
        let config = AppConfig {
            close_behavior: self.close_behavior,
            speedup_unit: self.speedup_unit,
            mail_dir: Some(self.watcher.get_mail_dir()),
            mode: Some(self.selected_mode),
            theme: self.theme,
            language: self.language,
            autostart: self.autostart,
            autostart_minimized: self.autostart_minimized,
            overlay: self.overlay_config.clone(),
            cloud_auth: self.cloud_auth.clone(),
            main_account_uids: self.main_account_uids.clone(),
        };
        config.save();
    }

    pub fn perform_exit_cloud_backup(&self) {
        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        if let Some(worker) = &self.sync_worker {
            let _ = worker.tx.send(crate::cloud_sync::SyncCommand::Shutdown(ack_tx));
            let _ = ack_rx.recv_timeout(std::time::Duration::from_secs(2));
        } else {
            perform_exit_cloud_backup(&self.watcher);
        }
    }

    pub fn quit_app(&self) {
        self.watcher.flush_vault_if_dirty();
        self.perform_exit_cloud_backup();
        self.watcher.stop();
        if let Some(ref t) = self.tray {
            t.hide_icon();
        }
        std::process::exit(0);
    }

    pub(crate) fn hide_to_tray(&self, _ctx: &egui::Context) {
        crate::tray::hide_main_window();
        let msg = match self.language {
            AppLanguage::Vietnamese => "Ứng dụng đã thu nhỏ xuống Khay hệ thống (System Tray). Nhấp vào biểu tượng ở khay để mở lại bất cứ lúc nào.",
            AppLanguage::English => "Application minimized to System Tray. Click the tray icon to restore at any time.",
        };
        self.watcher.log("INFO", msg);
    }

    pub(crate) fn poll_watcher_events(&mut self, ctx: &egui::Context) -> bool {
        let mut had_events = false;
        while let Ok(evt) = self.rx_ui.try_recv() {
            had_events = true;
            match evt {
                WatcherUiEvent::Log { level: _, message } => {
                    self.console_logs.push_back(message);
                    if self.console_logs.len() > 1000 {
                        self.console_logs.pop_front();
                    }
                }
                WatcherUiEvent::StatusChanged(status) => {
                    self.watcher_status = status;
                    ctx.request_repaint();
                }
                WatcherUiEvent::StatsUpdated { .. } => {
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    ctx.request_repaint();
                }
                WatcherUiEvent::MailDetected { is_new, .. } => {
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    self.last_mail_arrival = std::time::Instant::now();
                    if is_new {
                        self.has_unbacked_data = true;
                        self.watcher.mark_cloud_dirty();
                    }
                    ctx.request_repaint();
                }
            }
        }
        had_events
    }

    pub(crate) fn poll_cloud_events(&mut self, ctx: &egui::Context) {
        if let Some(res) = poll_cloud_receiver(&mut self.cloud_rx, &mut self.cloud_busy) {
            match res {
                Ok(Ok(auth)) => {
                    let email = auth.user.email.clone();
                    self.cloud_auth.access_token = Some(auth.access_token);
                    self.cloud_auth.refresh_token = Some(auth.refresh_token);
                    self.cloud_auth.user_id = Some(auth.user.id);
                    self.cloud_auth.user_email = Some(auth.user.email);
                    self.save_config();
                    if let Some(ref worker) = self.sync_worker {
                        let _ = worker.tx.send(crate::cloud_sync::SyncCommand::UpdateConfig(self.cloud_auth.clone()));
                        let _ = worker.tx.send(crate::cloud_sync::SyncCommand::TriggerDelta);
                    }
                    let msg = match self.language {
                        AppLanguage::Vietnamese => format!("Đăng nhập Google thành công: {}", email),
                        AppLanguage::English => format!("Google login successful: {}", email),
                    };
                    self.watcher.log("SUCCESS", &msg);
                }
                Ok(Err(e)) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => format!("Đăng nhập Google thất bại: {}", e),
                        AppLanguage::English => format!("Google login failed: {}", e),
                    };
                    self.watcher.log("ERROR", &msg);
                }
                Err(()) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => "Đăng nhập Google thất bại: luồng xác thực bị ngắt kết nối".to_string(),
                        AppLanguage::English => "Google login failed: auth thread disconnected".to_string(),
                    };
                    self.watcher.log("ERROR", &msg);
                }
            }
        }

        if let Some(res) = poll_cloud_receiver(&mut self.restore_rx, &mut self.cloud_busy) {
            match res {
                Ok(Ok(cnt)) => {
                    self.watcher.recompute_filtered_records_and_stats();
                    self.reports_sort_dirty = true;
                    self.ap_econ_dirty = true;
                    self.refresh_cached_overlay_records();
                    let success_msg = match self.language {
                        AppLanguage::Vietnamese => format!("Khôi phục thành công {} chiến báo từ Cloud!", cnt),
                        AppLanguage::English => format!("Successfully restored {} reports from Cloud!", cnt),
                    };
                    self.watcher.log("SUCCESS", &success_msg);
                }
                Ok(Err(e)) => {
                    self.watcher.log("ERROR", &e);
                }
                Err(()) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => "Khôi phục từ Cloud thất bại: luồng xử lý bị đóng bất ngờ".to_string(),
                        AppLanguage::English => "Restore from Cloud failed: worker thread disconnected".to_string(),
                    };
                    self.watcher.log("ERROR", &msg);
                }
            }
        }

        if let Some(res) = poll_cloud_receiver(&mut self.sync_rx, &mut self.cloud_busy) {
            match res {
                Ok(Ok(cnt)) => {
                    let success_msg = match self.language {
                        AppLanguage::Vietnamese => {
                            if cnt > 0 {
                                format!("Đồng bộ thành công {} bản ghi lên Cloud!", cnt)
                            } else {
                                "Dữ liệu đám mây đã đồng bộ mới nhất!".to_string()
                            }
                        }
                        AppLanguage::English => {
                            if cnt > 0 {
                                format!("Successfully synced {} records to Cloud!", cnt)
                            } else {
                                "Cloud data is already up to date!".to_string()
                            }
                        }
                    };
                    self.watcher.log("SUCCESS", &success_msg);
                }
                Ok(Err(e)) => {
                    let err_msg = match self.language {
                        AppLanguage::Vietnamese => format!("Đồng bộ lên Cloud thất bại: {}", e),
                        AppLanguage::English => format!("Cloud sync failed: {}", e),
                    };
                    self.watcher.log("ERROR", &err_msg);
                }
                Err(()) => {
                    let msg = match self.language {
                        AppLanguage::Vietnamese => "Đồng bộ lên Cloud thất bại: luồng xử lý bị đóng bất ngờ".to_string(),
                        AppLanguage::English => "Cloud sync failed: worker thread disconnected".to_string(),
                    };
                    self.watcher.log("ERROR", &msg);
                }
            }
        }

        if let Some(ref worker) = self.sync_worker {
            let mut need_save = false;
            let has_new_tokens = worker.state.read().map(|s| s.new_tokens.is_some()).unwrap_or(false);
            if has_new_tokens {
                if let Ok(mut state) = worker.state.write() {
                    if let Some((acc, ref_tok)) = state.new_tokens.take() {
                        self.cloud_auth.access_token = Some(acc);
                        self.cloud_auth.refresh_token = Some(ref_tok);
                        need_save = true;
                    }
                }
            }

            if let Ok(state) = worker.state.read() {
                if let Some(ref t) = state.last_sync_time {
                    if self.cloud_auth.last_synced.as_deref() != Some(t.as_str()) {
                        self.cloud_auth.last_synced = Some(t.clone());
                        need_save = true;
                    }
                }
                if state.is_syncing || state.is_restoring || state.retry_in_secs.is_some() || self.sync_rx.is_some() {
                    ctx.request_repaint_after(std::time::Duration::from_millis(250));
                }
            }

            if need_save {
                self.save_config();
            }
        }
    }

    pub(crate) fn get_cloud_credentials(&self) -> (String, String) {
        self.cloud_auth.get_credentials()
    }

    pub(crate) fn start_google_login(&mut self) {
        if self.cloud_busy {
            return;
        }

        let (url, key) = self.get_cloud_credentials();
        let lang = self.language;

        if url.is_empty() || key.is_empty() {
            let err_msg = match lang {
                AppLanguage::Vietnamese => "Chưa cấu hình Supabase URL / Anon Key (cần thiết lập trong rok_config.json hoặc biến môi trường build ROK_SUPABASE_URL).",
                AppLanguage::English => "Supabase URL / Anon Key not configured (set in rok_config.json or compile with ROK_SUPABASE_URL).",
            };
            self.watcher.log("ERROR", err_msg);
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.cloud_rx = Some(rx);
        self.cloud_busy = true;

        let start_msg = match lang {
            AppLanguage::Vietnamese => "Đang mở trình duyệt để đăng nhập Google qua Supabase...",
            AppLanguage::English => "Opening browser for Google login via Supabase...",
        };
        self.watcher.log("INFO", start_msg);

        let _ = std::thread::Builder::new()
            .name("rok-google-login".into())
            .spawn(move || {
                let res = crate::cloud_sync::start_google_login(&url, &key);
                let _ = tx.send(res);
            });
    }

    pub(crate) fn trigger_cloud_backup(&mut self) {
        if self.cloud_busy {
            return;
        }

        if let Some(ref worker) = self.sync_worker {
            let (tx, rx) = std::sync::mpsc::channel();
            if worker.tx.send(crate::cloud_sync::SyncCommand::SyncNow(Some(tx))).is_ok() {
                self.sync_rx = Some(rx);
                self.cloud_busy = true;

                let msg = match self.language {
                    AppLanguage::Vietnamese => "Đang gửi yêu cầu đồng bộ ngay lên Cloud...",
                    AppLanguage::English => "Requested immediate cloud sync...",
                };
                self.watcher.log("INFO", msg);
            }
        }
    }

    pub(crate) fn trigger_cloud_restore(&mut self) {
        if self.cloud_busy {
            return;
        }

        if let Some(ref worker) = self.sync_worker {
            let (tx, rx) = std::sync::mpsc::channel();
            if worker.tx.send(crate::cloud_sync::SyncCommand::Restore(tx)).is_ok() {
                self.restore_rx = Some(rx);
                self.cloud_busy = true;

                let lang = self.language;
                let msg = match lang {
                    AppLanguage::Vietnamese => "Bắt đầu khôi phục dữ liệu từ Cloud...",
                    AppLanguage::English => "Starting restore from Cloud...",
                };
                self.watcher.log("INFO", msg);
            }
        }
    }

    pub(crate) fn toggle_watcher(&self) {
        if self.watcher.is_watching() {
            self.watcher.stop();
        } else {
            self.watcher.start(false);
        }
    }

    pub(crate) fn apply_date_preset(&mut self, preset: DateFilterPreset) {
        self.reports_sort_dirty = true;
        self.ap_econ_dirty = true;
        self.selected_date_preset = preset.clone();
        let today_vn = crate::analyzer::current_date_vn();
        match preset {
            DateFilterPreset::AllTime => {
                self.watcher.set_date_filter(None, None, None);
            }
            DateFilterPreset::Today => {
                self.watcher.set_date_filter(None, None, Some(1));
            }
            DateFilterPreset::Yesterday => {
                let yest_vn = today_vn - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));
                self.watcher.set_date_filter(Some(yest_vn), Some(yest_vn), None);
            }
            DateFilterPreset::Last3Days => {
                self.watcher.set_date_filter(None, None, Some(3));
            }
            DateFilterPreset::Last7Days => {
                self.watcher.set_date_filter(None, None, Some(7));
            }
            DateFilterPreset::Last14Days => {
                self.watcher.set_date_filter(None, None, Some(14));
            }
            DateFilterPreset::Last30Days => {
                self.watcher.set_date_filter(None, None, Some(30));
            }
            DateFilterPreset::Custom => {
                if self.date_from_input.is_empty() {
                    let start = today_vn - chrono::Duration::try_days(6).unwrap_or_else(|| chrono::Duration::days(6));
                    self.date_from_input = start.to_string();
                }
                if self.date_to_input.is_empty() {
                    self.date_to_input = today_vn.to_string();
                }
                self.apply_custom_date_filter();
            }
        }
    }

    pub(crate) fn apply_custom_date_filter(&mut self) {
        self.reports_sort_dirty = true;
        self.ap_econ_dirty = true;
        let s = parse_date_str(&self.date_from_input);
        let e = parse_date_str(&self.date_to_input);
        self.selected_date_preset = DateFilterPreset::Custom;
        self.watcher.set_date_filter(s, e, None);
    }

    pub(crate) fn export_active_csv(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("rok_battles.csv")
            .add_filter("CSV File", &["csv"])
            .save_file()
        {
            let records = self.watcher.get_active_records();
            let w = crate::exporters::generate_csv(&records, self.language);
            if let Err(e) = std::fs::write(&path, w) {
                let err_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Không thể lưu file CSV: {}", e),
                    AppLanguage::English => format!("Failed to save CSV file: {}", e),
                };
                self.watcher.log("ERROR", &err_msg);
            } else {
                let ok_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Đã xuất {} trận đánh ra file CSV: {}", records.len(), path.display()),
                    AppLanguage::English => format!("Exported {} battle records to CSV: {}", records.len(), path.display()),
                };
                self.watcher.log("SUCCESS", &ok_msg);
            }
        }
    }

    pub(crate) fn export_active_json(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("rok_battles.json")
            .add_filter("JSON File", &["json"])
            .save_file()
        {
            let records = self.watcher.get_active_records();
            let stats = self.watcher.get_stats();
            if let Err(e) = crate::exporters::export_json(&records, &stats, &path) {
                let err_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Không thể lưu file JSON: {}", e),
                    AppLanguage::English => format!("Failed to save JSON file: {}", e),
                };
                self.watcher.log("ERROR", &err_msg);
            } else {
                let ok_msg = match self.language {
                    AppLanguage::Vietnamese => format!("Đã xuất {} trận đánh ra file JSON: {}", records.len(), path.display()),
                    AppLanguage::English => format!("Exported {} battle records to JSON: {}", records.len(), path.display()),
                };
                self.watcher.log("SUCCESS", &ok_msg);
            }
        }
    }
}
