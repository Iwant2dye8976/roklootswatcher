import React, { useState, useMemo } from "react";
import {
  Swords,
  Sparkles,
  Clock,
  TrendingUp,
  ChevronRight,
  BarChart2,
  Shield,
  Activity,
  ArrowUpRight,
  ArrowDownRight,
  Crosshair,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
  Zap,
  X,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n, formatSpeedupMinutes } from "../i18n";
import { BattleRecord } from "../types";
import { ResourceIcon } from "../components/ResourceIcon";

type BenchmarkSortKey =
  | "target"
  | "level"
  | "ap"
  | "runs"
  | "yield"
  | "covenant"
  | "rating";

const getTargetType = (tb: {
  target_name?: string;
  battle_type?: string;
}): "fort" | "motte" | "barbarian" | "other" => {
  if (tb.battle_type) {
    const bt = tb.battle_type.toLowerCase();
    if (bt.includes("fort")) return "fort";
    if (bt.includes("motte")) return "motte";
    if (bt.includes("barb")) return "barbarian";
  }
  const name = (tb.target_name || "").toLowerCase();
  if (name.includes("fort")) return "fort";
  if (name.includes("motte")) return "motte";
  if (name.includes("barb")) return "barbarian";
  return "other";
};

const getTargetLevel = (tb: {
  target_name?: string;
  level?: number;
}): number => {
  if (tb.level !== undefined && tb.level > 0) return tb.level;
  const match = (tb.target_name || "").match(/(\d+)/);
  return match ? parseInt(match[1], 10) : 0;
};

