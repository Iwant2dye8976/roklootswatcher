use std::fs;
use std::path::Path;
use serde_json::json;

use crate::lilith_stream::decode_mail_file;
use crate::model::{BattleRecord, Statistics};

pub fn export_json(
    records: &[BattleRecord],
    stats: &Statistics,
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = out_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let payload = json!({
        "statistics": stats,
        "battles": records
    });

    let s = serde_json::to_string_pretty(&payload)?;
    fs::write(out_path, s)?;
    println!("[+] Đã xuất file JSON: {}", out_path.display());
    Ok(())
}

pub fn export_individual_mails_json(
    records: &[BattleRecord],
    mail_dir: &Path,
    out_dir: &Path,
) {
    let mails_json_dir = out_dir.join("filtered_mails_json");
    if let Err(e) = fs::create_dir_all(&mails_json_dir) {
        eprintln!("[-] Không thể tạo thư mục {}: {}", mails_json_dir.display(), e);
        return;
    }

    let binary_vault_dir = out_dir.join("mail_vault");

    for r in records {
        let out_file = mails_json_dir.join(format!("{}.json", r.filename));
        // Nếu file json đã tồn tại và hợp lệ, giữ nguyên không cần ghi lại
        if out_file.is_file() {
            continue;
        }

        // Tìm nguồn nhị phân: ưu tiên mailcache, nếu game đã xóa thì lấy từ mail_vault
        let mut src_path = mail_dir.join(&r.filename);
        if !src_path.is_file() {
            src_path = binary_vault_dir.join(&r.filename);
        }

        if src_path.is_file() {
            if let Ok(mut m_full) = decode_mail_file(&src_path) {
                let box_name = m_full.get("box").and_then(|v| v.as_str()).unwrap_or("");
                if box_name != "Report" && box_name != "Archive" {
                    continue;
                }

                if let crate::lilith_stream::LilithValue::Table(ref mut map) = m_full {
                    let mut meta = indexmap::IndexMap::new();
                    meta.insert("damage_pct".into(), crate::lilith_stream::LilithValue::Number(r.damage_pct as f64));
                    meta.insert("plunder_tier".into(), crate::lilith_stream::LilithValue::Number(r.plunder_tier as f64));
                    meta.insert("time_vn".into(), crate::lilith_stream::LilithValue::String(r.time_vn.clone()));
                    map.insert("_stats_metadata".into(), crate::lilith_stream::LilithValue::Table(meta));

                    if let Ok(s) = serde_json::to_string_pretty(&m_full) {
                        let _ = fs::write(&out_file, s);
                    }
                }
            }
        } else {
            // Trường hợp file nhị phân đã bị game xóa và chưa kịp vào vault,
            // xuất bản ghi từ dữ liệu cấu trúc BattleRecord đã lưu trong RAM/Vault
            if let Ok(s) = serde_json::to_string_pretty(r) {
                let _ = fs::write(&out_file, s);
            }
        }
    }
}
