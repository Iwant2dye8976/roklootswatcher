import React, { useState, useEffect, useRef } from "react";
import {
  Search,
  LayoutDashboard,
  Swords,
  Package,
  Settings,
  RefreshCw,
  Sun,
  Moon,
  FileSpreadsheet,
  FileCode,
  ClipboardCopy,
  X,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import * as api from "../api/tauri";

export const CommandPalette: React.FC = () => {
  const {
    commandPaletteOpen,
    setCommandPaletteOpen,
    setTab,
    updateFilter,
    toggleTheme,
    toggleLanguage,
    rescan,
    togglePause,
    addToast,
    language,
  } = useAppStore();

  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (commandPaletteOpen) {
      setQuery("");
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [commandPaletteOpen]);

  if (!commandPaletteOpen) return null;

  const actions = [
    {
      id: "nav-dash",
      title: language === "vi" ? "Mở Tổng Quan (Dashboard)" : "Go to Dashboard",
      category: "Navigation",
      icon: LayoutDashboard,
      run: () => setTab(0),
    },
    {
      id: "nav-battles",
      title: language === "vi" ? "Mở Chiến Báo (Battles)" : "Go to Battle Reports",
      category: "Navigation",
      icon: Swords,
      run: () => setTab(1),
    },
    {
      id: "nav-inventory",
      title: language === "vi" ? "Mở Kho Đồ (Inventory)" : "Go to Inventory & Drops",
      category: "Navigation",
      icon: Package,
      run: () => setTab(2),
    },
    {
      id: "nav-settings",
      title: language === "vi" ? "Mở Cài Đặt (Settings)" : "Go to Settings",
      category: "Navigation",
      icon: Settings,
      run: () => setTab(3),
    },
    {
      id: "act-rescan",
      title: language === "vi" ? "Quét lại Mailcache" : "Rescan Mailcache",
      category: "Actions",
      icon: RefreshCw,
      run: () => rescan(),
    },
    {
      id: "act-pause",
      title:
        language === "vi"
          ? "Tạm dừng / Tiếp tục theo dõi"
          : "Toggle Watcher Pause/Resume",
      category: "Actions",
      icon: RefreshCw,
      run: () => togglePause(),
    },
    {
      id: "filter-motte",
      title: "Filter: Motte",
      category: "Filter",
      icon: Swords,
      run: () => updateFilter({ mode: "motte" }),
    },
    {
      id: "filter-fort",
      title: "Filter: Barbarian Forts",
      category: "Filter",
      icon: Swords,
      run: () => updateFilter({ mode: "fort" }),
    },
    {
      id: "filter-barb",
      title: "Filter: Barbarians",
      category: "Filter",
      icon: Swords,
      run: () => updateFilter({ mode: "barbarian" }),
    },
    {
      id: "filter-gather",
      title: "Filter: Gathering",
      category: "Filter",
      icon: Package,
      run: () => updateFilter({ mode: "gathering" }),
    },
    {
      id: "filter-today",
      title: "Date Preset: Today",
      category: "Filter",
      icon: RefreshCw,
      run: () => updateFilter({ datePreset: "Today" }),
    },
    {
      id: "filter-yesterday",
      title: "Date Preset: Yesterday",
      category: "Filter",
      icon: RefreshCw,
      run: () => updateFilter({ datePreset: "Yesterday" }),
    },
    {
      id: "filter-7d",
      title: "Date Preset: 7 Days",
      category: "Filter",
      icon: RefreshCw,
      run: () => updateFilter({ datePreset: "Last7Days" }),
    },
    {
      id: "copy-summary",
      title:
        language === "vi"
          ? "Sao chép tóm tắt chiến lợi phẩm vào Clipboard"
          : "Copy Loot Summary to Clipboard",
      category: "Export",
      icon: ClipboardCopy,
      run: async () => {
        try {
          const text = await api.getClipboardSummary();
          await navigator.clipboard.writeText(text);
          addToast({
            type: "success",
            text:
              language === "vi"
                ? "Đã chép tóm tắt vào Clipboard!"
                : "Copied loot summary to clipboard!",
          });
        } catch (e) {
          addToast({ type: "error", text: String(e) });
        }
      },
    },
    {
      id: "export-csv",
      title: "Export Active Battles to CSV",
      category: "Export",
      icon: FileSpreadsheet,
      run: async () => {
        const res = await api.exportActiveData("csv");
        if (res) {
          addToast({ type: "success", text: `CSV: ${res}` });
        }
      },
    },
    {
      id: "export-json",
      title: "Export Active Battles to JSON",
      category: "Export",
      icon: FileCode,
      run: async () => {
        const res = await api.exportActiveData("json");
        if (res) {
          addToast({ type: "success", text: `JSON: ${res}` });
        }
      },
    },
    {
      id: "toggle-theme",
      title: "Toggle Dark / Light Theme",
      category: "Preferences",
      icon: Sun,
      run: () => toggleTheme(),
    },
    {
      id: "toggle-lang",
      title: `Switch Language (${
        language === "vi" ? "Tiếng Việt → English" : "English → Tiếng Việt"
      })`,
      category: "Preferences",
      icon: Moon,
      run: () => toggleLanguage(),
    },
  ];

  const filtered = actions.filter(
    (a) =>
      a.title.toLowerCase().includes(query.toLowerCase()) ||
      a.category.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div
      onClick={() => setCommandPaletteOpen(false)}
      className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/60 backdrop-blur-xs"
    >
      <div
        className="w-full max-w-lg bg-binance-card border border-binance-hairline rounded-md shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input Bar */}
        <div className="flex items-center gap-2.5 px-4 py-3 border-b border-binance-hairline">
          <Search className="w-4 h-4 text-binance-yellow shrink-0" />
          <input
            ref={inputRef}
            type="text"
            placeholder={
              language === "vi"
                ? "Nhập tên lệnh để tìm nhanh..."
                : "Type a command or search..."
            }
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") setCommandPaletteOpen(false);
              if (e.key === "Enter" && filtered.length > 0) {
                filtered[0].run();
                setCommandPaletteOpen(false);
              }
            }}
            className="w-full bg-transparent text-sm text-binance-body placeholder-binance-muted outline-none"
          />
          <button
            onClick={() => setCommandPaletteOpen(false)}
            className="p-1 rounded text-binance-muted hover:text-binance-body"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Action List */}
        <div className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-xs text-binance-muted">
              No matching commands
            </div>
          ) : (
            filtered.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  key={item.id}
                  onClick={() => {
                    item.run();
                    setCommandPaletteOpen(false);
                  }}
                  className="w-full flex items-center gap-3 px-3 py-2 rounded-xs text-left hover:bg-binance-elevated text-binance-body transition group text-xs"
                >
                  <Icon className="w-4 h-4 text-binance-muted group-hover:text-binance-yellow transition-colors" />
                  <span className="font-medium">{item.title}</span>
                  <span className="ml-auto text-[10px] font-mono text-binance-muted uppercase px-1.5 py-0.5 rounded bg-binance-canvas border border-binance-hairline">
                    {item.category}
                  </span>
                </button>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
