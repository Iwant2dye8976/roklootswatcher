# Gathering Reports Watcher & Tracker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real-time and offline watcher capabilities for Rise of Kingdoms gathering reports (`type: "Rss"`), refactor `motte_level` to a unified backward-compatible `target_level`, and display gathering yields seamlessly in the reports table and item vault.

**Architecture:** Detect `type: "Rss"` mails in `classifier.rs` and extract total resource yields (`ResValue + talentAdd`) mapped to `loot.yaml` standard IDs `(1, 1..=5)`. Refactor `motte_level` to `target_level` with Serde aliases across `model.rs`, `vault.rs`, `analyzer.rs`, and `gui.rs` to maintain 100% backward compatibility with existing vault records. Add `FilterMode::Gathering` with dedicated table layout branching in `gui.rs` and real-time watcher ingestion in `watcher.rs`.

**Tech Stack:** Rust 2021 edition, `eframe` / `egui 0.29`, `serde` / `serde_json`, `notify` 6.1, `indexmap` 2.6.

**Spec:** [docs/superpowers/specs/2026-10-23-gathering-watcher-design.md](file:///C:/roklootswatcher/docs/superpowers/specs/2026-10-23-gathering-watcher-design.md)

## Global Constraints

- Windows environment with Rust 2021 edition.
- Mail files are encoded in Lilith's binary stream format (`lilith_stream.rs`).
- Must maintain 100% backward compatibility with existing records in `rok_history_vault.json` using `#[serde(alias = "motte_level", alias = "node_level", default)]`.
- Map coordinates `pos_x` and `pos_y` must be set to 0 for gathering reports.
- KvK Crystals (`ResCollectCrystal`) must be ignored.
- Gathered resources must align with `loot.yaml` canonical names: "Thực phẩm", "Gỗ", "Đá", "Vàng", "Đá quý".

## Review Focus

1. **Missing or float `talentAdd`**: Malformed or float bonus values must round properly to `i64` without panic; test pins this in Task 3.
2. **Unknown `ResType`**: Any unexpected `ResType` must gracefully fallback to `"Mỏ Thu Thập Cấp {Level}"` and `"Tài nguyên khác"` without crashing; test pins this in Task 3.
3. **Legacy Vault JSON without `target_level`**: Old JSON records containing only `"motte_level"` (or neither) must deserialize smoothly into `target_level`; test pins this in Task 1.
4. **Malformed RSS body**: RSS mail where `body.content` is null/missing must return an `Err` without panicking; test pins this in Task 3.
5. **Combined `AllRewards` table rendering**: In `FilterMode::AllRewards`, gathering records with 0 damage and 0 speedups must display without breaking column alignments; verified in Task 5.

---

### Task 1: Refactor `motte_level` to `target_level` with Serde Aliases & Add `FilterMode::Gathering` in `src/model.rs`

**Files:**
- Modify: `src/model.rs:34-94`, `src/model.rs:294-315`
- Test: `src/model.rs` (inline unit tests)

**Interfaces:**
- Consumes: `serde::{Serialize, Deserialize}`
- Produces: `BattleRecord.target_level`, `BattleRecord.talent_add`, `FilterMode::Gathering`, `is_zero(&i64) -> bool`

- [ ] **Step 1: Write unit tests in `src/model.rs` for backward-compatible deserialization and `FilterMode::Gathering`**

```rust
#[test]
fn test_battle_record_target_level_serde_backward_compat() {
    let legacy_json = r#"{
        "filename": "test.mail",
        "mail_id": "12345",
        "battle_type": "motte",
        "time_raw": 1000,
        "time_utc": "2026-10-23 00:00:00 UTC",
        "time_vn": "2026-10-23 07:00:00",
        "target": "Motte 25",
        "motte_level": "25",
        "pos_x": 100,
        "pos_y": 200,
        "damage_pct": 100,
        "plunder_tier": 1,
        "items": []
    }"#;
    let rec: BattleRecord = serde_json::from_str(legacy_json).expect("Must deserialize legacy motte_level");
    assert_eq!(rec.target_level, "25");
    assert_eq!(rec.talent_add, 0);
}

#[test]
fn test_filter_mode_gathering() {
    assert_eq!(FilterMode::from_str_loose("gathering"), FilterMode::Gathering);
    assert_eq!(FilterMode::from_str_loose("gather"), FilterMode::Gathering);
    assert_eq!(FilterMode::from_str_loose("rss"), FilterMode::Gathering);
    assert_eq!(FilterMode::Gathering.as_str(), "gathering");
    assert!(FilterMode::Gathering.matches("gathering"));
    assert!(!FilterMode::Gathering.matches("motte"));
    assert!(!FilterMode::AllTargets.matches("gathering"));
    assert!(FilterMode::AllRewards.matches("gathering"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --bin rok_watcher test_battle_record_target_level_serde_backward_compat`
Expected: FAIL due to missing `target_level` or `FilterMode::Gathering`

- [ ] **Step 3: Implement `target_level`, `talent_add`, and `FilterMode::Gathering` in `src/model.rs`**

- In `FilterMode`:
  - Add variant `Gathering`.
  - In `as_str()`: return `"gathering"`.
  - In `display_label()`: return `"Chỉ Thu Thập (Gathering)"`.
  - In `from_str_loose()`: match `"gathering" | "gather" | "rss"` -> `FilterMode::Gathering`.
  - In `matches()`: `AllTargets` matches `"motte" | "fort" | "barbarian"`, `Gathering` matches `"gathering"`, `AllRewards` matches `true`.
- Add helper function:
  ```rust
  pub fn is_zero(v: &i64) -> bool { *v == 0 }
  ```
- In `BattleRecord`:
  - Replace `pub motte_level: String` with:
    ```rust
    #[serde(alias = "motte_level", alias = "node_level", default)]
    pub target_level: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub talent_add: i64,
    ```
- Update any internal test constructors in `src/model.rs` that passed `motte_level:` to pass `target_level:`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --bin rok_watcher test_battle_record_target_level_serde_backward_compat test_filter_mode_gathering`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(model): refactor motte_level to target_level with serde alias and add FilterMode::Gathering"
```

---

### Task 2: Update References from `motte_level` to `target_level` in `src/vault.rs` & `src/analyzer.rs`

**Files:**
- Modify: `src/vault.rs:185-225`, `src/vault.rs:520-560`, `src/vault.rs:610-760`
- Modify: `src/analyzer.rs:345-410`
- Test: `cargo test --bin rok_watcher`

**Interfaces:**
- Consumes: `BattleRecord.target_level` from `src/model.rs`
- Produces: Clean compilation and passing tests across vault migration and analyzer

- [ ] **Step 1: Check existing compiler errors after Task 1**

Run: `cargo check`
Expected: Errors pointing to `r.motte_level` usages in `vault.rs` and `analyzer.rs`

- [ ] **Step 2: Update `motte_level` references in `src/vault.rs`**

- In `sanitize_barbarian_records`: change `r.motte_level` to `r.target_level`.
- In `extract_records_from_folder`: change `motte_level` identifier to `target_level`.
- In test helpers within `vault.rs`: update struct literals from `motte_level: ...` to `target_level: ...`.
- In `test_strip_level_60_from_barbarians_in_vault`: change `b.get("motte_level")` check to `b.get("target_level").or_else(|| b.get("motte_level"))` and assert on `r.target_level`.

- [ ] **Step 3: Update `motte_level` references in `src/analyzer.rs`**

- In `src/analyzer.rs`: update any mock `BattleRecord` constructors from `motte_level:` to `target_level:`.

- [ ] **Step 4: Run vault and analyzer tests to verify they pass**

Run: `cargo test --bin rok_watcher vault analyzer`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/vault.rs src/analyzer.rs
git commit -m "refactor(vault,analyzer): update motte_level references to target_level"
```

---

### Task 3: Implement Gathering Classifier & Details Extractor in `src/classifier.rs`

**Files:**
- Modify: `src/classifier.rs:85-186`, `src/classifier.rs:300-340`, `src/classifier.rs:520-683`
- Test: `src/classifier.rs` (unit tests against `mail_vault/Persistent.Mail.1002220217856803764`)

**Interfaces:**
- Consumes: `LilithValue`, `BattleRecord`, `ItemDrop`, `format_timestamp`
- Produces: `classify_mail(mail: &LilithValue) -> Option<&'static str>` returning `Some("gathering")`, `extract_gathering_details(mail: &LilithValue, filename: &str) -> Result<BattleRecord, String>`

- [ ] **Step 1: Write unit tests in `src/classifier.rs` for gathering mail classification and extraction**

```rust
#[test]
fn test_classify_and_extract_gathering_mail() {
    let path = "mail_vault/Persistent.Mail.1002220217856803764";
    let mail = crate::lilith_stream::decode_mail_file(path).expect("Must decode sample RSS mail");
    assert_eq!(classify_mail(&mail), Some("gathering"));

    let rec = extract_gathering_details(&mail, "Persistent.Mail.1002220217856803764")
        .expect("Must extract gathering details");
    assert_eq!(rec.battle_type, "gathering");
    assert_eq!(rec.target, "Mỏ Đá Cấp 6");
    assert_eq!(rec.target_level, "6");
    assert_eq!(rec.pos_x, 0);
    assert_eq!(rec.pos_y, 0);
    assert_eq!(rec.talent_add, 4071);
    assert_eq!(rec.items.len(), 1);
    assert_eq!(rec.items[0].itype, 1);
    assert_eq!(rec.items[0].sub_type, 3);
    assert_eq!(rec.items[0].name_vi, "Đá");
    assert_eq!(rec.items[0].value, 27533); // 23461.9 + 4071.0 rounded
    assert_eq!(rec.item_totals.get("Đá"), Some(&27533));
}

#[test]
fn test_extract_gathering_unknown_res_type_fallback() {
    use indexmap::IndexMap;
    let mut content = IndexMap::new();
    content.insert("ResType".to_string(), LilithValue::U8(99));
    content.insert("Level".to_string(), LilithValue::U8(4));
    content.insert("ResValue".to_string(), LilithValue::Number(50000.0));
    content.insert("talentAdd".to_string(), LilithValue::Number(500.0));

    let mut body = IndexMap::new();
    body.insert("content".to_string(), LilithValue::Table(content));

    let mut mail_map = IndexMap::new();
    mail_map.insert("box".to_string(), LilithValue::String("Report".to_string()));
    mail_map.insert("type".to_string(), LilithValue::String("Rss".to_string()));
    mail_map.insert("time".to_string(), LilithValue::Number(1700000000.0));
    mail_map.insert("id".to_string(), LilithValue::String("999".to_string()));
    mail_map.insert("body".to_string(), LilithValue::Table(body));

    let mail = LilithValue::Table(mail_map);
    let rec = extract_gathering_details(&mail, "mock.mail").expect("Should extract with fallback");
    assert_eq!(rec.target, "Mỏ Thu Thập Cấp 4");
    assert_eq!(rec.items[0].name_vi, "Tài nguyên khác");
    assert_eq!(rec.items[0].value, 50500);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --bin rok_watcher test_classify_and_extract_gathering_mail`
Expected: FAIL due to missing function `extract_gathering_details`

- [ ] **Step 3: Implement `classify_mail` branch and `extract_gathering_details` in `src/classifier.rs`**

- In `classify_mail(mail: &LilithValue)`:
  - If `box_name != "Report" && box_name != "Archive" { return None; }`
  - Before checking fast paths, if `mail.get("type").and_then(|v| v.as_str()) == Some("Rss")`, return `Some("gathering")`.
- Implement `extract_gathering_details(mail: &LilithValue, filename: &str) -> Result<BattleRecord, String>`:
  - Extract `mail_id` and timestamp via `format_timestamp(raw_time)`.
  - Extract `body.content` table; if missing return error.
  - Read `res_type = content.get("ResType").and_then(|v| v.as_u32()).unwrap_or(0)`.
  - Read `level = content.get("Level").and_then(|v| v.as_u32()).unwrap_or(0)`.
  - Read `res_value = content.get("ResValue").and_then(|v| v.as_f64()).unwrap_or(0.0)`.
  - Read `talent_add = content.get("talentAdd").and_then(|v| v.as_f64()).unwrap_or(0.0)`.
  - Map `res_type` to `(res_name, category)`:
    - `1` => ("Thực phẩm", "resource")
    - `2` => ("Gỗ", "resource")
    - `3` => ("Đá", "resource")
    - `4` => ("Vàng", "resource")
    - `5` => ("Đá quý", "gems")
    - `_` => ("Tài nguyên khác", "resource")
  - Target string: if `res_type >= 1 && res_type <= 5 { format!("Mỏ {} Cấp {}", res_name, level) } else { format!("Mỏ Thu Thập Cấp {}", level) }`.
  - Compute `base_val = res_value.round() as i64`, `talent_val = talent_add.round() as i64`, `total_val = base_val + talent_val`.
  - Construct `ItemDrop`:
    ```rust
    ItemDrop {
        itype: 1,
        sub_type: res_type,
        value: total_val,
        name_vi: res_name.to_string(),
        category: category.to_string(),
    }
    ```
  - Put into `items`, tally `item_totals.insert(res_name.to_string(), total_val)`.
  - Build and return `BattleRecord` with:
    - `battle_type: "gathering".to_string()`
    - `target`
    - `target_level: if level > 0 { level.to_string() } else { String::new() }`
    - `talent_add: talent_val`
    - `pos_x: 0, pos_y: 0, damage_pct: 0, plunder_tier: 0`
    - `items, item_totals`
    - `content: String::new()`

- Update `extract_motte_details`: rename internal field `motte_level:` to `target_level:`.

- [ ] **Step 4: Run unit tests to verify they pass**

Run: `cargo test --bin rok_watcher test_classify_and_extract_gathering_mail test_extract_gathering_unknown_res_type_fallback`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/classifier.rs
git commit -m "feat(classifier): implement gathering mail classification and extraction"
```

---

### Task 4: Integrate Gathering Ingestion in `src/watcher.rs`

**Files:**
- Modify: `src/watcher.rs:310-380`
- Test: `src/watcher.rs` (unit tests / integration test)

**Interfaces:**
- Consumes: `classifier::classify_mail`, `classifier::extract_gathering_details`, `vault::VaultState`
- Produces: Watcher ingestion and notification for gathering reports

- [ ] **Step 1: Write unit test in `src/watcher.rs` for processing gathering mail**

```rust
#[test]
fn test_watcher_process_gathering_mail() {
    let path = "mail_vault/Persistent.Mail.1002220217856803764";
    let mail = crate::lilith_stream::decode_mail_file(path).expect("Decode mail");
    let btype = crate::classifier::classify_mail(&mail);
    assert_eq!(btype, Some("gathering"));

    let rec = crate::classifier::extract_gathering_details(&mail, "Persistent.Mail.1002220217856803764")
        .expect("Extract gathering");
    assert_eq!(rec.battle_type, "gathering");
    assert_eq!(rec.target_level, "6");
}
```

- [ ] **Step 2: Update mail processing loop in `src/watcher.rs`**

- In `process_mail_file` / `process_mail_bytes`:
  - When `classify_mail(&mail)` returns `Some("gathering")`:
    - Call `extract_gathering_details(&mail, filename)`.
    - If valid, format notification text:
      ```rust
      let res_name = rec.items.first().map(|i| i.name_vi.as_str()).unwrap_or("Tài nguyên");
      let total_val = rec.items.first().map(|i| i.value).unwrap_or(0);
      let notif_body = format!("Thu thập hoàn tất: +{} {}", crate::model::format_num(total_val), res_name);
      ```
    - Push record into active records and notify UI refresh channel.
    - If desktop notification enabled, send notification with title `"Rise of Kingdoms - Thu Thập"` and body `notif_body`.

- [ ] **Step 3: Run watcher unit tests**

Run: `cargo test --bin rok_watcher test_watcher_process_gathering_mail`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/watcher.rs
git commit -m "feat(watcher): integrate gathering mail ingestion and notifications"
```

---

### Task 5: GUI Presentation - Filter Mode, Table Layout & Details Modal in `src/gui.rs`

**Files:**
- Modify: `src/gui.rs:840-865`, `src/gui.rs:1335-1630`, `src/gui.rs:2400-2550`
- Test: `cargo check`, `cargo test --bin rok_watcher`

**Interfaces:**
- Consumes: `FilterMode::Gathering`, `BattleRecord.target_level`, `BattleRecord.talent_add`
- Produces: Updated Filter combo box, `is_gathering` layout branch in Reports table, and gathering Details modal

- [ ] **Step 1: Check compiler status in `gui.rs`**

Run: `cargo check`
Expected: Check if any references to `motte_level` remain in `gui.rs` and identify exact line numbers.

- [ ] **Step 2: Add `FilterMode::Gathering` to filter combo box in `src/gui.rs`**

- In the filter mode combo box (around line 850):
  ```rust
  ui.selectable_value(&mut self.selected_mode, FilterMode::Gathering, FilterMode::Gathering.display_label());
  ```

- [ ] **Step 3: Implement `is_gathering` Reports Table Column Layout in `src/gui.rs`**

- Around line 1340:
  ```rust
  let is_barb = self.selected_mode == FilterMode::Barbarian;
  let is_gathering = self.selected_mode == FilterMode::Gathering;
  ```
- Define gathering-specific column widths:
  - `COL_TIME`: 116.0
  - `COL_TYPE`: 64.0 (for badge "THU THẬP")
  - `COL_GATHER_LVL`: 60.0 (Cấp mỏ)
  - `COL_GATHER_RES_TYPE`: 90.0 (Tài nguyên)
  - `COL_GATHER_TOTAL`: 110.0 (Sản lượng)
  - `COL_GATHER_BONUS`: 100.0 (Điểm cộng)
  - `COL_VIEW`: 44.0
- When `is_gathering`:
  - Render headers: "Thời gian", "Loại", "Mục tiêu", "Cấp mỏ", "Tài nguyên", "Sản lượng", "Điểm cộng", "Chi tiết".
  - Render rows:
    - Time: `r.time_vn`
    - Type: badge `"THU THẬP"` (color teal/accent)
    - Target: `r.target`
    - Cấp mỏ: `r.target_level`
    - Tài nguyên: first item name (`r.items[0].name_vi`)
    - Sản lượng: `format_num(r.items[0].value)` (color gold)
    - Điểm cộng: if `r.talent_add > 0 { format!("+{}", format_num(r.talent_add)) } else { "-".into() }`
    - View button: opens details modal.
- In shared table modes (`AllRewards`):
  - Ensure `r.target_level` is used in any level displays, and gathering rows show total resource value under the resource column cleanly.

- [ ] **Step 4: Update Details Modal for Gathering Records in `src/gui.rs`**

- In the record details modal:
  - If `r.battle_type == "gathering"`:
    - Title: `"Chi tiết Báo cáo Thu Thập"`
    - Show cards:
      - Mục tiêu & Cấp mỏ: `r.target` (Cấp `r.target_level`)
      - Sản lượng gốc: `format_num(total - r.talent_add)`
      - Thưởng tài năng / công nghệ: `format_num(r.talent_add)`
      - Tổng tài nguyên nhận được: `format_num(total)`
    - Omit combat-only cards (damage %, plunder tier, barbarian kill XP).

- [ ] **Step 5: Update CLI support in `src/main.rs` for `--mode gathering`**

- In `main.rs`: ensure `--mode gathering` parses correctly via `FilterMode::from_str_loose` and filters records accordingly.

- [ ] **Step 6: Run full test suite and compilation check**

Run: `cargo test --bin rok_watcher`
Expected: ALL tests pass with 0 failures

- [ ] **Step 7: Commit**

```bash
git add src/gui.rs src/main.rs
git commit -m "feat(gui): add gathering filter mode, dedicated table layout, and details modal"
```
