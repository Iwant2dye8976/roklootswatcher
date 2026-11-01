# Per-Record Cloud Sync (`report_records`) & Background Worker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace monolithic JSON vault cloud sync with incremental per-record streaming to Supabase (`report_records`) driven by a non-blocking background worker thread.

**Architecture:** A dedicated `CloudSyncWorker` thread coordinates delta uploads and paginated restores through an `mpsc` channel. Unsynced records are queried directly from local SQLite (`synced_at IS NULL`) and uploaded in idempotent 200-item batches with PostgREST `resolution=merge-duplicates`, decoupling sync from the UI and making exit near-instant.

**Tech Stack:** Rust (2021 edition), `rusqlite` (bundled SQLite 3 with WAL mode), `ureq` (HTTP client), `serde` / `serde_json`, `chrono`, `eframe` / `egui`.

**Spec:** [docs/superpowers/specs/2026-11-01-cloud-sync-per-record-design.md](file:///C:/roklootswatcher/docs/superpowers/specs/2026-11-01-cloud-sync-per-record-design.md)

## Global Constraints

- Cloud table name is `report_records` with composite uniqueness on `(user_id, dedup_key)`.
- Batch size is fixed at 200 records per PostgREST HTTP request.
- Local storage remains `battle_records` in SQLite with `synced_at INTEGER` tracking.
- App exit bounded wait is strictly capped at 2 seconds; unsynced records remain safe in SQLite.
- Zero UI thread blocking: all network and REST calls run inside `CloudSyncWorker`.
- Must compile cleanly with `cargo check` and `cargo test` on Windows MSVC.

## Review Focus

1. **Network offline during background sync**: Worker catches connection error, enters exponential backoff (2s up to 60s), UI remains interactive, unsynced rows remain `synced_at IS NULL`.
2. **Abrupt exit during sync**: Main thread signals worker with 2s timeout; if timeout expires, thread detaches safely without data corruption because SQLite WAL guarantees atomicity.
3. **Expired OAuth access token (HTTP 401)**: Worker calls `refresh_session` and transparently retries the failed batch with the new token.
4. **Legacy account restore**: If `report_records` has 0 rows, restore queries legacy `user_vaults`, unpacks data into SQLite, and worker streams them to `report_records`.
5. **Duplicate key conflict during cloud upsert**: PostgREST header `Prefer: resolution=merge-duplicates` resolves conflicting `dedup_key` without failing the batch.

---

### Task 1: SQLite Vault Sync Querying & Indexing

**Files:**
- Modify: `src/vault_migrate.rs:35-45` (add index on `synced_at`)
- Modify: `src/vault.rs:380-450` (implement unsynced query, mark synced, sync counts)
- Test: `src/vault.rs` (unit tests for unsynced querying and status tracking)

**Interfaces:**
- Consumes: `BattleRecord`, `get_record_dedup_key(&BattleRecord)` from `src/vault.rs`
- Produces:
  - `MailVault::get_unsynced_records(&self, limit: usize) -> rusqlite::Result<Vec<(String, BattleRecord)>>`
  - `MailVault::mark_records_synced(&self, dedup_keys: &[String], timestamp: i64) -> rusqlite::Result<usize>`
  - `MailVault::get_sync_counts(&self) -> rusqlite::Result<(usize, usize)>`
  - `MailVault::get_cloud_user_id(&self) -> rusqlite::Result<Option<String>>`
  - `MailVault::set_cloud_user_id(&self, user_id: &str) -> rusqlite::Result<()>`

- [ ] **Step 1: Write the failing tests in `src/vault.rs`**

```rust
#[test]
fn test_vault_unsynced_and_mark_synced() {
    let tmp = tempfile::tempdir().unwrap();
    let vault = MailVault::new(tmp.path());
    let mut rec = create_test_record("mail_001", 1000);
    assert!(vault.upsert(&mut rec));

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
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_vault_unsynced_and_mark_synced`
Expected: FAIL with missing methods on `MailVault`

- [ ] **Step 3: Implement index in `vault_migrate.rs` and methods in `vault.rs`**

- In `vault_migrate.rs`: add `CREATE INDEX IF NOT EXISTS idx_synced_at ON battle_records(synced_at);`
- In `vault.rs`:
  - `get_unsynced_records`: queries `SELECT * FROM battle_records WHERE synced_at IS NULL ORDER BY time_raw ASC LIMIT ?1`
  - `mark_records_synced`: executes transactional batch update `UPDATE battle_records SET synced_at = ?1 WHERE dedup_key = ?2`
  - `get_sync_counts`: queries total rows and rows `WHERE synced_at IS NULL`
  - `get_cloud_user_id` / `set_cloud_user_id`: reads/writes `vault_meta` key `'synced_user_id'`

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_vault_unsynced_and_mark_synced`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/vault.rs src/vault_migrate.rs
git commit -m "feat(vault): add unsynced querying, sync marking, and idx_synced_at"
```

---

### Task 2: PostgREST `report_records` Batch Operations

**Files:**
- Modify: `src/cloud_sync.rs:390-480`
- Test: `src/cloud_sync.rs` (unit tests for payload formation and batch responses)

**Interfaces:**
- Consumes: `BattleRecord`, `DEFAULT_SUPABASE_URL`, `DEFAULT_SUPABASE_ANON_KEY`
- Produces:
  - `pub fn build_report_record_payload(user_id: &str, dedup_key: &str, record: &BattleRecord) -> serde_json::Value`
  - `pub fn upload_report_records_batch(supabase_url: &str, anon_key: &str, access_token: &str, user_id: &str, records: &[(String, BattleRecord)], timeout: Option<Duration>) -> Result<usize, String>`
  - `pub fn fetch_report_records_page(supabase_url: &str, anon_key: &str, access_token: &str, user_id: &str, limit: usize, offset: usize, timeout: Option<Duration>) -> Result<(Vec<BattleRecord>, bool), String>`
  - `pub fn check_legacy_user_vault(supabase_url: &str, anon_key: &str, access_token: &str, timeout: Option<Duration>) -> Result<Option<Vec<BattleRecord>>, String>`

- [ ] **Step 1: Write failing unit test for payload formatting and page decoding**

```rust
#[test]
fn test_build_report_record_payload() {
    let rec = BattleRecord {
        filename: "mail1.txt".into(),
        mail_id: "m123".into(),
        battle_type: "fort".into(),
        time_raw: 1690000000,
        time_utc: "2023-07-22T00:00:00Z".into(),
        time_vn: "2023-07-22 07:00:00".into(),
        target: "Barbarian Fort Level 5".into(),
        target_level: "5".into(),
        talent_add: 0,
        pos_x: 100,
        pos_y: 200,
        damage_pct: 100,
        plunder_tier: 1,
        items: vec![],
        item_totals: HashMap::new(),
        content: "".into(),
    };
    let payload = build_report_record_payload("user_abc", "id:m123", &rec);
    assert_eq!(payload["user_id"], "user_abc");
    assert_eq!(payload["dedup_key"], "id:m123");
    assert_eq!(payload["mail_id"], "m123");
    assert_eq!(payload["battle_type"], "fort");
    assert_eq!(payload["time_raw"], 1690000000);
    assert!(payload["data"].is_object());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_build_report_record_payload`
Expected: FAIL with "cannot find function `build_report_record_payload`"

- [ ] **Step 3: Implement PostgREST client functions in `src/cloud_sync.rs`**

- `build_report_record_payload`: constructs JSON with columns matching `report_records` schema.
- `upload_report_records_batch`: sends array of payloads to `/rest/v1/report_records` with header `Prefer: resolution=merge-duplicates`. Handles HTTP 404 with a descriptive error.
- `fetch_report_records_page`: GET `/rest/v1/report_records?user_id=eq.{user_id}&order=time_raw.asc&limit={limit}&offset={offset}`, extracts and deserializes `data` field of each row into `BattleRecord`. Returns `(records, has_more)`.
- `check_legacy_user_vault`: calls legacy `/rest/v1/user_vaults?select=vault_data`, returns `None` if empty or 404, or `Some(Vec<BattleRecord>)` if legacy data exists.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_build_report_record_payload`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cloud_sync.rs
git commit -m "feat(cloud_sync): implement report_records PostgREST batching and pagination"
```

---

### Task 3: `CloudSyncWorker` Actor & Command Protocol

**Files:**
- Modify: `src/cloud_sync.rs:500-650`
- Test: `src/cloud_sync.rs` (unit tests for worker channels and state machine)

**Interfaces:**
- Consumes: `MailVault` from `src/vault.rs`, `upload_report_records_batch`, `fetch_report_records_page`, `check_legacy_user_vault`
- Produces:
  - `pub enum SyncCommand { TriggerDelta, SyncNow, Restore(mpsc::Sender<Result<usize, String>>), Shutdown(mpsc::Sender<()>) }`
  - `pub struct SyncWorkerHandle { pub tx: mpsc::Sender<SyncCommand>, pub state: Arc<RwLock<SyncWorkerState>> }`
  - `pub fn spawn_sync_worker(vault: Arc<MailVault>, config: CloudAuthConfig) -> SyncWorkerHandle`

- [ ] **Step 1: Write test for worker commands and shutdown signaling**

```rust
#[test]
fn test_sync_worker_spawn_and_shutdown() {
    let tmp = tempfile::tempdir().unwrap();
    let vault = Arc::new(MailVault::new(tmp.path()));
    let cfg = CloudAuthConfig::default();
    let worker = spawn_sync_worker(vault, cfg);

    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
    assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_sync_worker_spawn_and_shutdown`
Expected: FAIL with "cannot find function `spawn_sync_worker`"

- [ ] **Step 3: Implement `CloudSyncWorker` thread loop**

- Worker thread manages:
  - `last_sync_time`, `is_syncing`, `pending_records_count`, `last_error`
  - Loop processes channel commands with a 1-second tick timeout.
  - On `TriggerDelta`: debounces (5s delay after last trigger), then fetches unsynced rows in batches of 200 and uploads.
  - On `SyncNow`: immediately flushes all unsynced batches to Supabase.
  - On `Restore`: pauses delta sync, paginates all rows from `report_records` (and legacy `user_vaults` if needed), inserts into SQLite with `synced_at = now()`, replies to callback sender.
  - On `Shutdown(ack_tx)`: executes one bounded flush attempt (up to 2 seconds timeout), signals `ack_tx.send(())`, and exits thread.
  - Auto-retries on network failures with exponential backoff up to 60 seconds.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_sync_worker_spawn_and_shutdown`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cloud_sync.rs
git commit -m "feat(cloud_sync): introduce CloudSyncWorker actor and lifecycle protocol"
```

---

### Task 4: Watcher Integration & Delta Sync Triggers

**Files:**
- Modify: `src/watcher.rs`
- Modify: `src/main.rs`
- Test: `tests/watcher_sync_tests.rs` or in-module tests

**Interfaces:**
- Consumes: `SyncCommand::TriggerDelta` from `src/cloud_sync.rs`, `Watcher` from `src/watcher.rs`
- Produces:
  - `Watcher::set_sync_channel(&self, tx: mpsc::Sender<SyncCommand>)`
  - Automatic `TriggerDelta` signal emitted whenever `Watcher::process_mail_file` saves a new record to `MailVault`.

- [ ] **Step 1: Write test verifying watcher notifies sync channel on new report**

```rust
#[test]
fn test_watcher_triggers_sync_on_new_record() {
    let (tx, rx) = std::sync::mpsc::channel();
    // Verify that when watcher processes a mail that modifies the vault, TriggerDelta is sent to rx
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_watcher_triggers_sync_on_new_record`
Expected: FAIL

- [ ] **Step 3: Implement channel hook in `src/watcher.rs`**

- Store optional `sync_tx: Arc<Mutex<Option<mpsc::Sender<SyncCommand>>>>` in `Watcher`.
- When new battle or gathering report is inserted via `vault.upsert(&mut record)`, if `auto_backup` is enabled and credentials are present, send `SyncCommand::TriggerDelta`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_watcher_triggers_sync_on_new_record`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/watcher.rs src/main.rs
git commit -m "feat(watcher): wire report insertion events to CloudSyncWorker TriggerDelta"
```

---

### Task 5: GUI State Integration, Settings UI & Graceful Exit

**Files:**
- Modify: `src/gui.rs:700-750` (exit backup handler)
- Modify: `src/gui.rs:970-1030` (manual backup & restore triggers)
- Modify: `src/gui.rs:4530-4610` (Cloud settings UI & sync status display)
- Modify: `src/i18n.rs` (add sync status localization keys)
- Test: `cargo check` and manual UI state verification

**Interfaces:**
- Consumes: `SyncWorkerHandle`, `SyncCommand`, `SyncWorkerState`
- Produces:
  - Non-blocking "Sync Now" button sending `SyncCommand::SyncNow`.
  - Non-blocking "Restore" button sending `SyncCommand::Restore` with progress dialog/spinner.
  - Live status indicator: `Idle`, `Syncing (N/Total)`, `Offline (retrying...)`.
  - Graceful exit sequence replacing `perform_exit_cloud_backup` with `Shutdown` signal (max 2-second join).

- [ ] **Step 1: Add localization keys in `src/i18n.rs`**

Add keys for:
- `CloudSyncStatusIdle`: "Đồng bộ: Đã cập nhật" / "Sync: Up to date"
- `CloudSyncStatusSyncing`: "Đồng bộ: Đang tải lên {0} bản ghi..." / "Sync: Uploading {0} records..."
- `CloudSyncStatusOffline`: "Đồng bộ: Mất kết nối, sẽ thử lại" / "Sync: Offline, will retry"
- `CloudSyncStatusRestoring`: "Đồng bộ: Đang khôi phục từ Cloud ({0} bản ghi)..." / "Sync: Restoring from Cloud ({0} records)..."

- [ ] **Step 2: Update manual sync, restore and status display in `src/gui.rs`**

- In `LootWatcherApp`: attach `SyncWorkerHandle`.
- In Settings view: show live sync state and unsynced counts from `sync_worker.state`.
- "Sync Now" calls `sync_worker.tx.send(SyncCommand::SyncNow)`.
- "Restore" sends `SyncCommand::Restore(resp_tx)` and tracks progress state.

- [ ] **Step 3: Replace `perform_exit_cloud_backup` in `src/gui.rs`**

- Replace synchronous JSON export and monolithic POST with:
  ```rust
  let (ack_tx, ack_rx) = std::sync::mpsc::channel();
  if let Some(worker) = &self.sync_worker {
      let _ = worker.tx.send(SyncCommand::Shutdown(ack_tx));
      let _ = ack_rx.recv_timeout(Duration::from_secs(2));
  }
  ```

- [ ] **Step 4: Verify compilation and tests**

Run: `cargo test` and `cargo check --release`
Expected: Zero compilation errors and all tests pass

- [ ] **Step 5: Commit**

```bash
git add src/gui.rs src/i18n.rs
git commit -m "feat(gui): integrate CloudSyncWorker status, restore flow, and non-blocking exit"
```

---

### Task 6: Full Verification & Scalability Regression Testing

**Files:**
- Test: `tests/per_record_sync_integration.rs`

- [ ] **Step 1: Write end-to-end integration test**

- Create mock local vault with 500 battle records.
- Verify `get_unsynced_records(200)` returns 200 records.
- Verify `mark_records_synced` leaves 300 unsynced.
- Verify second batch leaves 100 unsynced.
- Verify third batch leaves 0 unsynced.
- Verify `Shutdown` completes cleanly within timeout.

- [ ] **Step 2: Run all tests in the workspace**

Run: `cargo test --all`
Expected: ALL PASS with zero failures

- [ ] **Step 3: Build release binary to ensure compiler optimizations and clean strip**

Run: `cargo build --release`
Expected: Successful build with exit code 0

- [ ] **Step 4: Commit**

```bash
git add tests/ docs/superpowers/plans/2026-11-01-cloud-sync-per-record.md
git commit -m "test: add integration tests for per-record sync and mark plan complete"
```
