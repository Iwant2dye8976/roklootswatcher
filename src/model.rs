use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub fn format_num<T: std::fmt::Display>(n: T) -> String {
    let s = n.to_string();
    let is_neg = s.starts_with('-');
    let raw = if is_neg {
        s.strip_prefix('-').unwrap_or(&s)
    } else {
        &s
    };
    let parts: Vec<&str> = raw.split('.').collect();
    let int_part = parts[0];
    let mut res = String::new();
    let len = int_part.chars().count();
    for (i, c) in int_part.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            res.push(',');
        }
        res.push(c);
    }
    if parts.len() > 1 {
        res.push('.');
        res.push_str(parts[1]);
    }
    if is_neg {
        format!("-{}", res)
    } else {
        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    #[serde(rename = "all-targets")]
    AllTargets,
    #[serde(rename = "motte")]
    Motte,
    #[serde(rename = "fort")]
    Fort,
    #[serde(rename = "barbarian")]
    Barbarian,
    #[serde(rename = "all-forts")]
    AllForts,
    #[serde(rename = "all-rewards")]
    AllRewards,
    #[serde(rename = "gathering")]
    Gathering,
}

impl FilterMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterMode::AllTargets => "all-targets",
            FilterMode::Motte => "motte",
            FilterMode::Fort => "fort",
            FilterMode::Barbarian => "barbarian",
            FilterMode::AllForts => "all-forts",
            FilterMode::AllRewards => "all-rewards",
            FilterMode::Gathering => "gathering",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            FilterMode::AllTargets => "Tất cả (Motte, Pháo Đài, Dã Man)",
            FilterMode::Motte => "Chỉ Motte (Motte Destroyed)",
            FilterMode::Fort => "Chỉ Pháo Đài (Barbarian Forts)",
            FilterMode::Barbarian => "Chỉ Dã Man (Normal Barbarians)",
            FilterMode::AllForts => "Cả Motte & Pháo Đài",
            FilterMode::AllRewards => "Tất cả thư có quà (All Loot)",
            FilterMode::Gathering => "Chỉ Thu Thập (Gathering)",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "motte" => FilterMode::Motte,
            "fort" => FilterMode::Fort,
            "barbarian" => FilterMode::Barbarian,
            "all-forts" => FilterMode::AllForts,
            "all-rewards" => FilterMode::AllRewards,
            "gathering" | "gather" | "rss" => FilterMode::Gathering,
            _ => FilterMode::AllTargets,
        }
    }

    pub fn matches(&self, battle_type: &str) -> bool {
        match self {
            FilterMode::AllTargets => matches!(battle_type, "motte" | "fort" | "barbarian"),
            FilterMode::Motte => battle_type == "motte",
            FilterMode::Fort => battle_type == "fort",
            FilterMode::Barbarian => battle_type == "barbarian",
            FilterMode::AllForts => matches!(battle_type, "motte" | "fort"),
            FilterMode::AllRewards => true,
            FilterMode::Gathering => battle_type == "gathering",
        }
    }
}

