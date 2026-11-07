# In-Game Overlay: Mini HUD & Custom Pinned Counters Design Spec

**Date:** 2026-11-07  
**Status:** In Review  
**Component:** In-Game Overlay (`src/overlay_ui.rs`, `src/overlay_tracker.rs`, `src/model.rs`, `src/gui/views/settings.rs`, `src/gui/lifecycle.rs`)  
**Design Standards:** Anti-Slop (Taste Skill 2.5), UI/UX Pro Max, Zero Emoji, Windows Mouse Pass-Through (`WS_EX_TRANSPARENT`)

---

## 1. Intent & The Why

Rise of Kingdoms players doing barbarian chaining, fort rally grinding, or Motte looting require immediate real-time feedback on their farming efficiency and drop yields without cluttering their game screen.

Currently, the in-game overlay displays a vertical stack of up to 5 comprehensive battle cards (~240px height). While informative, this card stack can obscure strategic battlefield areas (rally target timers, enemy marches, skill cooldowns). 

This project delivers:
1. **Mini HUD (Compact Ribbon / Ticker Mode)**: A minimalist horizontal HUD ribbon (~30–34px height) showing live daily combat counts and custom pinned item totals.
2. **Custom Pinned Counters**: User-selected items (e.g. Books of Covenant, Arrows of Resistance, Gems, Universal Speedups, Gold Keys) tracking **today's rolling total** (UTC+7 / Vietnam time).
3. **Global Hotkey Cycling (Approach 3)**: A system-wide Windows hotkey (`Ctrl + Shift + H`) allowing players to cycle overlay modes (`Cards` → `MiniHud` → `Combined` → `Hidden`) on-the-fly without alt-tabbing or leaving the game window.
4. **Strict Zero-Emoji, Anti-Slop Visual Taste**: Pure typographic badges, high-contrast monospace tabular figures, balanced padding, and semantic RoK palette colors.

---

## 2. Goals, Non-Goals & Anti-Goals

### 2.1 Goals
- **Multi-Mode Support**: Support `Cards`, `MiniHud`, `Combined`, and `Hidden` display modes via an enum persisted in `OverlayConfig`.
- **Today's Rolling Pinned Totals**: Pinned counters reflect all reports accumulated since 00:00:00 VN time (UTC+7) of the current day, matching the main Dashboard KPI calculations.
- **Dynamic Viewport Geometry**: Automatically size the Windows overlay viewport to fit the selected mode (e.g., 32px height for Mini HUD vs. ~230px for Cards).
- **Global Hotkey Handler**: Win32 `RegisterHotKey` listener cycling modes seamlessly and saving user state.
- **Settings UI**: Clean item selector chips, hotkey toggle, and mode selection inside the Settings view.
- **Zero Allocations in Render Loop**: Pre-calculate and memoize today's pinned totals when mail reports arrive, maintaining sub-millisecond 60 FPS viewport renders.

### 2.2 Non-Goals
- **In-Overlay Mouse Interaction**: The overlay window retains `WS_EX_TRANSPARENT` mouse pass-through. Interactive clicks do NOT stop in the overlay, guaranteeing zero interference with in-game troop controls.
- **Database Schema Changes**: No new SQLite tables or vault migrations. All queries leverage existing indexed columns (`time_raw`, `battle_type`, `items_json`).

### 2.3 Anti-Goals (Strict Prohibitions)
- **NO EMOJIS**: Under no circumstances will unicode emojis (e.g. ⚔️, 🏰, 💎, 📚, 🎯, 🔑) be used in the HUD, card view, or settings. All indicators must use typographic labels (`FORT`, `BARB`, `MOTTE`, `GEMS`, `BOOK`, `ARROW`, `KEYS`, `SPEEDUP`).
- **NO JITTER**: Quantities must use monospace tabular figures (`egui::FontFamily::Monospace`) so numbers increment smoothly without shifting adjacent badges.

---

## 3. Approaches Considered & Decision

