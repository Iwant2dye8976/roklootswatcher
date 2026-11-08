import React, { useState, useEffect, useRef, useCallback } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  Search,
  FileSpreadsheet,
  FileCode,
  ClipboardCopy,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  ExternalLink,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n, formatSpeedupMinutes } from "../i18n";
import { BattleRecord } from "../types";
import * as api from "../api/tauri";

export const BattlesView: React.FC = () => {
  const {
    language,
    setSelectedBattle,
    addToast,
    mode,
    datePreset,
    dateFrom,
    dateTo,
    accountUid,
    speedupUnit,
  } = useAppStore();
  const { t } = useI18n(language);

  const [records, setRecords] = useState<BattleRecord[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [searchInput, setSearchInput] = useState<string>("");
  const [debouncedSearch, setDebouncedSearch] = useState<string>("");
  const [sortCol, setSortCol] = useState<string>("time");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  const parentRef = useRef<HTMLDivElement>(null);
  const isGatheringMode = mode === "gathering";

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(searchInput);
    }, 150);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  const loadRecords = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.queryBattles(debouncedSearch, sortCol, sortDir);
      setRecords(data);
    } catch (err) {
      console.error("Failed to query battles:", err);
    } finally {
      setLoading(false);
    }
  }, [debouncedSearch, sortCol, sortDir]);

  useEffect(() => {
    loadRecords();
  }, [loadRecords, mode, datePreset, dateFrom, dateTo, accountUid, language]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api
      .onWatcherUpdated(() => {
        loadRecords();
      })
      .then((u) => {
        unlisten = u;
      });
    return () => {
      if (unlisten) unlisten();
    };
  }, [loadRecords]);

  const rowVirtualizer = useVirtualizer({
    count: records.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,
    overscan: 10,
  });

  const handleSort = (col: string) => {
    if (sortCol === col) {
      setSortDir((prev) => (prev === "asc" ? "desc" : "asc"));
    } else {
      setSortCol(col);
      setSortDir("desc");
    }
  };

  const renderSortIcon = (col: string) => {
    if (sortCol !== col)
      return <ArrowUpDown className="w-3 h-3 text-binance-muted opacity-60" />;
    return sortDir === "asc" ? (
      <ArrowUp className="w-3 h-3 text-binance-yellow" />
    ) : (
      <ArrowDown className="w-3 h-3 text-binance-yellow" />
    );
  };

  const handleCopySummary = async () => {
    try {
      const summary = await api.getClipboardSummary();
      await navigator.clipboard.writeText(summary);
      addToast({
        type: "success",
        text:
          language === "vi"
            ? "Đã sao chép tóm tắt chiến lợi phẩm vào Clipboard!"
            : "Copied loot summary to clipboard!",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleExport = async (fmt: "csv" | "json") => {
    try {
      const savedPath = await api.exportActiveData(fmt);
      if (savedPath) {
        addToast({
          type: "success",
          text:
            language === "vi"
              ? `Đã xuất file: ${savedPath}`
              : `Exported to ${savedPath}`,
        });
      }
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden p-4 sm:p-5 select-none space-y-3 bg-binance-canvas text-binance-body">
      {/* Search & Export Toolbar */}
      <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-wrap items-center justify-between gap-3 shrink-0">
        <div className="relative flex-1 min-w-[220px] max-w-md">
          <Search className="w-4 h-4 text-binance-muted absolute left-3 top-2.5" />
          <input
            type="text"
            placeholder={t("battlesSearch")}
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            className="w-full bg-binance-canvas border border-binance-hairline rounded-sm pl-9 pr-3 py-1.5 text-xs text-binance-body placeholder-binance-muted outline-none focus:border-binance-yellow/60 font-mono"
          />
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <span className="font-mono text-xs text-binance-muted mr-1">
            {t("matchingCount")}{" "}
            <span className="text-binance-yellow font-semibold">
              {records.length.toLocaleString()}
            </span>
          </span>

          <button
            onClick={handleCopySummary}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-sm bg-binance-elevated hover:bg-binance-hairline border border-binance-hairline text-binance-body text-xs font-medium transition"
          >
            <ClipboardCopy className="w-3.5 h-3.5 text-binance-info" />
            <span>{t("btnCopySummary")}</span>
          </button>

          <button
            onClick={() => handleExport("csv")}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-sm bg-binance-elevated hover:bg-binance-hairline border border-binance-hairline text-binance-body text-xs font-medium transition"
          >
            <FileSpreadsheet className="w-3.5 h-3.5 text-binance-green" />
            <span>{t("btnExportCsv")}</span>
          </button>

          <button
            onClick={() => handleExport("json")}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-sm bg-binance-elevated hover:bg-binance-hairline border border-binance-hairline text-binance-body text-xs font-medium transition"
          >
            <FileCode className="w-3.5 h-3.5 text-binance-yellow" />
            <span>{t("btnExportJson")}</span>
          </button>
        </div>
      </div>

      {/* Virtualized Table Container */}
      <div className="flex-1 flex flex-col rounded-md bg-binance-card border border-binance-hairline overflow-hidden">
        {/* Table Header */}
        <div className="h-10 border-b border-binance-hairline bg-binance-canvas grid grid-cols-12 items-center px-4 text-[11px] font-mono font-semibold text-binance-muted shrink-0">
          <div
            onClick={() => handleSort("time")}
            className="col-span-2 flex items-center gap-1 cursor-pointer hover:text-binance-body"
          >
            <span>{t("colTime")}</span>
            {renderSortIcon("time")}
          </div>

          <div
            onClick={() => handleSort("type")}
            className="col-span-1 flex items-center gap-1 cursor-pointer hover:text-binance-body text-center justify-center"
          >
            <span>{t("colType")}</span>
            {renderSortIcon("type")}
          </div>

          <div
            onClick={() => handleSort("target")}
            className="col-span-3 flex items-center gap-1 cursor-pointer hover:text-binance-body pl-2"
          >
            <span>{t("colTarget")}</span>
            {renderSortIcon("target")}
          </div>

          {isGatheringMode ? (
            <>
              <div
                onClick={() => handleSort("bonus")}
                className="col-span-2 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colBonus")}</span>
                {renderSortIcon("bonus")}
              </div>
              <div
                onClick={() => handleSort("resources")}
                className="col-span-2 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colResources")}</span>
                {renderSortIcon("resources")}
              </div>
              <div
                onClick={() => handleSort("gems")}
                className="col-span-1 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colGems")}</span>
                {renderSortIcon("gems")}
              </div>
            </>
          ) : (
            <>
              <div
                onClick={() => handleSort("damage")}
                className="col-span-1 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colDmg")}</span>
                {renderSortIcon("damage")}
              </div>

              <div
                onClick={() => handleSort("tier")}
                className="col-span-1 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colTier")}</span>
                {renderSortIcon("tier")}
              </div>

              <div
                onClick={() => handleSort("crystals")}
                className="col-span-1 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colCrystals")}</span>
                {renderSortIcon("crystals")}
              </div>

              <div
                onClick={() => handleSort("speedups")}
                className="col-span-2 flex items-center gap-1 cursor-pointer hover:text-binance-body justify-center"
              >
                <span>{t("colUnivSpd")}</span>
                {renderSortIcon("speedups")}
              </div>
            </>
          )}

          <div className="col-span-1 text-right">{t("colActions")}</div>
        </div>

        {/* Table Body */}
        <div ref={parentRef} className="flex-1 overflow-y-auto">
          {loading ? (
            <div className="p-8 text-center text-xs text-binance-muted font-mono">
              {t("loadingBattles")}
            </div>
          ) : records.length === 0 ? (
            <div className="p-8 text-center text-xs text-binance-muted">
              {t("emptyBattles")}
            </div>
          ) : (
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: "100%",
                position: "relative",
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const b = records[virtualRow.index];
                const badgeColor =
                  b.battle_type === "motte"
                    ? "bg-binance-red/20 text-binance-red border-binance-red/40"
                    : b.battle_type === "fort"
                    ? "bg-binance-yellow/20 text-binance-yellow border-binance-yellow/40"
                    : b.battle_type === "gathering"
                    ? "bg-binance-green/20 text-binance-green border-binance-green/40"
                    : "bg-binance-info/20 text-binance-info border-binance-info/40";

                return (
                  <div
                    key={b.id || virtualRow.index}
                    onClick={() => setSelectedBattle(b)}
                    style={{
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    className="grid grid-cols-12 items-center px-4 text-xs hover:bg-binance-elevated/60 cursor-pointer border-b border-binance-hairline/60 transition font-mono"
                  >
                    <div className="col-span-2 text-binance-muted truncate">
                      {b.time_vn || b.time_utc}
                    </div>

                    <div className="col-span-1 flex justify-center">
                      <span
                        className={`px-1.5 py-0.2 rounded-xs text-[9px] font-bold uppercase border ${badgeColor}`}
                      >
                        {b.battle_type}
                      </span>
                    </div>

                    <div className="col-span-3 text-binance-body truncate pl-2 font-sans font-medium">
                      {b.display_target || b.target}
                    </div>

                    {isGatheringMode ? (
                      <>
                        <div className="col-span-2 text-center text-binance-green font-medium">
                          {b.talent_add > 0
                            ? `+${b.talent_add.toLocaleString()}`
                            : "-"}
                        </div>
                        <div className="col-span-2 text-center text-binance-yellow font-medium">
                          {b.total_rss
                            ? b.total_rss >= 1_000_000
                              ? `${(b.total_rss / 1_000_000).toFixed(2)}M`
                              : b.total_rss.toLocaleString()
                            : "-"}
                        </div>
                        <div className="col-span-1 text-center text-binance-info font-bold">
                          {b.gems ? `+${b.gems.toLocaleString()}` : "-"}
                        </div>
                      </>
                    ) : (
                      <>
                        <div className="col-span-1 text-center text-binance-yellow">
                          {b.battle_type === "gathering"
                            ? "-"
                            : `${b.damage_pct}%`}
                        </div>

                        <div className="col-span-1 text-center text-binance-yellow">
                          {b.plunder_tier > 0 ? `T${b.plunder_tier}` : "-"}
                        </div>

                        <div className="col-span-1 text-center text-binance-yellow font-bold">
                          {b.crystals ? `+${b.crystals.toLocaleString()}` : "-"}
                        </div>

                        <div className="col-span-2 text-center text-binance-green font-medium">
                          {b.speedup_univ_mins
                            ? formatSpeedupMinutes(
                                b.speedup_univ_mins,
                                speedupUnit,
                                language
                              )
                            : "-"}
                        </div>
                      </>
                    )}

                    <div className="col-span-1 flex justify-end">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedBattle(b);
                        }}
                        className="p-1 rounded text-binance-muted hover:text-binance-yellow hover:bg-binance-elevated transition"
                      >
                        <ExternalLink className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
