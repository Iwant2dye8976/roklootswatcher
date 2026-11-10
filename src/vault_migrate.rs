use std::fs;
use std::path::Path;
use rusqlite::{params, Connection};
use crate::vault::{get_record_dedup_key, VaultStorageData};

#[allow(dead_code)]
pub const SCHEMA_VERSION: u32 = 1;

pub fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS vault_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO vault_meta (key, value) VALUES ('schema_version', '1');

        CREATE TABLE IF NOT EXISTS battle_records (
            mail_id          TEXT NOT NULL,
            filename         TEXT NOT NULL DEFAULT '',
            battle_type      TEXT NOT NULL DEFAULT '',
            time_raw         INTEGER NOT NULL DEFAULT 0,
            time_utc         TEXT NOT NULL DEFAULT '',
            time_vn          TEXT NOT NULL DEFAULT '',
            target           TEXT NOT NULL DEFAULT '',
            target_level     TEXT NOT NULL DEFAULT '',
            talent_add       INTEGER NOT NULL DEFAULT 0,
            pos_x            INTEGER NOT NULL DEFAULT 0,
            pos_y            INTEGER NOT NULL DEFAULT 0,
            damage_pct       INTEGER NOT NULL DEFAULT 0,
            plunder_tier     INTEGER NOT NULL DEFAULT 0,
            items_json       TEXT NOT NULL DEFAULT '[]',
            item_totals_json TEXT NOT NULL DEFAULT '{}',
            content          TEXT NOT NULL DEFAULT '',
            account_id       TEXT NOT NULL DEFAULT '',
            player_name      TEXT NOT NULL DEFAULT '',
            dedup_key        TEXT NOT NULL,
            synced_at        INTEGER,
            PRIMARY KEY (dedup_key)
        );
        CREATE INDEX IF NOT EXISTS idx_time_raw ON battle_records(time_raw);
        CREATE INDEX IF NOT EXISTS idx_battle_type ON battle_records(battle_type);
        CREATE INDEX IF NOT EXISTS idx_mail_id ON battle_records(mail_id);
        CREATE INDEX IF NOT EXISTS idx_filename ON battle_records(filename);
        CREATE INDEX IF NOT EXISTS idx_synced_at ON battle_records(synced_at);

        CREATE TABLE IF NOT EXISTS ignored_mail_files (
            filename   TEXT PRIMARY KEY,
            account_id TEXT NOT NULL DEFAULT ''
        );",
    )?;
    let _ = conn.execute("ALTER TABLE battle_records ADD COLUMN account_id TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE battle_records ADD COLUMN player_name TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_account_id ON battle_records(account_id)", []);
    Ok(())
}

