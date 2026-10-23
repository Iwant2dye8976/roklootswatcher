# Design Document: Gathering Reports Watcher & Tracker

**Date:** 2026-10-23  
**Status:** In Review  
**Topic:** Gathering Reports (`type: "Rss"`) Watcher & Unified Target Level  

---

## 1. Intent and The Why

The application currently monitors and processes Rise of Kingdoms combat and rally reports (`motte`, `fort`, `barbarian`, and general loot rewards), extracting item drops, speedups, and tokens into a unified dashboard and vault.

Players also frequently send gathering marches to resource tiles (Food, Wood, Stone, Gold, Gems). The game records completed gathering marches as mail reports of `type: "Rss"`. Adding gathering watcher capabilities enables players to automatically track their gathering yields, review gathering history, and see total accumulated resources over time alongside their combat activity.

---

## 2. Goals, Non-Goals, and Anti-Goals

### Goals
- **Real-Time & Batch Detection**: Detect and classify incoming gathering reports (`type: "Rss"` in `box == "Report"`) both in live watcher mode and offline vault scans.
- **Resource Yield Extraction**: Accurately compute total resource yields combining base yield (`ResValue`) and talent/tech bonuses (`talentAdd`), mapping `ResType` to standard resource names matching `loot.yaml`:
  - `1`: "Thực phẩm" (Food)
  - `2`: "Gỗ" (Wood)
  - `3`: "Đá" (Stone)
  - `4`: "Vàng" (Gold)
  - `5`: "Đá quý" (Gems)
- **Unified Level Architecture**: Refactor `motte_level` on `BattleRecord` to `target_level` with backward-compatible Serde aliases (`#[serde(alias = "motte_level", alias = "node_level")]`). This cleans up historical technical debt across Motte, Fort, Barbarian, and Gathering records.
- **Identical Pipeline Integration**: Store gathering records directly in `rok_history_vault.json` and display them seamlessly in the "Trận đánh" (Reports) table, item tally, and CLI exports.
- **Filtering**: Add a dedicated `FilterMode::Gathering` ("Chỉ Thu Thập") while preserving existing combat filter behaviors.

### Non-Goals
- **Map Coordinates**: Do not extract or display tile map coordinates (`pos_x` and `pos_y` set to 0 per user specification).
- **KvK Crystals**: Do not extract crystal gathering amounts (`ResCollectCrystal` is explicitly excluded per user specification).
- **Separate Standalone Dashboard**: No separate multi-tab dashboard or complex march scheduler; gathering integrates into the existing report and loot views.

### Anti-Goals
- **Breaking Historical Vault Data**: Must NOT break backward compatibility with existing `rok_history_vault.json` records that currently use `"motte_level"`.
- **Polluting Combat Totals**: `FilterMode::AllTargets` must continue to represent combat/rally targets (`motte`, `fort`, `barbarian`), while `FilterMode::Gathering` isolates gathering, and `FilterMode::AllRewards` covers all reports.

---

## 3. Constraints

- **Language & Runtime**: Rust 2021 edition, `eframe` / `egui 0.29`, `serde` / `serde_json`.
- **Binary Parsing**: Mail files are encoded in Lilith's binary stream format (`lilith_stream.rs`).
- **File System**: Live directory watching via `notify` crate on `%PROGRAMFILES(X86)%\Rise of Kingdoms\Rise of Kingdoms Game\save\mailcache`.

---

## 4. Approaches Considered & Selected

### Approach A: Dedicated `node_level` Field
Add `pub node_level: String` to `BattleRecord`, leaving `motte_level` empty for gathering reports and populated for combat reports.
- *Trade-off*: Leaves existing fields untouched, but permanently keeps two split level concepts across the codebase, complicating table rendering, sorting, and exports.
- *Status*: Rejected.

### Approach B: Unified `target_level` with Backward-Compatible Serde Aliases (Selected)
Rename `motte_level` to `target_level` on `BattleRecord`, with `#[serde(alias = "motte_level", alias = "node_level")]`. Old files containing `"motte_level"` deserialize seamlessly into `target_level`.
- *Trade-off*: Touches references across `classifier.rs`, `vault.rs`, `gui.rs`, and tests, but cleanly unifies level handling for Motte, Fort, Barbarian, and Gathering.
- *Status*: **Selected by user**.

---

## 5. Architectural & Implementation Details

### 5.1 Data Model (`src/model.rs`)
1. **`BattleRecord` Refactoring & Extension**:
   ```rust
   pub struct BattleRecord {
       pub filename: String,
       pub mail_id: String,
       pub battle_type: String, // "gathering", "motte", "fort", "barbarian", "reward_other"
       pub time_raw: i64,
       pub time_utc: String,
       pub time_vn: String,
       pub target: String,      // e.g. "Mỏ Thực phẩm Cấp 6"
       #[serde(alias = "motte_level", alias = "node_level", default)]
       pub target_level: String,// e.g. "6", "5", "47-52"
       #[serde(default, skip_serializing_if = "crate::model::is_zero")]
       pub talent_add: i64,     // Bonus talent/tech yield for gathering details
       pub pos_x: i32,          // 0 for gathering
       pub pos_y: i32,          // 0 for gathering
       pub damage_pct: i32,     // 0 for gathering
       pub plunder_tier: i32,   // 0 for gathering
       pub items: Vec<ItemDrop>,
       #[serde(default, skip_serializing_if = "HashMap::is_empty")]
       pub item_totals: HashMap<String, i64>,
       #[serde(default, skip_serializing_if = "String::is_empty")]
       pub content: String,
   }
   ```
