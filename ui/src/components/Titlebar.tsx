import React from "react";
import {
  Minus,
  Square,
  X,
  Database,
  ShieldCheck,
  ShieldAlert,
  Loader2,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n } from "../i18n";
import * as api from "../api/tauri";
import { TopAccountSection } from "./TopAccountSection";

export const Titlebar: React.FC = () => {
  const { snapshot, language, setCommandPaletteOpen } = useAppStore();
  const { t } = useI18n(language);

  const isScanning = Boolean(snapshot?.is_scanning);
  const isWatching = Boolean(snapshot?.is_watching);

  return (
    <header
      data-tauri-drag-region
      className="h-10 w-full bg-binance-canvas border-b border-binance-hairline flex items-center justify-between px-3 select-none z-50 text-xs shrink-0"
    >
      {/* Left: App Branding & Status Badges */}
      <div className="flex items-center gap-3" data-tauri-drag-region>
        <div
          className="flex items-center gap-2 font-semibold tracking-wide"
          data-tauri-drag-region
        >
          <span className="w-2 h-2 rounded-xs bg-binance-yellowSolid shadow-sm shadow-binance-yellow/40" />
          <span className="text-binance-yellow font-mono text-[11px] uppercase tracking-wider font-bold">
            RoK
          </span>
          <span className="text-binance-body font-medium">Loot Watcher</span>
        </div>

        <span className="px-1.5 py-0.5 rounded-sm bg-binance-card text-[10px] text-binance-muted font-mono border border-binance-hairline">
          v{snapshot?.version || "2.7.2"}
        </span>

        {/* Live Vault Record Count Badge */}
        <div
          data-no-drag
          className="hidden sm:flex items-center gap-1.5 px-2 py-0.5 rounded-sm bg-binance-card border border-binance-hairline text-binance-body text-[11px]"
          title={`${t("statusVault")}: ${snapshot?.vault_count?.toLocaleString() || 0} ${t("recordsCount")}`}
        >
          <Database className="w-3 h-3 text-binance-yellow" />
          <span className="font-mono font-medium text-binance-yellow">
            {(snapshot?.vault_count || 0).toLocaleString()}
          </span>
          <span className="text-[10px] text-binance-muted">
            {t("recordsCount")}
          </span>
        </div>

        {/* Watcher Status Badge */}
        <div
          data-no-drag
          title={snapshot?.watcher_status_text || ""}
          className={`hidden md:flex items-center gap-1 px-2 py-0.5 rounded-sm text-[10px] font-medium border ${
            isScanning
              ? "bg-binance-info/10 text-binance-info border-binance-info/30"
              : isWatching
              ? "bg-binance-green/10 text-binance-green border-binance-green/30"
              : "bg-binance-yellow/10 text-binance-yellow border-binance-yellow/30"
          }`}
        >
          {isScanning ? (
            <Loader2 className="w-3 h-3 animate-spin" />
          ) : isWatching ? (
            <ShieldCheck className="w-3 h-3" />
          ) : (
            <ShieldAlert className="w-3 h-3" />
          )}
          <span>
            {isScanning
              ? t("statusScanning")
              : isWatching
              ? t("statusRunning")
              : t("statusPaused")}
          </span>
        </div>
      </div>

      {/* Center: Command Palette Hint */}
      <button
        data-no-drag
        onClick={() => setCommandPaletteOpen(true)}
        className="hidden lg:flex items-center gap-2 px-3 py-1 rounded-md bg-binance-card hover:bg-binance-elevated border border-binance-hairline text-binance-muted hover:text-binance-body transition text-[11px]"
      >
        <span>{t("searchOrCommand")}</span>
        <kbd className="px-1.5 py-0.2 bg-binance-elevated rounded text-[9px] font-mono text-binance-muted border border-binance-hairline">
          Ctrl+K
        </kbd>
      </button>

      {/* Right: Account Section & Window Controls */}
      <div className="flex items-center gap-2" data-no-drag>
        <TopAccountSection />

        <div className="w-[1px] h-4 bg-binance-hairline" />

        <div className="flex items-center gap-0.5">
          <button
            onClick={() => api.windowMinimize()}
            title="Minimize"
            className="w-7 h-7 flex items-center justify-center rounded hover:bg-binance-elevated text-binance-muted hover:text-binance-body transition"
          >
            <Minus className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => api.windowMaximize()}
            title="Maximize"
            className="w-7 h-7 flex items-center justify-center rounded hover:bg-binance-elevated text-binance-muted hover:text-binance-body transition"
          >
            <Square className="w-3 h-3" />
          </button>
          <button
            onClick={() => api.windowClose()}
            title="Close"
            className="w-7 h-7 flex items-center justify-center rounded hover:bg-red-600 text-binance-muted hover:text-white transition"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </header>
  );
};
