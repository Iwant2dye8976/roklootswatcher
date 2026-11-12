use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{BattleRecord, FilterMode};
use crate::vault_migrate;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

pub fn extract_receiver_from_raw_bytes(data: &[u8]) -> Option<String> {
    let needle = b"\"receiver\": \"player_";
    if let Some(pos) = data.windows(needle.len()).position(|w| w == needle) {
        let start = pos + needle.len();
        let mut end = start;
        while end < data.len() && data[end] != b'"' {
            end += 1;
        }
        if end > start {
            if let Ok(uid_str) = std::str::from_utf8(&data[start..end]) {
                return Some(uid_str.trim().to_string());
            }
        }
    }
    None
}

pub fn read_file_shared<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let p = path.as_ref();
    let mut options = OpenOptions::new();
    options.read(true);

    #[cfg(windows)]
    {
        options.share_mode(7);
    }

    let mut file = options.open(p)?;
    let file_size = file.metadata()?.len();
    if file_size > crate::lilith_stream::MAX_MAIL_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Mail file exceeds max size limit ({} bytes > {} bytes)",
                file_size,
                crate::lilith_stream::MAX_MAIL_FILE_SIZE
            ),
        ));
    }

    let mut buf = Vec::with_capacity(file_size as usize);
    file.by_ref()
        .take(crate::lilith_stream::MAX_MAIL_FILE_SIZE + 1)
        .read_to_end(&mut buf)?;
    if buf.len() as u64 > crate::lilith_stream::MAX_MAIL_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Mail file exceeds max size limit",
        ));
    }
    Ok(buf)
}

pub fn get_record_dedup_key(r: &BattleRecord) -> String {
    if !r.mail_id.is_empty() {
        format!("id:{}", r.mail_id)
    } else if !r.filename.is_empty() {
        format!("file:{}", r.filename)
    } else {
        format!(
            "sig:{}:{}:{}:{}:{}",
            r.time_raw, r.battle_type, r.pos_x, r.pos_y, r.damage_pct
        )
    }
}

pub fn json_as_i64(v: Option<&serde_json::Value>) -> Option<i64> {
    let val = v?;
    if let Some(i) = val.as_i64() {
        Some(i)
    } else if let Some(u) = val.as_u64() {
        Some(u as i64)
    } else if let Some(f) = val.as_f64() {
        Some(f.round() as i64)
    } else if let Some(s) = val.as_str() {
        s.trim().parse::<f64>().ok().map(|f| f.round() as i64)
    } else {
        None
    }
}

pub fn json_as_u32(v: Option<&serde_json::Value>) -> Option<u32> {
    json_as_i64(v).and_then(|i| if (0..=u32::MAX as i64).contains(&i) { Some(i as u32) } else { None })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStorageData {
    pub version: u32,
    pub last_updated: String,
    pub total_records: usize,
    pub records: Vec<BattleRecord>,
}

pub struct MailVault {
    db_path: PathBuf,
    binary_dir: PathBuf,
    conn: Mutex<Connection>,
    is_corrupted: bool,
    version: std::sync::atomic::AtomicU64,
}

impl std::fmt::Debug for MailVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailVault")
            .field("db_path", &self.db_path)
            .field("binary_dir", &self.binary_dir)
            .field("is_corrupted", &self.is_corrupted)
            .finish()
    }
}

fn row_to_battle_record(row: &rusqlite::Row) -> rusqlite::Result<BattleRecord> {
    let mail_id: String = row.get("mail_id")?;
    let filename: String = row.get("filename")?;
    let battle_type: String = row.get("battle_type")?;
    let time_raw: i64 = row.get("time_raw")?;
    let time_utc: String = row.get("time_utc")?;
    let time_vn: String = row.get("time_vn")?;
    let target: String = row.get("target")?;
    let target_level: String = row.get("target_level")?;
    let talent_add: i64 = row.get("talent_add")?;
    let pos_x: i32 = row.get("pos_x")?;
    let pos_y: i32 = row.get("pos_y")?;
    let damage_pct: i32 = row.get("damage_pct")?;
    let plunder_tier: i32 = row.get("plunder_tier")?;
    let items_json: String = row.get("items_json")?;
    let item_totals_json: String = row.get("item_totals_json")?;
    let content: String = row.get("content")?;

    let items = serde_json::from_str(&items_json).unwrap_or_default();
    let item_totals = serde_json::from_str(&item_totals_json).unwrap_or_default();
    let account_id: String = row.get("account_id").unwrap_or_default();
    let player_name: String = row.get("player_name").unwrap_or_default();
    let pvp_combat_type: String = row.get("pvp_combat_type").unwrap_or_default();
    let primary_commander_id: u32 = row.get("primary_commander_id").unwrap_or_default();
    let primary_commander_name: String = row.get("primary_commander_name").unwrap_or_default();
    let secondary_commander_id: u32 = row.get("secondary_commander_id").unwrap_or_default();
    let secondary_commander_name: String = row.get("secondary_commander_name").unwrap_or_default();
    let target_primary_commander_id: u32 = row.get("target_primary_commander_id").unwrap_or_default();
    let target_primary_commander_name: String = row.get("target_primary_commander_name").unwrap_or_default();
    let target_secondary_commander_id: u32 = row.get("target_secondary_commander_id").unwrap_or_default();
    let target_secondary_commander_name: String = row.get("target_secondary_commander_name").unwrap_or_default();
    let primary_is_expertised: bool = row.get("primary_is_expertised").map(|v: i32| v != 0).unwrap_or(false);
    let secondary_is_expertised: bool = row.get("secondary_is_expertised").map(|v: i32| v != 0).unwrap_or(false);
    let target_primary_is_expertised: bool = row.get("target_primary_is_expertised").map(|v: i32| v != 0).unwrap_or(false);
    let target_secondary_is_expertised: bool = row.get("target_secondary_is_expertised").map(|v: i32| v != 0).unwrap_or(false);
    let target_player_name: String = row.get("target_player_name").unwrap_or_default();
    let kp_gained: f64 = row.get("kp_gained").unwrap_or_default();
    let kp_lost: f64 = row.get("kp_lost").unwrap_or_default();
    let dead_dealt: u64 = row.get("dead_dealt").unwrap_or_default();
    let dead_taken: u64 = row.get("dead_taken").unwrap_or_default();
    let severely_wounded_dealt: u64 = row.get("severely_wounded_dealt").unwrap_or_default();
    let severely_wounded_taken: u64 = row.get("severely_wounded_taken").unwrap_or_default();

    Ok(BattleRecord {
        filename,
        mail_id,
        battle_type,
        time_raw,
        time_utc,
        time_vn,
        target,
        target_level,
        talent_add,
        pos_x,
        pos_y,
        damage_pct,
        plunder_tier,
        items,
        item_totals,
        content,
        account_id,
        player_name,
        pvp_combat_type,
        primary_commander_id,
        primary_commander_name,
        secondary_commander_id,
        secondary_commander_name,
        target_primary_commander_id,
        target_primary_commander_name,
        target_secondary_commander_id,
        target_secondary_commander_name,
        primary_is_expertised,
        secondary_is_expertised,
        target_primary_is_expertised,
        target_secondary_is_expertised,
        target_player_name,
        kp_gained,
        kp_lost,
        dead_dealt,
        dead_taken,
        severely_wounded_dealt,
        severely_wounded_taken,
    })
}

impl MailVault {
    pub fn new(out_dir: &Path) -> Self {
        let db_path = out_dir.join("rok_history_vault.db");
        let binary_dir = out_dir.join("mail_vault");

        let _ = fs::create_dir_all(out_dir);
        if let Err(e) = fs::create_dir_all(&binary_dir) {
            eprintln!("[-] Không thể tạo thư mục mail_vault: {}", e);
        }

        let (conn, mut is_corrupted) = match Connection::open(&db_path) {
            Ok(c) => (c, false),
            Err(e) => {
                eprintln!("[!] Không thể mở SQLite database {:?}: {}", db_path, e);
                let mem = Connection::open_in_memory().unwrap_or_else(|_| panic!("Failed to open in-memory SQLite"));
                (mem, true)
            }
        };

        if !is_corrupted {
            if let Err(e) = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA busy_timeout = 5000;") {
                eprintln!("[!] Cảnh báo WAL mode: {}", e);
            }
            if let Err(e) = vault_migrate::create_schema(&conn) {
                eprintln!("[!] Cảnh báo khởi tạo schema: {}", e);
                let corrupt_path = db_path.with_extension("corrupt");
                let _ = fs::copy(&db_path, &corrupt_path);
                is_corrupted = true;
            }
        }

        let mut vault = Self {
            db_path,
            binary_dir,
            conn: Mutex::new(conn),
            is_corrupted,
            version: std::sync::atomic::AtomicU64::new(1),
        };

