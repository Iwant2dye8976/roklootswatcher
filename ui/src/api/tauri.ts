import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  AppSnapshotDto,
  DashboardDto,
  BattleRecord,
  ItemStatDto,
  ResourceSummaryDto,
  MaterialsSummaryDto,
  ApBottlesSummaryDto,
  OverlayPayloadDto,
  OverlayConfig,
  CloudStatusDto,
  FilterMode,
  DateFilterPreset,
  SpeedupUnit,
  AppLanguage,
  AppTheme,
  CloseBehavior,
} from "../types";

export const isTauri = () => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

export async function getInitialState(): Promise<AppSnapshotDto> {
  if (!isTauri()) {
    return getMockSnapshot();
  }
  return await invoke<AppSnapshotDto>("get_initial_state");
}

export async function setFilterState(
  mode: FilterMode,
  datePreset: DateFilterPreset,
  dateFrom: string,
  dateTo: string,
  accountUid?: string,
  speedupUnit?: SpeedupUnit
): Promise<DashboardDto> {
  if (!isTauri()) return getMockDashboard();
  return await invoke<DashboardDto>("set_filter_state", {
    mode,
    datePreset,
    dateFrom,
    dateTo,
    accountUid: accountUid || null,
    speedupUnit: speedupUnit || null,
  });
}

export async function setPreferences(
  theme: AppTheme,
  language: AppLanguage,
  closeBehavior: CloseBehavior
): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_preferences", { theme, language, closeBehavior });
}

export async function getDashboardData(): Promise<DashboardDto> {
  if (!isTauri()) return getMockDashboard();
  return await invoke<DashboardDto>("get_dashboard_data");
}

export async function queryBattles(
  search: string,
  sortCol?: string,
  sortDir?: "asc" | "desc"
): Promise<BattleRecord[]> {
  if (!isTauri()) return getMockBattles();
  return await invoke<BattleRecord[]>("query_battles", {
    search,
    sortCol: sortCol || null,
    sortDir: sortDir || null,
  });
}

export async function queryInventory(
  search: string,
  category: string,
  sortCol?: string,
  sortDir?: "asc" | "desc"
): Promise<{
  items: ItemStatDto[];
  resources: ResourceSummaryDto;
  materials: MaterialsSummaryDto;
  apBottles: ApBottlesSummaryDto;
}> {
  if (!isTauri()) return getMockInventory();
  return await invoke<{
    items: ItemStatDto[];
    resources: ResourceSummaryDto;
    materials: MaterialsSummaryDto;
    apBottles: ApBottlesSummaryDto;
  }>("query_inventory", {
    search,
    category,
    sortCol: sortCol || null,
    sortDir: sortDir || null,
  });
}

export async function triggerRescan(): Promise<void> {
  if (!isTauri()) return;
  await invoke("trigger_rescan");
}

export async function triggerForceRescan(): Promise<void> {
  if (!isTauri()) return;
  await invoke("trigger_force_rescan");
}

export async function setAutoScan(enabled: boolean): Promise<boolean> {
  if (!isTauri()) return enabled;
  return await invoke<boolean>("set_auto_scan", { enabled });
}

export async function toggleWatcherPause(): Promise<boolean> {
  if (!isTauri()) return true;
  return await invoke<boolean>("toggle_watcher_pause");
}

export async function pickMailDirectory(): Promise<string | null> {
  if (!isTauri()) return null;
  return await invoke<string | null>("pick_mail_directory");
}

export async function setMailDirectory(path: string): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_mail_directory", { path });
}

export async function resetDefaultMailDirectory(): Promise<string> {
  if (!isTauri()) return "C:\\Program Files (x86)\\Rise of Kingdoms\\Rise of Kingdoms Game\\save\\mailcache";
  return await invoke<string>("reset_default_mail_directory");
}

export async function openDirectory(kind: "mailcache" | "vault" | "exe"): Promise<void> {
  if (!isTauri()) return;
  await invoke("open_directory", { kind });
}

export async function setMainAccounts(uids: string[]): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_main_accounts", { uids });
}

export async function setIgnoreNonMainReports(enabled: boolean): Promise<boolean> {
  if (!isTauri()) return enabled;
  return await invoke<boolean>("set_ignore_non_main_reports", { enabled });
}

export async function deleteSecondaryRecords(): Promise<number> {
  if (!isTauri()) return 0;
  return await invoke<number>("delete_secondary_records");
}

export async function exportActiveData(format: "csv" | "json"): Promise<string | null> {
  if (!isTauri()) return "test_export." + format;
  return await invoke<string | null>("export_active_data", { format });
}

export async function getClipboardSummary(): Promise<string> {
  if (!isTauri()) return "RoK Loot Summary";
  return await invoke<string>("get_clipboard_summary");
}

export async function exportVaultBackup(): Promise<string | null> {
  if (!isTauri()) return "rok_vault_backup.json";
  return await invoke<string | null>("export_vault_backup");
}

export async function importVaultBackup(): Promise<number | null> {
  if (!isTauri()) return 0;
  return await invoke<number | null>("import_vault_backup");
}