export const DashboardView: React.FC = () => {
  const { dashboard, language, setTab, setSelectedBattle, speedupUnit } =
    useAppStore();
  const { t } = useI18n(language);

  const [hoveredDayIdx, setHoveredDayIdx] = useState<number | null>(null);
  const [bmTypeFilter, setBmTypeFilter] = useState<
    "all" | "fort" | "motte" | "barbarian"
  >("all");
  const [bmSearch, setBmSearch] = useState("");
  const [bmSortKey, setBmSortKey] = useState<BenchmarkSortKey>("yield");
  const [bmSortDir, setBmSortDir] = useState<"asc" | "desc">("desc");

  const ap = dashboard?.ap_economics || {
    total_ap_consumed: 0,
    total_ap_recovered: 0,
    net_ap_spent: 0,
    rebate_pct: 0,
    speedup_mins_per_1k_ap: 0,
    gems_per_1k_ap: 0,
    daily_timeline: [],
    target_benchmarks: [],
  };

  const handleBenchmarkSort = (key: BenchmarkSortKey) => {
    if (bmSortKey === key) {
      setBmSortDir(bmSortDir === "desc" ? "asc" : "desc");
    } else {
      setBmSortKey(key);
      setBmSortDir("desc");
    }
  };

  const maxBenchmarkYield = useMemo(() => {
    const list = ap.target_benchmarks || [];
    if (list.length === 0) return 1;
    return Math.max(
      1,
      ...list.map((b) => b.speedup_mins_per_1k_ap || 0)
    );
  }, [ap.target_benchmarks]);

  const filteredBenchmarks = useMemo(() => {
    let list = ap.target_benchmarks || [];
    if (bmTypeFilter !== "all") {
      list = list.filter((b) => getTargetType(b) === bmTypeFilter);
    }
    if (bmSearch.trim()) {
      const q = bmSearch.toLowerCase().trim();
      list = list.filter((b) => {
        const lvl = getTargetLevel(b);
        return (
          (b.target_name || "").toLowerCase().includes(q) ||
          (lvl > 0 && String(lvl).includes(q))
        );
      });
    }

    return [...list].sort((a, b) => {
      let cmp = 0;
      switch (bmSortKey) {
        case "target":
          cmp = (a.target_name || "").localeCompare(b.target_name || "");
          break;
        case "level":
          cmp = getTargetLevel(a) - getTargetLevel(b);
          break;
        case "ap":
          cmp = (a.unit_ap_cost || 0) - (b.unit_ap_cost || 0);
          break;
        case "runs":
          cmp = (a.battle_count || 0) - (b.battle_count || 0);
          break;
        case "yield":
          cmp =
            (a.speedup_mins_per_1k_ap || 0) -
            (b.speedup_mins_per_1k_ap || 0);
          break;
        case "covenant":
          cmp = (a.covenant_drop_rate || 0) - (b.covenant_drop_rate || 0);
          break;
        case "rating": {
          const rankVal = (badge?: string) => {
            if (!badge) return 1;
            const b = String(badge);
            if (b.includes("S+")) return 4;
            if (b.includes("S")) return 3;
            if (b.includes("A")) return 2;
            return 1;
          };
          cmp = rankVal(a.tier_badge) - rankVal(b.tier_badge);
          break;
        }
      }
      return bmSortDir === "desc" ? -cmp : cmp;
    });
  }, [ap.target_benchmarks, bmTypeFilter, bmSearch, bmSortKey, bmSortDir]);

  const dailyTimeline = ap.daily_timeline || [];
  const maxApDaily = Math.max(
    1000,
    ...dailyTimeline.map((p) => p.ap_consumed || 0)
  );
  const maxSpdDaily = Math.max(
    1,
    ...dailyTimeline.map((p) => p.speedup_hours || 0)
  );

  // SVG dimensions for Dual-Axis Combo Chart
  const svgViewWidth = 700;
  const svgViewHeight = 180;
  const chartPadLeft = 46;
  const chartPadRight = 46;
  const chartBaselineY = 152;
  const chartTopY = 16;
  const chartAvailH = chartBaselineY - chartTopY;
  const chartPlotWidth = svgViewWidth - chartPadLeft - chartPadRight;

  const comboChartPoints = useMemo(() => {
    if (dailyTimeline.length === 0) return [];
    const count = dailyTimeline.length;
    const stepX = count > 1 ? chartPlotWidth / (count - 1) : chartPlotWidth / 2;

    return dailyTimeline.map((item, idx) => {
      const x = count > 1 ? chartPadLeft + idx * stepX : svgViewWidth / 2;
      const spd = item.speedup_hours || 0;
      const apConsumed = item.ap_consumed || 0;
      const lineY = chartBaselineY - (spd / maxSpdDaily) * chartAvailH;
      const barH = Math.max(2, (apConsumed / maxApDaily) * chartAvailH);
      const barY = chartBaselineY - barH;
      const barWidth = Math.max(8, Math.min(28, (chartPlotWidth / count) * 0.55));
      const yieldMinPer1k = apConsumed > 0 ? (spd * 60) / (apConsumed / 1000) : 0;

      return {
        idx,
        item,
        x,
        lineY,
        barY,
        barH,
        barWidth,
        spd,
        apConsumed,
        yieldMinPer1k,
      };
    });
  }, [dailyTimeline, maxApDaily, maxSpdDaily, chartPlotWidth, chartAvailH]);

  const comboLinePath = useMemo(() => {
    if (comboChartPoints.length === 0) return "";
    if (comboChartPoints.length === 1) {
      const p = comboChartPoints[0];
      return `M ${p.x - 20} ${p.lineY} L ${p.x + 20} ${p.lineY}`;
    }
    let d = `M ${comboChartPoints[0].x} ${comboChartPoints[0].lineY}`;
    for (let i = 0; i < comboChartPoints.length - 1; i++) {
      const p0 = comboChartPoints[i];
      const p1 = comboChartPoints[i + 1];
      const mx = (p0.x + p1.x) / 2;
      d += ` C ${mx} ${p0.lineY}, ${mx} ${p1.lineY}, ${p1.x} ${p1.lineY}`;
    }
    return d;
  }, [comboChartPoints]);

  const comboAreaPath = useMemo(() => {
    if (comboChartPoints.length === 0) return "";
    if (comboChartPoints.length === 1) return "";
    const first = comboChartPoints[0];
    const last = comboChartPoints[comboChartPoints.length - 1];
    return `${comboLinePath} L ${last.x} ${chartBaselineY} L ${first.x} ${chartBaselineY} Z`;
  }, [comboLinePath, comboChartPoints]);

  if (!dashboard) {
    return (
      <div className="flex-1 flex items-center justify-center p-8 text-binance-muted font-mono text-xs">
        Connecting to telemetry stream...
      </div>
    );
  }

  const d = dashboard;

  const formatSpeedup = (mins: number) =>
    formatSpeedupMinutes(mins || 0, speedupUnit, language);

  const totalCounts = Math.max(
    1,
    (d.motte_count || 0) +
      (d.fort_count || 0) +
      (d.barb_count || 0) +
      (d.gathering_count || 0)
  );
  const mottePct = ((d.motte_count || 0) / totalCounts) * 100;
  const fortPct = ((d.fort_count || 0) / totalCounts) * 100;
  const barbPct = ((d.barb_count || 0) / totalCounts) * 100;
  const gatherPct = ((d.gathering_count || 0) / totalCounts) * 100;

  const topItemsList = d.top_items || [];
  const maxItemQty = Math.max(
    1,
    ...topItemsList.slice(0, 10).map((it) => it.total_quantity || 0)
  );

  const renderSortHeader = (
    label: string,
    key: BenchmarkSortKey,
    align: "left" | "center" | "right" = "center"
  ) => {
    const isActive = bmSortKey === key;
    return (
      <th
        onClick={() => handleBenchmarkSort(key)}
        className={`pb-2 font-medium cursor-pointer select-none transition-colors hover:text-binance-yellow ${
          align === "left"
            ? "text-left"
            : align === "right"
            ? "text-right"
            : "text-center"
        } ${isActive ? "text-binance-yellow font-semibold" : "text-binance-muted"}`}
      >
        <div
          className={`inline-flex items-center gap-1 ${
            align === "left"
              ? "justify-start"
              : align === "right"
              ? "justify-end"
              : "justify-center"
          }`}
        >
          <span>{label}</span>
          {isActive ? (
            bmSortDir === "desc" ? (
              <ArrowDown className="w-3 h-3 text-binance-yellow" />
            ) : (
              <ArrowUp className="w-3 h-3 text-binance-yellow" />
            )
          ) : (
            <ArrowUpDown className="w-2.5 h-2.5 opacity-40 hover:opacity-100" />
          )}
        </div>
      </th>
    );
  };

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
            {(d.total_battles || 0).toLocaleString()}
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
              {(d.total_battles || 0).toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {d.motte_count || 0} {t("modeMotte")} · {d.fort_count || 0} {t("modeFort")} ·{" "}
            {d.barb_count || 0} {t("modeBarbarian")}
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-red/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("apConsumed")}</span>
            <ArrowDownRight className="w-3.5 h-3.5 text-binance-red" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-red tracking-tight">
              {(ap.total_ap_consumed || 0).toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            Net: {(ap.net_ap_spent || 0).toLocaleString()} AP
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-green/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("apRecovered")}</span>
            <ArrowUpRight className="w-3.5 h-3.5 text-binance-green" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-green tracking-tight">
              +{(ap.total_ap_recovered || 0).toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-green font-mono font-medium">
            +{(ap.rebate_pct || 0).toFixed(1)}%
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardSpeedups")}</span>
            <Clock className="w-3.5 h-3.5 text-binance-yellow" />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-body tracking-tight">
              {formatSpeedup(d.speedup_total_mins || 0)}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {(ap.speedup_mins_per_1k_ap || 0).toFixed(0)}m / 1k AP
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardCrystals")}</span>
            <ResourceIcon
              resource="crystals"
              size="sm"
              className="w-4 h-4 object-contain"
              fallback={<Sparkles className="w-3.5 h-3.5 text-binance-yellow" />}
            />
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-yellow tracking-tight">
              {(d.total_crystals || 0).toLocaleString()}
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {t("avgCrystal").replace("{:.1}", (d.avg_crystals || 0).toFixed(1))}
          </div>
        </div>

        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
          <div className="flex items-center justify-between text-[11px] text-binance-muted">
            <span className="font-medium">{t("cardResources")}</span>
            <div className="flex items-center -space-x-1 shrink-0" title="RoK Resources (Food, Wood, Stone, Gold)">
              <ResourceIcon resource="food" size="xs" className="w-3.5 h-3.5 object-contain" />
              <ResourceIcon resource="wood" size="xs" className="w-3.5 h-3.5 object-contain" />
              <ResourceIcon resource="stone" size="xs" className="w-3.5 h-3.5 object-contain" />
              <ResourceIcon resource="gold" size="xs" className="w-3.5 h-3.5 object-contain" />
            </div>
          </div>
          <div className="my-1.5">
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-body tracking-tight">
              {((d.total_resources || 0) / 1_000_000).toFixed(2)}M
            </div>
          </div>
          <div className="text-[10px] text-binance-muted font-mono truncate">
            {(d.total_resources || 0).toLocaleString()} RSS
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
            {(d.total_battles || 0).toLocaleString()} {t("encountersAnalyzed")}
          </span>
        </div>

        <div className="h-2 w-full bg-binance-canvas rounded-xs overflow-hidden flex">
          {mottePct > 0 && (
            <div
              style={{ width: `${mottePct}%` }}
              className="h-full bg-binance-red transition-all"
              title={`${t("modeMotte")}: ${d.motte_count || 0} (${(mottePct || 0).toFixed(1)}%)`}
            />
          )}
          {fortPct > 0 && (
            <div
              style={{ width: `${fortPct}%` }}
              className="h-full bg-binance-yellowSolid transition-all"
              title={`${t("modeFort")}: ${d.fort_count || 0} (${(fortPct || 0).toFixed(1)}%)`}
            />
          )}
          {barbPct > 0 && (
            <div
              style={{ width: `${barbPct}%` }}
              className="h-full bg-binance-info transition-all"
              title={`${t("modeBarbarian")}: ${d.barb_count || 0} (${(barbPct || 0).toFixed(1)}%)`}
            />
          )}
          {gatherPct > 0 && (
            <div
              style={{ width: `${gatherPct}%` }}
              className="h-full bg-binance-green transition-all"
              title={`${t("modeGathering")}: ${d.gathering_count || 0} (${(gatherPct || 0).toFixed(1)}%)`}
            />
          )}
        </div>

        <div className="flex items-center gap-4 flex-wrap text-[11px] font-mono text-binance-muted pt-0.5">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-red" />
            <span className="text-binance-body">{t("modeMotte")}:</span>{" "}
            {d.motte_count || 0} ({(mottePct || 0).toFixed(1)}%)
          </span>
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-yellowSolid" />
            <span className="text-binance-body">{t("modeFort")}:</span>{" "}
            {d.fort_count || 0} ({(fortPct || 0).toFixed(1)}%)
          </span>
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-xs bg-binance-info" />
            <span className="text-binance-body">{t("modeBarbarian")}:</span>{" "}
            {d.barb_count || 0} ({(barbPct || 0).toFixed(1)}%)
          </span>
          {(d.gathering_count || 0) > 0 && (
            <span className="flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-xs bg-binance-green" />
              <span className="text-binance-body">{t("modeGathering")}:</span>{" "}
              {d.gathering_count || 0} ({(gatherPct || 0).toFixed(1)}%)
            </span>
          )}
        </div>
      </div>

      {/* 4. Main 8 / 4 Statistical Matrix */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <div className="lg:col-span-8 space-y-4">
          {/* AP Timeline Card */}
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between">
            <div className="flex items-center justify-between text-xs mb-2 border-b border-binance-hairline pb-2.5">
              <div className="flex items-center gap-2">
                <TrendingUp className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("apTimeline")}
                </span>
              </div>
              <div className="flex items-center gap-3 text-[10px] font-mono">
                <span className="flex items-center gap-1.5 text-emerald-400 font-medium">
                  <span className="w-2.5 h-2 rounded-xs bg-emerald-500/50 border border-emerald-400" />{" "}
                  {t("apConsumed")} (Left Y)
                </span>
                <span className="flex items-center gap-1.5 text-sky-400 font-medium">
                  <span className="w-2.5 h-0.5 bg-sky-400" />{" "}
                  {t("cardSpeedups")} (h) (Right Y)
                </span>
              </div>
            </div>

            {/* Dual-Axis SVG Combo Chart */}
            <div className="w-full relative py-1 border-b border-binance-hairline pb-2">
              {dailyTimeline.length === 0 ? (
                <div className="h-44 w-full flex items-center justify-center text-xs font-mono text-binance-muted">
                  {t("noBenchmarkMatches")}
                </div>
              ) : (
                <div className="relative h-44 w-full select-none">
                  <svg
                    className="w-full h-full overflow-visible"
                    viewBox={`0 0 ${svgViewWidth} ${svgViewHeight}`}
                    preserveAspectRatio="none"
                  >
                    <defs>
                      <linearGradient id="apBarGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#10b981" stopOpacity="0.75" />
                        <stop offset="100%" stopColor="#10b981" stopOpacity="0.15" />
                      </linearGradient>
                      <linearGradient id="spdAreaGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#38bdf8" stopOpacity="0.25" />
                        <stop offset="100%" stopColor="#38bdf8" stopOpacity="0.0" />
                      </linearGradient>
                    </defs>

                    {/* Horizontal Reference Grid Lines */}
                    <line
                      x1={chartPadLeft - 8}
                      y1={chartTopY}
                      x2={svgViewWidth - chartPadRight + 8}
                      y2={chartTopY}
                      stroke="#2b313a"
                      strokeDasharray="3,3"
                      strokeOpacity="0.5"
                    />
                    <line
                      x1={chartPadLeft - 8}
                      y1={(chartTopY + chartBaselineY) / 2}
                      x2={svgViewWidth - chartPadRight + 8}
                      y2={(chartTopY + chartBaselineY) / 2}
                      stroke="#2b313a"
                      strokeDasharray="3,3"
                      strokeOpacity="0.4"
                    />
                    <line
                      x1={chartPadLeft - 8}
                      y1={chartBaselineY}
                      x2={svgViewWidth - chartPadRight + 8}
                      y2={chartBaselineY}
                      stroke="#2b313a"
                      strokeOpacity="0.75"
                    />

                    {/* Left Y Axis Labels (AP Consumed - Soft Green) */}
                    <text
                      x={chartPadLeft - 10}
                      y={chartTopY + 3}
                      fill="#34d399"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="end"
                    >
                      {maxApDaily >= 1000 ? `${(maxApDaily / 1000).toFixed(0)}k` : maxApDaily}
                    </text>
                    <text
                      x={chartPadLeft - 10}
                      y={(chartTopY + chartBaselineY) / 2 + 3}
                      fill="#34d399"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="end"
                    >
                      {maxApDaily >= 2000 ? `${(maxApDaily / 2000).toFixed(0)}k` : `${Math.round(maxApDaily / 2)}`}
                    </text>
                    <text
                      x={chartPadLeft - 10}
                      y={chartBaselineY}
                      fill="#34d399"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="end"
                    >
                      0
                    </text>

                    {/* Right Y Axis Labels (Speedup Hours - Soft Blue) */}
                    <text
                      x={svgViewWidth - chartPadRight + 10}
                      y={chartTopY + 3}
                      fill="#38bdf8"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="start"
                    >
                      {`${maxSpdDaily.toFixed(0)}h`}
                    </text>
                    <text
                      x={svgViewWidth - chartPadRight + 10}
                      y={(chartTopY + chartBaselineY) / 2 + 3}
                      fill="#38bdf8"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="start"
                    >
                      {`${(maxSpdDaily / 2).toFixed(1)}h`}
                    </text>
                    <text
                      x={svgViewWidth - chartPadRight + 10}
                      y={chartBaselineY}
                      fill="#38bdf8"
                      fontSize="9"
                      fontFamily="monospace"
                      textAnchor="start"
                    >
                      0h
                    </text>

                    {/* Histogram Bars (AP Consumed - Soft Green) */}
                    {comboChartPoints.map((pt) => {
                      const isHovered = hoveredDayIdx === pt.idx;
                      return (
                        <rect
                          key={`bar-${pt.idx}`}
                          x={pt.x - pt.barWidth / 2}
                          y={pt.barY}
                          width={pt.barWidth}
                          height={pt.barH}
                          rx={2}
                          fill="url(#apBarGrad)"
                          stroke={isHovered ? "#6ee7b7" : "#10b981"}
                          strokeWidth={isHovered ? 1.5 : 1}
                          strokeOpacity={isHovered ? 1 : 0.7}
                          className="transition-all duration-150 cursor-pointer"
                          onMouseEnter={() => setHoveredDayIdx(pt.idx)}
                          onMouseLeave={() => setHoveredDayIdx(null)}
                        />
                      );
                    })}

                    {/* Yield Curve Gradient Area */}
                    {comboAreaPath && (
                      <path
                        d={comboAreaPath}
                        fill="url(#spdAreaGrad)"
                        className="pointer-events-none"
                      />
                    )}

                    {/* Yield Curve Line (Speedup Hours - Soft Blue) */}
                    {comboLinePath && (
                      <path
                        d={comboLinePath}
                        fill="none"
                        stroke="#38bdf8"
                        strokeWidth="2.2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        className="pointer-events-none"
                      />
                    )}

                    {/* Data Points / Glow Nodes */}
                    {comboChartPoints.map((pt) => {
                      const isHovered = hoveredDayIdx === pt.idx;
                      return (
                        <g key={`dot-${pt.idx}`}>
                          {/* Active vertical guideline on hover */}
                          {isHovered && (
                            <line
                              x1={pt.x}
                              y1={chartTopY}
                              x2={pt.x}
                              y2={chartBaselineY}
                              stroke="#38bdf8"
                              strokeWidth="1"
                              strokeDasharray="2,2"
                              strokeOpacity="0.8"
                              className="pointer-events-none"
                            />
                          )}
                          <circle
                            cx={pt.x}
                            cy={pt.lineY}
                            r={isHovered ? 5.5 : 3.5}
                            fill="#38bdf8"
                            stroke="#181a20"
                            strokeWidth={isHovered ? 2.5 : 1.5}
                            className="cursor-pointer transition-all duration-150"
                            onMouseEnter={() => setHoveredDayIdx(pt.idx)}
                            onMouseLeave={() => setHoveredDayIdx(null)}
                          />
                        </g>
                      );
                    })}

                    {/* X Axis Date Labels */}
                    {comboChartPoints.map((pt) => {
                      const isHovered = hoveredDayIdx === pt.idx;
                      return (
                        <text
                          key={`lbl-${pt.idx}`}
                          x={pt.x}
                          y={chartBaselineY + 16}
                          fill={isHovered ? "#eaecef" : "#848e9c"}
                          fontWeight={isHovered ? "bold" : "normal"}
                          fontSize="9"
                          fontFamily="monospace"
                          textAnchor="middle"
                          className="cursor-pointer transition-colors"
                          onMouseEnter={() => setHoveredDayIdx(pt.idx)}
                          onMouseLeave={() => setHoveredDayIdx(null)}
                        >
                          {pt.item.date_label || "-"}
                        </text>
                      );
                    })}
                  </svg>

                  {/* Floating Tooltip Box */}
                  {hoveredDayIdx !== null && comboChartPoints[hoveredDayIdx] && (
                    (() => {
                      const pt = comboChartPoints[hoveredDayIdx];
                      const leftPct = (pt.x / svgViewWidth) * 100;
                      return (
                        <div
                          style={{
                            left: `${Math.max(8, Math.min(84, leftPct))}%`,
                            top: "-12px",
                            transform: "translateX(-50%)",
                          }}
                          className="absolute bg-binance-elevated border border-binance-hairline text-binance-body text-[10px] font-mono px-3 py-2 rounded-sm shadow-2xl z-30 pointer-events-none whitespace-nowrap min-w-[140px]"
                        >
                          <div className="font-bold text-binance-yellow border-b border-binance-hairline pb-1 mb-1.5 flex items-center justify-between gap-3">
                            <span>{pt.item.date_label || "-"}</span>
                            <span className="text-emerald-400 font-normal">
                              {pt.yieldMinPer1k > 0
                                ? `${pt.yieldMinPer1k.toFixed(0)}m / 1k AP`
                                : "0m / 1k AP"}
                            </span>
                          </div>
                          <div className="flex items-center justify-between gap-3 text-emerald-400">
                            <span>{t("apConsumed")}:</span>
                            <span className="font-bold">
                              {pt.apConsumed.toLocaleString()} AP
                            </span>
                          </div>
                          <div className="flex items-center justify-between gap-3 text-sky-400">
                            <span>{t("cardSpeedups")}:</span>
                            <span className="font-bold">
                              {pt.spd.toFixed(1)}h
                            </span>
                          </div>
                        </div>
                      );
                    })()
                  )}
                </div>
              )}
            </div>

            <div className="grid grid-cols-3 gap-2 pt-2.5 text-center text-[10px] font-mono text-binance-muted">
              <div>
                <span>{t("apSpeedupYield")}:</span>{" "}
                <span className="text-binance-green font-bold">
                  {((ap.speedup_mins_per_1k_ap || 0) / 60).toFixed(1)}h
                </span>
              </div>
              <div>
                <span>{t("apGemYield")}:</span>{" "}
                <span className="text-binance-yellow font-bold">
                  {(ap.gems_per_1k_ap || 0).toFixed(0)}
                </span>
              </div>
              <div>
                <span>{t("apRecoveryRatio")}:</span>{" "}
                <span className="text-binance-green font-bold">
                  +{(ap.rebate_pct || 0).toFixed(1)}%
                </span>
              </div>
            </div>
          </div>

          {/* Target Efficiency Benchmarks Table */}
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-3">
            <div className="flex items-center justify-between text-xs pb-1 border-b border-binance-hairline">
              <div className="flex items-center gap-2">
                <Crosshair className="w-4 h-4 text-binance-yellow" />
                <span className="font-semibold text-binance-body">
                  {t("apBenchmarks")}
                </span>
                <span className="px-1.5 py-0.5 rounded text-[10px] bg-binance-elevated border border-binance-hairline text-binance-muted font-mono">
                  {filteredBenchmarks.length}
                </span>
              </div>
              <span className="text-[10px] font-mono text-binance-muted">
                {t("rankedByEfficiency")}
              </span>
            </div>

            {/* Filter Tabs & Quick Search Toolbar */}
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-1 bg-binance-elevated/70 p-0.5 rounded-md border border-binance-hairline text-[11px]">
                <button
                  onClick={() => setBmTypeFilter("all")}
                  className={`px-2.5 py-1 rounded transition font-medium ${
                    bmTypeFilter === "all"
                      ? "bg-binance-yellowSolid text-binance-ink font-semibold shadow-xs"
                      : "text-binance-muted hover:text-binance-body"
                  }`}
                >
                  {t("typeAll")}
                </button>
                <button
                  onClick={() => setBmTypeFilter("fort")}
                  className={`flex items-center gap-1 px-2.5 py-1 rounded transition font-medium ${
                    bmTypeFilter === "fort"
                      ? "bg-indigo-500 text-white font-semibold shadow-xs"
                      : "text-binance-muted hover:text-indigo-400"
                  }`}
                >
                  <Shield className="w-3 h-3" />
                  <span>{t("typeForts")}</span>
                </button>
                <button
                  onClick={() => setBmTypeFilter("motte")}
                  className={`flex items-center gap-1 px-2.5 py-1 rounded transition font-medium ${
                    bmTypeFilter === "motte"
                      ? "bg-amber-500 text-binance-ink font-semibold shadow-xs"
                      : "text-binance-muted hover:text-amber-400"
                  }`}
                >
                  <Crosshair className="w-3 h-3" />
                  <span>{t("typeMotte")}</span>
                </button>
                <button
                  onClick={() => setBmTypeFilter("barbarian")}
                  className={`flex items-center gap-1 px-2.5 py-1 rounded transition font-medium ${
                    bmTypeFilter === "barbarian"
                      ? "bg-rose-500 text-white font-semibold shadow-xs"
                      : "text-binance-muted hover:text-rose-400"
                  }`}
                >
                  <Swords className="w-3 h-3" />
                  <span>{t("typeBarbarians")}</span>
                </button>
              </div>

              {/* Quick Search */}
              <div className="relative flex items-center text-xs">
                <Search className="w-3.5 h-3.5 absolute left-2 text-binance-muted pointer-events-none" />
                <input
                  type="text"
                  value={bmSearch}
                  onChange={(e) => setBmSearch(e.target.value)}
                  placeholder={t("searchBenchmarksPlaceholder")}
                  className="pl-7 pr-6 py-1 w-36 sm:w-48 rounded bg-binance-elevated/70 border border-binance-hairline text-[11px] text-binance-body placeholder-binance-muted/60 focus:outline-none focus:border-binance-yellow/60 font-mono transition"
                />
                {bmSearch && (
                  <button
                    onClick={() => setBmSearch("")}
                    className="absolute right-1.5 p-0.5 rounded-full hover:bg-binance-hairline text-binance-muted hover:text-binance-body"
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
            </div>

            {/* Benchmarks Table */}
            <div className="overflow-x-auto max-h-72 overflow-y-auto rounded border border-binance-hairline/60">
              <table className="w-full text-left text-[11px] font-mono">
                <thead className="sticky top-0 bg-binance-card z-10">
                  <tr className="border-b border-binance-hairline text-[10px] text-binance-muted uppercase">
                    {renderSortHeader(t("colTarget"), "target", "left")}
                    {renderSortHeader(t("colLevel"), "level", "center")}
                    {renderSortHeader(t("colApCost"), "ap", "center")}
                    {renderSortHeader(t("colRuns"), "runs", "center")}
                    {renderSortHeader(t("colYield1k"), "yield", "right")}
                    {renderSortHeader(t("colCovenantDrop"), "covenant", "center")}
                    {renderSortHeader(t("colRating"), "rating", "right")}
                  </tr>
                </thead>
                <tbody className="divide-y divide-binance-hairline/60">
                  {filteredBenchmarks.length === 0 ? (
                    <tr>
                      <td
                        colSpan={7}
                        className="py-8 text-center text-binance-muted text-xs font-mono"
                      >
                        {t("noBenchmarkMatches")}
                      </td>
                    </tr>
                  ) : (
                    filteredBenchmarks.map((tb, idx) => {
                      const bType = getTargetType(tb);
                      const lvl = getTargetLevel(tb);
                      const targetName = tb.target_name || "-";
                      const unitAp = tb.unit_ap_cost || 0;
                      const battleCount = tb.battle_count || 0;
                      const yieldVal = tb.speedup_mins_per_1k_ap || 0;
                      const covenantRate = tb.covenant_drop_rate || 0;
                      const badge = String(tb.tier_badge || "TIER B");

                      return (
                        <tr
                          key={idx}
                          className="hover:bg-binance-elevated/70 transition-colors group"
                        >
                          {/* Target Name + Clean Type Tag (NO EMOJI) */}
                          <td className="py-2 px-2 text-binance-body font-medium truncate max-w-[170px]">
                            <div className="flex items-center gap-1.5">
                              <span
                                className={`px-1.5 py-0.5 rounded text-[9px] font-bold tracking-wider font-mono uppercase ${
                                  bType === "fort"
                                    ? "bg-indigo-500/15 text-indigo-400 border border-indigo-500/30"
                                    : bType === "motte"
                                    ? "bg-amber-500/15 text-amber-400 border border-amber-500/30"
                                    : bType === "barbarian"
                                    ? "bg-rose-500/15 text-rose-400 border border-rose-500/30"
                                    : "bg-binance-elevated text-binance-muted border border-binance-hairline"
                                }`}
                              >
                                {bType === "fort"
                                  ? "FORT"
                                  : bType === "motte"
                                  ? "MOTTE"
                                  : bType === "barbarian"
                                  ? "BARB"
                                  : (tb.battle_type || "TARGET").toUpperCase()}
                              </span>
                              <span className="truncate text-xs font-semibold">
                                {targetName}
                              </span>
                            </div>
                          </td>

                          {/* Level */}
                          <td className="py-2 text-center">
                            <span className="px-1.5 py-0.5 rounded bg-binance-elevated/90 border border-binance-hairline text-binance-body text-[10px] font-mono font-bold">
                              {lvl > 0 ? `Lvl ${lvl}` : "-"}
                            </span>
                          </td>

                          {/* AP Cost */}
                          <td className="py-2 text-center text-binance-muted text-[11px] font-mono">
                            <div className="inline-flex items-center gap-0.5">
                              <Zap className="w-3 h-3 text-binance-yellow/70" />
                              <span>{unitAp}</span>
                            </div>
                          </td>

                          {/* Runs */}
                          <td className="py-2 text-center text-binance-yellow font-bold text-xs font-mono">
                            {battleCount}
                          </td>

                          {/* Speedup Yield / 1k AP + Progress Bar Meter */}
                          <td className="py-2 pr-2 text-right">
                            <div className="inline-flex items-center justify-end gap-2">
                              <span className="text-binance-green font-bold text-xs font-mono">
                                {yieldVal.toFixed(0)}m
                              </span>
                              <div className="w-14 h-1.5 bg-binance-elevated rounded-full overflow-hidden border border-binance-hairline/60">
                                <div
                                  className="h-full bg-gradient-to-r from-binance-yellow to-binance-green rounded-full transition-all duration-300"
                                  style={{
                                    width: `${Math.min(
                                      100,
                                      Math.max(
                                        8,
                                        (yieldVal / maxBenchmarkYield) * 100
                                      )
                                    )}%`,
                                  }}
                                />
                              </div>
                            </div>
                          </td>

                          {/* Covenant Drop Rate */}
                          <td className="py-2 text-center text-[11px] font-mono">
                            {covenantRate > 0 ? (
                              <div className="inline-flex items-center gap-1.5 text-binance-yellow font-medium">
                                <span className="w-1.5 h-1.5 rounded-full bg-binance-green animate-pulse" />
                                <span>
                                  {covenantRate.toFixed(1)}%
                                </span>
                              </div>
                            ) : (
                              <span className="text-binance-muted/40">-</span>
                            )}
                          </td>

                          {/* Rating Tier Badge */}
                          <td className="py-2 pr-2 text-right">
                            <span
                              className={`px-1.5 py-0.5 rounded text-[9px] font-mono font-bold uppercase tracking-wider ${
                                badge.includes("S+")
                                  ? "bg-binance-yellowSolid text-binance-ink font-extrabold shadow-xs"
                                  : badge.includes("S")
                                  ? "bg-binance-green/20 text-binance-green border border-binance-green/40"
                                  : badge.includes("A")
                                  ? "bg-sky-500/15 text-sky-400 border border-sky-500/30 font-medium"
                                  : "bg-binance-card text-binance-muted border border-binance-hairline"
                              }`}
                            >
                              {badge}
                            </span>
                          </td>
                        </tr>
                      );
                    })
                  )}
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
              {topItemsList.slice(0, 10).map((it, idx) => {
                const totalQty = it.total_quantity || 0;
                const volWidthPct = (totalQty / maxItemQty) * 100;

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
                          <ResourceIcon
                            resource={it}
                            size="lg"
                            className="shrink-0"
                          />
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
                          {totalQty.toLocaleString()}
                        </span>
                      </div>

                      <div className="flex items-center justify-between text-[10px] text-binance-muted font-mono">
                        <span className="text-binance-green">
                          {t("lblDropRate")}: {(it.drop_rate || 0).toFixed(1)}%
                        </span>
                        <span>
                          {t("lblAvgRun")}: {(it.avg_per_battle || 0).toFixed(1)}
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
              {(d.latest_battles || []).map((b: BattleRecord, idx) => {
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
