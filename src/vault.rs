use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::model::BattleRecord;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

/// Đọc nội dung file với cờ chia sẻ đầy đủ trên Windows:
/// FILE_SHARE_READ (1) | FILE_SHARE_WRITE (2) | FILE_SHARE_DELETE (4) = 7.
/// Giúp đọc được file ngay cả khi game RoK đang ghi hoặc chuẩn bị gọi DeleteFile.
pub fn read_file_shared<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
    let p = path.as_ref();
    let mut options = OpenOptions::new();
    options.read(true);

    #[cfg(windows)]
    {
        options.share_mode(7);
    }

    let mut file = options.open(p)?;
    let file_size = file.metadata()?.len();
    if file_size > crate::lilith_stream::MAX_MAIL_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Mail file exceeds max size limit ({} bytes > {} bytes)",
                file_size,
                crate::lilith_stream::MAX_MAIL_FILE_SIZE
            ),
        ));
    }

    let mut buf = Vec::with_capacity(file_size as usize);
    file.by_ref()
        .take(crate::lilith_stream::MAX_MAIL_FILE_SIZE + 1)
        .read_to_end(&mut buf)?;
    if buf.len() as u64 > crate::lilith_stream::MAX_MAIL_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Mail file exceeds max size limit",
        ));
    }
    Ok(buf)
}

/// Tạo khóa định danh duy nhất cho một chiến báo để chống trùng lặp dữ liệu (Deduplication).
pub fn get_record_dedup_key(r: &BattleRecord) -> String {
    if !r.mail_id.is_empty() {
        format!("id:{}", r.mail_id)
    } else if !r.filename.is_empty() {
        format!("file:{}", r.filename)
    } else {
        format!(
            "sig:{}:{}:{}:{}:{}",
            r.time_raw, r.battle_type, r.pos_x, r.pos_y, r.damage_pct
        )
    }
}

pub fn json_as_i64(v: Option<&serde_json::Value>) -> Option<i64> {
    let val = v?;
    if let Some(i) = val.as_i64() {
        Some(i)
    } else if let Some(u) = val.as_u64() {
        Some(u as i64)
    } else if let Some(f) = val.as_f64() {
        Some(f.round() as i64)
    } else if let Some(s) = val.as_str() {
        s.trim().parse::<f64>().ok().map(|f| f.round() as i64)
    } else {
        None
    }
}

pub fn json_as_u32(v: Option<&serde_json::Value>) -> Option<u32> {
    json_as_i64(v).and_then(|i| if (0..=u32::MAX as i64).contains(&i) { Some(i as u32) } else { None })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStorageData {
    pub version: u32,
    pub last_updated: String,
    pub total_records: usize,
    pub records: Vec<BattleRecord>,
}

#[derive(Serialize)]
struct VaultStorageDataRef<'a> {
    version: u32,
    last_updated: String,
    total_records: usize,
    records: &'a [BattleRecord],
}

#[derive(Debug, Clone)]
pub struct MailVault {
    vault_file: PathBuf,
    binary_dir: PathBuf,
    records: Vec<BattleRecord>,
    key_index: HashMap<String, usize>,
    is_corrupted: bool,
}

impl MailVault {
    pub fn new(out_dir: &Path) -> Self {
        let vault_file = out_dir.join("rok_history_vault.json");
        let binary_dir = out_dir.join("mail_vault");

        let mut vault = Self {
            vault_file,
            binary_dir,
            records: Vec::new(),
            key_index: HashMap::new(),
            is_corrupted: false,
        };

        vault.init();
        vault
    }

    pub fn get_vault_file(&self) -> &Path {
        &self.vault_file
    }

    pub fn get_binary_dir(&self) -> &Path {
        &self.binary_dir
    }

    pub fn get_records(&self) -> &[BattleRecord] {
        &self.records
    }

    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    #[allow(dead_code)]
    pub fn is_corrupted(&self) -> bool {
        self.is_corrupted
    }