pub fn is_zero(v: &i64) -> bool {
    *v == 0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFilterPreset {
    AllTime,
    Today,
    Yesterday,
    Last3Days,
    Last7Days,
    Last14Days,
    Last30Days,
    Custom,
}

impl DateFilterPreset {
    pub fn label(&self) -> &'static str {
        match self {
            DateFilterPreset::AllTime => "Từ trước đến nay",
            DateFilterPreset::Today => "Hôm nay",
            DateFilterPreset::Yesterday => "Hôm qua",
            DateFilterPreset::Last3Days => "3 ngày gần nhất",
            DateFilterPreset::Last7Days => "7 ngày qua",
            DateFilterPreset::Last14Days => "14 ngày qua",
            DateFilterPreset::Last30Days => "30 ngày qua",
            DateFilterPreset::Custom => "Tùy chọn ngày",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpeedupUnit {
    #[default]
    #[serde(rename = "minutes")]
    Minutes,
    #[serde(rename = "hours")]
    Hours,
    #[serde(rename = "days")]
    Days,
    #[serde(rename = "dhm")]
    Dhm,
}

/// Chuyển đổi số phút tăng tốc thành chuỗi ngày, giờ, phút đầy đủ (vd: "2 ngày 5 giờ 30 phút").
pub fn format_speedup_dhm(minutes: i64) -> String {
    if minutes <= 0 {
        return "0 phút".to_string();
    }
    let days = minutes / 1440;
    let rem = minutes % 1440;
    let hours = rem / 60;
    let mins = rem % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} ngày", days));
    }
    if hours > 0 {
        parts.push(format!("{} giờ", hours));
    }
    if mins > 0 || parts.is_empty() {
        parts.push(format!("{} phút", mins));
    }
    parts.join(" ")
}

/// Chuyển đổi số phút tăng tốc thành chuỗi viết tắt (vd: "2d 5h 30p").
pub fn format_speedup_dhm_short(minutes: i64) -> String {
    if minutes <= 0 {
        return "0p".to_string();
    }
    let days = minutes / 1440;
    let rem = minutes % 1440;
    let hours = rem / 60;
    let mins = rem % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if mins > 0 || parts.is_empty() {
        parts.push(format!("{}p", mins));
    }
    parts.join(" ")
}

#[allow(dead_code)]
impl SpeedupUnit {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "h" | "hour" | "hours" | "gio" | "giờ" => SpeedupUnit::Hours,
            "d" | "day" | "days" | "ngay" | "ngày" => SpeedupUnit::Days,
            "dhm" | "ngaygio" | "ngày giờ" | "ngày giờ phút" => SpeedupUnit::Dhm,
            _ => SpeedupUnit::Minutes,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SpeedupUnit::Minutes => "Phút (Minutes)",
            SpeedupUnit::Hours => "Giờ (Hours)",
            SpeedupUnit::Days => "Ngày (Days)",
            SpeedupUnit::Dhm => "Ngày / Giờ / Phút (DHM)",
        }
    }

    pub fn short_suffix(&self) -> &'static str {
        match self {
            SpeedupUnit::Minutes => "p",
            SpeedupUnit::Hours => "h",
            SpeedupUnit::Days => "d",
            SpeedupUnit::Dhm => "",
        }
    }

    pub fn unit_name(&self) -> &'static str {
        match self {
            SpeedupUnit::Minutes => "phút",
            SpeedupUnit::Hours => "giờ",
            SpeedupUnit::Days => "ngày",
            SpeedupUnit::Dhm => "",
        }
    }

    pub fn format_duration(&self, minutes: f64) -> String {
        match self {
            SpeedupUnit::Minutes => {
                format!("{}{}", format_num(minutes.round() as i64), self.short_suffix())
            }
            SpeedupUnit::Hours => {
                let hours = minutes / 60.0;
                if (hours - hours.round()).abs() < 0.05 {
                    format!("{:.0}{}", hours, self.short_suffix())
                } else {
                    format!("{:.1}{}", hours, self.short_suffix())
                }
            }
            SpeedupUnit::Days => {
                let days = minutes / 1440.0;
                if (days - days.round()).abs() < 0.01 {
                    format!("{:.1}{}", days, self.short_suffix())
                } else {
                    format!("{:.2}{}", days, self.short_suffix())
                }
            }
            SpeedupUnit::Dhm => {
                format_speedup_dhm_short(minutes.round() as i64)
            }
        }
    }

    pub fn format_kpi(&self, minutes: i64) -> (String, String) {
        match self {
            SpeedupUnit::Minutes => (
                format_num(minutes),
                "phút".to_string(),
            ),
            SpeedupUnit::Hours => {
                let hours = minutes as f64 / 60.0;
                let val_str = if (hours - hours.round()).abs() < 0.05 {
                    format!("{:.0}", hours)
                } else {
                    format!("{:.1}", hours)
                };
                (val_str, "giờ".to_string())
            }
            SpeedupUnit::Days => {
                let days = minutes as f64 / 1440.0;
                let val_str = if (days - days.round()).abs() < 0.01 {
                    format!("{:.1}", days)
                } else {
                    format!("{:.2}", days)
                };
                (val_str, "ngày".to_string())
            }
            SpeedupUnit::Dhm => (
                format_speedup_dhm(minutes),
                "".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDrop {
    #[serde(rename = "type")]
    pub itype: u32,
    #[serde(rename = "subType")]
    pub sub_type: u32,
    pub value: i64,
    pub pic: String,
    pub name_vi: String,
    pub name_en: String,
    pub category: String,
    pub sprites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecord {
    pub filename: String,
    pub mail_id: String,
    pub battle_type: String,
    pub time_raw: i64,
    pub time_utc: String,
    pub time_vn: String,
    pub target: String,
    #[serde(alias = "motte_level", alias = "node_level", default)]
    pub target_level: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub talent_add: i64,
    pub pos_x: i32,
    pub pos_y: i32,
    pub damage_pct: i32,
    pub plunder_tier: i32,
    pub items: Vec<ItemDrop>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub item_totals: HashMap<String, i64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
}

impl BattleRecord {
    pub fn crystal_quantity(&self) -> i64 {
        let total: i64 = self
            .items
            .iter()
            .filter(|it| (it.itype == 1 && it.sub_type == 9) || it.name_vi.contains("Pha Lê") || it.name_en.contains("Crystal"))
            .map(|it| it.value)
            .sum();

        if total > 0 {
            total
        } else {
            self.item_totals.get("Pha Lê").copied().unwrap_or(0)
        }
    }

    pub fn item_quantity(&self, itype: u32, sub_type: u32) -> i64 {
        let sum: i64 = self
            .items
            .iter()
            .filter(|it| it.itype == itype && it.sub_type == sub_type)
            .map(|it| it.value)
            .sum();

        if sum > 0 {
            sum
        } else {
            // Fallback for custom or synthesized item entries in item_totals
            match (itype, sub_type) {
                (1, 9) => self.crystal_quantity(),
                (2, 7006) => self.item_totals.get("Gói Tài Nguyên Cấp 2").copied().unwrap_or(0),
                (2, 130) => self.item_totals.get("Sách Kiến Thức Cấp 3").copied().unwrap_or(0),
                (2, 92) => self.item_totals.get("Tăng tốc chữa lành 5 phút").copied().unwrap_or(0),
                (2, 58) => self.item_totals.get("Tăng tốc 5 phút").copied().unwrap_or(0),
                (2, 9998) => self.item_totals.get("Tượng đèn sao mới").copied().unwrap_or(0),
                (2, 182) => self.item_totals.get("Khôi phục nhanh điểm hành động").copied().unwrap_or(0),
                (2, 124) => self.item_totals.get("Khôi phục điểm hành động cơ bản").copied().unwrap_or(0),
                _ => 0,
            }
        }
    }

    #[allow(dead_code)]
    pub fn compute_item_totals(&self) -> HashMap<String, i64> {
        let mut totals = HashMap::new();
        for it in &self.items {
            *totals.entry(it.name_vi.clone()).or_insert(0) += it.value;
        }
        totals
    }

    pub fn univ_speedup_minutes(&self) -> i64 {
        if !self.items.is_empty() {
            let mut total = 0;
            for it in &self.items {
                if let ConvertedValue::Speedup { category: "Đa Năng", minutes } = get_item_conversion(it.itype, it.sub_type) {
                    total += it.value * minutes;
                }
            }
            total
        } else {
            self.item_quantity(2, 34)
                + self.item_quantity(2, 58) * 5
                + self.item_quantity(2, 35) * 15
                + self.item_quantity(2, 60) * 30
                + self.item_quantity(2, 36) * 60
                + self.item_quantity(2, 37) * 180
                + self.item_quantity(2, 38) * 480
                + self.item_quantity(2, 39) * 900
                + self.item_quantity(2, 40) * 1440
                + self.item_quantity(2, 41) * 4320
        }
    }

    pub fn heal_speedup_minutes(&self) -> i64 {
        if !self.items.is_empty() {
            let mut total = 0;
            for it in &self.items {
                if let ConvertedValue::Speedup { category: "Chữa Lành", minutes } = get_item_conversion(it.itype, it.sub_type) {
                    total += it.value * minutes;
                }
            }
            total
        } else {
            self.item_quantity(2, 91)
                + self.item_quantity(2, 92) * 5
                + self.item_quantity(2, 94) * 15
                + self.item_quantity(2, 96) * 60
                + self.item_quantity(2, 97) * 180
                + self.item_quantity(2, 98) * 480
                + self.item_quantity(2, 99) * 900
        }
    }

    pub fn training_speedup_minutes(&self) -> i64 {
        if !self.items.is_empty() {
            let mut total = 0;
            for it in &self.items {
                if let ConvertedValue::Speedup { category: "Huấn Luyện", minutes } = get_item_conversion(it.itype, it.sub_type) {
                    total += it.value * minutes;
                }
            }
            total
        } else {
            self.item_quantity(2, 71)
                + self.item_quantity(2, 72) * 5
                + self.item_quantity(2, 74) * 15
                + self.item_quantity(2, 76) * 60
                + self.item_quantity(2, 77) * 180
                + self.item_quantity(2, 78) * 480
                + self.item_quantity(2, 79) * 900
        }
    }

    #[allow(dead_code)]
    pub fn total_speedup_minutes(&self) -> i64 {
        self.univ_speedup_minutes() + self.heal_speedup_minutes() + self.training_speedup_minutes()
    }

    #[allow(dead_code)]
    pub fn format_total_speedup_dhm(&self) -> String {
        format_speedup_dhm(self.total_speedup_minutes())
    }

    pub fn total_resources_breakdown(&self) -> ResourceTotals {
        let mut totals = ResourceTotals::default();
        for it in &self.items {
            if let ConvertedValue::Resource { kind, multiplier } = get_item_conversion(it.itype, it.sub_type) {
                let val = it.value * multiplier;
                match kind {
                    ResourceKind::Food => {
                        totals.food_val += val;
                        totals.food_packs += it.value;
                    }
                    ResourceKind::Wood => {
                        totals.wood_val += val;
                        totals.wood_packs += it.value;
                    }
                    ResourceKind::Stone => {
                        totals.stone_val += val;
                        totals.stone_packs += it.value;
                    }
                    ResourceKind::Gold => {
                        totals.gold_val += val;
                        totals.gold_packs += it.value;
                    }
                    ResourceKind::ChoicePack => {
                        totals.choice_val += val;
                        totals.choice_packs += it.value;
                    }
                }
            }
        }
        totals
    }

    pub fn total_resource_value(&self) -> i64 {
        self.total_resources_breakdown().total_rss_val()
    }

    pub fn total_exp_value(&self) -> i64 {
        let mut total = 0;
        for it in &self.items {
            if let ConvertedValue::Exp(mult) = get_item_conversion(it.itype, it.sub_type) {
                total += it.value * mult;
            }
        }
        total
    }

    pub fn materials_breakdown(&self) -> MaterialTotals {
        let mut totals = MaterialTotals::default();
        for it in &self.items {
            if it.itype == 2 {
                match it.sub_type {
                    412 => totals.leather_normal += it.value,
                    419 => totals.leather_advanced += it.value,
                    413 => totals.iron_normal += it.value,
                    420 => totals.iron_advanced += it.value,
                    417 => totals.ebony_normal += it.value,
                    424 => totals.ebony_advanced += it.value,
                    414 => totals.bone_normal += it.value,
                    421 => totals.bone_advanced += it.value,
                    428 => totals.bone_elite += it.value,
                    435 => totals.bone_epic += it.value,
                    _ => {}
                }
            }
        }
        totals
    }

    pub fn ap_breakdown(&self) -> ApTotals {
        let mut totals = ApTotals::default();
        for it in &self.items {
            if it.itype == 2 {
                match it.sub_type {
                    182 => totals.ap_50 += it.value,
                    124 => totals.ap_100 += it.value,
                    125 => totals.ap_500 += it.value,
                    126 => totals.ap_1000 += it.value,
                    _ => {}
                }
            }
        }
        totals
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Food,
    Wood,
    Stone,
    Gold,
    ChoicePack,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceTotals {
    pub food_val: i64,
    pub food_packs: i64,
    pub wood_val: i64,
    pub wood_packs: i64,
    pub stone_val: i64,
    pub stone_packs: i64,
    pub gold_val: i64,
    pub gold_packs: i64,
    pub choice_val: i64,
    pub choice_packs: i64,
}

impl ResourceTotals {
    pub fn total_rss_val(&self) -> i64 {
        // Gói tài nguyên C không có giá trị cụ thể, chỉ tính số lượng gói.
        // Tổng tài nguyên chỉ quy đổi các loại tài nguyên cụ thể (Ngô, Gỗ, Đá, Vàng).
        self.food_val + self.wood_val + self.stone_val + self.gold_val
    }

    #[allow(dead_code)]
    pub fn total_base_rss_val(&self) -> i64 {
        self.food_val + self.wood_val + self.stone_val + self.gold_val
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialKind {
    Leather,     // Da
    IronOre,     // Quặng sắt
    Ebony,       // Gỗ mun
    AnimalBone,  // Xương động vật
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialTotals {
    pub leather_normal: i64,   // (2, 412) Da BT
    pub leather_advanced: i64, // (2, 419) Da NC
    pub iron_normal: i64,      // (2, 413) Sắt BT
    pub iron_advanced: i64,    // (2, 420) Sắt NC
    pub ebony_normal: i64,     // (2, 417) Gỗ BT
    pub ebony_advanced: i64,   // (2, 424) Gỗ NC
    pub bone_normal: i64,      // (2, 414) Xương BT
    pub bone_advanced: i64,    // (2, 421) Xương NC
    pub bone_elite: i64,       // (2, 428) Xương TN
    pub bone_epic: i64,        // (2, 435) Xương ST
}

impl MaterialTotals {
    pub fn total_normal(&self) -> i64 {
        self.leather_normal + self.iron_normal + self.ebony_normal + self.bone_normal
    }

    pub fn total_advanced(&self) -> i64 {
        self.leather_advanced + self.iron_advanced + self.ebony_advanced + self.bone_advanced
    }

    pub fn total_elite(&self) -> i64 {
        self.bone_elite
    }

    pub fn total_epic(&self) -> i64 {
        self.bone_epic
    }

    pub fn leather_total_converted(&self) -> i64 {
        self.leather_normal + self.leather_advanced * 4
    }
    pub fn iron_total_converted(&self) -> i64 {
        self.iron_normal + self.iron_advanced * 4
    }
    pub fn ebony_total_converted(&self) -> i64 {
        self.ebony_normal + self.ebony_advanced * 4
    }
    pub fn bone_total_converted(&self) -> i64 {
        self.bone_normal + self.bone_advanced * 4 + self.bone_elite * 16 + self.bone_epic * 64
    }
    pub fn total_pieces(&self) -> i64 {
        self.total_normal() + self.total_advanced() + self.total_elite() + self.total_epic()
    }
    pub fn total_converted_normal(&self) -> i64 {
        self.leather_total_converted()
            + self.iron_total_converted()
            + self.ebony_total_converted()
            + self.bone_total_converted()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApTotals {
    pub ap_50: i64,   // (2, 182) Khôi phục nhanh 50 AP
    pub ap_100: i64,  // (2, 124) Khôi phục cơ bản 100 AP
    pub ap_500: i64,  // (2, 125) Khôi phục trung bình 500 AP
    pub ap_1000: i64, // (2, 126) Khôi phục cao cấp 1000 AP
}

impl ApTotals {
    pub fn total_bottles(&self) -> i64 {
        self.ap_50 + self.ap_100 + self.ap_500 + self.ap_1000
    }
    pub fn total_ap_points(&self) -> i64 {
        self.ap_50 * 50 + self.ap_100 * 100 + self.ap_500 * 500 + self.ap_1000 * 1000
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertedValue {
    Resource { kind: ResourceKind, multiplier: i64 },
    Exp(i64),
    Speedup { category: &'static str, minutes: i64 },
    Gem(i64),
    Crystal,
    ArrowOfResistance,
    BookOfCovenant,
    Starlight,
    ApRecovery(i64),
    EquipmentMaterial { kind: MaterialKind, tier_mult: i64 },
    AllianceChest,
    Other,
}

pub fn get_item_conversion(itype: u32, isub: u32) -> ConvertedValue {
    if itype == 1 {
        match isub {
            1 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 1 },
            2 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 1 },
            3 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 1 },
            4 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 1 },
            9 => ConvertedValue::Crystal,
            _ => ConvertedValue::Other,
        }
    } else if itype == 4 {
        ConvertedValue::AllianceChest
    } else if itype == 2 {
        match isub {
            // Thực phẩm (Food)
            141 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 1_000 },
            2 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 50_000 },
            3 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 150_000 },
            4 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 500_000 },
            5 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 1_500_000 },
            6 => ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 5_000_000 },

            // Gỗ (Wood)
            142 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 1_000 },
            8 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 50_000 },
            9 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 150_000 },
            10 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 500_000 },
            11 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 1_500_000 },
            12 => ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 5_000_000 },

            // Đá (Stone)
            143 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 750 },
            14 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 37_500 },
            15 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 112_500 },
            16 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 375_000 },
            17 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 1_125_000 },
            18 => ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 3_750_000 },

            // Vàng (Gold)
            144 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 500 },
            21 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 50_000 },
            22 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 200_000 },
            23 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 600_000 },
            24 => ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 2_000_000 },

            // Gói Tài Nguyên Tùy Chọn (Choice Packs)
            // Không có giá trị cụ thể quy đổi ra RSS cố định, chỉ cần số lượng gói
            7003..=7009 => ConvertedValue::Resource { kind: ResourceKind::ChoicePack, multiplier: 0 },

            // Sách EXP Tướng: 1 sách kinh nghiệm cấp 3 = 1000 kinh nghiệm
            128 => ConvertedValue::Exp(100),
            129 => ConvertedValue::Exp(500),
            130 => ConvertedValue::Exp(1_000),
            147 => ConvertedValue::Exp(10_000),
            148 => ConvertedValue::Exp(50_000),
            149 => ConvertedValue::Exp(200_000),
            150 => ConvertedValue::Exp(500_000),

            // Tăng tốc Huấn luyện
            71 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 1 },
            72 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 5 },
            74 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 15 },
            76 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 60 },
            77 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 180 },
            78 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 480 },
            79 => ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 900 },

            // Tăng tốc Chữa lành
            91 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 1 },
            92 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 5 },
            94 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 15 },
            96 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 60 },
            97 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 180 },
            98 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 480 },
            99 => ConvertedValue::Speedup { category: "Chữa Lành", minutes: 900 },

            // Tăng tốc Đa năng
            34 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 1 },
            58 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 5 },
            35 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 15 },
            60 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 30 },
            36 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 60 },
            37 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 180 },
            38 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 480 },
            39 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 900 },
            40 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 1440 },
            41 => ConvertedValue::Speedup { category: "Đa Năng", minutes: 4320 },

            // Tăng tốc Xây dựng & Nghiên cứu
            65 => ConvertedValue::Speedup { category: "Xây Dựng", minutes: 30 },
            84 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 15 },
            85 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 30 },
            86 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 60 },
            87 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 180 },
            88 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 480 },
            89 => ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 900 },

            // Đá quý
            26 => ConvertedValue::Gem(10),
            27 => ConvertedValue::Gem(50),
            28 => ConvertedValue::Gem(100),
            29 => ConvertedValue::Gem(200),
            30 => ConvertedValue::Gem(500),
            31 => ConvertedValue::Gem(1000),
            32 => ConvertedValue::Gem(2000),

            // Vật phẩm pháo đài & tháp canh
            145 => ConvertedValue::ArrowOfResistance,
            109 => ConvertedValue::BookOfCovenant,

            // Đèn sao
            9996..=10004 => ConvertedValue::Starlight,

            // Điểm hành động AP
            182 => ConvertedValue::ApRecovery(50),
            124 => ConvertedValue::ApRecovery(100),
            125 => ConvertedValue::ApRecovery(500),
            126 => ConvertedValue::ApRecovery(1000),

            // Nguyên liệu trang bị (Vật liệu: da, sắt, gỗ, xương động vật)
            412 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::Leather, tier_mult: 1 },
            419 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::Leather, tier_mult: 4 },
            413 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::IronOre, tier_mult: 1 },
            420 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::IronOre, tier_mult: 4 },
            417 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::Ebony, tier_mult: 1 },
            424 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::Ebony, tier_mult: 4 },
            414 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::AnimalBone, tier_mult: 1 },
            421 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::AnimalBone, tier_mult: 4 },
            428 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::AnimalBone, tier_mult: 16 },
            435 => ConvertedValue::EquipmentMaterial { kind: MaterialKind::AnimalBone, tier_mult: 64 },

            _ => ConvertedValue::Other,
        }
    } else {
        ConvertedValue::Other
    }
}

