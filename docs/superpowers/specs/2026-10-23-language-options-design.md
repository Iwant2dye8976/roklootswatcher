# Design Document: Bilingual Language Support (Vietnamese & English)

- **Date:** 2026-10-23
- **Topic:** Bilingual Interface & Data Display (Vietnamese / English)
- **Status:** Proposed

---

## 1. Intent & Context
ROK Loot Watcher was originally developed with a Vietnamese user interface, while using some English terminology ("Motte", "Report", "Dashboard"). To make the application accessible to international users and Vietnamese players who prefer playing Rise of Kingdoms in English, the application needs a first-class language toggle supporting both Vietnamese and English.

This change allows users to switch languages instantly at runtime, with their preference saved in `rok_config.json`. The active language dynamically controls not only interface chrome (sidebar, tabs, dialogs, buttons, tooltips, system tray), but also live-translated in-game battle targets (e.g., "Barbarian Level 45" vs "Dã Man Cấp 45"), inventory items (pulling English/Vietnamese names from `loot.yaml`), and exported CSV / JSON reports.

---

## 2. Goals, Non-Goals, Anti-Goals

### Goals
- **Seamless Language Switching:** Provide a clean selector in the Settings tab (and optionally a quick-toggle badge on the header/sidebar) allowing users to toggle between Vietnamese and English without restarting the app.
- **Persistent Preference:** Store the user's choice in `AppConfig` (`rok_config.json`) with backwards compatibility for existing config files.
- **Dynamic Content Localization:**
  - Game targets format dynamically according to the active language (e.g. `Barbarian Level 52` / `Dã Man Cấp 52`, `Barbarian Fort Level 10` / `Pháo Đài Cấp 10`, `Motte 13`).
  - Inventory items display `name_en` when English is active, and `name_vi` when Vietnamese is active.
  - Item categories ("All", "Resources", "Speedups", "Crystals", "Experience", "Equipment", "Others") translate cleanly.
- **Export Consistency:** Exported CSV and JSON files reflect the currently active language for column headers and item names.
- **Zero Overhead & Zero Dependency:** Keep the application as a single self-contained Windows executable with compile-time type safety.

### Non-Goals
- Translating historical raw mail body dumps or raw text content received directly from game network packets.
- Adding arbitrary external translation file editors or community `.po` files.
- Modifying historical `rok_history_vault.json` schema or keys.

### Anti-Goals
- **No Data Loss / Desynchronization:** Changing languages must never rewrite the persistent vault timestamps, coordinates, or unique identifiers.
- **No Runtime Performance Degradation:** Immediate-mode UI rendering (60 FPS) must not perform repetitive string allocation, parsing, or hash-table lookups per frame.

---

## 3. Approaches Considered

### Approach 1: Strongly-Typed Compile-Time Module (`src/i18n.rs`) — *(Selected)*
- **Description:** Define an `AppLanguage` enum (`Vietnamese`, `English`) in `src/model.rs` and create a dedicated `src/i18n.rs` module providing zero-cost lookup functions (`i18n::tr(Key, AppLanguage) -> &'static str`). Add display helpers `record.display_target(lang)` and `item.display_name(lang)`.
- **Trade-offs:** Compile-time verified (missing keys cause compile errors), zero heap allocation per lookup, and keeps the binary 100% portable.
- **Verdict:** Selected by user.

### Approach 2: External Localization Files (JSON / YAML)
- **Description:** Load `locales/en.json` and `locales/vi.json` from disk at startup into dynamic HashMaps.
- **Trade-offs:** Allows modifying strings without recompilation, but adds file I/O on startup, breaks single-file standalone distribution, and adds hash lookup costs per frame.
- **Verdict:** Rejected due to distribution complexity and fragility on portable installations.

---

## 4. Detailed Design & Architecture

