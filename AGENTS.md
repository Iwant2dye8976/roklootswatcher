# Agent Instructions: RoK Loot Watcher

Windows desktop & CLI tool monitoring Rise of Kingdoms `mailcache`, decoding Lilith binary format, and aggregating combat loot in `eframe`/`egui`.

## Quick Commands

- **Build**: `cargo build` / `cargo build --release`
- **Test**: `cargo test` (all tests must pass)
- **Check / Lint**: `cargo clippy -- -D warnings`
- **Run GUI**: `cargo run` (or `.\target\debug\rok_watcher.exe`)
- **Run CLI**: `cargo run -- --cli --mode motte --days 7`

## Architecture Map

- `src/main.rs`: Entry point, CLI argument parsing, Windows single-instance guard.
- `src/lilith_stream.rs`: Binary stream decoder (tags: `0x00` Null, `0x01` Bool, `0x02` U8, `0x03` F64, `0x04` String, `0x05` Table, `0xFF` End).
- `src/classifier.rs`: Mail classifier (`motte`, `fort`, `barbarian`, `gathering`, `reward_other`).
- `src/vault.rs`: Persistent JSON database (`rok_history_vault.json`) and raw binary backup (`mail_vault/`).
- `src/watcher.rs`: Real-time filesystem observer using `notify` with debouncing and Windows shared reads.
- `src/analyzer.rs`: Time-zone filtering (UTC+7 / Vietnam time) and loot KPI math.
- `src/gui.rs`: Immediate-mode UI via `eframe` / `egui` (Dashboard, Battles table, Inventory, Settings).
- `src/single_instance.rs` & `src/tray.rs`: Windows Mutex, activation event signaling, and system tray.

## Invariants & Rules

1. **Working Directory**: In GUI mode, resolve relative files against `std::env::current_exe()?.parent()`. Windows Autostart spawns from `C:\Windows\System32`.
2. **Safe File I/O**:
   - Mail reads must use Windows full share flags (`FILE_SHARE_READ | WRITE | DELETE = 7`) via `vault::read_file_shared`.
   - Vault and export writes must write to a `.tmp` file first and rename atomically. Never overwrite vault directly without validating input.
3. **No UI Blocking**:
   - Heavy parsing must run on Rayon worker threads in batches (`MAIL_DECODE_BATCH_SIZE`).
   - Never decode files or perform blocking I/O inside `egui::App::update`.
4. **Timestamps & Dates**:
   - RoK binary mail timestamps are microsecond epoch values (`raw_time / 1_000_000` to get seconds).
   - Relative filters (Today, 7 Days) must use rolling day counts against `current_date_vn()`, not static dates.
5. **Branching & Worktree Policy**:
   - If the task can be done on `main`, do it directly on `main`—no worktree needed.
   - Use a git worktree (`git worktree add -b <branch> ../<worktree-dir> main`) only when strict isolation is required (e.g. parallel sub-agents or high-risk experimental spikes). Merge into `main` and clean up immediately after verification.
6. **Automatic Version Bumping**:
   - Automatically increment the app version in `Cargo.toml` after any code or feature changes (patch for bugfixes/refactors, minor for new features).
   - Keep `Cargo.lock` updated by running `cargo check` after bumping.

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
| **Completion Check** | `verification-before-completion` | Run `cargo test` and `cargo clippy`, bump version in `Cargo.toml`, and confirm clean output before claiming tasks done. |
