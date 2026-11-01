# Design Document: Per-Record Cloud Synchronization & Background Worker

- **Date**: 2026-11-01
- **Feature**: Per-Record Cloud Sync (`report_records`) & Background Sync Engine
- **Status**: Proposed / Under Review
- **Author**: Antigravity Pair Programming

---

## 1. Executive Summary & Intent

RoK Loot Watcher currently persists battle and gathering reports locally in an atomic SQLite database (`rok_history_vault.db` with WAL mode). However, cloud synchronization with Supabase still uploads the entire vault as a monolithic JSON payload into a single row in `user_vaults`. 

As player vaults grow beyond 5,000–10,000 reports, this monolithic model hits two critical limits:
1. **Supabase Kong API Gateway Limit (`HTTP 413 Payload Too Large`)**: Hard 10 MB ceiling on request body size.
2. **On-Exit Backup Timeout**: Uploading tens of megabytes over residential broadband frequently exceeds the application's strict 4-second on-exit window.

This project transitions cloud storage to a **per-record architecture** using a new Supabase table (`report_records`), combined with a **dedicated background worker thread** that continuously streams unsynced records in small, idempotent batches. This eliminates payload limits, prevents UI freezing, makes exit instantaneous, and gracefully migrates legacy data without manual user intervention.

---

## 2. Goals, Non-Goals & Anti-Goals

### Goals
- **Unlimited Scalability**: Vault backups never hit the 10 MB Kong gateway limit, supporting 100,000+ reports per user seamlessly.
- **Incremental Delta Sync**: Only new or modified reports (`synced_at IS NULL`) are sent to the cloud, typically consuming only ~2–20 KB per sync cycle.
- **Zero UI Interruption**: Cloud synchronization runs entirely on a dedicated background worker thread with retry logic and exponential backoff; UI frames remain unblocked at 60 FPS.
- **Instantaneous Exit**: Application shutdown takes milliseconds because there is no 50 MB payload to upload on exit; any unsynced rows are safely persisted in local SQLite and will automatically sync on the next launch.
- **Self-Healing Legacy Migration**: Seamlessly transition existing backups from `user_vaults` into `report_records` without requiring manual database migration scripts from users.
- **Bidirectional Restore**: Provide fast, paginated cloud restore that reconstructs the local SQLite vault from `report_records`.

### Non-Goals
- Real-time multiplayer synchronization or cross-player data sharing (reports remain private to the authenticated player via Supabase Row-Level Security).
- Deprecating SQLite local storage (SQLite remains the primary source of truth locally).
- Modifying game packet capture or report parsing logic.

### Anti-Goals
- Blocking the UI thread or holding up app exit for more than 2 seconds when network issues occur.
- Forcing users to run manual SQL DDL commands in their Supabase console before the app will function.
- Losing local report history if the cloud connection is unavailable.

---

## 3. Approaches Considered & Trade-offs

| Approach | Description | Trade-offs | Verdict |
|---|---|---|---|
| **Approach 1: Client-Orchestrated Migration with Dedicated Worker Actor** | Client syncs delta records in batches of 200 to `report_records`. Background thread handles retries and exit flushes. If `report_records` is empty during restore, falls back to unpacking legacy `user_vaults` and backfilling. | Requires slightly more client-side fallback code for legacy restore, but is 100% resilient, requires zero manual SQL intervention, and never breaks existing installations. | **Chosen (User Selected)** |
| **Approach 2: Strict Server-Side SQL Migration** | Admin/developer runs a SQL script on Supabase that converts `user_vaults` JSON into `report_records` rows via PostgreSQL `jsonb_to_recordset()`. Client only communicates with `report_records`. | Simpler client code, but requires manual Supabase console intervention. If a user runs the client before the SQL script is run, legacy data cannot be restored. | Rejected |
| **Approach 3: Cloudflare / Supabase Edge Functions** | Custom Edge Function receives compressed delta dumps and writes to PostgreSQL. | Adds deployment dependencies and maintenance overhead without providing any advantage over native PostgREST batch upserts. | Rejected |

---

## 4. Architecture & Technical Design

### 4.1 Supabase Schema (`report_records`)

The new cloud table stores individual reports with metadata columns for fast indexing and a flexible `data` JSONB column for the full `BattleRecord` struct. The developer will run this migration script directly in the Supabase SQL Editor:

```sql
-- DDL for Supabase SQL Editor (also saved to resources/supabase_migration_report_records.sql)
CREATE TABLE IF NOT EXISTS report_records (
    id          UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id     UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
    dedup_key   TEXT NOT NULL,
    mail_id     TEXT NOT NULL DEFAULT '',
    battle_type TEXT NOT NULL DEFAULT '',
    time_raw    BIGINT NOT NULL DEFAULT 0,
    time_utc    TEXT NOT NULL DEFAULT '',
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT now(),
    UNIQUE (user_id, dedup_key)
);

-- Row Level Security (RLS)
ALTER TABLE report_records ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage their own report records"
ON report_records
FOR ALL
USING (auth.uid() = user_id)
WITH CHECK (auth.uid() = user_id);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_report_records_user_time ON report_records (user_id, time_raw);
CREATE INDEX IF NOT EXISTS idx_report_records_user_type ON report_records (user_id, battle_type);
```

Because the developer provisions `report_records` via SQL editor before release, the client code targets `report_records` as the authoritative store. If `report_records` returns HTTP 404, the client reports a clear configuration error ("Bảng report_records chưa được khởi tạo trên Supabase") rather than silently attempting an obsolete 50 MB monolithic upload.

