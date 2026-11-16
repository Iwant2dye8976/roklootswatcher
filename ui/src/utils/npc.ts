/**
 * Resolves an authentic in-game NPC target emblem sprite from /npc/
 * based on battle type, target name, and target level.
 */
export function getNpcSprite(
  battleType?: string,
  targetName?: string,
  targetLevel?: string | number
): { src: string; fallbackText: string } {
  const bt = (battleType || "").toLowerCase();
  const name = (targetName || "").toLowerCase();
  const lvl =
    typeof targetLevel === "number"
      ? targetLevel
      : parseInt(String(targetLevel || "").replace(/\D+/g, "") || "0", 10);

  if (bt === "fort" || name.includes("fort") || name.includes("pháo đài")) {
    if (lvl >= 6 || name.includes("s10") || name.includes("kvk")) {
      return { src: "/npc/barbarian_fort_lod3.png", fallbackText: "FORT" };
    }
    return { src: "/npc/barbarian_fort_lod3.png", fallbackText: "FORT" };
  }

  if (
    bt === "motte" ||
    name.includes("motte") ||
    name.includes("guardian") ||
    name.includes("hộ vệ")
  ) {
    return {
      src: "/npc/S10_barbarian_fort_lod3.png",
      fallbackText: "MOTTE",
    };
  }

  if (name.includes("viking")) {
    return { src: "/npc/viking_barbarian_lod3.png", fallbackText: "VIK" };
  }

  if (
    lvl >= 41 ||
    name.includes("english") ||
    name.includes("soldier") ||
    name.includes("binh") ||
    name.includes("s10")
  ) {
    return { src: "/npc/S10_barbarian_lod3.png", fallbackText: "BARB" };
  }

  if (
    bt === "barbarian" ||
    name.includes("barbarian") ||
    name.includes("dã man")
  ) {
    return { src: "/npc/barbarian_lod3.png", fallbackText: "BARB" };
  }

  if (
    bt === "gathering" ||
    name.includes("gather") ||
    name.includes("thu thập") ||
    name.includes("mỏ")
  ) {
    if (name.includes("logging") || name.includes("gỗ")) {
      return { src: "/items/resources/img_wood.png", fallbackText: "WOOD" };
    }
    if (name.includes("stone") || name.includes("đá")) {
      return { src: "/items/resources/img_stone.png", fallbackText: "STONE" };
    }
    if (name.includes("cropland") || name.includes("thực phẩm")) {
      return { src: "/items/resources/img_food.png", fallbackText: "FOOD" };
    }
    if (name.includes("gold") || name.includes("vàng")) {
      return { src: "/items/resources/img_gold.png", fallbackText: "GOLD" };
    }
    if (name.includes("gem") || name.includes("đá quý")) {
      return { src: "/items/resources/img_gem.png", fallbackText: "GEM" };
    }
    return { src: "/items/resources/img_icon_item_7.png", fallbackText: "RSS" };
  }

  // Default fallback to barbarian
  return { src: "/npc/barbarian_lod3.png", fallbackText: "NPC" };
}

/**
 * Returns formatted target name based on npc.yaml conventions.
 */
export function lookupNpcName(levelOrTarget: string | number): string {
  const lvl =
    typeof levelOrTarget === "number"
      ? levelOrTarget
      : parseInt(String(levelOrTarget).replace(/\D+/g, "") || "0", 10);
  if (lvl >= 41 && lvl <= 55) {
    return `Lvl ${lvl} English Soldiers`;
  }
  if (lvl >= 1 && lvl <= 40) {
    return `Lvl ${lvl} Barbarians`;
  }
  return String(levelOrTarget);
}