1. **Approach 1: Multi-Mode Display with Settings-Only Switch**  
   - Modes: `Cards`, `MiniHud`, `Combined`.  
   - *Pros:* Simplest, zero extra background threads.  
   - *Cons:* Player must alt-tab to the main GUI to switch between Mini HUD and detailed cards.
2. **Approach 2: In-Overlay Accordion Toggle Button**  
   - Expand/collapse chevron rendered inside the overlay.  
   - *Rejected:* Breaks Windows `WS_EX_TRANSPARENT` mouse click-through; causes disastrous misclicks during intense game rallies.
3. **Approach 3: Global Hotkey Toggle with Multi-Mode HUD and Daily Totals (Selected)**  
   - Global Windows hotkey (`Ctrl + Shift + H`) cycles modes instantly while in full-screen or windowed RoK.  
   - Full Settings tab configuration with presets and search picker.  
   - *Trades off:* Requires Win32 message registration and thread-safe signaling to the GUI loop.

---

## 4. Architecture & Data Contracts

### 4.1 Data Models (`src/model.rs`)

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

pub struct OverlayConfig {
    pub enabled: bool,
    pub filter: FilterMode,
    pub opacity: f32,
    pub text_color_hex: String,
    pub accent_color_hex: String,
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub font_size: f32,
    // --- New Fields ---
    pub mode: OverlayDisplayMode,
    pub pinned_items: Vec<PinnedItemKey>,
    pub hotkey_enabled: bool,
}
```

**Default Pinned Items**:
- `PinnedItemKey::CatalogItem { itype: 1, isub: 2, canonical_name: "Gems".into() }`
- `PinnedItemKey::CatalogItem { itype: 2, isub: 3, canonical_name: "Book of Covenant".into() }`
- `PinnedItemKey::CatalogItem { itype: 2, isub: 4, canonical_name: "Arrow of Resistance".into() }`
- `PinnedItemKey::SpeedupUniversal`

### 4.2 Structured Memoized Cache (`RoKLootWatcherApp`)

To decouple rendering from computation and solve duration vs discrete item typing:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CachedPinnedChip {
    pub label: String,        // e.g. "GEMS", "BOOK", "ARROW", "SPEEDUP"
    pub display_val: String,  // e.g. "1,450", "32", "14h 30m"
    pub text_color: egui::Color32,
    pub bg_tint: egui::Color32,
}

pub struct RoKLootWatcherApp {
    // ... existing fields ...
    pub cached_today_run_counts: (usize, usize, usize), // (Fort, Motte, Barb)
    pub cached_today_pinned_chips: Vec<CachedPinnedChip>,
}
```

- **Calculation in `lifecycle.rs`**:
  - Filtered by today (Vietnam time `UTC+7`) and constrained to `main_account_uids` (respecting account isolation).
  - **Obeys `overlay_config.filter`**: Aggregates only reports matching the active `FilterMode` (e.g. `FilterMode::Fort` counts only Fort loot, `FilterMode::Barbarian` counts only Barbarian drops, `FilterMode::AllRewards` counts across all types).
  - Changing the report filter in Settings or via hotkey immediately recalculates `cached_today_pinned_chips` and `cached_today_run_counts`.
  - For `PinnedItemKey::SpeedupUniversal`, sums `rec.univ_speedup_mins` and formats as `format_speedup_compact(minutes)` (e.g. `14h 30m`).
  - For `PinnedItemKey::CatalogItem`, sums matching items from `rec.item_totals` and formats as `format_num(total)`.
  - Generates zero allocations during 60 FPS viewport rendering.

### 4.3 Windows Global Hotkey Listener (`src/overlay_tracker.rs` / `src/hotkey.rs`)

- Registered using Win32 API:
  ```rust
  RegisterHotKey(hwnd, HOTKEY_OVERLAY_CYCLE_ID, MOD_CONTROL | MOD_SHIFT, 0x48 /* 'H' */);
  ```
