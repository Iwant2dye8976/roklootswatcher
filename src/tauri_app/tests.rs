use std::collections::HashMap;
use chrono::TimeZone;

use crate::analyzer::{current_date_vn, generate_statistics};
use crate::loot_catalog::LootCatalog;
use crate::model::*;
use crate::tauri_app::commands::*;
use crate::tauri_app::dto::*;
use crate::watcher::RoKMailWatcher;

fn make_drop(
    itype: u32,
    sub_type: u32,
    value: i64,
    name_vi: &str,
    name_en: &str,
    category: &str,
) -> ItemDrop {
    ItemDrop {
        itype,
        sub_type,
        value,
        pic: String::new(),
        name_vi: name_vi.to_string(),
        name_en: name_en.to_string(),
        category: category.to_string(),
        sprites: Vec::new(),
    }
}

fn make_today_record(
    mail_id: &str,
    account_id: &str,
    player_name: &str,
    battle_type: &str,
    target: &str,
    items: Vec<ItemDrop>,
) -> BattleRecord {
    let today = current_date_vn();
    let naive_dt = today.and_hms_opt(12, 30, 0).unwrap();
    let vn_offset = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
    let dt_vn = vn_offset.from_local_datetime(&naive_dt).unwrap();
    let dt_utc = dt_vn.with_timezone(&chrono::Utc);

    let mut rec = BattleRecord {
        filename: format!("Persistent.Mail.{}", mail_id),
        mail_id: mail_id.to_string(),
        battle_type: battle_type.to_string(),
        time_raw: dt_utc.timestamp() * 1_000_000,
        time_utc: dt_utc.format("%Y-%m-%d %H:%M:%S").to_string(),
        time_vn: dt_vn.format("%Y-%m-%d %H:%M:%S").to_string(),
        target: target.to_string(),
        target_level: "5".to_string(),
        talent_add: 5,
        pos_x: 500,
        pos_y: 600,
        damage_pct: 80,
        plunder_tier: 2,
        items,
        item_totals: HashMap::new(),
        content: String::new(),
        account_id: account_id.to_string(),
        player_name: player_name.to_string(),
    };
    rec.item_totals = rec.compute_item_totals();
    rec
}

