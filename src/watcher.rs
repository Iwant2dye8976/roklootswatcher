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
use crate::lilith_stream::decode_mail_file;
use crate::loot_catalog::LootCatalog;
use crate::model::{BattleRecord, FilterMode, Statistics};
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
        self.log("INFO", &format!("Đã chuyển thư mục theo dõi sang: {}", new_dir.display()));
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
        self.log("INFO", &format!("Đã chuyển chế độ mục tiêu: {}", new_mode.display_label()));
        let (total_mode, active_cnt) = self.recompute_filtered_records_and_stats();
        self.log(
            "SUCCESS",
            &format!(
                "Bộ lọc cập nhật tức thì: Tìm thấy {} trận ({} trận khớp ngày).",
                total_mode, active_cnt
            ),
        );
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

        let today_vn = crate::analyzer::current_date_vn();
        let yest_vn = today_vn - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));

        match (s, e) {
            (Some(s_d), Some(e_d)) => {
                if s_d == e_d {
                    if s_d == today_vn {
                        format!("Hôm nay ({})", s_d)
                    } else if s_d == yest_vn {
                        format!("Hôm qua ({})", s_d)
                    } else {
                        format!("Ngày {}", s_d)
                    }
                } else {
                    format!("Từ {} đến {}", s_d, e_d)
                }
            }
            (Some(s_d), None) => format!("Từ {} đến nay", s_d),
            (None, Some(e_d)) => format!("Đến ngày {}", e_d),
            (None, None) => {
                if let Some(days) = d {
                    format!("{} ngày gần nhất", days)
                } else {
                    "Tất cả thời gian".to_string()
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

        self.log(
            "INFO",
            &format!(
                "Đã áp dụng lọc ngày: {} ({} trận)",
                self.get_date_range_label(),
                active_cnt
            ),
        );
    }

    pub fn start(&self, initial_scan: bool) {
        if *self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            return;
        }

        let dir = self.get_mail_dir();
        if !dir.is_dir() {
            self.log(
                "WARN",
                &format!("Thư mục mailcache không tồn tại: {}", dir.display()),
            );
            self.set_status(WatcherStatus::Stopped(format!(
                "Không tìm thấy: {}",
                dir.display()
            )));
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
        if !*self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            return;
        }

        if let Some(stop_tx) = self.stop_signal.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = stop_tx.send(());
        }

        self.flush_vault_if_dirty();
        crate::tray::trim_memory_working_set();

        *self.is_watching.write().unwrap_or_else(|e| e.into_inner()) = false;
        self.set_status(WatcherStatus::Stopped("Đã dừng theo dõi".into()));
        self.log("INFO", "Đã tạm dừng theo dõi thư mục mailcache");
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

        self.set_status(WatcherStatus::Scanning("Đang quét và đồng bộ kho lưu trữ Vault...".into()));
        self.log("INFO", &format!("Bắt đầu quét thư mục game: {} (Chế độ: {})", mail_dir.display(), mode.as_str()));

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
        self.log("INFO", &format!("Tìm thấy tổng cộng {} file thư nhị phân (gồm cả trong mailcache & kho vault).", total_cache_files));

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
            self.log(
                "INFO",
                &format!(
                    "Fast-Skip: Đã bỏ qua {} file thư đã đồng bộ trong Vault (0 MB RAM, 0 ms I/O).",
                    skipped_count
                ),
            );
        }

        let new_files_count = files_to_decode.len();
        if new_files_count > 0 {
            self.log(
                "INFO",
                &format!("Phát hiện {} file thư mới cần giải mã (xử lý theo batch 50 file).", new_files_count),
            );
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
                    if btype == "gathering" {
                        crate::classifier::extract_gathering_details(&m, &fname).ok()
                    } else {
                        extract_motte_details(&m, &fname, catalog, Some(btype)).ok()
                    }
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

        self.log(
            "SUCCESS",
            &format!(
                "Đồng bộ Vault hoàn tất: Tổng số {} thư trong kho lưu trữ (+{} thư mới). Đang lọc {} trận theo chế độ ({} trận khớp ngày).",
                total_all, new_added, mode_cnt, active_cnt
            ),
        );

        if !*self.is_watching.read().unwrap_or_else(|e| e.into_inner()) {
            self.start_observer();
        } else {
            self.set_status(WatcherStatus::Running(format!(
                "Đang theo dõi: {}",
                mail_dir.display()
            )));
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

            let mut watcher = match RecommendedWatcher::new(tx_notify, Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    self_clone.log("ERROR", &format!("Lỗi khởi tạo RecommendedWatcher: {}", e));
                    return;
                }
            };

            if let Err(e) = watcher.watch(&dir_to_watch, RecursiveMode::NonRecursive) {
                self_clone.log("ERROR", &format!("Lỗi lắng nghe thư mục {}: {}", dir_to_watch.display(), e));
                self_clone.set_status(WatcherStatus::Error(e.to_string()));
                return;
            }

            *self_clone.is_watching.write().unwrap_or_else(|e| e.into_inner()) = true;
            self_clone.set_status(WatcherStatus::Running(format!(
                "Đang theo dõi: {}",
                dir_to_watch.display()
            )));
            self_clone.log("INFO", &format!("Bắt đầu lắng nghe thay đổi thời gian thực tại: {}", dir_to_watch.display()));

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
                            self_clone.log("WARN", &format!("Lỗi watcher event: {}", e));
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

        let rec = if btype == "gathering" {
            match crate::classifier::extract_gathering_details(&m, &fname) {
                Ok(r) => r,
                Err(_) => return,
            }
        } else {
            match extract_motte_details(&m, &fname, &self.catalog, Some(btype)) {
                Ok(r) => r,
                Err(_) => return,
            }
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
        if btype == "gathering" {
            self.log(
                "SUCCESS",
                &format!(
                    "Phát hiện thu thập {}: {} (Cấp {}) [Đã bảo vệ trong Vault] | Thu hoạch: {}",
                    if is_new { "mới" } else { "cập nhật" },
                    rec.target,
                    rec.target_level,
                    preview_str
                ),
            );
        } else {
            self.log(
                "SUCCESS",
                &format!(
                    "Phát hiện {}: {} ({}% ST, Tier {}) [Đã bảo vệ trong Vault] | Quà: {}",
                    if is_new { "mới" } else { "cập nhật" },
                    rec.target,
                    rec.damage_pct,
                    rec.plunder_tier,
                    preview_str
                ),
            );
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
        let path = "mail_vault/Persistent.Mail.1002220217856803764";
        let mail = decode_mail_file(path).expect("Decode mail");
        let btype = classify_mail(&mail);
        assert_eq!(btype, Some("gathering"));

        let rec = crate::classifier::extract_gathering_details(&mail, "Persistent.Mail.1002220217856803764")
            .expect("Extract gathering");
        assert_eq!(rec.battle_type, "gathering");
        assert_eq!(rec.target_level, "6");
        assert_eq!(rec.target, "Mỏ Đá Cấp 6");
    }
}
