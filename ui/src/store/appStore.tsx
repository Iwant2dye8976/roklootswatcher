import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  useRef,
} from "react";
import {
  AppSnapshotDto,
  DashboardDto,
  BattleRecord,
  FilterMode,
  DateFilterPreset,
  SpeedupUnit,
  AppLanguage,
  AppTheme,
  CloseBehavior,
} from "../types";
import * as api from "../api/tauri";

export interface ToastMessage {
  id: string;
  type: "info" | "success" | "warning" | "error";
  text: string;
  duration?: number;
}

interface AppStoreContextType {
  tab: number;
  setTab: (tab: number) => void;
  snapshot: AppSnapshotDto | null;
  dashboard: DashboardDto | null;
  loading: boolean;
  theme: AppTheme;
  language: AppLanguage;
  closeBehavior: CloseBehavior;
  mode: FilterMode;
  datePreset: DateFilterPreset;
  dateFrom: string;
  dateTo: string;
  accountUid?: string;
  speedupUnit: SpeedupUnit;
  consoleLogs: string[];
  selectedBattle: BattleRecord | null;
  setSelectedBattle: (b: BattleRecord | null) => void;
  commandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;
  toasts: ToastMessage[];
  addToast: (msg: Omit<ToastMessage, "id">) => void;
  removeToast: (id: string) => void;
  refreshSnapshot: () => Promise<void>;
  refreshDashboard: () => Promise<void>;
  updateFilter: (newFilters: {
    mode?: FilterMode;
    datePreset?: DateFilterPreset;
    dateFrom?: string;
    dateTo?: string;
    accountUid?: string;
    speedupUnit?: SpeedupUnit;
  }) => Promise<void>;
  toggleTheme: () => Promise<void>;
  toggleLanguage: () => Promise<void>;
  setCloseBehaviorPref: (cb: CloseBehavior) => Promise<void>;
  rescan: () => Promise<void>;
  togglePause: () => Promise<void>;
  clearLogs: () => void;
}

const AppStoreContext = createContext<AppStoreContextType | null>(null);

