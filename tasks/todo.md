# Tasks: PvP Battle Tracking & Troop Trade Ratios

## Task 1: Domain & Math Foundation
**Description:** Add `TroopCombatStats`, `FilterMode::PvP`, and unit KP formulas (T1–T5 multipliers) in `src/model.rs`.
**Acceptance criteria:**
- [x] `FilterMode::PvP` added with string alias `"pvp"`.
- [x] `TroopCombatStats` struct created with commander pair fields and safe `trade_ratio_pct()` calculation.
- [x] Comprehensive unit tests for KP formulas across all unit tiers.
**Verification:**
- [x] `cargo test model::tests` passes.
**Files likely touched:** `src/model.rs`

## Task 2: PvP Mail Classifier & Extraction
**Description:** Detect PvP battle mails in `src/classifier.rs` and extract commanders, casualties taken/dealt, and calculate KP values.
**Acceptance criteria:**
- [x] `classify_mail()` detects player battle reports as `"pvp"`.
- [x] Extraction functions populate commanders and KP values onto `BattleRecord`.
- [x] Unit tests for open-field and rally sample payloads pass.
**Verification:**
- [x] `cargo test classifier::tests` passes.
**Files likely touched:** `src/classifier.rs`

## Task 3: SQLite Schema Migration for PvP Records
**Description:** Add optional columns for commanders, KP gained, and KP lost in `src/vault.rs` and `src/vault_migrate.rs`.
**Acceptance criteria:**
- [x] Schema migration transparently alters `reports` table if columns are missing.
- [x] Records can be inserted, queried, and roundtripped through SQLite with PvP attributes intact.
**Verification:**
- [x] `cargo test vault::tests` passes.
**Files likely touched:** `src/vault.rs`, `src/vault_migrate.rs`

## Checkpoint 1: Foundation Complete
- [x] Tasks 1-3 complete and tested.

## Task 4: Troop Session Aggregator
**Description:** Implement rolling today aggregation in `src/analyzer.rs` to group battle records by unique commander pair and compute session KP totals and trade percentages.
**Acceptance criteria:**
- [x] Aggregation function produces a map of `(Primary, Secondary) -> TroopCombatStats`.
- [x] Handles 1-commander marches, tie breaks, and zero casualty edges.
**Verification:**
- [x] `cargo test analyzer::tests` passes.
**Files likely touched:** `src/analyzer.rs`

## Task 5: Tauri IPC & Overlay DTO Extension
**Description:** Wire `TroopTradeSummaryDto` into `OverlayPayloadDto` in `src/tauri_app/dto.rs` and update `overlay_mgr.rs`.
**Acceptance criteria:**
- [x] `OverlayPayloadDto` transmits today's aggregated troop trade summaries to the frontend.
- [x] Background overlay polling loop includes latest troop trades.
**Verification:**
- [x] `cargo test` passes.
**Files likely touched:** `src/tauri_app/dto.rs`, `src/tauri_app/overlay_mgr.rs`

## Task 6: Floating HUD Overlay Troop Card
**Description:** Build the real-time Troop Trades display in `ui/src/views/OverlayView.tsx` showing commander pairs, KP gained, and color-coded trade ratio percentages.
**Acceptance criteria:**
- [x] Displays active troops for today on the overlay.
- [x] Highlights ratio $\ge 120\%$ (green), $100\%-119\%$ (cyan), $<100\%$ (amber/rose).
- [x] Respects user HUD styling (transparency, scale, opacity).
**Verification:**
- [x] `npm --prefix ui run build` succeeds without TypeScript errors.
**Files likely touched:** `ui/src/types.ts`, `ui/src/views/OverlayView.tsx`

## Task 7: Battles View Integration & Final Polish
**Description:** Add PvP filter option and combat details to `ui/src/views/BattlesView.tsx`.
**Acceptance criteria:**
- [x] Users can filter by "PvP Battles" in the main UI.
- [x] Battle items show commanders and KP trade summary.
**Verification:**
- [x] `npm --prefix ui run build` succeeds.
- [x] `cargo test` passes 100%.
- [x] `cargo clippy -- -D warnings` zero warnings.
- [x] Bump version in `Cargo.toml` and compile release binary `.\rok_watcher.exe`.
**Files likely touched:** `ui/src/views/BattlesView.tsx`, `Cargo.toml`
