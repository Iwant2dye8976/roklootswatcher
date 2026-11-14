export type FilterMode =
  | "all-targets"
  | "motte"
  | "fort"
  | "barbarian"
  | "all-forts"
  | "all-rewards"
  | "gathering"
  | "pvp";

export type DateFilterPreset =
  | "AllTime"
  | "Today"
  | "Yesterday"
  | "Last3Days"
  | "Last7Days"
  | "Last14Days"
  | "Last15Days"
  | "Last30Days"
  | "Custom";

export type SpeedupUnit = "minutes" | "hours" | "days" | "dhm";
export type AppLanguage = "vi" | "en";
export type AppTheme = "dark" | "light";
export type CloseBehavior = "tray" | "close";
export type OverlayDisplayMode = "Cards" | "MiniHud" | "Combined" | "Hidden";

export type PinnedItemKey =
  | "SpeedupUniversal"
  | "SpeedupTraining"
  | "SpeedupHealing"
  | {
      CatalogItem: {
        itype: number;
        isub: number;
        canonical_name: string;
      };
    };

export interface LootItem {
  name_vi: string;
  name_en: string;
  display_name?: string;
  category?: string;
  itype: number;
  sub_type: number;
  value: number;
  converted_value?: string;
  icon?: string;
  sprites?: string[];
}

export interface BattleRecord {
  id: string;
  file_name: string;
  time_raw: number;
  time_vn: string;
  time_utc: string;
  target: string;
  target_level: string;
  battle_type: "motte" | "fort" | "barbarian" | "gathering" | string;
  damage_pct: number;
  plunder_tier: number;
  talent_add: number;
  pos_x: number;
  pos_y: number;
  account_uid: string;
  account_name: string;
  items: LootItem[];
  display_target?: string;
  crystals?: number;
  tome_exp?: number;
  covenant_books?: number;
  arrows?: number;
  speedup_univ_mins?: number;
  speedup_train_mins?: number;
  speedup_heal_mins?: number;
  total_rss?: number;
  choice_packs?: number;
  gems?: number;
  pvp_combat_type?: string;
  primary_commander_id?: number;
  primary_commander_name?: string;
  secondary_commander_id?: number;
  secondary_commander_name?: string;
  target_primary_commander_id?: number;
  target_primary_commander_name?: string;
  target_secondary_commander_id?: number;
  target_secondary_commander_name?: string;
  primary_is_expertised?: boolean;
  secondary_is_expertised?: boolean;
  target_primary_is_expertised?: boolean;
  target_secondary_is_expertised?: boolean;
  target_player_name?: string;
  kills_dealt?: number;
  kills_taken?: number;
  kp_gained?: number;
  kp_lost?: number;
  trade_ratio_pct?: number;
  dead_dealt?: number;
  dead_taken?: number;
  severely_wounded_dealt?: number;
  severely_wounded_taken?: number;
}

export interface DailyApPoint {
  date_label: string;
  ap_consumed: number;
  speedup_hours: number;
}

export interface TargetBenchmark {
  target_name: string;
  unit_ap_cost: number;
  battle_count: number;
  speedup_mins_per_1k_ap: number;
  covenant_drop_rate: number;
  tier_badge: string;
  battle_type?: string;
  level?: number;
}

export interface ApEconomics {
  total_ap_consumed: number;
  total_ap_recovered: number;
  net_ap_spent: number;
  rebate_pct: number;
  speedup_mins_per_1k_ap: number;
  gems_per_1k_ap: number;
  daily_timeline: DailyApPoint[];
  target_benchmarks: TargetBenchmark[];
}

export interface ItemStatDto {
  itype: number;
  sub_type: number;
  name_vi: string;
  name_en: string;
  display_name?: string;
  category?: string;
  total_quantity: number;
  converted_value?: string;
  battle_count: number;
  drop_rate: number;
  avg_per_battle: number;
  max_in_battle: number;
  by_battle_type: Record<string, number>;
  sprites?: string[];
  icon?: string;
}

export interface ResourceSummaryDto {
  total_val: number;
  food_val: number;
  wood_val: number;
  stone_val: number;
  gold_val: number;
  gem_val: number;
  food_packs: number;
  wood_packs: number;
  stone_packs: number;
  gold_packs: number;
  gem_packs: number;
  choice_packs: number;
}

export interface MaterialsSummaryDto {
  leather_total: number;
  iron_total: number;
  ebony_total: number;
  bone_total: number;
  total_normal: number;
  total_advanced: number;
  total_converted: number;
}