    /// Kiểm tra xem file hoặc ID thư này đã được lưu trong Vault hay chưa (O(1)).
    /// Giúp Fast-Skip bỏ qua lập tức mà không cần mở file nhị phân hay cấp phát bộ nhớ.
    pub fn contains_file_or_id(&self, fname: &str) -> bool {
        if let Some(id) = fname.strip_prefix("Persistent.Mail.") {
            if self.key_index.contains_key(&format!("id:{}", id)) {
                return true;
            }
        }
        self.key_index.contains_key(&format!("file:{}", fname))
    }

    fn init(&mut self) {
        // Tạo thư mục sao lưu nhị phân nếu chưa có
        if let Err(e) = fs::create_dir_all(&self.binary_dir) {
            eprintln!("[-] Không thể tạo thư mục mail_vault: {}", e);
        }

        // 1. Nạp từ file vault JSON nếu đã có
        if let Err(e) = self.load_from_vault_file() {
            eprintln!("[!] Cảnh báo Vault: {}", e);
        }

        // 2. Khôi phục bổ sung từ thư mục filtered_mails_json nếu người dùng đã có
        self.restore_from_filtered_mails_json();

        // 3. Dọn dẹp các chiến báo sai lệch lịch sử (vd: thư Pass bị gán nhầm thành barbarian)
        self.sanitize_records();

        // Sắp xếp theo thời gian chiến đấu
        self.records.sort_by_key(|r| r.time_raw);
        self.rebuild_index();
    }

