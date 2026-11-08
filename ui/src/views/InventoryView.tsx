import React, { useState, useEffect, useCallback } from "react";
import {
  Search,
  Coins,
  Hammer,
  Zap,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n } from "../i18n";
import {
  ItemStatDto,
  ResourceSummaryDto,
  MaterialsSummaryDto,
  ApBottlesSummaryDto,
} from "../types";
import * as api from "../api/tauri";

export const InventoryView: React.FC = () => {
  const { language, mode, datePreset, dateFrom, dateTo, accountUid } =
    useAppStore();
  const { t } = useI18n(language);

  const [category, setCategory] = useState<string>("Tất cả");
  const [searchInput, setSearchInput] = useState<string>("");
  const [debouncedSearch, setDebouncedSearch] = useState<string>("");
  const [sortCol, setSortCol] = useState<string>("qty");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
  const [items, setItems] = useState<ItemStatDto[]>([]);
  const [resources, setResources] = useState<ResourceSummaryDto | null>(null);
  const [materials, setMaterials] = useState<MaterialsSummaryDto | null>(null);
  const [apBottles, setApBottles] = useState<ApBottlesSummaryDto | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(searchInput);
    }, 150);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  const categories = [
    { id: "Tất cả", label: t("catAll") },
    { id: "Tài Nguyên", label: t("catRss") },
    { id: "Vật Liệu", label: t("catMats") },
    { id: "Tăng Tốc", label: t("catSpeedups") },
    { id: "Pha Lê", label: t("catCrystals") },
    { id: "Sách & Tên", label: t("catBooks") },
    { id: "Khác", label: t("catOther") },
  ];

  const loadInventory = useCallback(async () => {
    setLoading(true);
    try {
      const data = await api.queryInventory(
        debouncedSearch,
        category,
        sortCol,
        sortDir
      );
      setItems(data.items);
      setResources(data.resources);
      setMaterials(data.materials);
      setApBottles(data.apBottles);
    } catch (err) {
      console.error("Failed to query inventory:", err);
    } finally {
      setLoading(false);
    }
  }, [debouncedSearch, category, sortCol, sortDir]);

  useEffect(() => {
    loadInventory();
  }, [loadInventory, mode, datePreset, dateFrom, dateTo, accountUid, language]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api
      .onWatcherUpdated(() => {
        loadInventory();
      })
      .then((u) => {
        unlisten = u;
      });
    return () => {
      if (unlisten) unlisten();
    };
  }, [loadInventory]);

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
      return (
        <ArrowUpDown className="w-3 h-3 text-binance-muted opacity-60 inline ml-1" />
      );
    return sortDir === "asc" ? (
      <ArrowUp className="w-3 h-3 text-binance-yellow inline ml-1" />
    ) : (
      <ArrowDown className="w-3 h-3 text-binance-yellow inline ml-1" />
    );
  };

  const hasRss =
    resources &&
    (resources.total_val > 0 ||
      resources.gem_val > 0 ||
      resources.choice_packs > 0);

  return (
    <div className="flex-1 overflow-y-auto p-4 sm:p-5 select-none space-y-4 bg-binance-canvas text-binance-body">
      {/* Category Pills & Search */}
      <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1.5 flex-wrap">
          {categories.map((c) => (
            <button
              key={c.id}
              onClick={() => setCategory(c.id)}
              className={`px-3 py-1 rounded-sm text-xs font-medium transition border ${
                category === c.id
                  ? "bg-binance-yellowSolid text-binance-ink border-binance-yellowSolid font-bold shadow-sm"
                  : "bg-binance-canvas text-binance-muted border-binance-hairline hover:text-binance-body hover:bg-binance-elevated"
              }`}
            >
              {c.label}
            </button>
          ))}
        </div>

        <div className="relative w-64">
          <Search className="w-3.5 h-3.5 text-binance-muted absolute left-3 top-2.5" />
          <input
            type="text"
            placeholder={t("invSearch")}
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            className="w-full bg-binance-canvas border border-binance-hairline rounded-sm pl-8 pr-3 py-1.5 text-xs text-binance-body placeholder-binance-muted outline-none font-mono"
          />
        </div>
      </div>

      {/* Resource Summary Banner */}
      {hasRss && resources && (
        <div className="p-4 rounded-md bg-binance-card border border-binance-hairline space-y-3">
          <div className="flex items-center justify-between text-xs border-b border-binance-hairline pb-2">
            <span className="font-semibold text-binance-yellow flex items-center gap-1.5 font-mono uppercase">
              <Coins className="w-4 h-4" />
              {t("rssSummary")}
            </span>
            <span className="font-mono text-binance-green font-bold">
              {(resources.total_val / 1_000_000).toFixed(2)}M RSS (
              {resources.total_val.toLocaleString()})
            </span>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2.5 text-xs">
            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssFood")}</div>
              <div className="text-base font-bold font-mono text-binance-yellow">
                {(resources.food_val / 1_000_000).toFixed(2)}M
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {resources.food_packs.toLocaleString()} {t("lblPacks")}
              </div>
            </div>

            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssWood")}</div>
              <div className="text-base font-bold font-mono text-binance-green">
                {(resources.wood_val / 1_000_000).toFixed(2)}M
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {resources.wood_packs.toLocaleString()} {t("lblPacks")}
              </div>
            </div>

            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssStone")}</div>
              <div className="text-base font-bold font-mono text-binance-info">
                {(resources.stone_val / 1_000_000).toFixed(2)}M
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {resources.stone_packs.toLocaleString()} {t("lblPacks")}
              </div>
            </div>

            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssGold")}</div>
              <div className="text-base font-bold font-mono text-binance-yellow">
                {(resources.gold_val / 1_000_000).toFixed(2)}M
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {resources.gold_packs.toLocaleString()} {t("lblPacks")}
              </div>
            </div>

            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssGems")}</div>
              <div className="text-base font-bold font-mono text-purple-500 dark:text-purple-400">
                {resources.gem_val.toLocaleString()}
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {resources.gem_packs.toLocaleString()} {t("lblDrops")}
              </div>
            </div>

            <div className="p-2.5 rounded-md bg-binance-canvas border border-binance-hairline">
              <div className="text-binance-muted">{t("rssChoicePacks")}</div>
              <div className="text-base font-bold font-mono text-binance-body">
                {resources.choice_packs.toLocaleString()}
              </div>
              <div className="text-[10px] text-binance-muted font-mono">
                {t("lblPacks")}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Materials & AP Bottles */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
        {materials && materials.total_converted > 0 && (
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2">
            <div className="text-xs font-semibold text-binance-body flex items-center gap-1.5 font-mono uppercase">
              <Hammer className="w-3.5 h-3.5 text-binance-green" />
              {t("matsSummary")}
            </div>
            <div className="text-[11px] font-mono text-binance-body space-y-1">
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("matLeather")}:</span>
                <span className="text-binance-green font-semibold">
                  {materials.leather_total.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("matIron")}:</span>
                <span className="text-binance-green font-semibold">
                  {materials.iron_total.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("matEbony")}:</span>
                <span className="text-binance-green font-semibold">
                  {materials.ebony_total.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("matBone")}:</span>
                <span className="text-binance-green font-semibold">
                  {materials.bone_total.toLocaleString()}
                </span>
              </div>
            </div>
            <div className="pt-1.5 border-t border-binance-hairline text-[11px] font-mono flex justify-between text-binance-muted">
              <span>{t("matTotalConverted")}</span>
              <span className="text-binance-green font-bold">
                {materials.total_converted.toLocaleString()}
              </span>
            </div>
          </div>
        )}

        {apBottles && apBottles.total_bottles > 0 && (
          <div className="p-3.5 rounded-md bg-binance-card border border-binance-hairline space-y-2">
            <div className="text-xs font-semibold text-binance-body flex items-center gap-1.5 font-mono uppercase">
              <Zap className="w-3.5 h-3.5 text-binance-yellow" />
              {t("apSummary")}
            </div>
            <div className="text-[11px] font-mono text-binance-body space-y-1">
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("ap50Label")}</span>
                <span className="text-binance-yellow font-semibold">
                  {apBottles.ap_50.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-binance-muted">{t("ap100Label")}</span>
                <span className="text-binance-yellow font-semibold">
                  {apBottles.ap_100.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-binance-muted">
                  {t("apTotalBottles")}
                </span>
                <span className="text-binance-body font-semibold">
                  {apBottles.total_bottles.toLocaleString()}
                </span>
              </div>
            </div>
            <div className="pt-1.5 border-t border-binance-hairline text-[11px] font-mono flex justify-between text-binance-muted">
              <span>{t("apTotalPoints")}</span>
              <span className="text-binance-yellow font-bold">
                {apBottles.total_points.toLocaleString()} AP
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Items Table */}
      <div className="rounded-md bg-binance-card border border-binance-hairline overflow-hidden">
        <div className="h-10 border-b border-binance-hairline bg-binance-canvas grid grid-cols-12 items-center px-4 text-[11px] font-mono font-semibold text-binance-muted">
          <div
            onClick={() => handleSort("name")}
            className="col-span-4 pl-1 cursor-pointer hover:text-binance-body flex items-center"
          >
            <span>{t("colItemName")}</span>
            {renderSortIcon("name")}
          </div>
          <div
            onClick={() => handleSort("qty")}
            className="col-span-2 text-right cursor-pointer hover:text-binance-body flex items-center justify-end"
          >
            <span>{t("colQty")}</span>
            {renderSortIcon("qty")}
          </div>
          <div className="col-span-2 text-center">{t("colConverted")}</div>
          <div
            onClick={() => handleSort("battles")}
            className="col-span-1 text-center cursor-pointer hover:text-binance-body flex items-center justify-center"
          >
            <span>{t("colBattles")}</span>
            {renderSortIcon("battles")}
          </div>
          <div
            onClick={() => handleSort("drop_rate")}
            className="col-span-1 text-center cursor-pointer hover:text-binance-body flex items-center justify-center"
          >
            <span>{t("colDropRate")}</span>
            {renderSortIcon("drop_rate")}
          </div>
          <div
            onClick={() => handleSort("avg")}
            className="col-span-1 text-right cursor-pointer hover:text-binance-body flex items-center justify-end"
          >
            <span>{t("colAvgPerBattle")}</span>
            {renderSortIcon("avg")}
          </div>
          <div
            onClick={() => handleSort("max")}
            className="col-span-1 text-right pr-1 cursor-pointer hover:text-binance-body flex items-center justify-end"
          >
            <span>{t("colMaxDrop")}</span>
            {renderSortIcon("max")}
          </div>
        </div>

        <div className="divide-y divide-binance-hairline/60 max-h-96 overflow-y-auto">
          {loading ? (
            <div className="p-8 text-center text-xs text-binance-muted font-mono">
              {t("loadingInventory")}
            </div>
          ) : items.length === 0 ? (
            <div className="p-8 text-center text-xs text-binance-muted">
              {t("emptyInventory")}
            </div>
          ) : (
            items.map((it, idx) => (
              <div
                key={`${it.itype}_${it.sub_type}_${idx}`}
                className="grid grid-cols-12 items-center px-4 py-2 text-xs hover:bg-binance-elevated/70 transition font-mono"
              >
                <div className="col-span-4 flex items-center gap-2 pl-1 min-w-0">
                  <span className="w-1.5 h-1.5 rounded-xs bg-binance-yellowSolid shrink-0" />
                  <span className="font-sans font-medium text-binance-body truncate">
                    {it.display_name ||
                      (language === "vi" ? it.name_vi : it.name_en)}
                  </span>
                </div>
                <div className="col-span-2 text-right font-bold text-binance-yellow">
                  {it.total_quantity.toLocaleString()}
                </div>
                <div className="col-span-2 text-center text-binance-green text-[11px] truncate">
                  {it.converted_value || "-"}
                </div>
                <div className="col-span-1 text-center text-binance-muted">
                  {it.battle_count.toLocaleString()}
                </div>
                <div className="col-span-1 text-center text-binance-muted">
                  {it.drop_rate.toFixed(1)}%
                </div>
                <div className="col-span-1 text-right text-binance-body">
                  {it.avg_per_battle.toFixed(1)}
                </div>
                <div className="col-span-1 text-right pr-1 text-binance-muted">
                  {it.max_in_battle.toLocaleString()}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