function applyDocumentTheme(t: AppTheme) {
  if (t === "dark") {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}

export const AppStoreProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [tab, setTab] = useState<number>(0);
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [dashboard, setDashboard] = useState<DashboardDto | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  const [theme, setTheme] = useState<AppTheme>("dark");
  const [language, setLanguage] = useState<AppLanguage>("vi");
  const [closeBehavior, setCloseBehavior] = useState<CloseBehavior>("tray");
  const [mode, setMode] = useState<FilterMode>("all-targets");
  const [datePreset, setDatePreset] = useState<DateFilterPreset>("Today");
  const [dateFrom, setDateFrom] = useState<string>("");
  const [dateTo, setDateTo] = useState<string>("");
  const [accountUid, setAccountUid] = useState<string | undefined>(undefined);
  const [speedupUnit, setSpeedupUnit] = useState<SpeedupUnit>("hours");

  const [consoleLogs, setConsoleLogs] = useState<string[]>([]);
  const [selectedBattle, setSelectedBattle] = useState<BattleRecord | null>(null);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState<boolean>(false);
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const pendingManualScanRef = useRef<boolean>(false);
  const languageRef = useRef<AppLanguage>(language);
  languageRef.current = language;

  const addToast = useCallback((msg: Omit<ToastMessage, "id">) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((prev) => [...prev, { ...msg, id }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, msg.duration || 4000);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const refreshSnapshot = useCallback(async () => {
    try {
      const snap = await api.getInitialState();
      setSnapshot(snap);
    } catch (e) {
      console.error("Failed to refresh snapshot:", e);
    }
  }, []);

  const refreshDashboard = useCallback(async () => {
    try {
      const data = await api.getDashboardData();
      setDashboard(data);
    } catch (e) {
      console.error("Failed to refresh dashboard:", e);
    }
  }, []);

  const updateFilter = useCallback(
    async (newFilters: {
      mode?: FilterMode;
      datePreset?: DateFilterPreset;
      dateFrom?: string;
      dateTo?: string;
      accountUid?: string;
      speedupUnit?: SpeedupUnit;
    }) => {
      const m = newFilters.mode ?? mode;
      const dp = newFilters.datePreset ?? datePreset;
      const isCustom = dp === "Custom";
      const df = isCustom ? (newFilters.dateFrom ?? dateFrom) : "";
      const dt = isCustom ? (newFilters.dateTo ?? dateTo) : "";
      const au =
        "accountUid" in newFilters ? newFilters.accountUid : accountUid;
      const su = newFilters.speedupUnit ?? speedupUnit;

      setMode(m);
      setDatePreset(dp);
      setDateFrom(df);
      setDateTo(dt);
      setAccountUid(au);
      setSpeedupUnit(su);

      try {
        const d = await api.setFilterState(m, dp, df, dt, au, su);
        setDashboard(d);
      } catch (err) {
        console.error("Failed to update filter:", err);
      }
    },
    [mode, datePreset, dateFrom, dateTo, accountUid, speedupUnit]
  );

  const toggleTheme = useCallback(async () => {
    const nextTheme: AppTheme = theme === "dark" ? "light" : "dark";
    setTheme(nextTheme);
    applyDocumentTheme(nextTheme);
    setSnapshot((prev) => (prev ? { ...prev, theme: nextTheme } : prev));
    await api.setPreferences(nextTheme, language, closeBehavior);
  }, [theme, language, closeBehavior]);

  const toggleLanguage = useCallback(async () => {
    const nextLang: AppLanguage = language === "vi" ? "en" : "vi";
    setLanguage(nextLang);
    setSnapshot((prev) => (prev ? { ...prev, language: nextLang } : prev));
    await api.setPreferences(theme, nextLang, closeBehavior);
    await refreshDashboard();
  }, [theme, language, closeBehavior, refreshDashboard]);

  const setCloseBehaviorPref = useCallback(
    async (cb: CloseBehavior) => {
      setCloseBehavior(cb);
      setSnapshot((prev) => (prev ? { ...prev, close_behavior: cb } : prev));
      await api.setPreferences(theme, language, cb);
    },
    [theme, language]
  );

  const rescan = useCallback(async () => {
    try {
      pendingManualScanRef.current = true;
      addToast({
        type: "info",
        text:
          language === "vi" ? "Đang quét lại mailcache..." : "Rescanning mailcache...",
      });
      await api.triggerRescan();
    } catch (e) {
      pendingManualScanRef.current = false;
      addToast({ type: "error", text: String(e) });
    }
  }, [language, addToast]);

  const togglePause = useCallback(async () => {
    try {
      const isWatching = await api.toggleWatcherPause();
      setSnapshot((prev) =>
        prev ? { ...prev, is_watching: isWatching } : prev
      );
      addToast({
        type: "info",
        text: isWatching
          ? language === "vi"
            ? "Đã tiếp tục theo dõi mail"
            : "Watcher resumed"
          : language === "vi"
          ? "Đã tạm dừng theo dõi mail"
          : "Watcher paused",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  }, [language, addToast]);

  const clearLogs = useCallback(() => {
    setConsoleLogs([]);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setCommandPaletteOpen((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    let isMounted = true;
    const cleanups: Array<() => void> = [];
    const register = (fn: () => void) => {
      if (isMounted) {
        cleanups.push(fn);
      } else {
        fn();
      }
    };

    async function init() {
      try {
        const snap = await api.getInitialState();
        if (!isMounted) return;
        setSnapshot(snap);
        setTheme(snap.theme);
        setLanguage(snap.language);
        setCloseBehavior(snap.close_behavior);
        setMode(snap.mode);
        setDatePreset(snap.date_preset);
        setDateFrom(snap.date_from);
        setDateTo(snap.date_to);
        setAccountUid(snap.account_uid);
        setSpeedupUnit(snap.speedup_unit);
        applyDocumentTheme(snap.theme);

        const dash = await api.getDashboardData();
        if (!isMounted) return;
        setDashboard(dash);
      } catch (err) {
        console.error("Initialization error:", err);
      } finally {
        if (isMounted) setLoading(false);
      }

      if (!isMounted) return;

      api.onWatcherUpdated(() => {
        refreshDashboard();
        refreshSnapshot();
      }).then(register);

      api.onWatcherStatus((statusPayload) => {
        const scanningNow = Boolean(statusPayload.is_scanning);
        if (!scanningNow && pendingManualScanRef.current) {
          pendingManualScanRef.current = false;
          refreshDashboard();
          refreshSnapshot();
          addToast({
            type: "success",
            text:
              languageRef.current === "vi"
                ? "Quét mail hoàn tất!"
                : "Mailcache scan complete!",
          });
        }
        setSnapshot((prev) =>
          prev
            ? {
                ...prev,
                is_watching: statusPayload.is_watching,
                is_scanning: scanningNow,
                watcher_status_text: statusPayload.status,
              }
            : prev
        );
      }).then(register);

      api.onWatcherLog((line) => {
        setConsoleLogs((prev) => [...prev.slice(-499), line]);
      }).then(register);

      api.onCloudUpdated((status) => {
        setSnapshot((prev) =>
          prev ? { ...prev, cloud_status: status } : prev
        );
      }).then(register);

      api.onCloudPushResult((res) => {
        const isVi = languageRef.current === "vi";
        if (res.success) {
          addToast({
            type: "success",
            text: isVi
              ? res.count > 0
                ? `Sao lưu Cloud thành công (${res.count} bản ghi đã tải lên)`
                : "Sao lưu Cloud thành công! Dữ liệu đã đồng bộ mới nhất."
              : res.count > 0
              ? `Cloud backup complete (${res.count} records uploaded)`
              : "Cloud backup complete! Everything is up to date.",
          });
        } else {
          addToast({
            type: "error",
            text: isVi
              ? `Sao lưu Cloud thất bại: ${res.error || "Lỗi không xác định"}`
              : `Cloud backup failed: ${res.error || "Unknown error"}`,
          });
        }
        refreshSnapshot();
      }).then(register);

      api.onCloudPullResult((res) => {
        const isVi = languageRef.current === "vi";
        if (res.success) {
          addToast({
            type: "success",
            text: isVi
              ? `Khôi phục Cloud thành công (${res.count} bản ghi đã tải về)`
              : `Cloud pull complete (${res.count} records restored)`,
          });
        } else {
          addToast({
            type: "error",
            text: isVi
              ? `Khôi phục Cloud thất bại: ${res.error || "Lỗi không xác định"}`
              : `Cloud pull failed: ${res.error || "Unknown error"}`,
          });
        }
        refreshSnapshot();
        refreshDashboard();
      }).then(register);
    }

    init();

    return () => {
      isMounted = false;
      cleanups.forEach((fn) => fn());
    };
  }, [refreshDashboard, refreshSnapshot, addToast]);

  return (
    <AppStoreContext.Provider
      value={{
        tab,
        setTab,
        snapshot,
        dashboard,
        loading,
        theme,
        language,
        closeBehavior,
        mode,
        datePreset,
        dateFrom,
        dateTo,
        accountUid,
        speedupUnit,
        consoleLogs,
        selectedBattle,
        setSelectedBattle,
        commandPaletteOpen,
        setCommandPaletteOpen,
        toasts,
        addToast,
        removeToast,
        refreshSnapshot,
        refreshDashboard,
        updateFilter,
        toggleTheme,
        toggleLanguage,
        setCloseBehaviorPref,
        rescan,
        togglePause,
        clearLogs,
      }}
    >
      {children}
    </AppStoreContext.Provider>
  );
};

export const useAppStore = () => {
  const ctx = useContext(AppStoreContext);
  if (!ctx) throw new Error("useAppStore must be used within AppStoreProvider");
  return ctx;
};
