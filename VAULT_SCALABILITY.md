# RoK Loot Watcher - Vault Scalability & 1-Year Growth Analysis

## Status: SQLite Migration Completed

As of November 2026, `rok_history_vault.json` has been replaced with `rok_history_vault.db` (SQLite with WAL mode).
This resolves the disk I/O and RAM scalability issues described below.
Cloud sync now exports only the vault JSON when needed, rather than reading the file directly.

---

This document outlines the projected performance, disk I/O, and cloud synchronization impacts as `rok_history_vault.json` grows over a 1-year operational window.

---

## 1. Projected Scale (1-Year Window)

* **Current Baseline (as of Oct 2026)**:
  * 1,501 reports $\approx$ **2.75 MB** in [`rok_history_vault.json`](file:///C:/roklootswatcher/rok_history_vault.json)
  * Average payload: **~1.83 KB** per JSON report entry.
* **1-Year Active Player Growth** (assuming 50–150 barbarian, fort, and gathering reports/day):
  * Cumulative reports: **20,000 to 50,000 records**
  * Estimated vault JSON size: **35 MB to 90 MB**

---

## 2. Issues That Will Arise at Scale

### A. Cloud Backup Failure (Critical Bottleneck)
1. **Supabase Kong API Gateway Limit (`HTTP 413 Payload Too Large`)**:
   * Hosted Supabase instances enforce a default API gateway body size limit of **10 MB** (`client_max_body_size`).
   * Because [`backup_vault`](file:///C:/roklootswatcher/src/cloud_sync.rs#L390) uploads the entire vault in a single `vault_data` JSON column, uploads will hard-fail once the vault reaches **~5,500 records** (approximately 2–3 months of active play).
2. **On-Exit Backup Timeout**:
   * [`perform_exit_cloud_backup`](file:///C:/roklootswatcher/src/gui.rs#L693-L738) enforces a strict **4-second network timeout** so the app terminates promptly without freezing the user's OS.
   * Uploading a 35–90 MB uncompressed payload over typical consumer broadband connections (10–50 Mbps upload) will consistently time out.

### B. Disk I/O & Serialization Overhead (Moderate Bottleneck)
1. **Full-Vault Flushes**:
   * [`MailVault::flush_to_file`](file:///C:/roklootswatcher/src/vault.rs#L347) uses an atomic rewrite pattern (writing to `.tmp` then replacing the target file).
   * At 50,000 records, every flush serializes and writes 90 MB to the SSD.
2. **Process Working Set (RAM)**:
   * Retaining 50,000 deserialized structs alongside transient JSON serialization strings will elevate application RAM from ~45 MB to **250–450 MB**.

---

## 3. Systems That Will Remain Fast & Safe

1. **GUI Rendering Performance**:
   * The reports and analytics tables in [`src/gui.rs`](file:///C:/roklootswatcher/src/gui.rs) utilize `egui` table virtualization. Only ~20–30 visible rows are calculated and drawn per frame, keeping UI rendering at a steady 60 FPS.
2. **In-Memory Filtering & Statistics**:
   * In compiled release Rust, single-pass filtering and statistics aggregation across 50,000 contiguous array records completes in **5–15 milliseconds**.
3. **Local Game Cache (`mailcache`)**:
   * The Rise of Kingdoms game engine automatically purges old mail files locally from `save/mailcache` after reaching internal mailbox quotas, preventing the game's directory from growing infinitely.

---

## 4. Remediation Roadmap (When Thresholds Are Approached)

1. **Gzip / Brotli Compression for Cloud Payloads (Lowest Effort, Highest Impact)**:
   * Vault data is repetitive textual JSON, yielding an **85–90% compression ratio**.
   * A 90 MB vault compresses down to **~8–10 MB**, staying within Supabase payload limits and uploading in 1–2 seconds.
2. **KvK / Seasonal Archive**:
   * Implement an archive boundary (e.g., "Archive records older than 90 days / previous KvK" into `rok_archive.json`).
   * Keeps active [`rok_history_vault.json`](file:///C:/roklootswatcher/rok_history_vault.json) below 3,000 records for fast flushes and minimal cloud egress.
3. **Incremental Cloud Sync or Supabase Storage**:
   * If keeping all history in one file is preferred, store compressed archives directly in a private Supabase Storage bucket rather than a single PostgREST database table row.