export async function cleanBinaryVault(): Promise<{ deleted_files: number; freed_bytes: number }> {
  if (!isTauri()) return { deleted_files: 0, freed_bytes: 0 };
  return await invoke<{ deleted_files: number; freed_bytes: number }>("clean_binary_vault");
}

export async function getOverlayState(): Promise<OverlayPayloadDto> {
  if (!isTauri()) return getMockOverlay();
  return await invoke<OverlayPayloadDto>("get_overlay_state");
}

export async function updateOverlayConfig(config: Partial<OverlayConfig>): Promise<void> {
  if (!isTauri()) return;
  await invoke("update_overlay_config", { config });
}

export async function getCloudStatus(): Promise<CloudStatusDto> {
  if (!isTauri()) {
    return {
      logged_in: false,
      pending_count: 0,
      is_syncing: false,
      is_restoring: false,
      auto_sync: true,
      backup_interval_minutes: 30,
      backup_on_exit: true,
      stay_login: true,
      login_in_progress: false,
    };
  }
  return await invoke<CloudStatusDto>("get_cloud_status");
}

export async function startGoogleLogin(): Promise<void> {
  if (!isTauri()) return;
  await invoke("start_google_login");
}

export async function cloudLogout(): Promise<void> {
  if (!isTauri()) return;
  await invoke("cloud_logout");
}

export async function triggerCloudPush(): Promise<void> {
  if (!isTauri()) return;
  await invoke("trigger_cloud_push");
}

export async function triggerCloudPull(): Promise<void> {
  if (!isTauri()) return;
  await invoke("trigger_cloud_pull");
}

export async function setCloudAutoSync(enabled: boolean): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_cloud_auto_sync", { enabled });
}

export async function setCloudConfig(opts: {
  autoBackup?: boolean;
  backupIntervalMinutes?: number;
  backupOnExit?: boolean;
  stayLogin?: boolean;
}): Promise<void> {
  if (!isTauri()) return;
  await invoke("set_cloud_config", {
    autoBackup: opts.autoBackup ?? null,
    backupIntervalMinutes: opts.backupIntervalMinutes ?? null,
    backupOnExit: opts.backupOnExit ?? null,
    stayLogin: opts.stayLogin ?? null,
  });
}

export async function setAutostart(enabled: boolean, minimized?: boolean): Promise<boolean> {
  if (!isTauri()) return enabled;
  return await invoke<boolean>("set_autostart", {
    enabled,
    minimized: minimized ?? null,
  });
}

export async function windowMinimizeToTray(): Promise<void> {
  if (!isTauri()) return;
  await invoke("window_minimize_to_tray");
}

export async function windowMinimize(): Promise<void> {
  if (!isTauri()) return;
  await invoke("window_minimize");
}

export async function windowMaximize(): Promise<void> {
  if (!isTauri()) return;
  await invoke("window_maximize");
}

export async function windowClose(): Promise<void> {
  if (!isTauri()) return;
  await invoke("window_close");
}

export function onWatcherUpdated(callback: () => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen("watcher://updated", callback);
}

export function onWatcherStatus(
  callback: (payload: { status: string; is_watching: boolean; is_scanning?: boolean }) => void
): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<{ status: string; is_watching: boolean; is_scanning?: boolean }>(
    "watcher://status",
    (e) => callback(e.payload)
  );
}

export function onWatcherLog(callback: (logLine: string) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<string>("watcher://log", (e) => callback(e.payload));
}

export function onCloudUpdated(callback: (status: CloudStatusDto) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<CloudStatusDto>("cloud://updated", (e) => callback(e.payload));
}

export function onOverlayUpdated(callback: (payload: OverlayPayloadDto) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<OverlayPayloadDto>("overlay://updated", (e) => callback(e.payload));
}

function getMockSnapshot(): AppSnapshotDto {
  return {
    version: "2.6.4",
    mail_dir: "C:\\Program Files\\Rise of Kingdoms\\mailcache",
    mode: "all-targets",
    date_preset: "Last7Days",
    date_from: "",
    date_to: "",
    account_uid: "102938475",
    available_accounts: [
      { uid: "102938475", name: "Governor 1 [Prime]" },
      { uid: "998877665", name: "Farm Account 2" },
    ],
    detected_accounts: [
      { uid: "102938475", name: "Governor 1 [Prime]" },
      { uid: "998877665", name: "Farm Account 2" },
    ],
    non_main_records_count: 14,
    main_account_uids: ["102938475"],
    ignore_non_main_reports: true,
    theme: "dark",
    language: "vi",
    speedup_unit: "hours",
    close_behavior: "tray",
    is_watching: true,
    is_scanning: false,
    watcher_status_text: "Đang theo dõi",
    vault_count: 1482,
    auto_scan: true,
    autostart: true,
    autostart_minimized: true,
    overlay_config: {
      enabled: true,
      mode: "Combined",
      filter: "all-rewards",
      opacity: 0.9,
      text_color_hex: "#F8FAFC",
      accent_color_hex: "#38BDF8",
      offset_x: 16,
      offset_y: 60,
      width: 240,
      font_size: 10.5,
      pinned_items: [
        { CatalogItem: { itype: 1, isub: 2, canonical_name: "Gems" } },
        { CatalogItem: { itype: 2, isub: 3, canonical_name: "Book of Covenant" } },
        { CatalogItem: { itype: 2, isub: 4, canonical_name: "Arrow of Resistance" } },
        "SpeedupUniversal",
      ],
      hotkey_enabled: true,
    },
    cloud_status: {
      logged_in: true,
      email: "governor@rok.gg",
      last_sync_time: "2026-11-08 20:00:00",
      pending_count: 0,
      is_syncing: false,
      is_restoring: false,
      auto_sync: true,
      backup_interval_minutes: 30,
      backup_on_exit: true,
      stay_login: true,
      login_in_progress: false,
    },
  };
}