pub fn get_item_converted_display(itype: u32, isub: u32, total_qty: i64) -> Option<String> {
    match get_item_conversion(itype, isub) {
        ConvertedValue::Resource { kind, multiplier } => {
            if kind == ResourceKind::ChoicePack {
                Some(format!("{} gói", format_num(total_qty)))
            } else {
                let total = total_qty * multiplier;
                let millions = (total as f64) / 1_000_000.0;
                let kind_label = match kind {
                    ResourceKind::Food => "Ngô",
                    ResourceKind::Wood => "Gỗ",
                    ResourceKind::Stone => "Đá",
                    ResourceKind::Gold => "Vàng",
                    ResourceKind::ChoicePack => "gói",
                };
                if millions >= 0.01 {
                    Some(format!("{:.2}M {}", millions, kind_label))
                } else {
                    Some(format!("{} {}", format_num(total), kind_label))
                }
            }
        }
        ConvertedValue::Exp(multiplier) => {
            let total = total_qty * multiplier;
            let millions = (total as f64) / 1_000_000.0;
            if millions >= 0.1 {
                Some(format!("{:.2}M EXP", millions))
            } else {
                Some(format!("{} EXP", format_num(total)))
            }
        }
        ConvertedValue::Speedup { minutes, .. } => {
            let total_m = total_qty * minutes;
            let hours = total_m as f64 / 60.0;
            if hours >= 1.0 {
                Some(format!("{:.1}h ({})", hours, format_num(total_m)))
            } else {
                Some(format!("{}p", format_num(total_m)))
            }
        }
        ConvertedValue::Gem(multiplier) => {
            let total = total_qty * multiplier;
            Some(format!("{} Gems", format_num(total)))
        }
        ConvertedValue::Crystal => {
            Some(format!("{} Pha Lê", format_num(total_qty)))
        }
        ConvertedValue::ArrowOfResistance => Some(format!("{} Mũi tên", format_num(total_qty))),
        ConvertedValue::BookOfCovenant => Some(format!("{} Sách ước", format_num(total_qty))),
        ConvertedValue::Starlight => Some(format!("{} Tượng", format_num(total_qty))),
        ConvertedValue::ApRecovery(points) => {
            Some(format!("{} bình ({} AP)", format_num(total_qty), format_num(total_qty * points)))
        }
        ConvertedValue::EquipmentMaterial { kind, tier_mult } => {
            let label = match kind {
                MaterialKind::Leather => "Da",
                MaterialKind::IronOre => "Sắt",
                MaterialKind::Ebony => "Gỗ mun",
                MaterialKind::AnimalBone => "Xương",
            };
            if tier_mult > 1 {
                Some(format!("{} {} (={})", format_num(total_qty), label, format_num(total_qty * tier_mult)))
            } else {
                Some(format!("{} {}", format_num(total_qty), label))
            }
        }
        ConvertedValue::AllianceChest => Some(format!("{} Rương LM", format_num(total_qty))),
        ConvertedValue::Other => None,
    }
}