### 4.2 SQLite Local Schema Integration (`vault.rs`)

Local SQLite already defines `synced_at INTEGER` in `battle_records` (default `NULL`).
We add an index and three primary methods to `MailVault`:

1. **Index**:
   ```sql
   CREATE INDEX IF NOT EXISTS idx_synced_at ON battle_records(synced_at);
   ```
2. **`get_unsynced_records(limit: usize) -> Vec<(String, BattleRecord)>`**:
   - Queries `SELECT dedup_key, ... FROM battle_records WHERE synced_at IS NULL ORDER BY time_raw ASC LIMIT ?`.
3. **`mark_records_synced(dedup_keys: &[String], timestamp: i64) -> rusqlite::Result<usize>`**:
   - Updates `synced_at = ?` for all confirmed keys inside a single write transaction.
4. **`get_sync_counts() -> (usize, usize)`**:
   - Returns `(total_records, unsynced_records)`.

### 4.3 Background Sync Engine (`cloud_sync.rs`)

A long-lived worker thread manages all cloud interactions (both delta uploads and restores) to prevent concurrent SQLite access:

```
[Watcher / GUI Events] 
       │ (Channel: SyncCommand)
       ▼
[CloudSyncWorker Thread] ──(Queries SQLite)──▶ [MailVault (WHERE synced_at IS NULL)]
       │
       ├── Batch: 100-200 records (~250 KB)
       ▼
[Supabase PostgREST: /rest/v1/report_records]
       │ (Prefer: resolution=merge-duplicates)
       ▼
[Update Local SQLite: synced_at = now()]
```

#### Command Channel:
```rust
pub enum SyncCommand {
    TriggerDelta,                 // Debounced trigger after watcher detects new mail (when auto_backup = true)
    SyncNow,                      // User clicked "Sync Now" button
    Restore(mpsc::Sender<RestoreProgress>), // Coordinated paginated restore
    Shutdown(mpsc::Sender<()>),   // App exit signal, flushes up to 2s then replies
}
```

#### Worker Behavior & User Configuration:
- **Respects Existing Settings**:
  - `auto_backup = true`: Worker syncs deltas on file watcher events (~5s debounce) and runs a periodic sweep every `backup_interval_minutes`.
  - `auto_backup = false`: Records accumulate locally in SQLite with `synced_at IS NULL`; sync only occurs on manual `SyncNow`.
  - `backup_on_exit = true`: Worker executes a bounded flush (max 2 seconds) upon `Shutdown`. If the timeout expires or network is offline, the app exits cleanly; unsynced records remain in SQLite for the next session.
- **Batch Size**: 200 records per HTTP request (~250–350 KB payload).
- **Endpoint**: `POST /rest/v1/report_records` with header `Prefer: resolution=merge-duplicates`.
- **Token Refresh**: Automatically detects HTTP 401 and refreshes OAuth tokens via Supabase Auth.
- **Retries**: On network errors, backs off exponentially (2s, 4s, 8s, up to 60s) without blocking GUI.
- **Account Tracking**: Stores `synced_user_id` in `vault_meta` to prevent accidental cross-account uploads if a different user logs in on the same workstation.

### 4.4 Transition & Restore Strategy

#### Idempotent Merge Restore:
When the user triggers "Restore from Cloud":
1. Delta sync is paused inside `CloudSyncWorker`.
2. Worker fetches paginated records from `GET /rest/v1/report_records?user_id=eq.{user_id}&order=time_raw.asc&limit=500&offset={offset}`.
3. If legacy `user_vaults` exists:
   - Worker also checks `user_vaults`. If legacy contains records not yet in `report_records`, it inserts them into SQLite.
   - Because SQLite enforces `PRIMARY KEY (dedup_key)`, merging is completely idempotent and collision-free.
4. Once all records are saved locally with `synced_at = now()`, delta sync resumes. This eliminates any risk of partial migration stranding data.

---

## 5. User Interface Integration (`gui.rs`)

1. **Status Indicator**:
   - Shows live sync status: `Idle (Up to date)`, `Syncing (N remaining)`, `Offline (Retrying in Xs)`.
2. **Sync Button**:
   - "Sync Now" button sends `SyncCommand::SyncNow` to the background thread without freezing the UI or spawning ad-hoc threads.
3. **Application Exit**:
   - Replaces the synchronous `perform_exit_cloud_backup` with a lightweight channel notification:
   ```rust
   let (done_tx, done_rx) = mpsc::channel();
   let _ = sync_tx.send(SyncCommand::Shutdown(done_tx));
   let _ = done_rx.recv_timeout(Duration::from_secs(2));
   ```

---

## 6. What's Left to the Builder

- Exact HTTP query parameters for PostgREST pagination (`order=time_raw.asc&offset={n}&limit={m}`).
- Exponential backoff jitter calculation in the worker loop.
- UI tooltip showing "Synced X of Y records".
- Optional: SQL file artifact in `resources/supabase_migration_report_records.sql` for users who manage self-hosted Supabase instances.

---

## 7. Verification & Testing Plan

1. **Unit Tests**:
   - `get_unsynced_records` returns expected batch size.
   - `mark_records_synced` updates timestamp and excludes records from next query.
   - Fallback logic: mock empty `report_records` response triggers `user_vaults` download.
2. **Integration / Smoke Tests**:
   - Add new mail file via watcher -> observe background worker sync within debounce window.
   - Kill app during sync -> relaunch and verify zero duplicate records and zero lost data.
   - Run on simulated offline network -> verify app UI remains responsive, retries gracefully, and exits instantly.