function getMockDashboard(): DashboardDto {
  return {
    total_battles: 1482,
    total_crystals: 68540,
    avg_crystals: 46.2,
    speedup_total_mins: 84600,
    speedup_univ_mins: 42000,
    speedup_train_mins: 28400,
    speedup_heal_mins: 14200,
    total_resources: 128500000,
    motte_count: 312,
    fort_count: 450,
    barb_count: 680,
    gathering_count: 40,
    ap_economics: {
      total_ap_consumed: 142000,
      total_ap_recovered: 28400,
      net_ap_spent: 113600,
      rebate_pct: 20.0,
      speedup_mins_per_1k_ap: 595,
      gems_per_1k_ap: 48,
      daily_timeline: [
        { date_label: "02/02", ap_consumed: 18000, speedup_hours: 180 },
        { date_label: "02/03", ap_consumed: 22000, speedup_hours: 215 },
        { date_label: "Today", ap_consumed: 12500, speedup_hours: 130 },
      ],
      target_benchmarks: [
        {
          target_name: "Motte Tier 5",
          unit_ap_cost: 150,
          battle_count: 312,
          speedup_mins_per_1k_ap: 620,
          covenant_drop_rate: 34.5,
          tier_badge: "TIER S+",
        },
      ],
    },
    top_items: [
      {
        itype: 1,
        sub_type: 9,
        name_vi: "Pha Lê",
        name_en: "Crystals",
        display_name: "Crystals",
        total_quantity: 68540,
        converted_value: "68.5k",
        battle_count: 762,
        drop_rate: 51.4,
        avg_per_battle: 89.9,
        max_in_battle: 150,
        by_battle_type: { motte: 42000, fort: 26540 },
      },
    ],
    latest_battles: getMockBattles(),
  };
}

function getMockBattles(): BattleRecord[] {
  return [
    {
      id: "b_001",
      file_name: "mail_001.bin",
      time_raw: 1731068000000000,
      time_vn: "2026-11-08 19:40:00",
      time_utc: "2026-11-08 12:40:00",
      target: "Motte Tier 5",
      target_level: "5",
      battle_type: "motte",
      damage_pct: 100,
      plunder_tier: 2,
      talent_add: 0,
      pos_x: 624,
      pos_y: 812,
      account_uid: "102938475",
      account_name: "Governor 1 [Prime]",
      items: [
        { name_vi: "Pha Lê", name_en: "Crystals", display_name: "Crystals", itype: 1, sub_type: 9, value: 150 },
      ],
      crystals: 150,
      speedup_univ_mins: 180,
    },
  ];
}

function getMockInventory() {
  return {
    items: [],
    resources: {
      total_val: 0,
      food_val: 0,
      wood_val: 0,
      stone_val: 0,
      gold_val: 0,
      gem_val: 0,
      food_packs: 0,
      wood_packs: 0,
      stone_packs: 0,
      gold_packs: 0,
      gem_packs: 0,
      choice_packs: 0,
    },
    materials: {
      leather_total: 0,
      iron_total: 0,
      ebony_total: 0,
      bone_total: 0,
      total_normal: 0,
      total_advanced: 0,
      total_converted: 0,
    },
    apBottles: {
      ap_50: 0,
      ap_100: 0,
      total_bottles: 0,
      total_points: 0,
    },
  };
}

function getMockOverlay(): OverlayPayloadDto {
  return {
    records: getMockBattles(),
    config: {
      enabled: true,
      mode: "Combined",
      filter: "all-rewards",
      opacity: 0.9,
      text_color_hex: "#F8FAFC",
      accent_color_hex: "#38BDF8",
      offset_x: 16,
      offset_y: 60,
      width: 240,
      font_size: 10.5,
      pinned_items: ["SpeedupUniversal"],
      hotkey_enabled: true,
    },
    pinned_chips: [
      { id: "SpeedupUniversal", label: "SPEEDUP", display_val: "24h 30m", color_hex: "#38bdf8" },
    ],
    today_runs: { motte: 10, fort: 5, barb: 12, gather: 2, total: 29 },
    last_mail_timestamp: "2026-11-08 19:40:00",
    rok_window_detected: true,
    idle_secs: 0,
  };
}
