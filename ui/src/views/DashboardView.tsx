import React, { useState } from "react";
import {
  Swords,
  Sparkles,
  Clock,
  Coins,
  TrendingUp,
  ChevronRight,
  BarChart2,
  Shield,
  Activity,
  ArrowUpRight,
  ArrowDownRight,
  Crosshair,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n, formatSpeedupMinutes } from "../i18n";
import { BattleRecord } from "../types";

export const DashboardView: React.FC = () => {
  const { dashboard, language, setTab, setSelectedBattle, speedupUnit } =
    useAppStore();
  const { t } = useI18n(language);

  const [hoveredDayIdx, setHoveredDayIdx] = useState<number | null>(null);

  if (!dashboard) {
    return (
      <div className="flex-1 flex items-center justify-center p-8 text-binance-muted font-mono text-xs">
        Connecting to telemetry stream...
      </div>
    );
  }

  const d = dashboard;
  const ap = d.ap_economics;

  const formatSpeedup = (mins: number) =>
    formatSpeedupMinutes(mins, speedupUnit, language);

  const totalCounts = Math.max(
    1,
    d.motte_count + d.fort_count + d.barb_count + d.gathering_count
  );
  const mottePct = (d.motte_count / totalCounts) * 100;
  const fortPct = (d.fort_count / totalCounts) * 100;
  const barbPct = (d.barb_count / totalCounts) * 100;
  const gatherPct = (d.gathering_count / totalCounts) * 100;

  const maxApDaily = Math.max(
    100,
    ...ap.daily_timeline.map((p) => p.ap_consumed)
  );
  const maxSpdDaily = Math.max(
    1,
    ...ap.daily_timeline.map((p) => p.speedup_hours)
  );

  const maxItemQty = Math.max(
    1,
    ...d.top_items.slice(0, 10).map((it) => it.total_quantity)
  );

  return (
    <div className="flex-1 overflow-y-auto p-4 sm:p-5 space-y-4 bg-binance-canvas select-none font-sans text-binance-body">
      {/* 1. View Header */}
      <div className="flex items-center justify-between border-b border-binance-hairline pb-3">
        <div className="flex items-center gap-2.5">
          <div className="w-2.5 h-2.5 rounded-xs bg-binance-yellowSolid shadow-sm shadow-binance-yellow/40" />
          <div>
            <h1 className="text-xs sm:text-sm font-bold tracking-wider text-binance-body uppercase font-mono flex items-center gap-2">
              <span>{t("dashTitle")}</span>
              <span className="text-[10px] text-binance-muted font-normal normal-case border border-binance-hairline px-1.5 py-0.2 rounded-xs bg-binance-card">
                {t("liveTelemetry")}
              </span>
            </h1>
            <p className="text-[11px] text-binance-muted mt-0.5 font-sans">
              {t("dashSubtitle")}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 text-[11px] font-mono text-binance-muted">
          <Activity className="w-3.5 h-3.5 text-binance-green" />
          <span className="hidden sm:inline">{t("vaultEncounters")}</span>
          <span className="text-binance-yellow font-bold">
            {d.total_battles.toLocaleString()}
          </span>
        </div>
      </div>

      {/* 2. Primary 6-Metric Ribbon */}
      <div className="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-6 gap-2.5">
        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardBattles")}</span>
            <Swords className="w-3.5 h-3.5 text-binance-yellow" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-yellow tracking-tight">
              {d.total_battles.toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {d.motte_count} {t("modeMotte")} · {d.fort_count} {t("modeFort")} ·{" "}
            {d.barb_count} {t("modeBarbarian")}
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-red/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("apConsumed")}</span>
            <ArrowDownRight className="w-3.5 h-3.5 text-binance-red" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-red tracking-tight">
              {ap.total_ap_consumed.toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            Net: {ap.net_ap_spent.toLocaleString()} AP
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-green/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("apRecovered")}</span>
            <ArrowUpRight className="w-3.5 h-3.5 text-binance-green" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-green tracking-tight">
              +{ap.total_ap_recovered.toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-green font-mono font-medium">
            +{ap.rebate_pct.toFixed(1)}%
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardSpeedups")}</span>
            <Clock className="w-3.5 h-3.5 text-binance-yellow" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-body tracking-tight">
              {formatSpeedup(d.speedup_total_mins)}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {ap.speedup_mins_per_1k_ap.toFixed(0)}m / 1k AP
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardCrystals")}</span>
            <Sparkles className="w-3.5 h-3.5 text-binance-yellow" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-yellow tracking-tight">
              {d.total_crystals.toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {t("avgCrystal").replace("{:.1}", d.avg_crystals.toFixed(1))}
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardResources")}</span>
            <Coins className="w-3.5 h-3.5 text-binance-yellow" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-body tracking-tight">
              {(d.total_resources / 1_000_000).toFixed(2)}M
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {d.total_resources.toLocaleString()} RSS
          </div>
        </div>
      </div>

      {/* 3. Target Encounter Distribution */}
      <div className="p-3 rounded-md bg-binance-card border border-binance-hairline space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="font-semibold text-binance-body tracking-wide">
            {t("distTitle")}
          </span>
          <span className="font-mono text-binance-muted text-[11px]">
            {d.total_battles.toLocaleString()} {t("encountersAnalyzed")}
          </span>
        </div>

        <div className="h-2 w-full bg-binance-canvas rounded-xs overflow-hidden flex">
          {mottePct > 0 && (
            <div
              style={{ width: `${mottePct}%` }}
              className="h-full bg-binance-red transition-all"
              title={`${t("modeMotte")}: ${d.motte_count} (${mottePct.toFixed(1)}%)`}
            />
          )}
          {fortPct > 0 && (
            <div
              style={{ width: `${fortPct}%` }}
              className="h-full bg-binance-yellowSolid transition-all"
              title={`${t("modeFort")}: ${d.fort_count} (${fortPct.toFixed(1)}%)`}
            />
          )}
          {barbPct > 0 && (
            <div
              style={{ width: `${barbPct}%` }}
              className="h-full bg-binance-info transition-all"
              title={`${t("modeBarbarian")}: ${d.barb_count} (${barbPct.toFixed(1)}%)`}
            />
          )}
          {gatherPct > 0 && (
            <div
              style={{ width: `${gatherPct}%` }}
              className="h-full bg-binance-green transition-all"
              title={`${t("modeGathering")}: ${d.gathering_count} (${gatherPct.toFixed(1)}%)`}
            />
          )}
        </div>

        <div className="flex items-center gap-4 flex-wrap text-[11px] font-mono text-binance-muted pt-0.5">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-red" />
            <span className="text-binance-body">{t("modeMotte")}:</span>{" "}
            {d.motte_count} ({mottePct.toFixed(1)}%)
          </span>
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-yellowSolid" />
            <span className="text-binance-body">{t("modeFort")}:</span>{" "}
            {d.fort_count} ({fortPct.toFixed(1)}%)
          </span>
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-info" />
            <span className="text-binance-body">{t("modeBarbarian")}:</span>{" "}
            {d.barb_count} ({barbPct.toFixed(1)}%)
          </span>
          {d.gathering_count > 0 && (
            <span className="flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-xs bg-binance-green" />
              <span className="text-binance-body">{t("modeGathering")}:</span>{" "}
              {d.gathering_count} ({gatherPct.toFixed(1)}%)
            </span>
          )}
        </div>
      </div>

      {/* 4. Main 8 / 4 Statistical Matrix */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <div className="lg:col-span-8 space-y-4">
          {/* AP Timeline Card */}
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between">
            <div className="flex items-center justify-between text-xs mb-3 border-b border-binance-hairline pb-2.5">
              <div className="flex items-center gap-2">
                <TrendingUp className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("apTimeline")}
                </span>
              </div>
              <div className="flex items-center gap-3 text-[10px] font-mono">
                <span className="flex items-center gap-1 text-binance-green font-medium">
                  <span className="w-2 h-2 rounded-xs bg-binance-green" />{" "}
                  {t("cardSpeedups")} (h)
                </span>
                <span className="flex items-center gap-1 text-binance-red font-medium">
                  <span className="w-2 h-2 rounded-xs bg-binance-red" />{" "}
                  {t("apConsumed")}
                </span>
              </div>
            </div>

            <div className="h-44 w-full flex items-end gap-2 pt-2 border-b border-binance-hairline pb-2 relative">
              <div className="absolute inset-0 flex flex-col justify-between pointer-events-none opacity-25">
                <div className="border-b border-binance-hairline w-full" />
                <div className="border-b border-binance-hairline w-full" />
                <div className="border-b border-binance-hairline w-full" />
              </div>

              {ap.daily_timeline.map((item, idx) => {
                const spdHeightPct = Math.max(
                  4,
                  (item.speedup_hours / maxSpdDaily) * 100
                );
                const apHeightPct = Math.max(
                  4,
                  (item.ap_consumed / maxApDaily) * 100
                );

                return (
                  <div
                    key={idx}
                    onMouseEnter={() => setHoveredDayIdx(idx)}
                    onMouseLeave={() => setHoveredDayIdx(null)}
                    className="flex-1 h-full flex flex-col justify-end items-center gap-1 group relative cursor-pointer z-10"
                  >
                    {hoveredDayIdx === idx && (
                      <div className="absolute -top-16 bg-binance-elevated border border-binance-hairline text-binance-body text-[10px] font-mono px-2.5 py-1.5 rounded-sm shadow-xl z-30 whitespace-nowrap pointer-events-none">
                        <div className="font-bold text-binance-yellow border-b border-binance-hairline pb-1 mb-1">
                          {item.date_label}
                        </div>
                        <div className="flex items-center justify-between gap-3 text-binance-green">
                          <span>{t("cardSpeedups")}:</span>
                          <span className="font-bold">
                            {item.speedup_hours.toFixed(1)}h
                          </span>
                        </div>
                        <div className="flex items-center justify-between gap-3 text-binance-red">
                          <span>AP:</span>
                          <span className="font-bold">
                            {item.ap_consumed.toLocaleString()} AP
                          </span>
                        </div>
                      </div>
                    )}

                    <div className="w-full flex items-end justify-center gap-1 h-full">
                      <div
                        style={{ height: `${spdHeightPct}%` }}
                        className="w-1/2 bg-binance-green/80 group-hover:bg-binance-green rounded-t-xs transition-all"
                      />
                      <div
                        style={{ height: `${apHeightPct}%` }}
                        className="w-1/2 bg-binance-red/80 group-hover:bg-binance-red rounded-t-xs transition-all"
                      />
                    </div>
                    <span className="text-[9px] font-mono text-binance-muted group-hover:text-binance-body truncate">
                      {item.date_label}
                    </span>
                  </div>
                );
              })}
            </div>

            <div className="grid grid-cols-3 gap-2 pt-2.5 text-center text-[10px] font-mono text-binance-muted">
              <div>
                <span>{t("apSpeedupYield")}:</span>{" "}
                <span className="text-binance-green font-bold">
                  {(ap.speedup_mins_per_1k_ap / 60).toFixed(1)}h
                </span>
              </div>
              <div>
                <span>{t("apGemYield")}:</span>{" "}
                <span className="text-binance-yellow font-bold">
                  {ap.gems_per_1k_ap.toFixed(0)}
                </span>
              </div>
              <div>
                <span>{t("apRecoveryRatio")}:</span>{" "}
                <span className="text-binance-green font-bold">
                  +{ap.rebate_pct.toFixed(1)}%
                </span>
              </div>
            </div>
          </div>

          {/* Target Efficiency Benchmarks Table */}
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2.5">
            <div className="flex items-center justify-between text-xs pb-1 border-b border-binance-hairline">
              <div className="flex items-center gap-2">
                <Crosshair className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("apBenchmarks")}
                </span>
              </div>
              <span className="text-[10px] font-mono text-binance-muted">
                {t("rankedByEfficiency")}
              </span>
            </div>

            <div className="overflow-x-auto max-h-56 overflow-y-auto">
              <table className="w-full text-left text-[11px] font-mono">
                <thead>
                  <tr className="border-b border-binance-hairline text-[10px] text-binance-muted uppercase">
                    <th className="pb-2 font-medium">{t("colTarget")}</th>
                    <th className="pb-2 font-medium text-center">
                      {t("colApCost")}
                    </th>
                    <th className="pb-2 font-medium text-center">
                      {t("colRuns")}
                    </th>
                    <th className="pb-2 font-medium text-center">
                      {t("colYield1k")}
                    </th>
                    <th className="pb-2 font-medium text-center">
                      {t("colCovenantDrop")}
                    </th>
                    <th className="pb-2 font-medium text-right">
                      {t("colRating")}
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-binance-hairline/60">
                  {ap.target_benchmarks.map((tb, idx) => (
                    <tr
                      key={idx}
                      className="hover:bg-binance-elevated/70 transition-colors"
                    >
                      <td className="py-2 text-binance-body font-medium truncate max-w-[140px]">
                        {tb.target_name}
                      </td>
                      <td className="py-2 text-center text-binance-muted">
                        {tb.unit_ap_cost}
                      </td>
                      <td className="py-2 text-center text-binance-yellow font-medium">
                        {tb.battle_count}
                      </td>
                      <td className="py-2 text-center text-binance-green font-bold">
                        {tb.speedup_mins_per_1k_ap.toFixed(0)}m
                      </td>
                      <td className="py-2 text-center text-binance-yellow">
                        {tb.covenant_drop_rate > 0
                          ? `${tb.covenant_drop_rate.toFixed(1)}%`
                          : "-"}
                      </td>
                      <td className="py-2 text-right">
                        <span
                          className={`px-1.5 py-0.5 rounded-xs text-[9px] font-mono font-bold ${
                            tb.tier_badge.includes("S+")
                              ? "bg-binance-yellowSolid text-binance-ink font-bold shadow-sm"
                              : tb.tier_badge.includes("S")
                              ? "bg-binance-green/20 text-binance-green border border-binance-green/40"
                              : "bg-binance-card text-binance-body border border-binance-hairline"
                          }`}
                        >
                          {tb.tier_badge}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {/* Right Column: Top Items & Latest Battles */}
        <div className="lg:col-span-4 space-y-4">
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2.5">
            <div className="flex items-center justify-between text-xs pb-1 border-b border-binance-hairline">
              <div className="flex items-center gap-2">
                <BarChart2 className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("topItems")}
                </span>
              </div>
              <button
                onClick={() => setTab(2)}
                className="text-binance-yellow hover:text-binance-yellowHover flex items-center gap-0.5 text-[11px] font-medium transition"
              >
                <span>{t("btnViewAll")}</span>
                <ChevronRight className="w-3.5 h-3.5" />
              </button>
            </div>

            <div className="space-y-1.5 max-h-72 overflow-y-auto pr-1">
              {d.top_items.slice(0, 10).map((it, idx) => {
                const volWidthPct = (it.total_quantity / maxItemQty) * 100;

                return (
                  <div
                    key={idx}
                    className="p-2 rounded-xs bg-binance-canvas border border-binance-hairline hover:bg-binance-elevated/40 transition relative overflow-hidden text-xs"
                  >
                    <div
                      style={{ width: `${volWidthPct}%` }}
                      className="absolute top-0 right-0 h-full bg-binance-yellow/10 pointer-events-none"
                    />

                    <div className="relative z-10 space-y-1">
                      <div className="flex items-center justify-between gap-2">
                        <div className="flex items-center gap-1.5 min-w-0">
                          <span className="text-[10px] font-mono text-binance-muted w-4 font-semibold">
                            #{idx + 1}
                          </span>
                          <span className="font-medium text-binance-body truncate">
                            {it.display_name ||
                              (language === "vi" ? it.name_vi : it.name_en)}
                          </span>
                          {it.converted_value && (
                            <span className="text-[10px] font-mono text-binance-green shrink-0">
                              ({it.converted_value})
                            </span>
                          )}
                        </div>
                        <span className="font-mono font-bold text-binance-yellow shrink-0">
                          {it.total_quantity.toLocaleString()}
                        </span>
                      </div>

                      <div className="flex items-center justify-between text-[10px] text-binance-muted font-mono">
                        <span className="text-binance-green">
                          {t("lblDropRate")}: {it.drop_rate.toFixed(1)}%
                        </span>
                        <span>
                          {t("lblAvgRun")}: {it.avg_per_battle.toFixed(1)}
                        </span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2.5">
            <div className="flex items-center justify-between text-xs pb-1 border-b border-binance-hairline">
              <div className="flex items-center gap-2">
                <Shield className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("latestBattles")}
                </span>
              </div>
              <button
                onClick={() => setTab(1)}
                className="text-binance-yellow hover:text-binance-yellowHover flex items-center gap-0.5 text-[11px] font-medium transition"
              >
                <span>{t("btnViewAll")}</span>
                <ChevronRight className="w-3.5 h-3.5" />
              </button>
            </div>

            <div className="space-y-1.5 max-h-72 overflow-y-auto pr-1">
              {d.latest_battles.map((b: BattleRecord, idx) => {
                const badgeColor =
                  b.battle_type === "motte"
                    ? "bg-binance-red/20 text-binance-red border-binance-red/30"
                    : b.battle_type === "fort"
                    ? "bg-binance-yellow/20 text-binance-yellow border-binance-yellow/30"
                    : b.battle_type === "gathering"
                    ? "bg-binance-green/20 text-binance-green border-binance-green/30"
                    : "bg-binance-info/20 text-binance-info border-binance-info/30";

                return (
                  <div
                    key={idx}
                    onClick={() => setSelectedBattle(b)}
                    className="p-2 rounded-xs bg-binance-canvas border border-binance-hairline hover:border-binance-yellow/40 hover:bg-binance-elevated/40 transition cursor-pointer flex items-center justify-between gap-2.5 text-xs"
                  >
                    <div className="flex flex-col gap-0.5 min-w-0">
                      <div className="flex items-center gap-1.5">
                        <span
                          className={`px-1 py-0.2 rounded-xs text-[9px] font-mono font-bold uppercase border ${badgeColor}`}
                        >
                          {b.battle_type}
                        </span>
                        <span className="font-semibold text-binance-body truncate">
                          {b.display_target || b.target}
                        </span>
                        {b.plunder_tier > 0 && (
                          <span className="text-[9px] font-mono text-binance-yellow">
                            T{b.plunder_tier}
                          </span>
                        )}
                      </div>
                      <div className="text-[10px] text-binance-muted font-mono truncate">
                        {b.time_vn || b.time_utc} · ({b.pos_x}, {b.pos_y}) ·{" "}
                        {b.battle_type === "gathering"
                          ? t("lblGatheringDone")
                          : `${b.damage_pct}% DMG`}
                      </div>
                    </div>

                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedBattle(b);
                      }}
                      className="px-2 py-0.5 rounded-xs bg-binance-elevated hover:bg-binance-yellowSolid hover:text-binance-ink text-binance-body text-[10px] font-mono font-medium transition shrink-0 border border-binance-hairline"
                    >
                      {t("btnDetail")}
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
