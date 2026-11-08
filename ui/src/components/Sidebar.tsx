import React from "react";
import {
  LayoutDashboard,
  Swords,
  Package,
  Settings,
  RefreshCw,
  Play,
  Pause,
  Sun,
  Moon,
  Minimize2,
  Languages,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n } from "../i18n";
import * as api from "../api/tauri";

export const Sidebar: React.FC = () => {
  const {
    tab,
    setTab,
    snapshot,
    language,
    theme,
    toggleTheme,
    toggleLanguage,
    rescan,
    togglePause,
  } = useAppStore();
  const { t } = useI18n(language);

  const navItems = [
    { idx: 0, label: t("navDashboard"), icon: LayoutDashboard },
    { idx: 1, label: t("navBattles"), icon: Swords },
    { idx: 2, label: t("navInventory"), icon: Package },
    { idx: 3, label: t("navSettings"), icon: Settings },
  ];

  const isScanning = Boolean(snapshot?.is_scanning);
  const isWatching = Boolean(snapshot?.is_watching);

  return (
    <aside className="w-56 bg-binance-canvas border-r border-binance-hairline flex flex-col justify-between p-3 select-none shrink-0">
      {/* Top Navigation */}
      <div className="space-y-1">
        <div className="px-2 py-2 mb-2">
          <div className="text-[10px] uppercase tracking-wider text-binance-muted font-mono font-semibold">
            {t("navSection")}
          </div>
        </div>

        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = tab === item.idx;
          return (
            <button
              key={item.idx}
              onClick={() => setTab(item.idx)}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-md text-xs font-medium transition-all ${
                isActive
                  ? "bg-binance-yellow/15 text-binance-yellow border border-binance-yellow/30 font-semibold"
                  : "text-binance-muted hover:text-binance-body hover:bg-binance-card border border-transparent"
              }`}
            >
              <Icon
                className={`w-4 h-4 transition-colors ${
                  isActive ? "text-binance-yellow" : "text-binance-muted"
                }`}
              />
              <span>{item.label}</span>
              {isActive && (
                <span className="ml-auto w-1.5 h-1.5 rounded-full bg-binance-yellowSolid shadow-sm shadow-binance-yellow/50" />
              )}
            </button>
          );
        })}
      </div>

      {/* Bottom Controls Card */}
      <div className="space-y-3 pt-3 border-t border-binance-hairline">
        {/* Watcher Status & Action Buttons */}
        <div className="p-2.5 rounded-md bg-binance-card border border-binance-hairline space-y-2">
          <div className="flex items-center justify-between text-[11px]">
            <span className="text-binance-muted font-mono">Watcher</span>
            <span
              className={`font-mono text-[10px] font-semibold px-1.5 py-0.2 rounded-xs ${
                isScanning
                  ? "bg-binance-info/20 text-binance-info"
                  : isWatching
                  ? "bg-binance-green/20 text-binance-green"
                  : "bg-binance-yellow/20 text-binance-yellow"
              }`}
            >
              {isScanning
                ? t("statusScanning")
                : isWatching
                ? t("statusRunning")
                : t("statusPaused")}
            </span>
          </div>

          <div className="grid grid-cols-2 gap-1.5">
            <button
              onClick={() => rescan()}
              disabled={isScanning}
              className="flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md bg-binance-elevated hover:bg-binance-hairline border border-binance-hairline text-binance-body text-[11px] font-medium transition disabled:opacity-50"
              title={t("btnScan")}
            >
              <RefreshCw
                className={`w-3 h-3 text-binance-yellow ${
                  isScanning ? "animate-spin" : ""
                }`}
              />
              <span>{t("btnScan")}</span>
            </button>

            <button
              onClick={() => togglePause()}
              className={`flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md border text-[11px] font-medium transition ${
                isWatching
                  ? "bg-binance-elevated hover:bg-binance-hairline border-binance-hairline text-binance-body"
                  : "bg-binance-yellow/10 hover:bg-binance-yellow/20 border-binance-yellow/30 text-binance-yellow"
              }`}
            >
              {isWatching ? (
                <>
                  <Pause className="w-3 h-3 text-binance-yellow" />
                  <span>{t("btnPause")}</span>
                </>
              ) : (
                <>
                  <Play className="w-3 h-3 text-binance-green" />
                  <span>{t("btnResume")}</span>
                </>
              )}
            </button>
          </div>
        </div>

        {/* Global Action Utility Buttons */}
        <div className="flex items-center justify-between px-1 text-binance-muted text-xs">
          <button
            onClick={() => toggleTheme()}
            className="p-1.5 rounded-md hover:bg-binance-card hover:text-binance-body transition"
            title={`${t("themeToggle")} (${theme})`}
          >
            {theme === "dark" ? (
              <Sun className="w-4 h-4 text-binance-yellow" />
            ) : (
              <Moon className="w-4 h-4 text-binance-yellow" />
            )}
          </button>

          <button
            onClick={() => toggleLanguage()}
            className="flex items-center gap-1 px-2 py-1 rounded-md hover:bg-binance-card hover:text-binance-body transition text-[11px] font-mono"
            title="Switch Language (VI/EN)"
          >
            <Languages className="w-3.5 h-3.5 text-binance-yellow" />
            <span>{language.toUpperCase()}</span>
          </button>

          <button
            onClick={() => api.windowMinimizeToTray()}
            className="p-1.5 rounded-md hover:bg-binance-card hover:text-binance-body transition"
            title={t("btnMinimizeTray")}
          >
            <Minimize2 className="w-4 h-4 text-binance-muted hover:text-binance-body" />
          </button>
        </div>
      </div>
    </aside>
  );
};