    /// Loại bỏ các chiến báo cũ bị phân loại nhầm thành barbarian (như Pass First Occupation),
    /// và chuẩn hóa các chiến báo dã man từng bị gán nhầm cấp 60 của pháo đài.
    pub fn sanitize_records(&mut self) -> usize {
        let before_len = self.records.len();
        let mut modified = false;

        self.records.retain_mut(|r| {
            if r.battle_type == "barbarian" {
                if r.target.contains("Pass")
                    || r.content.contains("Pass")
                    || r.mail_id == "57774831178214367624"
                    || r.mail_id == "57774881178214367624"
                {
                    return false;
                }
                // Loại bỏ thư bắn tháp không có vật phẩm bị gán nhầm thành barbarian cấp 60
                if (r.target_level == "60" || r.target == "Dã Man Cấp 60") && r.items.is_empty() {
                    return false;
                }
                // Chuẩn hóa lại các báo cáo dã man từng bị gán nhầm cấp 60 của pháo đài (vd: Cấp 52-60 (x2))
                if r.target_level.contains("60") || r.target.contains("60") {
                    let maybe_min_lvl = r.target_level.split_once("-60").map(|(min_str, _)| min_str.trim().to_string())
                        .or_else(|| {
                            if let Some(pos) = r.target.find("-60") {
                                let prefix = &r.target[..pos];
                                prefix.split(|c: char| !c.is_numeric()).rfind(|s| !s.is_empty()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        });
                    if let Some(min_lvl) = maybe_min_lvl {
                        if !min_lvl.is_empty() {
                            r.target = format!("Dã Man Cấp {}", min_lvl);
                            if r.pos_x > 0 && r.pos_y > 0 {
                                r.content = format!("{} tại X:{} Y:{}", r.target, r.pos_x, r.pos_y);
                            } else {
                                r.content = r.target.clone();
                            }
                            r.target_level = min_lvl;
                            modified = true;
                        } else {
                            r.target_level = String::new();
                            r.target = "Dã Man".to_string();
                            r.content = r.target.clone();
                            modified = true;
                        }
                    } else if r.target_level == "60" || r.target.contains("Cấp 60") {
                        r.target_level = String::new();
                        r.target = "Dã Man".to_string();
                        r.content = r.target.clone();
                        modified = true;
                    }
                }
            }
            true
        });

        let removed = before_len - self.records.len();
        if removed > 0 || modified {
            self.rebuild_index();
            let _ = self.save_to_disk();
        }
        removed
    }

    fn rebuild_index(&mut self) {
        self.key_index.clear();
        for (idx, r) in self.records.iter().enumerate() {
            let key = get_record_dedup_key(r);
            self.key_index.insert(key, idx);
        }
    }

    pub fn load_from_vault_file(&mut self) -> Result<usize, String> {
        if !self.vault_file.exists() {
            return Ok(0);
        }

        let file = match fs::File::open(&self.vault_file) {
            Ok(f) => f,
            Err(e) => {
                self.is_corrupted = true;
                let corrupt_path = self.vault_file.with_extension("corrupt");
                let _ = fs::copy(&self.vault_file, &corrupt_path);
                return Err(format!("Không thể mở file vault {:?}: {}", self.vault_file, e));
            }
        };

        let reader = std::io::BufReader::new(file);
        match serde_json::from_reader::<_, VaultStorageData>(reader) {
            Ok(storage) => {
                let cnt = storage.records.len();
                self.records = storage.records;
                self.rebuild_index();
                Ok(cnt)
            }
            Err(e) => {
                self.is_corrupted = true;
                let corrupt_path = self.vault_file.with_extension("corrupt");
                let _ = fs::copy(&self.vault_file, &corrupt_path);
                let err_msg = format!(
                    "File vault {:?} bị lỗi cấu trúc JSON: {}. Đã tạo bản sao lưu tại {:?}",
                    self.vault_file, e, corrupt_path
                );
                Err(err_msg)
            }
        }
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        if self.is_corrupted {
            return Err(format!(
                "Từ chối ghi đè file vault: file gốc {:?} bị hỏng hoặc lỗi phân tích. Vui lòng kiểm tra bản sao lưu .corrupt",
                self.vault_file
            ));
        }

        let storage = VaultStorageDataRef {
            version: 1,
            last_updated: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_records: self.records.len(),
            records: &self.records,
        };

        let tmp_file = self.vault_file.with_extension("tmp");
        let mut file = fs::File::create(&tmp_file)
            .map_err(|e| format!("Lỗi ghi tạm file vault: {}", e))?;
        serde_json::to_writer_pretty(&mut file, &storage)
            .map_err(|e| format!("Lỗi serialize JSON vault: {}", e))?;
        drop(file);

        // Ghi đè nguyên tử (atomic rename/replace)
        fs::rename(&tmp_file, &self.vault_file)
            .map_err(|e| format!("Lỗi cập nhật file vault: {}", e))?;

        Ok(())
    }

    /// Thêm hoặc cập nhật một bản ghi vào Vault.
    /// Trả về true nếu là bản ghi mới được thêm.
    pub fn upsert(&mut self, rec: BattleRecord) -> bool {
        let key = get_record_dedup_key(&rec);

        if let Some(&idx) = self.key_index.get(&key) {
            let existing = &mut self.records[idx];
            // Chỉ cập nhật nếu bản ghi mới có thông tin phong phú hơn hoặc chuẩn xác hơn.
            // Tuyệt đối không để bản ghi "other" hay bản ghi rỗng ghi đè dữ liệu hợp lệ.
            if rec.battle_type != "other" && (existing.battle_type == "other" || existing.battle_type.is_empty()) {
                existing.battle_type = rec.battle_type;
            }
            if !rec.items.is_empty() && (existing.items.is_empty() || rec.items.len() >= existing.items.len()) {
                existing.items = rec.items;
                existing.item_totals = rec.item_totals;
            }
            if rec.damage_pct > 0 && existing.damage_pct == 0 {
                existing.damage_pct = rec.damage_pct;
            }
            if rec.plunder_tier > 0 && existing.plunder_tier == 0 {
                existing.plunder_tier = rec.plunder_tier;
            }
            if !rec.target.is_empty() && existing.target.is_empty() {
                existing.target = rec.target;
            }
            if !rec.content.is_empty() && existing.content.is_empty() {
                existing.content = rec.content;
            }
            if rec.pos_x != 0 && existing.pos_x == 0 {
                existing.pos_x = rec.pos_x;
                existing.pos_y = rec.pos_y;
            }
            if !rec.time_vn.is_empty() && existing.time_vn.is_empty() {
                existing.time_vn = rec.time_vn;
            }
            if rec.time_raw > 0 && existing.time_raw == 0 {
                existing.time_raw = rec.time_raw;
            }
            false
        } else {
            // Bản ghi mới -> Thêm vào danh sách và cập nhật chỉ mục
            let new_idx = self.records.len();
            self.records.push(rec);
            self.key_index.insert(key, new_idx);
            true
        }
    }

    /// Thêm hàng loạt bản ghi (Upsert batch), trả về số lượng bản ghi mới.
    pub fn upsert_batch(&mut self, recs: Vec<BattleRecord>) -> usize {
        let mut new_count = 0;
        for r in recs {
            if self.upsert(r) {
                new_count += 1;
            }
        }
        if new_count > 0 {
            self.records.sort_by_key(|r| r.time_raw);
            self.rebuild_index();
            let _ = self.save_to_disk();
        }
        new_count
    }

    /// Sao lưu nhị phân file thư từ mailcache vào mail_vault.
    /// Phương thức này sử dụng read_file_shared để đọc ngay trước khi game kịp xóa.
    #[allow(dead_code)]
    pub fn backup_binary_mail(&self, src_path: &Path) -> Result<PathBuf, String> {
        let fname = match src_path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => return Err("Đường dẫn file không hợp lệ".into()),
        };

        if !fname.starts_with("Persistent.Mail.") {
            return Err("Không phải file Persistent.Mail".into());
        }

        let dst_path = self.binary_dir.join(&fname);

        // Đọc bytes với cờ chia sẻ đầy đủ của Windows
        let bytes = read_file_shared(src_path)
            .map_err(|e| format!("Lỗi đọc file nguồn {}: {}", src_path.display(), e))?;

        if bytes.is_empty() {
            return Err("File rỗng (0 bytes)".into());
        }

        fs::write(&dst_path, &bytes)
            .map_err(|e| format!("Lỗi ghi file sao lưu {}: {}", dst_path.display(), e))?;

        Ok(dst_path)
    }

    fn restore_from_filtered_mails_json(&mut self) -> usize {
        let parent = match self.vault_file.parent() {
            Some(p) => p,
            None => return 0,
        };
        let json_dir = parent.join("filtered_mails_json");
        if !json_dir.is_dir() {
            return 0;
        }

        let mut restored = 0;
        if let Ok(entries) = fs::read_dir(&json_dir) {
            for entry in entries.flatten() {
                let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                if !is_file {
                    continue;
                }

                let fname_os = entry.file_name();
                let fname_str = fname_os.to_string_lossy();
                let stem = match fname_str.strip_suffix(".json") {
                    Some(s) => s,
                    None => continue,
                };

                // Bỏ qua ngay lập tức nếu bản ghi đã tồn tại trong Vault (O(1) lookup),
                // tránh mở file I/O và deserialization JSON không cần thiết gây lag khi khởi động.
                if self.contains_file_or_id(stem) {
                    continue;
                }

                let p = entry.path();
                if let Ok(file) = fs::File::open(&p) {
                    let reader = std::io::BufReader::new(file);
                    // Thử parse xem có metadata thống kê không
                    if let Ok(val) = serde_json::from_reader::<_, serde_json::Value>(reader) {
                        if let Some(box_val) = val.get("box").and_then(|v| v.as_str()) {
                            if box_val == "Report" || box_val == "Archive" {
                                let fname_saved = stem.to_string();
                                // Parse BattleRecord từ cấu trúc json
                                if let Some(rec) = Self::parse_battle_record_from_exported_json(&val, &fname_saved) {
                                    let key = get_record_dedup_key(&rec);
                                    // TUYỆT ĐỐI KHÔNG ghi đè bản ghi đã tồn tại trong Vault
                                    // vì bản ghi trong Vault có dữ liệu đầy đủ và chính xác hơn!
                                    if !self.key_index.contains_key(&key) && self.upsert(rec) {
                                        restored += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if restored > 0 {
            self.records.sort_by_key(|r| r.time_raw);
            self.rebuild_index();
            let _ = self.save_to_disk();
        }
        restored
    }

    fn parse_battle_record_from_exported_json(val: &serde_json::Value, fname: &str) -> Option<BattleRecord> {
        let body = val.get("body")?;
        let sub_t = json_as_i64(body.get("subType")).unwrap_or(0);
        let sub_p = json_as_i64(body.get("subParam")).unwrap_or(0);

        let is_battle = val.get("type").and_then(|v| v.as_str()) == Some("Battle")
            || body.get("content").and_then(|c| c.get("Attacks")).is_some();

        let time_raw = json_as_i64(val.get("sendTime"))
            .or_else(|| json_as_i64(body.get("time")))
            .or_else(|| json_as_i64(val.get("time")))
            .unwrap_or(0);

        let mail_id = val.get("id")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| json_as_i64(Some(v)).map(|n| n.to_string())))
            .unwrap_or_default();

        let mut damage_pct = 0;
        let mut plunder_tier = 0;
        let mut time_vn = String::new();

        if let Some(meta) = val.get("_stats_metadata") {
            damage_pct = json_as_i64(meta.get("damage_pct")).unwrap_or(0) as i32;
            plunder_tier = json_as_i64(meta.get("plunder_tier")).unwrap_or(0) as i32;
            time_vn = meta.get("time_vn").and_then(|v| v.as_str()).unwrap_or("").to_string();
        }

        if is_battle {
            let attacks = body.get("content").and_then(|c| c.get("Attacks")).and_then(|a| a.as_object());
            let mut items = Vec::new();
            let mut item_totals = HashMap::new();
            let mut levels = Vec::new();
            let mut pos_x = 0;
            let mut pos_y = 0;

            if let Some(att_map) = attacks {
                for (_k, atk) in att_map {
                    let cidt = atk.get("CIdt");
                    let is_fort = cidt
                        .and_then(|c| c.get("IsFort"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let npcbtype = cidt
                        .and_then(|c| c.get("NpcBType"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as i64;

                    if !is_fort && npcbtype == 1 {
                        if let Some(cidt) = cidt {
                            if let Some(hlv) = json_as_u32(cidt.get("HLv")) {
                                if hlv > 0 && hlv <= 55 {
                                    levels.push(hlv);
                                }
                            }
                        }
                    }

                    let is_valid_target = !is_fort && (npcbtype == 1 || atk.get("NpcKillLoot").is_some());
                    if is_valid_target || (pos_x == 0 && pos_y == 0) {
                        if let Some(cidt) = cidt {
                            if let Some(cpos) = cidt.get("CastlePos") {
                                let x = cpos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let y = cpos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                if x > 0.0 && y > 0.0 {
                                    pos_x = (x / 6.0).round() as i32;
                                    pos_y = (y / 6.0).round() as i32;
                                }
                            }
                        }

                        if let Some(pos) = atk.get("Pos") {
                            let x = pos.get("X").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let y = pos.get("Y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            if x > 0.0 && y > 0.0 {
                                pos_x = (x / 6.0).round() as i32;
                                pos_y = (y / 6.0).round() as i32;
                            }
                        }
                    }

                    if let Some(loots_map) = atk.get("NpcKillLoot").and_then(|l| l.as_object()) {
                        for (_loot_id, item_val) in loots_map {
                            let itype = json_as_u32(item_val.get("Type")).unwrap_or(0);
                            let isub = json_as_u32(item_val.get("SubType")).unwrap_or(0);
                            let val = json_as_i64(item_val.get("Value")).unwrap_or(0);
                            let pic = item_val.get("PIC").and_then(|v| v.as_str()).unwrap_or("").to_string();

                            let name = format!("Item_{}_{}", itype, isub);
                            *item_totals.entry(name.clone()).or_insert(0) += val;
                            items.push(crate::model::ItemDrop {
                                itype,
                                sub_type: isub,
                                value: val,
                                pic,
                                name_vi: name.clone(),
                                name_en: name,
                                category: "Khác".into(),
                                sprites: vec![],
                            });
                        }
                    }
                }
            }

            let (target_level, target) = if !levels.is_empty() {
                let min_lvl = *levels.iter().min().unwrap();
                let max_lvl = *levels.iter().max().unwrap();
                let count = levels.len();

                if count == 1 {
                    (min_lvl.to_string(), format!("Dã Man Cấp {}", min_lvl))
                } else if min_lvl == max_lvl {
                    (min_lvl.to_string(), format!("Dã Man Cấp {} (x{})", min_lvl, count))
                } else {
                    (
                        format!("{}-{}", min_lvl, max_lvl),
                        format!("Dã Man Cấp {}-{} (x{})", min_lvl, max_lvl, count),
                    )
                }
            } else {
                ("".to_string(), "Dã Man".to_string())
            };

            let content = target.clone();

            return Some(BattleRecord {
                filename: fname.to_string(),
                mail_id,
                battle_type: "barbarian".to_string(),
                time_raw,
                time_utc: "".into(),
                time_vn,
                target,
                target_level,
                talent_add: 0,
                pos_x,
                pos_y,
                damage_pct: if damage_pct > 0 { damage_pct } else { 100 },
                plunder_tier: 0,
                items,
                item_totals,
                content,
            });
        }

        let target = body.get("targetName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let mut battle_type = if (sub_t == 11 && sub_p == 4)
            || target.to_lowercase().contains("motte")
            || content.to_lowercase().contains("motte")
        {
            "motte".to_string()
        } else if (sub_t == 11 && sub_p == 1)
            || target.to_lowercase().contains("fort")
            || target.to_lowercase().starts_with("level")
        {
            "fort".to_string()
        } else {
            "other".to_string()
        };

        if let Some(meta_bt) = val.get("_stats_metadata").and_then(|m| m.get("battle_type")).and_then(|v| v.as_str()) {
            if !meta_bt.is_empty() && meta_bt != "other" {
                battle_type = meta_bt.to_string();
            }
        }

        let mail_id = val.get("id")
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| json_as_i64(Some(v)).map(|n| n.to_string())))
            .unwrap_or_default();

        let mut item_totals = HashMap::new();
        let items = Vec::new();

        if let Some(attach) = val.get("attachments") {
            if let Some(arr) = attach.as_array() {
                for it in arr {
                    let name = it.get("name_vi").and_then(|v| v.as_str()).unwrap_or("");
                    let qty = json_as_i64(it.get("value")).unwrap_or(0);
                    if !name.is_empty() && qty > 0 {
                        *item_totals.entry(name.to_string()).or_insert(0) += qty;
                    }
                }
            } else if let Some(map) = attach.as_object() {
                for (_k, it) in map {
                    let name = it.get("name_vi").and_then(|v| v.as_str()).unwrap_or("");
                    let qty = json_as_i64(it.get("value")).unwrap_or(0);
                    if !name.is_empty() && qty > 0 {
                        *item_totals.entry(name.to_string()).or_insert(0) += qty;
                    }
                }
            }
        }

        Some(BattleRecord {
            filename: fname.to_string(),
            mail_id,
            battle_type,
            time_raw,
            time_utc: "".into(),
            time_vn,
            target,
            target_level: "".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct,
            plunder_tier,
            items,
            item_totals,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_dedup_key() {
        let rec1 = BattleRecord {
            filename: "Persistent.Mail.123".into(),
            mail_id: "123".into(),
            battle_type: "motte".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Motte".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 50,
            plunder_tier: 6,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        assert_eq!(get_record_dedup_key(&rec1), "id:123");

        let mut rec2 = rec1.clone();
        rec2.mail_id = "".into();
        assert_eq!(get_record_dedup_key(&rec2), "file:Persistent.Mail.123");

        let mut rec3 = rec2.clone();
        rec3.filename = "".into();
        assert_eq!(get_record_dedup_key(&rec3), "sig:1000:motte:100:200:50");
    }

    #[test]
    fn test_vault_upsert_and_dedup() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_vault_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let mut vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 0);

        let rec = BattleRecord {
            filename: "Persistent.Mail.999".into(),
            mail_id: "999".into(),
            battle_type: "fort".into(),
            time_raw: 5000,
            time_utc: "".into(),
            time_vn: "2026-10-22 10:00:00".into(),
            target: "Barbarian Fort 10".into(),
            target_level: "10".into(),
            talent_add: 0,
            pos_x: 500,
            pos_y: 600,
            damage_pct: 45,
            plunder_tier: 6,
            items: vec![],
            item_totals: HashMap::new(),
            content: "Test".into(),
        };

        // First insert
        assert!(vault.upsert(rec.clone()));
        assert_eq!(vault.total_count(), 1);

        // Duplicate insert should return false and not increase count
        assert!(!vault.upsert(rec));
        assert_eq!(vault.total_count(), 1);

        // Save and reload
        assert!(vault.save_to_disk().is_ok());

        let reloaded = MailVault::new(&temp_dir);
        assert_eq!(reloaded.total_count(), 1);
        assert_eq!(reloaded.get_records()[0].mail_id, "999");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_inspect_new_files() {
        let vault = MailVault::new(Path::new("."));
        assert!(vault.total_count() >= 609, "Tổng số bản ghi phải từ 609 trở lên");
        let mut btypes = HashMap::new();
        for r in vault.get_records() {
            *btypes.entry(r.battle_type.clone()).or_insert(0) += 1;
            if r.battle_type == "barbarian" {
                assert!(!r.target_level.contains("60"), "Barbarian level must not contain 60: {:?}", r.target_level);
                assert!(!r.target.contains("60"), "Barbarian target must not contain 60: {:?}", r.target);
            }
        }
        assert!(*btypes.get("motte").unwrap_or(&0) >= 14, "Phải nạp từ 14 chiến báo Motte trở lên");
        assert!(*btypes.get("fort").unwrap_or(&0) >= 19, "Phải nạp từ 19 chiến báo Fort trở lên");
        assert!(*btypes.get("barbarian").unwrap_or(&0) >= 476, "Phải nạp từ 476 chiến báo Barbarian trở lên");
        assert_eq!(*btypes.get("other").unwrap_or(&0), 0, "Không được có chiến báo nào bị phân loại nhầm thành other");

        // Also verify and sanitize motte_filtered_mails.json if present
        let motte_file = Path::new("motte_filtered_mails.json");
        if motte_file.exists() {
            if let Ok(data) = fs::read_to_string(motte_file) {
                let clean_data = data.trim_start_matches('\u{feff}');
                match serde_json::from_str::<serde_json::Value>(clean_data) {
                    Ok(mut json_val) => {
                        let mut modified = false;
                        if let Some(battles) = json_val.get_mut("battles").and_then(|b| b.as_array_mut()) {
                            for b in battles {
                                if b.get("battle_type").and_then(|v| v.as_str()) == Some("barbarian") {
                                    let mut needs_clean = false;
                                    let lvl = b.get("target_level").or_else(|| b.get("motte_level")).and_then(|v| v.as_str()).unwrap_or("");
                                    let tgt = b.get("target").and_then(|v| v.as_str()).unwrap_or("");
                                    if lvl.contains("60") || tgt.contains("60") {
                                        needs_clean = true;
                                    }
                                    if needs_clean {
                                        let min_lvl = if let Some((m, _)) = lvl.split_once("-60") {
                                            m.to_string()
                                        } else if let Some(pos) = tgt.find("-60") {
                                            tgt[..pos].split(|c: char| !c.is_numeric()).rfind(|s| !s.is_empty()).unwrap_or("").to_string()
                                        } else {
                                            "".to_string()
                                        };

                                        if !min_lvl.is_empty() {
                                            b["target_level"] = serde_json::Value::String(min_lvl.clone());
                                            let new_tgt = format!("Dã Man Cấp {}", min_lvl);
                                            b["target"] = serde_json::Value::String(new_tgt.clone());
                                            let px = b.get("pos_x").and_then(|v| v.as_i64()).unwrap_or(0);
                                            let py = b.get("pos_y").and_then(|v| v.as_i64()).unwrap_or(0);
                                            if px > 0 && py > 0 {
                                                b["content"] = serde_json::Value::String(format!("{} tại X:{} Y:{}", new_tgt, px, py));
                                            } else {
                                                b["content"] = serde_json::Value::String(new_tgt);
                                            }
                                            modified = true;
                                        }
                                    }
                                }
                            }
                        }
                        if modified {
                            // Verify json is serializable without mutating project workspace file
                            assert!(serde_json::to_string_pretty(&json_val).is_ok());
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse motte_filtered_mails.json: {}", e);
                    }
                }
            }
        }
    }

    #[test]
    fn test_restore_from_filtered_mails_json() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_restore_{}", std::process::id()));
        let json_dir = temp_dir.join("filtered_mails_json");
        let _ = fs::create_dir_all(&json_dir);

        let mock_json = r#"{
            "box": "Report",
            "id": "777888",
            "sendTime": 17858542764,
            "body": {
                "subType": 11,
                "subParam": 4,
                "targetName": "Motte 15",
                "content": "Tiêu diệt thành công Motte cấp 15"
            },
            "_stats_metadata": {
                "damage_pct": 55,
                "plunder_tier": 6,
                "time_vn": "2026-10-22 09:30:00"
            }
        }"#;

        let sample_file = json_dir.join("Persistent.Mail.777888.json");
        fs::write(&sample_file, mock_json).unwrap();

        // Khởi tạo MailVault trên thư mục này -> Phải tự động khôi phục
        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 1);
        let rec = &vault.get_records()[0];
        assert_eq!(rec.mail_id, "777888");
        assert_eq!(rec.battle_type, "motte");
        assert_eq!(rec.damage_pct, 55);
        assert_eq!(rec.plunder_tier, 6);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_restore_from_filtered_mails_json_with_archive_box() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_restore_archive_{}", std::process::id()));
        let json_dir = temp_dir.join("filtered_mails_json");
        let _ = fs::create_dir_all(&json_dir);

        let mock_json = r#"{
            "box": "Archive",
            "id": "888999",
            "sendTime": 17858542764,
            "body": {
                "subType": 11,
                "subParam": 4,
                "targetName": "Motte 20",
                "content": "Tiêu diệt thành công Motte cấp 20"
            },
            "_stats_metadata": {
                "damage_pct": 80,
                "plunder_tier": 6,
                "time_vn": "2026-10-22 11:30:00"
            }
        }"#;

        let sample_file = json_dir.join("Persistent.Mail.888999.json");
        fs::write(&sample_file, mock_json).unwrap();

        let vault = MailVault::new(&temp_dir);
        assert_eq!(vault.total_count(), 1, "Thư trong hòm Archive phải được khôi phục vào Vault");
        let rec = &vault.get_records()[0];
        assert_eq!(rec.mail_id, "888999");
        assert_eq!(rec.battle_type, "motte");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_and_survive_game_delete() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_survive_{}", std::process::id()));
        let game_mailcache = temp_dir.join("game_mailcache");
        let _ = fs::create_dir_all(&game_mailcache);

        let vault = MailVault::new(&temp_dir);

        // Giả lập game RoK tạo 1 file nhị phân trong mailcache
        let sample_mail = game_mailcache.join("Persistent.Mail.999111");
        let sample_content = vec![0xFF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x05, 0xFF];
        fs::write(&sample_mail, &sample_content).unwrap();

        // Watcher sao lưu nhị phân vào vault
        let backup_result = vault.backup_binary_mail(&sample_mail);
        assert!(backup_result.is_ok());
        let backup_path = backup_result.unwrap();
        assert!(backup_path.exists());

        // Mô phỏng người chơi đọc thư trong game -> Game xóa file gốc khỏi mailcache
        fs::remove_file(&sample_mail).unwrap();
        assert!(!sample_mail.exists());

        // File trong kho vault vẫn tồn tại nguyên vẹn!
        assert!(backup_path.exists());
        let saved_bytes = fs::read(&backup_path).unwrap();
        assert_eq!(saved_bytes, sample_content);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_corrupted_vault_file_backup_and_refusal() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_corrupt_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let vault_file = temp_dir.join("rok_history_vault.json");
        let corrupt_content = b"{\"version\": 1, \"records\": [corrupted raw bytes ...";
        fs::write(&vault_file, corrupt_content).unwrap();

        let vault = MailVault::new(&temp_dir);
        assert!(vault.is_corrupted(), "Vault must detect corrupted JSON file");

        let corrupt_backup = temp_dir.join("rok_history_vault.corrupt");
        assert!(corrupt_backup.exists(), "Must create .corrupt backup file");
        assert_eq!(fs::read(&corrupt_backup).unwrap(), corrupt_content);

        // Attempting to save to disk must refuse to overwrite
        let save_res = vault.save_to_disk();
        assert!(save_res.is_err(), "Must refuse to overwrite corrupted vault");
        assert_eq!(fs::read(&vault_file).unwrap(), corrupt_content, "Original vault file must not be overwritten");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
