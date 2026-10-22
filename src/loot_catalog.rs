use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub name_vi: String,
    pub name_en: String,
    pub category: String,
    pub sprites: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct YamlItemEntry {
    name: Option<HashMap<String, String>>,
    sprite: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct YamlLootRoot {
    loot: Option<HashMap<u32, HashMap<u32, YamlItemEntry>>>,
}

#[derive(Debug, Clone, Default)]
pub struct LootCatalog {
    entries: HashMap<(u32, u32), ItemInfo>,
}

const EMBEDDED_LOOT_YAML: &str = include_str!("../loot.yaml");

impl LootCatalog {
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load_from_candidates(custom_path: Option<&Path>) -> Self {
        let mut candidates = Vec::new();
        if let Some(p) = custom_path {
            candidates.push(p.to_path_buf());
        }

        if let Ok(env_path) = std::env::var("ROK_LOOT_YAML") {
            candidates.push(PathBuf::from(env_path));
        }

        if let Ok(cur_dir) = std::env::current_dir() {
            candidates.push(cur_dir.join("loot.yaml"));
        }
        candidates.push(PathBuf::from("loot.yaml"));

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("loot.yaml"));
                if let Some(grandparent) = parent.parent() {
                    candidates.push(grandparent.join("loot.yaml"));
                }
            }
        }

        for p in candidates {
            if p.is_file() {
                if let Ok(catalog) = Self::load_file(&p) {
                    println!("[+] Đã tải bảng loot table từ file ngoài: {}", p.display());
                    return catalog;
                }
            }
        }

        // Tự động nạp dữ liệu loot table nhúng sẵn trong file thực thi EXE (hoàn toàn portable độc lập)
        if let Ok(catalog) = Self::load_from_str(EMBEDDED_LOOT_YAML) {
            println!("[+] Đã nạp thành công bảng loot table tích hợp sẵn trong EXE (Embedded)!");
            return catalog;
        }

        eprintln!("[!] Cảnh báo: Không tìm thấy loot.yaml. Sử dụng bảng dịch mặc định.");
        Self::empty()
    }

    pub fn load_from_str(text: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: YamlLootRoot = serde_yaml::from_str(text)?;
        let mut entries = HashMap::new();

        if let Some(type_map) = parsed.loot {
            for (itype, sub_map) in type_map {
                for (isub, item_def) in sub_map {
                    let mut name_vi = None;
                    let mut name_en = None;
                    if let Some(ref names) = item_def.name {
                        name_vi = names.get("vi").cloned();
                        name_en = names.get("en").cloned();
                    }
                    let sprites = item_def.sprite.unwrap_or_default();

                    let cat = Self::determine_category(itype, isub);

                    let name_vi = name_vi.unwrap_or_else(|| {
                        name_en.clone().unwrap_or_else(|| {
                            format!("Vật phẩm (Type {}, SubType {})", itype, isub)
                        })
                    });
                    let name_en = name_en.unwrap_or_else(|| name_vi.clone());

                    entries.insert(
                        (itype, isub),
                        ItemInfo {
                            name_vi,
                            name_en,
                            category: cat.to_string(),
                            sprites,
                        },
                    );
                }
            }
        }

        Ok(Self { entries })
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut f = File::open(path.as_ref())?;
        let mut text = String::new();
        f.read_to_string(&mut text)?;
        Self::load_from_str(&text)
    }

    pub fn determine_category(itype: u32, isub: u32) -> &'static str {
        if itype == 1 {
            if isub == 9 {
                "Pha Lê"
            } else {
                "Tài nguyên"
            }
        } else if itype == 4 {
            "Rương Liên Minh"
        } else if itype == 2 {
            match isub {
                // Tài nguyên: Gói tài nguyên, Đá, Vàng, Gỗ, Ngô, Đá quý
                2..=24 | 26..=32 | 141..=144 | 7003..=7009 | 8002..=8004 => "Tài nguyên",

                // Vật liệu: Da, Quặng sắt, Gỗ mun, Xương động vật, Rương vật liệu
                412..=435 | 7051..=7054 => "Vật liệu",

                // Tăng tốc
                34..=43 | 58..=70 | 71..=79 | 84..=89 | 91..=99 => "Tăng tốc",

                // Sách Kinh Nghiệm (EXP)
                128..=135 | 147..=150 => "Sách Kinh Nghiệm",

                // Điểm Hành Động (AP)
                124..=126 | 182 => "Điểm Hành Động (AP)",

                // Vật phẩm Pháo Đài & Man Rợ đặc thù
                109 => "Sách Giao Ước",
                145 => "Mũi Tên Chống Trả",

                // Tượng Nâng Cấp Chỉ Huy
                9989..=10005 => "Tượng Chỉ Huy",

                // Trang bị
                21000..=22000 => "Trang Bị",

                _ => "Vật phẩm khác",
            }
        } else {
            "Vật phẩm khác"
        }
    }

    pub fn get_item_info(&self, itype: u32, isub: u32, pic: Option<&str>) -> ItemInfo {
        if let Some(entry) = self.entries.get(&(itype, isub)) {
            let mut res = entry.clone();
            // Re-normalize category to ensure user-defined precision
            res.category = Self::determine_category(itype, isub).to_string();
            return res;
        }

        let cat = Self::determine_category(itype, isub);
        let pic_str = pic.unwrap_or("");

        if pic_str == "img_Crystal" || (itype == 1 && isub == 9) {
            ItemInfo {
                name_vi: "Pha Lê".into(),
                name_en: "Crystals".into(),
                category: "Pha Lê".into(),
                sprites: vec!["img_Crystal.png".into()],
            }
        } else if pic_str == "img_icon_item_HeroCarving_Star4" || isub == 9998 {
            ItemInfo {
                name_vi: "Tượng đèn sao mới".into(),
                name_en: "Brand-new Starlight Sculpture".into(),
                category: "Tượng Chỉ Huy".into(),
                sprites: vec!["img_icon_item_HeroCarving_Star4.png".into()],
            }
        } else {
            ItemInfo {
                name_vi: format!("Vật phẩm ({}, {})", itype, isub),
                name_en: format!("Item ({}, {})", itype, isub),
                category: cat.to_string(),
                sprites: Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_loot_catalog() {
        let catalog = LootCatalog::load_from_str(EMBEDDED_LOOT_YAML).expect("Embedded loot.yaml must be valid YAML");
        assert!(catalog.entries.len() > 100);
        // Verify Motte items exist
        let crystal = catalog.get_item_info(1, 9, None);
        assert_eq!(crystal.name_vi, "Pha Lê");
        assert_eq!(crystal.category, "Pha Lê");
    }

    #[test]
    fn test_item_categories() {
        let catalog = LootCatalog::load_from_str(EMBEDDED_LOOT_YAML).expect("Embedded loot.yaml must be valid YAML");

        // Gói tài nguyên, Đá, Vàng, Gỗ, Ngô, Đá quý -> Tài nguyên
        assert_eq!(catalog.get_item_info(1, 1, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(1, 2, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(1, 3, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(1, 4, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(1, 5, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(2, 26, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(2, 7006, None).category, "Tài nguyên");
        assert_eq!(catalog.get_item_info(2, 141, None).category, "Tài nguyên");

        // Da, Quặng sắt, Gỗ mun, Xương -> Vật liệu
        assert_eq!(catalog.get_item_info(2, 412, None).category, "Vật liệu");
        assert_eq!(catalog.get_item_info(2, 419, None).category, "Vật liệu");
        assert_eq!(catalog.get_item_info(2, 413, None).category, "Vật liệu");
        assert_eq!(catalog.get_item_info(2, 420, None).category, "Vật liệu");
        assert_eq!(catalog.get_item_info(2, 417, None).category, "Vật liệu");
        assert_eq!(catalog.get_item_info(2, 414, None).category, "Vật liệu");

        // Tăng tốc -> Tăng tốc
        assert_eq!(catalog.get_item_info(2, 34, None).category, "Tăng tốc");
        assert_eq!(catalog.get_item_info(2, 71, None).category, "Tăng tốc");
        assert_eq!(catalog.get_item_info(2, 92, None).category, "Tăng tốc");
    }
}
