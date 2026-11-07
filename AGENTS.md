# Agent Instructions: RoK Loot Watcher

Windows desktop & CLI tool monitoring Rise of Kingdoms `mailcache`, decoding Lilith binary format, and aggregating combat loot in `eframe`/`egui`. Features persistent SQLite WAL storage, per-record Supabase cloud sync, an in-game floating HUD overlay, and bilingual support (EN/VI).

## Quick Commands

- **Build**: `cargo build` / `cargo build --release`
- **Build & Update Root Binary**: `cargo build --release; Copy-Item -Force .\target\release\rok_watcher.exe .\rok_watcher.exe`
- **Test**: `cargo test` (339+ tests across unit & integration suites must pass)
- **Check / Lint**: `cargo clippy -- -D warnings`
- **Run GUI**: `.\rok_watcher.exe` (or `cargo run`)
- **Run CLI**: `.\rok_watcher.exe --cli --mode motte --days 7` (or `cargo run -- --cli --mode motte --days 7`)

## Architecture Map

### Core & Domain
- `src/main.rs`: Entry point, CLI argument parsing, Windows single-instance guard, and subsystem coordinator.
- `src/model.rs`: Core domain models (`ReportRecord`, `BattleReport`, `LootItem`, `FilterOptions`, `AppConfig`, `OverlayConfig`, etc.).
- `src/lilith_stream.rs`: Binary stream decoder for Lilith mail cache (tags: `0x00` Null, `0x01` Bool, `0x02` U8, `0x03` F64, `0x04` String, `0x05` Table, `0xFF` End).
- `src/classifier.rs`: Mail classifier & payload parser (`motte`, `fort`, `barbarian`, `gathering`, `reward_other`, account UID extraction).
- `src/loot_catalog.rs`: Embedded item database (`loot.json`) for instant zero-file runtime lookup of names, icons, and resource pack definitions.

### Storage & Sync
- `src/vault.rs`: Persistent SQLite database (`rok_history_vault.db`) in WAL mode, O(1) deduplication, multi-account UID isolation, and raw binary backup (`mail_vault/`).
- `src/vault_migrate.rs`: Automatic migration engine from legacy JSON vault (`rok_history_vault.json`) to SQLite schema.
- `src/cloud_sync.rs`: Background actor-based Supabase synchronization (per-record batch upsert, delta sync, retry exponential backoff, loopback Google OAuth server, offline queue).
- `src/exporters.rs`: Export engine for CSV, JSON, and clipboard summaries.

### Ingestion & Analysis
- `src/watcher.rs`: Real-time filesystem observer using `notify` with debouncing, multi-account separation, and Windows full-share reads.
- `src/analyzer.rs`: Time-zone filtering (UTC+7 / Vietnam time), loot KPI math, AP economics, and target drop rate statistics.

