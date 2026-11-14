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
  ChevronLeft,
  ZoomIn,
  ZoomOut,
  RotateCcw,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n, formatSpeedupMinutes } from "../i18n";
import { BattleRecord } from "../types";
import { ResourceIcon } from "../components/ResourceIcon";
import { CommanderCardPair } from "@/components/CommanderCardPair";

const formatKp = (val: number) => {
  if (val >= 1_000_000_000) return `${(val / 1_000_000_000).toFixed(2)}B`;
  if (val >= 1_000_000) return `${(val / 1_000_000).toFixed(2)}M`;
  if (val >= 1_000) return `${(val / 1_000).toFixed(1)}k`;
  return val.toLocaleString();
};

const buildSplinePath = (pts: Array<{ x: number; y: number }>) => {
  if (pts.length === 0) return "";
  if (pts.length === 1) return `M ${pts[0].x - 15} ${pts[0].y} L ${pts[0].x + 15} ${pts[0].y}`;
  let d = `M ${pts[0].x} ${pts[0].y}`;
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[i];
    const p1 = pts[i + 1];
    const mx = (p0.x + p1.x) / 2;
    d += ` C ${mx} ${p0.y}, ${mx} ${p1.y}, ${p1.x} ${p1.y}`;
  }
  return d;
};

const buildAreaPath = (
  pts: Array<{ x: number; y: number }>,
  baselineY: number,
  linePath: string
) => {
  if (pts.length <= 1 || !linePath) return "";
  const first = pts[0];
  const last = pts[pts.length - 1];
  return `${linePath} L ${last.x} ${baselineY} L ${first.x} ${baselineY} Z`;
};

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

export const ResponsiveChartSvg: React.FC<
  React.SVGProps<SVGSVGElement>
> = ({ children, ...props }) => (
  <svg {...props} preserveAspectRatio="xMidYMid meet">
    {children}
  </svg>
);

export const PvpChartNode: React.FC<
  React.SVGProps<SVGCircleElement> & { hasEngagement: boolean }
> = ({ hasEngagement, ...props }) => (
  <circle {...props} stroke={hasEngagement ? "#a855f7" : "#0b0e11"} />
);

const PVP_CHART_MIN_VISIBLE_POINTS = 3;
const PVP_CHART_MAX_ZOOM = 8;

export const getPvpChartRange = (
  pointCount: number,
  zoom: number,
  centerIndex = (pointCount - 1) / 2
): { start: number; end: number } => {
  const count = Math.max(0, Math.floor(pointCount));
  if (count === 0) return { start: 0, end: 0 };

  const safeZoom = Number.isFinite(zoom)
    ? Math.min(PVP_CHART_MAX_ZOOM, Math.max(1, zoom))
    : 1;
  const visibleCount = Math.min(
    count,
    Math.max(Math.min(PVP_CHART_MIN_VISIBLE_POINTS, count), Math.ceil(count / safeZoom))
  );
  const safeCenter = Number.isFinite(centerIndex)
    ? Math.min(count - 1, Math.max(0, centerIndex))
    : (count - 1) / 2;
  const maxStart = count - visibleCount;
  const start = Math.min(
    maxStart,
    Math.max(0, Math.round(safeCenter - visibleCount / 2))
  );

  return { start, end: start + visibleCount };
};

