# Periodic and On-Exit Cloud Auto-Backup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace per-mail auto-backup with a user-configurable periodic backup interval (in minutes) and an independent on-exit backup bounded by a strict network timeout.

**Architecture:** Update `CloudAuthConfig` to store `backup_interval_minutes` and `backup_on_exit`. In `RoKLootWatcherApp`, maintain dirty/unbacked-data tracking; `poll_cloud_events` checks an elapsed timer against the custom interval, while `quit_app` executes a bounded synchronous upload (4s timeout) before process termination. Settings UI provides independent controls for both behaviors with bilingual support.

**Tech Stack:** Rust (2021 edition), `eframe`/`egui` 0.29, `ureq` 2.10, `serde`/`serde_json`, `chrono`.

**Spec:** [docs/superpowers/specs/2026-10-28-cloud-backup-schedule-design.md](file:///C:/roklootswatcher/docs/superpowers/specs/2026-10-28-cloud-backup-schedule-design.md)

## Global Constraints
- Clamped interval range: `5..=1440` minutes (default 30 minutes).
- Exit backup timeout: strict 4 seconds timeout using `ureq` agent configuration.
- Minimizing to tray (`CloseBehavior::Tray`) MUST remain instantaneous with zero network calls.
- Exit backup only triggers on full termination (`quit_app`).
- If no new reports arrived since the last backup (`!has_unbacked_data`), uploads are skipped.
- All new UI text must be localized in both English and Vietnamese in `src/i18n.rs`.

## Review Focus
1. `backup_interval_minutes` input is 0 or < 5 -> Clamped to 5 minutes minimum.
2. `backup_interval_minutes` input is > 1440 -> Clamped to 1440 minutes (24 hours) maximum.
3. Network connection times out or fails on exit -> `quit_app` catches error, logs warning, and terminates process within 4s without freezing.
4. Legacy `config.json` without new fields -> Deserializes gracefully to 30 minutes and `backup_on_exit: true`.
5. Periodic interval elapses with zero new records -> Upload is skipped, preserving Supabase bandwidth.

---

### Task 1: Model & Configuration for Scheduled and Exit Cloud Backup

**Files:**
- Modify: `src/model.rs:1615-1640`
- Test: `src/model.rs:2905-2930`

**Interfaces:**
- Produces:
  - `CloudAuthConfig::backup_interval_minutes: u32`
  - `CloudAuthConfig::backup_on_exit: bool`
  - `CloudAuthConfig::clamped_interval_minutes(&self) -> u32`
  - Serde helper `fn default_backup_interval_minutes() -> u32 { 30 }`

- [ ] **Step 1: Write the failing test for configuration serialization, defaults, and interval clamping**

In `src/model.rs` tests:
```rust
#[test]
fn test_cloud_auth_backup_schedule_config_defaults_and_clamping() {
    let legacy_json = r#"{"auto_backup":true}"#;
    let auth: CloudAuthConfig = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(auth.backup_interval_minutes, 30);
    assert!(auth.backup_on_exit);
    assert_eq!(auth.clamped_interval_minutes(), 30);

    let mut auth_low = CloudAuthConfig::default();
    auth_low.backup_interval_minutes = 2;
    assert_eq!(auth_low.clamped_interval_minutes(), 5);

    let mut auth_high = CloudAuthConfig::default();
    auth_high.backup_interval_minutes = 2000;
    assert_eq!(auth_high.clamped_interval_minutes(), 1440);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --bin rok_watcher test_cloud_auth_backup_schedule_config_defaults_and_clamping`
Expected: FAIL with "no field `backup_interval_minutes`"

- [ ] **Step 3: Implement fields and clamping in `src/model.rs`**

Add `backup_interval_minutes` and `backup_on_exit` to `CloudAuthConfig` with serde defaults:
```rust
fn default_backup_interval_minutes() -> u32 {
    30
}

// In CloudAuthConfig:
#[serde(default = "default_backup_interval_minutes")]
pub backup_interval_minutes: u32,
#[serde(default = "default_true")]
pub backup_on_exit: bool,
```
Implement `clamped_interval_minutes(&self) -> u32`:
```rust
impl CloudAuthConfig {
    pub fn clamped_interval_minutes(&self) -> u32 {
        self.backup_interval_minutes.clamp(5, 1440)
    }
}
```
Update `CloudAuthConfig::default()` to initialize `backup_interval_minutes: 30, backup_on_exit: true`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --bin rok_watcher test_cloud_auth_backup_schedule_config_defaults_and_clamping`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(model): add backup_interval_minutes and backup_on_exit to CloudAuthConfig"
```

---

### Task 2: Localization Keys for Cloud Auto-Backup Settings

**Files:**
- Modify: `src/i18n.rs:160-220`, `src/i18n.rs:320-380`, `src/i18n.rs:650-750`
- Test: `src/i18n.rs:tests`

**Interfaces:**
- Produces:
  - `TrKey::SettingsCloudAutoBackupPeriodic`
  - `TrKey::SettingsCloudAutoBackupMinutes`
  - `TrKey::SettingsCloudAutoBackupOnExit`

- [ ] **Step 1: Write the failing test for new localization keys**

In `src/i18n.rs` tests:
```rust
#[test]
fn test_cloud_backup_schedule_localization_keys() {
    assert!(!AppLanguage::Vietnamese.t(TrKey::SettingsCloudAutoBackupPeriodic).is_empty());
    assert!(!AppLanguage::English.t(TrKey::SettingsCloudAutoBackupPeriodic).is_empty());
    assert!(!AppLanguage::Vietnamese.t(TrKey::SettingsCloudAutoBackupMinutes).is_empty());
    assert!(!AppLanguage::English.t(TrKey::SettingsCloudAutoBackupMinutes).is_empty());
    assert!(!AppLanguage::Vietnamese.t(TrKey::SettingsCloudAutoBackupOnExit).is_empty());
    assert!(!AppLanguage::English.t(TrKey::SettingsCloudAutoBackupOnExit).is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --bin rok_watcher test_cloud_backup_schedule_localization_keys`
Expected: FAIL with variant not found in `TrKey`

- [ ] **Step 3: Implement localization keys in `src/i18n.rs`**

Add variants to `TrKey` enum and match arms in `AppLanguage::t`:
- `SettingsCloudAutoBackupPeriodic`:
  - EN: "Periodic auto-backup every"
  - VI: "Tự động sao lưu định kỳ mỗi"
- `SettingsCloudAutoBackupMinutes`:
  - EN: "minutes"
  - VI: "phút"
- `SettingsCloudAutoBackupOnExit`:
  - EN: "Auto-backup on exit"
  - VI: "Tự động sao lưu khi thoát ứng dụng"
Add keys to `ALL_KEYS` array.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --bin rok_watcher test_cloud_backup_schedule_localization_keys`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/i18n.rs
git commit -m "feat(i18n): add localization keys for periodic and on-exit backup"
```

---

### Task 3: Timeout-Bounded Cloud Vault Backup

**Files:**
- Modify: `src/cloud_sync.rs:382-435`
- Test: `src/cloud_sync.rs:tests`

**Interfaces:**
- Consumes: `ureq::builder()`
- Produces: `backup_vault(..., timeout: Option<std::time::Duration>) -> Result<(usize, Option<(String, String)>), String>`

- [ ] **Step 1: Write the failing test for backup_vault with custom timeout parameter**

In `src/cloud_sync.rs` tests:
```rust
#[test]
fn test_backup_vault_accepts_timeout_param() {
    let dummy_path = std::path::Path::new("dummy_nonexistent_vault.json");
    let res = backup_vault(
        "https://example.supabase.co",
        "dummy_key",
        "dummy_token",
        None,
        "dummy_user",
        dummy_path,
        Some(std::time::Duration::from_secs(4)),
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Cannot read vault file"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --bin rok_watcher test_backup_vault_accepts_timeout_param`
Expected: FAIL with argument count mismatch

- [ ] **Step 3: Update `backup_vault` to accept timeout and configure `ureq::Agent`**

In `src/cloud_sync.rs`:
```rust
pub fn backup_vault(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    user_id: &str,
    vault_path: &Path,
    timeout: Option<std::time::Duration>,
) -> Result<(usize, Option<(String, String)>), String>
```
Create agent:
```rust
let effective_timeout = timeout.unwrap_or(std::time::Duration::from_secs(15));
let agent = ureq::builder()
    .timeout(effective_timeout)
    .build();
```
Use `agent.post(&url)...` instead of global `ureq::post(&url)`.
Update all existing call sites (including `refresh_session` and tests) to pass timeout accordingly.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --bin rok_watcher test_backup_vault_accepts_timeout_param`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/cloud_sync.rs
git commit -m "feat(cloud): add configurable timeout to backup_vault via ureq Agent"
```

---

### Task 4: Integrate Periodic Interval & On-Exit Backup in GUI

**Files:**
- Modify: `src/gui.rs:205-225`, `src/gui.rs:640-660`, `src/gui.rs:688-755`, `src/gui.rs:875-920`, `src/gui.rs:4475-4505`
- Test: `src/gui.rs:tests`

**Interfaces:**
- Consumes:
  - `CloudAuthConfig::clamped_interval_minutes`
  - `CloudAuthConfig::backup_on_exit`
  - `TrKey::SettingsCloudAutoBackupPeriodic`
  - `TrKey::SettingsCloudAutoBackupMinutes`
  - `TrKey::SettingsCloudAutoBackupOnExit`
  - `backup_vault(..., Some(Duration::from_secs(4)))`

- [ ] **Step 1: Write failing tests for dirty state tracking and interval calculation**

In `src/gui.rs` tests:
```rust
#[test]
fn test_cloud_auto_backup_dirty_flag_state() {
    let mut config = AppConfig::default();
    config.cloud_auth.auto_backup = true;
    config.cloud_auth.backup_interval_minutes = 15;
    config.cloud_auth.backup_on_exit = true;

    assert_eq!(config.cloud_auth.clamped_interval_minutes(), 15);
    assert!(config.cloud_auth.backup_on_exit);
}
```

- [ ] **Step 2: Run test to verify compilation and baseline**

Run: `cargo test --bin rok_watcher test_cloud_auto_backup_dirty_flag_state`
Expected: PASS

- [ ] **Step 3: Implement periodic timer and on-exit execution in `src/gui.rs`**

1. In `RoKLootWatcherApp`:
   - Add field `pub has_unbacked_data: bool` (default true on startup if vault has records).
   - Add field `pub last_cloud_backup_time: Option<std::time::Instant>`.
2. In `WatcherUiEvent::MailDetected`:
   - When `is_new == true`: `self.has_unbacked_data = true`. Remove `self.pending_auto_backup = Some(...)` 3-second debounce.
3. In `poll_cloud_events(ctx)`:
   - Check periodic interval:
     ```rust
     if !self.cloud_busy && self.cloud_auth.auto_backup && self.cloud_auth.access_token.is_some() && self.has_unbacked_data {
         let interval_duration = std::time::Duration::from_secs(self.cloud_auth.clamped_interval_minutes() as u64 * 60);
         let should_run = match self.last_cloud_backup_time {
             Some(last) => std::time::Instant::now().duration_since(last) >= interval_duration,
             None => true,
         };
         if should_run {
             self.trigger_cloud_backup();
         }
     }
     ```
   - When cloud backup succeeds in `poll_cloud_events`: set `self.has_unbacked_data = false; self.last_cloud_backup_time = Some(std::time::Instant::now());`.
4. In `quit_app(&self)`:
   - If `self.cloud_auth.backup_on_exit && self.has_unbacked_data && self.cloud_auth.access_token.is_some()`:
     - Run synchronous `backup_vault` with `Some(std::time::Duration::from_secs(4))`.
     - Log success or warning on error/timeout.
   - Proceed to `self.watcher.stop()`, tray icon hide, and `process::exit(0)`.
5. In `render_view_settings`:
   - Replace old auto_backup checkbox with:
     - Row 1: Checkbox for `self.cloud_auth.auto_backup` ("Periodic auto-backup every"), followed by `egui::DragValue::new(&mut self.cloud_auth.backup_interval_minutes).clamp_range(5..=1440)` and label ("minutes").
     - Row 2: Checkbox for `self.cloud_auth.backup_on_exit` ("Auto-backup on exit").

- [ ] **Step 4: Run full test suite and verify clean compilation**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/gui.rs
git commit -m "feat(gui): integrate periodic interval and bounded on-exit cloud backup"
```
