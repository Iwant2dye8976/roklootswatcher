use serde::{Deserialize, Serialize};
use crate::analyzer::ApEconomics;
use crate::model::{
    get_item_converted_display_for_lang, AppLanguage, AppTheme, BattleRecord, CloseBehavior,
    DateFilterPreset, FilterMode, OverlayConfig, OverlayDisplayMode, OverlayFadeStyle,
    PinnedItemKey, SpeedupUnit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummaryDto {
    pub uid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStatusDto {
    pub logged_in: bool,
    pub email: Option<String>,
    pub last_sync_time: Option<String>,
    pub pending_count: usize,
    pub is_syncing: bool,
    pub is_restoring: bool,
    pub sync_error: Option<String>,
    pub auto_sync: bool,
    pub backup_interval_minutes: u32,
    pub backup_on_exit: bool,
    pub stay_login: bool,
    pub login_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSnapshotDto {
    pub version: String,
    pub mail_dir: String,
    pub mode: FilterMode,
    pub date_preset: DateFilterPreset,
    pub date_from: String,
    pub date_to: String,
    pub account_uid: Option<String>,
    pub available_accounts: Vec<AccountSummaryDto>,
    pub detected_accounts: Vec<AccountSummaryDto>,
    pub non_main_records_count: usize,
    pub main_account_uids: Vec<String>,
    pub ignore_non_main_reports: bool,
    pub theme: AppTheme,
    pub language: AppLanguage,
    pub speedup_unit: SpeedupUnit,
    pub close_behavior: CloseBehavior,
    pub is_watching: bool,
    pub is_scanning: bool,
    pub watcher_status_text: String,
    pub vault_count: usize,
    pub auto_scan: bool,
    pub autostart: bool,
    pub autostart_minimized: bool,
    pub overlay_config: OverlayConfig,
    pub cloud_status: CloudStatusDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootItemDto {
    pub name_vi: String,
    pub name_en: String,
    pub display_name: String,
    pub category: String,
    pub itype: u32,
    pub sub_type: u32,
    pub value: i64,
    pub converted_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecordDto {
    pub id: String,
    pub file_name: String,
    pub time_raw: i64,
    pub time_vn: String,
    pub time_utc: String,
    pub target: String,
    pub target_level: String,
    pub battle_type: String,
    pub damage_pct: i32,
    pub plunder_tier: i32,
    pub talent_add: i64,
    pub pos_x: i32,
    pub pos_y: i32,
    pub account_uid: String,
    pub account_name: String,
    pub items: Vec<LootItemDto>,
    pub display_target: Option<String>,
    pub crystals: Option<i64>,
    pub speedup_univ_mins: Option<i64>,
    pub speedup_train_mins: Option<i64>,
    pub speedup_heal_mins: Option<i64>,
    pub total_rss: Option<i64>,
    pub tome_exp: Option<i64>,
    pub covenant_books: Option<i64>,
    pub arrows: Option<i64>,
    pub choice_packs: Option<i64>,
    pub gems: Option<i64>,
}

impl BattleRecordDto {
    pub fn from_record(r: &BattleRecord, lang: AppLanguage) -> Self {
        let cry = r.crystal_quantity();
        let univ_mins = r.univ_speedup_minutes();
        let train_mins = r.training_speedup_minutes();
        let heal_mins = r.heal_speedup_minutes();
        let rss_breakdown = r.total_resources_breakdown();
        let total_rss = rss_breakdown.total_rss_val();
        let choice_packs = rss_breakdown.choice_packs;
        let tome_exp = r.total_exp_value();
        let books = r.item_quantity(2, 109);
        let arrows = r.item_quantity(2, 145);
        let gems = r.compute_gems();

        Self {
            id: r.mail_id.clone(),
            file_name: r.filename.clone(),
            time_raw: r.time_raw,
            time_vn: r.time_vn.clone(),
            time_utc: r.time_utc.clone(),
            target: r.target.clone(),
            target_level: r.target_level.clone(),
            battle_type: r.battle_type.clone(),
            damage_pct: r.damage_pct,
            plunder_tier: r.plunder_tier,
            talent_add: r.talent_add,
            pos_x: r.pos_x,
            pos_y: r.pos_y,
            account_uid: r.account_id.clone(),
            account_name: r.player_name.clone(),
            items: r
                .items
                .iter()
                .map(|it| LootItemDto {
                    name_vi: it.name_vi.clone(),
                    name_en: it.name_en.clone(),
                    display_name: it.display_name(lang).to_string(),
                    category: it.category.clone(),
                    itype: it.itype,
                    sub_type: it.sub_type,
                    value: it.value,
                    converted_value: get_item_converted_display_for_lang(
                        it.itype,
                        it.sub_type,
                        it.value,
                        lang,
                    ),
                })
                .collect(),
            display_target: Some(r.display_target(lang)),
            crystals: (cry > 0).then_some(cry),
            speedup_univ_mins: (univ_mins > 0).then_some(univ_mins),
            speedup_train_mins: (train_mins > 0).then_some(train_mins),
            speedup_heal_mins: (heal_mins > 0).then_some(heal_mins),
            total_rss: (total_rss > 0).then_some(total_rss),
            tome_exp: (tome_exp > 0).then_some(tome_exp),
            covenant_books: (books > 0).then_some(books),
            arrows: (arrows > 0).then_some(arrows),
            choice_packs: (choice_packs > 0).then_some(choice_packs),
            gems: (gems > 0).then_some(gems),
        }
    }
}

pub fn localize_item_stat_name(
    itype: u32,
    sub_type: u32,
    name_vi: &str,
    name_en: &str,
    lang: AppLanguage,
) -> String {
    match lang {
        AppLanguage::Vietnamese => name_vi.to_string(),
        AppLanguage::English => {
            let trimmed = name_en.trim();
            let effective = if !trimmed.is_empty() {
                trimmed
            } else {
                name_vi.trim()
            };
            match effective {
                "Thực phẩm" => "Food".to_string(),
                "Gỗ" => "Wood".to_string(),
                "Đá" => "Stone".to_string(),
                "Vàng" => "Gold".to_string(),
                "Đá quý" => "Gems".to_string(),
                "Tài nguyên khác" => "Other Resources".to_string(),
                other => {
                    if other == name_vi.trim() && itype == 1 {
                        match sub_type {
                            1 => "Food".to_string(),
                            2 => "Wood".to_string(),
                            3 => "Stone".to_string(),
                            4 => "Gold".to_string(),
                            5 => "Gems".to_string(),
                            9 => "Crystals".to_string(),
                            _ => other.to_string(),
                        }
                    } else {
                        other.to_string()
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStatDto {
    pub itype: u32,
    pub sub_type: u32,
    pub name_vi: String,
    pub name_en: String,
    pub display_name: String,
    pub category: String,
    pub total_quantity: i64,
    pub converted_value: Option<String>,
    pub battle_count: usize,
    pub drop_rate: f64,
    pub avg_per_battle: f64,
    pub max_in_battle: i64,
    pub by_battle_type: std::collections::BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummaryDto {
    pub total_val: i64,
    pub food_val: i64,
    pub wood_val: i64,
    pub stone_val: i64,
    pub gold_val: i64,
    pub gem_val: i64,
    pub food_packs: i64,
    pub wood_packs: i64,
    pub stone_packs: i64,
    pub gold_packs: i64,
    pub gem_packs: i64,
    pub choice_packs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialsSummaryDto {
    pub leather_total: i64,
    pub iron_total: i64,
    pub ebony_total: i64,
    pub bone_total: i64,
    pub total_normal: i64,
    pub total_advanced: i64,
    pub total_converted: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApBottlesSummaryDto {
    pub ap_50: i64,
    pub ap_100: i64,
    pub total_bottles: i64,
    pub total_points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDto {
    pub total_battles: usize,
    pub total_crystals: i64,
    pub avg_crystals: f64,
    pub speedup_total_mins: i64,
    pub speedup_univ_mins: i64,
    pub speedup_train_mins: i64,
    pub speedup_heal_mins: i64,
    pub total_resources: i64,
    pub motte_count: usize,
    pub fort_count: usize,
    pub barb_count: usize,
    pub gathering_count: usize,
    pub ap_economics: ApEconomics,
    pub top_items: Vec<ItemStatDto>,
    pub latest_battles: Vec<BattleRecordDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedChipDto {
    pub id: String,
    pub label: String,
    pub display_val: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayRunsDto {
    pub motte: usize,
    pub fort: usize,
    pub barb: usize,
    pub gather: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPayloadDto {
    pub records: Vec<BattleRecordDto>,
    pub config: OverlayConfig,
    pub pinned_chips: Vec<PinnedChipDto>,
    pub today_runs: TodayRunsDto,
    pub last_mail_timestamp: Option<String>,
    pub rok_window_detected: bool,
    pub idle_secs: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OverlayConfigUpdateDto {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub mode: Option<OverlayDisplayMode>,
    #[serde(default)]
    pub filter: Option<FilterMode>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub text_color_hex: Option<String>,
    #[serde(default)]
    pub accent_color_hex: Option<String>,
    #[serde(default)]
    pub offset_x: Option<f32>,
    #[serde(default)]
    pub offset_y: Option<f32>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub pinned_items: Option<Vec<PinnedItemKey>>,
    #[serde(default)]
    pub hotkey_enabled: Option<bool>,
    #[serde(default)]
    pub transparent_bg: Option<bool>,
    #[serde(default)]
    pub fade_timeout_secs: Option<u32>,
    #[serde(default)]
    pub fade_style: Option<OverlayFadeStyle>,
    #[serde(default)]
    pub fade_dim_opacity: Option<f32>,
}
