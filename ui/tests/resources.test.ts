import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import test from "node:test";
import { getResourceIconLayers } from "../src/utils/resources.ts";

test("resolves loot background and item sprites into public asset paths", () => {
  assert.deepEqual(
    getResourceIconLayers({
      sprites: ["img_icon_item_BGBlue.png", "img_icon_item_9.png"],
    }),
    [
      "./items/backgrounds/img_icon_item_BGBlue.png",
      "./items/others/img_icon_item_9.png",
    ]
  );
});

test("resolves material and resource sprites into their asset folders", () => {
  assert.deepEqual(
    getResourceIconLayers({
      sprites: ["img_icon_item_BGWhite.png", "img_icon_item_mat_15.png"],
    }),
    [
      "./items/backgrounds/img_icon_item_BGWhite.png",
      "./items/materials/img_icon_item_mat_15.png",
    ]
  );
  assert.deepEqual(getResourceIconLayers("food"), [
    "./items/resources/img_food.png",
  ]);
});

test("serves the newly added catalog material sprites", () => {
  const materialSprites = [
    "img_icon_item_mat_14.png",
    "img_icon_item_mat_24.png",
    "img_icon_item_mat_29.png",
    "img_icon_item_mat_34.png",
  ];

  for (const sprite of materialSprites) {
    assert.deepEqual(getResourceIconLayers({ sprites: [sprite] }), [
      `./items/materials/${sprite}`,
    ]);
    assert.ok(
      existsSync(new URL(`../public/items/materials/${sprite}`, import.meta.url)),
      `Missing frontend asset for ${sprite}`
    );
  }
});

test("resolves and serves the new resource pack sprites", () => {
  const resourceSprites = [
    "img_icon_item_19.png",
    "img_icon_item_20.png",
    "img_icon_item_21.png",
    "img_icon_item_22.png",
    "img_icon_item_23.png",
  ];

  for (const sprite of resourceSprites) {
    assert.deepEqual(getResourceIconLayers({ sprites: [sprite] }), [
      `./items/resources/${sprite}`,
    ]);
    assert.deepEqual(getResourceIconLayers(sprite), [
      `./items/resources/${sprite}`,
    ]);
    assert.ok(
      existsSync(new URL(`../public/items/resources/${sprite}`, import.meta.url)),
      `Missing frontend asset for ${sprite}`
    );
  }
});

test("serves the new HeroCarving star sprites", () => {
  const sprites = [
    "img_icon_item_HeroCarving_Star2.png",
    "img_icon_item_HeroCarving_Star3.png",
  ];

  for (const sprite of sprites) {
    assert.deepEqual(getResourceIconLayers({ sprites: [sprite] }), [
      `./items/others/${sprite}`,
    ]);
    assert.ok(
      existsSync(new URL(`../public/items/others/${sprite}`, import.meta.url)),
      `Missing frontend asset for ${sprite}`
    );
  }
});

test("resolves every pinned preset ID to an existing icon sprite", () => {
  const pinnedPresetIcons: Array<[string, string]> = [
    ["gems", "./items/resources/img_gem.png"],
    ["crystals", "./items/resources/img_Crystal.png"],
    ["books", "./items/others/img_icon_item_24.png"],
    ["arrows", "./items/others/img_icon_item_2.png"],
    ["spd_univ", "./items/others/img_icon_item_10.png"],
    ["spd_train", "./items/others/img_icon_item_12.png"],
    ["spd_heal", "./items/others/img_icon_item_11.png"],
  ];

  for (const [presetId, iconPath] of pinnedPresetIcons) {
    assert.deepEqual(getResourceIconLayers(presetId), [iconPath]);
    assert.ok(
      existsSync(
        new URL(`../public/${iconPath.replace(/^\.\//, "")}`, import.meta.url)
      ),
      `Missing frontend asset for pinned preset ${presetId}`
    );
  }
});

test("resolves backend overlay chip IDs to their icon sprites", () => {
  const overlayChipIcons: Array<[string, string]> = [
    ["Catalog:1:2:Gems", "./items/resources/img_gem.png"],
    ["Catalog:1:9:Crystals", "./items/resources/img_Crystal.png"],
    ["Catalog:2:3:Book of Covenant", "./items/others/img_icon_item_24.png"],
    ["Catalog:2:4:Arrow of Resistance", "./items/others/img_icon_item_2.png"],
    ["SpeedupUniversal", "./items/others/img_icon_item_10.png"],
    ["SpeedupTraining", "./items/others/img_icon_item_12.png"],
    ["SpeedupHealing", "./items/others/img_icon_item_11.png"],
  ];

  for (const [chipId, iconPath] of overlayChipIcons) {
    assert.deepEqual(getResourceIconLayers(chipId), [iconPath]);
  }
});
