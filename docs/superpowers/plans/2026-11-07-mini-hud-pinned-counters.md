# Mini HUD and Custom Pinned Counters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a sleek, anti-slop, zero-emoji in-game Mini HUD ribbon and custom pinned daily counters for Rise of Kingdoms, with global hotkey cycling (`Ctrl + Shift + H`) between `Cards`, `MiniHud`, `Combined`, and `Hidden` display modes.

**Architecture:** 
- `src/model.rs`: `OverlayDisplayMode`, `PinnedItemKey`, and extended `OverlayConfig`.
- `src/overlay_tracker.rs`: Dynamic geometry calculations per mode and Win32 `RegisterHotKey` listener.
- `src/overlay_ui.rs`: Zero-emoji, typographic Mini HUD ribbon, `CachedPinnedChip` rendering, and Combined viewport layout.
- `src/gui/lifecycle.rs`: Today's filtered pinned totals aggregator obeying `overlay_config.filter` and account isolation.
- `src/gui/views/settings.rs`: Mode selector, hotkey toggle, and bilingual pinned item picker with quick presets.

---

### Task 1: Core Models & Pinned Item Key Serde (`src/model.rs`)

**Files:**
- Modify: `src/model.rs`

- [ ] **Step 1: Define `OverlayDisplayMode` and `PinnedItemKey`**
  In `src/model.rs`:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
  pub enum OverlayDisplayMode {
      #[default]
      Cards,
      MiniHud,
      Combined,
      Hidden,
  }

  impl OverlayDisplayMode {
      pub fn next(&self) -> Self {
          match self {
              Self::Cards => Self::MiniHud,
              Self::MiniHud => Self::Combined,
              Self::Combined => Self::Hidden,
              Self::Hidden => Self::Cards,
          }
      }
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub enum PinnedItemKey {
      CatalogItem { itype: u32, isub: u32, canonical_name: String },
      SpeedupUniversal,
      SpeedupTraining,
      SpeedupHealing,
  }
  ```

- [ ] **Step 2: Extend `OverlayConfig`**
  Add fields with serde default fallbacks:
  ```rust
  #[serde(default = "default_overlay_mode")]
  pub mode: OverlayDisplayMode,
  #[serde(default = "default_overlay_pinned_items")]
  pub pinned_items: Vec<PinnedItemKey>,
  #[serde(default = "default_overlay_hotkey_enabled")]
  pub hotkey_enabled: bool,
  ```
  Provide default constructor returning:
  - `mode: OverlayDisplayMode::Cards`
  - `pinned_items`: Gems (`1_2`), Book of Covenant (`2_3`), Arrow of Resistance (`2_4`), SpeedupUniversal.
  - `hotkey_enabled: true`.

- [ ] **Step 3: Write unit tests in `src/model.rs`**
  Test:
  - Backward compatible deserialization of legacy JSON lacking `mode` or `pinned_items`.
  - Serialization roundtrip for `OverlayConfig`.
  - `OverlayDisplayMode::next()` state transitions.

- [ ] **Step 4: Run tests**
  Run: `cargo test model::tests::test_overlay_config`

- [ ] **Step 5: Commit**
  ```bash
  git add src/model.rs
  git commit -m "feat(model): add OverlayDisplayMode, PinnedItemKey and extend OverlayConfig"
  ```

---

### Task 2: Daily Pinned Aggregation & Caching Engine (`src/overlay_ui.rs`, `src/gui/lifecycle.rs`, `src/gui/mod.rs`)

**Files:**
- Modify: `src/overlay_ui.rs`
- Modify: `src/gui/lifecycle.rs`
- Modify: `src/gui/mod.rs`

- [ ] **Step 1: Define `CachedPinnedChip`**
  In `src/overlay_ui.rs`:
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct CachedPinnedChip {
      pub key: crate::model::PinnedItemKey,
      pub label: String,        // e.g. "GEMS", "BOOKS", "ARROWS", "SPEEDUP"
      pub display_val: String,  // e.g. "1,450", "32", "14h 30m"
      pub text_color: Color32,
  }
  ```

- [ ] **Step 2: Implement aggregation math in `src/gui/lifecycle.rs`**
  Add helper `calculate_today_pinned_chips`:
  - Pulls reports from `watcher.get_all_vault_records()`.
  - Filters by today's date (`time_vn >= today_start_vn`).
  - Obeys `self.overlay_config.filter` (`FilterMode::Fort` only aggregates Fort reports, etc.).
  - Filters by `main_account_uids` if configured.
  - Calculates:
    - Run counts: `(fort_count, motte_count, barb_count)`.
    - Pinned items: sums matching item quantities or speedup minutes (`univ_speedup_mins`, `train_speedup_mins`, `heal_speedup_mins`).
  - Formats duration via compact formatter (e.g. `14h 30m` or `45m`) and item counts with `format_num`.

- [ ] **Step 3: Wire into `refresh_cached_overlay_records` and `RoKLootWatcherApp::new`**
  Store on `RoKLootWatcherApp`:
  - `pub cached_today_run_counts: (usize, usize, usize)`
  - `pub cached_today_pinned_chips: Vec<CachedPinnedChip>`
  Update on every mail event and filter preset change.

- [ ] **Step 4: Write unit tests in `src/gui/tests.rs`**
  Test:
  - Today filter restriction and account UID isolation.
  - Obeys active `FilterMode` (Fort only vs All Rewards).
  - Duration formatting and discrete quantity formatting.

- [ ] **Step 5: Run tests**
  Run: `cargo test gui::tests`

- [ ] **Step 6: Commit**
  ```bash
  git add src/overlay_ui.rs src/gui/lifecycle.rs src/gui/mod.rs src/gui/tests.rs
  git commit -m "feat(gui): implement today's pinned items aggregation and caching"
  ```

---

### Task 3: Dynamic Window Geometry & Windows Global Hotkey (`src/overlay_tracker.rs`, `src/gui/mod.rs`)

**Files:**
- Modify: `src/overlay_tracker.rs`
- Modify: `src/gui/mod.rs`

- [ ] **Step 1: Update `calculate_overlay_geometry` for all modes**
  In `src/overlay_tracker.rs`:
  ```rust
  match config.mode {
      OverlayDisplayMode::MiniHud => {
          let height = (config.font_size * 2.2).clamp(28.0, 36.0);
          let width = config.width.clamp(320.0, 600.0);
          // Clamp pos_x, pos_y within bounds
      }
      OverlayDisplayMode::Cards => {
          let height = (card_height * 5.0 + 18.0).ceil();
          let width = config.width.clamp(200.0, 450.0);
      }
      OverlayDisplayMode::Combined => {
          let mini_h = (config.font_size * 2.2).clamp(28.0, 36.0);
          let cards_h = (card_height * 5.0 + 18.0).ceil();
          let height = mini_h + 6.0 + cards_h;
          let width = config.width.max(420.0).clamp(420.0, 640.0);
      }
      OverlayDisplayMode::Hidden => {
          // Handled via OverlayVisibilityAction::Hide
      }
  }
  ```
  In `determine_overlay_visibility_action`, return `Hide` if `config.mode == OverlayDisplayMode::Hidden`.

- [ ] **Step 2: Implement Win32 Global Hotkey Listener (`Ctrl + Shift + H`)**
  In `src/overlay_tracker.rs` (under `#[cfg(windows)]`):
  - Function `start_overlay_hotkey_listener(rx_action_tx: std::sync::mpsc::Sender<()>) -> HotkeyListenerGuard`.
  - Spawns background thread with a hidden Win32 message window.
  - Calls `RegisterHotKey(hwnd, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT, 0x48 /* 'H' */)`.
  - Dispatches to channel on `WM_HOTKEY`.
  - Unregisters on drop.

- [ ] **Step 3: Wire hotkey receiver into `RoKLootWatcherApp::update`**
  - Check channel in `update`:
    ```rust
    if self.overlay_config.hotkey_enabled {
        while let Ok(()) = self.hotkey_rx.try_recv() {
            self.overlay_config.mode = self.overlay_config.mode.next();
            self.save_config();
            ctx.request_repaint();
        }
    }
    ```

- [ ] **Step 4: Write unit tests in `src/overlay_tracker.rs`**
  Test geometry output and action determination for all 4 display modes.

- [ ] **Step 5: Run tests**
  Run: `cargo test overlay_tracker::tests`

- [ ] **Step 6: Commit**
  ```bash
  git add src/overlay_tracker.rs src/gui/mod.rs
  git commit -m "feat(overlay): add multi-mode dynamic geometry and Win32 global hotkey listener"
  ```

---

### Task 4: Zero-Emoji Mini HUD & Combined Mode Viewport Renderer (`src/overlay_ui.rs`)

**Files:**
- Modify: `src/overlay_ui.rs`
- Modify: `src/gui/mod.rs`

- [ ] **Step 1: Implement `render_mini_hud_ribbon`**
  In `src/overlay_ui.rs`:
  - Outer frame: Dark Slate `#0f172a` at `config.opacity * dim_factor`, rounding 5.0, 1px stroke `#334155`.
  - Left Run Badges:
    - `[FORT <count>]` (Amber `#f59e0b` text, faint amber bg tint, bold 10pt)
    - `[MOTTE <count>]` (Rose `#f43f5e` text, faint rose bg tint, bold 10pt)
    - `[BARB <count>]` (Emerald `#10b981` text, faint emerald bg tint, bold 10pt)
  - Divider: vertical `|` separator in muted gray `#475569`.
  - Right Pinned Chips:
    - Render each `CachedPinnedChip`:
      - Label in semantic color (Cyan for Gems, Amber for Books, Orange for Arrows, Blue for Speedup).
      - Value in monospace tabular figures (`egui::FontFamily::Monospace`).
  - Strict Anti-Slop verification: **ZERO EMOJIS**.

- [ ] **Step 2: Update `render_overlay_viewport` for mode dispatch**
  - If `OverlayDisplayMode::MiniHud`: calls `render_mini_hud_ribbon`.
  - If `OverlayDisplayMode::Cards`: renders existing 5 battle cards.
  - If `OverlayDisplayMode::Combined`: renders `render_mini_hud_ribbon`, adds 4px space, then renders 5 battle cards.

- [ ] **Step 3: Update `src/gui/mod.rs` overlay invocation**
  Pass `cached_today_run_counts` and `cached_today_pinned_chips` to `render_overlay_viewport`.

- [ ] **Step 4: Write unit tests in `src/overlay_ui.rs`**
  Test:
  - Mini HUD ribbon text generation contains zero emojis.
  - Tabular font family applied to display values.
  - Empty cards fallback vs Mini HUD ribbon rendering.

- [ ] **Step 5: Run tests**
  Run: `cargo test overlay_ui::tests`

- [ ] **Step 6: Commit**
  ```bash
  git add src/overlay_ui.rs src/gui/mod.rs
  git commit -m "feat(overlay): render typographic zero-emoji mini hud ribbon and combined mode"
  ```

---

### Task 5: Settings UI - Mode Switcher, Pinned Item Selector & Presets (`src/gui/views/settings.rs`)

**Files:**
- Modify: `src/gui/views/settings.rs`
- Modify: `src/gui/tests.rs`

- [ ] **Step 1: Add Display Mode Selector & Hotkey Toggle in Settings**
  In `src/gui/views/settings.rs`:
  - Segmented buttons for `[ Cards ]`, `[ Mini HUD ]`, `[ Combined ]`.
  - Checkbox: `Global Hotkey (Ctrl + Shift + H)` with status label.

- [ ] **Step 2: Add Pinned Items Manager UI**
  - Render active pinned chips with `[x]` remove button (maximum 5 items).
  - Quick-preset chips:
    - `[+ Castle Push]` (Books, Arrows, Gems)
    - `[+ War & Training]` (Universal Speedup, Training Speedup, Gems)
    - `[+ Gathering]` (Gems, Level 5 Resource Packs)
  - Searchable ComboBox / dropdown using `LootCatalog` items with bilingual support.

- [ ] **Step 3: Write unit tests in `src/gui/tests.rs`**
  Test:
  - Preset click populates correct `PinnedItemKey` items up to 5 max.
  - Removing a pinned chip updates `OverlayConfig`.
  - Mode switcher updates `OverlayConfig.mode` and triggers config save.

- [ ] **Step 4: Run tests**
  Run: `cargo test gui::tests`

- [ ] **Step 5: Commit**
  ```bash
  git add src/gui/views/settings.rs src/gui/tests.rs
  git commit -m "feat(settings): add overlay mode switcher, hotkey toggle, and pinned item selector"
  ```

---

### Task 6: Verification, Clippy Clean & Version Bump

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Run clippy check**
  Run: `cargo clippy -- -D warnings`
  Fix any warnings.

- [ ] **Step 2: Run all unit tests**
  Run: `cargo test --bin rok_watcher`
  Confirm all tests pass.

- [ ] **Step 3: Run integration tests**
  Run: `cargo test --test per_record_sync_integration -- --test-threads=1`
  Confirm all integration tests pass.

- [ ] **Step 4: Bump minor version in `Cargo.toml`**
  Increment version from `2.2.2` to `2.3.0` (new feature per `AGENTS.md`).
  Run: `cargo check` to update `Cargo.lock`.

- [ ] **Step 5: Commit**
  ```bash
  git add Cargo.toml Cargo.lock
  git commit -m "chore: bump version to 2.3.0 after mini hud and pinned counters implementation"
  ```
