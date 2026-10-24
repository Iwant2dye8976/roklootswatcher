#![windows_subsystem = "windows"]

mod analyzer;
mod armor;
mod autostart;
mod classifier;
mod exporters;
mod gui;
mod i18n;
mod lilith_stream;
mod loot_catalog;
mod model;
pub mod overlay_tracker;
pub mod overlay_ui;
mod single_instance;
mod tray;
mod vault;
mod watcher;

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;
use chrono::NaiveDate;
use eframe::egui;

use crate::analyzer::{filter_records_by_date, generate_statistics, parse_date_str};
use crate::classifier::{classify_mail, extract_motte_details};
use crate::exporters::{export_individual_mails_json, export_json};
use crate::gui::RoKLootWatcherApp;
use crate::lilith_stream::decode_mail_file;
use crate::loot_catalog::LootCatalog;
use crate::model::{format_num, get_item_converted_display, AppLanguage, FilterMode, SpeedupUnit};
use crate::tray::SystemTrayManager;
use crate::watcher::RoKMailWatcher;

#[cfg(windows)]
fn attach_parent_console() {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
            fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
            fn SetConsoleCP(wCodePageID: u32) -> i32;
        }
        // ATTACH_PARENT_PROCESS = (DWORD)-1 = 0xFFFFFFFF
        if AttachConsole(0xFFFFFFFF) != 0 {
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Check if help was requested
    if args.iter().any(|a| a == "--help" || a == "-h") {
        #[cfg(windows)]
        attach_parent_console();
        print_help();
        return Ok(());
    }

    // Check if CLI mode was requested
    if args.iter().any(|a| a == "--cli" || a == "-c") {
        #[cfg(windows)]
        attach_parent_console();
        return run_cli_mode(&args);
    }

    run_gui_mode(&args)
}

fn print_help() {
    println!("Rise of Kingdoms - Motte & Barbarian Reward Analyzer (Rust Edition)");
    println!("Usage: rok_watcher [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --cli                    Chạy ở chế độ dòng lệnh (Headless)");
    println!("  --mail-dir <PATH>        Đường dẫn thư mục mailcache");
    println!("  --loot-yaml <PATH>       Đường dẫn file loot.yaml");
    println!("  --mode <MODE>            motte, fort, barbarian, all-targets, all-forts, all-rewards, gathering");
    println!("  --start-date <YYYY-MM-DD> Lọc từ ngày");
    println!("  --end-date <YYYY-MM-DD>   Lọc đến ngày");
    println!("  --days <N>               Lọc N ngày gần nhất (vd: 7, 30)");
    println!("  --speedup-unit, -u <UNIT> Đơn vị tăng tốc: minutes (mặc định), hours, days");
    println!("  --out-dir <PATH>         Thư mục xuất file báo cáo");
    println!("  --json                   Xuất dữ liệu đầy đủ ra file JSON");
    println!("  --multi-instance         Cho phép mở nhiều phiên bản giao diện đồng thời");
    println!("  --minimized, --tray      Khởi động thu nhỏ trực tiếp vào Khay hệ thống");
    println!("  --help, -h               Hiện trợ giúp này");
}

