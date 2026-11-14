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
      return { src: "/npc/S10_barbarian_fort_lod3.png", fallbackText: "FORT" };
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
    name.includes("english") ||
    name.includes("soldier") ||
    name.includes("binh") ||
    lvl >= 41 ||
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
    return { src: "/items/resources/img_icon_item_7.png", fallbackText: "RSS" };
  }

  // Default fallback to barbarian
  return { src: "/npc/barbarian_lod3.png", fallbackText: "NPC" };
}
