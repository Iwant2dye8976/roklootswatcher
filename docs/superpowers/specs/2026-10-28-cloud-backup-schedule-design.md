# Design: Periodic and On-Exit Cloud Auto-Backup

## 1. Intent & The Why
The existing auto-backup triggered a full vault JSON upload 3 seconds after every new report. During heavy gameplay (fort trains or barbarian chaining), reports arrive in rapid succession, resulting in excessive network usage, log spam, and wasted Supabase storage/egress quotas. We are replacing this with a user-configurable periodic interval and an independent on-exit backup.

## 2. Goals, Non-Goals, and Anti-Goals
- **Goals:**
  - Allow users to configure periodic auto-backup with a custom interval in minutes (5 to 1440 minutes, default 30).
  - Provide an independent toggle for auto-backup on full application exit.
  - Only trigger periodic or on-exit uploads if new reports have arrived since the last backup.
  - Keep exit backup bounded by a strict network timeout (4 seconds) so the app never hangs on shutdown.
  - Ensure minimizing to the system tray remains instant with zero network requests.
- **Non-Goals:**
  - Delta or chunked vault synchronization (full vault JSON upload remains the mechanism).
  - Cloud conflict resolution or two-way auto-merging across multiple simultaneous machines.
- **Anti-Goals:**
  - Never block the UI thread during normal operation or periodic syncs.
  - Never prevent or hang application exit if the internet connection is dead or slow.

## 3. Constraints
- **Framework:** `eframe` / `egui` 0.29 on Windows.
- **Networking:** `ureq` with synchronous timeout capabilities.
- **Local persistence:** `AppConfig` stored as JSON via `serde`.
- **Localization:** Bilingual support (English and Vietnamese) for all new UI strings.

## 4. Approaches Considered
- **Approach 1 (Chosen): Bounded Synchronous Exit + Background Interval Poller**
  - *Periodic sync:* Elapsed time is tracked against the interval in `poll_cloud_events`. If elapsed $\ge$ interval and new data exists, dispatches the existing non-blocking worker thread.
  - *Exit sync:* In `quit_app`, flushes local vault files, then calls `backup_vault` synchronously with a 4-second timeout if `backup_on_exit` is enabled and new records exist.
  - *Trade-offs:* Guaranteed completion before process termination without event loop race conditions or shutdown hangs. Exit is delayed only by the active upload time (~1–2s, max 4s).
- **Approach 2 (Rejected): Asynchronous Exit Interception with Progress UI**
  - Intercepts viewport close event with `CancelClose` and renders a modal spinner until the worker thread returns or a safety timer triggers.
  - *Trade-offs:* Complex state machine prone to hanging shutdown or conflicting with Windows OS signals and tray exit events.

## 5. Detailed Design & Decisions
### 5.1 Configuration (`CloudAuthConfig` in `src/model.rs`)
- `auto_backup`: Retained as the toggle for periodic auto-backup (boolean, default `false`).
- `backup_interval_minutes`: Custom interval in minutes (u32, default `30`, clamped `5..=1440`).
- `backup_on_exit`: Independent toggle for auto-backup on full shutdown (boolean, default `true`).
- Serde defaults ensure full backward compatibility with older `config.json` files.

### 5.2 Dirty / Unbacked-Up Tracking (`src/gui.rs`)
- Track `last_cloud_backup_time: Option<Instant>` and `records_at_last_backup: usize` (or `has_unbacked_data: bool`).
- When a new mail report arrives, set `has_unbacked_data = true`.
- When a cloud backup succeeds, update `last_cloud_backup_time = Some(Instant::now())`, `has_unbacked_data = false`, and save `last_synced`.
- If `has_unbacked_data == false`, periodic ticks and exit backup skip the network call entirely.

### 5.3 Exit Sequence (`src/gui.rs:quit_app`)
1. Flush vault to local disk (`self.watcher.flush_vault_if_dirty()`).
2. Stop watcher service.
3. If `self.cloud_auth.backup_on_exit`, user is logged in, and `has_unbacked_data` is true:
   - Call `backup_vault` with a 4-second timeout.
   - Log outcome (success or warn if timed out / offline).
4. Hide tray icon and exit process via `std::process::exit(0)`.
5. Minimizing to tray (`hide_to_tray`) does not run the exit backup.

### 5.4 Settings UI (`src/gui.rs:render_view_settings`)
Under the Cloud section when logged in:
1. Row 1: `[x] Auto-backup every [ 30 ] minutes` (DragValue / numeric box with 5..=1440 range).
2. Row 2: `[x] Auto-backup on exit`.
3. Bilingual labels added to `src/i18n.rs`.

## 6. What's Left to the Builder
- Wire the UI controls and clamp logic in `src/gui.rs`.
- Configure `ureq` timeout in `backup_vault` or dedicated exit call.
- Add unit tests verifying serialization defaults and dirty check logic.