fn run_cli_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut mail_dir = crate::watcher::detect_default_mail_dir();
    let mut loot_yaml: Option<PathBuf> = None;
    let mut mode = FilterMode::Motte;
    let mut start_date: Option<NaiveDate> = None;
    let mut end_date: Option<NaiveDate> = None;
    let mut days: Option<u32> = None;
    let mut speedup_unit = SpeedupUnit::Minutes;
    let mut out_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut export_json_flag = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cli" | "-c" => {
                // Already checked
            }
            "--mail-dir" => {
                if i + 1 < args.len() {
                    mail_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --mail-dir yêu cầu đường dẫn thư mục.");
                    return Ok(());
                }
            }
            "--loot-yaml" => {
                if i + 1 < args.len() {
                    loot_yaml = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --loot-yaml yêu cầu đường dẫn file YAML.");
                    return Ok(());
                }
            }
            "--mode" => {
                if i + 1 < args.len() {
                    mode = FilterMode::from_str_loose(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --mode yêu cầu tên chế độ lọc.");
                    return Ok(());
                }
            }
            "--start-date" => {
                if i + 1 < args.len() {
                    start_date = parse_date_str(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --start-date yêu cầu giá trị ngày.");
                    return Ok(());
                }
            }
            "--end-date" => {
                if i + 1 < args.len() {
                    end_date = parse_date_str(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --end-date yêu cầu giá trị ngày.");
                    return Ok(());
                }
            }
            "--days" => {
                if i + 1 < args.len() {
                    days = args[i + 1].parse().ok();
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --days yêu cầu số ngày.");
                    return Ok(());
                }
            }
            "--speedup-unit" | "-u" => {
                if i + 1 < args.len() {
                    speedup_unit = SpeedupUnit::from_str_loose(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --speedup-unit yêu cầu đơn vị (minutes/hours/days/dhm).");
                    return Ok(());
                }
            }
            "--out-dir" => {
                if i + 1 < args.len() {
                    out_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                } else {
                    eprintln!("[!] Lỗi: Tham số --out-dir yêu cầu đường dẫn thư mục xuất.");
                    return Ok(());
                }
            }
            "--json" | "--export-json" | "-j" => {
                export_json_flag = true;
            }
            unknown => {
                eprintln!("[!] Cảnh báo: Bỏ qua tham số không xác định: {}", unknown);
            }
        }
        i += 1;
    }

    println!("[*] Đang khởi động ROK Analyzer Engine (Rust) với Persistent Vault...");
    let catalog = LootCatalog::load_from_candidates(loot_yaml.as_deref());

    // Khởi tạo MailVault để lưu trữ bền vững
    let mut vault = crate::vault::MailVault::new(&out_dir);
    let vault_dir = vault.get_binary_dir().to_path_buf();
    let _ = std::fs::create_dir_all(&vault_dir);

    println!("[*] Quét thư mục mailcache: {}", mail_dir.display());
    let mut all_files_map = std::collections::HashMap::new();

    if mail_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&mail_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if fname.starts_with("Persistent.Mail.") {
                        // Tự động sao lưu nhị phân vào vault
                        let backup_target = vault_dir.join(&fname);
                        if !backup_target.exists() {
                            let _ = std::fs::copy(&p, &backup_target);
                        }
                        all_files_map.insert(fname, p);
                    }
                }
            }
        }
    }

    // Quét bổ sung từ mail_vault (các file nhị phân đã lưu mà game đã xóa)
    if let Ok(entries) = std::fs::read_dir(&vault_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                let fname = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if fname.starts_with("Persistent.Mail.") {
                    all_files_map.entry(fname).or_insert(p);
                }
            }
        }
    }

    println!("[*] Tổng số file cache & vault: {}", all_files_map.len());

    use rayon::prelude::*;
    let catalog_ref = &catalog;
    let file_paths: Vec<PathBuf> = all_files_map.into_values().collect();
    let mut scanned_records = Vec::new();
    for chunk in file_paths.chunks(crate::lilith_stream::MAIL_DECODE_BATCH_SIZE) {
        let chunk_records: Vec<_> = chunk
            .par_iter()
            .filter_map(|p| {
                let fname = p.file_name()?.to_string_lossy();
                let m = decode_mail_file(p).ok()?;
                let box_name = m.get("box").and_then(|v| v.as_str()).unwrap_or("");
                if box_name != "Report" && box_name != "Archive" {
                    return None;
                }
                let btype = classify_mail(&m)?;
                extract_motte_details(&m, &fname, catalog_ref, Some(btype)).ok()
            })
            .collect();
        scanned_records.extend(chunk_records);
    }

    // Upsert vào Persistent Vault
    let new_recs_count = vault.upsert_batch(scanned_records);
    let all_records = vault.get_records();
    let total_records = all_records.len();

    println!(
        "[+] Kho lưu trữ Vault: Tổng {} trận (thêm mới {} trận).",
        total_records, new_recs_count
    );

    let matched_count = all_records
        .iter()
        .filter(|r| mode.matches(&r.battle_type))
        .count();

    let mut active_records = if start_date.is_some() || end_date.is_some() || days.is_some() {
        let filtered = filter_records_by_date(
            all_records.iter().filter(|r| mode.matches(&r.battle_type)),
            start_date,
            end_date,
            days,
        );
        println!("[+] Đã áp dụng bộ lọc ngày: {} thư khớp", filtered.len());
        filtered
    } else {
        println!(
            "[+] Tìm thấy {} thư khớp điều kiện (chế độ: '{}')",
            matched_count,
            mode.as_str()
        );
        all_records
            .iter()
            .filter(|r| mode.matches(&r.battle_type))
            .cloned()
            .collect()
    };
    active_records.sort_by_key(|r| r.time_raw);

    if active_records.is_empty() {
        println!("[-] Không tìm thấy thư nào phù hợp.");
        return Ok(());
    }

    let stats = generate_statistics(&active_records);

    // Export files
    if export_json_flag {
        let json_path = out_dir.join("motte_filtered_mails.json");
        export_json(&active_records, &stats, &json_path)?;
        export_individual_mails_json(&active_records, &mail_dir, &out_dir);
    }

    println!("\n{}", "=".repeat(88));
    println!(
        "TỔNG HỢP THỐNG KÊ {} TRẬN (CHẾ ĐỘ: {})",
        stats.total_battles,
        mode.as_str().to_uppercase()
    );
    println!("{}", "=".repeat(88));
    println!(
        "- Sát thương: Thấp nhất {}% | Cao nhất {}% | Trung bình {:.1}%",
        stats.min_damage_pct, stats.max_damage_pct, stats.avg_damage_pct
    );

    // Detailed Resource Breakdown in Millions
    let res = stats.compute_resources();
    if res.total_rss_val() > 0 {
        println!("{}", "-".repeat(88));
        println!("THỐNG KÊ TÀI NGUYÊN (QUY ĐỔI SỐ LƯỢNG GÓI x GIÁ TRỊ GÓI -> TRIỆU / M):");
        if res.food_packs * 1_000 == res.food_val {
            println!(
                "  • Thực phẩm (Ngô): {:>10} gói  x  1.000 = {:>8.2}M Thực phẩm",
                format_num(res.food_packs),
                res.food_val as f64 / 1_000_000.0
            );
        } else if res.food_packs == res.food_val {
            println!(
                "  • Thực phẩm (Ngô): {:>10} RSS           = {:>8.2}M Thực phẩm",
                format_num(res.food_val),
                res.food_val as f64 / 1_000_000.0
            );
        } else {
            println!(
                "  • Thực phẩm (Ngô): {:>10} gói           = {:>8.2}M Thực phẩm",
                format_num(res.food_packs),
                res.food_val as f64 / 1_000_000.0
            );
        }

        if res.wood_packs * 1_000 == res.wood_val {
            println!(
                "  • Gỗ (Wood):        {:>10} gói  x  1.000 = {:>8.2}M Gỗ",
                format_num(res.wood_packs),
                res.wood_val as f64 / 1_000_000.0
            );
        } else if res.wood_packs == res.wood_val {
            println!(
                "  • Gỗ (Wood):        {:>10} RSS           = {:>8.2}M Gỗ",
                format_num(res.wood_val),
                res.wood_val as f64 / 1_000_000.0
            );
        } else {
            println!(
                "  • Gỗ (Wood):        {:>10} gói           = {:>8.2}M Gỗ",
                format_num(res.wood_packs),
                res.wood_val as f64 / 1_000_000.0
            );
        }

        if res.stone_packs * 750 == res.stone_val {
            println!(
                "  • Đá (Stone):       {:>10} gói  x    750 = {:>8.2}M Đá",
                format_num(res.stone_packs),
                res.stone_val as f64 / 1_000_000.0
            );
        } else if res.stone_packs == res.stone_val {
            println!(
                "  • Đá (Stone):       {:>10} RSS           = {:>8.2}M Đá",
                format_num(res.stone_val),
                res.stone_val as f64 / 1_000_000.0
            );
        } else {
            println!(
                "  • Đá (Stone):       {:>10} gói           = {:>8.2}M Đá",
                format_num(res.stone_packs),
                res.stone_val as f64 / 1_000_000.0
            );
        }

        if res.gold_packs * 500 == res.gold_val {
            println!(
                "  • Vàng (Gold):      {:>10} gói  x    500 = {:>8.2}M Vàng",
                format_num(res.gold_packs),
                res.gold_val as f64 / 1_000_000.0
            );
        } else if res.gold_packs == res.gold_val {
            println!(
                "  • Vàng (Gold):      {:>10} RSS           = {:>8.2}M Vàng",
                format_num(res.gold_val),
                res.gold_val as f64 / 1_000_000.0
            );
        } else {
            println!(
                "  • Vàng (Gold):      {:>10} gói           = {:>8.2}M Vàng",
                format_num(res.gold_packs),
                res.gold_val as f64 / 1_000_000.0
            );
        }
        if res.choice_packs > 0 {
            println!(
                "  • Gói C Tự Chọn:    {:>10} gói          = {:>8.2}M Tùy chọn",
                format_num(res.choice_packs),
                res.choice_val as f64 / 1_000_000.0
            );
        }
        println!(
            "  => TỔNG TÀI NGUYÊN THỰC NHẬN: {:.2} TRIỆU ({} RSS)",
            res.total_rss_val() as f64 / 1_000_000.0,
            format_num(res.total_rss_val())
        );
    }

    // Speedups & Other Drops
    let (t_min, h_min, u_min) = stats.compute_speedups_by_category();
    println!("{}", "-".repeat(88));
    println!("THỐNG KÊ TĂNG TỐC & VẬT PHẨM GIÁ TRỊ CAO:");
    if t_min > 0 {
        let (t_val, t_unit) = speedup_unit.format_kpi(t_min, AppLanguage::Vietnamese);
        println!("  • Tăng tốc huấn luyện: {} {} ({} phút)", t_val, t_unit, format_num(t_min));
    }
    if h_min > 0 {
        let (h_val, h_unit) = speedup_unit.format_kpi(h_min, AppLanguage::Vietnamese);
        println!("  • Tăng tốc chữa lành:  {} {} ({} phút)", h_val, h_unit, format_num(h_min));
    }
    if u_min > 0 {
        let (u_val, u_unit) = speedup_unit.format_kpi(u_min, AppLanguage::Vietnamese);
        println!("  • Tăng tốc đa năng:   {} {} ({} phút)", u_val, u_unit, format_num(u_min));
    }
    let total_exp = stats.compute_total_exp();
    if total_exp > 0 {
        println!("  • Tổng EXP Tướng:      {:.2}M EXP ({})", total_exp as f64 / 1_000_000.0, format_num(total_exp));
    }
    let arrows = stats.compute_arrows_of_resistance();
    if arrows > 0 {
        println!("  • Mũi tên chống trả:   {}", format_num(arrows));
    }
    let books = stats.compute_books_of_covenant();
    if books > 0 {
        println!("  • Sách ước nguyện:     {}", format_num(books));
    }
    let gems = stats.compute_gems();
    if gems > 0 {
        println!("  • Đá quý (Gems):       {}", format_num(gems));
    }

    // Materials (Da, Sắt, Gỗ, Xương) & Action Points (AP)
    let mats = stats.compute_materials();
    let ap = stats.compute_ap();
    if mats.total_pieces() > 0 || ap.total_bottles() > 0 {
        println!("{}", "-".repeat(88));
        println!("THỐNG KÊ VẬT LIỆU TRANG BỊ & ĐIỂM HÀNH ĐỘNG (AP):");
        println!("  • Phân loại: BT = Bình Thường (Cấp 1), NC = Nâng Cao (Cấp 2, x4 BT)");
        if mats.leather_total_converted() > 0 {
            println!(
                "  • Da (Leather):         {:>6} BT | {:>5} NC  => {:>6} quy đổi BT",
                format_num(mats.leather_normal),
                format_num(mats.leather_advanced),
                format_num(mats.leather_total_converted())
            );
        }
        if mats.iron_total_converted() > 0 {
            println!(
                "  • Quặng sắt (Iron Ore):  {:>6} BT | {:>5} NC  => {:>6} quy đổi BT",
                format_num(mats.iron_normal),
                format_num(mats.iron_advanced),
                format_num(mats.iron_total_converted())
            );
        }
        if mats.ebony_total_converted() > 0 {
            println!(
                "  • Gỗ mun (Ebony):        {:>6} BT | {:>5} NC  => {:>6} quy đổi BT",
                format_num(mats.ebony_normal),
                format_num(mats.ebony_advanced),
                format_num(mats.ebony_total_converted())
            );
        }
        if mats.bone_total_converted() > 0 {
            println!(
                "  • Xương động vật (Bone): {:>6} BT | {:>5} NC  => {:>6} quy đổi BT",
                format_num(mats.bone_normal),
                format_num(mats.bone_advanced),
                format_num(mats.bone_total_converted())
            );
        }
        if mats.total_pieces() > 0 {
            println!(
                "  => TỔNG VẬT LIỆU: {} BT + {} NC = {} mảnh ({} quy đổi bình thường)",
                format_num(mats.total_normal()),
                format_num(mats.total_advanced()),
                format_num(mats.total_pieces()),
                format_num(mats.total_converted_normal())
            );
        }
        if ap.total_bottles() > 0 {
            println!(
                "  • Điểm Hành Động (AP):     {} bình 50 AP, {} bình 100 AP => TỔNG {} AP ({} bình)",
                format_num(ap.ap_50),
                format_num(ap.ap_100),
                format_num(ap.total_ap_points()),
                format_num(ap.total_bottles())
            );
        }
    }

    println!("{}", "-".repeat(88));
    println!(
        "{:<38} | {:<10} | {:>10} | {:>16} | {:>10}",
        "VẬT PHẨM (loot.yaml)", "TYPE/SUB", "TỔNG CỘNG", "QUY ĐỔI / THỰC", "TRUNG BÌNH"
    );
    println!("{}", "-".repeat(88));

    let mut sorted_items: Vec<_> = stats.item_stats.values().collect();
    sorted_items.sort_by_key(|a| std::cmp::Reverse(a.total_quantity));

    for it in sorted_items {
        let conv_str = get_item_converted_display(it.itype, it.sub_type, it.total_quantity).unwrap_or_else(|| "-".to_string());
        println!(
            "{:<38} | {}/{:<7} | {:>10} | {:>16} | {:>10.1}",
            it.name_vi, it.itype, it.sub_type, format_num(it.total_quantity), conv_str, it.avg_per_battle
        );
    }
    println!("{}", "=".repeat(88));

    Ok(())
}