#[test]
fn test_serde_wire_contracts_exact_ts_and_legacy_aliases() {
    // FilterMode exact TS strings
    let modes = [
        ("all-targets", FilterMode::AllTargets),
        ("motte", FilterMode::Motte),
        ("fort", FilterMode::Fort),
        ("barbarian", FilterMode::Barbarian),
        ("all-forts", FilterMode::AllForts),
        ("all-rewards", FilterMode::AllRewards),
        ("gathering", FilterMode::Gathering),
    ];
    for (wire, expected) in modes {
        let json = format!("\"{}\"", wire);
        let parsed: FilterMode = serde_json::from_str(&json).expect("deserialize FilterMode");
        assert_eq!(parsed, expected);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }

    // DateFilterPreset exact TS strings (every preset in GlobalFilterBar.tsx) + legacy aliases
    let ts_presets = [
        ("AllTime", DateFilterPreset::AllTime, None),
        ("Today", DateFilterPreset::Today, Some(1)),
        ("Yesterday", DateFilterPreset::Yesterday, None),
        ("Last3Days", DateFilterPreset::Last3Days, Some(3)),
        ("Last7Days", DateFilterPreset::Last7Days, Some(7)),
        ("Last14Days", DateFilterPreset::Last14Days, Some(14)),
        ("Last15Days", DateFilterPreset::Last15Days, Some(15)),
        ("Last30Days", DateFilterPreset::Last30Days, Some(30)),
        ("Custom", DateFilterPreset::Custom, None),
    ];
    for (wire, expected_variant, expected_days) in ts_presets {
        let json = format!("\"{}\"", wire);
        let parsed: DateFilterPreset =
            serde_json::from_str(&json).expect("deserialize TS DateFilterPreset");
        assert_eq!(parsed, expected_variant);
        assert_eq!(parsed.to_days(), expected_days);
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }
    assert_eq!(
        serde_json::from_str::<DateFilterPreset>("\"all\"").unwrap(),
        DateFilterPreset::AllTime
    );
    assert_eq!(
        serde_json::from_str::<DateFilterPreset>("\"Days7\"").unwrap(),
        DateFilterPreset::Last7Days
    );
    assert_eq!(
        serde_json::from_str::<DateFilterPreset>("\"Days15\"").unwrap(),
        DateFilterPreset::Last15Days
    );

    // SpeedupUnit, AppTheme, AppLanguage, CloseBehavior, OverlayDisplayMode
    assert_eq!(
        serde_json::to_string(&SpeedupUnit::Hours).unwrap(),
        "\"hours\""
    );
    assert_eq!(
        serde_json::from_str::<SpeedupUnit>("\"Hours\"").unwrap(),
        SpeedupUnit::Hours
    );
    assert_eq!(serde_json::to_string(&AppTheme::Light).unwrap(), "\"light\"");
    assert_eq!(
        serde_json::from_str::<AppTheme>("\"Light\"").unwrap(),
        AppTheme::Light
    );
    assert_eq!(
        serde_json::to_string(&AppLanguage::Vietnamese).unwrap(),
        "\"vi\""
    );
    assert_eq!(
        serde_json::to_string(&CloseBehavior::Tray).unwrap(),
        "\"tray\""
    );
    assert_eq!(
        serde_json::from_str::<CloseBehavior>("\"Close\"").unwrap(),
        CloseBehavior::Close
    );
    assert_eq!(
        serde_json::from_str::<OverlayDisplayMode>("\"Full\"").unwrap(),
        OverlayDisplayMode::Cards
    );
    assert_eq!(
        serde_json::from_str::<OverlayDisplayMode>("\"Mini\"").unwrap(),
        OverlayDisplayMode::MiniHud
    );
    assert_eq!(
        serde_json::to_string(&OverlayDisplayMode::Cards).unwrap(),
        "\"Cards\""
    );

    // PinnedItemKey roundtrip (unit variants + CatalogItem with isub / sub_type alias)
    let key_spd: PinnedItemKey = serde_json::from_str("\"SpeedupUniversal\"").unwrap();
    assert_eq!(key_spd, PinnedItemKey::SpeedupUniversal);

    let key_cat_json = r#"{"CatalogItem":{"itype":1,"isub":2,"canonical_name":"Gems"}}"#;
    let key_cat: PinnedItemKey = serde_json::from_str(key_cat_json).unwrap();
    assert_eq!(
        key_cat,
        PinnedItemKey::CatalogItem {
            itype: 1,
            isub: 2,
            canonical_name: "Gems".to_string(),
        }
    );
    let key_alias: PinnedItemKey = serde_json::from_str(
        r#"{"CatalogItem":{"itype":2,"sub_type":109,"canonical_name":"Book of Covenant"}}"#,
    )
    .unwrap();
    assert_eq!(
        key_alias,
        PinnedItemKey::CatalogItem {
            itype: 2,
            isub: 109,
            canonical_name: "Book of Covenant".to_string(),
        }
    );
}

