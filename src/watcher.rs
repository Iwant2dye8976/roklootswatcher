use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use chrono::NaiveDate;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::analyzer::{filter_records_by_date, generate_statistics};
use crate::classifier::{classify_mail, extract_motte_details};
use crate::i18n::filter_mode_name;
use crate::lilith_stream::decode_mail_file;
use crate::loot_catalog::LootCatalog;
use crate::model::{AppLanguage, BattleRecord, FilterMode, Statistics};
use crate::vault::{read_file_shared, MailVault};

pub const DEFAULT_MAIL_DIR: &str =
    r"C:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache";

pub fn detect_default_mail_dir() -> PathBuf {
    if let Ok(env_val) = std::env::var("ROK_MAIL_DIR") {
        let p = PathBuf::from(env_val);
        if p.is_dir() {
            return p;
        }
    }
    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache"),
        PathBuf::from(r"D:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache"),
        PathBuf::from(r"D:\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache"),
        PathBuf::from(r"E:\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache"),
    ];
    for c in candidates {
        if c.is_dir() {
            return c;
        }
    }
    PathBuf::from(DEFAULT_MAIL_DIR)
}

#[derive(Debug, Clone)]
pub enum WatcherStatus {
    Running(String),
    Scanning(String),
    Stopped(String),
    Error(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WatcherUiEvent {
    Log {
        level: String,
        message: String,
    },
    StatusChanged(WatcherStatus),
    StatsUpdated {
        stats: Statistics,
        active_count: usize,
    },
    MailDetected {
        target: String,
        damage_pct: i32,
        tier: i32,
        is_new: bool,
        loot_summary: String,
    },
}

pub struct RoKMailWatcher {
    mail_dir: Arc<RwLock<PathBuf>>,
    out_dir: Arc<RwLock<PathBuf>>,
    mode: Arc<RwLock<FilterMode>>,
    language: Arc<RwLock<AppLanguage>>,

    start_date: Arc<RwLock<Option<NaiveDate>>>,
    end_date: Arc<RwLock<Option<NaiveDate>>>,
    days: Arc<RwLock<Option<u32>>>,

    active_records: Arc<RwLock<Arc<Vec<BattleRecord>>>>,
    stats: Arc<RwLock<Statistics>>,
    catalog: Arc<LootCatalog>,
    vault: Arc<RwLock<MailVault>>,

    vault_dirty: Arc<AtomicBool>,
    last_vault_mutation: Arc<Mutex<Instant>>,

    is_watching: Arc<RwLock<bool>>,
    is_scanning: Arc<RwLock<bool>>,

    ui_sender: Sender<WatcherUiEvent>,
    stop_signal: Arc<Mutex<Option<Sender<()>>>>,
}

impl RoKMailWatcher {
    pub fn new(
        mail_dir: Option<PathBuf>,
        out_dir: Option<PathBuf>,
        mode: FilterMode,
        catalog: LootCatalog,
        ui_sender: Sender<WatcherUiEvent>,
    ) -> Self {
        let m_dir = mail_dir.unwrap_or_else(detect_default_mail_dir);
        let o_dir = out_dir.unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        // Khởi tạo Persistent Vault: tự động nạp lịch sử đã lưu trước đó
        let vault = MailVault::new(&o_dir);
        let watcher = Self {
            mail_dir: Arc::new(RwLock::new(m_dir)),
            out_dir: Arc::new(RwLock::new(o_dir)),
            mode: Arc::new(RwLock::new(mode)),
            language: Arc::new(RwLock::new(AppLanguage::default())),

            start_date: Arc::new(RwLock::new(None)),
            end_date: Arc::new(RwLock::new(None)),
            days: Arc::new(RwLock::new(None)),

            active_records: Arc::new(RwLock::new(Arc::new(Vec::new()))),
            stats: Arc::new(RwLock::new(Statistics::default())),
            catalog: Arc::new(catalog),
            vault: Arc::new(RwLock::new(vault)),

            vault_dirty: Arc::new(AtomicBool::new(false)),
            last_vault_mutation: Arc::new(Mutex::new(Instant::now())),

            is_watching: Arc::new(RwLock::new(false)),
            is_scanning: Arc::new(RwLock::new(false)),

            ui_sender,
            stop_signal: Arc::new(Mutex::new(None)),
        };

        // Tính toán thống kê ban đầu từ Vault nếu có
        watcher.recompute_filtered_records_and_stats();
        watcher
    }

    pub fn set_language(&self, lang: AppLanguage) {
        *self.language.write().unwrap_or_else(|e| e.into_inner()) = lang;
    }

    pub fn get_language(&self) -> AppLanguage {
        *self.language.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get_vault_total_count(&self) -> usize {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).total_count()
    }

    pub fn get_vault_file_path(&self) -> PathBuf {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).get_vault_file().to_path_buf()
    }

    pub fn get_vault_binary_dir(&self) -> PathBuf {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).get_binary_dir().to_path_buf()
    }

