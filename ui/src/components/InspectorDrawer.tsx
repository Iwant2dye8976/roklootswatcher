import React, { useEffect } from "react";
import { X, MapPin, Crosshair, Award, Clock, FileText, User } from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n, formatSpeedupMinutes } from "../i18n";
import { CommanderCardPair } from "./CommanderCardPair";
import { ResourceIcon } from "./ResourceIcon";

export const InspectorDrawer: React.FC = () => {
  const { selectedBattle, setSelectedBattle, language, speedupUnit } = useAppStore();
  const { t } = useI18n(language);

  useEffect(() => {
    if (!selectedBattle) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setSelectedBattle(null);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [selectedBattle, setSelectedBattle]);

  if (!selectedBattle) return null;

  const b = selectedBattle;
  const isGathering = b.battle_type === "gathering";
  const isPvp = b.battle_type === "pvp";
  const totalSpdMins =
    (b.speedup_univ_mins || 0) +
    (b.speedup_train_mins || 0) +
    (b.speedup_heal_mins || 0);

  const badgeColor =
    b.battle_type === "motte"
      ? "bg-binance-red/20 text-binance-red border-binance-red/40"
      : b.battle_type === "fort"
      ? "bg-binance-yellow/20 text-binance-yellow border-binance-yellow/40"
      : isGathering
      ? "bg-binance-green/20 text-binance-green border-binance-green/40"
      : isPvp
      ? "bg-fuchsia-500/20 text-fuchsia-400 border-fuchsia-500/40"
      : "bg-binance-info/20 text-binance-info border-binance-info/40";

  return (
    <aside className="w-80 bg-binance-canvas border-l border-binance-hairline flex flex-col justify-between p-4 z-40 select-none shrink-0 shadow-2xl overflow-y-auto">
      {/* Top Header */}
      <div className="space-y-4">
        <div className="flex items-center justify-between border-b border-binance-hairline pb-3">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-xs bg-binance-yellowSolid" />
            <h3 className="font-mono text-xs font-semibold uppercase tracking-wider text-binance-body">
              {t("inspectorTitle")}
            </h3>
          </div>
          <button
            onClick={() => setSelectedBattle(null)}
            title={t("inspectorClose")}
            className="p-1 rounded-sm text-binance-muted hover:text-binance-body hover:bg-binance-elevated transition"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Target Name & Badges */}
        <div className="p-3 rounded-md bg-binance-card border border-binance-hairline space-y-2">
          <div className="text-sm font-semibold text-binance-body">
            {b.display_target || b.target}
          </div>

          <div className="flex items-center gap-2 flex-wrap">
            <span
              className={`px-2 py-0.5 rounded text-[10px] font-mono font-bold uppercase border ${badgeColor}`}
            >
              {b.battle_type}
            </span>

            {b.plunder_tier > 0 && (
              <span className="flex items-center gap-1 px-2 py-0.5 rounded bg-binance-yellow/15 text-binance-yellow border border-binance-yellow/30 text-[10px] font-mono font-semibold">
                <Award className="w-3 h-3" />
                Tier {b.plunder_tier}
              </span>
            )}
          </div>

          <div className="pt-2 text-[11px] text-binance-muted space-y-1">
            <div className="flex items-center gap-1.5">
              <Clock className="w-3.5 h-3.5 text-binance-muted" />
              <span>{b.time_vn || b.time_utc}</span>
            </div>

            <div className="flex items-center gap-1.5">
              <MapPin className="w-3.5 h-3.5 text-binance-muted" />
              <span>
                {t("inspectorCoord")}: ({b.pos_x}, {b.pos_y})
              </span>
            </div>

            {isPvp ? (
              <div className="space-y-1.5 pt-1 border-t border-binance-hairline text-binance-body font-mono">
                <div className="flex items-center gap-2.5 text-fuchsia-400 font-semibold">
                  <CommanderCardPair
                    primaryId={b.primary_commander_id}
                    primaryName={b.primary_commander_name}
                    primaryIsExpertised={b.primary_is_expertised}
                    secondaryId={b.secondary_commander_id}
                    secondaryName={b.secondary_commander_name}
                    secondaryIsExpertised={b.secondary_is_expertised}
                    size="md"
                  />
                  <span>
                    {t("colCommanders")}:{" "}
                    {b.primary_commander_name
                      ? b.secondary_commander_name
                        ? `${b.primary_commander_name} + ${b.secondary_commander_name}`
                        : b.primary_commander_name
                      : "Unknown"}
                  </span>
                </div>
                {(b.target_primary_commander_id || b.target_primary_commander_name) && (
                  <div className="flex items-center gap-2.5 text-binance-muted">
                    <CommanderCardPair
                      primaryId={b.target_primary_commander_id}
                      primaryName={b.target_primary_commander_name}
                      primaryIsExpertised={b.target_primary_is_expertised}
                      secondaryId={b.target_secondary_commander_id}
                      secondaryName={b.target_secondary_commander_name}
                      secondaryIsExpertised={b.target_secondary_is_expertised}
                      size="md"
                    />
                    <span>
                      {t("colTarget")}:{" "}
                      {b.target_primary_commander_name
                        ? b.target_secondary_commander_name
                          ? `${b.target_primary_commander_name} + ${b.target_secondary_commander_name}`
                          : b.target_primary_commander_name
                        : "Unknown"}
                    </span>
                  </div>
                )}
                <div className="text-binance-yellow font-bold">
                  {t("colKpGained")}: +
                  {(
                    b.kills_dealt ??
                    (b.dead_dealt || 0) + (b.severely_wounded_dealt || 0)
                  ).toLocaleString()}{" "}
                  ({t("colTradeRatio")}:{" "}
                  {b.trade_ratio_pct != null
                    ? `${b.trade_ratio_pct.toFixed(0)}%`
                    : "100%"}
                  )
                </div>
                <div className="text-binance-red">
                  {t("colKpLost")}: -
                  {(
                    b.kills_taken ??
                    (b.dead_taken || 0) + (b.severely_wounded_taken || 0)
                  ).toLocaleString()}
                </div>
                <div className="text-binance-muted text-[10px]">
                  Sev. Wounded: +{(b.severely_wounded_dealt || 0).toLocaleString()} / -
                  {(b.severely_wounded_taken || 0).toLocaleString()} · Dead: +
                  {(b.dead_dealt || 0).toLocaleString()} / -
                  {(b.dead_taken || 0).toLocaleString()}
                </div>
                {b.kp_gained != null && b.kp_gained > 0 && (
                  <div className="text-slate-400 text-[10px]">
                    KP: +{Math.round(b.kp_gained).toLocaleString()} / -
                    {Math.round(b.kp_lost || 0).toLocaleString()}
                  </div>
                )}
              </div>
            ) : isGathering ? (
              b.talent_add > 0 && (
                <div className="flex items-center gap-1.5 text-binance-green">
                  <Award className="w-3.5 h-3.5" />
                  <span>
                    {t("colBonus")}: +{b.talent_add.toLocaleString()}
                  </span>
                </div>
              )
            ) : (
              <div className="flex items-center gap-1.5 text-binance-yellow">
                <Crosshair className="w-3.5 h-3.5" />
                <span>
                  {t("inspectorDamage")}: {b.damage_pct}%
                </span>
              </div>
            )}
          </div>

          {/* Quick Summary Chips */}
          {(b.crystals || totalSpdMins > 0 || b.total_rss || b.gems) && (
            <div className="pt-2 border-t border-binance-hairline flex flex-wrap gap-1.5 text-[10px] font-mono">
              {b.crystals ? (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-binance-yellow font-semibold">
                  <ResourceIcon resource="crystals" size="lg" className="object-contain shrink-0" />
                  {b.crystals.toLocaleString()} {t("colCrystals")}
                </span>
              ) : null}
              {totalSpdMins > 0 ? (
                <span className="px-1.5 py-0.5 rounded bg-binance-elevated text-binance-green font-semibold">
                  {formatSpeedupMinutes(totalSpdMins, speedupUnit, language)}
                </span>
              ) : null}
              {b.total_rss ? (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-binance-body">
                  <ResourceIcon resource="food" size="lg" className="object-contain shrink-0" />
                  {(b.total_rss / 1_000_000).toFixed(2)}M RSS
                </span>
              ) : null}
              {b.gems ? (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-binance-info font-semibold">
                  <ResourceIcon resource="gems" size="lg" className="object-contain shrink-0" />
                  {b.gems.toLocaleString()} Gems
                </span>
              ) : null}
            </div>
          )}
        </div>

        {/* Loot Drops Section */}
        <div className="space-y-2">
          <div className="text-xs font-semibold text-binance-body flex items-center justify-between">
            <span>{t("inspectorDrops")}</span>
            <span className="font-mono text-[10px] text-binance-muted">
              {b.items.length}
            </span>
          </div>

          <div className="space-y-1.5 max-h-72 overflow-y-auto pr-1">
            {b.items.map((item, idx) => (
              <div
                key={idx}
                className="flex items-center justify-between p-2 rounded-xs bg-binance-card border border-binance-hairline text-xs gap-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <ResourceIcon
                    resource={item}
                    size="lg"
                    className="shrink-0 object-contain"
                  />
                  <div className="flex flex-col min-w-0">
                    <span className="font-medium text-binance-body truncate">
                      {item.display_name ||
                        (language === "vi" ? item.name_vi : item.name_en)}
                    </span>
                    <span className="text-[10px] font-mono text-binance-muted truncate">
                      {item.converted_value
                        ? item.converted_value
                        : `${item.category || "Item"} (${item.itype}/${item.sub_type})`}
                    </span>
                  </div>
                </div>
                <div className="text-right shrink-0">
                  <span className="font-mono font-bold text-binance-yellow">
                    +{item.value.toLocaleString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Bottom Metadata */}
      <div className="pt-4 border-t border-binance-hairline text-[10px] font-mono text-binance-muted space-y-1">
        <div className="flex items-center gap-1.5 truncate">
          <FileText className="w-3 h-3 text-binance-muted shrink-0" />
          <span className="truncate">{b.file_name}</span>
        </div>
        <div className="flex items-center gap-1.5 truncate">
          <User className="w-3 h-3 text-binance-muted shrink-0" />
          <span className="truncate">
            {b.account_name
              ? `${b.account_name} (${b.account_uid})`
              : b.account_uid || "Unknown"}
          </span>
        </div>
      </div>
    </aside>
  );
};
