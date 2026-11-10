import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  getMockSnapshot,
  getMockDashboard,
  getMockBattles,
  getMockInventory,
  getMockOverlay,
} from "./mockData";
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

export async function cancelGoogleLogin(): Promise<void> {
  if (!isTauri()) return;
  await invoke("cancel_google_login");
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

export interface CloudOperationResult {
  success: boolean;
  count: number;
  error?: string | null;
}

export function onCloudPushResult(callback: (res: CloudOperationResult) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<CloudOperationResult>("cloud://push-result", (e) => callback(e.payload));
}

export function onCloudPullResult(callback: (res: CloudOperationResult) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<CloudOperationResult>("cloud://pull-result", (e) => callback(e.payload));
}

export function onOverlayUpdated(callback: (payload: OverlayPayloadDto) => void): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<OverlayPayloadDto>("overlay://updated", (e) => callback(e.payload));
}