export const DashboardView: React.FC = () => {
  const {
    dashboard,
    language,
    setTab,
    setSelectedBattle,
    speedupUnit,
    mode,
    updateFilter,
  } = useAppStore();
  const { t } = useI18n(language);
  const isPvpMode = mode === "pvp";

  const [hoveredDayIdx, setHoveredDayIdx] = useState<number | null>(null);
  const [hoveredPvpIdx, setHoveredPvpIdx] = useState<number | null>(null);
  const [apViewMode, setApViewMode] = useState<"daily" | "weekly">("daily");
  const [apZoom, setApZoom] = useState(1);
  const [apZoomCenter, setApZoomCenter] = useState<number | undefined>(undefined);
  const [pvpViewMode, setPvpViewMode] = useState<"auto" | "hourly" | "daily" | "weekly">("auto");
  const [pvpZoom, setPvpZoom] = useState(1);
  const [pvpZoomCenter, setPvpZoomCenter] = useState<number | undefined>(
    undefined
  );
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

  // Transform AP timeline according to view mode
  const processedApTimeline = useMemo(() => {
    if (dailyTimeline.length === 0) return [];

    if (apViewMode === "weekly") {
      const chunks: typeof dailyTimeline = [];
      for (let i = 0; i < dailyTimeline.length; i += 7) {
        const slice = dailyTimeline.slice(i, i + 7);
        const startLabel = slice[0].date_label;
        const endLabel = slice[slice.length - 1].date_label;
        chunks.push({
          date_label:
            startLabel === endLabel ? startLabel : `${startLabel} - ${endLabel}`,
          ap_consumed: slice.reduce((sum, p) => sum + (p.ap_consumed || 0), 0),
          speedup_hours:
            Math.round(
              slice.reduce((sum, p) => sum + (p.speedup_hours || 0), 0) * 10
            ) / 10,
        });
      }
      return chunks;
    }

    return dailyTimeline;
  }, [dailyTimeline, apViewMode]);

  const apRange = getPvpChartRange(
    processedApTimeline.length,
    apZoom,
    apZoomCenter
  );

  const visibleApTimeline = useMemo(
    () => processedApTimeline.slice(apRange.start, apRange.end),
    [processedApTimeline, apRange.start, apRange.end]
  );

  const maxApDaily = useMemo(() => {
    if (visibleApTimeline.length === 0) return 1000;
    return Math.max(
      1000,
      ...visibleApTimeline.map((p) => p.ap_consumed || 0)
    );
  }, [visibleApTimeline]);

  const maxSpdDaily = useMemo(() => {
    if (visibleApTimeline.length === 0) return 1;
    return Math.max(
      1,
      ...visibleApTimeline.map((p) => p.speedup_hours || 0)
    );
  }, [visibleApTimeline]);

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
    if (visibleApTimeline.length === 0) return [];
    const count = visibleApTimeline.length;
    const stepX = count > 1 ? chartPlotWidth / (count - 1) : chartPlotWidth / 2;

    return visibleApTimeline.map((item, idx) => {
      const x = count > 1 ? chartPadLeft + idx * stepX : svgViewWidth / 2;
      const spd = item.speedup_hours || 0;
      const apConsumed = item.ap_consumed || 0;
      const lineY = chartBaselineY - (spd / maxSpdDaily) * chartAvailH;
      const barH = Math.max(3, (apConsumed / maxApDaily) * chartAvailH);
      const barY = chartBaselineY - barH;
      const barWidth = Math.max(10, Math.min(32, (chartPlotWidth / count) * 0.52));
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
  }, [visibleApTimeline, maxApDaily, maxSpdDaily, chartPlotWidth, chartAvailH]);

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

  const visibleApPointCount = apRange.end - apRange.start;
  const canZoomInAp =
    visibleApPointCount >
    Math.min(PVP_CHART_MIN_VISIBLE_POINTS, processedApTimeline.length);
  const apEffectiveZoom =
    visibleApPointCount > 0
      ? processedApTimeline.length / visibleApPointCount
      : 1;
  const canPanApPrevious = apRange.start > 0;
  const canPanApNext = apRange.end < processedApTimeline.length;

  const zoomApChart = (factor: number) => {
    const nextZoom = Math.min(
      PVP_CHART_MAX_ZOOM,
      Math.max(1, apZoom * factor)
    );
    const currentCenter = (apRange.start + apRange.end) / 2;
    setApZoom(nextZoom);
    setApZoomCenter(currentCenter);
    setHoveredDayIdx(null);
  };

  const panApChart = (direction: -1 | 1) => {
    const shift = Math.max(1, Math.floor(visibleApPointCount * 0.75));
    const currentCenter = (apRange.start + apRange.end) / 2;
    setApZoomCenter(currentCenter + direction * shift);
    setHoveredDayIdx(null);
  };

  const resetApChartZoom = () => {
    setApZoom(1);
    setApZoomCenter(undefined);
    setHoveredDayIdx(null);
  };

  const pvpStats = dashboard?.pvp_stats;
  const rawPvpTimeline = useMemo(() => pvpStats?.timeline || [], [pvpStats]);
  const isActuallyHourly = pvpStats?.is_hourly ?? false;

  const processedPvpTimeline = useMemo(() => {
    if (rawPvpTimeline.length === 0) return [];

    const effectiveMode =
      pvpViewMode === "auto"
        ? isActuallyHourly
          ? "hourly"
          : "daily"
        : pvpViewMode;

    if (effectiveMode === "weekly") {
      if (!isActuallyHourly) {
        const chunks: typeof rawPvpTimeline = [];
        for (let i = 0; i < rawPvpTimeline.length; i += 7) {
          const slice = rawPvpTimeline.slice(i, i + 7);
          const startLabel = slice[0].label;
          const endLabel = slice[slice.length - 1].label;
          const killsDealt = slice.reduce((s, p) => s + (p.kills_dealt || 0), 0);
          const killsTaken = slice.reduce((s, p) => s + (p.kills_taken || 0), 0);
          const kpGained = slice.reduce((s, p) => s + (p.kp_gained || 0), 0);
          const kpLost = slice.reduce((s, p) => s + (p.kp_lost || 0), 0);
          const t4Dealt = slice.reduce((s, p) => s + (p.kills_t4_dealt || 0), 0);
          const t5Dealt = slice.reduce((s, p) => s + (p.kills_t5_dealt || 0), 0);
          const t4Taken = slice.reduce((s, p) => s + (p.kills_t4_taken || 0), 0);
          const t5Taken = slice.reduce((s, p) => s + (p.kills_t5_taken || 0), 0);
          const battleCount = slice.reduce((s, p) => s + (p.battle_count || 0), 0);
          const ratio =
            killsTaken > 0
              ? Math.round((killsDealt / killsTaken) * 1000) / 10
              : kpLost > 0
              ? Math.round((kpGained / kpLost) * 1000) / 10
              : killsDealt > 0
              ? 100
              : 0;
          chunks.push({
            label: startLabel === endLabel ? startLabel : `${startLabel} - ${endLabel}`,
            timestamp: slice[0]?.timestamp || 0,
            kills_dealt: killsDealt,
            kills_taken: killsTaken,
            kills_t4_dealt: t4Dealt,
            kills_t5_dealt: t5Dealt,
            kills_t4_taken: t4Taken,
            kills_t5_taken: t5Taken,
            kp_gained: kpGained,
            kp_lost: kpLost,
            trade_ratio_pct: ratio,
            battle_count: battleCount,
          });
        }
        return chunks;
      } else {
        const chunks: typeof rawPvpTimeline = [];
        for (let i = 0; i < rawPvpTimeline.length; i += 6) {
          const slice = rawPvpTimeline.slice(i, i + 6);
          const killsDealt = slice.reduce((s, p) => s + (p.kills_dealt || 0), 0);
          const killsTaken = slice.reduce((s, p) => s + (p.kills_taken || 0), 0);
          const kpGained = slice.reduce((s, p) => s + (p.kp_gained || 0), 0);
          const kpLost = slice.reduce((s, p) => s + (p.kp_lost || 0), 0);
          const t4Dealt = slice.reduce((s, p) => s + (p.kills_t4_dealt || 0), 0);
          const t5Dealt = slice.reduce((s, p) => s + (p.kills_t5_dealt || 0), 0);
          const t4Taken = slice.reduce((s, p) => s + (p.kills_t4_taken || 0), 0);
          const t5Taken = slice.reduce((s, p) => s + (p.kills_t5_taken || 0), 0);
          const battleCount = slice.reduce((s, p) => s + (p.battle_count || 0), 0);
          const ratio =
            killsTaken > 0
              ? Math.round((killsDealt / killsTaken) * 1000) / 10
              : kpLost > 0
              ? Math.round((kpGained / kpLost) * 1000) / 10
              : killsDealt > 0
              ? 100
              : 0;
          chunks.push({
            label: `${slice[0].label}-${slice[slice.length - 1].label}`,
            timestamp: slice[0]?.timestamp || 0,
            kills_dealt: killsDealt,
            kills_taken: killsTaken,
            kills_t4_dealt: t4Dealt,
            kills_t5_dealt: t5Dealt,
            kills_t4_taken: t4Taken,
            kills_t5_taken: t5Taken,
            kp_gained: kpGained,
            kp_lost: kpLost,
            trade_ratio_pct: ratio,
            battle_count: battleCount,
          });
        }
        return chunks;
      }
    }

    if (effectiveMode === "daily" && isActuallyHourly) {
      const chunks: typeof rawPvpTimeline = [];
      for (let i = 0; i < rawPvpTimeline.length; i += 4) {
        const slice = rawPvpTimeline.slice(i, i + 4);
        const killsDealt = slice.reduce((s, p) => s + (p.kills_dealt || 0), 0);
        const killsTaken = slice.reduce((s, p) => s + (p.kills_taken || 0), 0);
        const kpGained = slice.reduce((s, p) => s + (p.kp_gained || 0), 0);
        const kpLost = slice.reduce((s, p) => s + (p.kp_lost || 0), 0);
        const t4Dealt = slice.reduce((s, p) => s + (p.kills_t4_dealt || 0), 0);
        const t5Dealt = slice.reduce((s, p) => s + (p.kills_t5_dealt || 0), 0);
        const t4Taken = slice.reduce((s, p) => s + (p.kills_t4_taken || 0), 0);
        const t5Taken = slice.reduce((s, p) => s + (p.kills_t5_taken || 0), 0);
        const battleCount = slice.reduce((s, p) => s + (p.battle_count || 0), 0);
        const ratio =
          killsTaken > 0
            ? Math.round((killsDealt / killsTaken) * 1000) / 10
            : kpLost > 0
            ? Math.round((kpGained / kpLost) * 1000) / 10
            : killsDealt > 0
            ? 100
            : 0;
        chunks.push({
          label: `${slice[0].label}-${slice[slice.length - 1].label}`,
          timestamp: slice[0]?.timestamp || 0,
          kills_dealt: killsDealt,
          kills_taken: killsTaken,
          kills_t4_dealt: t4Dealt,
          kills_t5_dealt: t5Dealt,
          kills_t4_taken: t4Taken,
          kills_t5_taken: t5Taken,
          kp_gained: kpGained,
          kp_lost: kpLost,
          trade_ratio_pct: ratio,
          battle_count: battleCount,
        });
      }
      return chunks;
    }

    return rawPvpTimeline;
  }, [rawPvpTimeline, pvpViewMode, isActuallyHourly]);

  const pvpRange = getPvpChartRange(
    processedPvpTimeline.length,
    pvpZoom,
    pvpZoomCenter
  );
  const visiblePvpTimeline = useMemo(
    () => processedPvpTimeline.slice(pvpRange.start, pvpRange.end),
    [processedPvpTimeline, pvpRange.start, pvpRange.end]
  );

  const maxPvpKills = useMemo(() => {
    if (visiblePvpTimeline.length === 0) return 1000;
    return Math.max(
      1000,
      ...visiblePvpTimeline.map((p) =>
        Math.max(p.kills_dealt || 0, p.kills_taken || 0)
      )
    );
  }, [visiblePvpTimeline]);

  const maxPvpRatio = useMemo(() => {
    if (visiblePvpTimeline.length === 0) return 200;
    const maxDataRatio = Math.max(
      0,
      ...visiblePvpTimeline.map((p) => p.trade_ratio_pct || 0)
    );
    return Math.max(150, Math.ceil(maxDataRatio / 50) * 50);
  }, [visiblePvpTimeline]);

  const pvpSvgViewWidth = 700;
  const pvpSvgViewHeight = 180;
  const pvpPadLeft = 46;
  const pvpPadRight = 46;
  const pvpBaselineY = 152;
  const pvpTopY = 16;
  const pvpAvailH = pvpBaselineY - pvpTopY;
  const pvpPlotWidth = pvpSvgViewWidth - pvpPadLeft - pvpPadRight;

  const pvpChartPoints = useMemo(() => {
    if (visiblePvpTimeline.length === 0) return [];
    const count = visiblePvpTimeline.length;
    const stepX = count > 1 ? pvpPlotWidth / (count - 1) : pvpPlotWidth / 2;

    return visiblePvpTimeline.map((item, idx) => {
      const x = count > 1 ? pvpPadLeft + idx * stepX : pvpSvgViewWidth / 2;
      const dealtY =
        pvpBaselineY -
        ((item.kills_dealt || 0) / maxPvpKills) * pvpAvailH;
      const takenY =
        pvpBaselineY -
        ((item.kills_taken || 0) / maxPvpKills) * pvpAvailH;
      const ratioY =
        pvpBaselineY -
        (Math.min(item.trade_ratio_pct || 0, maxPvpRatio) / maxPvpRatio) *
          pvpAvailH;

      return {
        idx,
        item,
        x,
        dealtY,
        takenY,
        ratioY,
      };
    });
  }, [visiblePvpTimeline, maxPvpKills, maxPvpRatio, pvpPlotWidth, pvpAvailH]);

  const pvpBreakEvenY = useMemo(() => {
    return pvpBaselineY - (100 / maxPvpRatio) * pvpAvailH;
  }, [maxPvpRatio, pvpAvailH]);

  const pvpDealtLinePath = useMemo(() => {
    return buildSplinePath(pvpChartPoints.map((p) => ({ x: p.x, y: p.dealtY })));
  }, [pvpChartPoints]);

  const pvpDealtAreaPath = useMemo(() => {
    return buildAreaPath(
      pvpChartPoints.map((p) => ({ x: p.x, y: p.dealtY })),
      pvpBaselineY,
      pvpDealtLinePath
    );
  }, [pvpChartPoints, pvpDealtLinePath]);

  const pvpTakenLinePath = useMemo(() => {
    return buildSplinePath(pvpChartPoints.map((p) => ({ x: p.x, y: p.takenY })));
  }, [pvpChartPoints]);

  const pvpTakenAreaPath = useMemo(() => {
    return buildAreaPath(
      pvpChartPoints.map((p) => ({ x: p.x, y: p.takenY })),
      pvpBaselineY,
      pvpTakenLinePath
    );
  }, [pvpChartPoints, pvpTakenLinePath]);

  const pvpRatioLinePath = useMemo(() => {
    return buildSplinePath(pvpChartPoints.map((p) => ({ x: p.x, y: p.ratioY })));
  }, [pvpChartPoints]);

  const pvpXAxisLabelInterval =
    visiblePvpTimeline.length > 16
      ? 3
      : Math.max(1, Math.ceil(pvpChartPoints.length / 10));

  const visiblePvpPointCount = pvpRange.end - pvpRange.start;
  const canZoomIn =
    visiblePvpPointCount >
    Math.min(PVP_CHART_MIN_VISIBLE_POINTS, processedPvpTimeline.length);
  const pvpEffectiveZoom =
    visiblePvpPointCount > 0
      ? processedPvpTimeline.length / visiblePvpPointCount
      : 1;
  const canPanPvpPrevious = pvpRange.start > 0;
  const canPanPvpNext = pvpRange.end < processedPvpTimeline.length;

  const zoomPvpChart = (factor: number) => {
    const nextZoom = Math.min(
      PVP_CHART_MAX_ZOOM,
      Math.max(1, pvpZoom * factor)
    );
    const currentCenter = (pvpRange.start + pvpRange.end) / 2;
    setPvpZoom(nextZoom);
    setPvpZoomCenter(currentCenter);
    setHoveredPvpIdx(null);
  };

  const panPvpChart = (direction: -1 | 1) => {
    const shift = Math.max(1, Math.floor(visiblePvpPointCount * 0.75));
    const currentCenter = (pvpRange.start + pvpRange.end) / 2;
    setPvpZoomCenter(currentCenter + direction * shift);
    setHoveredPvpIdx(null);
  };

  const resetPvpChartZoom = () => {
    setPvpZoom(1);
    setPvpZoomCenter(undefined);
    setHoveredPvpIdx(null);
  };

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
      (d.gathering_count || 0) +
      (d.pvp_count || 0)
  );
  const mottePct = ((d.motte_count || 0) / totalCounts) * 100;
  const fortPct = ((d.fort_count || 0) / totalCounts) * 100;
  const barbPct = ((d.barb_count || 0) / totalCounts) * 100;
  const gatherPct = ((d.gathering_count || 0) / totalCounts) * 100;
  const pvpPct = ((d.pvp_count || 0) / totalCounts) * 100;

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
          <span className="hidden sm:inline">
            {isPvpMode ? t("pvpTotalBattles") : t("vaultEncounters")}
          </span>
          <span className="text-binance-yellow font-bold">
            {(isPvpMode
              ? (pvpStats?.total_battles || 0)
              : (d.total_battles || 0)
            ).toLocaleString()}
          </span>
        </div>
      </div>

      {/* 2. Primary 6-Metric Ribbon (PvE Only) */}
      {!isPvpMode && (
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
            <span className="flex items-center gap-1">
                <ResourceIcon
                  resource="img_icon_item_3.png"
                  size="sm"
                  className="w-4 h-4 object-contain"
                />
                <ArrowDownRight className="w-3.5 h-3.5 text-binance-red" />
            </span>
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
            <span className="flex items-center gap-1">
                <ResourceIcon
                  resource="img_icon_item_3.png"
                  size="sm"
                  className="w-4 h-4 object-contain"
                />
                <ArrowUpRight className="w-3.5 h-3.5 text-binance-green" />
            </span>
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
            <div className="text-xl sm:text-2xl font-bold font-mono text-binance-info tracking-tight">
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
      )}

      {/* 3. Target Encounter Distribution (PvE Only) */}
      {!isPvpMode && (
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
          {pvpPct > 0 && (
            <div
              style={{ width: `${pvpPct}%` }}
              className="h-full bg-purple-500 transition-all"
              title={`${t("modePvp")}: ${d.pvp_count || 0} (${(pvpPct || 0).toFixed(1)}%)`}
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
          {(d.pvp_count || 0) > 0 && (
            <span className="flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-xs bg-purple-500" />
              <span className="text-binance-body">{t("modePvp")}:</span>{" "}
              {d.pvp_count || 0} ({(pvpPct || 0).toFixed(1)}%)
            </span>
          )}
        </div>
      </div>
      )}

      {/* 4. Main 8 / 4 Statistical Matrix (PvE Only: AP Combo Chart, Target Benchmarks Table, Top Items, Latest Battles) */}
      {!isPvpMode && (
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <div className="lg:col-span-8 space-y-4">
          {/* AP Timeline Card */}
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between space-y-3">
            <div className="flex items-center justify-between text-xs border-b border-binance-hairline pb-2.5 flex-wrap gap-2">
              <div className="flex items-center gap-2.5 flex-wrap">
                <div className="flex items-center gap-1.5">
                  <TrendingUp className="w-4 h-4 text-binance-yellow" />
                  <span className="font-semibold text-binance-body">
                    {t("apTimeline")}
                  </span>
                </div>

                {/* View Mode Segmented Control */}
                <div className="flex items-center rounded bg-binance-elevated/70 p-0.5 border border-binance-hairline text-[10px] font-mono">
                  {(["daily", "weekly"] as const).map((vm) => (
                    <button
                      key={vm}
                      type="button"
                      onClick={() => {
                        setApViewMode(vm);
                        setApZoom(1);
                        setApZoomCenter(undefined);
                        setHoveredDayIdx(null);
                      }}
                      className={`px-2 py-0.5 rounded transition-all capitalize ${
                        apViewMode === vm
                          ? "bg-binance-yellow text-binance-canvas font-bold shadow-xs"
                          : "text-binance-muted hover:text-binance-body"
                      }`}
                    >
                      {vm === "daily" ? t("viewModeDaily") : t("viewModeWeekly")}
                    </button>
                  ))}
                </div>

                {/* AP Chart Zoom & Pan Controls */}
                <div
                  role="group"
                  aria-label={t("apChartZoomControls")}
                  className="flex items-center gap-0.5 rounded border border-binance-hairline bg-binance-elevated/50 px-1 py-0.5"
                >
                  <button
                    type="button"
                    onClick={() => panApChart(-1)}
                    disabled={!canPanApPrevious}
                    aria-label={t("apChartPreviousRange")}
                    title={t("apChartPreviousRange")}
                    className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-binance-yellow"
                  >
                    <ChevronLeft className="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => zoomApChart(0.5)}
                    disabled={apZoom <= 1}
                    aria-label={t("apChartZoomOut")}
                    title={t("apChartZoomOut")}
                    className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-binance-yellow"
                  >
                    <ZoomOut className="w-3.5 h-3.5" />
                  </button>
                  <span
                    aria-live="polite"
                    className="min-w-9 text-center text-[10px] font-mono text-binance-muted font-semibold"
                  >
                    {Math.round(apEffectiveZoom * 100)}%
                  </span>
                  <button
                    type="button"
                    onClick={() => zoomApChart(2)}
                    disabled={!canZoomInAp}
                    aria-label={t("apChartZoomIn")}
                    title={t("apChartZoomIn")}
                    className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-binance-yellow"
                  >
                    <ZoomIn className="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={resetApChartZoom}
                    disabled={apZoom <= 1}
                    aria-label={t("apChartZoomReset")}
                    title={t("apChartZoomReset")}
                    className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-binance-yellow"
                  >
                    <RotateCcw className="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => panApChart(1)}
                    disabled={!canPanApNext}
                    aria-label={t("apChartNextRange")}
                    title={t("apChartNextRange")}
                    className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-binance-yellow"
                  >
                    <ChevronRight className="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>

              {/* Legend Indicators */}
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
              {processedApTimeline.length === 0 ? (
                <div className="h-44 w-full flex flex-col items-center justify-center text-xs font-mono text-binance-muted gap-2">
                  <Zap className="w-8 h-8 text-binance-muted/30 stroke-1" />
                  <span>{t("noBenchmarkMatches")}</span>
                </div>
              ) : (
                <div className="relative h-44 w-full select-none">
                  <ResponsiveChartSvg
                    className="w-full h-full overflow-visible"
                    viewBox={`0 0 ${svgViewWidth} ${svgViewHeight}`}
                  >
                    <defs>
                      <linearGradient id="apBarGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#10b981" stopOpacity="0.85" />
                        <stop offset="100%" stopColor="#10b981" stopOpacity="0.15" />
                      </linearGradient>
                      <linearGradient id="spdAreaGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#38bdf8" stopOpacity="0.30" />
                        <stop offset="100%" stopColor="#38bdf8" stopOpacity="0.0" />
                      </linearGradient>
                    </defs>

                    {/* Subtle Horizontal Reference Grid Lines */}
                    {[0.0, 0.33, 0.66, 1.0].map((frac, idx) => {
                      const y = chartBaselineY - frac * chartAvailH;
                      const apVal = Math.round(frac * maxApDaily);
                      const spdVal = (frac * maxSpdDaily).toFixed(1);
                      return (
                        <g key={`ap-grid-${idx}`}>
                          <line
                            x1={chartPadLeft}
                            y1={y}
                            x2={svgViewWidth - chartPadRight}
                            y2={y}
                            stroke="#2b313a"
                            strokeWidth="0.8"
                            strokeDasharray={frac === 0 ? undefined : "3,3"}
                            strokeOpacity={frac === 0 ? 0.8 : 0.45}
                          />
                          {/* Left Y-axis Label (AP Consumed - Soft Green) */}
                          <text
                            x={chartPadLeft - 8}
                            y={y + 3}
                            fill="#34d399"
                            fontSize="8.5"
                            fontFamily="monospace"
                            textAnchor="end"
                          >
                            {apVal >= 1000 ? `${(apVal / 1000).toFixed(0)}k` : apVal}
                          </text>
                          {/* Right Y-axis Label (Speedups - Soft Blue) */}
                          <text
                            x={svgViewWidth - chartPadRight + 8}
                            y={y + 3}
                            fill="#38bdf8"
                            fontSize="8.5"
                            fontFamily="monospace"
                            textAnchor="start"
                          >
                            {`${spdVal}h`}
                          </text>
                        </g>
                      );
                    })}

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
                          rx={3}
                          fill="url(#apBarGrad)"
                          stroke={isHovered ? "#6ee7b7" : "#10b981"}
                          strokeWidth={isHovered ? 1.8 : 1}
                          strokeOpacity={isHovered ? 1 : 0.65}
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
                        strokeWidth="2.4"
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
                  </ResponsiveChartSvg>

                  {/* Floating Tooltip Box */}
                  {hoveredDayIdx !== null && comboChartPoints[hoveredDayIdx] && (
                    (() => {
                      const pt = comboChartPoints[hoveredDayIdx];
                      const leftPct = (pt.x / svgViewWidth) * 100;
                      return (
                        <div
                          style={{
                            left: `${Math.max(10, Math.min(82, leftPct))}%`,
                            top: "-14px",
                            transform: "translateX(-50%)",
                          }}
                          className="absolute bg-binance-elevated/95 backdrop-blur-md border border-binance-hairline text-binance-body text-[11px] font-mono px-3 py-2 rounded-md shadow-2xl z-30 pointer-events-none whitespace-nowrap min-w-[160px]"
                        >
                          <div className="font-bold text-binance-yellow border-b border-binance-hairline pb-1 mb-1.5 flex items-center justify-between gap-3">
                            <span>{pt.item.date_label || "-"}</span>
                            <span className="text-emerald-400 font-normal">
                              {pt.yieldMinPer1k > 0
                                ? `${pt.yieldMinPer1k.toFixed(0)}m / 1k AP`
                                : "0m / 1k AP"}
                            </span>
                          </div>
                          <div className="flex items-center justify-between gap-3 text-emerald-400 text-[10px]">
                            <span>{t("apConsumed")}:</span>
                            <span className="font-bold">
                              {pt.apConsumed.toLocaleString()} AP
                            </span>
                          </div>
                          <div className="flex items-center justify-between gap-3 text-sky-400 text-[10px]">
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
      )}

      {/* 5. Dedicated PvP Combat Analytics Section (Only when filter is PvP) */}
      {isPvpMode && (
      <div className="space-y-3 pt-2">
        {/* Section Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="p-1.5 rounded-md bg-purple-500/10 border border-purple-500/20 text-purple-400">
              <Swords className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-sm font-bold text-binance-body tracking-wider uppercase flex items-center gap-2">
                <span>{t("pvpAnalyticsTitle")}</span>
                {pvpStats && pvpStats.total_battles > 0 && (
                  <span className="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-purple-500/20 text-purple-300 border border-purple-500/30">
                    {pvpStats.total_battles.toLocaleString()} {t("recordsCount")}
                  </span>
                )}
              </h2>
              <p className="text-[11px] text-binance-muted">
                {t("pvpAnalyticsSubtitle")}
              </p>
            </div>
          </div>

          {/* Quick jump to full Battles view */}
          <button
            onClick={() => setTab(1)}
            className="text-purple-400 hover:text-purple-300 flex items-center gap-1 text-xs font-medium transition"
          >
            <span>{t("btnViewAll")}</span>
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Clean Empty State when 0 PvP Battles */}
        {!pvpStats || pvpStats.total_battles === 0 ? (
          <div className="p-8 rounded-md bg-binance-card border border-binance-hairline flex flex-col items-center justify-center text-center space-y-3">
            <div className="w-12 h-12 rounded-full bg-binance-elevated flex items-center justify-center text-binance-muted">
              <Swords className="w-6 h-6 text-purple-400/60" />
            </div>
            <div className="space-y-1">
              <div className="font-semibold text-binance-body text-sm">
                {t("pvpEmptyTitle")}
              </div>
              <div className="text-xs text-binance-muted max-w-md">
                {t("pvpEmptyDesc")}
              </div>
            </div>
            <div className="flex items-center gap-2 pt-1">
              <button
                onClick={() => updateFilter({ datePreset: "AllTime" })}
                className="px-3 py-1.5 rounded-md bg-binance-elevated hover:bg-binance-hairline text-binance-body text-xs font-medium transition"
              >
                {t("pvpViewAllTime")}
              </button>
              <button
                onClick={() => setTab(1)}
                className="px-3 py-1.5 rounded-md bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold transition"
              >
                {t("pvpViewBattles")}
              </button>
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {/* 6 Key KPI Summary Cards */}
            <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6 gap-3">
              {/* 1. Total Battles */}
              <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-purple-500/40 transition">
                <div className="text-[11px] text-binance-muted font-medium">
                  {t("pvpTotalBattles")}
                </div>
                <div className="my-1.5">
                  <div className="text-xl sm:text-2xl font-bold font-mono text-purple-300 tracking-tight">
                    {pvpStats.total_battles.toLocaleString()}
                  </div>
                </div>
                <div className="text-[10px] text-binance-muted font-mono truncate">
                  {
                    pvpStats.timeline.filter((point) => point.battle_count > 0)
                      .length
                  }{" "}
                  {pvpStats.is_hourly ? "active hours" : "active days"}
                </div>
              </div>

              {/* 2. Enemies Killed */}
              <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-green/40 transition">
                <div className="text-[11px] text-binance-muted font-medium">
                  {t("pvpKillsDealt")}
                </div>
                <div className="my-1.5">
                  <div className="text-xl sm:text-2xl font-bold font-mono text-binance-green tracking-tight">
                    {pvpStats.kills_dealt.toLocaleString()}
                  </div>
                </div>
                <div className="text-[10px] text-binance-muted font-mono truncate">
                  T4: {pvpStats.kills_t4_dealt.toLocaleString()} · T5:{" "}
                  {pvpStats.kills_t5_dealt.toLocaleString()}
                </div>
              </div>

              {/* 3. Casualties Taken */}
              <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-red/40 transition">
                <div className="text-[11px] text-binance-muted font-medium">
                  {t("pvpKillsTaken")}
                </div>
                <div className="my-1.5">
                  <div className="text-xl sm:text-2xl font-bold font-mono text-binance-red tracking-tight">
                    {pvpStats.kills_taken.toLocaleString()}
                  </div>
                </div>
                <div className="text-[10px] text-binance-muted font-mono truncate">
                  T4: {pvpStats.kills_t4_taken.toLocaleString()} · T5:{" "}
                  {pvpStats.kills_t5_taken.toLocaleString()}
                </div>
              </div>

              {/* 4. Net KP */}
              {(() => {
                const netKp = pvpStats.kp_gained - pvpStats.kp_lost;
                const isNetPositive = netKp >= 0;
                return (
                  <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
                    <div className="text-[11px] text-binance-muted font-medium flex items-center justify-between">
                      <span>{t("pvpNetKp")}</span>
                      {isNetPositive ? (
                        <ArrowUpRight className="w-3.5 h-3.5 text-binance-green" />
                      ) : (
                        <ArrowDownRight className="w-3.5 h-3.5 text-binance-red" />
                      )}
                    </div>
                    <div className="my-1.5">
                      <div
                        className={`text-xl sm:text-2xl font-bold font-mono tracking-tight ${
                          isNetPositive
                            ? "text-binance-green"
                            : "text-binance-red"
                        }`}
                      >
                        {isNetPositive ? "+" : ""}
                        {formatKp(netKp)}
                      </div>
                    </div>
                    <div className="text-[10px] text-binance-muted font-mono truncate">
                      +{formatKp(pvpStats.kp_gained)} / -{formatKp(pvpStats.kp_lost)}
                    </div>
                  </div>
                );
              })()}

              {/* 5. Overall Trade Ratio */}
              {(() => {
                const ratioPct = pvpStats.trade_ratio_pct;
                const isFavorable = ratioPct >= 100.0;
                return (
                  <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-binance-yellow/40 transition">
                    <div className="text-[11px] text-binance-muted font-medium">
                      {t("pvpTradeRatio")}
                    </div>
                    <div className="my-1.5">
                      <div
                        className={`text-xl sm:text-2xl font-bold font-mono tracking-tight ${
                          isFavorable ? "text-binance-green" : "text-binance-red"
                        }`}
                      >
                        {(ratioPct / 100).toFixed(2)}x
                      </div>
                    </div>
                    <div className="text-[10px] font-mono text-binance-yellow truncate">
                      {ratioPct.toFixed(1)}% KP efficiency
                    </div>
                  </div>
                );
              })()}

              {/* 6. Top Performing March */}
              <div className="p-3 rounded-md bg-binance-card border border-binance-hairline flex flex-col justify-between hover:border-purple-500/40 transition">
                <div className="text-[11px] text-binance-muted font-medium truncate">
                  {t("pvpTopMarch")}
                </div>
                {pvpStats.top_march ? (
                  <div className="my-1 flex items-center gap-2">
                    <div className="flex -space-x-2 shrink-0">
                      <CommanderCardPair
                        primaryId={pvpStats.top_march.primary_commander_id}
                        primaryName={pvpStats.top_march.primary_commander_name}
                        primaryIsExpertised={pvpStats.top_march.primary_is_expertised}
                        secondaryId={pvpStats.top_march.secondary_commander_id}
                        secondaryName={pvpStats.top_march.secondary_commander_name}
                        secondaryIsExpertised={pvpStats.top_march.secondary_is_expertised}
                        size="md"
                      />
                    </div>
                    <div className="min-w-0">
                      <div className="text-xs font-semibold text-binance-body truncate">
                        {pvpStats.top_march.display_pair_name}
                      </div>
                      <div className="text-[10px] font-mono text-binance-green">
                        {(pvpStats.top_march.trade_ratio_pct / 100).toFixed(2)}x
                        ratio
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="text-xs font-mono text-binance-muted my-2">
                    {t("pvpNoMarch")}
                  </div>
                )}
                <div className="text-[10px] text-binance-muted font-mono truncate">
                  {pvpStats.top_march
                    ? `${pvpStats.top_march.battle_count} battles recorded`
                    : "-"}
                </div>
              </div>
            </div>

            {/* SVG Multi-Line / Area Chart */}
            <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-3">
              <div className="flex items-center justify-between text-xs border-b border-binance-hairline pb-2.5 flex-wrap gap-2">
                <div className="flex items-center gap-2.5 flex-wrap">
                  <div className="flex items-center gap-1.5">
                    <Swords className="w-4 h-4 text-purple-400" />
                    <span className="font-semibold text-binance-body">
                      {t("pvpChartTitle")}
                    </span>
                  </div>

                  {/* View Mode Segmented Control */}
                  <div className="flex items-center rounded bg-binance-elevated/70 p-0.5 border border-binance-hairline text-[10px] font-mono">
                    {(["auto", "hourly", "daily", "weekly"] as const).map((vm) => (
                      <button
                        key={vm}
                        type="button"
                        onClick={() => {
                          setPvpViewMode(vm);
                          setPvpZoom(1);
                          setPvpZoomCenter(undefined);
                          setHoveredPvpIdx(null);
                        }}
                        className={`px-2 py-0.5 rounded transition-all capitalize ${
                          pvpViewMode === vm
                            ? "bg-purple-500 text-white font-bold shadow-xs"
                            : "text-binance-muted hover:text-binance-body"
                        }`}
                      >
                        {vm === "auto"
                          ? "Auto"
                          : vm === "hourly"
                          ? t("viewModeHourly")
                          : vm === "daily"
                          ? t("viewModeDaily")
                          : t("viewModeWeekly")}
                      </button>
                    ))}
                  </div>

                  {/* PvP Zoom & Pan Controls */}
                  <div
                    role="group"
                    aria-label={t("pvpChartZoomControls")}
                    className="flex items-center gap-0.5 rounded border border-binance-hairline bg-binance-elevated/50 px-1 py-0.5"
                  >
                    <button
                      type="button"
                      onClick={() => panPvpChart(-1)}
                      disabled={!canPanPvpPrevious}
                      aria-label={t("pvpChartPreviousRange")}
                      title={t("pvpChartPreviousRange")}
                      className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-purple-400"
                    >
                      <ChevronLeft className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => zoomPvpChart(0.5)}
                      disabled={pvpZoom <= 1}
                      aria-label={t("pvpChartZoomOut")}
                      title={t("pvpChartZoomOut")}
                      className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-purple-400"
                    >
                      <ZoomOut className="w-3.5 h-3.5" />
                    </button>
                    <span
                      aria-live="polite"
                      className="min-w-9 text-center text-[10px] font-mono text-binance-muted font-semibold"
                    >
                      {Math.round(pvpEffectiveZoom * 100)}%
                    </span>
                    <button
                      type="button"
                      onClick={() => zoomPvpChart(2)}
                      disabled={!canZoomIn}
                      aria-label={t("pvpChartZoomIn")}
                      title={t("pvpChartZoomIn")}
                      className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-purple-400"
                    >
                      <ZoomIn className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={resetPvpChartZoom}
                      disabled={pvpZoom <= 1}
                      aria-label={t("pvpChartZoomReset")}
                      title={t("pvpChartZoomReset")}
                      className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-purple-400"
                    >
                      <RotateCcw className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => panPvpChart(1)}
                      disabled={!canPanPvpNext}
                      aria-label={t("pvpChartNextRange")}
                      title={t("pvpChartNextRange")}
                      className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-purple-400"
                    >
                      <ChevronRight className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {/* Legend Indicators */}
                <div className="flex items-center gap-3 text-[10px] font-mono flex-wrap">
                  <span className="flex items-center gap-1.5 text-binance-green font-medium">
                    <span className="w-2.5 h-0.5 bg-binance-green" />{" "}
                    {t("pvpKillsDealt")} (Left Y)
                  </span>
                  <span className="flex items-center gap-1.5 text-binance-red font-medium">
                    <span className="w-2.5 h-0.5 bg-binance-red" />{" "}
                    {t("pvpKillsTaken")} (Left Y)
                  </span>
                  <span className="flex items-center gap-1.5 text-binance-yellow font-medium">
                    <span className="w-2.5 h-0.5 bg-binance-yellow" />{" "}
                    {t("pvpTradeRatio")} (Right Y)
                  </span>
                  <span className="flex items-center gap-1.5 text-binance-muted/70">
                    <span className="w-2.5 h-0.5 border-b border-dashed border-binance-muted" />{" "}
                    {t("pvpBreakEven")}
                  </span>
                  <span className="flex items-center gap-1.5 text-purple-300 font-medium">
                    <span className="w-2 h-2 rounded-full border border-purple-400" />{" "}
                    {t("pvpTotalBattles")}
                  </span>
                </div>
              </div>

              {/* Chart Canvas */}
              <div className="w-full relative py-1 border-b border-binance-hairline pb-2">
                {visiblePvpTimeline.length === 0 ? (
                  <div className="h-48 w-full flex flex-col items-center justify-center text-xs font-mono text-binance-muted gap-2">
                    <Crosshair className="w-8 h-8 text-binance-muted/30 stroke-1" />
                    <span>{t("pvpEmptyTitle")}</span>
                  </div>
                ) : (
                  <div className="relative h-48 w-full select-none">
                    <ResponsiveChartSvg
                      viewBox={`0 0 ${pvpSvgViewWidth} ${pvpSvgViewHeight}`}
                      className="w-full h-full overflow-visible"
                    >
                      <defs>
                        <linearGradient
                          id="pvpDealtGrad"
                          x1="0"
                          y1="0"
                          x2="0"
                          y2="1"
                        >
                          <stop
                            offset="0%"
                            stopColor="#0ecb81"
                            stopOpacity="0.32"
                          />
                          <stop
                            offset="100%"
                            stopColor="#0ecb81"
                            stopOpacity="0.0"
                          />
                        </linearGradient>
                        <linearGradient
                          id="pvpTakenGrad"
                          x1="0"
                          y1="0"
                          x2="0"
                          y2="1"
                        >
                          <stop
                            offset="0%"
                            stopColor="#f6465d"
                            stopOpacity="0.25"
                          />
                          <stop
                            offset="100%"
                            stopColor="#f6465d"
                            stopOpacity="0.0"
                          />
                        </linearGradient>
                      </defs>

                      {/* Subtle Horizontal Gridlines */}
                      {[0.25, 0.5, 0.75, 1.0].map((frac, gIdx) => {
                        const y = pvpBaselineY - frac * pvpAvailH;
                        const killsVal = Math.round(frac * maxPvpKills);
                        const ratioVal = Math.round(frac * maxPvpRatio);
                        return (
                          <g key={`pvp-grid-${gIdx}`}>
                            <line
                              x1={pvpPadLeft}
                              y1={y}
                              x2={pvpSvgViewWidth - pvpPadRight}
                              y2={y}
                              stroke="#2b313a"
                              strokeWidth="0.8"
                              strokeDasharray="3,3"
                              strokeOpacity="0.45"
                            />
                            <text
                              x={pvpPadLeft - 8}
                              y={y + 3}
                              fill="#5e6673"
                              fontSize="8"
                              fontFamily="monospace"
                              textAnchor="end"
                            >
                              {killsVal >= 1000
                                ? `${(killsVal / 1000).toFixed(0)}k`
                                : killsVal}
                            </text>
                            <text
                              x={pvpSvgViewWidth - pvpPadRight + 8}
                              y={y + 3}
                              fill="#5e6673"
                              fontSize="8"
                              fontFamily="monospace"
                              textAnchor="start"
                            >
                              {ratioVal}%
                            </text>
                          </g>
                        );
                      })}

                      {/* Dashed Break-Even 1.0x (100%) Reference Line */}
                      {pvpBreakEvenY >= pvpTopY &&
                        pvpBreakEvenY <= pvpBaselineY && (
                          <g>
                            <line
                              x1={pvpPadLeft}
                              y1={pvpBreakEvenY}
                              x2={pvpSvgViewWidth - pvpPadRight}
                              y2={pvpBreakEvenY}
                              stroke="#fcd535"
                              strokeWidth="1"
                              strokeDasharray="4,4"
                              strokeOpacity="0.45"
                            />
                            <text
                              x={pvpSvgViewWidth - pvpPadRight + 8}
                              y={pvpBreakEvenY + 3}
                              fill="#fcd535"
                              fontSize="8"
                              fontFamily="monospace"
                              fontWeight="bold"
                              textAnchor="start"
                              opacity="0.8"
                            >
                              1.0x
                            </text>
                          </g>
                        )}

                      {/* Area Fills */}
                      {pvpDealtAreaPath && (
                        <path
                          d={pvpDealtAreaPath}
                          fill="url(#pvpDealtGrad)"
                          className="pointer-events-none"
                        />
                      )}
                      {pvpTakenAreaPath && (
                        <path
                          d={pvpTakenAreaPath}
                          fill="url(#pvpTakenGrad)"
                          className="pointer-events-none"
                        />
                      )}

                      {/* Line Paths */}
                      {pvpDealtLinePath && (
                        <path
                          d={pvpDealtLinePath}
                          fill="none"
                          stroke="#0ecb81"
                          strokeWidth="2.2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="pointer-events-none"
                        />
                      )}
                      {pvpTakenLinePath && (
                        <path
                          d={pvpTakenLinePath}
                          fill="none"
                          stroke="#f6465d"
                          strokeWidth="2.2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="pointer-events-none"
                        />
                      )}
                      {pvpRatioLinePath && (
                        <path
                          d={pvpRatioLinePath}
                          fill="none"
                          stroke="#fcd535"
                          strokeWidth="2.5"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="pointer-events-none"
                        />
                      )}

                      {/* Nodes & Interactive Points */}
                      {pvpChartPoints.map((pt) => {
                        const isHovered = hoveredPvpIdx === pt.idx;
                        return (
                          <g key={`pvp-pt-${pt.idx}`}>
                            {isHovered && (
                              <line
                                x1={pt.x}
                                y1={pvpTopY}
                                x2={pt.x}
                                y2={pvpBaselineY}
                                stroke="#a855f7"
                                strokeWidth="1"
                                strokeDasharray="2,2"
                                strokeOpacity="0.8"
                                className="pointer-events-none"
                              />
                            )}
                            {/* Dealt Dot */}
                            <PvpChartNode
                              cx={pt.x}
                              cy={pt.dealtY}
                              r={isHovered ? 5.5 : 3}
                              fill="#0ecb81"
                              hasEngagement={pt.item.battle_count > 0}
                              strokeWidth={isHovered ? 2 : 1.5}
                              className="cursor-pointer transition-all"
                              onMouseEnter={() => setHoveredPvpIdx(pt.idx)}
                              onMouseLeave={() => setHoveredPvpIdx(null)}
                            />
                            {/* Taken Dot */}
                            <PvpChartNode
                              cx={pt.x}
                              cy={pt.takenY}
                              r={isHovered ? 5.5 : 3}
                              fill="#f6465d"
                              hasEngagement={pt.item.battle_count > 0}
                              strokeWidth={isHovered ? 2 : 1.5}
                              className="cursor-pointer transition-all"
                              onMouseEnter={() => setHoveredPvpIdx(pt.idx)}
                              onMouseLeave={() => setHoveredPvpIdx(null)}
                            />
                            {/* Ratio Dot */}
                            <PvpChartNode
                              cx={pt.x}
                              cy={pt.ratioY}
                              r={isHovered ? 6.5 : 4}
                              fill="#fcd535"
                              hasEngagement={pt.item.battle_count > 0}
                              strokeWidth={isHovered ? 2.5 : 1.5}
                              className="cursor-pointer transition-all"
                              onMouseEnter={() => setHoveredPvpIdx(pt.idx)}
                              onMouseLeave={() => setHoveredPvpIdx(null)}
                            />
                          </g>
                        );
                      })}

                      {/* X Axis Labels */}
                      {pvpChartPoints.map((pt) => {
                        if (
                          pt.idx % pvpXAxisLabelInterval !== 0 &&
                          pt.idx !== pvpChartPoints.length - 1
                        ) {
                          return null;
                        }
                        const isHovered = hoveredPvpIdx === pt.idx;
                        return (
                          <text
                            key={`pvp-lbl-${pt.idx}`}
                            x={pt.x}
                            y={pvpBaselineY + 16}
                            fill={isHovered ? "#eaecef" : "#848e9c"}
                            fontWeight={isHovered ? "bold" : "normal"}
                            fontSize="9"
                            fontFamily="monospace"
                            textAnchor="middle"
                            className="cursor-pointer transition-colors"
                            onMouseEnter={() => setHoveredPvpIdx(pt.idx)}
                            onMouseLeave={() => setHoveredPvpIdx(null)}
                          >
                            {pt.item.label}
                          </text>
                        );
                      })}
                    </ResponsiveChartSvg>

                    {/* Interactive Floating Tooltip */}
                    {hoveredPvpIdx !== null && pvpChartPoints[hoveredPvpIdx] && (
                      (() => {
                        const pt = pvpChartPoints[hoveredPvpIdx];
                        const item = pt.item;
                        const leftPct = (pt.x / pvpSvgViewWidth) * 100;
                        const alignRight = leftPct > 65;
                        const netKp = item.kp_gained - item.kp_lost;

                        return (
                          <div
                            style={{
                              left: `${leftPct}%`,
                              top: `${
                                Math.min(pt.ratioY, pt.dealtY, pt.takenY) * 0.7
                              }px`,
                              transform: alignRight
                                ? "translate(-105%, 0)"
                                : "translate(15%, 0)",
                            }}
                            className="absolute z-20 pointer-events-none p-2.5 rounded-md bg-binance-elevated/95 border border-purple-500/40 shadow-xl backdrop-blur-md text-[11px] font-mono space-y-1.5 min-w-[210px]"
                          >
                            <div className="flex items-center justify-between pb-1 border-b border-binance-hairline">
                              <span className="font-bold text-binance-body">
                                {item.label}
                              </span>
                              <span className="text-purple-400 font-semibold">
                                {item.battle_count}{" "}
                                {t("pvpTotalBattles").toLowerCase()}
                              </span>
                            </div>

                            <div className="space-y-1 text-[10px]">
                              <div className="flex items-center justify-between">
                                <span className="text-binance-green font-medium">
                                  {t("pvpKillsDealt")}:
                                </span>
                                <span className="font-bold text-binance-green">
                                  {item.kills_dealt.toLocaleString()} (T4:{" "}
                                  {item.kills_t4_dealt.toLocaleString()} · T5:{" "}
                                  {item.kills_t5_dealt.toLocaleString()})
                                </span>
                              </div>
                              <div className="flex items-center justify-between">
                                <span className="text-binance-red font-medium">
                                  {t("pvpKillsTaken")}:
                                </span>
                                <span className="font-bold text-binance-red">
                                  {item.kills_taken.toLocaleString()} (T4:{" "}
                                  {item.kills_t4_taken.toLocaleString()} · T5:{" "}
                                  {item.kills_t5_taken.toLocaleString()})
                                </span>
                              </div>
                              <div className="flex items-center justify-between pt-0.5 border-t border-binance-hairline/60">
                                <span className="text-binance-muted">
                                  {t("pvpNetKp")}:
                                </span>
                                <span
                                  className={`font-bold ${
                                    netKp >= 0
                                      ? "text-binance-green"
                                      : "text-binance-red"
                                  }`}
                                >
                                  {netKp >= 0 ? "+" : ""}
                                  {formatKp(netKp)}
                                </span>
                              </div>
                              <div className="flex items-center justify-between">
                                <span className="text-binance-yellow font-semibold">
                                  {t("pvpTradeRatio")}:
                                </span>
                                <span className="font-bold text-binance-yellow text-xs">
                                  {(item.trade_ratio_pct / 100).toFixed(2)}x (
                                  {item.trade_ratio_pct.toFixed(1)}%)
                                </span>
                              </div>
                            </div>
                          </div>
                        );
                      })()
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
      )}
    </div>
  );
};
