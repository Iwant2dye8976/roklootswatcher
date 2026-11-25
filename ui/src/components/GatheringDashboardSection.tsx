import React from "react";
import { Package, ChevronRight } from "lucide-react";
import { BattleRecord, DashboardDto, AppLanguage } from "../types";
import { useI18n } from "../i18n";
import { ResourceIcon } from "./ResourceIcon";

const formatRss = (val: number) => {
  if (val >= 1_000_000_000) return `${(val / 1_000_000_000).toFixed(2)}B`;
  if (val >= 1_000_000) return `${(val / 1_000_000).toFixed(2)}M`;
  if (val >= 1_000) return `${(val / 1_000).toFixed(1)}k`;
  return val.toLocaleString();
};

interface GatheringDashboardSectionProps {
  dashboard: DashboardDto;
  language: AppLanguage;
  onViewAllReports: () => void;
  onSelectBattle: (b: BattleRecord) => void;
}

/** Dedicated Dashboard view for Gathering mode: hero total, 5 resource cards, and recent logs. */
export const GatheringDashboardSection: React.FC<GatheringDashboardSectionProps> = ({
  dashboard: d,
  language,
  onViewAllReports,
  onSelectBattle,
}) => {
  const { t } = useI18n(language);
  const gs = d.gathering_summary ?? {
    food: 0,
    wood: 0,
    stone: 0,
    gold: 0,
    gems: 0,
    total_rss: d.total_resources || 0,
  };

  const resourceCards: Array<{
    key: "food" | "wood" | "stone" | "gold" | "gems";
    label: string;
    value: number;
    isGems?: boolean;
    accent: string;
  }> = [
    { key: "food", label: t("rssFood"), value: gs.food, accent: "hover:border-binance-green/50" },
    { key: "wood", label: t("rssWood"), value: gs.wood, accent: "hover:border-amber-400/50" },
    { key: "stone", label: t("rssStone"), value: gs.stone, accent: "hover:border-slate-400/50" },
    { key: "gold", label: t("rssGold"), value: gs.gold, accent: "hover:border-binance-yellow/50" },
    { key: "gems", label: t("rssGems"), value: gs.gems, isGems: true, accent: "hover:border-cyan-400/50" },
  ];

  return (
    <div className="space-y-4">
      {/* Hero Total RSS Card */}
      <div className="p-4 rounded-md bg-binance-card border border-binance-hairline flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-md bg-binance-green/10 border border-binance-green/20 text-binance-green">
            <Package className="w-5 h-5" />
          </div>
          <div>
            <div className="text-[11px] font-medium text-binance-muted uppercase tracking-wider">
              {t("gatheringTotalRss")}
            </div>
            <div className="flex items-baseline gap-2.5 mt-0.5">
              <span className="text-2xl sm:text-3xl font-bold font-mono text-binance-yellow tracking-tight">
                {formatRss(gs.total_rss)}
              </span>
              <span className="text-xs font-mono text-binance-muted">
                ({gs.total_rss.toLocaleString()})
              </span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2 px-3 py-1.5 rounded bg-binance-canvas border border-binance-hairline text-xs font-mono self-start sm:self-auto">
          <span className="text-binance-muted">{t("gatheringTotalTrips")}:</span>
          <span className="font-bold text-binance-green">
            {(d.gathering_count || 0).toLocaleString()}
          </span>
        </div>
      </div>

      {/* 5-Card Resource Breakdown (Food, Wood, Stone, Gold, Gems) */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
        {resourceCards.map((rc) => (
          <div
            key={rc.key}
            className={`p-3.5 rounded-md bg-binance-card border border-binance-hairline transition flex flex-col justify-between gap-2 ${rc.accent}`}
          >
            <div className="flex items-center gap-2">
              <ResourceIcon resource={rc.key} size="lg" />
              <span className="text-xs font-semibold text-binance-body">
                {rc.label}
              </span>
            </div>
            <div>
              <div className="text-xl font-bold font-mono text-binance-body tracking-tight">
                {rc.isGems ? rc.value.toLocaleString() : formatRss(rc.value)}
              </div>
              {!rc.isGems && (
                <div className="text-[10px] font-mono text-binance-muted mt-0.5">
                  {rc.value.toLocaleString()}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Recent Gathering Logs */}
      <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2.5">
        <div className="flex items-center justify-between text-xs pb-1.5 border-b border-binance-hairline">
          <div className="flex items-center gap-2">
            <Package className="w-4 h-4 text-binance-green" />
            <span className="font-semibold text-binance-body">
              {t("gatheringRecentLogs")}
            </span>
          </div>
          <button
            onClick={onViewAllReports}
            className="text-binance-yellow hover:text-binance-yellowHover flex items-center gap-0.5 text-[11px] font-medium transition"
          >
            <span>{t("btnViewAll")}</span>
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
        </div>

        {(d.latest_battles || []).length === 0 ? (
          <div className="py-6 text-center text-xs text-binance-muted">
            {t("gatheringNoLogs")}
          </div>
        ) : (
          <div className="divide-y divide-binance-hairline/60 max-h-80 overflow-y-auto pr-1">
            {(d.latest_battles || []).map((b: BattleRecord, idx) => (
              <div
                key={idx}
                onClick={() => onSelectBattle(b)}
                className="py-2 px-1.5 hover:bg-binance-elevated/40 transition cursor-pointer flex items-center justify-between gap-2.5 text-xs rounded-xs"
              >
                <div className="flex flex-col gap-0.5 min-w-0">
                  <div className="flex items-center gap-1.5">
                    <span className="px-1 py-0.2 rounded-xs text-[9px] font-mono font-bold uppercase border bg-binance-green/20 text-binance-green border-binance-green/30">
                      {b.battle_type}
                    </span>
                    <span className="font-semibold text-binance-body truncate">
                      {b.display_target || b.target}
                    </span>
                  </div>
                  <div className="text-[10px] text-binance-muted font-mono truncate">
                    {b.time_vn || b.time_utc} · ({b.pos_x}, {b.pos_y}) · {t("lblGatheringDone")}
                  </div>
                </div>

                <ChevronRight className="w-3.5 h-3.5 text-binance-muted shrink-0" />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