fn run_gui_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Đảm bảo thư mục làm việc luôn là thư mục chứa file .exe (tránh lỗi khi Windows Autostart chạy từ C:\Windows\System32)
    let _ = crate::autostart::ensure_working_dir_is_exe_dir();

    let allow_multiple = args.iter().any(|a| a == "--multi-instance");
    let start_minimized = args.iter().any(|a| a == "--minimized" || a == "--tray");

    // Kiểm tra nếu app đã mở sẵn hay chưa để tránh mở thêm
    let single_instance = if !allow_multiple {
        match crate::single_instance::windows::check_or_acquire() {
            crate::single_instance::windows::CheckResult::AlreadyExists => {
                #[cfg(windows)]
                attach_parent_console();
                println!("[-] Ứng dụng ROK Loot Watcher đang chạy sẵn. Đã chuyển tiêu điểm về cửa sổ đang hoạt động.");
                return Ok(());
            }
            crate::single_instance::windows::CheckResult::Running(guard) => Some(guard),
        }
    } else {
        None
    };

    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let cur_dir = std::env::current_dir().unwrap_or_else(|_| {
        exe_dir.clone().unwrap_or_else(|| PathBuf::from("."))
    });

    let mut icon_candidates = vec![
        cur_dir.join("rok_icon.png"),
        cur_dir.join("rok_icon.ico"),
    ];

    if let Some(ref ed) = exe_dir {
        icon_candidates.push(ed.join("rok_icon.png"));
        icon_candidates.push(ed.join("rok_icon.ico"));
        if let Some(parent) = ed.parent() {
            icon_candidates.push(parent.join("rok_icon.png"));
            icon_candidates.push(parent.join("rok_icon.ico"));
        }
    }

    let icon_path = icon_candidates.into_iter().find(|p| p.is_file());

    let tray = match SystemTrayManager::new(icon_path.as_deref()) {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("[!] Cảnh báo: Không thể khởi tạo System Tray: {}", e);
            None
        }
    };

    let catalog = LootCatalog::load_from_candidates(None);
    let (tx_ui, rx_ui) = channel();

    let config = crate::model::AppConfig::load();
    let initial_mail_dir = config.mail_dir.clone().unwrap_or_else(crate::watcher::detect_default_mail_dir);
    let initial_mode = config.mode.unwrap_or(FilterMode::AllTargets);

    let watcher = Arc::new(RoKMailWatcher::new(
        Some(initial_mail_dir),
        Some(cur_dir),
        initial_mode,
        catalog,
        tx_ui,
    ));

    // Load window icon for eframe (từ file ngoài hoặc fallback dữ liệu nhúng sẵn)
    const EMBEDDED_ICON_BYTES: &[u8] = include_bytes!("../rok_icon.png");
    let window_icon = icon_path.as_ref().and_then(|p| {
        if let Ok(img) = image::open(p) {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }
            .into()
        } else {
            None
        }
    }).or_else(|| {
        if let Ok(img) = image::load_from_memory(EMBEDDED_ICON_BYTES) {
            let rgba = img.into_rgba8();
            let (w, h) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }
            .into()
        } else {
            None
        }
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Rise of Kingdoms - Motte, Fort & Barbarian Loot Watcher")
        .with_inner_size(egui::vec2(1380.0, 860.0))
        .with_min_inner_size(egui::vec2(1100.0, 700.0));

    if start_minimized {
        viewport = viewport.with_visible(false);
    }

    if let Some(icon) = window_icon {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let watcher_clone = Arc::clone(&watcher);
    let single_instance_opt = single_instance;
    let _ = eframe::run_native(
        "RoK Loot Watcher",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(RoKLootWatcherApp::new(
                cc,
                watcher_clone,
                rx_ui,
                tray,
                single_instance_opt,
                start_minimized,
            )))
        }),
    );

    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_init_performance() {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let start = std::time::Instant::now();
        let vault = crate::vault::MailVault::new(&root_dir);
        let elapsed = start.elapsed();
        // Warm vault init should complete in well under 1000ms even in debug mode (was >670ms prior to optimization)
        assert!(elapsed.as_millis() < 1000, "MailVault::init took too long: {:?}", elapsed);
        assert!(vault.total_count() > 0);
    }
}
