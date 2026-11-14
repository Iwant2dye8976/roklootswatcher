import assert from "node:assert/strict";
import { test } from "node:test";
import { getNpcSprite } from "../src/utils/npc.ts";

test("resolves Barbarian Fort to authentic fort sprites", () => {
  const lowFort = getNpcSprite("fort", "Barbarian Fort 5", "5");
  assert.equal(lowFort.src, "/npc/barbarian_fort_lod3.png");
  assert.equal(lowFort.fallbackText, "FORT");

  const highFort = getNpcSprite("fort", "Barbarian Fort 10", "10");
  assert.equal(highFort.src, "/npc/S10_barbarian_fort_lod3.png");

  const kvkFort = getNpcSprite("fort", "S10 Fort", "5");
  assert.equal(kvkFort.src, "/npc/S10_barbarian_fort_lod3.png");
});

test("resolves Motte and Guardian to guardian sprites", () => {
  const motte = getNpcSprite("motte", "Motte of the Cursed", "15");
  assert.equal(motte.src, "/npc/S10_barbarian_guardian_lod3.png");
  assert.equal(motte.fallbackText, "MOTTE");

  const guardian = getNpcSprite("barbarian", "Holy Guardian", "0");
  assert.equal(guardian.src, "/npc/S10_barbarian_guardian_lod3.png");
});

test("resolves Barbarians, English Soldiers, and Vikings correctly", () => {
  const lowBarb = getNpcSprite("barbarian", "Barbarians Lvl 25", "25");
  assert.equal(lowBarb.src, "/npc/barbarian_lod3.png");
  assert.equal(lowBarb.fallbackText, "BARB");

  const englishSoldier = getNpcSprite("barbarian", "Lvl 45 English Soldiers", "45");
  assert.equal(englishSoldier.src, "/npc/S10_barbarian_lod3.png");

  const highBarb = getNpcSprite("barbarian", "Barbarians Lvl 50", "50");
  assert.equal(highBarb.src, "/npc/S10_barbarian_lod3.png");

  const viking = getNpcSprite("barbarian", "Viking Invaders", "30");
  assert.equal(viking.src, "/npc/viking_barbarian_lod3.png");
});

test("resolves Gathering to resource icon", () => {
  const gather = getNpcSprite("gathering", "Food Deposit", "6");
  assert.equal(gather.src, "/items/resources/img_icon_item_7.png");
  assert.equal(gather.fallbackText, "RSS");
});
