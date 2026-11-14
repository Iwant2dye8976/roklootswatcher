import React, { useState, useEffect } from "react";
import { OverlayPayloadDto } from "../types";
import * as api from "../api/tauri";
import { CommanderCardPair } from "../components/CommanderCardPair";
import { ResourceIcon } from "../components/ResourceIcon";
import { getNpcSprite } from "../utils/npc";

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
  const maxCards = Math.max(1, Math.min(5, config.card_count ?? 3));

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
    if (pct >= 120) return "text-emerald-800";
    if (pct >= 100) return "text-sky-800";
    return "text-rose-800";
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
      const dimTarget = Math.max(
        0.05,
        Math.min(userOpacity, config.fade_dim_opacity ?? 0.25)
      );
      effectiveOpacity = dimTarget;
    }
  } else if (fadeTimeout === 0 && totalIdle >= 60) {
    effectiveOpacity = Math.max(0.35, userOpacity * 0.92);
  }

  const baseFontPx = Math.max(8.0, Math.min(32, config.font_size || 10.5));
  const textHex = config.text_color_hex || "#F8FAFC";
  const accentHex = config.accent_color_hex || "#38BDF8";
  const isTransparentBg = !!config.transparent_bg;

  // Ultra-compact micro ticker pill styling
  const pillBgClass = isTransparentBg
    ? "bg-black/40 border border-white/10 shadow-xs"
    : "bg-[#060a14]/85 backdrop-blur-md border border-slate-700/70 shadow-sm";

  return (
    <div
      style={{
        opacity: effectiveOpacity,
        pointerEvents: "none",
        userSelect: "none",
        fontSize: `${baseFontPx}px`,
        color: textHex,
        textShadow: "0 1px 2px rgba(0,0,0,0.85)",
      }}
      className={`w-full h-full select-none font-sans antialiased overflow-hidden ${
        isMiniOnly ? "px-1.5 py-0.5" : "p-1.5"
      }`}
    >
      <div className="flex flex-col gap-1 w-full">
        {/* Pinned Chips Ribbon (MiniHud & Combined only) - Ultra-Compact Micro Ticker */}
        {showChips && (
          <div className="flex items-center gap-1 flex-wrap">
            {/* Today Runs Counter Pill */}
            <div
              className={`flex items-center gap-1 px-1.5 py-0.5 rounded font-mono text-[0.88em] text-slate-200 shrink-0 leading-none font-medium ${pillBgClass}`}
            >
              <span className="text-red-400 font-bold">{today_runs.motte}M</span>
              <span className="text-slate-500">·</span>
              <span className="text-amber-400 font-bold">{today_runs.fort}F</span>
              <span className="text-slate-500">·</span>
              <span className="text-sky-400 font-bold">{today_runs.barb}B</span>
              {today_runs.gather > 0 && (
                <>
                  <span className="text-slate-500">·</span>
                  <span className="text-emerald-400 font-bold">
                    {today_runs.gather}G
                  </span>
                </>
              )}
              {(today_runs.pvp || 0) > 0 && (
                <>
                  <span className="text-slate-500">·</span>
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

        {/* Active Session Troop Trade Ratios (PvP Cards) */}
        {showCards && config.filter === "pvp" && (
          <div className="space-y-1">
            {troop_trades.length === 0 ? (
              <div
                className="relative rounded-md overflow-hidden px-4 py-3 pe-10 text-center text-[0.88em] font-mono italic text-slate-700 min-h-[48px] flex items-center justify-center"
                style={{
                  backgroundImage: "url('/hud/img_MailBarSelected.png')",
                  backgroundSize: "100% 100%",
                  backgroundRepeat: "no-repeat",
                  textShadow: "none",
                }}
              >
                <span>No PvP troop trades recorded today...</span>
              </div>
            ) : (
              troop_trades.slice(0, maxCards).map((troop) => {
                const combatType = troop.combat_type || "open-field";
                const badgeClass =
                  combatType === "rally"
                    ? "bg-rose-100 text-rose-900 border-rose-300"
                    : combatType === "garrison"
                    ? "bg-indigo-100 text-indigo-900 border-indigo-300"
                    : "bg-fuchsia-100 text-fuchsia-900 border-fuchsia-300";

                const badgeText =
                  combatType === "open-field"
                    ? "Field"
                    : combatType === "rally"
                    ? "Rally"
                    : "Garrison";

                return (
                  <div
                    key={troop.troop_id}
                    className="relative rounded-md overflow-hidden px-3.5 py-2 pe-10 flex items-center gap-2.5 min-h-[58px]"
                    style={{
                      backgroundImage: "url('/hud/img_MailBarSelected.png')",
                      backgroundSize: "100% 100%",
                      backgroundRepeat: "no-repeat",
                      textShadow: "none",
                    }}
                  >
                    {/* Structure: Left CommanderPair, Right 3-lines */}
                    <div className="flex items-center gap-2.5 w-full min-w-0">
                      {/* Left: Commander Pair */}
                      <div className="shrink-0 flex items-center justify-center">
                        <CommanderCardPair
                          primaryId={troop.primary_commander_id}
                          primaryName={troop.primary_commander_name}
                          primaryIsExpertised={troop.primary_is_expertised}
                          secondaryId={troop.secondary_commander_id}
                          secondaryName={troop.secondary_commander_name}
                          secondaryIsExpertised={troop.secondary_is_expertised}
                          size="sm"
                          showWakeupBadge={false}
                          showEmblem={false}
                        />
                      </div>

                      {/* Right 3-line Stack */}
                      <div className="flex flex-col justify-between flex-1 min-w-0 overflow-hidden leading-tight">
                        {/* Line 1: [Type of report] */}
                        <div className="flex items-center justify-between gap-1 min-w-0">
                          <span className="font-bold truncate text-[0.92em] text-slate-900 tracking-tight">
                            {troop.display_pair_name}
                          </span>
                          <span
                            className={`text-[8.5px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border leading-none shrink-0 ${badgeClass}`}
                          >
                            {badgeText}
                          </span>
                        </div>

                        {/* Line 2: [Statistic] */}
                        <div className="flex items-center justify-between gap-1.5 min-w-0 text-[0.88em] font-mono">
                          <span className="min-w-0 truncate text-amber-900 font-bold">
                            +
                            {formatCompactCount(
                              troop.kills_dealt ?? troop.kp_gained
                            )}{" "}
                            Kills
                          </span>
                          <span
                            className={`shrink-0 whitespace-nowrap font-bold ${getRatioBadgeClass(
                              troop.trade_ratio_pct
                            )}`}
                          >
                            Ratio{" "}
                            {troop.trade_ratio_pct > 999
                              ? ">999%"
                              : `${troop.trade_ratio_pct.toFixed(0)}%`}
                          </span>
                        </div>

                        {/* Line 3: [Date] */}
                        <div className="flex items-center justify-between gap-1 min-w-0 text-[0.80em] font-mono text-slate-600">
                          <span className="min-w-0 truncate">
                            {troop.battle_count} battle
                            {troop.battle_count > 1 ? "s" : ""}
                          </span>
                          <span className="shrink-0 text-purple-900 font-semibold">
                            PvP
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        )}

        {/* Recent Battle Drop Cards (PvE Cards) */}
        {showCards && config.filter !== "pvp" && (
          <div className="space-y-1">
            {records.length === 0 ? (
              <div
                className="relative rounded-md overflow-hidden px-4 py-3 pe-10 text-center text-[0.88em] font-mono italic text-slate-700 min-h-[48px] flex items-center justify-center"
                style={{
                  backgroundImage: "url('/hud/img_MailBarSelected.png')",
                  backgroundSize: "100% 100%",
                  backgroundRepeat: "no-repeat",
                  textShadow: "none",
                }}
              >
                <span>Awaiting combat reports...</span>
              </div>
            ) : (
              records.slice(0, maxCards).map((rec, idx) => {
                const npcSprite = getNpcSprite(
                  rec.battle_type,
                  rec.display_target || rec.target,
                  rec.target_level
                );

                const typeBadgeClass =
                  rec.battle_type === "motte"
                    ? "bg-red-100 text-red-900 border-red-300"
                    : rec.battle_type === "fort"
                    ? "bg-amber-100 text-amber-900 border-amber-300"
                    : rec.battle_type === "gathering"
                    ? "bg-emerald-100 text-emerald-900 border-emerald-300"
                    : "bg-sky-100 text-sky-900 border-sky-300";

                const typeBadgeLabel =
                  rec.battle_type === "motte"
                    ? "Motte"
                    : rec.battle_type === "fort"
                    ? "Fort"
                    : rec.battle_type === "gathering"
                    ? "Gather"
                    : "Barb";

                return (
                  <div
                    key={rec.id || idx}
                    className="relative rounded-md overflow-hidden px-3.5 py-2 pe-10 flex items-center gap-2.5 min-h-[58px]"
                    style={{
                      backgroundImage: "url('/hud/img_MailBarSelected.png')",
                      backgroundSize: "100% 100%",
                      backgroundRepeat: "no-repeat",
                      textShadow: "none",
                    }}
                  >
                    {/* Structure: Left Target Emblem, Right 3-lines */}
                    <div className="flex items-center gap-2.5 w-full min-w-0">
                      {/* Left: NPC Target Emblem */}
                      <div className="shrink-0 w-10 h-10 flex items-center justify-center p-0.5 overflow-hidden">
                        <img
                          src={npcSprite.src}
                          alt={npcSprite.fallbackText}
                          className="w-full h-full object-contain filter drop-shadow-sm"
                          onError={(e) => {
                            // If sprite fails to load, show text fallback
                            const target = e.target as HTMLElement;
                            target.style.display = "none";
                            if (target.parentElement) {
                              target.parentElement.innerText =
                                npcSprite.fallbackText;
                            }
                          }}
                        />
                      </div>

                      {/* Right 3-line Stack */}
                      <div className="flex flex-col justify-between flex-1 min-w-0 overflow-hidden leading-tight">
                        {/* Line 1: [Type of report] */}
                        <div className="flex items-center justify-between gap-1 min-w-0">
                          <span className="font-bold truncate text-[0.92em] text-slate-900 tracking-tight">
                            {rec.display_target || rec.target}
                          </span>
                          <span
                            className={`text-[8.5px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border leading-none shrink-0 ${typeBadgeClass}`}
                          >
                            {typeBadgeLabel}
                          </span>
                        </div>

                        {/* Line 2: [Statistic] */}
                        <div className="flex items-center justify-between gap-1 min-w-0 text-[0.88em] font-mono">
                          {rec.items && rec.items.length > 0 ? (
                            <div className="flex items-center gap-1.5 truncate">
                              {rec.items.slice(0, 5).map((it, iIdx) => (
                                <span
                                  key={iIdx}
                                  className="inline-flex items-center gap-0.5 text-amber-900 font-bold shrink-0"
                                >
                                  <ResourceIcon
                                    resource={it}
                                    size="xs"
                                    className="w-3.5 h-3.5 object-contain"
                                  />
                                  <span>+{formatCompactCount(it.value)}</span>
                                </span>
                              ))}
                            </div>
                          ) : (
                            <span className="text-slate-700">
                              {rec.battle_type === "gathering"
                                ? "Complete"
                                : `Damage: ${rec.damage_pct}%`}
                            </span>
                          )}

                          <span className="font-bold shrink-0 text-[0.88em] text-slate-800">
                            {rec.battle_type === "gathering"
                              ? rec.talent_add > 0
                                ? `+${rec.talent_add.toLocaleString()}`
                                : "OK"
                              : `${rec.damage_pct}%${
                                  rec.plunder_tier > 0
                                    ? ` · T${rec.plunder_tier}`
                                    : ""
                                }`}
                          </span>
                        </div>

                        {/* Line 3: [Date] */}
                        <div className="flex items-center justify-between gap-1 min-w-0 text-[0.80em] font-mono text-slate-600">
                          <span className="min-w-0 truncate">
                            ({rec.pos_x}, {rec.pos_y})
                          </span>
                          <span className="shrink-0 whitespace-nowrap text-slate-500 font-medium">
                            {rec.time_vn ? rec.time_vn.slice(11, 16) : ""}
                          </span>
                        </div>
                      </div>
                    </div>
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
