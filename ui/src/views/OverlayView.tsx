import React, { useState, useEffect } from "react";
import { OverlayPayloadDto } from "../types";
import * as api from "../api/tauri";
import { CommanderCardPair } from "../components/CommanderCardPair";
import { ResourceIcon } from "../components/ResourceIcon";

export const OverlayView: React.FC = () => {
  const [data, setData] = useState<OverlayPayloadDto | null>(null);
  const [extraIdleSecs, setExtraIdleSecs] = useState<number>(0);

  useEffect(() => {
    api
      .getOverlayState()
      .then((payload) => {
        setData(payload);
        setExtraIdleSecs(0);
      })
      .catch(console.error);

    const unlistenPromise = api.onOverlayUpdated((payload) => {
      setData(payload);
      setExtraIdleSecs(0);
    });

    const timer = window.setInterval(() => {
      setExtraIdleSecs((prev) => prev + 5);
    }, 5000);

    return () => {
      window.clearInterval(timer);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  if (!data || !data.config.enabled || data.config.mode === "Hidden") {
    return null;
  }

  const { config, records, pinned_chips, troop_trades = [], today_runs } = data;
  const showChips = config.mode === "MiniHud" || config.mode === "Combined";
  const showCards = config.mode === "Cards" || config.mode === "Combined";
  const isMiniOnly = config.mode === "MiniHud";

  const formatCompactCount = (val: number): string => {
    if (val >= 1_000_000) {
      return `${(val / 1_000_000).toFixed(2)}M`;
    }
    if (val >= 1_000) {
      return `${(val / 1_000).toFixed(1)}K`;
    }
    return Math.round(val).toLocaleString();
  };

  const getRatioBadgeClass = (pct: number): string => {
    if (pct >= 120) return "text-emerald-400";
    if (pct >= 100) return "text-sky-400";
    return "text-rose-400";
  };

  const totalIdle = (data.idle_secs || 0) + extraIdleSecs;
  const userOpacity = Math.max(0.3, Math.min(1.0, config.opacity || 0.95));
  const fadeTimeout = config.fade_timeout_secs ?? 0;
  const isFaded = fadeTimeout > 0 && totalIdle >= fadeTimeout;

  let effectiveOpacity = userOpacity;
  if (isFaded) {
    if (config.fade_style === "Hide") {
      effectiveOpacity = 0.0;
    } else {
      // Dim mode: user custom dim opacity (default 0.25), capped at userOpacity
      const dimTarget = Math.max(0.05, Math.min(userOpacity, config.fade_dim_opacity ?? 0.25));
      effectiveOpacity = dimTarget;
    }
  } else if (fadeTimeout === 0 && totalIdle >= 60) {
    // Subtle idle dimming when auto-fade is disabled
    effectiveOpacity = Math.max(0.35, userOpacity * 0.92);
  }

  const baseFontPx = Math.max(8.0, Math.min(32, config.font_size || 10.5));
  const textHex = config.text_color_hex || "#F8FAFC";
  const accentHex = config.accent_color_hex || "#38BDF8";
  const isTransparentBg = !!config.transparent_bg;

  // Pure Transparent (100% transparent background) vs Transparent Color Background (75% dark translucent tint)
  const pillBgClass = isTransparentBg
    ? "bg-transparent border border-white/10"
    : "bg-[#060a14]/75 backdrop-blur-[2px] border border-slate-700/80 shadow-md";
  const cardBgClass = isTransparentBg
    ? "bg-transparent border border-slate-700/50 shadow-none"
    : "bg-[#060a14]/75 backdrop-blur-[2px] border border-slate-700/80 shadow-xl";

  return (
    <div
      style={{
        opacity: effectiveOpacity,
        pointerEvents: "none",
        userSelect: "none",
        fontSize: `${baseFontPx}px`,
        color: textHex,
        textShadow: "0 1px 2px rgba(0,0,0,0.95), 0 0 2px rgba(0,0,0,0.95)",
      }}
      className={`w-full h-full select-none font-sans antialiased overflow-hidden transition-opacity duration-700 ease-in-out ${
        isMiniOnly ? "px-1.5 py-0.5" : "p-1.5"
      }`}
    >
      <div className="flex flex-col gap-1 w-full">
        {/* Pinned Chips Ribbon (MiniHud & Combined only) */}
        {showChips && (
          <div className="flex items-center gap-1 flex-wrap">
            {/* Today Runs Counter Pill (Battle count M-F-B rendered FIRST so it never overflows or hides) */}
            <div
              className={`flex items-center gap-1 px-1.5 py-0.5 rounded font-mono text-[0.88em] text-slate-200 shrink-0 leading-none font-medium ${pillBgClass}`}
            >
              <span className="text-red-400 font-bold">{today_runs.motte}M</span>
              <span>·</span>
              <span className="text-amber-400 font-bold">{today_runs.fort}F</span>
              <span>·</span>
              <span className="text-sky-400 font-bold">{today_runs.barb}B</span>
              {today_runs.gather > 0 && (
                <>
                  <span>·</span>
                  <span className="text-emerald-400 font-bold">
                    {today_runs.gather}G
                  </span>
                </>
              )}
              {(today_runs.pvp || 0) > 0 && (
                <>
                  <span>·</span>
                  <span className="text-fuchsia-400 font-bold">
                    {today_runs.pvp}P
                  </span>
                </>
              )}
            </div>

            {/* Pinned Loot Chips */}
            {pinned_chips.map((chip) => (
              <div
                key={chip.id}
                className={`flex items-center gap-1 px-1.5 py-0.5 rounded font-mono shrink-0 leading-none ${pillBgClass}`}
              >
                <ResourceIcon
                  resource={chip.id}
                  size="xs"
                  className="w-3.5 h-3.5 object-contain shrink-0"
                />
                <span
                  style={{ color: chip.color_hex || accentHex }}
                  className="font-bold text-[0.88em]"
                >
                  {chip.label}
                </span>
                <span className="font-bold" style={{ color: textHex }}>
                  {chip.display_val}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Active Session Troop Trade Ratios (Cards & Combined mode) */}
        {showCards && config.filter === "pvp" && (
          <div className="space-y-1">
            {troop_trades.length === 0 ? (
              <div
                className={`px-2 py-1 rounded text-[0.85em] text-slate-300 font-mono italic text-center ${cardBgClass}`}
              >
                No PvP troop trades recorded today...
              </div>
            ) : (
              troop_trades
                .slice(0, 5)
                .map((troop) => {
                  const combatType = troop.combat_type || "open-field";
                  const borderClass =
                    combatType === "rally"
                      ? "border-l-rose-500"
                      : combatType === "garrison"
                      ? "border-l-sky-400"
                      : "border-l-fuchsia-400";

                  const badgeClass =
                    combatType === "rally"
                      ? "bg-rose-500/20 text-rose-300 border-rose-500/40"
                      : combatType === "garrison"
                      ? "bg-sky-500/20 text-sky-300 border-sky-500/40"
                      : "bg-fuchsia-500/20 text-fuchsia-300 border-fuchsia-500/40";

                  const badgeText =
                    combatType === "open-field"
                      ? "Field"
                      : combatType === "rally"
                      ? "Rally"
                      : "Garrison";

                  return (
                    <div
                      key={troop.troop_id}
                      className={`px-2 py-1.5 rounded border-l-[3px] ${borderClass} flex flex-col gap-0.5 leading-tight font-mono ${cardBgClass}`}
                    >
                      <div className="flex items-center justify-between gap-1 min-w-0">
                        <div className="flex items-center gap-1.5 min-w-0">
                          <CommanderCardPair
                            primaryId={troop.primary_commander_id}
                            primaryName={troop.primary_commander_name}
                            primaryIsExpertised={troop.primary_is_expertised}
                            secondaryId={troop.secondary_commander_id}
                            secondaryName={troop.secondary_commander_name}
                            secondaryIsExpertised={troop.secondary_is_expertised}
                            size="sm"
                            showWakeupBadge={true}
                            showEmblem={true}
                          />
                          <span
                            className="font-bold truncate text-[0.92em]"
                            style={{ color: textHex }}
                          >
                            {troop.display_pair_name}
                          </span>
                        </div>
                        <span
                          className={`text-[8.5px] font-bold uppercase tracking-wider px-1 py-0.5 rounded border leading-none shrink-0 ${badgeClass}`}
                        >
                          {badgeText}
                        </span>
                      </div>
                      <div className="flex items-center justify-between gap-1.5 text-[0.88em]">
                        <span className="text-amber-300 font-bold">
                          +{formatCompactCount(troop.kills_dealt ?? troop.kp_gained)} Kills
                        </span>
                        <span
                          className={`font-bold ${getRatioBadgeClass(
                            troop.trade_ratio_pct
                          )}`}
                        >
                          Ratio{" "}
                          {troop.trade_ratio_pct > 999
                            ? ">999%"
                            : `${troop.trade_ratio_pct.toFixed(0)}%`}
                        </span>
                      </div>
                    </div>
                  );
                })
            )}
          </div>
        )}

        {/* Recent Battle Drop Cards (Cards & Combined only) */}
        {showCards && config.filter !== "pvp" && (
          <div className="space-y-1">
            {records.length === 0 ? (
              <div className={`px-2 py-1.5 rounded text-[0.88em] text-slate-300 font-mono italic text-center ${cardBgClass}`}>
                Awaiting combat reports...
              </div>
            ) : (
              records
                .slice(0, 5)
                .map((rec, idx) => {
                const borderClass =
                  rec.battle_type === "motte"
                    ? "border-l-red-500"
                    : rec.battle_type === "fort"
                    ? "border-l-amber-400"
                    : rec.battle_type === "gathering"
                    ? "border-l-emerald-400"
                    : "border-l-sky-400";

                return (
                  <div
                    key={rec.id || idx}
                    className={`px-2 py-1.5 rounded border-l-[3px] ${borderClass} flex flex-col gap-0.5 leading-tight ${cardBgClass}`}
                  >
                    <div className="flex items-center justify-between gap-1">
                      <span
                        className="font-bold truncate"
                        style={{ color: textHex }}
                      >
                        {rec.display_target || rec.target}
                      </span>
                      <span className="text-[0.85em] font-mono text-slate-300 font-medium shrink-0">
                        {rec.time_vn ? rec.time_vn.slice(11, 16) : ""}
                      </span>
                    </div>

                    <div className="flex items-center justify-between text-[0.88em] font-mono text-slate-300">
                      <span>
                        ({rec.pos_x}, {rec.pos_y})
                      </span>
                      <span style={{ color: accentHex }} className="font-bold">
                        {rec.battle_type === "gathering"
                          ? rec.talent_add > 0
                            ? `+${rec.talent_add.toLocaleString()}`
                            : "OK"
                          : `${rec.damage_pct}%${
                              rec.plunder_tier > 0 ? ` · T${rec.plunder_tier}` : ""
                            }`}
                      </span>
                    </div>

                    {rec.items.length > 0 && (
                      <div className="text-[0.88em] text-amber-300 font-mono font-medium truncate">
                        +
                        {rec.items
                          .slice(0, 2)
                          .map(
                            (it) =>
                              `${it.value.toLocaleString()} ${
                                it.display_name || it.name_vi || it.name_en
                              }`
                          )
                          .join(" · ")}
                      </div>
                    )}
                  </div>
                );
              })
            )}
          </div>
        )}
      </div>
    </div>
  );
};
