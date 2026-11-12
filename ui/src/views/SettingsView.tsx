import React, { useState, useEffect, useRef } from "react";
import {
  Folder,
  FolderOpen,
  RotateCcw,
  RefreshCw,
  Sliders,
  Cloud,
  Terminal,
  Trash2,
  LogIn,
  LogOut,
  Loader2,
  ArrowUpCircle,
  ArrowDownCircle,
  Power,
  Users,
  Plus,
  X,
  Database,
  Download,
  Upload,
  Sparkles,
  Check,
  AlertCircle,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useConfirm } from "../components/ConfirmDialog";
import { useI18n } from "../i18n";
import {
  FilterMode,
  OverlayConfig,
  OverlayDisplayMode,
  PinnedItemKey,
} from "../types";
import { ResourceIcon } from "../components/ResourceIcon";
import * as api from "../api/tauri";

interface PinnedPreset {
  id: string;
  labelEn: string;
  labelVi: string;
  key: PinnedItemKey;
}

const PINNED_PRESETS: PinnedPreset[] = [
  {
    id: "gems",
    labelEn: "Gems",
    labelVi: "Đá Quý",
    key: { CatalogItem: { itype: 1, isub: 2, canonical_name: "Gems" } },
  },
  {
    id: "crystals",
    labelEn: "Crystals",
    labelVi: "Pha Lê",
    key: { CatalogItem: { itype: 1, isub: 9, canonical_name: "Crystals" } },
  },
  {
    id: "books",
    labelEn: "Covenant Books",
    labelVi: "Sách Giao Ước",
    key: {
      CatalogItem: { itype: 2, isub: 3, canonical_name: "Book of Covenant" },
    },
  },
  {
    id: "arrows",
    labelEn: "Arrows",
    labelVi: "Tên Kháng Cự",
    key: {
      CatalogItem: { itype: 2, isub: 4, canonical_name: "Arrow of Resistance" },
    },
  },
  {
    id: "spd_univ",
    labelEn: "Univ Speedup",
    labelVi: "Tốc Chung",
    key: "SpeedupUniversal",
  },
  {
    id: "spd_train",
    labelEn: "Train Speedup",
    labelVi: "Tốc Lính",
    key: "SpeedupTraining",
  },
  {
    id: "spd_heal",
    labelEn: "Heal Speedup",
    labelVi: "Tốc Viện",
    key: "SpeedupHealing",
  },
];

function isSamePinnedKey(a?: PinnedItemKey | null, b?: PinnedItemKey | null): boolean {
  if (!a || !b) return false;
  if (typeof a === "string" && typeof b === "string") {
    return a === b;
  }
  if (
    typeof a === "object" &&
    typeof b === "object" &&
    "CatalogItem" in a &&
    "CatalogItem" in b
  ) {
    return (
      a.CatalogItem?.canonical_name?.toLowerCase() ===
      b.CatalogItem?.canonical_name?.toLowerCase()
    );
  }
  return false;
}

function normalizeUid(uid: string): string {
  return uid.replace(/^player_/, "").trim();
}

const DEFAULT_OVERLAY_CFG: OverlayConfig = {
  enabled: false,
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
    {
      CatalogItem: { itype: 2, isub: 4, canonical_name: "Arrow of Resistance" },
    },
    "SpeedupUniversal",
  ],
  hotkey_enabled: true,
  transparent_bg: false,
  fade_timeout_secs: 0,
  fade_style: "Dim",
  fade_dim_opacity: 0.25,
};

