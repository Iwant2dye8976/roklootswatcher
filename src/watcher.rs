use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use chrono::NaiveDate;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};

use crate::analyzer::{filter_records_by_date, generate_statistics};
use crate::classifier::{classify_mail, extract_motte_details};
use crate::i18n::filter_mode_name;
use crate::lilith_stream::decode_mail_file;
use crate::loot_catalog::LootCatalog;
use crate::model::{AppLanguage, BattleRecord, FilterMode, Statistics};
use crate::vault::{read_file_shared, MailVault};

pub const DEFAULT_MAIL_DIR: &str =
    r"C:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache";

pub fn probe_candidate_is_mailcache(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let is_mailcache = path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("mailcache"));
    let parent_is_save = path
        .parent()
        .and_then(|p| p.file_name())
        .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("save"));
    is_mailcache && parent_is_save
}

#[cfg(windows)]
fn find_running_rok_mailcache() -> Option<PathBuf> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let hwnd = crate::overlay_tracker::find_rok_window()?;
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }

        if let Some(exe_path) = crate::overlay_tracker::get_process_exe_path(pid) {
            if let Some(parent) = exe_path.parent() {
                let candidate = parent.join("save").join("mailcache");
                if probe_candidate_is_mailcache(&candidate) {
                    return Some(candidate);
                }
                if let Some(grandparent) = parent.parent() {
                    let candidate2 = grandparent.join("save").join("mailcache");
                    if probe_candidate_is_mailcache(&candidate2) {
                        return Some(candidate2);
                    }
                }
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn find_running_rok_mailcache() -> Option<PathBuf> {
    None
}

pub(crate) fn probe_directory_tree(dir: &Path, max_depth: usize) -> Option<PathBuf> {
    if max_depth == 0 {
        return None;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let name_lower = name_str.to_lowercase();

        if name_lower.starts_with('$')
            || name_lower == "windows"
            || name_lower == "system volume information"
            || name_lower == "recovery"
            || name_lower == "msocache"
            || name_lower == "users"
            || name_lower == "programdata"
            || name_lower == "node_modules"
            || name_lower == ".git"
        {
            continue;
        }

        let child_path = entry.path();

        if name_lower.contains("rise of kingdoms") {
            let direct_cache = child_path.join("save").join("mailcache");
            if probe_candidate_is_mailcache(&direct_cache) {
                return Some(direct_cache);
            }
            let game_cache = child_path
                .join("Rise of Kingdoms Game")
                .join("save")
                .join("mailcache");
            if probe_candidate_is_mailcache(&game_cache) {
                return Some(game_cache);
            }
        }

        if max_depth > 1 {
            if let Some(found) = probe_directory_tree(&child_path, max_depth - 1) {
                return Some(found);
            }
        }
    }

    None
}

pub fn scan_drive_candidates() -> Option<PathBuf> {
    for drive in 'C'..='Z' {
        let drive_root = format!(r"{}:\", drive);
        let root_path = Path::new(&drive_root);
        if !root_path.is_dir() {
            continue;
        }

        let fixed_candidates = [
            format!(r"{}:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache", drive),
            format!(r"{}:\Program Files\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache", drive),
            format!(r"{}:\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache", drive),
            format!(r"{}:\Games\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache", drive),
            format!(r"{}:\Game\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache", drive),
            format!(r"{}:\ROK\Rise of Kingdoms Game\save\mailcache", drive),
        ];

        for cand_str in &fixed_candidates {
            let cand = PathBuf::from(cand_str);
            if probe_candidate_is_mailcache(&cand) {
                return Some(cand);
            }
        }

        let games_dir = PathBuf::from(format!(r"{}:\Games", drive));
        if games_dir.is_dir() {
            if let Some(found) = probe_directory_tree(&games_dir, 2) {
                return Some(found);
            }
        }

        if let Some(found) = probe_directory_tree(root_path, 2) {
            return Some(found);
        }
    }
    None
}

pub fn scan_for_mailcache() -> Option<PathBuf> {
    if let Ok(env_val) = std::env::var("ROK_MAIL_DIR") {
        let clean_val = env_val.trim().trim_matches('"');
        let p = PathBuf::from(clean_val);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Some(p) = find_running_rok_mailcache() {
        return Some(p);
    }
    if let Some(p) = scan_drive_candidates() {
        return Some(p);
    }
    None
}

pub fn detect_default_mail_dir() -> PathBuf {
    scan_for_mailcache().unwrap_or_else(|| PathBuf::from(DEFAULT_MAIL_DIR))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    selected_account: Arc<RwLock<Option<String>>>,
    main_account_uids: Arc<RwLock<Vec<String>>>,

    active_records: Arc<RwLock<Arc<Vec<BattleRecord>>>>,
    stats: Arc<RwLock<Statistics>>,
    catalog: Arc<LootCatalog>,
    pub(crate) vault: Arc<RwLock<MailVault>>,

    vault_dirty: Arc<AtomicBool>,
    has_unbacked_cloud_data: Arc<AtomicBool>,
    last_vault_mutation: Arc<Mutex<Instant>>,

    is_watching: Arc<RwLock<bool>>,
    is_scanning: Arc<RwLock<bool>>,

    ui_sender: Sender<WatcherUiEvent>,
    stop_signal: Arc<Mutex<Option<Sender<()>>>>,
    sync_tx: Arc<Mutex<Option<Sender<crate::cloud_sync::SyncCommand>>>>,
}

pub type Watcher = RoKMailWatcher;

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

        let vault = MailVault::new(&o_dir);
        let watcher = Self {
            mail_dir: Arc::new(RwLock::new(m_dir)),
            out_dir: Arc::new(RwLock::new(o_dir)),
            mode: Arc::new(RwLock::new(mode)),
            language: Arc::new(RwLock::new(AppLanguage::default())),

            start_date: Arc::new(RwLock::new(None)),
            end_date: Arc::new(RwLock::new(None)),
            days: Arc::new(RwLock::new(None)),
            selected_account: Arc::new(RwLock::new(None)),
            main_account_uids: Arc::new(RwLock::new(Vec::new())),

            active_records: Arc::new(RwLock::new(Arc::new(Vec::new()))),
            stats: Arc::new(RwLock::new(Statistics::default())),
            catalog: Arc::new(catalog),
            vault: Arc::new(RwLock::new(vault)),

            vault_dirty: Arc::new(AtomicBool::new(false)),
            has_unbacked_cloud_data: Arc::new(AtomicBool::new(false)),
            last_vault_mutation: Arc::new(Mutex::new(Instant::now())),

            is_watching: Arc::new(RwLock::new(false)),
            is_scanning: Arc::new(RwLock::new(false)),

            ui_sender,
            stop_signal: Arc::new(Mutex::new(None)),
            sync_tx: Arc::new(Mutex::new(None)),
        };

        watcher.recompute_filtered_records_and_stats();
        watcher
    }

    pub fn set_sync_channel(&self, tx: Sender<crate::cloud_sync::SyncCommand>) {
        if let Ok(mut lock) = self.sync_tx.lock() {
            *lock = Some(tx);
        }
    }

    pub fn send_sync_command(&self, cmd: crate::cloud_sync::SyncCommand) -> bool {
        if let Ok(lock) = self.sync_tx.lock() {
            if let Some(tx) = &*lock {
                return tx.send(cmd).is_ok();
            }
        }
        false
    }

    pub fn process_mail_file(&self, path: &Path) {
        self.process_file_async(path);
    }

    fn notify_sync_trigger(&self) {
        if let Ok(lock) = self.sync_tx.lock() {
            if let Some(tx) = &*lock {
                let _ = tx.send(crate::cloud_sync::SyncCommand::TriggerDelta);
            }
        }
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

    #[allow(dead_code)]
    pub fn get_vault_file_path(&self) -> PathBuf {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).get_vault_file().to_path_buf()
    }

    #[allow(dead_code)]
    pub fn reload_vault(&self) {
        let out_dir = self.out_dir.read().unwrap_or_else(|e| e.into_inner()).clone();
        let new_vault = crate::vault::MailVault::new(&out_dir);
        *self.vault.write().unwrap_or_else(|e| e.into_inner()) = new_vault;
        self.recompute_filtered_records_and_stats();
    }

    pub fn clean_binary_vault(&self) -> Result<(usize, u64), String> {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).clean_binary_vault()
    }

    pub fn get_vault_binary_dir(&self) -> PathBuf {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).get_binary_dir().to_path_buf()
    }

    pub fn export_vault_json(&self) -> Result<String, String> {
        self.vault.read().unwrap_or_else(|e| e.into_inner()).export_vault_json()
    }

    pub fn import_vault_json(&self, json: &str) -> Result<usize, String> {
        let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
        v.import_records_from_json(json)
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
        let all = {
            let vault = self.vault.read().unwrap_or_else(|e| e.into_inner());
            vault.get_records()
        };
        let mode = self.get_mode();
        let sel_acc = self.selected_account.read().unwrap_or_else(|e| e.into_inner()).clone();
        let main_uids = self.main_account_uids.read().unwrap_or_else(|e| e.into_inner()).clone();
        let matches_acc = |r: &BattleRecord| {
            let clean_rec = r.account_id.strip_prefix("player_").unwrap_or(&r.account_id).trim();
            if !main_uids.is_empty() {
                let is_main = !clean_rec.is_empty() && main_uids.iter().any(|u| {
                    let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                    clean_u == clean_rec
                });
                if !is_main {
                    return false;
                }
            }
            match &sel_acc {
                Some(acc) => {
                    let clean_acc = acc.strip_prefix("player_").unwrap_or(acc).trim();
                    !clean_acc.is_empty() && clean_rec == clean_acc
                }
                None => true,
            }
        };

        let total_mode_count = all
            .iter()
            .filter(|r| mode.matches(&r.battle_type) && matches_acc(r))
            .count();

        let s = *self.start_date.read().unwrap_or_else(|e| e.into_inner());
        let e = *self.end_date.read().unwrap_or_else(|e| e.into_inner());
        let d = *self.days.read().unwrap_or_else(|e| e.into_inner());

        let active_vec = if s.is_none() && e.is_none() && d.is_none() {
            all.into_iter()
                .filter(|r| mode.matches(&r.battle_type) && matches_acc(r))
                .collect()
        } else {
            filter_records_by_date(
                all.iter().filter(|r| mode.matches(&r.battle_type) && matches_acc(r)),
                s,
                e,
                d,
            )
        };

        let active_cnt = active_vec.len();
        let new_stats = generate_statistics(&active_vec);

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

    pub fn set_account_filter(&self, account: Option<String>) {
        *self.selected_account.write().unwrap_or_else(|e| e.into_inner()) = account;
        let (total_mode, active_cnt) = self.recompute_filtered_records_and_stats();
        let lang = self.get_language();
        let msg = match lang {
            AppLanguage::Vietnamese => format!("Lọc tài khoản: Tìm thấy {} trận ({} trận khớp ngày).", total_mode, active_cnt),
            AppLanguage::English => format!("Account filter: Found {} records ({} matching date).", total_mode, active_cnt),
        };
        self.log("INFO", &msg);
    }

    pub fn get_selected_account(&self) -> Option<String> {
        self.selected_account.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn get_main_account_uids(&self) -> Vec<String> {
        self.main_account_uids.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_main_account_uids(&self, uids: Vec<String>) {
        *self.main_account_uids.write().unwrap_or_else(|e| e.into_inner()) = uids;
        self.recompute_filtered_records_and_stats();
    }

    pub fn count_non_main_records(&self) -> usize {
        let main_uids = self.get_main_account_uids();
        if main_uids.is_empty() {
            return 0;
        }
        let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
        v.count_records_except_uids(&main_uids)
    }

    pub fn delete_non_main_records(&self) -> usize {
        let main_uids = self.get_main_account_uids();
        if main_uids.is_empty() {
            return 0;
        }
        let deleted = {
            let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
            v.delete_records_except_uids(&main_uids)
        };
        if deleted > 0 {
            self.recompute_filtered_records_and_stats();
            self.notify_sync_trigger();
        }
        deleted
    }

    pub fn get_all_detected_accounts(&self) -> Vec<(String, String)> {
        let all = {
            let vault = self.vault.read().unwrap_or_else(|e| e.into_inner());
            vault.get_records()
        };
        let mut map: HashMap<String, String> = HashMap::new();
        for r in &all {
            let clean_acc = r.account_id.strip_prefix("player_").unwrap_or(&r.account_id).trim();
            if !clean_acc.is_empty() {
                let entry = map.entry(clean_acc.to_string()).or_default();
                if entry.is_empty() && !r.player_name.is_empty() {
                    *entry = r.player_name.clone();
                }
            }
        }
        let mut list: Vec<(String, String)> = map
            .into_iter()
            .map(|(acc_id, p_name)| {
                let label = if !p_name.is_empty() {
                    format!("{} ({})", p_name, acc_id)
                } else {
                    format!("UID: {}", acc_id)
                };
                (acc_id, label)
            })
            .collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        list
    }

    pub fn get_accounts(&self) -> Vec<(String, String)> {
        let all_detected = self.get_all_detected_accounts();
        let main_uids = self.get_main_account_uids();
        if main_uids.is_empty() {
            return all_detected;
        }
        all_detected
            .into_iter()
            .filter(|(acc_id, _)| {
                let clean_acc = acc_id.strip_prefix("player_").unwrap_or(acc_id).trim();
                main_uids.iter().any(|u| {
                    let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                    clean_u == clean_acc
                })
            })
            .collect()
    }

    pub fn get_active_records(&self) -> Arc<Vec<BattleRecord>> {
        Arc::clone(&*self.active_records.read().unwrap_or_else(|e| e.into_inner()))
    }

    pub fn with_vault_records<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[BattleRecord]) -> R,
    {
        let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
        let recs = v.get_records();
        f(&recs)
    }

    pub fn get_latest_vault_records(&self, filter: FilterMode, limit: usize) -> Vec<BattleRecord> {
        let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
        v.query_latest_filtered(filter, limit)
    }

    pub fn get_today_vault_records(&self, filter: FilterMode) -> Vec<BattleRecord> {
        let all = {
            let vault = self.vault.read().unwrap_or_else(|e| e.into_inner());
            vault.get_records()
        };
        let today = crate::analyzer::current_date_vn();
        let main_uids = self.get_main_account_uids();

        let filtered_by_date = crate::analyzer::filter_records_by_date(
            &all,
            Some(today),
            Some(today),
            None,
        );

        filtered_by_date
            .into_iter()
            .filter(|r| {
                if !main_uids.is_empty() {
                    let clean_rec = r.account_id.strip_prefix("player_").unwrap_or(&r.account_id).trim();
                    let is_main = !clean_rec.is_empty() && main_uids.iter().any(|u| {
                        let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                        clean_u == clean_rec
                    });
                    if !is_main {
                        return false;
                    }
                }

                match filter {
                    FilterMode::AllRewards => true,
                    FilterMode::AllTargets => r.battle_type != "gathering",
                    FilterMode::Fort => r.battle_type == "fort",
                    FilterMode::Motte => r.battle_type == "motte",
                    FilterMode::Barbarian => r.battle_type == "barbarian",
                    FilterMode::Gathering => r.battle_type == "gathering",
                    FilterMode::AllForts => matches!(r.battle_type.as_str(), "motte" | "fort"),
                }
            })
            .collect()
    }

    pub fn get_active_count(&self) -> usize {
        self.active_records.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn mark_vault_dirty(&self) {
        self.vault_dirty.store(true, Ordering::SeqCst);
        self.has_unbacked_cloud_data.store(true, Ordering::SeqCst);
        if let Ok(mut t) = self.last_vault_mutation.lock() {
            *t = Instant::now();
        }
    }

    pub fn has_unbacked_cloud_data(&self) -> bool {
        self.has_unbacked_cloud_data.load(Ordering::SeqCst)
    }

    pub fn set_cloud_clean(&self) {
        self.has_unbacked_cloud_data.store(false, Ordering::SeqCst);
    }

    pub fn mark_cloud_dirty(&self) {
        self.has_unbacked_cloud_data.store(true, Ordering::SeqCst);
    }

    pub fn set_cloud_unbacked(&self, unbacked: bool) {
        self.has_unbacked_cloud_data.store(unbacked, Ordering::SeqCst);
    }

    pub fn flush_vault_if_dirty(&self) {
        if self.vault_dirty.swap(false, Ordering::SeqCst) {
            let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
            let save_res = v.save_to_disk();
            drop(v);

            if let Err(e) = save_res {
                self.vault_dirty.store(true, Ordering::SeqCst);
                let err_msg = match self.get_language() {
                    AppLanguage::Vietnamese => format!("Lỗi lưu vault vào đĩa: {}", e),
                    AppLanguage::English => format!("Error saving vault to disk: {}", e),
                };
                self.log("ERROR", &err_msg);
            } else {
                self.auto_export();
            }
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

        if let Ok(entries) = std::fs::read_dir(&mail_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if fname.starts_with("Persistent.Mail.") {                        all_files_map.insert(fname, p);
                    }
                }
            }
        }

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

        let (files_to_decode, skipped_count): (Vec<PathBuf>, usize) = if force_all {
            (all_files_map.into_values().collect(), 0)
        } else {
            let v = self.vault.read().unwrap_or_else(|e| e.into_inner());
            let mut to_decode = Vec::new();
            let mut skipped = 0;
            for (fname, p) in all_files_map {
                if v.contains_file_or_id(&fname) && v.record_has_account_id(&fname) {
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
        let main_uids = self.get_main_account_uids();

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
                    let rec = extract_motte_details(&m, &fname, catalog, Some(btype)).ok()?;
                    if !main_uids.is_empty() {
                        let clean_rec = rec.account_id.strip_prefix("player_").unwrap_or(&rec.account_id).trim();
                        let is_main = !clean_rec.is_empty() && main_uids.iter().any(|u| {
                            let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                            clean_u == clean_rec
                        });
                        if !is_main {
                            return None;
                        }
                    }
                    let backup_dst = vault_dir.join(&*fname);
                    if !backup_dst.exists() && fpath.starts_with(&mail_dir) {
                        let _ = std::fs::copy(fpath, &backup_dst);
                    }
                    Some(rec)
                })
                .collect();
            scanned_records.extend(chunk_records);
        }

        let new_added = if !scanned_records.is_empty() {
            let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
            v.upsert_batch(scanned_records)
        } else {
            0
        };

        if new_added > 0 {
            self.notify_sync_trigger();
        }

        let total_all = self.vault.read().unwrap_or_else(|e| e.into_inner()).total_count();

        let (mode_cnt, active_cnt) = self.recompute_filtered_records_and_stats();
        *self.is_scanning.write().unwrap_or_else(|e| e.into_inner()) = false;

        self.auto_export();

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
                if rx_stop.try_recv().is_ok() {
                    self_clone.flush_vault_if_dirty();
                    break;
                }

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

        for attempt in 0..6 {
            if let Ok(b) = read_file_shared(path) {
                if b.len() >= 10 && b[0] == 0xFF && b[9] == 0x05 {
                    file_buf = b;
                    break;
                }
            }

            let vault_path = self.get_vault_binary_dir().join(&fname);
            if vault_path.is_file() {
                if let Ok(b) = read_file_shared(&vault_path) {
                    if b.len() >= 10 && b[0] == 0xFF && b[9] == 0x05 {
                        file_buf = b;
                        break;
                    }
                }
            }

            let sleep_ms = 15 * (1 << attempt);
            thread::sleep(Duration::from_millis(sleep_ms));
        }

        if file_buf.is_empty() {
            return;
        }

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

        let main_uids = self.get_main_account_uids();
        if !main_uids.is_empty() {
            let clean_rec = rec.account_id.strip_prefix("player_").unwrap_or(&rec.account_id).trim();
            let is_main = !clean_rec.is_empty() && main_uids.iter().any(|u| {
                let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                clean_u == clean_rec
            });
            if !is_main {
                return;
            }
        }

        let vault_dir = self.get_vault_binary_dir();
        let backup_dst = vault_dir.join(&fname);
        if !backup_dst.exists() {
            let _ = std::fs::write(&backup_dst, &file_buf);
        }

        let is_new = {
            let mut v = self.vault.write().unwrap_or_else(|e| e.into_inner());
            v.upsert(rec.clone())
        };

        self.mark_vault_dirty();

        if is_new {
            self.notify_sync_trigger();
        }

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
            selected_account: Arc::clone(&self.selected_account),
            main_account_uids: Arc::clone(&self.main_account_uids),
            active_records: Arc::clone(&self.active_records),
            stats: Arc::clone(&self.stats),
            catalog: Arc::clone(&self.catalog),
            vault: Arc::clone(&self.vault),
            vault_dirty: Arc::clone(&self.vault_dirty),
            has_unbacked_cloud_data: Arc::clone(&self.has_unbacked_cloud_data),
            last_vault_mutation: Arc::clone(&self.last_vault_mutation),
            is_watching: Arc::clone(&self.is_watching),
            is_scanning: Arc::clone(&self.is_scanning),
            ui_sender: self.ui_sender.clone(),
            stop_signal: Arc::clone(&self.stop_signal),
            sync_tx: Arc::clone(&self.sync_tx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_process_gathering_mail() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("tests/fixtures/Persistent.Mail.1002220217856803764");
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
    fn test_watcher_cloud_unbacked_data_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_cloud_dirty_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(temp_dir.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        assert!(!watcher.has_unbacked_cloud_data());

        watcher.mark_vault_dirty();
        assert!(watcher.has_unbacked_cloud_data());

        watcher.set_cloud_clean();
        assert!(!watcher.has_unbacked_cloud_data());

        watcher.mark_cloud_dirty();
        assert!(watcher.has_unbacked_cloud_data());

        watcher.set_cloud_unbacked(false);
        assert!(!watcher.has_unbacked_cloud_data());

        watcher.set_cloud_unbacked(true);
        assert!(watcher.has_unbacked_cloud_data());

        let _ = std::fs::remove_dir_all(&temp_dir);
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

        assert!(!watcher.is_watching());

        watcher.mark_vault_dirty();
        assert!(watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst));

        watcher.stop();
        assert!(!watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst), "Dirty flag must be cleared by flush on stop");
        assert!(temp_dir.join("motte_filtered_mails.json").exists(), "Auto export should be executed when dirty vault is flushed on stop");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_dirty_retained_on_error() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rok_test_dirty_err_{}_{}", std::process::id(), unique));
        let _ = std::fs::create_dir_all(&temp_dir);

        let vault_file = temp_dir.join("rok_history_vault.json");
        std::fs::write(&vault_file, b"{ corrupted_json").unwrap();

        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(temp_dir.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        watcher.mark_vault_dirty();
        assert!(watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst));

        watcher.flush_vault_if_dirty();
        assert!(
            watcher.vault_dirty.load(std::sync::atomic::Ordering::SeqCst),
            "Dirty flag must be retained (set back to true) when save_to_disk fails"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_watcher_date_range_label_bilingual() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_date_label_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            None,
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "Tất cả thời gian");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "All time");

        watcher.set_date_filter(None, None, Some(7));
        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "7 ngày gần nhất");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "Last 7 days");

        let d1 = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();
        watcher.set_date_filter(Some(d1), Some(d2), None);
        watcher.set_language(AppLanguage::Vietnamese);
        assert_eq!(watcher.get_date_range_label(), "Từ 2026-10-01 đến 2026-10-10");
        watcher.set_language(AppLanguage::English);
        assert_eq!(watcher.get_date_range_label(), "From 2026-10-01 to 2026-10-10");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_process_file_skips_non_report_backup() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_skip_backup_{}", std::process::id()));
        let mailcache = temp_dir.join("mailcache");
        let _ = std::fs::create_dir_all(&mailcache);

        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(mailcache.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        let mut valid = vec![0; 10];
        valid[0] = 0xFF;
        valid[9] = 0x05;
        valid.push(0xFF);

        let mail_path = mailcache.join("Persistent.Mail.888888");
        std::fs::write(&mail_path, &valid).unwrap();

        watcher.process_file_async(&mail_path);

        let vault_file = watcher.get_vault_binary_dir().join("Persistent.Mail.888888");
        assert!(!vault_file.exists(), "Non-report mail must NOT be backed up to mail_vault");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_for_mailcache_env_var_valid_and_invalid() {
        let temp = std::env::temp_dir().join("rok_test_mailcache_valid");
        let _ = std::fs::create_dir_all(&temp);

        std::env::set_var("ROK_MAIL_DIR", &temp);
        assert_eq!(scan_for_mailcache(), Some(temp.clone()));

        std::env::set_var("ROK_MAIL_DIR", "Z:\\non_existent_folder_path_xyz_123");
        let res = scan_for_mailcache();
        assert_ne!(res, Some(PathBuf::from("Z:\\non_existent_folder_path_xyz_123")));

        std::env::remove_var("ROK_MAIL_DIR");
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_probe_drive_path_validates_is_dir() {
        let temp_file = std::env::temp_dir().join("rok_test_not_a_dir.tmp");
        let _ = std::fs::write(&temp_file, b"not a dir");
        assert!(!probe_candidate_is_mailcache(&temp_file));
        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn test_probe_candidate_is_mailcache_success_and_failures() {
        let base = std::env::temp_dir().join(format!("rok_test_probe_{}", std::process::id()));
        let valid_mc = base.join("save").join("mailcache");
        let invalid_parent = base.join("other").join("mailcache");
        let invalid_child = base.join("save").join("other");

        let _ = std::fs::create_dir_all(&valid_mc);
        let _ = std::fs::create_dir_all(&invalid_parent);
        let _ = std::fs::create_dir_all(&invalid_child);

        assert!(probe_candidate_is_mailcache(&valid_mc));
        assert!(!probe_candidate_is_mailcache(&invalid_parent));
        assert!(!probe_candidate_is_mailcache(&invalid_child));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_probe_directory_tree_finds_nested_candidate() {
        let base = std::env::temp_dir().join(format!("rok_test_tree_{}", std::process::id()));
        let nested_mc = base
            .join("CustomGames")
            .join("Rise of Kingdoms")
            .join("Rise of Kingdoms Game")
            .join("save")
            .join("mailcache");
        let _ = std::fs::create_dir_all(&nested_mc);

        let found = probe_directory_tree(&base, 2);
        assert_eq!(found, Some(nested_mc));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_detect_default_mail_dir_fallback() {
        std::env::remove_var("ROK_MAIL_DIR");
        let p = detect_default_mail_dir();
        assert!(!p.as_os_str().is_empty());
    }

    #[test]
    fn test_watcher_export_and_import_vault_json() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rok_test_watcher_json_{}_{}", std::process::id(), unique));
        let _ = std::fs::create_dir_all(&temp_dir);

        let (tx, _rx) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(temp_dir.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        assert_eq!(watcher.get_vault_total_count(), 0);

        // Test export empty vault
        let export_res = watcher.export_vault_json();
        assert!(export_res.is_ok());
        let json_str = export_res.unwrap();
        assert!(json_str.contains("\"records\": []") || json_str.contains("\"records\":[]"));

        // Test import valid JSON with records
        let sample_json = r#"{
            "version": 1,
            "last_updated": "2026-11-01 12:00:00",
            "total_records": 1,
            "records": [
                {
                    "filename": "mail_001.bin",
                    "mail_id": "999901",
                    "battle_type": "barbarian",
                    "time_raw": 1700000000,
                    "time_utc": "2026-11-01 12:00:00",
                    "time_vn": "2026-11-01 19:00:00",
                    "target": "Barbarian Fort 5",
                    "target_level": "5",
                    "talent_add": 0,
                    "pos_x": 100,
                    "pos_y": 200,
                    "damage_pct": 100,
                    "plunder_tier": 0,
                    "items": [],
                    "item_totals": {},
                    "content": ""
                }
            ]
        }"#;

        let import_res = watcher.import_vault_json(sample_json);
        assert_eq!(import_res.unwrap(), 1);
        assert_eq!(watcher.get_vault_total_count(), 1);

        // Test export non-empty vault
        let exported_after = watcher.export_vault_json().unwrap();
        assert!(exported_after.contains("999901"));

        // Test import invalid JSON returns Err
        let bad_import = watcher.import_vault_json("{ not valid json");
        assert!(bad_import.is_err());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_watcher_triggers_sync_on_new_record() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_trig_{}_{}", std::process::id(), unique));
        let mailcache = temp_dir.join("mailcache");
        let _ = std::fs::create_dir_all(&mailcache);

        let (tx_ui, _rx_ui) = std::sync::mpsc::channel();
        let watcher = RoKMailWatcher::new(
            Some(mailcache.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx_ui,
        );

        let (sync_tx, sync_rx) = std::sync::mpsc::channel();
        watcher.set_sync_channel(sync_tx);

        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture_path = manifest_dir.join("tests/fixtures/Persistent.Mail.1002220217856803764");
        let mail_path = mailcache.join("Persistent.Mail.1002220217856803764");
        std::fs::copy(&fixture_path, &mail_path).expect("Failed to copy fixture");

        // First process: new record added -> should send TriggerDelta
        watcher.process_mail_file(&mail_path);

        let cmd = sync_rx.recv_timeout(std::time::Duration::from_millis(500));
        assert!(cmd.is_ok(), "Expected TriggerDelta to be received on new record");
        match cmd.unwrap() {
            crate::cloud_sync::SyncCommand::TriggerDelta => {}
            other => panic!("Expected TriggerDelta, got {:?}", other),
        }

        // Second process: duplicate record -> should NOT send TriggerDelta
        watcher.process_mail_file(&mail_path);
        let second_cmd = sync_rx.try_recv();
        assert!(second_cmd.is_err(), "Duplicate record should not trigger delta sync");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_watcher_sync_channel_resilience_none_and_disconnected() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_res_{}_{}", std::process::id(), unique));
        let mailcache = temp_dir.join("mailcache");
        let _ = std::fs::create_dir_all(&mailcache);

        let (tx_ui, _rx_ui) = std::sync::mpsc::channel();
        let watcher = Watcher::new(
            Some(mailcache.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx_ui,
        );

        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let fixture_path = manifest_dir.join("tests/fixtures/Persistent.Mail.1002220217856803764");
        let mail_path = mailcache.join("Persistent.Mail.1002220217856803764");
        std::fs::copy(&fixture_path, &mail_path).expect("Failed to copy fixture");

        // 1. Without sync channel set: should process safely without error
        watcher.process_mail_file(&mail_path);
        assert_eq!(watcher.get_vault_total_count(), 1);

        // 2. With disconnected sync channel (receiver dropped): should not panic
        let (sync_tx, sync_rx) = std::sync::mpsc::channel();
        drop(sync_rx);
        watcher.set_sync_channel(sync_tx);
        // Force notify:
        watcher.notify_sync_trigger();

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_account_filtering_and_separation() {
        use std::sync::mpsc::channel;
        let temp_dir = std::env::temp_dir().join(format!("rok_test_acc_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let (tx, _rx) = channel();
        let watcher = RoKMailWatcher::new(
            Some(temp_dir.clone()),
            Some(temp_dir.clone()),
            FilterMode::AllTargets,
            LootCatalog::empty(),
            tx,
        );

        let rec_a = BattleRecord {
            mail_id: "m_a".into(),
            account_id: "11111".into(),
            player_name: "GovernorA".into(),
            battle_type: "fort".into(),
            ..Default::default()
        };
        let rec_b = BattleRecord {
            mail_id: "m_b".into(),
            account_id: "22222".into(),
            player_name: "GovernorB".into(),
            battle_type: "fort".into(),
            ..Default::default()
        };

        {
            let mut v = watcher.vault.write().unwrap();
            v.upsert(rec_a);
            v.upsert(rec_b);
        }

        // Initially no account filter -> 2 active records
        watcher.recompute_filtered_records_and_stats();
        assert_eq!(watcher.get_active_count(), 2);

        let accounts = watcher.get_accounts();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0], ("11111".to_string(), "GovernorA (11111)".to_string()));
        assert_eq!(accounts[1], ("22222".to_string(), "GovernorB (22222)".to_string()));

        // Filter for Account A only
        watcher.set_account_filter(Some("11111".to_string()));
        assert_eq!(watcher.get_active_count(), 1);
        assert_eq!(watcher.get_active_records()[0].account_id, "11111");

        // Filter for Account B only
        watcher.set_account_filter(Some("22222".to_string()));
        assert_eq!(watcher.get_active_count(), 1);
        assert_eq!(watcher.get_active_records()[0].account_id, "22222");

        // Clear account filter -> back to 2
        watcher.set_account_filter(None);
        assert_eq!(watcher.get_active_count(), 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