export interface ApBottlesSummaryDto {
  ap_50: number;
  ap_100: number;
  total_bottles: number;
  total_points: number;
}

export interface PvpTimelinePoint {
  label: string;
  timestamp: number;
  kills_dealt: number;
  kills_taken: number;
  kills_t4_dealt: number;
  kills_t5_dealt: number;
  kills_t4_taken: number;
  kills_t5_taken: number;
  kp_gained: number;
  kp_lost: number;
  trade_ratio_pct: number;
  battle_count: number;
}

export interface PvpDashboardData {
  total_battles: number;
  kills_dealt: number;
  kills_taken: number;
  kills_t4_dealt: number;
  kills_t5_dealt: number;
  kills_t4_taken: number;
  kills_t5_taken: number;
  kp_gained: number;
  kp_lost: number;
  trade_ratio_pct: number;
  top_march?: TroopTradeSummaryDto;
  timeline: PvpTimelinePoint[];
  is_hourly: boolean;
}

export interface DashboardDto {
  total_battles: number;
  total_crystals: number;
  avg_crystals: number;
  speedup_total_mins: number;
  speedup_univ_mins: number;
  speedup_train_mins: number;
  speedup_heal_mins: number;
  total_resources: number;
  motte_count: number;
  fort_count: number;
  barb_count: number;
  gathering_count: number;
  pvp_count?: number;
  ap_economics: ApEconomics;
  top_items: ItemStatDto[];
  latest_battles: BattleRecord[];
  pvp_stats?: PvpDashboardData;
}

export type OverlayFadeStyle = "Dim" | "Hide";

export interface OverlayConfig {
  enabled: boolean;
  mode: OverlayDisplayMode;
  filter: FilterMode;
  opacity: number;
  text_color_hex: string;
  accent_color_hex: string;
  offset_x: number;
  offset_y: number;
  width: number;
  font_size: number;
  pinned_items: PinnedItemKey[];
  hotkey_enabled: boolean;
  transparent_bg?: boolean;
  fade_timeout_secs?: number;
  fade_style?: OverlayFadeStyle;
  fade_dim_opacity?: number;
  card_count?: number;
}

export interface PinnedChipDto {
  id: string;
  label: string;
  display_val: string;
  color_hex: string;
}

export interface TroopTradeSummaryDto {
  troop_id: string;
  display_pair_name: string;
  primary_commander_id?: number;
  primary_commander_name: string;
  secondary_commander_id?: number;
  secondary_commander_name: string;
  primary_is_expertised?: boolean;
  secondary_is_expertised?: boolean;
  combat_type?: "open-field" | "rally" | "garrison" | string;
  kills_dealt?: number;
  kills_taken?: number;
  kp_gained: number;
  kp_lost: number;
  trade_ratio_pct: number;
  battle_count: number;
}

export interface OverlayPayloadDto {
  records: BattleRecord[];
  config: OverlayConfig;
  pinned_chips: PinnedChipDto[];
  troop_trades?: TroopTradeSummaryDto[];
  today_runs: {
    motte: number;
    fort: number;
    barb: number;
    gather: number;
    pvp?: number;
    total: number;
  };
  last_mail_timestamp?: string;
  rok_window_detected: boolean;
  idle_secs: number;
}

export interface CloudStatusDto {
  logged_in: boolean;
  email?: string;
  last_sync_time?: string;
  pending_count: number;
  is_syncing: boolean;
  is_restoring: boolean;
  sync_error?: string;
  auto_sync: boolean;
  backup_interval_minutes: number;
  backup_on_exit: boolean;
  stay_login: boolean;
  login_in_progress: boolean;
}

export interface AppSnapshotDto {
  version: string;
  mail_dir: string;
  mode: FilterMode;
  date_preset: DateFilterPreset;
  date_from: string;
  date_to: string;
  account_uid?: string;
  available_accounts: Array<{ uid: string; name: string }>;
  detected_accounts: Array<{ uid: string; name: string }>;
  non_main_records_count: number;
  main_account_uids: string[];
  ignore_non_main_reports: boolean;
  theme: AppTheme;
  language: AppLanguage;
  speedup_unit: SpeedupUnit;
  close_behavior: CloseBehavior;
  is_watching: boolean;
  is_scanning: boolean;
  watcher_status_text: string;
  vault_count: number;
  auto_scan: boolean;
  autostart: boolean;
  autostart_minimized: boolean;
  overlay_config: OverlayConfig;
  cloud_status: CloudStatusDto;
}
