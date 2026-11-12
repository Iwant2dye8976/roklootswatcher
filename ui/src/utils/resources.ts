/**
 * Resource Icon Utilities for Rise of Kingdoms Loot Watcher.
 * Maps resource types, item codes, and resource names to game asset sprites.
 */

export type ResourceKey =
  | "food"
  | "wood"
  | "stone"
  | "gold"
  | "gems"
  | "crystals";

export const RESOURCE_ICONS: Record<ResourceKey, string> = {
  food: "./items/resources/img_food.png",
  wood: "./items/resources/img_wood.png",
  stone: "./items/resources/img_stone.png",
  gold: "./items/resources/img_gold.png",
  gems: "./items/resources/img_gem.png",
  crystals: "./items/resources/img_Crystal.png",
};

const RESOURCE_ITEM_SPRITES = new Set([
  "img_icon_item_19.png",
  "img_icon_item_20.png",
  "img_icon_item_21.png",
  "img_icon_item_22.png",
  "img_icon_item_23.png",
]);

export interface ResourceItemInput {
  itype?: number;
  sub_type?: number;
  name_vi?: string;
  name_en?: string;
  display_name?: string;
  category?: string;
  icon?: string;
  sprites?: string[];
}

function getItemSpritePath(spriteName: string): string | null {
  const filename = spriteName.split(/[\\/]/).pop() || "";

  if (RESOURCE_ITEM_SPRITES.has(filename)) {
    return `./items/resources/${filename}`;
  }
  if (filename.startsWith("img_icon_item_BG")) {
    return `./items/backgrounds/${filename}`;
  }
  if (filename.startsWith("img_icon_item_mat_")) {
    return `./items/materials/${filename}`;
  }
  if (filename.startsWith("img_icon_item_")) {
    return `./items/others/${filename}`;
  }

  return getResourceIcon(filename);
}

/**
 * Resolves all available sprite layers for a catalog item, preserving their
 * background-to-foreground order. Resource labels still use their standard icon.
 */
export function getResourceIconLayers(
  target?: ResourceItemInput | string | number | null,
  subType?: number
): string[] | null {
  if (typeof target === "object" && target !== null && target.sprites?.length) {
    const spritePaths = target.sprites
      .map(getItemSpritePath)
      .filter((path): path is string => path !== null);
    if (spritePaths.length > 0) return spritePaths;
  }

  if (typeof target === "string") {
    const spritePath = getItemSpritePath(target);
    if (spritePath) return [spritePath];
  }

  const resourceIcon = getResourceIcon(target, subType);
  return resourceIcon ? [resourceIcon] : null;
}

/**
 * Resolves the relative URL path to a resource icon.
 * Returns null if the item is not a recognized resource or has no matching sprite.
 */
