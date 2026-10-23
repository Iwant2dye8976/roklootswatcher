# Bilingual Language Support (Vietnamese & English) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a seamless, zero-cost bilingual localization system (Vietnamese and English) across the entire GUI, dynamic battle target labels, inventory item names, CSV exports, and system tray menu.

**Architecture:** Define a strongly-typed `AppLanguage` enum in `src/model.rs` persisted in `rok_config.json`. Create a compile-time dictionary `src/i18n.rs` with `TrKey` enum keys returning `&'static str` for zero-allocation 60 FPS UI rendering. Add dynamic `r.display_target(lang)` and `it.display_name(lang)` methods that preserve canonical storage invariance in memory and disk while dynamically formatting game targets and items. Localize CSV exports in `src/exporters.rs` while keeping JSON vault schemas strictly invariant.

**Tech Stack:** Rust 2021 edition, `eframe` / `egui 0.29`, `muda 0.15` (system tray menu), `serde` / `serde_json`.

**Spec:** [docs/superpowers/specs/2026-10-23-language-options-design.md](file:///C:/roklootswatcher/docs/superpowers/specs/2026-10-23-language-options-design.md)

## Global Constraints

- Windows environment with Rust 2021 edition.
- Canonical storage invariance: `BattleRecord.target` and `rok_history_vault.json` schema must strictly retain canonical values; translation occurs strictly on-the-fly at the presentation and export layer.
- Zero per-frame heap allocation: UI translation lookups via `src/i18n.rs` must return `&'static str`.
- Backwards compatibility: Existing `rok_config.json` files lacking `"language"` must deserialize cleanly to `AppLanguage::Vietnamese`.
- Single self-contained Windows executable: No external `.po` or dynamic JSON locale files loaded at runtime.
- Automated background exports (`motte_filtered_mails.json`) must maintain invariant JSON schema keys.

## Review Focus

1. **Fallback for Missing or Empty `name_en` in `ItemDrop`**: If an item in `loot.yaml` has an empty `name_en` string, `display_name(AppLanguage::English)` must fall back to `name_vi` instead of displaying blank; pinned in Task 2.
2. **Chained Barbarian Battle Target Formatting**: Barbarian battles with counts (e.g. `"Dã Man Cấp 47-52 (x27)"`) must translate cleanly to `"Barbarian Level 47-52 (x27)"` preserving the count suffix; pinned in Task 2.
3. **Legacy `rok_config.json` Without Language Key**: Deserializing an existing config file that lacks the `"language"` key must transparently default to `AppLanguage::Vietnamese` without error or resetting other settings; pinned in Task 1.
4. **CSV Export Quoting and Special Characters**: Localized headers with parentheses (e.g. `"Damage (%)"`, `"Sát thương (%)"`) and multi-word target names must parse cleanly in standard spreadsheet software; pinned in Task 4.
5. **Runtime Tray Language Synchronization**: Switching languages in Settings must update tray menu texts immediately without requiring application restart; pinned in Task 5.

---

### Task 1: `AppLanguage` Enum, Display & Configuration Persistence in `src/model.rs`

**Files:**
- Modify: `src/model.rs:50-80`, `src/model.rs:1160-1200`
- Test: `src/model.rs` (inline unit tests)

**Interfaces:**
- Consumes: `serde::{Serialize, Deserialize}`
- Produces: `AppLanguage` (`Vietnamese`, `English`), `AppConfig.language`

- [ ] **Step 1: Write failing unit tests in `src/model.rs` for `AppLanguage` and config persistence**

```rust
#[test]
fn test_app_language_serde_and_default() {
    assert_eq!(AppLanguage::default(), AppLanguage::Vietnamese);

    let vi_json = r#""vi""#;
    let en_json = r#""en""#;
    assert_eq!(serde_json::from_str::<AppLanguage>(vi_json).unwrap(), AppLanguage::Vietnamese);
    assert_eq!(serde_json::from_str::<AppLanguage>(en_json).unwrap(), AppLanguage::English);

    assert_eq!(serde_json::to_string(&AppLanguage::Vietnamese).unwrap(), r#""vi""#);
    assert_eq!(serde_json::to_string(&AppLanguage::English).unwrap(), r#""en""#);
}

#[test]
fn test_app_config_language_persistence() {
    let legacy_config = r#"{
        "close_behavior": "tray",
        "speedup_unit": "minutes",
        "theme": "dark",
        "autostart": false,
        "autostart_minimized": true
    }"#;
    let cfg: AppConfig = serde_json::from_str(legacy_config).expect("Must deserialize legacy config without language");
    assert_eq!(cfg.language, AppLanguage::Vietnamese);

    let en_config = r#"{
        "language": "en"
    }"#;
    let cfg_en: AppConfig = serde_json::from_str(en_config).expect("Must deserialize config with en");
    assert_eq!(cfg_en.language, AppLanguage::English);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_app_language_serde_and_default`
Expected: FAIL with "cannot find type `AppLanguage`"

- [ ] **Step 3: Implement `AppLanguage` and add `language` to `AppConfig` in `src/model.rs`**

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

impl std::fmt::Display for AppLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppLanguage::Vietnamese => write!(f, "Tiếng Việt"),
            AppLanguage::English => write!(f, "English"),
        }
    }
}
```

Add field `#[serde(default)] pub language: AppLanguage` to `AppConfig` and `language: AppLanguage::default()` to `impl Default for AppConfig`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_app_language_serde_and_default test_app_config_language_persistence`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(i18n): add AppLanguage enum and config persistence"
```

---

### Task 2: Dynamic Display Helpers for Targets and Loot Items in `src/model.rs`

**Files:**
- Modify: `src/model.rs:280-350`
- Test: `src/model.rs` (inline unit tests)

**Interfaces:**
- Consumes: `AppLanguage` from Task 1
- Produces: `ItemDrop::display_name(&self, lang: AppLanguage) -> &str`, `BattleRecord::display_target(&self, lang: AppLanguage) -> String`

- [ ] **Step 1: Write failing unit tests in `src/model.rs` for dynamic target and item naming**

```rust
#[test]
fn test_item_drop_display_name_bilingual() {
    let item_with_both = ItemDrop {
        itype: 1,
        sub_type: 9,
        value: 100,
        pic: "".into(),
        name_vi: "Pha Lê".into(),
        name_en: "Crystal".into(),
        category: "Pha Lê".into(),
        sprites: vec![],
    };
    assert_eq!(item_with_both.display_name(AppLanguage::Vietnamese), "Pha Lê");
    assert_eq!(item_with_both.display_name(AppLanguage::English), "Crystal");

    let item_vi_only = ItemDrop {
        itype: 2,
        sub_type: 9999,
        value: 1,
        pic: "".into(),
        name_vi: "Vật phẩm lạ".into(),
        name_en: "".into(),
        category: "Khác".into(),
        sprites: vec![],
    };
    assert_eq!(item_vi_only.display_name(AppLanguage::Vietnamese), "Vật phẩm lạ");
    assert_eq!(item_vi_only.display_name(AppLanguage::English), "Vật phẩm lạ");
}

