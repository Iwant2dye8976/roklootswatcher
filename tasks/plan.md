# Implementation Plan: PvP Battle Tracking & Troop Trade Ratios

## Overview
Decompose the implementation of PvP battle report parsing, Kill Point (KP) calculations, troop commander pair grouping, and live floating HUD overlay display into discrete, testable vertical slices.

## Architecture Decisions
- **Non-breaking Domain Model Extension**: Add optional PvP fields to `BattleRecord` and define dedicated `TroopCombatStats` so existing PvE gathering and barbarian flows remain unaffected.
- **SQLite Schema Migration**: Add new columns to `reports` with default values so existing vaults upgrade transparently without dropping user data.
- **Rolling Today Aggregation**: Aggregate troop stats within `analyzer.rs` for records matching `time_vn` on the current Vietnam date.
- **HUD Integration**: Add `troop_trades` to `OverlayPayloadDto`, rendering a sleek, translucent troop scorecard in `OverlayView.tsx`.

## Task List

### Phase 1: Domain & Math Foundation
- [ ] Task 1: Add `TroopCombatStats`, `FilterMode::PvP`, and KP calculation helpers in `src/model.rs`
- [ ] Task 2: Implement PvP mail classification & extraction logic in `src/classifier.rs`
- [ ] Task 3: Extend SQLite schema in `src/vault.rs` and `src/vault_migrate.rs` for PvP fields

### Checkpoint 1: Engine Tests
- [ ] All unit tests pass in `src/model.rs`, `src/classifier.rs`, and `src/vault.rs`

### Phase 2: Analyzer & IPC Pipeline
- [ ] Task 4: Implement troop session aggregation in `src/analyzer.rs` (KP totals, commander pairs, trade ratio %)
- [ ] Task 5: Expose `TroopTradeSummaryDto` in `src/tauri_app/dto.rs` and wire into `overlay_mgr.rs`

### Checkpoint 2: Data Pipeline
- [ ] IPC tests and overlay geometry/payload tests pass cleanly

### Phase 3: In-Game Floating HUD & Frontend UI
- [ ] Task 6: Update `ui/src/types.ts` and build the live Troop Trades HUD component in `ui/src/views/OverlayView.tsx`
- [ ] Task 7: Add PvP filter option and combat columns to `ui/src/views/BattlesView.tsx`

### Checkpoint 3: Complete & Release
- [ ] Frontend build succeeds (`npm --prefix ui run build`)
- [ ] `cargo test` passes 100%
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Version bump in `Cargo.toml` and release binary built to root `.\rok_watcher.exe`

## Risks and Mitigations
| Risk | Impact | Mitigation |
|---|---|---|
| Binary mail structure variations between open-field and rally mails | Med | Handle missing hero IDs gracefully with optional fallbacks (`Unknown Commander`). |
| Division by zero when a troop takes 0 casualties | High | Hardcoded guard returning `999.9%` or `100.0%` when `kp_lost == 0`. |
| HUD overlay size overflow with many marches | Low | Cap displayed troops on mini HUD to top 5 most active marches or add scroll/paging. |