        vault.init(out_dir);
        vault
    }

    pub fn version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn bump_version(&self) {
        self.version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn init(&mut self, out_dir: &Path) {
        if self.is_corrupted {
            return;
        }

        let json_path = out_dir.join("rok_history_vault.json");
        if json_path.exists() {
            let count: i64 = {
                let conn = self.conn();
                conn.query_row("SELECT COUNT(*) FROM battle_records", [], |r| r.get(0))
                    .unwrap_or(0)
            };
            if count == 0 {
                let mig_res = {
                    let conn = self.conn();
                    vault_migrate::migrate_from_json(&json_path, &conn)
                };
                match mig_res {
                    Ok(_) => {
                        let migrated_path = out_dir.join("rok_history_vault.json.migrated");
                        let _ = fs::rename(&json_path, &migrated_path);
                    }
                    Err(e) => {
                        eprintln!("[!] Cảnh báo Vault: {}", e);
                        self.is_corrupted = true;
                        return;
                    }
                }
            }
        }

        self.restore_from_filtered_mails_json();
        self.sanitize_records();
    }

    pub fn get_vault_file(&self) -> &Path {
        &self.db_path
    }

    pub fn get_binary_dir(&self) -> &Path {
        &self.binary_dir
    }

    pub fn get_records(&self) -> Vec<BattleRecord> {
        let conn = self.conn();
        let mut stmt = match conn.prepare("SELECT * FROM battle_records ORDER BY time_raw ASC") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[!] Lỗi prepare get_records: {}", e);
                return Vec::new();
            }
        };

        let rows = match stmt.query_map([], row_to_battle_record) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[!] Lỗi query get_records: {}", e);
                return Vec::new();
            }
        };

        rows.flatten().collect()
    }

    pub fn total_count(&self) -> usize {
        let conn = self.conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM battle_records", [], |r| r.get(0))
            .unwrap_or(0);
        count as usize
    }

    pub fn contains_file_or_id(&self, fname: &str) -> bool {
        let id_part = fname.strip_prefix("Persistent.Mail.").unwrap_or(fname);
        let p1 = fname;
        let p2 = format!("file:{}", fname);
        let p3 = format!("id:{}", id_part);
        let p4 = format!("id:{}", fname);

        let conn = self.conn();
        let mut stmt = match conn.prepare_cached(
            "SELECT 1 FROM battle_records WHERE dedup_key = ?1 OR dedup_key = ?2 OR dedup_key = ?3 OR dedup_key = ?4 LIMIT 1"
        ) {
            Ok(s) => s,
            Err(_) => return false,
        };

        stmt.exists(params![p1, p2, p3, p4]).unwrap_or(false)
    }

    pub fn record_has_account_id(&self, fname: &str) -> bool {
        let conn = self.conn();
        conn.query_row(
            "SELECT 1 FROM battle_records WHERE (filename = ?1 OR mail_id = ?1) AND account_id IS NOT NULL AND account_id != '' LIMIT 1",
            params![fname],
            |_| Ok(()),
        )
        .is_ok()
    }

    pub fn is_file_ignored(&self, fname: &str, main_uids: &[String]) -> bool {
        let conn = self.conn();
        let acc: Option<String> = conn
            .query_row(
                "SELECT account_id FROM ignored_mail_files WHERE filename = ?1 LIMIT 1",
                params![fname],
                |r| r.get(0),
            )
            .ok();

        if let Some(acc_id) = acc {
            let clean_acc = acc_id.strip_prefix("player_").unwrap_or(&acc_id).trim();
            !main_uids.iter().any(|u| {
                let clean_u = u.strip_prefix("player_").unwrap_or(u).trim();
                clean_u == clean_acc
            })
        } else {
            false
        }
    }

    pub fn mark_files_ignored(&self, files: &[(String, String)]) {
        if files.is_empty() {
            return;
        }
        let mut conn = self.conn();
        if let Ok(tx) = conn.transaction() {
            if let Ok(mut stmt) = tx.prepare_cached(
                "INSERT OR REPLACE INTO ignored_mail_files (filename, account_id) VALUES (?1, ?2)",
            ) {
                for (fname, acc_id) in files {
                    let _ = stmt.execute(params![fname, acc_id]);
                }
            }
            let _ = tx.commit();
        };
    }

    pub fn clear_ignored_files(&self) {
        let conn = self.conn();
        let _ = conn.execute("DELETE FROM ignored_mail_files", []);
    }

    pub fn backfill_missing_account_ids(&mut self, out_dir: &Path) -> usize {
        let empty_records: Vec<(String, String, String)> = {
            let conn = self.conn();
            let mut stmt = match conn.prepare(
                "SELECT dedup_key, filename, mail_id FROM battle_records WHERE account_id IS NULL OR account_id = ''"
            ) {
                Ok(s) => s,
                Err(_) => return 0,
            };
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            });
            match rows {
                Ok(r) => r.flatten().collect(),
                Err(_) => Vec::new(),
            }
        };

        if empty_records.is_empty() {
            return 0;
        }

        let default_mail_dir = PathBuf::from(r"C:\Program Files (x86)\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache");
        let mut bin_map: HashMap<String, PathBuf> = HashMap::new();
        for d in [&self.binary_dir, &default_mail_dir] {
            if let Ok(entries) = fs::read_dir(d) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            bin_map.entry(name.to_string()).or_insert(p);
                        }
                    }
                }
            }
        }

        let json_dir = out_dir.join("filtered_mails_json");
        let mut json_map: HashMap<String, PathBuf> = HashMap::new();
        if let Ok(entries) = fs::read_dir(&json_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        json_map.entry(name.to_string()).or_insert(p);
                    }
                }
            }
        }

        let mut updates: Vec<(String, String)> = Vec::new();

        for (dedup_key, filename, mail_id) in empty_records {
            let mut found_account: Option<String> = None;

            let mut bin_keys = vec![filename.clone()];
            if !mail_id.is_empty() {
                bin_keys.push(format!("Persistent.Mail.{}", mail_id));
            }

            for key in bin_keys {
                if let Some(cand) = bin_map.get(&key) {
                    if let Ok(bytes) = read_file_shared(cand) {
                        if let Some(uid) = extract_receiver_from_raw_bytes(&bytes) {
                            found_account = Some(uid);
                            break;
                        }
                    }
                }
            }

            if found_account.is_none() {
                let mut json_keys = vec![format!("{}.json", filename)];
                if !mail_id.is_empty() {
                    json_keys.push(format!("Persistent.Mail.{}.json", mail_id));
                }

                for key in json_keys {
                    if let Some(cand) = json_map.get(&key) {
                        if let Ok(content) = fs::read_to_string(cand) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(rec_str) = val.get("receiver").and_then(|r| r.as_str()) {
                                    let clean = rec_str.strip_prefix("player_").unwrap_or(rec_str).trim();
                                    if !clean.is_empty() {
                                        found_account = Some(clean.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(uid) = found_account {
                updates.push((dedup_key, uid));
            }
        }

        if updates.is_empty() {
            return 0;
        }

        let updated_count = updates.len();
        let mut conn = self.conn();
        if let Ok(tx) = conn.transaction() {
            if let Ok(mut stmt) = tx.prepare_cached("UPDATE battle_records SET account_id = ?1 WHERE dedup_key = ?2") {
                for (dedup_key, uid) in updates {
                    let _ = stmt.execute(params![uid, dedup_key]);
                }
            }
            let _ = tx.commit();
        }

        updated_count
    }

    pub fn count_records_except_uids(&self, main_uids: &[String]) -> usize {
        if main_uids.is_empty() {
            return 0;
        }
        let clean_uids: Vec<String> = main_uids
            .iter()
            .map(|u| u.strip_prefix("player_").unwrap_or(u).trim().to_string())
            .filter(|u| !u.is_empty())
            .collect();
        if clean_uids.is_empty() {
            return 0;
        }

        let conn = self.conn();
        let placeholders: Vec<String> = (1..=clean_uids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT COUNT(*) FROM battle_records WHERE REPLACE(COALESCE(account_id, ''), 'player_', '') NOT IN ({})",
            placeholders.join(", ")
        );
        let params: Vec<&dyn rusqlite::ToSql> = clean_uids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        conn.query_row(&sql, rusqlite::params_from_iter(params), |r| r.get::<_, usize>(0)).unwrap_or(0)
    }

    pub fn delete_records_except_uids(&mut self, main_uids: &[String]) -> (usize, Vec<String>) {
        if main_uids.is_empty() {
            return (0, Vec::new());
        }
        let clean_uids: Vec<String> = main_uids
            .iter()
            .map(|u| u.strip_prefix("player_").unwrap_or(u).trim().to_string())
            .filter(|u| !u.is_empty())
            .collect();
        if clean_uids.is_empty() {
            return (0, Vec::new());
        }

        let mut conn = self.conn();
        let placeholders: Vec<String> = (1..=clean_uids.len()).map(|i| format!("?{}", i)).collect();
        let select_files_sql = format!(
            "SELECT filename, dedup_key FROM battle_records WHERE REPLACE(COALESCE(account_id, ''), 'player_', '') NOT IN ({})",
            placeholders.join(", ")
        );
        let params: Vec<&dyn rusqlite::ToSql> = clean_uids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        // 1. Collect all filenames and dedup_keys of non-main records to delete from disk caches
        let (files_to_delete, keys_to_delete): (Vec<String>, Vec<String>) = {
            if let Ok(mut stmt) = conn.prepare(&select_files_sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(params.iter().copied()), |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    rows.flatten().unzip()
                } else {
                    (Vec::new(), Vec::new())
                }
            } else {
                (Vec::new(), Vec::new())
            }
        };

        // 2. Perform deletion inside an immediate transaction
        let delete_sql = format!(
            "DELETE FROM battle_records WHERE REPLACE(COALESCE(account_id, ''), 'player_', '') NOT IN ({})",
            placeholders.join(", ")
        );

        let deleted = {
            if let Ok(tx) = conn.transaction() {
                let n = tx.execute(&delete_sql, rusqlite::params_from_iter(params)).unwrap_or(0);
                let _ = tx.commit();
                n
            } else {
                0
            }
        };

        if deleted == 0 {
            return (0, Vec::new());
        }

        self.bump_version();

        // 3. Remove raw binary files from mail_vault
        for fname in &files_to_delete {
            let p = self.binary_dir.join(fname);
            if p.is_file() {
                let _ = fs::remove_file(p);
            }
        }

        // 4. Remove exported JSON files from filtered_mails_json so they never restore
        if let Some(parent) = self.db_path.parent() {
            let json_dir = parent.join("filtered_mails_json");
            for fname in &files_to_delete {
                let p = json_dir.join(format!("{}.json", fname));
                if p.is_file() {
                    let _ = fs::remove_file(p);
                }
            }
        }

        // 5. Force WAL checkpoint to flush to disk
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

        (deleted, keys_to_delete)
    }

    fn upsert_inner(conn: &Connection, rec: BattleRecord) -> bool {
        let dedup_key = get_record_dedup_key(&rec);

        let existing: Option<BattleRecord> = {
            let mut select_stmt = match conn.prepare_cached(
                "SELECT * FROM battle_records WHERE dedup_key = ?1"
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[!] Lỗi prepare select for upsert: {}", e);
                    return false;
                }
            };

            select_stmt
                .query_row(params![&dedup_key], row_to_battle_record)
                .optional()
                .unwrap_or(None)
        };

        if let Some(mut existing) = existing {
            let mut modified = false;
            if rec.battle_type != "other" && (existing.battle_type == "other" || existing.battle_type.is_empty()) {
                existing.battle_type = rec.battle_type;
                modified = true;
            }
            if !rec.items.is_empty()
                && (existing.items.is_empty()
                    || existing.items != rec.items
                    || existing.item_totals != rec.item_totals)
            {
                existing.items = rec.items;
                existing.item_totals = rec.item_totals;
                modified = true;
            }
            if rec.damage_pct > 0 && existing.damage_pct == 0 {
                existing.damage_pct = rec.damage_pct;
                modified = true;
            }
            if rec.plunder_tier > 0 && existing.plunder_tier == 0 {
                existing.plunder_tier = rec.plunder_tier;
                modified = true;
            }
            if !rec.target.is_empty() && existing.target.is_empty() {
                existing.target = rec.target;
                modified = true;
            }
            if !rec.content.is_empty() && existing.content.is_empty() {
                existing.content = rec.content;
                modified = true;
            }
            if rec.pos_x != 0 && existing.pos_x == 0 {
                existing.pos_x = rec.pos_x;
                existing.pos_y = rec.pos_y;
                modified = true;
            }
            if !rec.time_vn.is_empty() && existing.time_vn.is_empty() {
                existing.time_vn = rec.time_vn;
                modified = true;
            }
            if rec.time_raw > 0 && existing.time_raw == 0 {
                existing.time_raw = rec.time_raw;
                modified = true;
            }
            if !rec.account_id.is_empty() && existing.account_id != rec.account_id {
                existing.account_id = rec.account_id;
                modified = true;
            }
            if !rec.player_name.is_empty() && existing.player_name != rec.player_name {
                existing.player_name = rec.player_name;
                modified = true;
            }

            if modified {
                let items_json = serde_json::to_string(&existing.items).unwrap_or_else(|_| "[]".to_string());
                let item_totals_json = serde_json::to_string(&existing.item_totals).unwrap_or_else(|_| "{}".to_string());

                let mut update_stmt = match conn.prepare_cached(
                    "UPDATE battle_records SET
                        battle_type = ?1,
                        damage_pct = ?2,
                        plunder_tier = ?3,
                        target = ?4,
                        content = ?5,
                        pos_x = ?6,
                        pos_y = ?7,
                        time_vn = ?8,
                        time_raw = ?9,
                        items_json = ?10,
                        item_totals_json = ?11,
                        account_id = ?12,
                        player_name = ?13,
                        synced_at = NULL
                    WHERE dedup_key = ?14"
                ) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[!] Lỗi prepare update for upsert: {}", e);
                        return false;
                    }
                };

                if let Err(e) = update_stmt.execute(params![
                    existing.battle_type,
                    existing.damage_pct,
                    existing.plunder_tier,
                    existing.target,
                    existing.content,
                    existing.pos_x,
                    existing.pos_y,
                    existing.time_vn,
                    existing.time_raw,
                    items_json,
                    item_totals_json,
                    existing.account_id,
                    existing.player_name,
                    dedup_key,
                ]) {
                    eprintln!("[!] Lỗi execute update for upsert: {}", e);
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            let items_json = serde_json::to_string(&rec.items).unwrap_or_else(|_| "[]".to_string());
            let item_totals_json = serde_json::to_string(&rec.item_totals).unwrap_or_else(|_| "{}".to_string());

            let mut insert_stmt = match conn.prepare_cached(
                "INSERT INTO battle_records (
                    mail_id, filename, battle_type, time_raw, time_utc, time_vn,
                    target, target_level, talent_add, pos_x, pos_y, damage_pct,
                    plunder_tier, items_json, item_totals_json, content, dedup_key, synced_at,
                    account_id, player_name,
                    pvp_combat_type, primary_commander_id, primary_commander_name,
                    secondary_commander_id, secondary_commander_name,
                    target_primary_commander_id, target_primary_commander_name,
                    target_secondary_commander_id, target_secondary_commander_name,
                    primary_is_expertised, secondary_is_expertised,
                    target_primary_is_expertised, target_secondary_is_expertised,
                    target_player_name,
                    kp_gained, kp_lost, dead_dealt, dead_taken,
                    severely_wounded_dealt, severely_wounded_taken
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, NULL, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39)"
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[!] Lỗi prepare insert for upsert: {}", e);
                    return false;
                }
            };

            match insert_stmt.execute(params![
                rec.mail_id,
                rec.filename,
                rec.battle_type,
                rec.time_raw,
                rec.time_utc,
                rec.time_vn,
                rec.target,
                rec.target_level,
                rec.talent_add,
                rec.pos_x,
                rec.pos_y,
                rec.damage_pct,
                rec.plunder_tier,
                items_json,
                item_totals_json,
                rec.content,
                dedup_key,
                rec.account_id,
                rec.player_name,
                rec.pvp_combat_type,
                rec.primary_commander_id,
                rec.primary_commander_name,
                rec.secondary_commander_id,
                rec.secondary_commander_name,
                rec.target_primary_commander_id,
                rec.target_primary_commander_name,
                rec.target_secondary_commander_id,
                rec.target_secondary_commander_name,
                rec.primary_is_expertised as i32,
                rec.secondary_is_expertised as i32,
                rec.target_primary_is_expertised as i32,
                rec.target_secondary_is_expertised as i32,
                rec.target_player_name,
                rec.kp_gained,
                rec.kp_lost,
                rec.dead_dealt,
                rec.dead_taken,
                rec.severely_wounded_dealt,
                rec.severely_wounded_taken,
            ]) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("[!] Lỗi execute insert for upsert: {}", e);
                    false
                }
            }
        }
    }

    pub fn upsert(&mut self, rec: BattleRecord) -> bool {
        let conn = self.conn();
        let is_new = Self::upsert_inner(&conn, rec);
        if is_new {
            self.bump_version();
        }
        is_new
    }

    pub fn upsert_batch(&mut self, recs: Vec<BattleRecord>) -> usize {
        if recs.is_empty() {
            return 0;
        }
        let mut conn = self.conn();
        let tx = match conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[!] Lỗi khởi tạo transaction trong upsert_batch: {}", e);
                return 0;
            }
        };

        let mut new_count = 0;
        for r in recs {
            if Self::upsert_inner(&tx, r) {
                new_count += 1;
            }
        }

        if let Err(e) = tx.commit() {
            eprintln!("[!] Lỗi commit transaction trong upsert_batch: {}", e);
            return 0;
        }
        drop(conn);

        if new_count > 0 {
            self.bump_version();
            let _ = self.save_to_disk();
        }
        new_count
    }

    pub fn get_unsynced_records(&self, limit: usize) -> rusqlite::Result<Vec<(String, BattleRecord)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare_cached(
            "SELECT * FROM battle_records WHERE synced_at IS NULL ORDER BY time_raw ASC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let dedup_key: String = row.get("dedup_key")?;
            let rec = row_to_battle_record(row)?;
            Ok((dedup_key, rec))
        })?;
        rows.collect()
    }

    pub fn mark_records_synced(&self, dedup_keys: &[String], timestamp: i64) -> rusqlite::Result<usize> {
        if dedup_keys.is_empty() {
            return Ok(0);
        }
        let mut conn = self.conn();
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut updated = 0;
        {
            let mut stmt = tx.prepare_cached(
                "UPDATE battle_records SET synced_at = ?1 WHERE dedup_key = ?2"
            )?;
            for key in dedup_keys {
                updated += stmt.execute(params![timestamp, key])?;
            }
        }
        tx.commit()?;
        Ok(updated)
    }

    pub fn get_sync_counts(&self) -> rusqlite::Result<(usize, usize)> {
        let conn = self.conn();
        conn.query_row(
            "SELECT COUNT(*), COUNT(CASE WHEN synced_at IS NULL THEN 1 END) FROM battle_records",
            [],
            |row| {
                let total: i64 = row.get(0)?;
                let unsynced: i64 = row.get(1)?;
                Ok((total as usize, unsynced as usize))
            },
        )
    }

    pub fn get_cloud_user_id(&self) -> rusqlite::Result<Option<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare_cached(
            "SELECT value FROM vault_meta WHERE key = 'synced_user_id'"
        )?;
        match stmt.query_row([], |row| row.get::<_, String>(0)) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_cloud_user_id(&self, user_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO vault_meta (key, value) VALUES ('synced_user_id', ?1)",
            params![user_id],
        )?;
        Ok(())
    }

    pub fn restore_records_batch(&self, recs: &[BattleRecord], timestamp: Option<i64>) -> rusqlite::Result<usize> {
        if recs.is_empty() {
            return Ok(0);
        }
        let mut conn = self.conn();
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut restored = 0;
        {
            let mut insert_stmt = tx.prepare_cached(
                "INSERT INTO battle_records (
                    mail_id, filename, battle_type, time_raw, time_utc, time_vn,
                    target, target_level, talent_add, pos_x, pos_y, damage_pct,
                    plunder_tier, items_json, item_totals_json, content, dedup_key, synced_at,
                    account_id, player_name,
                    pvp_combat_type, primary_commander_id, primary_commander_name,
                    secondary_commander_id, secondary_commander_name,
                    target_primary_commander_id, target_primary_commander_name,
                    target_secondary_commander_id, target_secondary_commander_name,
                    primary_is_expertised, secondary_is_expertised,
                    target_primary_is_expertised, target_secondary_is_expertised,
                    target_player_name,
                    kp_gained, kp_lost, dead_dealt, dead_taken,
                    severely_wounded_dealt, severely_wounded_taken
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40)
                ON CONFLICT(dedup_key) DO UPDATE SET
                    synced_at = COALESCE(battle_records.synced_at, excluded.synced_at),
                    account_id = CASE WHEN battle_records.account_id IS NULL OR battle_records.account_id = '' THEN excluded.account_id ELSE battle_records.account_id END,
                    player_name = CASE WHEN battle_records.player_name IS NULL OR battle_records.player_name = '' THEN excluded.player_name ELSE battle_records.player_name END"
            )?;
            for r in recs {
                let dedup_key = get_record_dedup_key(r);
                let items_json = serde_json::to_string(&r.items).unwrap_or_else(|_| "[]".to_string());
                let item_totals_json = serde_json::to_string(&r.item_totals).unwrap_or_else(|_| "{}".to_string());
                let changed = insert_stmt.execute(params![
                    r.mail_id,
                    r.filename,
                    r.battle_type,
                    r.time_raw,
                    r.time_utc,
                    r.time_vn,
                    r.target,
                    r.target_level,
                    r.talent_add,
                    r.pos_x,
                    r.pos_y,
                    r.damage_pct,
                    r.plunder_tier,
                    items_json,
                    item_totals_json,
                    r.content,
                    dedup_key,
                    timestamp,
                    r.account_id,
                    r.player_name,
                    r.pvp_combat_type,
                    r.primary_commander_id,
                    r.primary_commander_name,
                    r.secondary_commander_id,
                    r.secondary_commander_name,
                    r.target_primary_commander_id,
                    r.target_primary_commander_name,
                    r.target_secondary_commander_id,
                    r.target_secondary_commander_name,
                    r.primary_is_expertised as i32,
                    r.secondary_is_expertised as i32,
                    r.target_primary_is_expertised as i32,
                    r.target_secondary_is_expertised as i32,
                    r.target_player_name,
                    r.kp_gained,
                    r.kp_lost,
                    r.dead_dealt,
                    r.dead_taken,
                    r.severely_wounded_dealt,
                    r.severely_wounded_taken,
                ])?;
                if changed > 0 {
                    restored += 1;
                }
            }
        }
        tx.commit()?;
        drop(conn);
        if restored > 0 {
            self.bump_version();
        }
        let _ = self.save_to_disk();
        Ok(restored)
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        if self.is_corrupted {
            return Err(format!(
                "Từ chối ghi đè file vault: file gốc {:?} bị hỏng hoặc lỗi phân tích. Vui lòng kiểm tra bản sao lưu .corrupt",
                self.db_path
            ));
        }
        let conn = self.conn();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")
            .map_err(|e| format!("WAL checkpoint error: {}", e))
    }

    #[allow(dead_code)]
    pub fn is_corrupted(&self) -> bool {
        self.is_corrupted
    }

    pub fn query_latest_filtered(&self, filter: FilterMode, limit: usize) -> Vec<BattleRecord> {
        let sql = match filter {
            FilterMode::AllRewards => {
                "SELECT * FROM battle_records WHERE battle_type != 'pvp' ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::AllTargets => {
                "SELECT * FROM battle_records WHERE battle_type IN ('motte', 'fort', 'barbarian') ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::AllForts => {
                "SELECT * FROM battle_records WHERE battle_type IN ('motte', 'fort') ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::Motte => {
                "SELECT * FROM battle_records WHERE battle_type = 'motte' ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::Fort => {
                "SELECT * FROM battle_records WHERE battle_type = 'fort' ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::Barbarian => {
                "SELECT * FROM battle_records WHERE battle_type = 'barbarian' ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::Gathering => {
                "SELECT * FROM battle_records WHERE battle_type = 'gathering' ORDER BY time_raw DESC LIMIT ?1"
            }
            FilterMode::PvP => {
                "SELECT * FROM battle_records WHERE battle_type = 'pvp' ORDER BY time_raw DESC LIMIT ?1"
            }
        };

        let conn = self.conn();
        let mut stmt = match conn.prepare_cached(sql) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[!] Lỗi prepare query_latest_filtered: {}", e);
                return Vec::new();
            }
        };

        let rows = match stmt.query_map(params![limit as i64], row_to_battle_record) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[!] Lỗi execute query_latest_filtered: {}", e);
                return Vec::new();
            }
        };

        rows.flatten().collect()
    }

    #[allow(dead_code)]
    pub fn export_vault_json(&self) -> Result<String, String> {
        let records = self.get_records();
        let total_records = records.len();
        let data = VaultStorageData {
            version: 1,
            last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_records,
            records,
        };
        serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize vault JSON: {}", e))
    }

    #[allow(dead_code)]
    pub fn import_records_from_json(&mut self, json: &str) -> Result<usize, String> {
        let clean_json = json.trim_start_matches('\u{feff}');
        let data: VaultStorageData = serde_json::from_str(clean_json)
            .map_err(|e| format!("Failed to parse vault JSON: {}", e))?;
        let count = self.upsert_batch(data.records);
        Ok(count)
    }

    pub fn sanitize_records(&mut self) -> usize {
        struct Cand {
            dedup_key: String,
            mail_id: String,
            target: String,
            content: String,
            target_level: String,
            items_json: String,
            pos_x: i32,
            pos_y: i32,
        }

        let (to_delete, to_update) = {
            let conn = self.conn();
            let mut stmt = match conn.prepare_cached(
                "SELECT dedup_key, mail_id, target, content, target_level, items_json, pos_x, pos_y
                 FROM battle_records
                 WHERE battle_type = 'barbarian'"
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[!] Lỗi prepare sanitize_records: {}", e);
                    return 0;
                }
            };

            let rows = match stmt.query_map([], |row| {
                Ok(Cand {
                    dedup_key: row.get(0)?,
                    mail_id: row.get(1)?,
                    target: row.get(2)?,
                    content: row.get(3)?,
                    target_level: row.get(4)?,
                    items_json: row.get(5)?,
                    pos_x: row.get(6)?,
                    pos_y: row.get(7)?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[!] Lỗi query sanitize_records: {}", e);
                    return 0;
                }
            };

            let mut to_delete = Vec::new();
            let mut to_update = Vec::new();

            for r in rows.flatten() {
                if r.target.contains("Pass")
                    || r.content.contains("Pass")
                    || r.mail_id == "57774831178214367624"
                    || r.mail_id == "57774881178214367624"
                {
                    to_delete.push(r.dedup_key);
                    continue;
                }
                let items_empty = r.items_json == "[]" || r.items_json.trim().is_empty();
                if (r.target_level == "60" || r.target == "Dã Man Cấp 60") && items_empty {
                    to_delete.push(r.dedup_key);
                    continue;
                }
                if r.target_level.contains("60") || r.target.contains("60") {
                    let maybe_min_lvl = r.target_level.split_once("-60").map(|(min_str, _)| min_str.trim().to_string())
                        .or_else(|| {
                            if let Some(pos) = r.target.find("-60") {
                                let prefix = &r.target[..pos];
                                prefix.split(|c: char| !c.is_numeric()).rfind(|s| !s.is_empty()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        });
                    if let Some(min_lvl) = maybe_min_lvl {
                        if !min_lvl.is_empty() {
                            let new_target = format!("Dã Man Cấp {}", min_lvl);
                            let new_content = if r.pos_x > 0 && r.pos_y > 0 {
                                format!("{} tại X:{} Y:{}", new_target, r.pos_x, r.pos_y)
                            } else {
                                new_target.clone()
                            };
                            to_update.push((r.dedup_key, new_target, min_lvl, new_content));
                        } else {
                            let new_target = "Dã Man".to_string();
                            let new_target_level = String::new();
                            let new_content = new_target.clone();
                            to_update.push((r.dedup_key, new_target, new_target_level, new_content));
                        }
                    } else if r.target_level == "60" || r.target.contains("Cấp 60") {
                        let new_target = "Dã Man".to_string();
                        let new_target_level = String::new();
                        let new_content = new_target.clone();
                        to_update.push((r.dedup_key, new_target, new_target_level, new_content));
                    }
                }
            }
            (to_delete, to_update)
        };

        let removed = to_delete.len();
        if !to_delete.is_empty() || !to_update.is_empty() {
            let mut conn = self.conn();
            let tx = match conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("[!] Lỗi khởi tạo transaction trong sanitize_records: {}", e);
                    return 0;
                }
            };

            for key in &to_delete {
                if let Err(e) = tx.execute("DELETE FROM battle_records WHERE dedup_key = ?1", params![key]) {
                    eprintln!("[!] Lỗi execute DELETE trong sanitize_records: {}", e);
                }
            }
            for (key, new_tgt, new_lvl, new_cnt) in &to_update {
                if let Err(e) = tx.execute(
                    "UPDATE battle_records SET target = ?1, target_level = ?2, content = ?3 WHERE dedup_key = ?4",
                    params![new_tgt, new_lvl, new_cnt, key],
                ) {
                    eprintln!("[!] Lỗi execute UPDATE trong sanitize_records: {}", e);
                }
            }

            if let Err(e) = tx.commit() {
                eprintln!("[!] Lỗi commit transaction trong sanitize_records: {}", e);
                return 0;
            }
            drop(conn);
            let _ = self.save_to_disk();
        }

        removed
    }

    #[allow(dead_code)]
    pub fn backup_binary_mail(&self, src_path: &Path) -> Result<PathBuf, String> {
        let fname = match src_path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return Err("Đường dẫn file không hợp lệ".into()),
        };

        if !fname.starts_with("Persistent.Mail.") {
            return Err("Không phải file Persistent.Mail".into());
        }

        let dst_path = self.binary_dir.join(&fname);

        let bytes = read_file_shared(src_path)
            .map_err(|e| format!("Lỗi đọc file nguồn {}: {}", src_path.display(), e))?;

        if bytes.is_empty() {
            return Err("File rỗng (0 bytes)".into());
        }

        fs::write(&dst_path, &bytes)
            .map_err(|e| format!("Lỗi ghi file sao lưu {}: {}", dst_path.display(), e))?;

        Ok(dst_path)
    }

    pub fn clean_binary_vault(&self) -> Result<(usize, u64), String> {
        if !self.binary_dir.exists() {
            return Ok((0, 0));
        }

        let mut deleted_count = 0;
        let mut bytes_freed = 0;

        let entries = fs::read_dir(&self.binary_dir)
            .map_err(|e| format!("Không thể đọc thư mục mail_vault: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    deleted_count += 1;
                    bytes_freed += len;
                }
            }
        }

        Ok((deleted_count, bytes_freed))
    }

    fn restore_from_filtered_mails_json(&mut self) -> usize {
        let parent = match self.db_path.parent() {
            Some(p) => p,
            None => return 0,
        };
        let json_dir = parent.join("filtered_mails_json");
        if !json_dir.is_dir() {
            return 0;
        }

        let mut restored = 0;
        if let Ok(entries) = fs::read_dir(&json_dir) {
            for entry in entries.flatten() {
                let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                if !is_file {
                    continue;
                }

                let fname_os = entry.file_name();
                let fname_str = fname_os.to_string_lossy();
                let stem = match fname_str.strip_suffix(".json") {
                    Some(s) => s,
                    None => continue,
                };

                if self.contains_file_or_id(stem) {
                    continue;
                }

                let p = entry.path();
                if let Ok(file) = fs::File::open(&p) {
                    let reader = std::io::BufReader::new(file);
                    if let Ok(val) = serde_json::from_reader::<_, serde_json::Value>(reader) {
                        if let Some(box_val) = val.get("box").and_then(|v| v.as_str()) {
                            if box_val == "Report" || box_val == "Archive" {
                                let fname_saved = stem.to_string();
                                if let Some(rec) = Self::parse_battle_record_from_exported_json(&val, &fname_saved) {
                                    if self.upsert(rec) {
                                        restored += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if restored > 0 {
            let _ = self.save_to_disk();
        }
        restored
    }

    fn parse_battle_record_from_exported_json(val: &serde_json::Value, fname: &str) -> Option<BattleRecord> {
        let body = val.get("body")?;
        let sub_t = json_as_i64(body.get("subType")).unwrap_or(0);
        let sub_p = json_as_i64(body.get("subParam")).unwrap_or(0);

        let is_battle = val.get("type").and_then(|v| v.as_str()) == Some("Battle")
            || body.get("content").and_then(|c| c.get("Attacks")).is_some();

        let time_raw = json_as_i64(val.get("sendTime"))
            .or_else(|| json_as_i64(body.get("time")))
            .or_else(|| json_as_i64(val.get("time")))
            .unwrap_or(0);

        let mail_id = val.get("id")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| json_as_i64(Some(v)).map(|n| n.to_string())))
            .unwrap_or_default();

        let mut damage_pct = 0;
        let mut plunder_tier = 0;
        let mut time_vn = String::new();

        if let Some(meta) = val.get("_stats_metadata") {
            damage_pct = json_as_i64(meta.get("damage_pct")).unwrap_or(0) as i32;
            plunder_tier = json_as_i64(meta.get("plunder_tier")).unwrap_or(0) as i32;
            time_vn = meta.get("time_vn").and_then(|v| v.as_str()).unwrap_or("").to_string();
        }

        let account_id = val.get("account_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .or_else(|| {
                val.get("receiver")
                    .and_then(|r| r.as_str())
                    .map(|r| r.strip_prefix("player_").unwrap_or(r).trim().to_string())
            })
            .unwrap_or_default();
        let player_name = val.get("player_name")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        if is_battle {
            let attacks = body.get("content").and_then(|c| c.get("Attacks")).and_then(|a| a.as_object());
            let mut items = Vec::new();
            let mut item_totals = HashMap::new();
            let mut levels = Vec::new();
            let mut pos_x = 0;
            let mut pos_y = 0;

            if let Some(att_map) = attacks {
                for (_k, atk) in att_map {
                    let cidt = atk.get("CIdt");
                    let is_fort = cidt
                        .and_then(|c| c.get("IsFort"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let npcbtype = cidt
                        .and_then(|c| c.get("NpcBType"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as i64;

                    if !is_fort && npcbtype == 1 {
                        if let Some(cidt) = cidt {
                            if let Some(hlv) = json_as_u32(cidt.get("HLv")) {
                                if hlv > 0 && hlv <= 55 {
                                    levels.push(hlv);
                                }
                            }
                        }
                    }

                    let is_valid_target = !is_fort && (npcbtype == 1 || atk.get("NpcKillLoot").is_some());
                    if is_valid_target || (pos_x == 0 && pos_y == 0) {
                        if let Some(cidt) = cidt {
                            if let Some(cpos) = cidt.get("CastlePos") {
                                let x = cpos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let y = cpos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                if x > 0.0 && y > 0.0 {
                                    pos_x = (x / 6.0).round() as i32;
                                    pos_y = (y / 6.0).round() as i32;
                                }
                            }
                        }

                        if let Some(pos) = atk.get("Pos") {
                            let x = pos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let y = pos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            if x > 0.0 && y > 0.0 {
                                pos_x = (x / 6.0).round() as i32;
                                pos_y = (y / 6.0).round() as i32;
                            }
                        }
                    }

                    if let Some(loots_map) = atk.get("NpcKillLoot").and_then(|l| l.as_object()) {
                        for (_loot_id, item_val) in loots_map {
                            let itype = json_as_u32(item_val.get("Type")).unwrap_or(0);
                            let isub = json_as_u32(item_val.get("SubType")).unwrap_or(0);
                            let val = json_as_i64(item_val.get("Value")).unwrap_or(0);
                            let pic = item_val.get("PIC").and_then(|v| v.as_str()).unwrap_or("").to_string();

                            let name = format!("Item_{}_{}", itype, isub);
                            *item_totals.entry(name.clone()).or_insert(0) += val;
                            items.push(crate::model::ItemDrop {
                                itype,
                                sub_type: isub,
                                value: val,
                                pic,
                                name_vi: name.clone(),
                                name_en: name,
                                category: "Khác".into(),
                                sprites: vec![],
                            });
                        }
                    }
                }
            }

            let (target_level, target) = crate::model::format_barbarian_target(&levels);

            let content = target.clone();

            return Some(BattleRecord {
                filename: fname.to_string(),
                mail_id,
                battle_type: "barbarian".to_string(),
                time_raw,
                time_utc: "".into(),
                time_vn,
                target,
                target_level,
                talent_add: 0,
                pos_x,
                pos_y,
                damage_pct: if damage_pct > 0 { damage_pct } else { 100 },
                plunder_tier: 0,
                items,
                item_totals,
                content,
                account_id: account_id.clone(),
                player_name: player_name.clone(),
                ..Default::default()
            });
        }

        let target = body.get("targetName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut battle_type = if (sub_t == 11 && sub_p == 4)
            || target.to_lowercase().contains("motte")
            || content.to_lowercase().contains("motte")
        {
            "motte".to_string()
        } else if (sub_t == 11 && sub_p == 1)
            || target.to_lowercase().contains("fort")
            || target.to_lowercase().starts_with("level")
        {
            "fort".to_string()
        } else {
            "other".to_string()
        };

        if let Some(meta_bt) = val.get("_stats_metadata").and_then(|m| m.get("battle_type")).and_then(|v| v.as_str()) {
            if !meta_bt.is_empty() && meta_bt != "other" {
                battle_type = meta_bt.to_string();
            }
        }

        let mail_id = val.get("id")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| json_as_i64(Some(v)).map(|n| n.to_string())))
            .unwrap_or_default();

        let mut item_totals = HashMap::new();
        let items = Vec::new();

        if let Some(attach) = val.get("attachments") {
            if let Some(arr) = attach.as_array() {
                for it in arr {
                    let name = it.get("name_vi").and_then(|v| v.as_str()).unwrap_or("");
                    let qty = json_as_i64(it.get("value")).unwrap_or(0);
                    if !name.is_empty() && qty > 0 {
                        *item_totals.entry(name.to_string()).or_insert(0) += qty;
                    }
                }
            } else if let Some(map) = attach.as_object() {
                for (_k, it) in map {
                    let name = it.get("name_vi").and_then(|v| v.as_str()).unwrap_or("");
                    let qty = json_as_i64(it.get("value")).unwrap_or(0);
                    if !name.is_empty() && qty > 0 {
                        *item_totals.entry(name.to_string()).or_insert(0) += qty;
                    }
                }
            }
        }

        Some(BattleRecord {
            filename: fname.to_string(),
            mail_id,
            battle_type,
            time_raw,
            time_utc: "".into(),
            time_vn,
            target,
            target_level: "".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct,
            plunder_tier,
            items,
            item_totals,
            content,
            account_id,
            player_name,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_sqlite_init_creates_db() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sqlite_init_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 0);
        assert!(temp_dir.join("rok_history_vault.db").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sqlite_wal_mode() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_wal_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let vault = MailVault::new(&temp_dir);
        let conn = vault.conn();
        let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0)).unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");

        let sync_mode: i64 = conn.query_row("PRAGMA synchronous;", [], |r| r.get(0)).unwrap();
        // 1 = NORMAL
        assert_eq!(sync_mode, 1);

        drop(conn);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_record_dedup_key() {
        let rec1 = BattleRecord {
            filename: "Persistent.Mail.123".into(),
            mail_id: "123".into(),
            battle_type: "motte".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Motte".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 50,
            plunder_tier: 6,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
        };

        assert_eq!(get_record_dedup_key(&rec1), "id:123");

        let mut rec2 = rec1.clone();
        rec2.mail_id = "".into();
        assert_eq!(get_record_dedup_key(&rec2), "file:Persistent.Mail.123");

        let mut rec3 = rec2.clone();
        rec3.filename = "".into();
        assert_eq!(get_record_dedup_key(&rec3), "sig:1000:motte:100:200:50");
    }

    #[test]
    fn test_vault_upsert_and_dedup() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_vault_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let mut vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 0);

        let rec = BattleRecord {
            filename: "Persistent.Mail.999".into(),
            mail_id: "999".into(),
            battle_type: "fort".into(),
            time_raw: 5000,
            time_utc: "".into(),
            time_vn: "2026-10-22 10:00:00".into(),
            target: "Barbarian Fort 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 500,
            pos_y: 600,
            damage_pct: 45,
            plunder_tier: 6,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Test".into(),
            ..Default::default()
        };

        assert!(vault.upsert(rec.clone()));
        assert_eq!(vault.total_count(), 1);

        assert!(!vault.upsert(rec));
        assert_eq!(vault.total_count(), 1);

        assert!(vault.save_to_disk().is_ok());

        let reloaded = MailVault::new(&temp_dir);
        assert_eq!(reloaded.total_count(), 1);
        assert_eq!(reloaded.get_records()[0].mail_id, "999");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_inspect_new_files() {
        if !Path::new("rok_history_vault.db").exists() && !Path::new("rok_history_vault.json").exists() {
            return;
        }
        let vault = MailVault::new(Path::new("."));
        if vault.total_count() < 609 {
            return;
        }
        let mut btypes = HashMap::new();
        for r in vault.get_records() {
            *btypes.entry(r.battle_type.clone()).or_insert(0) += 1;
            if r.battle_type == "barbarian" {
                assert!(!r.target_level.contains("60"), "Barbarian level must not contain 60: {:?}", r.target_level);
                assert!(!r.target.contains("60"), "Barbarian target must not contain 60: {:?}", r.target);
            }
        }
        if vault.total_count() == 609 {
            assert!(*btypes.get("motte").unwrap_or(&0) >= 14, "Phải nạp từ 14 chiến báo Motte trở lên");
            assert!(*btypes.get("fort").unwrap_or(&0) >= 19, "Phải nạp từ 19 chiến báo Fort trở lên");
            assert!(*btypes.get("barbarian").unwrap_or(&0) >= 476, "Phải nạp từ 476 chiến báo Barbarian trở lên");
            assert_eq!(*btypes.get("other").unwrap_or(&0), 0, "Không được có chiến báo nào bị phân loại nhầm thành other");
        }

        let motte_file = Path::new("motte_filtered_mails.json");
        if motte_file.exists() {
            if let Ok(data) = fs::read_to_string(motte_file) {
                let clean_data = data.trim_start_matches('\u{feff}');
                match serde_json::from_str::<serde_json::Value>(clean_data) {
                    Ok(mut json_val) => {
                        let mut modified = false;
                        if let Some(battles) = json_val.get_mut("battles").and_then(|b| b.as_array_mut()) {
                            for b in battles {
                                if b.get("battle_type").and_then(|v| v.as_str()) == Some("barbarian") {
                                    let mut needs_clean = false;
                                    let lvl = b.get("target_level").or_else(|| b.get("motte_level")).and_then(|v| v.as_str()).unwrap_or("");
                                    let tgt = b.get("target").and_then(|v| v.as_str()).unwrap_or("");
                                    if lvl.contains("60") || tgt.contains("60") {
                                        needs_clean = true;
                                    }
                                    if needs_clean {
                                        let min_lvl = if let Some((m, _)) = lvl.split_once("-60") {
                                            m.to_string()
                                        } else if let Some(pos) = tgt.find("-60") {
                                            tgt[..pos].split(|c: char| !c.is_numeric()).rfind(|s| !s.is_empty()).unwrap_or("").to_string()
                                        } else {
                                            "".to_string()
                                        };

                                        if !min_lvl.is_empty() {
                                            b["target_level"] = serde_json::Value::String(min_lvl.clone());
                                            let new_tgt = format!("Dã Man Cấp {}", min_lvl);
                                            b["target"] = serde_json::Value::String(new_tgt.clone());
                                            let px = b.get("pos_x").and_then(|v| v.as_i64()).unwrap_or(0);
                                            let py = b.get("pos_y").and_then(|v| v.as_i64()).unwrap_or(0);
                                            if px > 0 && py > 0 {
                                                b["content"] = serde_json::Value::String(format!("{} tại X:{} Y:{}", new_tgt, px, py));
                                            } else {
                                                b["content"] = serde_json::Value::String(new_tgt);
                                            }
                                            modified = true;
                                        }
                                    }
                                }
                            }
                        }
                        if modified {
                            assert!(serde_json::to_string_pretty(&json_val).is_ok());
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse motte_filtered_mails.json: {}", e);
                    }
                }
            }
        }
    }

    #[test]
    fn test_restore_from_filtered_mails_json() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_restore_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let json_dir = temp_dir.join("filtered_mails_json");
        let _ = fs::create_dir_all(&json_dir);

        let mock_json = r#"{
            "box": "Report",
            "id": "777888",
            "sendTime": 17858542764,
            "body": {
                "subType": 11,
                "subParam": 4,
                "targetName": "Motte 15",
                "content": "Tiêu diệt thành công Motte cấp 15"
            },
            "_stats_metadata": {
                "damage_pct": 55,
                "plunder_tier": 6,
                "time_vn": "2026-10-22 09:30:00"
            }
        }"#;

        let sample_file = json_dir.join("Persistent.Mail.777888.json");
        fs::write(&sample_file, mock_json).unwrap();

        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 1);
        let rec = &vault.get_records()[0];
        assert_eq!(rec.mail_id, "777888");
        assert_eq!(rec.battle_type, "motte");
        assert_eq!(rec.damage_pct, 55);
        assert_eq!(rec.plunder_tier, 6);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_restore_from_filtered_mails_json_with_archive_box() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_restore_archive_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let json_dir = temp_dir.join("filtered_mails_json");
        let _ = fs::create_dir_all(&json_dir);

        let mock_json = r#"{
            "box": "Archive",
            "id": "888999",
            "sendTime": 17858542764,
            "body": {
                "subType": 11,
                "subParam": 4,
                "targetName": "Motte 20",
                "content": "Tiêu diệt thành công Motte cấp 20"
            },
            "_stats_metadata": {
                "damage_pct": 80,
                "plunder_tier": 6,
                "time_vn": "2026-10-22 11:30:00"
            }
        }"#;

        let sample_file = json_dir.join("Persistent.Mail.888999.json");
        fs::write(&sample_file, mock_json).unwrap();

        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 1, "Thư trong hòm Archive phải được khôi phục vào Vault");
        let rec = &vault.get_records()[0];
        assert_eq!(rec.mail_id, "888999");
        assert_eq!(rec.battle_type, "motte");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_and_survive_game_delete() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_survive_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let game_mailcache = temp_dir.join("game_mailcache");
        let _ = fs::create_dir_all(&game_mailcache);

        let vault = MailVault::new(&temp_dir);

        let sample_mail = game_mailcache.join("Persistent.Mail.999111");
        let sample_content = vec![0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x05, 0xFF];
        fs::write(&sample_mail, &sample_content).unwrap();

        let backup_result = vault.backup_binary_mail(&sample_mail);
        assert!(backup_result.is_ok());
        let backup_path = backup_result.unwrap();
        assert!(backup_path.exists());

        fs::remove_file(&sample_mail).unwrap();
        assert!(!sample_mail.exists());

        assert!(backup_path.exists());
        let saved_bytes = fs::read(&backup_path).unwrap();
        assert_eq!(saved_bytes, sample_content);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_corrupted_vault_file_backup_and_refusal() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_corrupt_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let vault_file = temp_dir.join("rok_history_vault.json");
        let corrupt_content = b"{\"version\": 1, \"records\": [corrupted raw bytes ...";
        fs::write(&vault_file, corrupt_content).unwrap();

        let vault = MailVault::new(&temp_dir);
        assert!(vault.is_corrupted(), "Vault must detect corrupted JSON file");

        let corrupt_backup = temp_dir.join("rok_history_vault.corrupt");
        assert!(corrupt_backup.exists(), "Must create .corrupt backup file");
        assert_eq!(fs::read(&corrupt_backup).unwrap(), corrupt_content);

        let save_res = vault.save_to_disk();
        assert!(save_res.is_err(), "Must refuse to overwrite corrupted vault");
        assert_eq!(fs::read(&vault_file).unwrap(), corrupt_content, "Original vault file must not be overwritten");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_contains_file_or_id_matching() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_contains_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let mut vault = MailVault::new(&temp_dir);

        let rec1 = BattleRecord {
            filename: "Persistent.Mail.1002220217856803764".into(),
            mail_id: "1002220217856803764".into(),
            battle_type: "barbarian".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "".into(),
            target_level: "".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 0,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
        };
        vault.upsert(rec1);

        assert!(vault.contains_file_or_id("Persistent.Mail.1002220217856803764"));
        assert!(vault.contains_file_or_id("1002220217856803764"));
        assert!(!vault.contains_file_or_id("Persistent.Mail.nonexistent"));

        let rec2 = BattleRecord {
            filename: "backup_file.mail".into(),
            mail_id: "".into(),
            battle_type: "fort".into(),
            time_raw: 2000,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "".into(),
            target_level: "".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 0,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
        };
        vault.upsert(rec2);

        assert!(vault.contains_file_or_id("backup_file.mail"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_latest_filtered() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_query_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);
        for i in 1..=10 {
            vault.upsert(BattleRecord {
                filename: format!("mail_{}", i),
                mail_id: i.to_string(),
                battle_type: if i % 2 == 0 { "fort".into() } else { "motte".into() },
                time_raw: i * 100,
                time_utc: "".into(),
                time_vn: "".into(),
                target: format!("Target {}", i),
                target_level: "1".into(),
                talent_add: 0,
                pos_x: 0,
                pos_y: 0,
                damage_pct: 100,
                plunder_tier: 0,
                items: vec![],
                item_totals: HashMap::new(),
                content: "".into(),
                ..Default::default()
            });
        }

        let forts = vault.query_latest_filtered(FilterMode::Fort, 3);
        assert_eq!(forts.len(), 3);
        assert_eq!(forts[0].time_raw, 1000);
        assert_eq!(forts[1].time_raw, 800);
        assert_eq!(forts[2].time_raw, 600);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clean_binary_vault() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_clean_vault_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let vault = MailVault::new(&temp_dir);
        let bin_dir = vault.get_binary_dir();

        fs::write(bin_dir.join("file1.bin"), b"12345").unwrap();
        fs::write(bin_dir.join("file2.bin"), b"67890").unwrap();
        fs::write(bin_dir.join("file3.bin"), b"abc").unwrap();

        let (count, bytes) = vault.clean_binary_vault().expect("clean binary vault");
        assert_eq!(count, 3);
        assert_eq!(bytes, 13);
        assert!(!bin_dir.join("file1.bin").exists());
        assert!(!bin_dir.join("file2.bin").exists());
        assert!(!bin_dir.join("file3.bin").exists());

        let (count2, bytes2) = vault.clean_binary_vault().expect("clean empty vault");
        assert_eq!(count2, 0);
        assert_eq!(bytes2, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_export_and_import_vault_json() {
        let temp1 = std::env::temp_dir().join(format!("rok_test_export_json_1_{}", std::process::id()));
        let temp2 = std::env::temp_dir().join(format!("rok_test_export_json_2_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp1);
        let _ = fs::remove_dir_all(&temp2);
        let _ = fs::create_dir_all(&temp1);
        let _ = fs::create_dir_all(&temp2);

        let mut v1 = MailVault::new(&temp1);
        let rec = BattleRecord {
            filename: "Persistent.Mail.555".into(),
            mail_id: "555".into(),
            battle_type: "fort".into(),
            time_raw: 9999,
            time_utc: "".into(),
            time_vn: "2026-11-01 10:00:00".into(),
            target: "Fort 5".into(),
            target_level: "5".into(),
            talent_add: 0,
            pos_x: 10,
            pos_y: 20,
            damage_pct: 100,
            plunder_tier: 5,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Export test".into(),
            ..Default::default()
        };
        v1.upsert(rec);

        let json_str = v1.export_vault_json().expect("Export json failed");
        assert!(json_str.contains("\"total_records\": 1"));
        assert!(json_str.contains("\"mail_id\": \"555\""));

        let mut v2 = MailVault::new(&temp2);
        let imported = v2.import_records_from_json(&json_str).expect("Import json failed");
        assert_eq!(imported, 1);
        assert_eq!(v2.total_count(), 1);

        let r = &v2.get_records()[0];
        assert_eq!(r.mail_id, "555");
        assert_eq!(r.target, "Fort 5");

        let _ = fs::remove_dir_all(&temp1);
        let _ = fs::remove_dir_all(&temp2);
    }

    #[test]
    fn test_migration_from_json_to_db() {
        let temp = std::env::temp_dir().join(format!("rok_test_auto_mig_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let _ = fs::create_dir_all(&temp);

        let json_path = temp.join("rok_history_vault.json");
        let initial_json = serde_json::json!({
            "version": 1,
            "last_updated": "2026-10-29 12:00:00",
            "total_records": 1,
            "records": [{
                "filename": "Persistent.Mail.888",
                "mail_id": "888",
                "battle_type": "motte",
                "time_raw": 1234,
                "time_utc": "",
                "time_vn": "2026-10-29 12:00:00",
                "target": "Motte 15",
                "target_level": "15",
                "pos_x": 100,
                "pos_y": 200,
                "damage_pct": 50,
                "plunder_tier": 6,
                "items": [],
                "content": "Migrate Test"
            }]
        });
        fs::write(&json_path, serde_json::to_string_pretty(&initial_json).unwrap()).unwrap();

        let vault = MailVault::new(&temp);
        assert_eq!(vault.total_count(), 1);
        assert!(temp.join("rok_history_vault.db").exists());
        assert!(temp.join("rok_history_vault.json.migrated").exists());
        assert!(!json_path.exists());

        let rec = &vault.get_records()[0];
        assert_eq!(rec.mail_id, "888");
        assert_eq!(rec.target, "Motte 15");

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_sanitize_records_sql() {
        let temp = std::env::temp_dir().join(format!("rok_test_sanitize_sql_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let _ = fs::create_dir_all(&temp);

        let mut vault = MailVault::new(&temp);

        // 1. Pass record to be deleted
        vault.upsert(BattleRecord {
            filename: "mail_pass".into(),
            mail_id: "57774831178214367624".into(),
            battle_type: "barbarian".into(),
            time_raw: 100,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Pass 1".into(),
            target_level: "1".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Pass".into(),
            ..Default::default()
        });

        // 2. Barbarian level 60 with empty items to be deleted
        vault.upsert(BattleRecord {
            filename: "mail_lvl60_empty".into(),
            mail_id: "60601".into(),
            battle_type: "barbarian".into(),
            time_raw: 200,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Dã Man Cấp 60".into(),
            target_level: "60".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Dã Man Cấp 60".into(),
            ..Default::default()
        });

        // 3. Barbarian level 50-60 with items to be updated to level 50
        vault.upsert(BattleRecord {
            filename: "mail_lvl50_60".into(),
            mail_id: "60602".into(),
            battle_type: "barbarian".into(),
            time_raw: 300,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Dã Man Cấp 50-60 (x2)".into(),
            target_level: "50-60".into(),
            talent_add: 0,
            pos_x: 10,
            pos_y: 20,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![crate::model::ItemDrop {
                itype: 1,
                sub_type: 1,
                value: 10,
                pic: "".into(),
                name_vi: "Item".into(),
                name_en: "Item".into(),
                category: "Loot".into(),
                sprites: vec![],
            }],
            item_totals: HashMap::new(),
            content: "Dã Man Cấp 50-60 (x2)".into(),
            ..Default::default()
        });

        assert_eq!(vault.total_count(), 3);
        let removed = vault.sanitize_records();
        assert_eq!(removed, 2);
        assert_eq!(vault.total_count(), 1);

        let remaining = &vault.get_records()[0];
        assert_eq!(remaining.mail_id, "60602");
        assert_eq!(remaining.target_level, "50");
        assert_eq!(remaining.target, "Dã Man Cấp 50");
        assert_eq!(remaining.content, "Dã Man Cấp 50 tại X:10 Y:20");

        let _ = fs::remove_dir_all(&temp);
    }

    fn create_test_record(mail_id: &str, time_raw: i64) -> BattleRecord {
        BattleRecord {
            filename: format!("Persistent.Mail.{}", mail_id),
            mail_id: mail_id.to_string(),
            battle_type: "fort".to_string(),
            time_raw,
            time_utc: "".to_string(),
            time_vn: "".to_string(),
            target: "Target".to_string(),
            target_level: "1".to_string(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 50,
            plunder_tier: 1,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Test content".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_vault_unsynced_and_mark_synced() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_unsynced_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);
        let rec = create_test_record("mail_001", 1000);
        assert!(vault.upsert(rec));

        let unsynced = vault.get_unsynced_records(10).unwrap();
        assert_eq!(unsynced.len(), 1);
        assert_eq!(unsynced[0].0, "id:mail_001");

        let marked = vault.mark_records_synced(&["id:mail_001".to_string()], 1700000000).unwrap();
        assert_eq!(marked, 1);

        let unsynced_after = vault.get_unsynced_records(10).unwrap();
        assert_eq!(unsynced_after.len(), 0);

        let (total, unsynced_cnt) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 1);
        assert_eq!(unsynced_cnt, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_cloud_user_id() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_cloud_user_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let vault = MailVault::new(&temp_dir);

        assert_eq!(vault.get_cloud_user_id().unwrap(), None);

        vault.set_cloud_user_id("user_abc123").unwrap();
        assert_eq!(vault.get_cloud_user_id().unwrap(), Some("user_abc123".to_string()));

        vault.set_cloud_user_id("user_xyz789").unwrap();
        assert_eq!(vault.get_cloud_user_id().unwrap(), Some("user_xyz789".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_sync_queries_ordering_and_edge_cases() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_edge_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);

        // Empty vault sync counts
        let (total, unsynced) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 0);
        assert_eq!(unsynced, 0);

        // Mark empty slice
        assert_eq!(vault.mark_records_synced(&[], 123456).unwrap(), 0);
        // Mark non-existent
        assert_eq!(vault.mark_records_synced(&["non_existent".to_string()], 123456).unwrap(), 0);

        // Insert records in non-chronological order
        vault.upsert(create_test_record("rec_3000", 3000));
        vault.upsert(create_test_record("rec_1000", 1000));
        vault.upsert(create_test_record("rec_2000", 2000));

        let (total, unsynced) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 3);
        assert_eq!(unsynced, 3);

        // Limit 2 with ordering check (should be rec_1000, rec_2000)
        let page = vault.get_unsynced_records(2).unwrap();
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].0, "id:rec_1000");
        assert_eq!(page[0].1.time_raw, 1000);
        assert_eq!(page[1].0, "id:rec_2000");
        assert_eq!(page[1].1.time_raw, 2000);

        // Mark rec_1000 as synced
        let updated = vault.mark_records_synced(&["id:rec_1000".to_string()], 1700000000).unwrap();
        assert_eq!(updated, 1);

        let (total, unsynced) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 3);
        assert_eq!(unsynced, 2);

        // Remaining unsynced records
        let page2 = vault.get_unsynced_records(10).unwrap();
        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].0, "id:rec_2000");
        assert_eq!(page2[1].0, "id:rec_3000");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_restore_records_batch() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_vault_restore_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);

        // Pre-insert an unsynced record
        vault.upsert(create_test_record("pre_exist", 500));
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 1);

        let mut rec1 = create_test_record("pre_exist", 500); // conflict with existing unsynced
        rec1.account_id = "11111".into();
        rec1.player_name = "PlayerOne".into();
        let mut rec2 = create_test_record("new_restored", 600);
        rec2.account_id = "22222".into();
        rec2.player_name = "PlayerTwo".into();
        let restored = vault.restore_records_batch(&[rec1, rec2], Some(1700000000)).unwrap();
        assert_eq!(restored, 2);

        // Check account_id and player_name are preserved on restore
        let recs = vault.get_records();
        let r_new = recs.iter().find(|r| r.mail_id == "new_restored").unwrap();
        assert_eq!(r_new.account_id, "22222");
        assert_eq!(r_new.player_name, "PlayerTwo");

        // All should now have synced_at set, so unsynced is 0
        let (total, unsynced) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 2);
        assert_eq!(unsynced, 0);

        // Test restoring with None (legacy fallback) leaves record unsynced
        let rec3 = create_test_record("legacy_restored", 700);
        let restored_legacy = vault.restore_records_batch(&[rec3], None).unwrap();
        assert_eq!(restored_legacy, 1);
        let (total2, unsynced2) = vault.get_sync_counts().unwrap();
        assert_eq!(total2, 3);
        assert_eq!(unsynced2, 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_mark_records_synced_with_nonexistent_key_returns_zero() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_vault_zero_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let vault = MailVault::new(&temp_dir);

        let marked = vault.mark_records_synced(&["non_existent_key".to_string()], 12345).unwrap();
        assert_eq!(marked, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_upsert_modified_record_resets_synced_at() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_vault_mod_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);

        let mut rec = create_test_record("mail_mod_1", 1000);
        rec.damage_pct = 0;
        rec.items = vec![];
        assert!(vault.upsert(rec.clone()));

        // Mark it synced
        vault.mark_records_synced(&["id:mail_mod_1".to_string()], 1700000000).unwrap();
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        // Upsert identical record -> modified is false -> returns false
        assert!(!vault.upsert(rec.clone()));
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        // Upsert with enriched items and damage_pct -> modified is true -> returns true & resets synced_at to NULL!
        let mut updated = rec.clone();
        updated.damage_pct = 50;
        updated.items = vec![crate::model::ItemDrop {
            itype: 1,
            sub_type: 1,
            value: 10,
            pic: "".into(),
            name_vi: "Item".into(),
            name_en: "Item".into(),
            category: "Loot".into(),
            sprites: vec![],
        }];
        assert!(vault.upsert(updated));

        // Now it must be unsynced again
        let unsynced = vault.get_unsynced_records(10).unwrap();
        assert_eq!(unsynced.len(), 1);
        assert_eq!(unsynced[0].1.damage_pct, 50);
        assert_eq!(unsynced[0].1.items.len(), 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_count_and_delete_records_except_uids() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_del_non_main_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);

        let mut r1 = create_test_record("mail_del_001", 1000);
        r1.account_id = "122980539".into();
        let mut r2 = create_test_record("mail_del_002", 2000);
        r2.account_id = "60200655".into();
        let mut r3 = create_test_record("mail_del_003", 3000);
        r3.account_id = "player_139825469".into();

        vault.upsert(r1);
        vault.upsert(r2);
        vault.upsert(r3);

        let main_uids = vec!["122980539".to_string()];
        let non_main_cnt = vault.count_records_except_uids(&main_uids);
        assert_eq!(non_main_cnt, 2);

        let (deleted, deleted_keys) = vault.delete_records_except_uids(&main_uids);
        assert_eq!(deleted, 2);
        assert_eq!(deleted_keys.len(), 2);

        assert_eq!(vault.total_count(), 1);
        let remaining = vault.get_records();
        assert_eq!(remaining[0].account_id, "122980539");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_vault_pvp_record_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_pvp_vault_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);
        let mut vault = MailVault::new(&temp_dir);

        let mut rec = create_test_record("pvp_mail_001", 5000);
        rec.battle_type = "pvp".to_string();
        rec.pvp_combat_type = "field".to_string();
        rec.primary_commander_id = 198;
        rec.primary_commander_name = "Liu Che".to_string();
        rec.secondary_commander_id = 140;
        rec.secondary_commander_name = "Joan Prime".to_string();
        rec.target_primary_commander_id = 179;
        rec.target_primary_commander_name = "Zhuge Liang".to_string();
        rec.target_secondary_commander_id = 6;
        rec.target_secondary_commander_name = "Yi Seong-Gye".to_string();
        rec.kp_gained = 350_000.0;
        rec.kp_lost = 120_000.0;
        rec.dead_dealt = 1000;
        rec.dead_taken = 200;
        rec.severely_wounded_dealt = 15000;
        rec.severely_wounded_taken = 5000;

        assert!(vault.upsert(rec));

        let pvp_list = vault.query_latest_filtered(FilterMode::PvP, 10);
        assert_eq!(pvp_list.len(), 1);
        let r = &pvp_list[0];
        assert_eq!(r.primary_commander_id, 198);
        assert_eq!(r.primary_commander_name, "Liu Che");
        assert_eq!(r.secondary_commander_id, 140);
        assert_eq!(r.secondary_commander_name, "Joan Prime");
        assert_eq!(r.target_primary_commander_id, 179);
        assert_eq!(r.target_primary_commander_name, "Zhuge Liang");
        assert_eq!(r.target_secondary_commander_id, 6);
        assert_eq!(r.target_secondary_commander_name, "Yi Seong-Gye");
        assert_eq!(r.kp_gained, 350_000.0);
        assert_eq!(r.kp_lost, 120_000.0);
        assert_eq!(r.dead_dealt, 1000);
        assert_eq!(r.dead_taken, 200);
        assert_eq!(r.severely_wounded_dealt, 15000);
        assert_eq!(r.severely_wounded_taken, 5000);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