- **State Machine Integration**:
  - When the user presses `Ctrl + Shift + H`:
    - If `mode == Hidden`: wakes up and switches to `Cards`.
    - If `mode == Combined`: cycles to `Hidden` (hiding the viewport window).
    - Other steps: `Cards -> MiniHud -> Combined`.
  - The Win32 listener remains active as long as `hotkey_enabled == true`, even when `mode == Hidden`, ensuring hotkeys always wake the HUD.
  - In Settings:
    - The master toggle `Enable Overlay` controls whether the overlay feature is active at all.
    - If `enabled == false`, hotkey is unregistered and overlay stays closed.
    - Mode selector allows picking `Cards`, `Mini HUD`, or `Combined`.

### 4.4 Viewport Geometry & Width Strategy (`src/overlay_tracker.rs`)

`calculate_overlay_geometry` accounts for `config.mode`:
- **`MiniHud` Mode**:
  - `height = (config.font_size * 2.1).clamp(28.0, 36.0)`
  - `width = config.width.clamp(320.0, 600.0)`
- **`Cards` Mode**:
  - `height = (card_height * 5.0 + 18.0).ceil()` (~230px)
  - `width = config.width.clamp(200.0, 450.0)`
- **`Combined` Mode**:
  - `width = config.width.max(420.0).clamp(420.0, 640.0)` (ensures the Mini HUD ribbon does not wrap while cards remain centered/comfortably padded).
  - `height = mini_hud_height + 6.0 + cards_height` (~270px)
- **`Hidden` Mode**:
  - Returns `OverlayVisibilityAction::Hide`.

---

## 5. Visual Layout & Typography (Anti-Slop / Taste 2.5)

### 5.1 Mini HUD Layout Specifications
Height: `30px` - `34px`. Padding: `4px` top/bottom, `8px` left/right.  
Frame: Dark Slate `#0f172a` at `config.opacity * dim_factor`, 1px stroke `#334155`, rounding `5px`.

```
+---------------------------------------------------------------------------------------------------+
| [FORT 14] [MOTTE 6] [BARB 32] | GEMS 1,450 | BOOKS 28 | ARROWS 84 | SPEEDUP 14h 30m               |
+---------------------------------------------------------------------------------------------------+
```

#### Run Badges (Left Group):
- **`FORT <count>`**: Amber text `#f59e0b`, Amber subtle bg `#f59e0b1f`, 1px border `#f59e0b40`, bold 10pt.
- **`MOTTE <count>`**: Rose text `#f43f5e`, Rose subtle bg `#f43f5e1f`, 1px border `#f43f5e40`, bold 10pt.
- **`BARB <count>`**: Emerald text `#10b981`, Emerald subtle bg `#10b9811f`, 1px border `#10b98140`, bold 10pt.

#### Divider:
- Vertical bar `|` in muted gray `#475569` with `4px` horizontal spacing.

#### Pinned Item Chips (Right Group):
- Monospace tabular figures (`egui::FontFamily::Monospace`) for all numerical counts to ensure zero layout jitter as numbers increment.
- **`GEMS <qty>`**: Cyan text `#06b6d4`.
- **`BOOKS <qty>`**: Amber text `#f59e0b`.
- **`ARROWS <qty>`**: Train-orange text `#fb923c`.
- **`SPEEDUP <formatted>`**: Heal-blue text `#38bdf8`.

---

## 6. Settings UI Specifications (`src/gui/views/settings.rs`)

1. **Mode Switcher**:
   - Segmented buttons: `[ Cards ]  [ Mini HUD ]  [ Combined ]`.
   - Global Hotkey toggle: `Checkbox: Global hotkey toggle (Ctrl + Shift + H)`.
2. **Pinned Items Manager**:
   - Searchable item dropdown (powered by `LootCatalog` with bilingual search support).
   - Preset quick-add buttons:
     - `[+ Castle Push]` (Books of Covenant, Arrows of Resistance, Gems)
     - `[+ War & Training]` (Universal Speedup, Training Speedup, Gems)
     - `[+ Gathering]` (Gems, Level 5 Resource Packs)
   - Active pinned chips with an `[x]` remove button:
     - Maximum 5 pinned items allowed.