pub fn migrate_from_json(json_path: &Path, conn: &Connection) -> Result<usize, String> {
    if !json_path.exists() {
        return Ok(0);
    }

    let file = fs::File::open(json_path)
        .map_err(|e| format!("Failed to open JSON vault {:?}: {}", json_path, e))?;
    let reader = std::io::BufReader::new(file);
    let storage: VaultStorageData = match serde_json::from_reader(reader) {
        Ok(s) => s,
        Err(e) => {
            let corrupt_path = json_path.with_extension("corrupt");
            let _ = fs::copy(json_path, &corrupt_path);
            return Err(format!(
                "File vault {:?} bị lỗi cấu trúc JSON: {}. Đã tạo bản sao lưu tại {:?}",
                json_path, e, corrupt_path
            ));
        }
    };

    conn.execute_batch("BEGIN TRANSACTION;")
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;

    let mut stmt = match conn.prepare(
        "INSERT OR IGNORE INTO battle_records (
            mail_id, filename, battle_type, time_raw, time_utc, time_vn,
            target, target_level, talent_add, pos_x, pos_y, damage_pct,
            plunder_tier, items_json, item_totals_json, content, account_id, player_name, dedup_key, synced_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, NULL)"
    ) {
        Ok(s) => s,
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            return Err(format!("Failed to prepare insert statement: {}", e));
        }
    };

    let mut inserted_count = 0;
    for rec in &storage.records {
        let dedup_key = get_record_dedup_key(rec);
        let items_json = serde_json::to_string(&rec.items).unwrap_or_else(|_| "[]".to_string());
        let item_totals_json = serde_json::to_string(&rec.item_totals).unwrap_or_else(|_| "{}".to_string());

        let res = stmt.execute(params![
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
            rec.account_id,
            rec.player_name,
            dedup_key,
        ]);

        match res {
            Ok(changes) => {
                inserted_count += changes;
            }
            Err(e) => {
                drop(stmt);
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(format!("Failed to insert record {}: {}", dedup_key, e));
            }
        }
    }

    drop(stmt);
    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(inserted_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_create_schema_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM battle_records", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        // Verify index exists
        let idx: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_time_raw'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(idx, "idx_time_raw");

        let idx_sync: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_synced_at'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(idx_sync, "idx_synced_at");
    }

    #[test]
    fn test_migrate_from_json_imports_records() {
        let temp = std::env::temp_dir().join(format!("rok_migrate_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::create_dir_all(&temp);
        let json_path = temp.join("rok_history_vault.json");
        let json_data = serde_json::json!({
            "version": 1,
            "last_updated": "2026-10-29 12:00:00",
            "total_records": 1,
            "records": [{
                "filename": "Persistent.Mail.100",
                "mail_id": "100",
                "battle_type": "motte",
                "time_raw": 5000,
                "time_utc": "",
                "time_vn": "2026-10-29 12:00:00",
                "target": "Motte 10",
                "target_level": "10",
                "pos_x": 100,
                "pos_y": 200,
                "damage_pct": 50,
                "plunder_tier": 6,
                "items": [],
                "content": "Test"
            }]
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&json_data).unwrap()).unwrap();

        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let count = migrate_from_json(&json_path, &conn).unwrap();
        assert_eq!(count, 1);

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM battle_records", [], |r| r.get(0)).unwrap();
        assert_eq!(total, 1);

        let mail_id: String = conn.query_row("SELECT mail_id FROM battle_records", [], |r| r.get(0)).unwrap();
        assert_eq!(mail_id, "100");

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_migrate_from_json_corrupted_file() {
        let temp = std::env::temp_dir().join(format!("rok_migrate_corrupt_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::create_dir_all(&temp);
        let json_path = temp.join("rok_history_vault.json");
        std::fs::write(&json_path, "not valid json {[[[").unwrap();

        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let res = migrate_from_json(&json_path, &conn);
        assert!(res.is_err());
        assert!(temp.join("rok_history_vault.corrupt").exists());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_migrate_from_json_nonexistent_file() {
        let temp = std::env::temp_dir().join(format!("rok_migrate_nonexist_{}", std::process::id()));
        let json_path = temp.join("nonexistent.json");
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let count = migrate_from_json(&json_path, &conn).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_migrate_from_json_dedup() {
        let temp = std::env::temp_dir().join(format!("rok_migrate_dedup_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        let _ = std::fs::create_dir_all(&temp);
        let json_path = temp.join("rok_history_vault.json");
        let record = serde_json::json!({
            "filename": "Persistent.Mail.100",
            "mail_id": "100",
            "battle_type": "motte",
            "time_raw": 5000,
            "time_utc": "",
            "time_vn": "2026-10-29 12:00:00",
            "target": "Motte 10",
            "target_level": "10",
            "pos_x": 100,
            "pos_y": 200,
            "damage_pct": 50,
            "plunder_tier": 6,
            "items": [],
            "content": "Test"
        });
        let json_data = serde_json::json!({
            "version": 1,
            "last_updated": "2026-10-29 12:00:00",
            "total_records": 2,
            "records": [record, record]
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&json_data).unwrap()).unwrap();

        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let count = migrate_from_json(&json_path, &conn).unwrap();
        assert_eq!(count, 1);

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM battle_records", [], |r| r.get(0)).unwrap();
        assert_eq!(total, 1);

        let _ = std::fs::remove_dir_all(&temp);
    }
}