### 4.1 Data Models (`src/model.rs`)
1. **`AppLanguage` Enum:**
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "lowercase")]
   pub enum AppLanguage {
       #[serde(rename = "vi")]
       Vietnamese,
       #[serde(rename = "en")]
       English,
   }

   impl Default for AppLanguage {
       fn default() -> Self {
           AppLanguage::Vietnamese
       }
   }
   ```
2. **`AppConfig` Extension:**
   ```rust
   pub struct AppConfig {
       // ... existing fields ...
       #[serde(default)]
       pub language: AppLanguage,
   }
   ```
3. **Display Methods:**
   - `ItemDrop::display_name(&self, lang: AppLanguage) -> &str`: returns `name_en` if English and not empty, else `name_vi`.
   - `BattleRecord::display_target(&self, lang: AppLanguage) -> String`:
     - If `battle_type == "fort"`: returns `"Barbarian Fort Level {level}"` (EN) vs `"Pháo Đài Cấp {level}"` (VI).
     - If `battle_type == "barbarian"`: returns `"Barbarian Level {level}"` (EN) vs `"Dã Man Cấp {level}"` (VI).
     - If `battle_type == "motte"`: returns `"Motte {level}"` (both).
     - Handles chained battle counts `"(x{count})"`.

### 4.2 Localization Dictionary (`src/i18n.rs`)
A dedicated module containing:
- Key enum `TrKey` covering all interface labels, tabs, table headers, filter modes, metric cards, and dialog texts.
- Translation lookup function: `pub fn tr(key: TrKey, lang: AppLanguage) -> &'static str` or `lang.t(key)`.
- Helper for category names: `pub fn category_name(cat: &str, lang: AppLanguage) -> &'static str`.
- Helper for date presets: `pub fn date_preset_name(preset: &DateFilterPreset, lang: AppLanguage) -> &'static str`.
- Helper for filter mode names: `pub fn filter_mode_name(mode: &FilterMode, lang: AppLanguage) -> &'static str`.

### 4.3 GUI Integration (`src/gui.rs`)
- Add `pub language: AppLanguage` field to `RoKLootWatcherApp`.
- In `render_view_settings`: Add a Language selector (Radio buttons or ComboBox for "Tiếng Việt" and "English").
- Replace hardcoded Vietnamese labels with calls to `self.language.t(TrKey::...)`.
- In `render_view_dashboard`:
  - 5 latest reports use `r.display_target(self.language)`.
  - Top 10 items use `it.display_name(self.language)`.
- In `render_view_battles`:
  - Table headers use translated column titles.
  - Rows render targets using `r.display_target(self.language)`.
  - Side inspector displays translated section labels and item names.
- In `render_view_inventory`:
  - Items list uses `it.display_name(self.language)`.
  - Category selector displays translated categories.

### 4.4 Exporters (`src/exporters.rs` & `src/gui.rs`)
- **Canonical JSON Schema Integrity:**
  - Automated background exports (`motte_filtered_mails.json`) and disk snapshots remain strictly canonical (invariant JSON keys: `"statistics"`, `"battles"`, `"battle_type"`, `"motte_level"`, etc.) so recovery routines, tests, and automated tools are never broken.
  - Manual JSON export retains canonical keys while including bilingual item properties (`name_vi`, `name_en`).
- **CSV Localization:**
  - Column headers dynamically switch based on active language:
    - VI: `Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút)`
    - EN: `Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min)`
  - Target names in CSV rows format via `r.display_target(lang)` (e.g. `"Barbarian Level 45"` in EN, `"Dã Man Cấp 45"` in VI).
  - Battle Type column in CSV rows formats via translated labels (e.g. `"Barbarian"`, `"Barbarian Fort"`, `"Motte"` in EN vs `"Dã Man"`, `"Pháo Đài"`, `"Motte"` in VI).

### 4.5 Data Invariance & Storage Guarantee
- `BattleRecord.target` in memory and in `rok_history_vault.json` strictly retains its original canonical value to ensure vault deduping, indexing, and sanitization tests (`r.target.contains("Pass")`, etc.) remain 100% intact.
- All translations for targets (`r.display_target(lang)`) and items (`it.display_name(lang)`) are computed dynamically on demand at the view and export layer.

---

## 5. What's Left to the Builder
- Exact phrasing and polish of minor descriptive tooltips in English.
- Exact styling and icon placement for the language selector control in the Settings view.