#[test]
fn test_overlay_config_partial_update_dto_leaves_unset_fields_none() {
    let partial: OverlayConfigUpdateDto =
        serde_json::from_str(r#"{"opacity": 0.75, "offset_x": 48}"#).unwrap();
    assert_eq!(partial.opacity, Some(0.75));
    assert_eq!(partial.offset_x, Some(48.0));
    assert!(partial.enabled.is_none());
    assert!(partial.mode.is_none());
    assert!(partial.filter.is_none());
    assert!(partial.pinned_items.is_none());
    assert!(partial.hotkey_enabled.is_none());
    assert!(partial.transparent_bg.is_none());
}

#[test]
fn test_resolve_date_filter_yesterday_excludes_today_and_clears_stale_custom_dates() {
    let today = current_date_vn();
    let expected_yest = today - chrono::Duration::days(1);

    let (s, e, days, cf, ct) =
        resolve_date_filter(DateFilterPreset::Yesterday, "2026-01-01", "2026-01-31");
    assert_eq!(s, Some(expected_yest));
    assert_eq!(e, Some(expected_yest));
    assert_eq!(days, None, "Yesterday must not pass rolling days=2");
    assert!(cf.is_empty());
    assert!(ct.is_empty());

    let (s7, e7, d7, cf7, ct7) =
        resolve_date_filter(DateFilterPreset::Last7Days, "2026-01-01", "2026-01-31");
    assert_eq!(s7, None);
    assert_eq!(e7, None);
    assert_eq!(d7, Some(7));
    assert!(cf7.is_empty());
    assert!(ct7.is_empty());

    let (sc, ec, dc, cfc, ctc) =
        resolve_date_filter(DateFilterPreset::Custom, " 2026-03-01 ", " 2026-03-10 ");
    assert_eq!(sc, chrono::NaiveDate::from_ymd_opt(2026, 3, 1));
    assert_eq!(ec, chrono::NaiveDate::from_ymd_opt(2026, 3, 10));
    assert_eq!(dc, None);
    assert_eq!(cfc, "2026-03-01");
    assert_eq!(ctc, "2026-03-10");
}

#[test]
fn test_sum_pinned_catalog_item_and_build_pinned_chips_accurate_item_ids() {
    // Include Wood (1,2) and Food Packs (2,3) & (2,4) alongside real Gems (1,5) + Gem Pack (2,27=10 gems),
    // Covenant Books (2,109), Arrows (2,145), Crystals (1,9), and Speedups!
    let rec = make_today_record(
        "1001",
        "102938475",
        "GovPrime",
        "barbarian",
        "Dã Man Cấp 40",
        vec![
            make_drop(1, 2, 50_000, "Gỗ", "Wood", "Tài Nguyên"),
            make_drop(2, 3, 10, "Gói Thực Phẩm Cấp 1", "Lv1 Food Pack", "Tài Nguyên"),
            make_drop(2, 4, 8, "Gói Thực Phẩm Cấp 2", "Lv2 Food Pack", "Tài Nguyên"),
            make_drop(1, 5, 30, "Đá quý", "Gems", "Tài Nguyên"),
            make_drop(2, 27, 4, "Gói Đá Quý (50)", "50 Gem Pack", "Tài Nguyên"), // 4 * 50 = 200 gems
            make_drop(1, 9, 120, "Pha Lê", "Crystals", "Pha Lê"),
            make_drop(2, 109, 3, "Sách Giao Ước", "Book of Covenant", "Vật Phẩm"),
            make_drop(2, 145, 5, "Mũi Tên Kháng Cự", "Arrow of Resistance", "Vật Phẩm"),
            make_drop(2, 36, 2, "Tăng tốc 60 phút", "60m Speedup", "Tăng Tốc"), // 120m univ
            make_drop(2, 76, 1, "Tăng tốc huấn luyện 60 phút", "60m Train Speedup", "Tăng Tốc"),
            make_drop(2, 96, 1, "Tăng tốc chữa lành 60 phút", "60m Heal Speedup", "Tăng Tốc"),
        ],
    );

    // Even when legacy default_overlay_pinned_items passes (1,2,"Gems"), (2,3,"Book of Covenant"), (2,4,"Arrow of Resistance"):
    assert_eq!(
        sum_pinned_catalog_item(&rec, "Gems", 1, 2),
        230,
        "Must sum direct (1,5)=30 + gem packs (2,27)*50=200 = 230, ignoring (1,2) Wood"
    );
    assert_eq!(
        sum_pinned_catalog_item(&rec, "Book of Covenant", 2, 3),
        3,
        "Must sum (2,109) Covenant Books, ignoring (2,3) Food Pack"
    );
    assert_eq!(
        sum_pinned_catalog_item(&rec, "Arrow of Resistance", 2, 4),
        5,
        "Must sum (2,145) Arrows, ignoring (2,4) Food Pack"
    );
    assert_eq!(sum_pinned_catalog_item(&rec, "Crystals", 1, 9), 120);

    let mut pinned = default_overlay_pinned_items();
    pinned.push(PinnedItemKey::SpeedupTraining);
    pinned.push(PinnedItemKey::SpeedupHealing);

    let chips = build_pinned_chips(&[rec], &pinned, "#38BDF8");
    assert_eq!(chips.len(), 6);
    assert_eq!(chips[0].label, "GEMS");
    assert_eq!(chips[0].display_val, "230");
    assert_eq!(chips[1].label, "BOOKS");
    assert_eq!(chips[1].display_val, "3");
    assert_eq!(chips[2].label, "ARROWS");
    assert_eq!(chips[2].display_val, "5");
    assert_eq!(chips[3].label, "SPEEDUP");
    assert_eq!(chips[3].display_val, "2h");
    assert_eq!(chips[4].label, "TRAIN");
    assert_eq!(chips[4].display_val, "1h");
    assert_eq!(chips[5].label, "HEAL");
    assert_eq!(chips[5].display_val, "1h");
}

#[test]
fn test_overlay_payload_respects_account_isolation() {
    let temp_dir = std::env::temp_dir().join(format!(
        "rok_tauri_overlay_uid_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);

    let (tx, _rx) = std::sync::mpsc::channel();
    let watcher = RoKMailWatcher::new(
        Some(temp_dir.clone()),
        Some(temp_dir.clone()),
        FilterMode::AllRewards,
        LootCatalog::empty(),
        tx,
    );

    let rec_main = make_today_record(
        "m_main",
        "player_111111",
        "MainGov",
        "motte",
        "Tiêu diệt Motte",
        vec![make_drop(1, 5, 50, "Đá quý", "Gems", "Tài Nguyên")],
    );
    let rec_farm = make_today_record(
        "m_farm",
        "player_222222",
        "FarmGov",
        "barbarian",
        "Dã Man Cấp 35",
        vec![make_drop(1, 5, 200, "Đá quý", "Gems", "Tài Nguyên")],
    );

    {
        let mut v = watcher.vault.write().unwrap();
        v.upsert(rec_main);
        v.upsert(rec_farm);
    }
    watcher.recompute_filtered_records_and_stats();

    let cfg = AppConfig::default();

    // 1. Selected account = 111111 -> only MainGov's 1 motte run & 50 gems
    let payload_main = get_overlay_payload_internal(&watcher, &cfg, Some("111111"), 12);
    assert_eq!(payload_main.today_runs.total, 1);
    assert_eq!(payload_main.today_runs.motte, 1);
    assert_eq!(payload_main.today_runs.barb, 0);
    assert_eq!(payload_main.pinned_chips[0].display_val, "50");
    assert_eq!(payload_main.idle_secs, 12);

    // 2. Main account UIDs = ["111111"] with no selected_account -> FarmGov still excluded!
    watcher.set_main_account_uids(vec!["111111".to_string()]);
    let payload_isolated = get_overlay_payload_internal(&watcher, &cfg, None, 0);
    assert_eq!(payload_isolated.today_runs.total, 1);
    assert_eq!(payload_isolated.pinned_chips[0].display_val, "50");

    drop(watcher);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_query_battles_from_slice_localized_search_and_sorting() {
    let mut r1 = make_today_record(
        "b1",
        "111",
        "Alpha",
        "fort",
        "Pháo Đài Cấp 10",
        vec![
            make_drop(2, 109, 5, "Sách Giao Ước", "Book of Covenant", "Vật Phẩm"),
            make_drop(1, 5, 20, "Đá quý", "Gems", "Tài Nguyên"),
        ],
    );
    r1.time_raw = 1_000_000;

    let mut r2 = make_today_record(
        "b2",
        "222",
        "Bravo",
        "barbarian",
        "Dã Man Cấp 25",
        vec![
            make_drop(2, 109, 1, "Sách Giao Ước", "Book of Covenant", "Vật Phẩm"),
            make_drop(1, 5, 100, "Đá quý", "Gems", "Tài Nguyên"),
        ],
    );
    r2.time_raw = 2_000_000;

    let slice = vec![r1, r2];

    // English localized target search ("Barbarian Fort Level 10") finds r1!
    let found_en = query_battles_from_slice(&slice, AppLanguage::English, "Barbarian Fort", None, None);
    assert_eq!(found_en.len(), 1);
    assert_eq!(found_en[0].id, "b1");
    assert_eq!(found_en[0].covenant_books, Some(5));

    // Sort by gems ascending -> b1 (20) then b2 (100)
    let sorted_gems_asc =
        query_battles_from_slice(&slice, AppLanguage::English, "", Some("gems"), Some("asc"));
    assert_eq!(sorted_gems_asc[0].id, "b1");
    assert_eq!(sorted_gems_asc[1].id, "b2");

    // Sort by books descending -> b1 (5) then b2 (1)
    let sorted_books_desc =
        query_battles_from_slice(&slice, AppLanguage::English, "", Some("books"), Some("desc"));
    assert_eq!(sorted_books_desc[0].id, "b1");
    assert_eq!(sorted_books_desc[1].id, "b2");
}

#[test]
fn test_query_inventory_disjoint_six_tabs_and_gem_packs() {
    let rec = make_today_record(
        "inv1",
        "111",
        "Alpha",
        "fort",
        "Pháo Đài Cấp 10",
        vec![
            make_drop(1, 1, 10_000, "Thực phẩm", "Food", "Tài Nguyên"),
            make_drop(2, 26, 7, "Gói Đá Quý (10)", "10 Gem Pack", "Tài Nguyên"), // 7 packs = 70 gems
            make_drop(2, 412, 4, "Da Thuộc Thường", "Normal Leather", "Vật Liệu"),
            make_drop(2, 36, 3, "Tăng tốc 60 phút", "60m Speedup", "Tăng Tốc"),
            make_drop(1, 9, 200, "Pha Lê", "Crystals", "Pha Lê"),
            make_drop(2, 109, 6, "Sách Giao Ước", "Book of Covenant", "Vật Phẩm"),
            make_drop(2, 145, 9, "Mũi Tên Kháng Cự", "Arrow of Resistance", "Vật Phẩm"),
            make_drop(2, 130, 12, "Sách Kiến Thức Cấp 3", "Lv3 Tome of Knowledge", "Kinh Nghiệm"),
            make_drop(2, 124, 2, "Bình 50 AP", "Basic AP Recovery", "Vật Phẩm"),
        ],
    );

    let stats = generate_statistics(&[rec]);
    let all_val = query_inventory_from_stats(&stats, AppLanguage::English, "", "All", None, None);
    let all_items = all_val["items"].as_array().unwrap();
    assert_eq!(all_items.len(), 9);
    assert_eq!(all_val["resources"]["gem_packs"].as_i64(), Some(7));
    assert_eq!(all_val["resources"]["gem_val"].as_i64(), Some(70));
    assert_eq!(all_val["materials"]["leather_total"].as_i64(), Some(4));

    // Verify all 6 tabs partition every single item cleanly without overlap or omission!
    let tabs = [
        "Resources",
        "Materials",
        "Speedups",
        "Crystals",
        "Books & Arrows",
        "Other",
    ];
    let mut seen_keys = Vec::new();
    for tab in tabs {
        let v = query_inventory_from_stats(&stats, AppLanguage::English, "", tab, None, None);
        let arr = v["items"].as_array().unwrap();
        for item in arr {
            let k = (
                item["itype"].as_u64().unwrap(),
                item["sub_type"].as_u64().unwrap(),
            );
            assert!(
                !seen_keys.contains(&k),
                "Item {:?} appeared in multiple inventory tabs!",
                k
            );
            seen_keys.push(k);
        }
    }
    assert_eq!(
        seen_keys.len(),
        9,
        "Every item in All must appear in exactly one of the 6 category tabs"
    );

    // Specifically verify Books, Arrows, and Tomes are in "Books & Arrows" and NOT in "Other"
    let books_tab =
        query_inventory_from_stats(&stats, AppLanguage::English, "", "Books & Arrows", None, None);
    let books_arr = books_tab["items"].as_array().unwrap();
    assert_eq!(books_arr.len(), 3);

    let other_tab =
        query_inventory_from_stats(&stats, AppLanguage::English, "", "Other", None, None);
    let other_arr = other_tab["items"].as_array().unwrap();
    assert_eq!(other_arr.len(), 1, "Only the 50 AP bottle should be in Other");
    assert_eq!(other_arr[0]["sub_type"].as_u64(), Some(124));
}
