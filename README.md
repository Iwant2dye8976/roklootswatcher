# RoK Loot Watcher

[English](README.md) | [Tiếng Việt](README.vi.md)

Inspired by: [rokbattles](https://github.com/rokbattles/rokbattles)

A high-performance Windows desktop application and CLI tool that monitors the Rise of Kingdoms `mailcache` directory in real-time. It decodes proprietary Lilith binary mail structures, stores combat loot history in a persistent SQLite WAL vault, synchronizes records to Supabase Cloud, and renders combat telemetry inside a Tauri v2 desktop shell powered by React 19, TypeScript, and Tailwind CSS.

---

## Key Features

### 1. Real-Time Lilith Mail Ingestion
- Real-time directory monitoring using Windows full-share file flags (`FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`).
- Decodes Lilith binary serialization tags (`0x00` Null, `0x01` Bool, `0x02` U8, `0x03` F64, `0x04` String, `0x05` Table, `0xFF` End) with zero file locks or game interference.
- Multi-threaded batch decoding powered by Rayon (`MAIL_DECODE_BATCH_SIZE`).
- Automatic account separation and multi-UID isolation.

### 2. SQLite History Vault & Deduplication
- Persistent local SQLite database (`rok_history_vault.db`) in WAL mode with connection busy timeouts and atomic transactions.
- Constant-time O(1) deduplication on report identifiers.
- Automatic raw binary mail backup to `mail_vault/`.
- Safe temporary-file writes and atomic renames for CSV and JSON data exports.
- Automatic schema migration from legacy JSON vaults.

### 3. Action Point (AP) Economics & Analytics
- **Dual-Axis SVG Combo Timeline**: Tracks daily AP consumption against speedup gains using soft green volume bars (input cost) and soft blue yield curves (return output), complete with interactive tooltips calculating minutes per 1,000 AP spent.
- **Target Efficiency Benchmarks**: Evaluates and ranks Barbarian Forts, Motte and Bailey, and Chained Barbarians by speedup yield, gem yield, and Book of Covenant drop rates.
- **AP Rebate Tracking**: Real-time calculation of AP potion return percentages from barbarian reward chests.
- Time-zone normalization with UTC+7 (Vietnam Time) rolling day windows.

### 4. In-Game HUD Overlay
- Lightweight, transparent WebView2 overlay tracking the active Rise of Kingdoms game window.
- DPI-aware coordinate calculation, custom offsets, and Win32 click-through (`WS_EX_TRANSPARENT`).
- Global toggle hotkey (`Ctrl+Shift+H`).
- Multiple display layouts: Full HUD, Mini HUD, Combined HUD, or Hidden.

### 5. Supabase Cloud Sync & Authentication
- Background actor-based synchronization worker (`CloudSyncWorker`) with channel communication.
- Per-record batch upsert with duplicate resolution (`resolution=merge-duplicates`).
- Smart exponential backoff retry for network transients and immediate error classification for auth failures.
- Secure local loopback Google OAuth server with CSRF state token verification.
- Offline change queue and full cross-device vault restore.

### 6. Modern Desktop Shell & Localization
- Tauri v2 architecture with React 19, TypeScript, and Binance-inspired dark theme.
- Virtualized lists for responsive browsing across tens of thousands of battle records.
- Zero-file embedded item database (`loot.json`) for instant startup.
- Full bilingual support (English and Vietnamese).
- Windows system tray minimization and Windows registry autostart support.

---

## Getting Started

### GUI Mode
Launch `rok_watcher.exe`. The application automatically scans standard drive letters to locate your Rise of Kingdoms `mailcache` path. You can also specify a custom path in the Settings tab.

### CLI Mode
Perform quick audits, calculations, and exports straight from PowerShell or Command Prompt:

```powershell
# Analyze all targets and output a JSON summary
.\rok_watcher.exe --cli --mode all-targets --json

# Analyze Motte and Bailey rallies over the last 7 days
.\rok_watcher.exe --cli --mode motte --days 7

# Analyze resource gathering over the last 30 days
.\rok_watcher.exe --cli --mode gathering --days 30

# View all available CLI flags
.\rok_watcher.exe --help
```

---

## Cloud Sync Setup (Supabase)

To enable cloud backup and multi-device synchronization:

1. Create a project on [Supabase](https://supabase.com).
2. Open the **SQL Editor** in the Supabase Dashboard and run the migration script:
   [`resources/supabase_migration_report_records.sql`](resources/supabase_migration_report_records.sql).
3. Enable the **Google Provider** under **Authentication -> Providers**.
4. In RoK Loot Watcher, navigate to **Settings -> Cloud Sync**, input your Supabase URL and Anon Key, and click **Login with Google**.

---

## Development & Build Workflow

### Prerequisites
- Rust 1.77+ (2021 edition)
- Node.js 18+ and npm
- Visual Studio C++ Build Tools (Windows)

### Commands

```powershell
# 1. Install UI dependencies
npm --prefix ui install

# 2. Build React UI
npm --prefix ui run build

# 3. Run unit and integration tests (339+ tests)
cargo test

# 4. Check linter rules
cargo clippy -- -D warnings

# 5. Build optimized release binary
cargo build --release

# 6. Copy release binary to project root
Copy-Item -Force .\target\release\rok_watcher.exe .\rok_watcher.exe
```

---

## Architecture Overview

```
roklootswatcher/
├── src/
│   ├── main.rs                 # CLI entry point, Windows mutex, subsystem coordinator
│   ├── model.rs                # Core domain types (ReportRecord, BattleReport, LootItem)
│   ├── lilith_stream.rs        # Binary stream parser for Lilith mail cache format
│   ├── classifier.rs           # Mail categorization & payload extraction
│   ├── loot_catalog.rs         # Embedded item database & resource pack resolver
│   ├── vault.rs                # SQLite WAL storage engine & deduplication
│   ├── vault_migrate.rs        # Database schema migrations
│   ├── watcher.rs              # Non-blocking filesystem observer
│   ├── analyzer.rs             # AP economics math & target drop rate statistics
│   ├── cloud_sync.rs           # Background actor-based Supabase sync worker
│   ├── overlay_tracker.rs      # Win32 window geometry tracker & hotkey listener
│   ├── autostart.rs            # Windows registry autostart manager
│   ├── exporters.rs            # CSV and JSON export routines
│   ├── i18n.rs                 # Bilingual translation engine (EN/VI)
│   └── tauri_app/              # Tauri v2 backend commands, DTOs & overlay window manager
├── ui/                         # React 19 + TypeScript + Tailwind CSS frontend
│   └── src/
│       ├── store/              # Central state store (Tauri event listeners)
│       ├── views/              # DashboardView, BattlesView, InventoryView, SettingsView, OverlayView
│       └── components/         # Titlebar, Sidebar, GlobalFilterBar, InspectorDrawer
└── resources/                  # SQL migration schemas & static resources
```

---

## License

Personal and community use for Rise of Kingdoms players. Distributed under the terms of the project repository.