export const SettingsView: React.FC = () => {
  const {
    snapshot,
    language,
    closeBehavior,
    setCloseBehaviorPref,
    consoleLogs,
    clearLogs,
    addToast,
    refreshSnapshot,
    refreshDashboard,
  } = useAppStore();
  const { t } = useI18n(language);
  const { confirm } = useConfirm();

  const [mailPath, setMailPath] = useState(snapshot?.mail_dir || "");
  const [autoScan, setAutoScan] = useState(snapshot?.auto_scan ?? true);
  const [autostart, setAutostart] = useState(snapshot?.autostart ?? false);
  const [autostartMinimized, setAutostartMinimized] = useState(
    snapshot?.autostart_minimized ?? true
  );
  const [mainUids, setMainUids] = useState<string[]>(
    snapshot?.main_account_uids || []
  );
  const [newUidInput, setNewUidInput] = useState("");
  const [isDeleting, setIsDeleting] = useState(false);
  const [vaultBusy, setVaultBusy] = useState(false);

  const [overlayCfg, setOverlayCfg] = useState<OverlayConfig>(
    snapshot?.overlay_config || DEFAULT_OVERLAY_CFG
  );

  const [cloudInterval, setCloudInterval] = useState<number>(
    snapshot?.cloud_status?.backup_interval_minutes ?? 30
  );

  const [autoScroll, setAutoScroll] = useState(true);
  const logContainerRef = useRef<HTMLDivElement>(null);
  const lastFadeTimeoutRef = useRef<number>(
    (overlayCfg.fade_timeout_secs ?? 0) > 0 ? (overlayCfg.fade_timeout_secs ?? 30) : 30
  );
  if ((overlayCfg.fade_timeout_secs ?? 0) > 0) {
    lastFadeTimeoutRef.current = overlayCfg.fade_timeout_secs!;
  }

  useEffect(() => {
    if (snapshot?.mail_dir !== undefined) {
      setMailPath(snapshot.mail_dir);
    }
  }, [snapshot?.mail_dir]);

  useEffect(() => {
    if (!snapshot) return;
    setAutoScan(snapshot.auto_scan);
    setAutostart(snapshot.autostart);
    setAutostartMinimized(snapshot.autostart_minimized);
    setMainUids(snapshot.main_account_uids || []);
    if (snapshot.overlay_config) {
      setOverlayCfg(snapshot.overlay_config);
    }
    if (snapshot.cloud_status) {
      setCloudInterval(snapshot.cloud_status.backup_interval_minutes);
    }
  }, [snapshot]);

  useEffect(() => {
    if (autoScroll && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [consoleLogs, autoScroll]);

  const mailDirty =
    mailPath.trim().length > 0 &&
    mailPath.trim() !== (snapshot?.mail_dir ?? "");

  const handleApplyMailDir = async () => {
    const trimmed = mailPath.trim();
    if (!trimmed || trimmed === snapshot?.mail_dir) return;
    try {
      await api.setMailDirectory(trimmed);
      await refreshSnapshot();
      await refreshDashboard();
      addToast({
        type: "success",
        text:
          language === "vi"
            ? `Đã cập nhật thư mục mail: ${trimmed}`
            : `Updated mail directory to ${trimmed}`,
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleBrowseMailDir = async () => {
    try {
      const picked = await api.pickMailDirectory();
      if (picked) {
        setMailPath(picked);
        await api.setMailDirectory(picked);
        await refreshSnapshot();
        await refreshDashboard();
        addToast({
          type: "success",
          text:
            language === "vi"
              ? `Đã chọn thư mục: ${picked}`
              : `Updated mail directory to ${picked}`,
        });
      }
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleResetDefault = async () => {
    try {
      const def = await api.resetDefaultMailDirectory();
      setMailPath(def);
      await refreshSnapshot();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đã khôi phục đường dẫn mặc định"
            : "Reset mail directory to default",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleForceRescan = async () => {
    try {
      await api.triggerForceRescan();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đang quét lại toàn bộ tệp mailcache..."
            : "Running full mailcache rescan...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleToggleAutoScan = async (nextVal: boolean) => {
    setAutoScan(nextVal);
    try {
      await api.setAutoScan(nextVal);
      await refreshSnapshot();
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleUpdateAutostart = async (
    nextEnabled: boolean,
    nextMinimized: boolean
  ) => {
    try {
      const res = await api.setAutostart(nextEnabled, nextMinimized);
      setAutostart(res);
      setAutostartMinimized(nextMinimized);
      await refreshSnapshot();
      addToast({
        type: "success",
        text: res
          ? language === "vi"
            ? "Đã cập nhật khởi động cùng Windows"
            : "Updated Windows startup settings"
          : language === "vi"
          ? "Đã tắt khởi động cùng Windows"
          : "Disabled Windows autostart",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleAddMainUid = async (rawUid?: string) => {
    const trimmed = normalizeUid(rawUid ?? newUidInput);
    if (!trimmed) return;
    if (mainUids.some((u) => normalizeUid(u) === trimmed)) {
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "UID này đã có trong danh sách chính"
            : "Account UID already in primary list",
      });
      return;
    }
    const updated = [...mainUids, trimmed];
    setMainUids(updated);
    if (!rawUid) setNewUidInput("");
    try {
      await api.setMainAccounts(updated);
      await refreshSnapshot();
      await refreshDashboard();
      addToast({
        type: "success",
        text:
          language === "vi"
            ? `Đã thêm UID chính: ${trimmed}`
            : `Added UID ${trimmed} to primary list`,
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleRemoveMainUid = async (uidToRemove: string) => {
    const updated = mainUids.filter((u) => u !== uidToRemove);
    setMainUids(updated);
    try {
      await api.setMainAccounts(updated);
      await refreshSnapshot();
      await refreshDashboard();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? `Đã gỡ UID ${uidToRemove}`
            : `Removed UID ${uidToRemove} from primary list`,
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleDeleteSecondary = async () => {
    const nonMainCount = snapshot?.non_main_records_count ?? 0;
    const ok = await confirm({
      title: language === "vi" ? "Xác nhận xóa chiến báo phụ" : "Delete Secondary Records",
      message:
        language === "vi"
          ? `Xóa vĩnh viễn ${nonMainCount} chiến báo không thuộc UID chính? Thao tác này không thể hoàn tác.`
          : `Permanently delete ${nonMainCount} non-primary account records? This cannot be undone.`,
      confirmLabel: language === "vi" ? "Xóa vĩnh viễn" : "Delete Records",
      cancelLabel: language === "vi" ? "Hủy" : "Cancel",
      isDestructive: true,
    });
    if (!ok) {
      return;
    }
    setIsDeleting(true);
    try {
      const count = await api.deleteSecondaryRecords();
      await refreshSnapshot();
      await refreshDashboard();
      addToast({
        type: "success",
        text:
          language === "vi"
            ? `Đã xóa ${count} chiến báo phụ`
            : `Deleted ${count} non-primary records`,
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    } finally {
      setIsDeleting(false);
    }
  };

  const handleExportVault = async () => {
    setVaultBusy(true);
    try {
      const saved = await api.exportVaultBackup();
      if (saved) {
        addToast({
          type: "success",
          text:
            language === "vi"
              ? `Đã sao lưu Vault: ${saved}`
              : `Exported Vault backup: ${saved}`,
        });
      }
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    } finally {
      setVaultBusy(false);
    }
  };

  const handleImportVault = async () => {
    setVaultBusy(true);
    try {
      const imported = await api.importVaultBackup();
      if (imported !== null && imported !== undefined) {
        await refreshSnapshot();
        await refreshDashboard();
        addToast({
          type: "success",
          text:
            language === "vi"
              ? `Đã nhập ${imported} chiến báo vào Vault`
              : `Imported ${imported} records into Vault`,
        });
      }
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    } finally {
      setVaultBusy(false);
    }
  };

  const handleCleanBinaries = async () => {
    setVaultBusy(true);
    try {
      const res = await api.cleanBinaryVault();
      const kb = (res.freed_bytes / 1024).toFixed(1);
      addToast({
        type: "success",
        text:
          language === "vi"
            ? `Đã dọn ${res.deleted_files} file rác (${kb} KB)`
            : `Cleaned ${res.deleted_files} orphan files (${kb} KB freed)`,
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    } finally {
      setVaultBusy(false);
    }
  };

  const overlayUpdateTimeoutRef = useRef<number | null>(null);

  const updateOverlay = (updates: Partial<OverlayConfig>, debounceMs = 0) => {
    setOverlayCfg((prev) => ({ ...prev, ...updates }));

    if (overlayUpdateTimeoutRef.current) {
      window.clearTimeout(overlayUpdateTimeoutRef.current);
    }

    if (debounceMs > 0) {
      overlayUpdateTimeoutRef.current = window.setTimeout(async () => {
        try {
          await api.updateOverlayConfig(updates);
        } catch (err) {
          addToast({ type: "error", text: String(err) });
        }
      }, debounceMs);
    } else {
      api.updateOverlayConfig(updates).catch((err) => {
        addToast({ type: "error", text: String(err) });
      });
    }
  };

  const togglePinnedItem = (presetKey: PinnedItemKey) => {
    const current = overlayCfg.pinned_items || [];
    const exists = current.some((k) => isSamePinnedKey(k, presetKey));
    const nextPinned = exists
      ? current.filter((k) => !isSamePinnedKey(k, presetKey))
      : [...current, presetKey];
    updateOverlay({ pinned_items: nextPinned });
  };

  const handleCloudConfigUpdate = async (opts: {
    autoBackup?: boolean;
    backupIntervalMinutes?: number;
    backupOnExit?: boolean;
    stayLogin?: boolean;
  }) => {
    try {
      await api.setCloudConfig(opts);
      await refreshSnapshot();
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleGoogleLogin = async () => {
    try {
      await api.startGoogleLogin();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đã mở trình duyệt đăng nhập Google OAuth..."
            : "Opened Google OAuth sign-in in browser...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleCancelLogin = async () => {
    try {
      await api.cancelGoogleLogin();
      addToast({
        type: "info",
        text: language === "vi" ? "Đã hủy đăng nhập" : "Login cancelled",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleCloudPush = async () => {
    try {
      await api.triggerCloudPush();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đang sao lưu dữ liệu lên Supabase..."
            : "Pushing vault backup to Supabase...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleCloudPull = async () => {
    try {
      await api.triggerCloudPull();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đang tải dữ liệu từ Supabase..."
            : "Restoring records from Supabase...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleCloudLogout = async () => {
    try {
      await api.cloudLogout();
      await refreshSnapshot();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đã đăng xuất tài khoản Cloud"
            : "Signed out of Cloud Sync",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const cloud = snapshot?.cloud_status;
  const unaddedDetectedAccounts = (snapshot?.detected_accounts || []).filter(
    (acc) => !mainUids.some((u) => normalizeUid(u) === normalizeUid(acc.uid))
  );

  const filterOptions: Array<{ value: FilterMode; label: string }> = [
    { value: "all-rewards", label: t("modeAllRewards") },
    { value: "all-targets", label: t("modeAll") },
    { value: "motte", label: t("modeMotte") },
    { value: "fort", label: t("modeFort") },
    { value: "all-forts", label: t("modeAllForts") },
    { value: "barbarian", label: t("modeBarbarian") },
    { value: "gathering", label: t("modeGathering") },
    { value: "pvp", label: t("modePvp") },
  ];

  const overlayModes: Array<{ id: OverlayDisplayMode; label: string }> = [
    { id: "Cards", label: "Full HUD" },
    { id: "MiniHud", label: "Mini HUD" },
    { id: "Combined", label: "Combined" },
    { id: "Hidden", label: "Hidden" },
  ];

  return (
    <div className="flex-1 overflow-y-auto p-5 select-none space-y-4">
      {/* 2-Column Bento Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Card 1: Mailcache Directory */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-3">
          <div className="flex items-center justify-between border-b border-binance-hairline pb-2">
            <div className="flex items-center gap-2">
              <Folder className="w-4 h-4 text-binance-turquoise" />
              <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
                {t("settingsMailTitle")}
              </h3>
            </div>
            <button
              onClick={() =>
                api
                  .openDirectory("mailcache")
                  .catch((e) => addToast({ type: "error", text: String(e) }))
              }
              className="flex items-center gap-1 px-2 py-1 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-[11px] font-mono text-binance-muted hover:text-binance-body transition"
              title={t("btnOpenFolder")}
            >
              <FolderOpen className="w-3.5 h-3.5" />
              <span>{t("btnOpenFolder")}</span>
            </button>
          </div>

          <p className="text-[11px] text-binance-muted">{t("settingsMailDesc")}</p>

          <div className="flex items-center gap-2">
            <input
              type="text"
              value={mailPath}
              onChange={(e) => setMailPath(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleApplyMailDir();
                }
              }}
              className="flex-1 bg-binance-canvas border border-binance-hairline rounded px-3 py-1.5 text-xs text-binance-body font-mono outline-none focus:border-binance-yellow"
            />
            {mailDirty && (
              <button
                onClick={handleApplyMailDir}
                className="flex items-center gap-1 px-2.5 py-1.5 rounded bg-binance-yellowSolid hover:bg-binance-yellowHover text-binance-ink text-xs font-semibold transition"
              >
                <Check className="w-3.5 h-3.5" />
                <span>{t("btnApply")}</span>
              </button>
            )}
            <button
              onClick={handleBrowseMailDir}
              className="px-3 py-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-xs font-medium transition"
            >
              {t("btnBrowse")}
            </button>
            <button
              onClick={handleResetDefault}
              title={t("btnResetDefault")}
              className="p-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-muted hover:text-binance-body transition"
            >
              <RotateCcw className="w-4 h-4" />
            </button>
          </div>

          <div className="flex items-center justify-between pt-1">
            <label className="flex items-center gap-2 cursor-pointer text-xs text-binance-body">
              <input
                type="checkbox"
                checked={autoScan}
                onChange={(e) => handleToggleAutoScan(e.target.checked)}
                className="rounded accent-amber-500"
              />
              <span>{t("autoScanStartup")}</span>
            </label>

            <button
              onClick={handleForceRescan}
              disabled={snapshot?.is_scanning}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-xs font-mono text-binance-body disabled:opacity-50 transition"
            >
              <RefreshCw
                className={`w-3.5 h-3.5 text-binance-yellow ${
                  snapshot?.is_scanning ? "animate-spin" : ""
                }`}
              />
              <span>{t("btnForceScan")}</span>
            </button>
          </div>
        </div>

        {/* Card 2: Cloud Sync (Supabase) */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-3">
          <div className="flex items-center justify-between border-b border-binance-hairline pb-2">
            <div className="flex items-center gap-2">
              <Cloud className="w-4 h-4 text-binance-info" />
              <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
                {t("settingsCloudTitle")}
              </h3>
            </div>
            {cloud?.is_syncing || cloud?.is_restoring ? (
              <span className="text-[10px] font-mono text-binance-yellow bg-binance-yellow/10 px-2 py-0.5 rounded border border-binance-yellow/30">
                {cloud.is_restoring ? "RESTORING" : "SYNCING"}
              </span>
            ) : cloud?.logged_in ? (
              <span className="text-[10px] font-mono text-binance-green bg-binance-green/10 px-2 py-0.5 rounded border border-binance-green/30">
                ONLINE
              </span>
            ) : (
              <span className="text-[10px] font-mono text-binance-muted bg-binance-elevated px-2 py-0.5 rounded border border-binance-hairline">
                OFFLINE
              </span>
            )}
          </div>

          {cloud?.sync_error && (
            <div className="flex items-start gap-2 p-2 rounded bg-binance-red/10 border border-binance-red/30 text-[11px] text-binance-red font-mono">
              <AlertCircle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
              <span className="break-all">{cloud.sync_error}</span>
            </div>
          )}

          {cloud?.logged_in ? (
            <div className="space-y-2.5">
              <div className="grid grid-cols-2 gap-2 text-[11px] font-mono">
                <div className="truncate text-binance-muted">
                  Account:{" "}
                  <span className="text-binance-body font-semibold">
                    {cloud.email || "Connected"}
                  </span>
                </div>
                <div className="text-right text-binance-muted">
                  {t("cloudPending")}{" "}
                  <span className="text-binance-yellow font-semibold">
                    {cloud.pending_count}
                  </span>
                </div>
              </div>
              {cloud.last_sync_time && (
                <div className="text-[11px] font-mono text-binance-muted">
                  {t("cloudLastSync")}{" "}
                  <span className="text-binance-body">{cloud.last_sync_time}</span>
                </div>
              )}

              <div className="flex items-center gap-2">
                <button
                  onClick={handleCloudPush}
                  disabled={cloud.is_syncing || cloud.is_restoring}
                  className="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded bg-binance-elevated hover:opacity-90 text-xs text-binance-body border border-binance-hairline font-medium disabled:opacity-50"
                >
                  <ArrowUpCircle className="w-3.5 h-3.5 text-binance-green" />
                  <span>{t("cloudPush")}</span>
                </button>
                <button
                  onClick={handleCloudPull}
                  disabled={cloud.is_syncing || cloud.is_restoring}
                  className="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded bg-binance-elevated hover:opacity-90 text-xs text-binance-body border border-binance-hairline font-medium disabled:opacity-50"
                >
                  <ArrowDownCircle className="w-3.5 h-3.5 text-binance-info" />
                  <span>{t("cloudPull")}</span>
                </button>
                <button
                  onClick={handleCloudLogout}
                  className="p-1.5 rounded bg-binance-red/10 hover:bg-binance-red/20 text-binance-red border border-binance-red/30"
                  title={t("cloudLogout")}
                >
                  <LogOut className="w-4 h-4" />
                </button>
              </div>
            </div>
          ) : (
            <div className="space-y-2.5">
              {cloud?.login_in_progress ? (
                <div className="flex items-center gap-2">
                  <div className="flex-1 flex items-center justify-center gap-2 py-2 rounded bg-binance-yellowSolid/40 text-binance-ink font-semibold text-xs">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>{t("cloudLoggingIn")}</span>
                  </div>
                  <button
                    onClick={handleCancelLogin}
                    className="px-3 py-2 rounded bg-binance-red/20 hover:bg-binance-red/30 text-binance-red border border-binance-red/40 text-xs font-semibold transition"
                    title={t("cloudCancelLogin")}
                  >
                    {t("cloudCancelLogin")}
                  </button>
                </div>
              ) : (
                <button
                  onClick={handleGoogleLogin}
                  className="w-full flex items-center justify-center gap-2 py-2 rounded bg-binance-yellowSolid hover:bg-binance-yellowHover text-binance-ink font-semibold text-xs transition"
                >
                  <LogIn className="w-4 h-4" />
                  <span>{t("cloudLogin")}</span>
                </button>
              )}
            </div>
          )}

          {/* Cloud Preferences */}
          <div className="pt-2 border-t border-binance-hairline grid grid-cols-1 sm:grid-cols-2 gap-2 text-[11px] text-binance-body">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={cloud?.auto_sync ?? true}
                onChange={(e) =>
                  handleCloudConfigUpdate({ autoBackup: e.target.checked })
                }
                className="rounded accent-amber-500"
              />
              <span>{t("cloudAutoBackup")}</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={cloud?.backup_on_exit ?? true}
                onChange={(e) =>
                  handleCloudConfigUpdate({ backupOnExit: e.target.checked })
                }
                className="rounded accent-amber-500"
              />
              <span>{t("cloudBackupOnExit")}</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={cloud?.stay_login ?? true}
                onChange={(e) =>
                  handleCloudConfigUpdate({ stayLogin: e.target.checked })
                }
                className="rounded accent-amber-500"
              />
              <span>{t("cloudStayLogin")}</span>
            </label>
            <div className="flex items-center gap-1.5 font-mono">
              <span className="text-binance-muted">{t("cloudIntervalLabel")}</span>
              <input
                type="number"
                min={5}
                max={1440}
                value={cloudInterval}
                onChange={(e) =>
                  setCloudInterval(Math.max(5, Number(e.target.value) || 5))
                }
                onBlur={() =>
                  handleCloudConfigUpdate({
                    backupIntervalMinutes: Math.min(
                      1440,
                      Math.max(5, cloudInterval)
                    ),
                  })
                }
                className="w-16 bg-binance-canvas border border-binance-hairline rounded px-1.5 py-0.5 text-xs text-binance-body text-center outline-none"
              />
            </div>
          </div>
        </div>

        {/* Card 3: Windows Startup & Window Behavior */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-3">
          <div className="flex items-center gap-2 border-b border-binance-hairline pb-2">
            <Power className="w-4 h-4 text-binance-green" />
            <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
              {t("settingsWindowStartupTitle")}
            </h3>
          </div>

          <div className="space-y-2 text-xs text-binance-body">
            <label className="flex items-center justify-between cursor-pointer">
              <span>{t("autostartEnable")}</span>
              <input
                type="checkbox"
                checked={autostart}
                onChange={(e) =>
                  handleUpdateAutostart(e.target.checked, autostartMinimized)
                }
                className="rounded accent-amber-500"
              />
            </label>

            <label
              className={`flex items-center justify-between cursor-pointer ${
                !autostart ? "opacity-50" : ""
              }`}
            >
              <span>{t("autostartMinimized")}</span>
              <input
                type="checkbox"
                checked={autostartMinimized}
                disabled={!autostart}
                onChange={(e) =>
                  handleUpdateAutostart(autostart, e.target.checked)
                }
                className="rounded accent-amber-500"
              />
            </label>
          </div>

          <div className="pt-2 border-t border-binance-hairline space-y-1.5">
            <div className="text-[11px] text-binance-muted">
              {t("closeBehaviorLabel")}
            </div>
            <div className="grid grid-cols-2 gap-1.5 text-xs">
              <button
                onClick={() => setCloseBehaviorPref("tray")}
                className={`py-1.5 px-2 rounded border font-medium transition ${
                  closeBehavior === "tray"
                    ? "bg-binance-yellow/15 border-binance-yellow/50 text-binance-yellow"
                    : "bg-binance-canvas border-binance-hairline text-binance-muted hover:text-binance-body"
                }`}
              >
                {t("closeToTray")}
              </button>
              <button
                onClick={() => setCloseBehaviorPref("close")}
                className={`py-1.5 px-2 rounded border font-medium transition ${
                  closeBehavior === "close"
                    ? "bg-binance-yellow/15 border-binance-yellow/50 text-binance-yellow"
                    : "bg-binance-canvas border-binance-hairline text-binance-muted hover:text-binance-body"
                }`}
              >
                {t("closeExitApp")}
              </button>
            </div>
          </div>
        </div>

        {/* Card 4: Primary Account UIDs & Isolation */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-3">
          <div className="flex items-center justify-between border-b border-binance-hairline pb-2">
            <div className="flex items-center gap-2">
              <Users className="w-4 h-4 text-binance-yellow" />
              <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
                {t("settingsAccountsTitle")}
              </h3>
            </div>
            <button
              onClick={handleDeleteSecondary}
              disabled={
                isDeleting ||
                mainUids.length === 0 ||
                (snapshot?.non_main_records_count ?? 0) === 0
              }
              className="flex items-center gap-1 px-2 py-1 rounded bg-binance-red/10 hover:bg-binance-red/20 text-binance-red border border-binance-red/30 text-[10px] font-mono transition disabled:opacity-40"
            >
              <Trash2 className="w-3 h-3" />
              <span>
                {t("pruneNonMain")} ({snapshot?.non_main_records_count ?? 0})
              </span>
            </button>
          </div>

          <div className="flex items-center gap-1.5 flex-wrap min-h-[26px]">
            {mainUids.length === 0 ? (
              <span className="text-[11px] text-binance-muted italic">
                {t("noMainAccounts")}
              </span>
            ) : (
              mainUids.map((uid) => {
                const known = snapshot?.detected_accounts?.find(
                  (a) => normalizeUid(a.uid) === normalizeUid(uid)
                );
                return (
                  <span
                    key={uid}
                    className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-binance-elevated text-binance-body font-mono text-xs border border-binance-hairline"
                  >
                    <span>
                      {known?.name ? `${known.name} (${uid})` : uid}
                    </span>
                    <button
                      onClick={() => handleRemoveMainUid(uid)}
                      className="text-binance-muted hover:text-binance-red"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                );
              })
            )}
          </div>

          {unaddedDetectedAccounts.length > 0 && (
            <div className="space-y-1">
              <div className="text-[10px] text-binance-muted font-mono">
                {t("detectedAccountsHint")}
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {unaddedDetectedAccounts.map((acc) => (
                  <button
                    key={acc.uid}
                    onClick={() => handleAddMainUid(acc.uid)}
                    className="flex items-center gap-1 px-2 py-0.5 rounded bg-binance-canvas hover:bg-binance-elevated border border-binance-hairline text-[11px] font-mono text-binance-muted hover:text-binance-body transition"
                  >
                    <Plus className="w-3 h-3 text-binance-yellow" />
                    <span>
                      {acc.name
                        ? `${acc.name} (${normalizeUid(acc.uid)})`
                        : normalizeUid(acc.uid)}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className="flex items-center gap-2 pt-1">
            <input
              type="text"
              placeholder={t("uidPlaceholder")}
              value={newUidInput}
              onChange={(e) => setNewUidInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  handleAddMainUid();
                }
              }}
              className="flex-1 bg-binance-canvas border border-binance-hairline rounded px-3 py-1.5 text-xs text-binance-body font-mono outline-none focus:border-binance-yellow"
            />
            <button
              onClick={() => handleAddMainUid()}
              className="flex items-center gap-1 px-3 py-1.5 rounded bg-binance-yellow/15 hover:bg-binance-yellow/25 border border-binance-yellow/40 text-binance-yellow text-xs font-medium transition"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>{t("btnAdd")}</span>
            </button>
          </div>

          {/* Ignore non-main reports toggle */}
          <div className="pt-2 border-t border-binance-hairline">
            <label className="flex items-center gap-2 cursor-pointer text-[11px] text-binance-body select-none">
              <input
                type="checkbox"
                checked={snapshot?.ignore_non_main_reports ?? true}
                onChange={async (e) => {
                  try {
                    await api.setIgnoreNonMainReports(e.target.checked);
                    await refreshSnapshot();
                  } catch (err) {
                    addToast({ type: "error", text: String(err) });
                  }
                }}
                className="rounded accent-amber-500"
              />
              <span>{t("ignoreNonMainReports")}</span>
            </label>
          </div>
        </div>

        {/* Card 5: SQLite Vault & Backup Maintenance */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-3 lg:col-span-2">
          <div className="flex items-center justify-between border-b border-binance-hairline pb-2">
            <div className="flex items-center gap-2">
              <Database className="w-4 h-4 text-binance-turquoise" />
              <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
                {t("settingsVaultTitle")}
              </h3>
            </div>
            <span className="text-[11px] font-mono text-binance-muted">
              {t("statusVault")}:{" "}
              <strong className="text-binance-body">
                {(snapshot?.vault_count ?? 0).toLocaleString()}
              </strong>{" "}
              {t("recordsCount")}
            </span>
          </div>

          <p className="text-[11px] text-binance-muted">{t("vaultDesc")}</p>

          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 text-xs">
            <button
              onClick={() =>
                api
                  .openDirectory("vault")
                  .catch((e) => addToast({ type: "error", text: String(e) }))
              }
              className="flex items-center justify-center gap-1.5 py-2 px-3 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body font-medium transition"
            >
              <FolderOpen className="w-3.5 h-3.5 text-binance-turquoise" />
              <span>{t("btnOpenVault")}</span>
            </button>
            <button
              onClick={handleExportVault}
              disabled={vaultBusy}
              className="flex items-center justify-center gap-1.5 py-2 px-3 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body font-medium transition disabled:opacity-50"
            >
              <Download className="w-3.5 h-3.5 text-binance-green" />
              <span>{t("btnExportVault")}</span>
            </button>
            <button
              onClick={handleImportVault}
              disabled={vaultBusy}
              className="flex items-center justify-center gap-1.5 py-2 px-3 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body font-medium transition disabled:opacity-50"
            >
              <Upload className="w-3.5 h-3.5 text-binance-info" />
              <span>{t("btnImportVault")}</span>
            </button>
            <button
              onClick={handleCleanBinaries}
              disabled={vaultBusy}
              className="flex items-center justify-center gap-1.5 py-2 px-3 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body font-medium transition disabled:opacity-50"
            >
              <Sparkles className="w-3.5 h-3.5 text-binance-yellow" />
              <span>{t("btnCleanBinaries")}</span>
            </button>
          </div>
        </div>

        {/* Card 6: HUD Overlay Configuration & D-Pad */}
        <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-4 lg:col-span-2">
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-binance-hairline pb-2">
            <div className="flex items-center gap-2">
              <Sliders className="w-4 h-4 text-binance-yellow" />
              <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
                {t("settingsOverlayTitle")}
              </h3>
            </div>
            <div className="flex items-center gap-4 text-xs">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={overlayCfg.hotkey_enabled ?? true}
                  onChange={(e) =>
                    updateOverlay({ hotkey_enabled: e.target.checked })
                  }
                  className="rounded accent-amber-500"
                />
                <span className="text-binance-muted">{t("overlayHotkey")}</span>
              </label>

              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={overlayCfg.enabled}
                  onChange={(e) => updateOverlay({ enabled: e.target.checked })}
                  className="rounded accent-amber-500"
                />
                <span className="text-binance-body font-semibold">
                  {t("overlayEnable")}
                </span>
              </label>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
            {/* Mode & Filter selection */}
            <div className="space-y-3">
              <div className="space-y-1.5">
                <label className="text-binance-muted font-mono text-[11px]">
                  {t("overlayMode")}
                </label>
                <div className="grid grid-cols-2 gap-1.5">
                  {overlayModes.map((m) => (
                    <button
                      key={m.id}
                      onClick={() => updateOverlay({ mode: m.id })}
                      className={`py-1.5 rounded text-center font-mono text-xs border transition ${
                        overlayCfg.mode === m.id
                          ? "bg-binance-yellow/15 text-binance-yellow border-binance-yellow/50 font-bold"
                          : "bg-binance-canvas border-binance-hairline text-binance-muted hover:text-binance-body"
                      }`}
                    >
                      {m.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="space-y-1">
                <label className="text-binance-muted font-mono text-[11px]">
                  {t("overlayFilter")}
                </label>
                <select
                  value={overlayCfg.filter || "all-rewards"}
                  onChange={(e) =>
                    updateOverlay({ filter: e.target.value as FilterMode })
                  }
                  className="w-full bg-binance-canvas border border-binance-hairline rounded px-2.5 py-1.5 text-xs text-binance-body outline-none"
                >
                  {filterOptions.map((opt) => (
                    <option key={opt.value} value={opt.value}>
                      {opt.label}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Sliders: Opacity, Width, Font Size */}
            <div className="space-y-2.5">
              <div className="space-y-1">
                <div className="flex justify-between text-[11px] font-mono text-binance-muted">
                  <span>{t("overlayOpacity")}</span>
                  <span className="text-binance-yellow font-bold">
                    {Math.round((overlayCfg.opacity ?? 0.9) * 100)}%
                  </span>
                </div>
                <input
                  type="range"
                  min="0.2"
                  max="1.0"
                  step="0.05"
                  value={overlayCfg.opacity ?? 0.9}
                  onChange={(e) =>
                    updateOverlay({ opacity: parseFloat(e.target.value) }, 100)
                  }
                  className="w-full accent-amber-500"
                />
              </div>

              <div className="space-y-1">
                <div className="flex justify-between text-[11px] font-mono text-binance-muted">
                  <span>{t("overlayWidth")}</span>
                  <span className="text-binance-body font-bold">
                    {Math.round(overlayCfg.width ?? 240)}px
                  </span>
                </div>
                <input
                  type="range"
                  min="180"
                  max="520"
                  step="10"
                  value={overlayCfg.width ?? 240}
                  onChange={(e) =>
                    updateOverlay({ width: parseFloat(e.target.value) }, 100)
                  }
                  className="w-full accent-amber-500"
                />
              </div>

              <div className="space-y-1">
                <div className="flex justify-between text-[11px] font-mono text-binance-muted">
                  <span>{t("overlayFontSize")}</span>
                  <span className="text-binance-body font-bold">
                    {(overlayCfg.font_size ?? 10.5).toFixed(1)}pt
                  </span>
                </div>
                <input
                  type="range"
                  min="8.0"
                  max="32.0"
                  step="0.5"
                  value={overlayCfg.font_size ?? 10.5}
                  onChange={(e) =>
                    updateOverlay({ font_size: parseFloat(e.target.value) }, 100)
                  }
                  className="w-full accent-amber-500"
                />
              </div>
            </div>

            {/* D-Pad Position Adjustment */}
            {(() => {
              const currX = Number.isFinite(overlayCfg.offset_x) ? overlayCfg.offset_x : 16;
              const currY = Number.isFinite(overlayCfg.offset_y) ? overlayCfg.offset_y : 60;
              return (
                <div className="space-y-2 flex flex-col items-center justify-center">
                  <span className="text-binance-muted font-mono text-[11px]">
                    {t("overlayDpad")} (X: {Math.round(currX)}, Y: {Math.round(currY)})
                  </span>
                  <div className="grid grid-cols-3 gap-1 w-28">
                    <div />
                    <button
                      onClick={() =>
                        updateOverlay({
                          offset_y: Math.max(0, currY - 10),
                        })
                      }
                      className="p-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-center font-mono"
                    >
                      ▲
                    </button>
                    <div />
                    <button
                      onClick={() =>
                        updateOverlay({
                          offset_x: Math.max(0, currX - 10),
                        })
                      }
                      className="p-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-center font-mono"
                    >
                      ◀
                    </button>
                    <button
                      onClick={() => updateOverlay({ offset_x: 16, offset_y: 60 })}
                      className="p-1.5 rounded bg-binance-canvas border border-binance-hairline text-binance-muted hover:text-binance-body text-center font-mono text-[10px]"
                      title="Reset (16, 60)"
                    >
                      ●
                    </button>
                    <button
                      onClick={() =>
                        updateOverlay({ offset_x: currX + 10 })
                      }
                      className="p-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-center font-mono"
                    >
                      ▶
                    </button>
                    <div />
                    <button
                      onClick={() =>
                        updateOverlay({ offset_y: currY + 10 })
                      }
                      className="p-1.5 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-center font-mono"
                    >
                      ▼
                    </button>
                    <div />
                  </div>
                </div>
              );
            })()}
          </div>

          {/* Pinned Today Counters */}
          <div className="pt-2 border-t border-binance-hairline space-y-1.5">
            <div className="text-[11px] font-mono text-binance-muted">
              {t("overlayPinned")}
            </div>
            <div className="flex flex-wrap gap-1.5">
              {PINNED_PRESETS.map((preset) => {
                const active = (overlayCfg.pinned_items || []).some((k) =>
                  isSamePinnedKey(k, preset.key)
                );
                return (
                  <button
                    key={preset.id}
                    onClick={() => togglePinnedItem(preset.key)}
                    className={`px-2.5 py-1 rounded text-xs font-mono border transition flex items-center gap-1.5 ${
                      active
                        ? "bg-binance-yellow/15 border-binance-yellow/50 text-binance-yellow font-semibold"
                        : "bg-binance-canvas border-binance-hairline text-binance-muted hover:text-binance-body"
                    }`}
                  >
                    <ResourceIcon
                      resource={preset.id}
                      size="xs"
                      className="w-3.5 h-3.5 object-contain shrink-0"
                    />
                    <span>{language === "vi" ? preset.labelVi : preset.labelEn}</span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Color Customization: Main Text & Accent Highlights */}
          <div className="pt-2 border-t border-binance-hairline grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            {/* Text Color Selection */}
            <div className="space-y-1.5">
              <div className="flex items-center justify-between text-[11px] font-mono">
                <span className="text-binance-muted">{t("overlayTextColor")}</span>
                <span className="font-bold font-mono" style={{ color: overlayCfg.text_color_hex || "#F8FAFC" }}>
                  {overlayCfg.text_color_hex || "#F8FAFC"}
                </span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {[
                  { hex: "#FFFFFF", name: "White" },
                  { hex: "#F8FAFC", name: "Slate" },
                  { hex: "#FCD535", name: "Gold" },
                  { hex: "#67E8F9", name: "Cyan" },
                  { hex: "#A3E635", name: "Lime" },
                  { hex: "#F472B6", name: "Pink" },
                ].map((swatch) => {
                  const active = (overlayCfg.text_color_hex || "#F8FAFC").toUpperCase() === swatch.hex.toUpperCase();
                  return (
                    <button
                      key={swatch.hex}
                      onClick={() => updateOverlay({ text_color_hex: swatch.hex })}
                      className={`w-6 h-6 rounded border transition-all ${
                        active ? "ring-2 ring-binance-yellow scale-110 border-white" : "border-binance-hairline hover:scale-105"
                      }`}
                      style={{ backgroundColor: swatch.hex }}
                      title={swatch.name}
                    />
                  );
                })}
                <label className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-binance-canvas border border-binance-hairline cursor-pointer text-[10px] font-mono text-binance-muted hover:text-binance-body">
                  <input
                    type="color"
                    value={overlayCfg.text_color_hex || "#F8FAFC"}
                    onChange={(e) => updateOverlay({ text_color_hex: e.target.value }, 80)}
                    className="w-4 h-4 rounded cursor-pointer border-0 p-0 bg-transparent"
                  />
                  <span>Custom</span>
                </label>
              </div>
            </div>

            {/* Accent Color Selection */}
            <div className="space-y-1.5">
              <div className="flex items-center justify-between text-[11px] font-mono">
                <span className="text-binance-muted">{t("overlayAccentColor")}</span>
                <span className="font-bold font-mono" style={{ color: overlayCfg.accent_color_hex || "#38BDF8" }}>
                  {overlayCfg.accent_color_hex || "#38BDF8"}
                </span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {[
                  { hex: "#38BDF8", name: "Sky" },
                  { hex: "#FB923C", name: "Amber" },
                  { hex: "#34D399", name: "Emerald" },
                  { hex: "#06B6D4", name: "Cyan" },
                  { hex: "#FCD535", name: "Yellow" },
                  { hex: "#F43F5E", name: "Rose" },
                  { hex: "#A855F7", name: "Violet" },
                ].map((swatch) => {
                  const active = (overlayCfg.accent_color_hex || "#38BDF8").toUpperCase() === swatch.hex.toUpperCase();
                  return (
                    <button
                      key={swatch.hex}
                      onClick={() => updateOverlay({ accent_color_hex: swatch.hex })}
                      className={`w-6 h-6 rounded border transition-all ${
                        active ? "ring-2 ring-binance-yellow scale-110 border-white" : "border-binance-hairline hover:scale-105"
                      }`}
                      style={{ backgroundColor: swatch.hex }}
                      title={swatch.name}
                    />
                  );
                })}
                <label className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-binance-canvas border border-binance-hairline cursor-pointer text-[10px] font-mono text-binance-muted hover:text-binance-body">
                  <input
                    type="color"
                    value={overlayCfg.accent_color_hex || "#38BDF8"}
                    onChange={(e) => updateOverlay({ accent_color_hex: e.target.value }, 80)}
                    className="w-4 h-4 rounded cursor-pointer border-0 p-0 bg-transparent"
                  />
                  <span>Custom</span>
                </label>
              </div>
            </div>
          </div>

          {/* Background Mode: Completely Transparent vs Translucent Color Background */}
          <div className="pt-2 border-t border-binance-hairline space-y-1.5">
            <div className="text-[11px] font-mono text-binance-muted">
              {language === "vi" ? "Kiểu nền HUD" : "HUD Background Style"}
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <button
                type="button"
                onClick={() => updateOverlay({ transparent_bg: false })}
                className={`flex items-center justify-center gap-2 p-2 rounded border text-left transition ${
                  !overlayCfg.transparent_bg
                    ? "border-binance-yellow bg-binance-elevated text-binance-body font-bold ring-1 ring-binance-yellow"
                    : "border-binance-hairline bg-binance-canvas text-binance-muted hover:text-binance-body hover:bg-binance-elevated"
                }`}
              >
                <div className="w-3.5 h-3.5 rounded bg-[#060a14]/75 border border-slate-600" />
                <div className="flex flex-col">
                  <span>{language === "vi" ? "Nền màu trong suốt" : "Translucent Tint"}</span>
                  <span className="text-[10px] opacity-75 font-normal">
                    {language === "vi" ? "Nền đen mờ 75%" : "75% tinted backdrop"}
                  </span>
                </div>
              </button>

              <button
                type="button"
                onClick={() => updateOverlay({ transparent_bg: true })}
                className={`flex items-center justify-center gap-2 p-2 rounded border text-left transition ${
                  overlayCfg.transparent_bg
                    ? "border-binance-yellow bg-binance-elevated text-binance-body font-bold ring-1 ring-binance-yellow"
                    : "border-binance-hairline bg-binance-canvas text-binance-muted hover:text-binance-body hover:bg-binance-elevated"
                }`}
              >
                <div className="w-3.5 h-3.5 rounded bg-transparent border border-dashed border-sky-400" />
                <div className="flex flex-col">
                  <span>{language === "vi" ? "Hoàn toàn trong suốt" : "Pure Transparent"}</span>
                  <span className="text-[10px] opacity-75 font-normal">
                    {language === "vi" ? "Không nền, chỉ chữ nổi" : "Zero background"}
                  </span>
                </div>
              </button>
            </div>
          </div>

          {/* Auto-Fade when idle after n seconds */}
          <div
            className={`pt-2 border-t border-binance-hairline space-y-2 transition-all duration-200 ${
              !overlayCfg.enabled ? "opacity-40 pointer-events-none select-none" : ""
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="text-[11px] font-mono text-binance-muted">
                {t("overlayAutoFade")}
              </span>
              <button
                type="button"
                disabled={!overlayCfg.enabled}
                onClick={() => {
                  const isCurrentlyOn = (overlayCfg.fade_timeout_secs ?? 0) > 0;
                  updateOverlay({
                    fade_timeout_secs: isCurrentlyOn ? 0 : (lastFadeTimeoutRef.current || 30),
                  });
                }}
                className={`relative inline-flex h-4 w-8 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
                  !overlayCfg.enabled
                    ? "cursor-not-allowed bg-binance-hairline"
                    : (overlayCfg.fade_timeout_secs ?? 0) > 0
                    ? "cursor-pointer bg-binance-yellow"
                    : "cursor-pointer bg-binance-hairline"
                }`}
              >
                <span
                  className={`pointer-events-none inline-block h-3 w-3 transform rounded-full bg-binance-canvas shadow ring-0 transition duration-200 ease-in-out ${
                    (overlayCfg.fade_timeout_secs ?? 0) > 0 ? "translate-x-4" : "translate-x-0"
                  }`}
                />
              </button>
            </div>

            {(overlayCfg.fade_timeout_secs ?? 0) > 0 && (
              <div className="space-y-2 pl-2 border-l-2 border-binance-yellow/50">
                {/* Timeout selector */}
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-binance-muted">{t("overlayFadeTimeout")}:</span>
                    <div className="flex items-center gap-1 font-mono">
                      <input
                        type="number"
                        min={1}
                        max={3600}
                        value={overlayCfg.fade_timeout_secs ?? 30}
                        onChange={(e) => {
                          const val = Math.max(1, Math.min(3600, parseInt(e.target.value, 10) || 1));
                          updateOverlay({ fade_timeout_secs: val });
                        }}
                        className="w-16 px-1.5 py-0.5 rounded bg-binance-canvas border border-binance-hairline text-center text-xs font-mono font-bold text-binance-yellow focus:border-binance-yellow focus:outline-none"
                      />
                      <span className="text-xs text-binance-muted">s</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-1.5 flex-wrap">
                    {[15, 30, 60, 120, 300].map((sec) => (
                      <button
                        key={sec}
                        type="button"
                        onClick={() => updateOverlay({ fade_timeout_secs: sec })}
                        className={`px-2 py-0.5 rounded text-xs font-mono border transition ${
                          overlayCfg.fade_timeout_secs === sec
                            ? "border-binance-yellow bg-binance-elevated text-binance-body font-bold"
                            : "border-binance-hairline bg-binance-canvas text-binance-muted hover:text-binance-body"
                        }`}
                      >
                        {sec}s
                      </button>
                    ))}
                  </div>
                </div>

                {/* Fade style selector (Dim vs Hide) */}
                <div className="space-y-1">
                  <div className="text-[11px] font-mono text-binance-muted">
                    {t("overlayFadeStyle")}
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <button
                      type="button"
                      onClick={() => updateOverlay({ fade_style: "Dim" })}
                      className={`flex items-center justify-center gap-2 p-1.5 rounded border text-left transition ${
                        (overlayCfg.fade_style ?? "Dim") === "Dim"
                          ? "border-binance-yellow bg-binance-elevated text-binance-body font-bold"
                          : "border-binance-hairline bg-binance-canvas text-binance-muted hover:text-binance-body"
                      }`}
                    >
                      <div className="w-2.5 h-2.5 rounded-full bg-slate-400 opacity-30 border border-slate-500" />
                      <span>{t("overlayFadeDim")}</span>
                    </button>

                    <button
                      type="button"
                      onClick={() => updateOverlay({ fade_style: "Hide" })}
                      className={`flex items-center justify-center gap-2 p-1.5 rounded border text-left transition ${
                        overlayCfg.fade_style === "Hide"
                          ? "border-binance-yellow bg-binance-elevated text-binance-body font-bold"
                          : "border-binance-hairline bg-binance-canvas text-binance-muted hover:text-binance-body"
                      }`}
                    >
                      <div className="w-2.5 h-2.5 rounded-full border border-dashed border-slate-400" />
                      <span>{t("overlayFadeHide")}</span>
                    </button>
                  </div>

                  {(overlayCfg.fade_style ?? "Dim") === "Dim" && (
                    <div className="pt-1 space-y-1">
                      <div className="flex items-center justify-between text-xs">
                        <span className="text-binance-muted">{t("overlayFadeDimOpacity")}:</span>
                        <span className="font-mono font-bold text-binance-yellow">
                          {Math.round((overlayCfg.fade_dim_opacity ?? 0.25) * 100)}%
                        </span>
                      </div>
                      <input
                        type="range"
                        min="0.05"
                        max="0.80"
                        step="0.05"
                        value={overlayCfg.fade_dim_opacity ?? 0.25}
                        onChange={(e) =>
                          updateOverlay({ fade_dim_opacity: parseFloat(e.target.value) }, 100)
                        }
                        className="w-full accent-amber-500"
                      />
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Live Overlay Preview Box */}
          <div className="pt-3 border-t border-binance-hairline space-y-2">
            <div className="flex items-center justify-between text-[11px] font-mono">
              <span className="text-binance-muted">
                {language === "vi" ? "Xem trước giao diện HUD trực quan" : "Live HUD Visual Preview"}
              </span>
              <div className="flex items-center gap-2">
                {(overlayCfg.fade_timeout_secs ?? 0) > 0 && (
                  <span
                    className={`text-[10px] border rounded px-1.5 py-0.2 ${
                      overlayCfg.enabled
                        ? "text-binance-yellow border-binance-yellow/40 bg-binance-yellow/10"
                        : "text-binance-muted border-binance-hairline bg-binance-hairline/20 opacity-60"
                    }`}
                  >
                    Fade {overlayCfg.fade_timeout_secs}s ({overlayCfg.fade_style === "Hide" ? "Hide" : `Dim ${Math.round((overlayCfg.fade_dim_opacity ?? 0.25) * 100)}%`})
                  </span>
                )}
                <span
                  className={`px-1.5 py-0.5 rounded text-[10px] font-mono border ${
                    overlayCfg.enabled
                      ? "bg-binance-green/15 text-binance-green border-binance-green/40"
                      : "bg-binance-muted/15 text-binance-muted border-binance-hairline"
                  }`}
                >
                  {overlayCfg.enabled
                    ? language === "vi"
                      ? "Đang bật"
                      : "Enabled"
                    : language === "vi"
                    ? "Đã tắt"
                    : "Disabled"}
                </span>
                <span className="text-[10px] text-binance-muted">
                  Mode: {overlayCfg.mode}
                </span>
              </div>
            </div>

            <div
              style={{
                opacity: overlayCfg.opacity ?? 0.95,
                fontSize: `${Math.max(8.0, Math.min(32, overlayCfg.font_size ?? 10.5))}px`,
                maxWidth: `${Math.max(220, Math.min(520, overlayCfg.width ?? 280))}px`,
                color: overlayCfg.text_color_hex || "#F8FAFC",
                textShadow: "0 1px 2px rgba(0,0,0,0.95), 0 0 2px rgba(0,0,0,0.95)",
              }}
              className={`rounded p-2 border border-slate-700/90 select-none transition-all duration-200 ${
                overlayCfg.transparent_bg
                  ? "bg-transparent border-dashed border-sky-400/60"
                  : "bg-[#060a14]/75 shadow-lg"
              }`}
            >
              {overlayCfg.mode === "Hidden" ? (
                <div className="text-[11px] text-slate-400 italic text-center py-2">
                  {language === "vi" ? "HUD đang ở chế độ ẩn" : "HUD is set to Hidden"}
                </div>
              ) : (
                <div className="space-y-1.5">
                  {(overlayCfg.mode === "MiniHud" || overlayCfg.mode === "Combined") && (
                    <div className="flex items-center gap-1 flex-wrap font-mono leading-none">
                      {/* Battle count pill rendered first matching in-game OverlayView */}
                      <div className={`flex items-center gap-1 px-1.5 py-0.5 rounded text-[0.88em] text-slate-200 font-medium ${overlayCfg.transparent_bg ? "bg-transparent border border-white/10" : "bg-[#060a14]/75 border border-slate-700/80"}`}>
                        <span className="text-red-400 font-bold">8M</span>
                        <span>·</span>
                        <span className="text-amber-400 font-bold">12F</span>
                        <span>·</span>
                        <span className="text-sky-400 font-bold">24B</span>
                      </div>
                      <div className={`flex items-center gap-1 px-1.5 py-0.5 rounded text-[0.88em] ${overlayCfg.transparent_bg ? "bg-transparent border border-white/10" : "bg-[#060a14]/75 border border-slate-700/80"}`}>
                        <span className="font-bold" style={{ color: overlayCfg.accent_color_hex || "#38BDF8" }}>GEMS</span>
                        <span className="font-bold" style={{ color: overlayCfg.text_color_hex || "#F8FAFC" }}>12,450</span>
                      </div>
                      <div className={`flex items-center gap-1 px-1.5 py-0.5 rounded text-[0.88em] ${overlayCfg.transparent_bg ? "bg-transparent border border-white/10" : "bg-[#060a14]/75 border border-slate-700/80"}`}>
                        <span className="font-bold" style={{ color: overlayCfg.accent_color_hex || "#38BDF8" }}>SPD</span>
                        <span className="font-bold" style={{ color: overlayCfg.text_color_hex || "#F8FAFC" }}>14h 20m</span>
                      </div>
                    </div>
                  )}

                  {(overlayCfg.mode === "Cards" || overlayCfg.mode === "Combined") && (
                    <div className="space-y-1">
                      <div className={`px-2 py-1.5 rounded border border-l-[3px] border-l-red-500 text-[0.88em] leading-tight ${overlayCfg.transparent_bg ? "bg-transparent border-slate-700/50" : "bg-[#060a14]/75 border-slate-700/80"}`}>
                        <div className="flex items-center justify-between font-bold" style={{ color: overlayCfg.text_color_hex || "#F8FAFC" }}>
                          <span>Motte of the Cursed</span>
                          <span className="text-slate-300 text-[0.85em] font-mono">14:32</span>
                        </div>
                        <div className="flex items-center justify-between text-slate-300 font-mono text-[0.85em]">
                          <span>(450, 782)</span>
                          <span className="font-bold" style={{ color: overlayCfg.accent_color_hex || "#38BDF8" }}>100% · T3</span>
                        </div>
                        <div className="text-amber-300 font-mono font-medium text-[0.85em] truncate">
                          +50 Gems · 60m Speedup · 5,000 Wood
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Console Logs Terminal Card */}
      <div className="p-4 rounded-lg bg-binance-card border border-binance-hairline space-y-2">
        <div className="flex items-center justify-between border-b border-binance-hairline pb-2">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-binance-turquoise" />
            <h3 className="text-xs font-semibold uppercase font-mono text-binance-body">
              {t("settingsConsoleTitle")} ({consoleLogs.length})
            </h3>
          </div>

          <div className="flex items-center gap-3 text-xs">
            <label className="flex items-center gap-1.5 cursor-pointer text-binance-muted hover:text-binance-body">
              <input
                type="checkbox"
                checked={autoScroll}
                onChange={(e) => setAutoScroll(e.target.checked)}
                className="rounded accent-amber-500"
              />
              <span className="text-[11px] font-mono">{t("autoScroll")}</span>
            </label>

            <button
              onClick={clearLogs}
              className="flex items-center gap-1 px-2 py-1 rounded bg-binance-elevated hover:opacity-90 border border-binance-hairline text-binance-body text-[11px] font-mono"
            >
              <Trash2 className="w-3 h-3 text-binance-red" />
              <span>{t("btnClearLogs")}</span>
            </button>
          </div>
        </div>

        {/* Terminal Output Box */}
        <div
          ref={logContainerRef}
          className="h-44 bg-binance-canvas border border-binance-hairline rounded p-2.5 font-mono text-[11px] overflow-y-auto space-y-1"
        >
          {consoleLogs.length === 0 ? (
            <div className="text-binance-muted italic">No console logs yet.</div>
          ) : (
            consoleLogs.map((line, idx) => {
              const isSuccess = line.includes("[SUCCESS]");
              const isError = line.includes("[ERROR]");
              const isWarn = line.includes("[WARN]");
              const isInfo = line.includes("[INFO]");

              const textColor = isSuccess
                ? "text-binance-green"
                : isError
                ? "text-binance-red font-semibold"
                : isWarn
                ? "text-binance-yellow"
                : isInfo
                ? "text-binance-info"
                : "text-binance-muted";

              return (
                <div key={idx} className={`${textColor} leading-tight`}>
                  {line}
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
