import React, { useState, useEffect } from "react";
import { Filter, Calendar, User, Clock, ChevronDown } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n } from "../i18n";
import { FilterMode, DateFilterPreset, SpeedupUnit } from "../types";

export const GlobalFilterBar: React.FC = () => {
  const {
    snapshot,
    language,
    mode,
    datePreset,
    dateFrom,
    dateTo,
    accountUid,
    speedupUnit,
    updateFilter,
  } = useAppStore();
  const { t } = useI18n(language);

  const [showCustomDate, setShowCustomDate] = useState(datePreset === "Custom");
  const [customStart, setCustomStart] = useState(dateFrom);
  const [customEnd, setCustomEnd] = useState(dateTo);

  useEffect(() => {
    setShowCustomDate(datePreset === "Custom");
    setCustomStart(dateFrom);
    setCustomEnd(dateTo);
  }, [datePreset, dateFrom, dateTo]);

  const modes: Array<{ id: FilterMode; label: string }> = [
    { id: "all-targets", label: t("modeAll") },
    { id: "motte", label: t("modeMotte") },
    { id: "fort", label: t("modeFort") },
    { id: "barbarian", label: t("modeBarbarian") },
    { id: "all-forts", label: t("modeAllForts") },
    { id: "all-rewards", label: t("modeAllRewards") },
    { id: "gathering", label: t("modeGathering") },
    { id: "pvp", label: t("modePvp") },
  ];

  const presets: Array<{ id: DateFilterPreset; label: string }> = [
    { id: "AllTime", label: t("presetAll") },
    { id: "Today", label: t("presetToday") },
    { id: "Yesterday", label: t("presetYesterday") },
    { id: "Last3Days", label: t("presetDays3") },
    { id: "Last7Days", label: t("presetDays7") },
    { id: "Last15Days", label: t("presetDays15") },
    { id: "Last30Days", label: t("presetDays30") },
    { id: "Custom", label: t("presetCustom") },
  ];

  const speedupUnits: Array<{ id: SpeedupUnit; label: string }> = [
    { id: "minutes", label: t("unitMinutes") },
    { id: "hours", label: t("unitHours") },
    { id: "days", label: t("unitDays") },
    { id: "dhm", label: t("unitDhm") },
  ];

  return (
    <div className="bg-binance-canvas border-b border-binance-hairline px-4 py-2.5 flex flex-wrap items-center justify-between gap-3 text-xs shrink-0 select-none">
      {/* Mode Filter Pills */}
      <div className="flex items-center gap-1.5 flex-wrap">
        <div className="flex items-center gap-1 text-binance-muted mr-1 font-medium">
          <Filter className="w-3.5 h-3.5 text-binance-yellow" />
          <span>{t("filterMode")}:</span>
        </div>
        {modes.map((m) => {
          const isSelected = mode === m.id;
          return (
            <button
              key={m.id}
              onClick={() => updateFilter({ mode: m.id })}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-all border ${
                isSelected
                  ? "bg-binance-yellowSolid text-binance-ink border-binance-yellowSolid font-bold shadow-sm"
                  : "bg-binance-card text-binance-muted border-binance-hairline hover:text-binance-body hover:bg-binance-elevated"
              }`}
            >
              {m.label}
            </button>
          );
        })}
      </div>

      {/* Date Presets & Accounts */}
      <div className="flex items-center gap-2.5 flex-wrap ml-auto">
        {/* Date Preset Selector */}
        <div className="flex items-center gap-1 bg-binance-card p-0.5 rounded-md border border-binance-hairline">
          <Calendar className="w-3.5 h-3.5 text-binance-muted ml-1.5" />
          {presets.map((p) => {
            const isSelected = datePreset === p.id;
            return (
              <button
                key={p.id}
                onClick={() => {
                  if (p.id === "Custom") {
                    setShowCustomDate(true);
                  } else {
                    setShowCustomDate(false);
                    updateFilter({
                      datePreset: p.id,
                      dateFrom: "",
                      dateTo: "",
                    });
                  }
                }}
                className={`px-2 py-0.5 rounded-sm text-[11px] font-medium transition ${
                  isSelected
                    ? "bg-binance-yellow/20 text-binance-yellow font-bold border border-binance-yellow/40"
                    : "text-binance-muted hover:text-binance-body"
                }`}
              >
                {p.label}
              </button>
            );
          })}
        </div>

        {/* Custom Date Inputs */}
        {showCustomDate && (
          <div className="flex items-center gap-1 bg-binance-card px-2 py-1 rounded-md border border-binance-hairline">
            <input
              type="date"
              value={customStart}
              onChange={(e) => setCustomStart(e.target.value)}
              className="bg-transparent text-[11px] text-binance-body outline-none font-mono"
            />
            <span className="text-binance-muted font-mono">→</span>
            <input
              type="date"
              value={customEnd}
              onChange={(e) => setCustomEnd(e.target.value)}
              className="bg-transparent text-[11px] text-binance-body outline-none font-mono"
            />
            <button
              onClick={() =>
                updateFilter({
                  datePreset: "Custom",
                  dateFrom: customStart,
                  dateTo: customEnd,
                })
              }
              className="px-2 py-0.5 bg-binance-yellowSolid hover:bg-binance-yellowHover text-binance-ink rounded-xs text-[10px] font-bold"
            >
              {t("btnApply")}
            </button>
          </div>
        )}

        {/* Account Selector Dropdown */}
        <div className="relative flex items-center bg-binance-card rounded-md border border-binance-hairline px-2 py-1">
          <User className="w-3.5 h-3.5 text-binance-muted mr-1.5" />
          <select
            value={accountUid || ""}
            onChange={(e) =>
              updateFilter({ accountUid: e.target.value || undefined })
            }
            className="bg-transparent text-[11px] text-binance-body outline-none cursor-pointer pr-4 appearance-none"
          >
            <option value="" className="bg-binance-card text-binance-body">
              {t("accountAll")}
            </option>
            {snapshot?.available_accounts?.map((acc) => (
              <option
                key={acc.uid}
                value={acc.uid}
                className="bg-binance-card text-binance-body"
              >
                {acc.name || acc.uid}
              </option>
            ))}
          </select>
          <ChevronDown className="w-3 h-3 text-binance-muted pointer-events-none absolute right-1.5" />
        </div>

        {/* Speedup Unit Selector */}
        <div className="relative flex items-center bg-binance-card rounded-md border border-binance-hairline px-2 py-1">
          <Clock className="w-3.5 h-3.5 text-binance-muted mr-1.5" />
          <select
            value={speedupUnit}
            onChange={(e) =>
              updateFilter({ speedupUnit: e.target.value as SpeedupUnit })
            }
            className="bg-transparent text-[11px] text-binance-body outline-none cursor-pointer pr-4 appearance-none font-mono"
          >
            {speedupUnits.map((u) => (
              <option
                key={u.id}
                value={u.id}
                className="bg-binance-card text-binance-body"
              >
                {u.label}
              </option>
            ))}
          </select>
          <ChevronDown className="w-3 h-3 text-binance-muted pointer-events-none absolute right-1.5" />
        </div>
      </div>
    </div>
  );
};