2. **`FilterMode` Updates**:
   - Add `FilterMode::Gathering`:
     - Serde representation: `"gathering"`.
     - UI display label: `"Chỉ Thu Thập (Gathering)"`.
     - Matching logic:
       - `FilterMode::AllTargets`: `matches!(battle_type, "motte" | "fort" | "barbarian")`
       - `FilterMode::Gathering`: `battle_type == "gathering"`
       - `FilterMode::AllRewards`: `true` (includes combat and gathering)

### 5.2 Classification & Extraction (`src/classifier.rs`)
1. **Classifier**:
   - In `classify_mail(mail: &LilithValue)`:
     - Check `box == "Report" || box == "Archive"`.
     - If `mail.get("type").and_then(|v| v.as_str()) == Some("Rss")`, return `Some("gathering")`.
2. **Extraction (`extract_gathering_details`)**:
   - Extract `body.content`:
     - `ResType`: mapped using standard IDs `(1, ResType)` matching `loot.yaml`:
       - `1`: "Thực phẩm" (Food)
       - `2`: "Gỗ" (Wood)
       - `3`: "Đá" (Stone)
       - `4`: "Vàng" (Gold)
       - `5`: "Đá quý" (Gems)
     - `Level`: node level formatted as string into `target_level` (e.g. `"6"`).
     - `ResValue` (f64) and `talentAdd` (f64):
       - `base_val = res_value.round() as i64`
       - `talent_val = talent_add.round() as i64`
       - `total_val = base_val + talent_val`
     - Construct target string: `"Mỏ {ResName} Cấp {Level}"` (e.g. "Mỏ Thực phẩm Cấp 6", "Mỏ Đá quý Cấp 2").
     - Populate `talent_add`: `talent_val`.
     - Populate `ItemDrop`:
       - `itype`: `1`
       - `sub_type`: `ResType`
       - `value`: `total_val`
       - `name_vi`: Resource name (matching `loot.yaml`)
       - `category`: `"resource"` (or `"gems"` if ResType == 5)
     - Add to `item_totals`.
     - Attachments check: if `mail.get("attachments")` contains extra drop items (e.g. event boxes), extract them as well.
     - Skip `ResCollectCrystal`, set `pos_x = 0`, `pos_y = 0`.

### 5.3 Storage & Migration (`src/vault.rs`)
- Replace internal references to `motte_level` with `target_level`.
- Backward compatibility:
  - Reading old JSON: Serde alias `#[serde(alias = "motte_level")]` automatically assigns the value to `target_level`.
  - Level cleanup logic in `vault.rs` operates on `target_level`.

### 5.4 UI Presentation & Table Layout (`src/gui.rs`)
1. **Filter Dropdown**:
   - Include `FilterMode::Gathering` ("Chỉ Thu Thập").
2. **Reports Table Layout**:
   - When `self.selected_mode == FilterMode::Gathering`:
     - Columns: Thời gian (Time) | Loại (Type) | Mục tiêu (Target) | Cấp mỏ (Level) | Loại tài nguyên (Resource Type) | Sản lượng (Total Yield) | Điểm cộng (Talent Bonus) | Chi tiết (View).
   - When in shared modes (`AllRewards`):
     - Displays rows with total resources listed under the Resource column, showing the standard target name.
3. **Details Modal**:
   - Displays Node Level, Base Harvested Amount, Talent Bonus, and Total Resources.
4. **Item Vault ("Kho vật phẩm")**:
   - Gathered resources use catalog `(1, 1..=5)`, aggregating directly under "Thực phẩm", "Gỗ", "Đá", "Vàng", and "Đá quý".

### 5.5 Watcher & Notifications (`src/watcher.rs`)
- The real-time directory watcher catches new incoming RSS report files, parses them through `extract_gathering_details`, saves them to the active vault, and dispatches a notification: `"Thu thập hoàn tất: +{amount} {res_name}"`.

---

## 6. Resolved Reviewer & Builder Calls

1. **Talent Bonus Storage**: Added `pub talent_add: i64` with `#[serde(default, skip_serializing_if = "crate::model::is_zero")]` on `BattleRecord` so the Details window can display base yield vs bonus talent without altering existing records or adding schema clutter.
2. **Catalog IDs & Names**: Standardized on `loot.yaml` IDs `(1, 1..=5)` and canonical names (`"Thực phẩm"`, `"Gỗ"`, `"Đá"`, `"Vàng"`, `"Đá quý"`). This ensures seamless aggregation in the Item Vault and correct KPI computations.
3. **Table Column Layout**: Introduced an `is_gathering` layout branch in `gui.rs` for `FilterMode::Gathering` to cleanly show node level, resource type, total amount, and talent bonus, avoiding empty column clutter.
