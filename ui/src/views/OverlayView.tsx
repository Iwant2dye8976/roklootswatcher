import React, { useState, useEffect } from "react";
import { OverlayPayloadDto } from "../types";
import * as api from "../api/tauri";

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

  const { config, records, pinned_chips, today_runs } = data;
  const showChips = config.mode === "MiniHud" || config.mode === "Combined";
  const showCards = config.mode === "Cards" || config.mode === "Combined";
  const isMiniOnly = config.mode === "MiniHud";

  const totalIdle = (data.idle_secs || 0) + extraIdleSecs;
  const idleFactor = totalIdle >= 60 ? 0.88 : 1.0;
  const userOpacity = Math.max(0.3, Math.min(1.0, config.opacity || 0.95));
  const effectiveOpacity = Math.max(0.35, Math.min(1.0, userOpacity * idleFactor));
  const baseFontPx = Math.max(8.5, Math.min(16, config.font_size || 10.5));
  const textHex = config.text_color_hex || "#F8FAFC";
  const accentHex = config.accent_color_hex || "#38BDF8";

  return (
    <div
      style={{
        opacity: effectiveOpacity,
        pointerEvents: "none",
        userSelect: "none",
        fontSize: `${baseFontPx}px`,
        color: textHex,
        textShadow: "0 1px 2px rgba(0,0,0,0.95), 0 0 1px rgba(0,0,0,0.9)",
      }}
      className={`w-full h-full select-none font-sans antialiased overflow-hidden transition-opacity duration-300 ${
        isMiniOnly ? "px-1.5 py-0.5" : "p-1.5"
      }`}
    >
      <div className="flex flex-col gap-1 w-full">
        {/* Pinned Chips Ribbon (MiniHud & Combined only) */}
        {showChips && (
          <div
            className={`flex items-center gap-1 ${
              isMiniOnly ? "flex-nowrap overflow-hidden" : "flex-wrap"
            }`}
          >
            {pinned_chips.map((chip) => (
              <div
                key={chip.id}
                className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-[#060a14]/98 border border-slate-700/90 shadow-md font-mono shrink-0 leading-none"
              >
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

            {/* Today Runs Counter Pill */}
            <div className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-[#060a14]/98 border border-slate-700/90 shadow-md font-mono text-[0.88em] text-slate-200 shrink-0 leading-none font-medium">
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
            </div>
          </div>
        )}

        {/* Recent Battle Drop Cards (Cards & Combined only) */}
        {showCards && (
          <div className="space-y-1">
            {records.length === 0 ? (
              <div className="px-2 py-1.5 rounded bg-[#060a14]/95 border border-slate-700/80 text-[0.88em] text-slate-300 font-mono italic text-center">
                Awaiting combat reports...
              </div>
            ) : (
              records.slice(0, 5).map((rec, idx) => {
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
                    className={`px-2 py-1.5 rounded bg-[#060a14]/98 border border-slate-700/90 border-l-[3px] ${borderClass} shadow-xl flex flex-col gap-0.5 leading-tight`}
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