    pub fn get_mail_dir(&self) -> PathBuf {
        self.mail_dir.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    #[allow(dead_code)]
    pub fn get_out_dir(&self) -> PathBuf {
        self.out_dir.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_mode(&self) -> FilterMode {
        *self.mode.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn is_watching(&self) -> bool {
        *self.is_watching.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn is_scanning(&self) -> bool {
        *self.is_scanning.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn log(&self, level: &str, msg: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let formatted = format!("[{}] [{}] {}", timestamp, level, msg);
        println!("{}", formatted);
        let _ = self.ui_sender.send(WatcherUiEvent::Log {
            level: level.to_string(),
            message: formatted,
        });
    }

    pub fn set_status(&self, status: WatcherStatus) {
        let _ = self.ui_sender.send(WatcherUiEvent::StatusChanged(status));
    }

    pub fn set_mail_dir(&self, new_dir: PathBuf) {
        let was_watching = self.is_watching();
        if was_watching {
            self.stop();
        }
        *self.mail_dir.write().unwrap_or_else(|e| e.into_inner()) = new_dir.clone();
        let msg = match self.get_language() {
            AppLanguage::Vietnamese => format!("Đã chuyển thư mục theo dõi sang: {}", new_dir.display()),
            AppLanguage::English => format!("Switched watch directory to: {}", new_dir.display()),
        };
        self.log("INFO", &msg);
        self.start(true);
    }

    pub fn recompute_filtered_records_and_stats(&self) -> (usize, usize) {
        let vault = self.vault.read().unwrap_or_else(|e| e.into_inner());
        let all = vault.get_records();
        let mode = self.get_mode();
        let total_mode_count = all
            .iter()
            .filter(|r| mode.matches(&r.battle_type))
            .count();

        let s = *self.start_date.read().unwrap_or_else(|e| e.into_inner());
        let e = *self.end_date.read().unwrap_or_else(|e| e.into_inner());
        let d = *self.days.read().unwrap_or_else(|e| e.into_inner());

        let active_vec = if s.is_none() && e.is_none() && d.is_none() {
            all.iter()
                .filter(|r| mode.matches(&r.battle_type))
                .cloned()
                .collect()
        } else {
            filter_records_by_date(
                all.iter().filter(|r| mode.matches(&r.battle_type)),
                s,
                e,
                d,
            )
        };

        let active_cnt = active_vec.len();
        let new_stats = generate_statistics(&active_vec);

        drop(vault);
        *self.active_records.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(active_vec);
        *self.stats.write().unwrap_or_else(|e| e.into_inner()) = new_stats.clone();

        let _ = self.ui_sender.send(WatcherUiEvent::StatsUpdated {
            stats: new_stats,
            active_count: active_cnt,
        });

        (total_mode_count, active_cnt)
    }

    pub fn set_mode(&self, new_mode: FilterMode) {
        *self.mode.write().unwrap_or_else(|e| e.into_inner()) = new_mode;
        let lang = self.get_language();
        let mode_lbl = filter_mode_name(&new_mode, lang);
        let msg1 = match lang {
            AppLanguage::Vietnamese => format!("Đã chuyển chế độ mục tiêu: {}", mode_lbl),
            AppLanguage::English => format!("Switched target filter mode: {}", mode_lbl),
        };
        self.log("INFO", &msg1);
        let (total_mode, active_cnt) = self.recompute_filtered_records_and_stats();
        let msg2 = match lang {
            AppLanguage::Vietnamese => format!(
                "Bộ lọc cập nhật tức thì: Tìm thấy {} trận ({} trận khớp ngày).",
                total_mode, active_cnt
            ),
            AppLanguage::English => format!(
                "Filter updated immediately: Found {} records ({} matching date).",
                total_mode, active_cnt
            ),
        };
        self.log("SUCCESS", &msg2);
    }

    pub fn get_active_records(&self) -> Arc<Vec<BattleRecord>> {
        Arc::clone(&*self.active_records.read().unwrap_or_else(|e| e.into_inner()))
    }

    pub fn get_active_count(&self) -> usize {
        self.active_records.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn mark_vault_dirty(&self) {
        self.vault_dirty.store(true, Ordering::SeqCst);
        if let Ok(mut t) = self.last_vault_mutation.lock() {
            *t = Instant::now();
        }
    }

    pub fn flush_vault_if_dirty(&self) {
        if self.vault_dirty.swap(false, Ordering::SeqCst) {
            let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
            let _ = v.save_to_disk();
        }
    }

    pub fn check_and_flush_vault_debounce(&self, debounce_delay: Duration) {
        if self.vault_dirty.load(Ordering::Relaxed) {
            let should_flush = if let Ok(t) = self.last_vault_mutation.lock() {
                t.elapsed() >= debounce_delay
            } else {
                false
            };

            if should_flush {
                self.flush_vault_if_dirty();
            }
        }
    }

    pub fn get_stats(&self) -> Statistics {
        self.stats.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_date_range_label(&self) -> String {
        let s = *self.start_date.read().unwrap_or_else(|e| e.into_inner());
        let e = *self.end_date.read().unwrap_or_else(|e| e.into_inner());
        let d = *self.days.read().unwrap_or_else(|e| e.into_inner());
        let lang = self.get_language();

        let today_vn = crate::analyzer::current_date_vn();
        let yest_vn = today_vn - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));

        match (s, e) {
            (Some(s_d), Some(e_d)) => {
                if s_d == e_d {
                    if s_d == today_vn {
                        match lang {
                            AppLanguage::Vietnamese => format!("Hôm nay ({})", s_d),
                            AppLanguage::English => format!("Today ({})", s_d),
                        }
                    } else if s_d == yest_vn {
                        match lang {
                            AppLanguage::Vietnamese => format!("Hôm qua ({})", s_d),
                            AppLanguage::English => format!("Yesterday ({})", s_d),
                        }
                    } else {
                        match lang {
                            AppLanguage::Vietnamese => format!("Ngày {}", s_d),
                            AppLanguage::English => format!("Date {}", s_d),
                        }
                    }
                } else {
                    match lang {
                        AppLanguage::Vietnamese => format!("Từ {} đến {}", s_d, e_d),
                        AppLanguage::English => format!("From {} to {}", s_d, e_d),
                    }
                }
            }
            (Some(s_d), None) => match lang {
                AppLanguage::Vietnamese => format!("Từ {} đến nay", s_d),
                AppLanguage::English => format!("From {} to present", s_d),
            },
            (None, Some(e_d)) => match lang {
                AppLanguage::Vietnamese => format!("Đến ngày {}", e_d),
                AppLanguage::English => format!("Until date {}", e_d),
            },
            (None, None) => {
                if let Some(days) = d {
                    if days == 1 {
                        match lang {
                            AppLanguage::Vietnamese => format!("Hôm nay ({})", today_vn),
                            AppLanguage::English => format!("Today ({})", today_vn),
                        }
                    } else {
                        match lang {
                            AppLanguage::Vietnamese => format!("{} ngày gần nhất", days),
                            AppLanguage::English => format!("Last {} days", days),
                        }
                    }
                } else {
                    match lang {
                        AppLanguage::Vietnamese => "Tất cả thời gian".to_string(),
                        AppLanguage::English => "All time".to_string(),
                    }
                }
            }
        }
    }

    pub fn set_date_filter(
        &self,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        days: Option<u32>,
    ) {
        *self.start_date.write().unwrap_or_else(|e| e.into_inner()) = start_date;
        *self.end_date.write().unwrap_or_else(|e| e.into_inner()) = end_date;
        *self.days.write().unwrap_or_else(|e| e.into_inner()) = days;

        let (_, active_cnt) = self.recompute_filtered_records_and_stats();

        let msg = match self.get_language() {
            AppLanguage::Vietnamese => format!(
                "Đã áp dụng lọc ngày: {} ({} trận)",
                self.get_date_range_label(),
                active_cnt
            ),
            AppLanguage::English => format!(
                "Applied date filter: {} ({} records)",
                self.get_date_range_label(),
                active_cnt
            ),
        };
        self.log("INFO", &msg);
    }

    pub fn start(&self, initial_scan: bool) {
        if *self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            return;
        }

        let dir = self.get_mail_dir();
        if !dir.is_dir() {
            let (warn_msg, status_msg) = match self.get_language() {
                AppLanguage::Vietnamese => (
                    format!("Thư mục mailcache không tồn tại: {}", dir.display()),
                    format!("Không tìm thấy: {}", dir.display()),
                ),
                AppLanguage::English => (
                    format!("Mailcache directory does not exist: {}", dir.display()),
                    format!("Not found: {}", dir.display()),
                ),
            };
            self.log("WARN", &warn_msg);
            self.set_status(WatcherStatus::Stopped(status_msg));
            return;
        }

        if initial_scan {
            let self_clone = self.clone_handle();
            thread::spawn(move || {
                self_clone.run_initial_scan(false);
            });
        } else {
            self.start_observer();
        }
    }

    pub fn stop(&self) {
        self.flush_vault_if_dirty();

        if !*self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            return;
        }

        if let Some(stop_tx) = self.stop_signal.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = stop_tx.send(());
        }

        crate::tray::trim_memory_working_set();

        *self.is_watching.write().unwrap_or_else(|e| e.into_inner()) = false;
        let (status_msg, log_msg) = match self.get_language() {
            AppLanguage::Vietnamese => ("Đã dừng theo dõi".to_string(), "Đã tạm dừng theo dõi thư mục mailcache"),
            AppLanguage::English => ("Watching stopped".to_string(), "Paused watching mailcache directory"),
        };
        self.set_status(WatcherStatus::Stopped(status_msg));
        self.log("INFO", log_msg);
    }

    pub fn rescan(&self) {
        let self_clone = self.clone_handle();
        thread::spawn(move || {
            self_clone.run_initial_scan(false);
        });
    }

    pub fn rescan_force(&self) {
        let self_clone = self.clone_handle();
        thread::spawn(move || {
            self_clone.run_initial_scan(true);
        });
    }

    fn run_initial_scan(&self, force_all: bool) {
        if *self.is_scanning.read().unwrap_or_else(|e| e.into_inner()) {
            return;
        }
        *self.is_scanning.write().unwrap_or_else(|e| e.into_inner()) = true;

        let mail_dir = self.get_mail_dir();
        let mode = self.get_mode();
        let vault_dir = self.get_vault_binary_dir();
        let _ = std::fs::create_dir_all(&vault_dir);

        let lang = self.get_language();
        let (scan_status, scan_start) = match lang {
            AppLanguage::Vietnamese => (
                "Đang quét và đồng bộ kho lưu trữ Vault...".to_string(),
                format!("Bắt đầu quét thư mục game: {} (Chế độ: {})", mail_dir.display(), mode.as_str()),
            ),
            AppLanguage::English => (
                "Scanning and synchronizing Vault archive...".to_string(),
                format!("Started scanning game directory: {} (Mode: {})", mail_dir.display(), mode.as_str()),
            ),
        };
        self.set_status(WatcherStatus::Scanning(scan_status));
        self.log("INFO", &scan_start);

        let mut all_files_map = HashMap::new();

        // 1. Quét file từ mailcache của game
        if let Ok(entries) = std::fs::read_dir(&mail_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if fname.starts_with("Persistent.Mail.") {
                        // Tự động sao lưu nhị phân tức thì vào mail_vault nếu chưa có
                        let backup_target = vault_dir.join(&fname);
                        if !backup_target.exists() {
                            let _ = std::fs::copy(&p, &backup_target);
                        }
                        all_files_map.insert(fname, p);
                    }
                }
            }
        }

        // 2. Quét bổ sung từ kho nhị phân mail_vault (các thư game đã xóa nhưng đã được vault lưu trước đó)
        if let Ok(entries) = std::fs::read_dir(&vault_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if fname.starts_with("Persistent.Mail.") {
                        all_files_map.entry(fname).or_insert(p);
                    }
                }
            }
        }

        let total_cache_files = all_files_map.len();
        let cache_msg = match lang {
            AppLanguage::Vietnamese => format!("Tìm thấy tổng cộng {} file thư nhị phân (gồm cả trong mailcache & kho vault).", total_cache_files),
            AppLanguage::English => format!("Found {} binary mail files total (including mailcache & vault).", total_cache_files),
        };
        self.log("INFO", &cache_msg);

        // FAST-SKIP: Lọc ra chỉ những file thực sự cần giải mã
        let (files_to_decode, skipped_count): (Vec<PathBuf>, usize) = if force_all {
            (all_files_map.into_values().collect(), 0)
        } else {
            let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
            let mut to_decode = Vec::new();
            let mut skipped = 0;
            for (fname, p) in all_files_map {
                if v.contains_file_or_id(&fname) {
                    skipped += 1;
                } else {
                    to_decode.push(p);
                }
            }
            (to_decode, skipped)
        };

        if skipped_count > 0 {
            let skip_msg = match lang {
                AppLanguage::Vietnamese => format!(
                    "Fast-Skip: Đã bỏ qua {} file thư đã đồng bộ trong Vault (0 MB RAM, 0 ms I/O).",
                    skipped_count
                ),
                AppLanguage::English => format!(
                    "Fast-Skip: Skipped {} mail files already synchronized in Vault (0 MB RAM, 0 ms I/O).",
                    skipped_count
                ),
            };
            self.log("INFO", &skip_msg);
        }

        let new_files_count = files_to_decode.len();
        if new_files_count > 0 {
            let new_msg = match lang {
                AppLanguage::Vietnamese => format!("Phát hiện {} file thư mới cần giải mã (xử lý theo batch 50 file).", new_files_count),
                AppLanguage::English => format!("Detected {} new mail files to decode (processing in batches of 50).", new_files_count),
            };
            self.log("INFO", &new_msg);
        }

        use rayon::prelude::*;
        let catalog = &self.catalog;
        let mut scanned_records: Vec<BattleRecord> = Vec::new();

        // Limit concurrently decoded mail trees to keep peak memory predictable.
        for chunk in files_to_decode.chunks(crate::lilith_stream::MAIL_DECODE_BATCH_SIZE) {
            let chunk_records: Vec<BattleRecord> = chunk
                .par_iter()
                .filter_map(|fpath| {
                    let fname = fpath.file_name()?.to_string_lossy();
                    let m = decode_mail_file(fpath).ok()?;
                    let box_name = m.get("box").and_then(|v| v.as_str()).unwrap_or("");
                    if box_name != "Report" && box_name != "Archive" {
                        return None;
                    }
                    let btype = classify_mail(&m)?;
                    extract_motte_details(&m, &fname, catalog, Some(btype)).ok()
                })
                .collect();
            scanned_records.extend(chunk_records);
        }

        // Gộp toàn bộ chiến báo mới vào Persistent Vault (không bao giờ ghi đè làm mất lịch sử cũ)
        let new_added = if !scanned_records.is_empty() {
            let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
            v.upsert_batch(scanned_records)
        } else {
            0
        };

        let total_all = self.vault.read().unwrap_or_else(|e| e.into_inner()).total_count();

        let (mode_cnt, active_cnt) = self.recompute_filtered_records_and_stats();
        *self.is_scanning.write().unwrap_or_else(|e| e.into_inner()) = false;

        self.auto_export();

        // Thu hồi ngay Working Set RAM nhàn rỗi sau khi quét hoàn tất
        crate::tray::trim_memory_working_set();

        let sync_msg = match lang {
            AppLanguage::Vietnamese => format!(
                "Đồng bộ Vault hoàn tất: Tổng số {} thư trong kho lưu trữ (+{} thư mới). Đang lọc {} trận theo chế độ ({} trận khớp ngày).",
                total_all, new_added, mode_cnt, active_cnt
            ),
            AppLanguage::English => format!(
                "Vault synchronization complete: {} total mails in archive (+{} new). Filtering {} records by mode ({} matching date).",
                total_all, new_added, mode_cnt, active_cnt
            ),
        };
        self.log("SUCCESS", &sync_msg);

        if !*self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            self.start_observer();
        } else {
            let watch_msg = match lang {
                AppLanguage::Vietnamese => format!("Đang theo dõi: {}", mail_dir.display()),
                AppLanguage::English => format!("Watching: {}", mail_dir.display()),
            };
            self.set_status(WatcherStatus::Running(watch_msg));
        }
    }

    fn start_observer(&self) {
        let mail_dir = self.get_mail_dir();
        let (tx_stop, rx_stop) = channel();
        *self.stop_signal.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx_stop);

        let self_clone = self.clone_handle();
        let dir_to_watch = mail_dir.clone();

        thread::spawn(move || {
            let (tx_notify, rx_notify) = channel();

            let lang = self_clone.get_language();
            let mut watcher = match RecommendedWatcher::new(tx_notify, Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    let err_msg = match lang {
                        AppLanguage::Vietnamese => format!("Lỗi khởi tạo RecommendedWatcher: {}", e),
                        AppLanguage::English => format!("Error initializing RecommendedWatcher: {}", e),
                    };
                    self_clone.log("ERROR", &err_msg);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&dir_to_watch, RecursiveMode::NonRecursive) {
                let err_msg = match lang {
                    AppLanguage::Vietnamese => format!("Lỗi lắng nghe thư mục {}: {}", dir_to_watch.display(), e),
                    AppLanguage::English => format!("Error watching directory {}: {}", dir_to_watch.display(), e),
                };
                self_clone.log("ERROR", &err_msg);
                self_clone.set_status(WatcherStatus::Error(e.to_string()));
                return;
            }

            *self_clone.is_watching.write().unwrap_or_else(|e| e.into_inner()) = true;
            let (status_text, log_text) = match lang {
                AppLanguage::Vietnamese => (
                    format!("Đang theo dõi: {}", dir_to_watch.display()),
                    format!("Bắt đầu lắng nghe thay đổi thời gian thực tại: {}", dir_to_watch.display()),
                ),
                AppLanguage::English => (
                    format!("Watching: {}", dir_to_watch.display()),
                    format!("Started real-time listening for changes at: {}", dir_to_watch.display()),
                ),
            };
            self_clone.set_status(WatcherStatus::Running(status_text));
            self_clone.log("INFO", &log_text);

            let mut recent_events: HashMap<PathBuf, Instant> = HashMap::new();

            loop {
                // Check stop signal
                if rx_stop.try_recv().is_ok() {
                    self_clone.flush_vault_if_dirty();
                    break;
                }

                // Debounce check for vault save: flush sau 1.5s nhàn rỗi kể từ lần ghi cuối
                self_clone.check_and_flush_vault_debounce(Duration::from_millis(1500));

                if let Ok(res) = rx_notify.recv_timeout(Duration::from_millis(100)) {
                    match res {
                        Ok(event) => {
                            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                                let is_create = matches!(event.kind, EventKind::Create(_));
                                for path in event.paths {
                                    let fname = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy();
                                    if !fname.starts_with("Persistent.Mail.") {
                                        continue;
                                    }

                                    let now = Instant::now();
                                    if recent_events.len() > 100 {
                                        recent_events.retain(|_, last_time| now.duration_since(*last_time) < Duration::from_secs(10));
                                    }

                                    // Zero-Latency Snatch: Đối với file mới tạo, chỉ debounce 50ms chống duplicate event trong tích tắc.
                                    // Đối với Modify, debounce 400ms.
                                    let debounce_threshold = if is_create {
                                        Duration::from_millis(50)
                                    } else {
                                        Duration::from_millis(400)
                                    };

                                    if let Some(last_time) = recent_events.get(&path) {
                                        if now.duration_since(*last_time) < debounce_threshold {
                                            continue;
                                        }
                                    }
                                    recent_events.insert(path.clone(), now);

                                    let self_inner = self_clone.clone_handle();
                                    thread::spawn(move || {
                                        self_inner.process_file_async(&path);
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            let warn_msg = match self_clone.get_language() {
                                AppLanguage::Vietnamese => format!("Lỗi watcher event: {}", e),
                                AppLanguage::English => format!("Watcher event error: {}", e),
                            };
                            self_clone.log("WARN", &warn_msg);
                        }
                    }
                }
            }
        });
    }

    fn process_file_async(&self, path: &Path) {
        let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let mut file_buf = Vec::new();

        // Zero-Latency Shared Read: Polling nhanh với cờ chia sẻ đầy đủ của Windows
        for attempt in 0..6 {
            if let Ok(b) = read_file_shared(path) {
                if b.len() >= 10 && b[0] == 0xFF && b[9] == 0x05 {
                    file_buf = b;
                    break;
                }
            }

            // Nếu file đã bị game xóa nhưng trước đó đã kịp lưu vào vault
            let vault_path = self.get_vault_binary_dir().join(&fname);
            if vault_path.is_file() {
                if let Ok(b) = read_file_shared(&vault_path) {
                    if b.len() >= 10 && b[0] == 0xFF && b[9] == 0x05 {
                        file_buf = b;
                        break;
                    }
                }
            }

            let sleep_ms = 15 * (1 << attempt); // 15ms, 30ms, 60ms, 120ms, 240ms...
            thread::sleep(Duration::from_millis(sleep_ms));
        }

        if file_buf.is_empty() {
            return;
        }

        // Sao lưu nhị phân ngay lập tức vào kho mail_vault trước khi game kịp xóa
        let vault_dir = self.get_vault_binary_dir();
        let backup_dst = vault_dir.join(&fname);
        if !backup_dst.exists() {
            let _ = std::fs::write(&backup_dst, &file_buf);
        }

        // Giải mã nội dung
        let m = match crate::lilith_stream::decode_mail_bytes(&file_buf) {
            Ok(val) => val,
            Err(_) => return,
        };

        let box_name = m.get("box").and_then(|v| v.as_str()).unwrap_or("");
        if box_name != "Report" && box_name != "Archive" {
            return;
        }

        let btype = match classify_mail(&m) {
            Some(t) => t,
            None => return,
        };

        let rec = match extract_motte_details(&m, &fname, &self.catalog, Some(btype)) {
            Ok(r) => r,
            Err(_) => return,
        };

        // Lưu bản ghi vào Persistent Vault
        let is_new = {
            let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
            v.upsert(rec.clone())
        };

        // Đánh dấu dirty để debouncer ghi đĩa một cách tối ưu
        self.mark_vault_dirty();

        self.recompute_filtered_records_and_stats();

        let loot_preview: Vec<String> = rec
            .item_totals
            .iter()
            .take(3)
            .map(|(k, v)| format!("{}: {}", k, crate::model::format_num(*v)))
            .collect();

        let preview_str = loot_preview.join(", ");
        let lang = self.get_language();
        let is_new_str = match (is_new, lang) {
            (true, AppLanguage::Vietnamese) => "mới",
            (false, AppLanguage::Vietnamese) => "cập nhật",
            (true, AppLanguage::English) => "new",
            (false, AppLanguage::English) => "updated",
        };
        let target_display = rec.display_target(lang);
        if btype == "gathering" {
            let msg = match lang {
                AppLanguage::Vietnamese => format!(
                    "Phát hiện thu thập {}: {} (Cấp {}) [Đã bảo vệ trong Vault] | Thu hoạch: {}",
                    is_new_str,
                    target_display,
                    rec.target_level,
                    preview_str
                ),
                AppLanguage::English => format!(
                    "Detected {} gathering: {} (Lvl {}) [Secured in Vault] | Harvest: {}",
                    is_new_str,
                    target_display,
                    rec.target_level,
                    preview_str
                ),
            };
            self.log("SUCCESS", &msg);
        } else {
            let msg = match lang {
                AppLanguage::Vietnamese => format!(
                    "Phát hiện {}: {} ({}% ST, Tier {}) [Đã bảo vệ trong Vault] | Quà: {}",
                    is_new_str,
                    target_display,
                    rec.damage_pct,
                    rec.plunder_tier,
                    preview_str
                ),
                AppLanguage::English => format!(
                    "Detected {} battle: {} ({}% DMG, Tier {}) [Secured in Vault] | Loot: {}",
                    is_new_str,
                    target_display,
                    rec.damage_pct,
                    rec.plunder_tier,
                    preview_str
                ),
            };
            self.log("SUCCESS", &msg);
        }

        self.auto_export();

        let _ = self.ui_sender.send(WatcherUiEvent::MailDetected {
            target: rec.target.clone(),
            damage_pct: rec.damage_pct,
            tier: rec.plunder_tier,
            is_new,
            loot_summary: preview_str,
        });
    }

    pub fn auto_export(&self) {
        static EXPORT_MUTEX: Mutex<()> = Mutex::new(());
        let _guard = EXPORT_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let recs = self.get_active_records();
        let stats = self.get_stats();
        let out_dir = self.get_out_dir();
        let mail_dir = self.get_mail_dir();

        let _ = crate::exporters::export_json(
            &recs,
            &stats,
            &out_dir.join("motte_filtered_mails.json"),
        );
        crate::exporters::export_individual_mails_json(&recs, &mail_dir, &out_dir);
    }

    fn clone_handle(&self) -> Self {
        Self {
            mail_dir: Arc::clone(&self.mail_dir),
            out_dir: Arc::clone(&self.out_dir),
            mode: Arc::clone(&self.mode),
            language: Arc::clone(&self.language),
            start_date: Arc::clone(&self.start_date),
            end_date: Arc::clone(&self.end_date),
            days: Arc::clone(&self.days),
            active_records: Arc::clone(&self.active_records),
            stats: Arc::clone(&self.stats),
            catalog: Arc::clone(&self.catalog),
            vault: Arc::clone(&self.vault),
            vault_dirty: Arc::clone(&self.vault_dirty),
            last_vault_mutation: Arc::clone(&self.last_vault_mutation),
            is_watching: Arc::clone(&self.is_watching),
            is_scanning: Arc::clone(&self.is_scanning),
            ui_sender: self.ui_sender.clone(),
            stop_signal: Arc::clone(&self.stop_signal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_process_gathering_mail() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("mail_vault/Persistent.Mail.1002220217856803764");
        let mail = decode_mail_file(&path).expect("Decode mail");
        let btype = classify_mail(&mail);
        assert_eq!(btype, Some("gathering"));

        let rec = crate::classifier::extract_gathering_details(&mail, "Persistent.Mail.1002220217856803764")
            .expect("Extract gathering");
        assert_eq!(rec.battle_type, "gathering");
        assert_eq!(rec.target_level, "6");
        assert_eq!(rec.target, "Mỏ Đá Cấp 6");
    }

    #[test]
    fn test_stop_flushes_dirty_vault_when_paused() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_flush_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(temp_dir.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        // Ensure watching is false (paused)
        assert!(!watcher.is_watching());

        // Mark dirty
        watcher.mark_vault_dirty();
        assert!(watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst));

        // Calling stop while watching is false should still flush dirty state
        watcher.stop();
        assert!(!watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst), "Dirty flag must be cleared by flush on stop");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_watcher_date_range_label_bilingual() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            None,
            None,
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        // Default all time
        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "Tất cả thời gian");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "All time");

        // 7 days rolling
        watcher.set_date_filter(None, None, Some(7));
        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "7 ngày gần nhất");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "Last 7 days");

        // Explicit dates
        let d1 = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        watcher.set_date_filter(Some(d1), Some(d2), None);
        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "Từ 2026-10-01 đến 2026-10-10");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "From 2026-10-01 to 2026-10-10");
    }
}