pub fn format_in_millions_vietnamese(val: i64) -> String {
    if val >= 1_000_000_000 {
        format!("{:.2} tỷ", (val as f64) / 1_000_000_000.0)
    } else if val >= 1_000_000 {
        format!("{:.2}M", (val as f64) / 1_000_000.0)
    } else if val >= 1_000 {
        format!("{:.1}k", (val as f64) / 1_000.0)
    } else {
        val.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStat {
    #[serde(rename = "type")]
    pub itype: u32,
    #[serde(rename = "subType")]
    pub sub_type: u32,
    pub name_vi: String,
    pub name_en: String,
    pub category: String,
    pub sprites: Vec<String>,
    pub pic: String,
    pub total_quantity: i64,
    pub battle_count: usize,
    pub max_in_battle: i64,
    pub min_in_battle: i64,
    pub avg_per_battle: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlunderTierStat {
    pub tier: i32,
    pub count: usize,
    pub min_dmg: i32,
    pub max_dmg: i32,
    pub avg_dmg: f64,
    pub avg_crystal: i64,
    pub avg_res_packs: i64,
    pub avg_tomes: i64,
    pub avg_heal_spd: i64,
    pub avg_univ_spd: i64,
    pub starlight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Statistics {
    pub total_battles: usize,
    pub avg_damage_pct: f64,
    pub min_damage_pct: i32,
    pub max_damage_pct: i32,
    pub item_stats: HashMap<String, ItemStat>,
    pub plunder_breakdown: HashMap<i32, PlunderTierStat>,
}

impl Statistics {
    pub fn compute_resources(&self) -> ResourceTotals {
        let mut totals = ResourceTotals::default();
        for it in self.item_stats.values() {
            if let ConvertedValue::Resource { kind, multiplier } = get_item_conversion(it.itype, it.sub_type) {
                let val = it.total_quantity * multiplier;
                match kind {
                    ResourceKind::Food => {
                        totals.food_val += val;
                        totals.food_packs += it.total_quantity;
                    }
                    ResourceKind::Wood => {
                        totals.wood_val += val;
                        totals.wood_packs += it.total_quantity;
                    }
                    ResourceKind::Stone => {
                        totals.stone_val += val;
                        totals.stone_packs += it.total_quantity;
                    }
                    ResourceKind::Gold => {
                        totals.gold_val += val;
                        totals.gold_packs += it.total_quantity;
                    }
                    ResourceKind::ChoicePack => {
                        totals.choice_val += val;
                        totals.choice_packs += it.total_quantity;
                    }
                }
            }
        }
        totals
    }

    pub fn compute_total_exp(&self) -> i64 {
        let mut total = 0;
        for it in self.item_stats.values() {
            if let ConvertedValue::Exp(mult) = get_item_conversion(it.itype, it.sub_type) {
                total += it.total_quantity * mult;
            }
        }
        total
    }

    pub fn compute_speedups_by_category(&self) -> (i64, i64, i64) {
        let mut training = 0;
        let mut healing = 0;
        let mut universal = 0;
        for it in self.item_stats.values() {
            if let ConvertedValue::Speedup { category, minutes } = get_item_conversion(it.itype, it.sub_type) {
                let m = it.total_quantity * minutes;
                match category {
                    "Huấn Luyện" => training += m,
                    "Chữa Lành" => healing += m,
                    "Đa Năng" => universal += m,
                    _ => universal += m,
                }
            }
        }
        (training, healing, universal)
    }

    #[allow(dead_code)]
    pub fn compute_speedup_total_minutes(&self) -> i64 {
        let (train, heal, univ) = self.compute_speedups_by_category();
        train + heal + univ
    }

    #[allow(dead_code)]
    pub fn format_total_speedup_dhm(&self) -> String {
        format_speedup_dhm(self.compute_speedup_total_minutes())
    }

    pub fn compute_crystals(&self) -> i64 {
        let mut total = 0;
        for it in self.item_stats.values() {
            if (it.itype == 1 && it.sub_type == 9) || it.name_vi.contains("Pha Lê") || it.name_en.contains("Crystal") {
                total += it.total_quantity;
            }
        }
        total
    }

    pub fn compute_gems(&self) -> i64 {
        let mut total = 0;
        for it in self.item_stats.values() {
            if let ConvertedValue::Gem(mult) = get_item_conversion(it.itype, it.sub_type) {
                total += it.total_quantity * mult;
            }
        }
        total
    }

    pub fn compute_arrows_of_resistance(&self) -> i64 {
        self.item_stats.get("2_145").map(|it| it.total_quantity).unwrap_or(0)
    }

    pub fn compute_books_of_covenant(&self) -> i64 {
        self.item_stats.get("2_109").map(|it| it.total_quantity).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn compute_starlight_sculptures(&self) -> i64 {
        let mut total = 0;
        for it in self.item_stats.values() {
            if let ConvertedValue::Starlight = get_item_conversion(it.itype, it.sub_type) {
                total += it.total_quantity;
            }
        }
        total
    }

    pub fn compute_materials(&self) -> MaterialTotals {
        let mut totals = MaterialTotals::default();
        for it in self.item_stats.values() {
            if it.itype == 2 {
                match it.sub_type {
                    412 => totals.leather_normal += it.total_quantity,
                    419 => totals.leather_advanced += it.total_quantity,
                    413 => totals.iron_normal += it.total_quantity,
                    420 => totals.iron_advanced += it.total_quantity,
                    417 => totals.ebony_normal += it.total_quantity,
                    424 => totals.ebony_advanced += it.total_quantity,
                    414 => totals.bone_normal += it.total_quantity,
                    421 => totals.bone_advanced += it.total_quantity,
                    428 => totals.bone_elite += it.total_quantity,
                    435 => totals.bone_epic += it.total_quantity,
                    _ => {}
                }
            }
        }
        totals
    }

    #[allow(dead_code)]
    pub fn compute_equipment_materials(&self) -> i64 {
        self.compute_materials().total_pieces()
    }

    pub fn compute_ap(&self) -> ApTotals {
        let mut totals = ApTotals::default();
        for it in self.item_stats.values() {
            if it.itype == 2 {
                match it.sub_type {
                    182 => totals.ap_50 += it.total_quantity,
                    124 => totals.ap_100 += it.total_quantity,
                    125 => totals.ap_500 += it.total_quantity,
                    126 => totals.ap_1000 += it.total_quantity,
                    _ => {}
                }
            }
        }
        totals
    }

    #[allow(dead_code)]
    pub fn compute_ap_points(&self) -> i64 {
        self.compute_ap().total_ap_points()
    }

    #[allow(dead_code)]
    pub fn compute_alliance_chests(&self) -> i64 {
        let mut total = 0;
        for it in self.item_stats.values() {
            if it.itype == 4 {
                total += it.total_quantity;
            }
        }
        total
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    #[default]
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

impl AppTheme {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            AppTheme::Dark => "Giao diện: Tối",
            AppTheme::Light => "Giao diện: Sáng",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
        }
    }

    #[allow(dead_code)]
    pub fn is_dark(&self) -> bool {
        matches!(self, AppTheme::Dark)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CloseBehavior {
    #[serde(rename = "tray")]
    #[default]
    Tray,
    #[serde(rename = "close")]
    Close,
}

#[allow(dead_code)]
impl CloseBehavior {
    pub fn label(&self) -> &'static str {
        match self {
            CloseBehavior::Tray => "Thu nhỏ vào Khay (Tray icon)",
            CloseBehavior::Close => "Thoát ứng dụng (Close)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CloseBehavior::Tray => "Bấm X sẽ ẩn vào khay hệ thống, ứng dụng vẫn tiếp tục theo dõi thư",
            CloseBehavior::Close => "Bấm X sẽ dừng theo dõi và tắt ứng dụng hoàn toàn",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub close_behavior: CloseBehavior,
    #[serde(default)]
    pub speedup_unit: SpeedupUnit,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default)]
    pub mail_dir: Option<PathBuf>,
    #[serde(default)]
    pub mode: Option<FilterMode>,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_autostart_minimized")]
    pub autostart_minimized: bool,
}

fn default_autostart_minimized() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            close_behavior: CloseBehavior::Tray,
            speedup_unit: SpeedupUnit::Minutes,
            theme: AppTheme::Dark,
            mail_dir: None,
            mode: None,
            autostart: false,
            autostart_minimized: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let cur = PathBuf::from("rok_config.json");
        if cur.exists() {
            return cur;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("rok_config.json");
                if candidate.exists() {
                    return candidate;
                }
                if let Ok(m) = std::fs::metadata(parent) {
                    if !m.permissions().readonly() {
                        return candidate;
                    }
                }
            }
        }
        cur
    }

    pub fn load() -> Self {
        let p = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let p = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&p, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_num() {
        assert_eq!(format_num(0), "0");
        assert_eq!(format_num(100), "100");
        assert_eq!(format_num(1000), "1,000");
        assert_eq!(format_num(1234567), "1,234,567");
        assert_eq!(format_num(-1234567), "-1,234,567");
        assert_eq!(format_num(-50), "-50");
        assert_eq!(format_num(1234.56), "1,234.56");
    }

    #[test]
    fn test_battle_record_item_quantity() {
        let rec = BattleRecord {
            filename: "test.mail".into(),
            mail_id: "123".into(),
            battle_type: "motte".into(),
            time_raw: 100,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Motte 12".into(),
            target_level: "12".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 50,
            plunder_tier: 6,
            items: vec![
                ItemDrop {
                    itype: 1,
                    sub_type: 9,
                    value: 500,
                    pic: "".into(),
                    name_vi: "Pha Lê".into(),
                    name_en: "Crystals".into(),
                    category: "".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 1,
                    sub_type: 9,
                    value: 250,
                    pic: "".into(),
                    name_vi: "Pha Lê".into(),
                    name_en: "Crystals".into(),
                    category: "".into(),
                    sprites: vec![],
                },
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        assert_eq!(rec.item_quantity(1, 9), 750);
        assert_eq!(rec.item_quantity(2, 7006), 0);
    }

    #[test]
    fn test_speedup_unit_formatting() {
        let mins = SpeedupUnit::Minutes;
        let hrs = SpeedupUnit::Hours;
        let days = SpeedupUnit::Days;

        assert_eq!(mins.format_duration(120.0), "120p");
        assert_eq!(hrs.format_duration(120.0), "2h");
        assert_eq!(hrs.format_duration(90.0), "1.5h");
        assert_eq!(days.format_duration(1440.0), "1.0d");
        assert_eq!(days.format_duration(2160.0), "1.50d");

        let (val_m, unit_m) = mins.format_kpi(720);
        assert_eq!(val_m, "720");
        assert_eq!(unit_m, "phút");

        let (val_h, unit_h) = hrs.format_kpi(720);
        assert_eq!(val_h, "12");
        assert_eq!(unit_h, "giờ");

        let (val_d, unit_d) = days.format_kpi(2880);
        assert_eq!(val_d, "2.0");
        assert_eq!(unit_d, "ngày");

        let dhm = SpeedupUnit::Dhm;
        assert_eq!(dhm.format_duration(120.0), "2h");
        assert_eq!(dhm.format_duration(150.0), "2h 30p");
        assert_eq!(dhm.format_duration(1440.0), "1d");
        assert_eq!(dhm.format_duration(1500.0), "1d 1h");
        assert_eq!(dhm.format_duration(2910.0), "2d 30p");

        let (val_dhm, unit_dhm) = dhm.format_kpi(1500);
        assert_eq!(val_dhm, "1 ngày 1 giờ");
        assert_eq!(unit_dhm, "");

        assert_eq!(format_speedup_dhm(0), "0 phút");
        assert_eq!(format_speedup_dhm(45), "45 phút");
        assert_eq!(format_speedup_dhm(125), "2 giờ 5 phút");
        assert_eq!(format_speedup_dhm(1500), "1 ngày 1 giờ");
        assert_eq!(format_speedup_dhm(2910), "2 ngày 30 phút");

        assert_eq!(SpeedupUnit::from_str_loose("hours"), SpeedupUnit::Hours);
        assert_eq!(SpeedupUnit::from_str_loose("d"), SpeedupUnit::Days);
        assert_eq!(SpeedupUnit::from_str_loose("minutes"), SpeedupUnit::Minutes);
        assert_eq!(SpeedupUnit::from_str_loose("dhm"), SpeedupUnit::Dhm);
        assert_eq!(SpeedupUnit::from_str_loose("ngày giờ"), SpeedupUnit::Dhm);
    }

    #[test]
    fn test_close_behavior_and_config() {
        assert_eq!(CloseBehavior::default(), CloseBehavior::Tray);

        let cfg_json = r#"{"close_behavior":"close","speedup_unit":"hours","mail_dir":"C:\\test\\mail","mode":"all-targets","autostart":true,"autostart_minimized":true}"#;
        let cfg: AppConfig = serde_json::from_str(cfg_json).unwrap();
        assert_eq!(cfg.close_behavior, CloseBehavior::Close);
        assert_eq!(cfg.speedup_unit, SpeedupUnit::Hours);
        assert_eq!(cfg.mail_dir, Some(PathBuf::from(r"C:\test\mail")));
        assert_eq!(cfg.mode, Some(FilterMode::AllTargets));
        assert_eq!(cfg.theme, AppTheme::Dark); // Defaults to Dark when not in JSON
        assert!(cfg.autostart);
        assert!(cfg.autostart_minimized);

        let serialized = serde_json::to_string(&cfg).unwrap();
        assert!(serialized.contains(r#""close_behavior":"close""#));
        assert!(serialized.contains(r#""speedup_unit":"hours""#));
        assert!(serialized.contains(r#""mode":"all-targets""#));
        assert!(serialized.contains(r#""theme":"dark""#));
        assert!(serialized.contains(r#""autostart":true"#));
        assert!(serialized.contains(r#""autostart_minimized":true"#));

        // Test theme toggle and serialization
        let toggled_theme = cfg.theme.toggle();
        assert_eq!(toggled_theme, AppTheme::Light);
        assert_eq!(toggled_theme.toggle(), AppTheme::Dark);
        let light_cfg = AppConfig {
            theme: AppTheme::Light,
            ..cfg
        };
        let light_json = serde_json::to_string(&light_cfg).unwrap();
        assert!(light_json.contains(r#""theme":"light""#));
        let deserialized_light: AppConfig = serde_json::from_str(&light_json).unwrap();
        assert_eq!(deserialized_light.theme, AppTheme::Light);

        // Test label and description
        assert_eq!(CloseBehavior::Tray.label(), "Thu nhỏ vào Khay (Tray icon)");
        assert_eq!(CloseBehavior::Close.label(), "Thoát ứng dụng (Close)");
        assert!(CloseBehavior::Tray.description().contains("ẩn vào khay"));
        assert!(CloseBehavior::Close.description().contains("tắt ứng dụng"));

        // Test DateFilterPreset AllTime label
        assert_eq!(DateFilterPreset::AllTime.label(), "Từ trước đến nay");
    }

    #[test]
    fn test_barbarian_resource_conversion_and_totals() {
        use super::*;

        let rec = BattleRecord {
            filename: "mail1".into(),
            mail_id: "1".into(),
            battle_type: "barbarian".into(),
            time_raw: 1000,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Dã man Cấp 25".into(),
            target_level: "".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![
                ItemDrop {
                    itype: 2,
                    sub_type: 141, // 1k Food
                    value: 10,
                    pic: "".into(),
                    name_vi: "Thực phẩm 1.000".into(),
                    name_en: "1,000 Food".into(),
                    category: "Tài nguyên".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 142, // 1k Wood
                    value: 20,
                    pic: "".into(),
                    name_vi: "Gỗ 1.000".into(),
                    name_en: "1,000 Wood".into(),
                    category: "Tài nguyên".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 143, // 750 Stone
                    value: 30,
                    pic: "".into(),
                    name_vi: "Đá 750".into(),
                    name_en: "750 Stone".into(),
                    category: "Tài nguyên".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 144, // 500 Gold
                    value: 40,
                    pic: "".into(),
                    name_vi: "Vàng 500".into(),
                    name_en: "500 Gold".into(),
                    category: "Tài nguyên".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 7005, // 1k Choice
                    value: 50,
                    pic: "".into(),
                    name_vi: "Gói C Cấp 1".into(),
                    name_en: "Resource Pack C".into(),
                    category: "Tài nguyên".into(),
                    sprites: vec![],
                },
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        let res = rec.total_resources_breakdown();
        // Food: 10 * 1,000 = 10,000
        assert_eq!(res.food_val, 10_000);
        assert_eq!(res.food_packs, 10);

        // Wood: 20 * 1,000 = 20,000
        assert_eq!(res.wood_val, 20_000);
        assert_eq!(res.wood_packs, 20);

        // Stone: 30 * 750 = 22,500
        assert_eq!(res.stone_val, 22_500);
        assert_eq!(res.stone_packs, 30);

        // Gold: 40 * 500 = 20,000
        assert_eq!(res.gold_val, 20_000);
        assert_eq!(res.gold_packs, 40);

        // Choice: không có giá trị cụ thể quy đổi sang M RSS, chỉ đếm số lượng gói
        assert_eq!(res.choice_val, 0);
        assert_eq!(res.choice_packs, 50);

        // Total: 10k + 20k + 22.5k + 20k = 72,500
        assert_eq!(res.total_rss_val(), 72_500);
        assert_eq!(rec.total_resource_value(), 72_500);

        // Millions formatting
        assert_eq!(format_in_millions_vietnamese(16_344_000), "16.34M");
        assert_eq!(format_in_millions_vietnamese(1_200_000_000), "1.20 tỷ");
        assert_eq!(format_in_millions_vietnamese(500_000), "500.0k");

        // Display string formatting for items
        assert_eq!(get_item_converted_display(2, 141, 16344), Some("16.34M Ngô".into()));
        assert_eq!(get_item_converted_display(2, 142, 15908), Some("15.91M Gỗ".into()));
        assert_eq!(get_item_converted_display(2, 143, 15015), Some("11.26M Đá".into()));
        assert_eq!(get_item_converted_display(2, 144, 14920), Some("7.46M Vàng".into()));
        assert_eq!(get_item_converted_display(2, 7005, 50), Some("50 gói".into()));
        // 1 sách kinh nghiệm cấp 3 = 1000 kinh nghiệm
        assert_eq!(get_item_converted_display(2, 130, 10), Some("10,000 EXP".into()));

        // Materials & AP
        let mut mats = MaterialTotals::default();
        mats.leather_normal = 84;
        mats.leather_advanced = 3; // 3 * 4 = 12
        assert_eq!(mats.leather_total_converted(), 96);

        mats.iron_normal = 87;
        mats.iron_advanced = 6; // 6 * 4 = 24
        assert_eq!(mats.iron_total_converted(), 111);

        mats.ebony_normal = 89;
        assert_eq!(mats.ebony_total_converted(), 89);

        mats.bone_normal = 41;
        mats.bone_advanced = 4; // 4 * 4 = 16
        assert_eq!(mats.bone_total_converted(), 57);

        assert_eq!(mats.total_normal(), 84 + 87 + 89 + 41);
        assert_eq!(mats.total_advanced(), 3 + 6 + 4);
        assert_eq!(mats.total_pieces(), 84 + 3 + 87 + 6 + 89 + 41 + 4);
        assert_eq!(mats.total_converted_normal(), 96 + 111 + 89 + 57);

        let mut ap = ApTotals::default();
        ap.ap_50 = 14;
        ap.ap_100 = 7;
        assert_eq!(ap.total_bottles(), 21);
        assert_eq!(ap.total_ap_points(), 14 * 50 + 7 * 100);

        assert_eq!(AppTheme::Dark.label(), "Giao diện: Tối");
        assert_eq!(AppTheme::Light.label(), "Giao diện: Sáng");
    }

    #[test]
    fn test_exp_book_level_3_value_1000() {
        // Yêu cầu: 1 sách kinh nghiệm cấp 3 = 1000 kinh nghiệm
        assert_eq!(get_item_conversion(2, 130), ConvertedValue::Exp(1_000));
        assert_eq!(get_item_converted_display(2, 130, 1), Some("1,000 EXP".into()));
        assert_eq!(get_item_converted_display(2, 130, 50), Some("50,000 EXP".into()));
        assert_eq!(get_item_converted_display(2, 130, 100), Some("0.10M EXP".into()));
        assert_eq!(get_item_converted_display(2, 130, 1_000), Some("1.00M EXP".into()));
    }

    #[test]
    fn test_resource_pack_c_no_fixed_value_only_quantity() {
        // Yêu cầu: Gói tài nguyên C không có giá trị cụ thể, chỉ cần số lượng
        assert_eq!(get_item_conversion(2, 7005), ConvertedValue::Resource { kind: ResourceKind::ChoicePack, multiplier: 0 });
        assert_eq!(get_item_conversion(2, 7006), ConvertedValue::Resource { kind: ResourceKind::ChoicePack, multiplier: 0 });
        assert_eq!(get_item_converted_display(2, 7005, 120), Some("120 gói".into()));
        assert_eq!(get_item_converted_display(2, 7006, 45), Some("45 gói".into()));

        let mut res = ResourceTotals::default();
        res.food_val = 100_000;
        res.wood_val = 200_000;
        res.choice_packs = 50;
        res.choice_val = 0;
        // Tổng RSS không cộng bừa giá trị của gói C
        assert_eq!(res.total_rss_val(), 300_000);
        assert_eq!(res.choice_packs, 50);
    }

    #[test]
    fn test_standard_resource_pack_conversions() {
        // Food packs (2, 3, 4, 5, 6, 141)
        assert_eq!(get_item_conversion(2, 141), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 1_000 });
        assert_eq!(get_item_conversion(2, 2), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 50_000 });
        assert_eq!(get_item_conversion(2, 3), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 150_000 });
        assert_eq!(get_item_conversion(2, 4), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 500_000 });
        assert_eq!(get_item_conversion(2, 5), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 1_500_000 });
        assert_eq!(get_item_conversion(2, 6), ConvertedValue::Resource { kind: ResourceKind::Food, multiplier: 5_000_000 });

        // Wood packs (8, 9, 10, 11, 12, 142)
        assert_eq!(get_item_conversion(2, 142), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 1_000 });
        assert_eq!(get_item_conversion(2, 8), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 50_000 });
        assert_eq!(get_item_conversion(2, 9), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 150_000 });
        assert_eq!(get_item_conversion(2, 10), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 500_000 });
        assert_eq!(get_item_conversion(2, 11), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 1_500_000 });
        assert_eq!(get_item_conversion(2, 12), ConvertedValue::Resource { kind: ResourceKind::Wood, multiplier: 5_000_000 });

        // Stone packs (14, 15, 16, 17, 18, 143)
        assert_eq!(get_item_conversion(2, 143), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 750 });
        assert_eq!(get_item_conversion(2, 14), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 37_500 });
        assert_eq!(get_item_conversion(2, 15), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 112_500 });
        assert_eq!(get_item_conversion(2, 16), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 375_000 });
        assert_eq!(get_item_conversion(2, 17), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 1_125_000 });
        assert_eq!(get_item_conversion(2, 18), ConvertedValue::Resource { kind: ResourceKind::Stone, multiplier: 3_750_000 });

        // Gold packs (21, 22, 23, 24, 144)
        assert_eq!(get_item_conversion(2, 144), ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 500 });
        assert_eq!(get_item_conversion(2, 21), ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 50_000 });
        assert_eq!(get_item_conversion(2, 22), ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 200_000 });
        assert_eq!(get_item_conversion(2, 23), ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 600_000 });
        assert_eq!(get_item_conversion(2, 24), ConvertedValue::Resource { kind: ResourceKind::Gold, multiplier: 2_000_000 });
    }

    #[test]
    fn test_all_speedups_and_gems_conversions() {
        // Universal Speedups
        assert_eq!(get_item_conversion(2, 34), ConvertedValue::Speedup { category: "Đa Năng", minutes: 1 });
        assert_eq!(get_item_conversion(2, 58), ConvertedValue::Speedup { category: "Đa Năng", minutes: 5 });
        assert_eq!(get_item_conversion(2, 35), ConvertedValue::Speedup { category: "Đa Năng", minutes: 15 });
        assert_eq!(get_item_conversion(2, 60), ConvertedValue::Speedup { category: "Đa Năng", minutes: 30 });
        assert_eq!(get_item_conversion(2, 36), ConvertedValue::Speedup { category: "Đa Năng", minutes: 60 });
        assert_eq!(get_item_conversion(2, 37), ConvertedValue::Speedup { category: "Đa Năng", minutes: 180 });
        assert_eq!(get_item_conversion(2, 38), ConvertedValue::Speedup { category: "Đa Năng", minutes: 480 });
        assert_eq!(get_item_conversion(2, 39), ConvertedValue::Speedup { category: "Đa Năng", minutes: 900 });
        assert_eq!(get_item_conversion(2, 40), ConvertedValue::Speedup { category: "Đa Năng", minutes: 1440 });
        assert_eq!(get_item_conversion(2, 41), ConvertedValue::Speedup { category: "Đa Năng", minutes: 4320 });

        // Building Speedup (65) & Not teleports/keys (45, 47, 48)
        assert_eq!(get_item_conversion(2, 65), ConvertedValue::Speedup { category: "Xây Dựng", minutes: 30 });
        assert_eq!(get_item_conversion(2, 45), ConvertedValue::Other);
        assert_eq!(get_item_conversion(2, 47), ConvertedValue::Other);
        assert_eq!(get_item_conversion(2, 48), ConvertedValue::Other);

        // Research Speedups (84..=89)
        assert_eq!(get_item_conversion(2, 84), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 15 });
        assert_eq!(get_item_conversion(2, 85), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 30 });
        assert_eq!(get_item_conversion(2, 86), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 60 });
        assert_eq!(get_item_conversion(2, 87), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 180 });
        assert_eq!(get_item_conversion(2, 88), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 480 });
        assert_eq!(get_item_conversion(2, 89), ConvertedValue::Speedup { category: "Nghiên Cứu", minutes: 900 });

        // Training Speedups
        assert_eq!(get_item_conversion(2, 71), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 1 });
        assert_eq!(get_item_conversion(2, 72), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 5 });
        assert_eq!(get_item_conversion(2, 74), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 15 });
        assert_eq!(get_item_conversion(2, 76), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 60 });
        assert_eq!(get_item_conversion(2, 77), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 180 });
        assert_eq!(get_item_conversion(2, 78), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 480 });
        assert_eq!(get_item_conversion(2, 79), ConvertedValue::Speedup { category: "Huấn Luyện", minutes: 900 }); // 15 hours

        // Healing Speedups
        assert_eq!(get_item_conversion(2, 92), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 5 });
        assert_eq!(get_item_conversion(2, 94), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 15 });
        assert_eq!(get_item_conversion(2, 96), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 60 });
        assert_eq!(get_item_conversion(2, 97), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 180 });
        assert_eq!(get_item_conversion(2, 98), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 480 });
        assert_eq!(get_item_conversion(2, 99), ConvertedValue::Speedup { category: "Chữa Lành", minutes: 900 }); // 15 hours

        // Gems
        assert_eq!(get_item_conversion(2, 32), ConvertedValue::Gem(2_000));
    }

    #[test]
    fn test_tomes_of_knowledge_and_talent_reset() {
        // 131 is Talent Reset, not EXP
        assert_eq!(get_item_conversion(2, 131), ConvertedValue::Other);

        // Tomes of Knowledge: 128 (100), 129 (500), 130 (1000), 147 (10k), 148 (50k), 149 (200k), 150 (500k)
        assert_eq!(get_item_conversion(2, 128), ConvertedValue::Exp(100));
        assert_eq!(get_item_conversion(2, 129), ConvertedValue::Exp(500));
        assert_eq!(get_item_conversion(2, 130), ConvertedValue::Exp(1_000));
        assert_eq!(get_item_conversion(2, 147), ConvertedValue::Exp(10_000));
        assert_eq!(get_item_conversion(2, 148), ConvertedValue::Exp(50_000));
        assert_eq!(get_item_conversion(2, 149), ConvertedValue::Exp(200_000));
        assert_eq!(get_item_conversion(2, 150), ConvertedValue::Exp(500_000));
    }

    #[test]
    fn test_barbarian_speedup_calculation() {
        // Yêu cầu: Báo cáo man rợ thống kê đầy đủ tăng tốc (Đa năng, Huấn luyện, Chữa lành)
        let rec = BattleRecord {
            filename: "barb_spd".into(),
            mail_id: "barb1".into(),
            battle_type: "barbarian".into(),
            time_raw: 12345,
            time_utc: "".into(),
            time_vn: "".into(),
            target: "Dã Man Cấp 45".into(),
            target_level: "45".into(),
            talent_add: 0,
            pos_x: 0,
            pos_y: 0,
            damage_pct: 100,
            plunder_tier: 0,
            items: vec![
                ItemDrop {
                    itype: 2,
                    sub_type: 34, // Tăng tốc 1 phút (Đa năng)
                    value: 15,
                    pic: "".into(),
                    name_vi: "Tăng tốc 1 phút".into(),
                    name_en: "1-Minute Speedup".into(),
                    category: "Tăng Tốc".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 71, // Tăng tốc huấn luyện 1 phút
                    value: 20,
                    pic: "".into(),
                    name_vi: "Tăng tốc huấn luyện 1 phút".into(),
                    name_en: "1-Minute Training Speedup".into(),
                    category: "Tăng Tốc".into(),
                    sprites: vec![],
                },
                ItemDrop {
                    itype: 2,
                    sub_type: 92, // Tăng tốc chữa lành 5 phút
                    value: 4,
                    pic: "".into(),
                    name_vi: "Tăng tốc chữa lành 5 phút".into(),
                    name_en: "5-Minute Healing Speedup".into(),
                    category: "Tăng Tốc".into(),
                    sprites: vec![],
                },
            ],
            item_totals: HashMap::new(),
            content: "".into(),
        };

        assert_eq!(rec.univ_speedup_minutes(), 15);
        assert_eq!(rec.training_speedup_minutes(), 20);
        assert_eq!(rec.heal_speedup_minutes(), 20); // 4 * 5 = 20 phút
        assert_eq!(rec.total_speedup_minutes(), 55);
    }

    #[test]
    fn test_battle_record_target_level_serde_backward_compat() {
        let legacy_json = r#"{
            "filename": "test.mail",
            "mail_id": "12345",
            "battle_type": "motte",
            "time_raw": 1000,
            "time_utc": "2026-10-23 00:00:00 UTC",
            "time_vn": "2026-10-23 07:00:00",
            "target": "Motte 25",
            "motte_level": "25",
            "pos_x": 100,
            "pos_y": 200,
            "damage_pct": 100,
            "plunder_tier": 1,
            "items": []
        }"#;
        let rec: BattleRecord = serde_json::from_str(legacy_json).expect("Must deserialize legacy motte_level");
        assert_eq!(rec.target_level, "25");
        assert_eq!(rec.talent_add, 0);
    }

    #[test]
    fn test_filter_mode_gathering() {
        assert_eq!(FilterMode::from_str_loose("gathering"), FilterMode::Gathering);
        assert_eq!(FilterMode::from_str_loose("gather"), FilterMode::Gathering);
        assert_eq!(FilterMode::from_str_loose("rss"), FilterMode::Gathering);
        assert_eq!(FilterMode::Gathering.as_str(), "gathering");
        assert!(FilterMode::Gathering.matches("gathering"));
        assert!(!FilterMode::Gathering.matches("motte"));
        assert!(!FilterMode::AllTargets.matches("gathering"));
        assert!(FilterMode::AllRewards.matches("gathering"));
    }
}
