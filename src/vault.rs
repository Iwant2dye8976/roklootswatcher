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
            if let Err(e) = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;") {
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
        };

        vault.init(out_dir);
        vault
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

        let rows = match stmt.query_map([], |row| row_to_battle_record(row)) {
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
                .query_row(params![&dedup_key], |row| row_to_battle_record(row))
                .optional()
                .unwrap_or(None)
        };

        if let Some(mut existing) = existing {
            let mut modified = false;
            if rec.battle_type != "other" && (existing.battle_type == "other" || existing.battle_type.is_empty()) {
                existing.battle_type = rec.battle_type;
                modified = true;
            }
            if !rec.items.is_empty() && (existing.items.is_empty() || rec.items.len() >= existing.items.len()) {
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
                        item_totals_json = ?11
                    WHERE dedup_key = ?12"
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
                    dedup_key,
                ]) {
                    eprintln!("[!] Lỗi execute update for upsert: {}", e);
                }
            }
            false
        } else {
            let items_json = serde_json::to_string(&rec.items).unwrap_or_else(|_| "[]".to_string());
            let item_totals_json = serde_json::to_string(&rec.item_totals).unwrap_or_else(|_| "{}".to_string());

            let mut insert_stmt = match conn.prepare_cached(
                "INSERT INTO battle_records (
                    mail_id, filename, battle_type, time_raw, time_utc, time_vn,
                    target, target_level, talent_add, pos_x, pos_y, damage_pct,
                    plunder_tier, items_json, item_totals_json, content, dedup_key, synced_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, NULL)"
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
        Self::upsert_inner(&conn, rec)
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
            let _ = self.save_to_disk();
        }
        new_count
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
                "SELECT * FROM battle_records ORDER BY time_raw DESC LIMIT ?1"
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
        };

        let conn = self.conn();
        let mut stmt = match conn.prepare_cached(sql) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[!] Lỗi prepare query_latest_filtered: {}", e);
                return Vec::new();
            }
        };

        let rows = match stmt.query_map(params![limit as i64], |row| row_to_battle_record(row)) {
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

            let (target_level, target) = if !levels.is_empty() {
                let min_lvl = *levels.iter().min().unwrap();
                let max_lvl = *levels.iter().max().unwrap();
                let count = levels.len();

                if count == 1 {
                    (min_lvl.to_string(), format!("Dã Man Cấp {}", min_lvl))
                } else if min_lvl == max_lvl {
                    (min_lvl.to_string(), format!("Dã Man Cấp {} (x{})", min_lvl, count))
                } else {
                    (
                        format!("{}-{}", min_lvl, max_lvl),
                        format!("Dã Man Cấp {}-{} (x{})", min_lvl, max_lvl, count),
                    )
                }
            } else {
                ("".to_string(), "Dã Man".to_string())
            };

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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_sqlite_init_creates_db() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sqlite_init_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 0);
        assert!(temp_dir.join("rok_history_vault.db").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sqlite_wal_mode() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_wal_{}", std::process::id()));
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
        assert!(vault.total_count() >= 609, "Tổng số bản ghi phải từ 609 trở lên");
        let mut btypes = HashMap::new();
        for r in vault.get_records() {
            *btypes.entry(r.battle_type.clone()).or_insert(0) += 1;
            if r.battle_type == "barbarian" {
                assert!(!r.target_level.contains("60"), "Barbarian level must not contain 60: {:?}", r.target_level);
                assert!(!r.target.contains("60"), "Barbarian target must not contain 60: {:?}", r.target);
            }
        }
        assert!(*btypes.get("motte").unwrap_or(&0) >= 14, "Phải nạp từ 14 chiến báo Motte trở lên");
        assert!(*btypes.get("fort").unwrap_or(&0) >= 19, "Phải nạp từ 19 chiến báo Fort trở lên");
        assert!(*btypes.get("barbarian").unwrap_or(&0) >= 476, "Phải nạp từ 476 chiến báo Barbarian trở lên");
        assert_eq!(*btypes.get("other").unwrap_or(&0), 0, "Không được có chiến báo nào bị phân loại nhầm thành other");

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
        };
        vault.upsert(rec2);

        assert!(vault.contains_file_or_id("backup_file.mail"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_latest_filtered() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_query_{}", std::process::id()));
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
}