### In-Game Overlay & Platform Integration
- `src/overlay_tracker.rs`: Real-time event aggregator and state tracker for recent combat drops.
- `src/overlay_ui.rs`: Always-on-top transparent in-game overlay (Full HUD, Mini HUD, Combined HUD) with click-through, global hotkeys, pinned counters, and D-pad position controls.
- `src/autostart.rs`: Windows registry autostart manager (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
- `src/single_instance.rs`: Windows Mutex and activation event signaling for single-process enforcement.
- `src/tray.rs`: System tray icon, minimize-to-tray, and context menu.
- `src/i18n.rs`: Bilingual localization engine (English / Vietnamese).

### GUI Architecture (`src/gui/`)
Immediate-mode UI built with `eframe` / `egui`:
- `src/gui/mod.rs`: App state coordinator, render loop, hotkey polling, and route dispatcher.
- `src/gui/views/dashboard.rs`: Loot overview, KPI summary cards, and quick targets.
- `src/gui/views/battles.rs`: Battles table with multi-column sorting, pagination, and search.
- `src/gui/views/inventory.rs`: Item catalog aggregation, resource conversions, and speedup totals.
- `src/gui/views/settings.rs`: Watcher paths, cloud sync credentials, overlay options, and autostart toggles.
- `src/gui/views/inspector.rs`: Low-level mail viewer and raw payload inspector.
- `src/gui/views/sidebar.rs`: Navigation sidebar, language selector, and status indicators.
- `src/gui/components.rs`: Reusable UI components (cards, badge pills, tables, stat rows).
- `src/gui/lifecycle.rs`: Window startup centering, restore geometry, and persistence.
- `src/gui/theme.rs`: Color tokens, fonts, and dark/light styling.

## Invariants & Rules

1. **Working Directory**: In GUI mode, resolve relative files against `std::env::current_exe()?.parent()`. Windows Autostart spawns from `C:\Windows\System32`.
2. **Safe File I/O & SQLite Vault**:
   - Mail reads must use Windows full share flags (`FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE = 7`) via `vault::read_file_shared`.
   - Vault is a persistent SQLite database (`rok_history_vault.db`) in WAL mode with connection busy timeouts and atomic transactions.
   - Any raw mail backup goes to `mail_vault/`.
   - Export writes must write to a `.tmp` file first and rename atomically.
3. **No UI Blocking**:
   - Heavy parsing must run on Rayon worker threads in batches (`MAIL_DECODE_BATCH_SIZE`).
   - Cloud sync operations must run asynchronously via background Actor channel workers (`CloudSyncWorker`).
   - Never decode files, run heavy queries, or perform blocking network/disk I/O inside `egui::App::update`.
4. **Timestamps & Dates**:
   - RoK binary mail timestamps are microsecond epoch values (`raw_time / 1_000_000` to get seconds).
   - Relative filters (Today, 7 Days, etc.) must use rolling day counts against `current_date_vn()` (UTC+7), not static dates.
5. **Multi-Account & UID Isolation**:
   - Reports are associated with user account IDs (`uid`).
   - Filtering and deletion of non-main records must respect the user's selected primary UID.
6. **Branching & Worktree Policy**:
   - If the task can be done on `main`, do it directly on `main`—no worktree needed.
   - Use a git worktree (`git worktree add -b <branch> ../<worktree-dir> main`) only when strict isolation is required (e.g. parallel sub-agents or high-risk experimental spikes). Merge into `main` and clean up immediately after verification.
7. **Automatic Version Bumping**:
   - Automatically increment the app version in `Cargo.toml` after any code or feature changes (patch for bugfixes/refactors, minor for new features).
   - Keep `Cargo.lock` updated by running `cargo check` after bumping.
8. **Automatic App Build & Binary Update**:
   - After any code changes, tests, and version bumping, automatically compile the release executable:
     ```powershell
     cargo build --release
     ```
   - Copy the newly compiled binary from `target\release\rok_watcher.exe` to `.\rok_watcher.exe` in the workspace root:
     ```powershell
     Copy-Item -Force .\target\release\rok_watcher.exe .\rok_watcher.exe
     ```
   - Verify that `.\rok_watcher.exe` is updated and runnable.

## Post-Change Verification Checklist

Before completing any task:
1. `cargo test`: Ensure all unit and integration tests pass.
2. `cargo clippy -- -D warnings`: Ensure zero warnings.
3. Bump version in `Cargo.toml` and update `Cargo.lock` (`cargo check`).
4. **Auto-build app**: Run `cargo build --release` and copy the output binary to `.\rok_watcher.exe`.

## Compatible Skills for Efficient Coding

| Task | Skill | When to Use |
|---|---|---|
| **Feature Work / Fixes** | `ponytail` | Default mode. Write the smallest change that fully works, zero bloat, native stdlib over dependencies. |
| **Task Workspace Isolation** | `using-git-worktrees` | Spin up an isolated worktree for independent or parallel tasks. |
| **Branch Integration** | `finishing-a-development-branch` | Merge verified worktree branches back into `main` and clean up. |
| **Quality Review** | `ponytail-review` | Inspect diffs for correctness, edge cases, safety under load, and unnecessary lines. |
| **Codebase Audit** | `ponytail-audit` | One-shot audit of bugs, performance, security, and dead code. |
| **Debugging** | `systematic-debugging` | Root-cause file lock failures, decoding errors, or UI freeze issues before touching code. |
| **New Parsers / Calculations** | `test-driven-development` | Write tests first for new item IDs, multiplier formulas, or binary tags. |
| **GUI & Styling** | `ui-ux-pro-max` / `design-system` | Consult for `egui` responsive layouts, contrast tokens, and table column sizing. |
| **Completion Check** | `verification-before-completion` | Run `cargo test` and `cargo clippy`, bump version in `Cargo.toml`, build release binary, and confirm clean output before claiming tasks done. |