#[test]
fn test_battle_record_display_target_bilingual() {
    let mut rec = BattleRecord {
        filename: "test.mail".into(),
        mail_id: "1".into(),
        battle_type: "fort".into(),
        time_raw: 0,
        time_utc: "".into(),
        time_vn: "".into(),
        target: "Pháo Đài Cấp 10".into(),
        target_level: "10".into(),
        talent_add: 0,
        pos_x: 0,
        pos_y: 0,
        damage_pct: 100,
        plunder_tier: 0,
        items: vec![],
        item_totals: HashMap::new(),
        content: "".into(),
    };
    assert_eq!(rec.display_target(AppLanguage::Vietnamese), "Pháo Đài Cấp 10");
    assert_eq!(rec.display_target(AppLanguage::English), "Barbarian Fort Level 10");

    // Chained Barbarians with count suffix
    rec.battle_type = "barbarian".into();
    rec.target = "Dã Man Cấp 47-52 (x27)".into();
    rec.target_level = "47-52".into();
    assert_eq!(rec.display_target(AppLanguage::Vietnamese), "Dã Man Cấp 47-52 (x27)");
    assert_eq!(rec.display_target(AppLanguage::English), "Barbarian Level 47-52 (x27)");

    // Motte target
    rec.battle_type = "motte".into();
    rec.target = "Motte 13".into();
    rec.target_level = "13".into();
    assert_eq!(rec.display_target(AppLanguage::Vietnamese), "Motte 13");
    assert_eq!(rec.display_target(AppLanguage::English), "Motte 13");

    // Fallback reward
    rec.battle_type = "reward".into();
    rec.target = "Phần thưởng".into();
    rec.target_level = "".into();
    assert_eq!(rec.display_target(AppLanguage::Vietnamese), "Phần thưởng");
    assert_eq!(rec.display_target(AppLanguage::English), "Reward");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_item_drop_display_name_bilingual test_battle_record_display_target_bilingual`
Expected: FAIL with "no method named `display_name`" and "no method named `display_target`"

- [ ] **Step 3: Implement `display_name` and `display_target` in `src/model.rs`**

```rust
impl ItemDrop {
    pub fn display_name(&self, lang: AppLanguage) -> &str {
        match lang {
            AppLanguage::English => {
                let trimmed = self.name_en.trim();
                if !trimmed.is_empty() {
                    trimmed
                } else {
                    &self.name_vi
                }
            }
            AppLanguage::Vietnamese => &self.name_vi,
        }
    }
}

impl BattleRecord {
    pub fn display_target(&self, lang: AppLanguage) -> String {
        match lang {
            AppLanguage::Vietnamese => self.target.clone(),
            AppLanguage::English => {
                if self.target.starts_with("Pháo Đài Cấp ") {
                    return self.target.replacen("Pháo Đài Cấp ", "Barbarian Fort Level ", 1);
                } else if self.target == "Pháo Đài Dã Man" {
                    return "Barbarian Fort".to_string();
                } else if self.target.starts_with("Dã Man Cấp ") {
                    return self.target.replacen("Dã Man Cấp ", "Barbarian Level ", 1);
                } else if self.target == "Dã Man" {
                    return "Barbarian".to_string();
                } else if self.target.starts_with("Mỏ Thu Thập Cấp ") {
                    return self.target.replacen("Mỏ Thu Thập Cấp ", "Resource Tile Level ", 1);
                } else if self.target == "Phần thưởng" {
                    return "Reward".to_string();
                }

                match self.battle_type.as_str() {
                    "fort" => {
                        if !self.target_level.is_empty() {
                            format!("Barbarian Fort Level {}", self.target_level)
                        } else {
                            "Barbarian Fort".to_string()
                        }
                    }
                    "barbarian" => {
                        if !self.target_level.is_empty() {
                            format!("Barbarian Level {}", self.target_level)
                        } else {
                            "Barbarian".to_string()
                        }
                    }
                    "motte" => {
                        if !self.target_level.is_empty() {
                            format!("Motte {}", self.target_level)
                        } else {
                            "Motte".to_string()
                        }
                    }
                    _ => self.target.clone(),
                }
            }
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_item_drop_display_name_bilingual test_battle_record_display_target_bilingual`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(i18n): add dynamic display_name and display_target methods"
```

---

### Task 3: Localization Engine & Zero-Cost Translation Dictionary in `src/i18n.rs`

**Files:**
- Create: `src/i18n.rs`
- Modify: `src/main.rs:1-15`
- Test: `src/i18n.rs` (inline unit tests)

**Interfaces:**
- Consumes: `AppLanguage`, `FilterMode`, `DateFilterPreset` from `src/model.rs`
- Produces: `TrKey`, `tr(TrKey, AppLanguage) -> &'static str`, `category_name(&str, AppLanguage) -> &'static str`, `filter_mode_label(&FilterMode, AppLanguage) -> &'static str`, `date_preset_label(&DateFilterPreset, AppLanguage) -> &'static str`

- [ ] **Step 1: Write failing unit tests in `src/i18n.rs` for translation coverage and category helpers**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppLanguage;

    #[test]
    fn test_i18n_all_keys_have_non_empty_translations() {
        for key in TrKey::all() {
            let vi = tr(key, AppLanguage::Vietnamese);
            let en = tr(key, AppLanguage::English);
            assert!(!vi.trim().is_empty(), "Key {:?} has empty Vietnamese translation", key);
            assert!(!en.trim().is_empty(), "Key {:?} has empty English translation", key);
        }
    }

    #[test]
    fn test_category_name_bilingual() {
        assert_eq!(category_name("Tất cả", AppLanguage::Vietnamese), "Tất cả");
        assert_eq!(category_name("Tất cả", AppLanguage::English), "All");
        assert_eq!(category_name("Tài nguyên", AppLanguage::English), "Resources");
        assert_eq!(category_name("Tăng Tốc", AppLanguage::English), "Speedups");
        assert_eq!(category_name("Pha Lê", AppLanguage::English), "Crystals");
        assert_eq!(category_name("Kinh Nghiệm", AppLanguage::English), "Experience");
        assert_eq!(category_name("Trang Bị", AppLanguage::English), "Equipment");
        assert_eq!(category_name("Khác", AppLanguage::English), "Others");
    }

    #[test]
    fn test_filter_and_date_preset_labels_bilingual() {
        assert_eq!(filter_mode_label(&FilterMode::Motte, AppLanguage::Vietnamese), "Chỉ Motte (Motte Destroyed)");
        assert_eq!(filter_mode_label(&FilterMode::Motte, AppLanguage::English), "Motte Only");
        assert_eq!(date_preset_label(&DateFilterPreset::Today, AppLanguage::Vietnamese), "Hôm nay");
        assert_eq!(date_preset_label(&DateFilterPreset::Today, AppLanguage::English), "Today");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_i18n_all_keys_have_non_empty_translations`
Expected: FAIL with "unresolved import / file not found"

- [ ] **Step 3: Implement `src/i18n.rs` and register `mod i18n;` in `src/main.rs`**

Add `mod i18n;` to `src/main.rs`.
Implement `TrKey` enum with `all() -> &'static [TrKey]`, `tr(TrKey, AppLanguage) -> &'static str`, and `impl AppLanguage { pub fn t(&self, key: TrKey) -> &'static str { tr(key, *self) } }`.
Include all keys for:
- Navigation sidebar (Dashboard, Battles, Inventory, Settings, Subtitles)
- Mini status card (Watching, Paused, Rescan, Scan, Vault battles count)
- Dashboard headers, metric cards, 5 latest reports, top 10 loot items
- Table columns (Time, Battle Type, Target, Damage, Plunder Tier, Crystals, Resources, Experience, Speedups, Actions)
- Filter toolbar (Report Type, Date Range, Speedup Unit, All Time, Today, Yesterday, 7 Days, 30 Days, Custom)
- Settings card titles, language selection, close behavior, autostart, mail folder pickers
- System tray context menu items

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_i18n_all_keys_have_non_empty_translations test_category_name_bilingual test_filter_and_date_preset_labels_bilingual`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/i18n.rs src/main.rs
git commit -m "feat(i18n): create localization engine and dictionary"
```

---

### Task 4: Exporter Localization in `src/exporters.rs` & `src/gui.rs`

**Files:**
- Modify: `src/exporters.rs:1-80`
- Modify: `src/gui.rs:620-645`
- Test: `src/exporters.rs` (inline unit tests)

**Interfaces:**
- Consumes: `BattleRecord`, `AppLanguage`, `BattleRecord::display_target`
- Produces: `generate_csv(records: &[BattleRecord], lang: AppLanguage) -> String`

- [ ] **Step 1: Write failing unit tests in `src/exporters.rs` for localized CSV generation**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AppLanguage, BattleRecord, ItemDrop};
    use std::collections::HashMap;

    #[test]
    fn test_generate_csv_vietnamese_and_english() {
        let rec = BattleRecord {
            filename: "mail1".into(),
            mail_id: "100".into(),
            battle_type: "fort".into(),
            time_raw: 12345,
            time_utc: "".into(),
            time_vn: "2026-10-23 12:00:00".into(),
            target: "Pháo Đài Cấp 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 1,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let csv_vi = generate_csv(&[rec.clone()], AppLanguage::Vietnamese);
        assert!(csv_vi.starts_with("\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút)\n"));
        assert!(csv_vi.contains("\"2026-10-23 12:00:00\",\"Pháo Đài\",\"Pháo Đài Cấp 10\",100,1,0,0,0,0,0,0,0\n"));

        let csv_en = generate_csv(&[rec], AppLanguage::English);
        assert!(csv_en.starts_with("\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min)\n"));
        assert!(csv_en.contains("\"2026-10-23 12:00:00\",\"Barbarian Fort\",\"Barbarian Fort Level 10\",100,1,0,0,0,0,0,0,0\n"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_generate_csv_vietnamese_and_english`
Expected: FAIL with "cannot find function `generate_csv`"

- [ ] **Step 3: Implement `generate_csv` in `src/exporters.rs` and update `export_active_csv` in `src/gui.rs`**

In `src/exporters.rs`:
```rust
pub fn generate_csv(records: &[BattleRecord], lang: AppLanguage) -> String {
    let header = match lang {
        AppLanguage::Vietnamese => "\u{FEFF}Thời gian,Loại trận,Mục tiêu,Sát thương (%),Tier quà,Pha lê,Tổng tài nguyên,Kinh nghiệm,Mũi tên,Tăng tốc chung (phút),Tăng tốc lính (phút),Tăng tốc trị (phút)\n",
        AppLanguage::English => "\u{FEFF}Time,Battle Type,Target,Damage (%),Loot Tier,Crystals,Total Resources,Experience,Arrows,Univ Speedup (min),Train Speedup (min),Heal Speedup (min)\n",
    };

    let mut w = String::from(header);
    for r in records {
        let btype_label = match (r.battle_type.as_str(), lang) {
            ("fort", AppLanguage::Vietnamese) => "Pháo Đài",
            ("fort", AppLanguage::English) => "Barbarian Fort",
            ("barbarian", AppLanguage::Vietnamese) => "Dã Man",
            ("barbarian", AppLanguage::English) => "Barbarian",
            ("motte", _) => "Motte",
            ("gathering", AppLanguage::Vietnamese) => "Thu Thập",
            ("gathering", AppLanguage::English) => "Gathering",
            (other, _) => other,
        };

        let target_display = r.display_target(lang);
        let cry = r.item_quantity(1, 9);
        let rss = r.total_resource_value();
        let exp = r.total_exp_value();
        let arrow = r.item_quantity(2, 145);

        w.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},{},{},{},{},{},{},{},{}\n",
            r.time_vn, btype_label, target_display, r.damage_pct, r.plunder_tier,
            cry, rss, exp, arrow,
            r.univ_speedup_minutes(), r.training_speedup_minutes(), r.heal_speedup_minutes()
        ));
    }
    w
}
```

In `src/gui.rs` `export_active_csv`:
Replace manual CSV string builder with `let w = crate::exporters::generate_csv(&records, self.language);`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_generate_csv_vietnamese_and_english`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/exporters.rs src/gui.rs
git commit -m "feat(i18n): localize CSV export headers, row battle types, and targets"
```

---

### Task 5: System Tray Dynamic Localization in `src/tray.rs`

**Files:**
- Modify: `src/tray.rs:18-50`, `src/tray.rs:120-140`
- Test: `src/tray.rs` (inline unit tests)

**Interfaces:**
- Consumes: `AppLanguage` from `src/model.rs`
- Produces: `SystemTrayManager::update_language(&self, lang: AppLanguage)`

- [ ] **Step 1: Write failing unit test in `src/tray.rs` for tray localization string constants**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppLanguage;

    #[test]
    fn test_tray_menu_text_constants() {
        assert_eq!(tray_show_text(AppLanguage::Vietnamese), "Hiện giao diện chính");
        assert_eq!(tray_show_text(AppLanguage::English), "Open Main Window");
        assert_eq!(tray_rescan_text(AppLanguage::Vietnamese), "Quét lại dữ liệu");
        assert_eq!(tray_rescan_text(AppLanguage::English), "Rescan Data");
        assert_eq!(tray_toggle_text(AppLanguage::Vietnamese), "Dừng / Tiếp tục theo dõi");
        assert_eq!(tray_toggle_text(AppLanguage::English), "Pause / Resume Watching");
        assert_eq!(tray_quit_text(AppLanguage::Vietnamese), "Thoát ứng dụng hoàn toàn");
        assert_eq!(tray_quit_text(AppLanguage::English), "Exit Application");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_tray_menu_text_constants`
Expected: FAIL with "cannot find function `tray_show_text`"

- [ ] **Step 3: Implement `tray_*_text` helpers and `update_language` method on `SystemTrayManager`**

```rust
pub fn tray_show_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Hiện giao diện chính",
        AppLanguage::English => "Open Main Window",
    }
}
// similar for rescan, toggle, quit, tooltip
```

In `SystemTrayManager`:
```rust
pub fn update_language(&self, lang: AppLanguage) {
    let _ = self.show_item.set_text(tray_show_text(lang));
    let _ = self.rescan_item.set_text(tray_rescan_text(lang));
    let _ = self.toggle_item.set_text(tray_toggle_text(lang));
    let _ = self.quit_item.set_text(tray_quit_text(lang));
    let tooltip = match lang {
        AppLanguage::Vietnamese => "Rise of Kingdoms - Loot Watcher (Nhấp để mở lại)",
        AppLanguage::English => "Rise of Kingdoms - Loot Watcher (Click to restore)",
    };
    let _ = self._tray_icon.set_tooltip(Some(tooltip));
}
```

In `SystemTrayManager::new`: accept initial `lang: AppLanguage` (or call `update_language(lang)`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_tray_menu_text_constants`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/tray.rs
git commit -m "feat(i18n): add dynamic language switching to system tray menu"
```

---

### Task 6: GUI Localization Integration across All Views in `src/gui.rs`

**Files:**
- Modify: `src/gui.rs`
- Test: Full cargo test suite & cargo build check

**Interfaces:**
- Consumes: `AppLanguage` from `src/model.rs`, `tr` from `src/i18n.rs`, `display_target`, `display_name`
- Produces: Fully bilingual GUI reactive to `self.language`

- [ ] **Step 1: Write integration test in `src/gui.rs` verifying GUI state holds and switches language**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppLanguage;

    #[test]
    fn test_gui_language_state_default() {
        let config = AppConfig::default();
        assert_eq!(config.language, AppLanguage::Vietnamese);
    }
}
```

- [ ] **Step 2: Add `pub language: AppLanguage` to `RoKLootWatcherApp` and wire to `save_config`**

In `RoKLootWatcherApp::new`:
```rust
let initial_language = config.language;
// In struct initialization:
language: initial_language,
```
In `save_config`:
```rust
let config = AppConfig {
    close_behavior: self.close_behavior,
    speedup_unit: self.speedup_unit,
    mail_dir: Some(self.watcher.get_mail_dir()),
    mode: Some(self.selected_mode),
    theme: self.theme,
    autostart: self.autostart,
    autostart_minimized: self.autostart_minimized,
    language: self.language,
};
```

- [ ] **Step 3: Add Language Selector in `render_view_settings`**

Under Card 3 (App Behavior & Protection) or as a dedicated "LANGUAGE / NGÔN NGỮ" card:
```rust
ui.horizontal(|ui| {
    ui.label(egui::RichText::new(self.language.t(TrKey::SettingsLanguageLabel)).strong().color(c.canvas_tint));
    let prev_lang = self.language;
    ui.selectable_value(&mut self.language, AppLanguage::Vietnamese, "Tiếng Việt");
    ui.selectable_value(&mut self.language, AppLanguage::English, "English");
    if self.language != prev_lang {
        self.save_config();
        if let Some(tray) = &self.tray {
            tray.update_language(self.language);
        }
    }
});
```

- [ ] **Step 4: Localize Sidebar, Dashboard, Battles, Inventory, and Logs views**

1. **Sidebar (`render_sidebar`)**:
   - Navigation item titles and descriptions use `self.language.t(TrKey::Nav...)`.
   - Mini status card texts ("Kho Vault: {} trận" / "Vault: {} battles", "Quét" / "Scan", "Dừng" / "Pause", "Bật" / "Resume").
2. **Dashboard (`render_view_dashboard`)**:
   - Header title & subtitle.
   - Filter toolbar: "Loại báo cáo:" / "Report Type:", Filter modes use `filter_mode_label(&mode, self.language)`, Date presets use `date_preset_label(&preset, self.language)`.
   - 9 Metric Card titles: "TỔNG SỐ TRẬN" / "TOTAL BATTLES", "TỔNG PHA LÊ" / "TOTAL CRYSTALS", etc.
   - 5 Latest reports: target uses `r.display_target(self.language)`.
   - Top 10 Loot Items: item names use `it.display_name(self.language)`.
3. **Battles View (`render_view_battles`)**:
   - Table column headers use translated labels.
   - Rows use `r.display_target(self.language)`.
   - Side inspector details use `self.language.t(...)` and `it.display_name(self.language)`.
4. **Inventory View (`render_view_inventory`)**:
   - Category selector uses `category_name(&cat, self.language)`.
   - Item rows use `it.display_name(self.language)`.
   - Table headers ("Vật phẩm" / "Item", "Phân loại" / "Category", "Tổng số lượng" / "Total Quantity", "Giá trị quy đổi" / "Value").
5. **Settings View (`render_view_settings`)**:
   - Section card titles, button labels ("Duyệt thư mục..." / "Browse...", "Quét Thư Mới" / "Scan New Mails", etc.).

- [ ] **Step 5: Run full test suite and verify build**

Run: `cargo test`
Expected: ALL tests PASS

Run: `cargo build`
Expected: Finished `dev` profile successfully

- [ ] **Step 6: Commit**

```bash
git add src/gui.rs
git commit -m "feat(gui): integrate bilingual translations across all views"
```