export function getResourceIcon(
  target?: ResourceItemInput | string | number | null,
  subType?: number
): string | null {
  if (target === undefined || target === null) return null;

  // 1. If target is a number (itype)
  if (typeof target === "number") {
    const itype = target;
    const isub = subType ?? 0;
    if (itype === 1) {
      switch (isub) {
        case 1:
          return RESOURCE_ICONS.food;
        case 2:
          return RESOURCE_ICONS.wood;
        case 3:
          return RESOURCE_ICONS.stone;
        case 4:
          return RESOURCE_ICONS.gold;
        case 5:
        case 10:
        case 12:
        case 25:
          return RESOURCE_ICONS.gems;
        case 9:
          return RESOURCE_ICONS.crystals;
      }
    }
    return null;
  }

  // 2. If target is a string identifier or key
  if (typeof target === "string") {
    const s = target.toLowerCase().trim();

    // Check direct icon filenames
    if (s.includes("img_food")) return RESOURCE_ICONS.food;
    if (s.includes("img_wood")) return RESOURCE_ICONS.wood;
    if (s.includes("img_stone")) return RESOURCE_ICONS.stone;
    if (s.includes("img_gold")) return RESOURCE_ICONS.gold;
    if (s.includes("img_gem")) return RESOURCE_ICONS.gems;
    if (s.includes("img_crystal")) return RESOURCE_ICONS.crystals;

    // Check resource keys and standard labels
    if (
      s === "food" ||
      s.startsWith("food") ||
      s === "rssfood" ||
      s.includes("thực phẩm") ||
      s.includes("proviant")
    ) {
      return RESOURCE_ICONS.food;
    }
    if (
      s === "wood" ||
      s.startsWith("wood") ||
      s === "rsswood" ||
      s.includes("gỗ") ||
      s.includes("holz")
    ) {
      return RESOURCE_ICONS.wood;
    }
    if (
      s === "stone" ||
      s.startsWith("stone") ||
      s === "rssstone" ||
      s === "đá" ||
      s.includes("mỏ đá") ||
      s.includes("stein")
    ) {
      return RESOURCE_ICONS.stone;
    }
    if (
      s === "gold" ||
      s.startsWith("gold") ||
      s === "rssgold" ||
      s.includes("vàng")
    ) {
      return RESOURCE_ICONS.gold;
    }
    if (
      s.includes("gem") ||
      s === "rssgems" ||
      s.includes("đá quý") ||
      s.includes("edelstein") ||
      s.includes("jewel")
    ) {
      return RESOURCE_ICONS.gems;
    }
    if (
      s.includes("crystal") ||
      s === "rsscrystals" ||
      s.includes("pha lê") ||
      s.includes("kristall")
    ) {
      return RESOURCE_ICONS.crystals;
    }

    return null;
  }

  // 3. If target is an object (LootItem, ItemStatDto, etc.)
  const {
    itype,
    sub_type: isub,
    icon,
    sprites,
    name_en,
    name_vi,
    display_name,
    category,
  } = target;

  // Check explicit icon or sprites
  const spriteName = icon || (sprites && sprites[0]);
  if (spriteName) {
    const s = spriteName.toLowerCase();
    if (s.includes("img_food")) return RESOURCE_ICONS.food;
    if (s.includes("img_wood")) return RESOURCE_ICONS.wood;
    if (s.includes("img_stone")) return RESOURCE_ICONS.stone;
    if (s.includes("img_gold")) return RESOURCE_ICONS.gold;
    if (s.includes("img_gem")) return RESOURCE_ICONS.gems;
    if (s.includes("img_crystal")) return RESOURCE_ICONS.crystals;
  }

  // Check itype === 1
  if (itype === 1 && isub !== undefined) {
    const iconByItype = getResourceIcon(itype, isub);
    if (iconByItype) return iconByItype;
  }

  // Check category
  if (category === "Pha Lê" || category === "Crystals") {
    return RESOURCE_ICONS.crystals;
  }

  // Check names for resource packs / items
  const nameCombined = `${name_en || ""} ${name_vi || ""} ${
    display_name || ""
  }`.toLowerCase();
  if (nameCombined.includes("gem") || nameCombined.includes("đá quý")) {
    return RESOURCE_ICONS.gems;
  }
  if (nameCombined.includes("crystal") || nameCombined.includes("pha lê")) {
    return RESOURCE_ICONS.crystals;
  }
  if (nameCombined.includes("food") || nameCombined.includes("thực phẩm")) {
    return RESOURCE_ICONS.food;
  }
  if (nameCombined.includes("wood") || nameCombined.includes("gỗ")) {
    return RESOURCE_ICONS.wood;
  }
  if (
    nameCombined.includes("stone") ||
    nameCombined.includes("mỏ đá") ||
    (nameCombined.includes("đá") && !nameCombined.includes("đá quý"))
  ) {
    return RESOURCE_ICONS.stone;
  }
  if (nameCombined.includes("gold") || nameCombined.includes("vàng")) {
    return RESOURCE_ICONS.gold;
  }

  return null;
}
