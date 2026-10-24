# Overlay Position D-Pad Controls, Independent Sizing, and See-Through Transparency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable users to position the RoK loot overlay anywhere on screen using interactive holdable D-Pad arrow controls or numeric coordinates, customize card width and font size independently, and choose 0% pure see-through transparency.

**Architecture:** Extend `OverlayConfig` with `offset_x`, `offset_y`, `width`, and `font_size` using backward-compatible serde defaults. Update `calculate_overlay_geometry` to compute window coordinates from game bounds and offsets. Integrate an egui 3x3 D-Pad with real-time pointer-down hold detection and drag values in the main GUI Settings tab, while updating `render_overlay_viewport` to dynamically scale font sizes and eliminate card backgrounds when opacity is set to 0%.

**Tech Stack:** Rust (edition 2021), egui 0.29, eframe 0.29, serde, windows-sys.

**Spec:** [`docs/superpowers/specs/2026-10-24-overlay-controls-transparency-design.md`](file:///C:/roklootswatcher/docs/superpowers/specs/2026-10-24-overlay-controls-transparency-design.md)

## Global Constraints
- Strictly ZERO Unicode emojis in any overlay text, labels, headers, or buttons.
- Click-through and window non-activation must be preserved (`with_mouse_passthrough(true)` and `with_active(false)`).
- Full backward-compatibility with legacy `rok_config.json` files missing the new fields.
- Zero compiler warnings under `cargo clippy -- -D warnings`.

## Review Focus
1. Legacy config loading: Deserializing a JSON lacking `offset_x`, `offset_y`, `width`, or `font_size` must populate valid defaults without failing.
2. Extreme offset bounds: Setting huge negative or positive offsets must not crash or trigger arithmetic underflow/overflow.
3. Pure see-through contrast: At 0.0 opacity, text and vertical accent bars must remain visible and legible over the game world.
4. Font size scaling: Cards must not clip or wrap into illegible artifacts across the entire 8.5pt–15.0pt range.
5. Continuous holding responsiveness: Holding a D-pad arrow button must continuously update the overlay position on each rendered frame.

---

### Task 1: Model & Config Schema Updates

**Files:**
- Modify: `src/model.rs`
- Test: `src/model.rs` (unit tests)

**Interfaces:**
- Produces:
  - `OverlayConfig` fields: `offset_x: f32`, `offset_y: f32`, `width: f32`, `font_size: f32`.
  - Helper functions: `default_overlay_offset_x() -> f32`, `default_overlay_offset_y() -> f32`, `default_overlay_width() -> f32`, `default_overlay_font_size() -> f32`.

- [ ] **Step 1: Write failing tests for new `OverlayConfig` fields and serde defaults**

Add tests to `mod tests` in `src/model.rs`:
```rust
#[test]
fn test_overlay_config_position_and_sizing_defaults() {
    let legacy_json = r#"{"enabled":true,"opacity":0.8,"text_color_hex":"#FFFFFF"}"#;
    let config: OverlayConfig = serde_json::from_str(legacy_json).expect("Must parse legacy config");
    assert_eq!(config.offset_x, 16.0);
    assert_eq!(config.offset_y, 60.0);
    assert_eq!(config.width, 240.0);
    assert_eq!(config.font_size, 10.5);
}

#[test]
fn test_overlay_config_custom_position_and_sizing_roundtrip() {
    let cfg = OverlayConfig {
        enabled: true,
        opacity: 0.0,
        text_color_hex: "#38BDF8".into(),
        filter: FilterMode::All,
        offset_x: 120.0,
        offset_y: 200.0,
        width: 320.0,
        font_size: 13.0,
    };
    let json = serde_json::to_string(&cfg).expect("Must serialize");
    let back: OverlayConfig = serde_json::from_str(&json).expect("Must deserialize");
    assert_eq!(back, cfg);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test model::tests::test_overlay_config_position_and_sizing_defaults`
Expected: FAIL due to missing fields in `OverlayConfig`.

- [ ] **Step 3: Implement new fields in `OverlayConfig` and helper functions**

In `src/model.rs`:
- Add `offset_x: f32`, `offset_y: f32`, `width: f32`, `font_size: f32` to `OverlayConfig` with `#[serde(default = "...")]` attributes.
- Implement default functions:
  - `fn default_overlay_offset_x() -> f32 { 16.0 }`
  - `fn default_overlay_offset_y() -> f32 { 60.0 }`
  - `fn default_overlay_width() -> f32 { 240.0 }`
  - `fn default_overlay_font_size() -> f32 { 10.5 }`
- Update `OverlayConfig::default()` to populate these default values.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test model::tests::test_overlay_config`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/model.rs
git commit -m "feat(model): add offset, width, and font_size fields to OverlayConfig with serde defaults"
```

---

### Task 2: Dynamic Geometry Tracker with Custom Offsets & Sizing

**Files:**
- Modify: `src/overlay_tracker.rs`
- Test: `src/overlay_tracker.rs` (unit tests)

**Interfaces:**
- Consumes: `OverlayConfig` from Task 1.
- Produces: `calculate_overlay_geometry(bounds: &RoKWindowBounds, config: &OverlayConfig, scale_factor: f32) -> OverlayGeometry`.

- [ ] **Step 1: Write failing tests for geometry calculation with custom offsets and font sizes**

In `src/overlay_tracker.rs` tests module:
```rust
#[test]
fn test_calculate_overlay_geometry_with_custom_offsets() {
    let bounds = RoKWindowBounds {
        x: 100,
        y: 100,
        width: 1920,
        height: 1080,
    };
    let cfg = OverlayConfig {
        enabled: true,
        opacity: 0.8,
        text_color_hex: "#FFFFFF".into(),
        filter: FilterMode::All,
        offset_x: 250.0,
        offset_y: 150.0,
        width: 300.0,
        font_size: 12.0,
    };
    let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
    assert_eq!(geom.pos_x, 350.0);
    assert_eq!(geom.pos_y, 250.0);
    assert_eq!(geom.width, 300.0);
    assert!(geom.height >= 180.0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test overlay_tracker::tests::test_calculate_overlay_geometry_with_custom_offsets`
Expected: FAIL due to signature difference or missing parameters.

- [ ] **Step 3: Update `calculate_overlay_geometry` signature and dynamic sizing**

In `src/overlay_tracker.rs`:
- Update `calculate_overlay_geometry(bounds: &RoKWindowBounds, config: &OverlayConfig, scale_factor: f32) -> OverlayGeometry`:
  - `pos_x = (bounds.x as f32 + config.offset_x) / scale;`
  - `pos_y = (bounds.y as f32 + config.offset_y) / scale;`
  - `width = config.width.clamp(180.0, 450.0);`
  - `card_height = (config.font_size * 3.3).max(28.0);`
  - `est_height = (card_height * 5.0 + 20.0).max(150.0);`
  - `max_height = ((bounds.height as f32 - 40.0) / scale).max(150.0);`
  - `height = est_height.min(max_height);`
- Update existing tests in `src/overlay_tracker.rs` to pass `&OverlayConfig::default()`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test overlay_tracker::tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/overlay_tracker.rs
git commit -m "feat(overlay): update geometry calculation to support custom offsets and dynamic card heights"
```

---

### Task 3: Dynamic Typography and Pure See-Through Rendering

**Files:**
- Modify: `src/overlay_ui.rs`
- Test: `src/overlay_ui.rs` (unit tests)

**Interfaces:**
- Consumes: `OverlayConfig` from Task 1.
- Produces: `render_overlay_viewport(ctx: &egui::Context, records: &[BattleRecord], config: &OverlayConfig, last_mail_time: Instant)`.

- [ ] **Step 1: Write test verifying see-through mode parameters**

In `src/overlay_ui.rs` test module:
```rust
#[test]
fn test_see_through_opacity_zero() {
    let cfg = OverlayConfig {
        enabled: true,
        opacity: 0.0,
        text_color_hex: "#F8FAFC".into(),
        filter: FilterMode::All,
        offset_x: 16.0,
        offset_y: 60.0,
        width: 240.0,
        font_size: 10.5,
    };
    // Verification helper or assert on styling bounds
    assert_eq!(cfg.opacity, 0.0);
}
```

- [ ] **Step 2: Implement dynamic typography scaling and 0.0 see-through transparency**

In `src/overlay_ui.rs`:
- Update `render_overlay_viewport`:
  - Calculate `eff_opacity = (config.opacity * dim_factor).clamp(0.0, 1.0);`
  - If `config.opacity <= 0.01`:
    - `bg_color = Color32::TRANSPARENT;`
    - `card_stroke = Stroke::NONE;`
  - Else:
    - `bg_color = Color32::from_rgba_unmultiplied(15, 23, 42, (220.0 * eff_opacity) as u8);`
    - `card_stroke = Stroke::new(1.0, Color32::from_white_alpha((35.0 * eff_opacity) as u8));`
  - Compute font sizes from `config.font_size`:
    - Title/tag: `config.font_size`
    - Subtitle/meta/drops: `(config.font_size * 0.88).max(8.0)`
    - Time stamp: `(config.font_size * 0.82).max(7.5)`
  - Card inner margin and accent rect dynamically adapt to `config.font_size`.
  - For see-through mode, text color maintains readable contrast: `base_text_color.linear_multiply(eff_opacity.max(0.70))`.

- [ ] **Step 3: Run all unit tests in `overlay_ui`**

Run: `cargo test overlay_ui::tests`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/overlay_ui.rs
git commit -m "feat(overlay): implement dynamic typography scaling and 0% see-through transparency"
```

---

### Task 4: Interactive D-Pad Arrow Controls & Sizing Sliders Integration

**Files:**
- Modify: `src/gui.rs`
- Test: `src/gui.rs` (unit tests)

**Interfaces:**
- Consumes: Task 1 (`OverlayConfig`), Task 2 (`calculate_overlay_geometry`), Task 3 (`render_overlay_viewport`).
- Produces: Integrated Settings UI panel with holdable D-Pad, offset inputs, width and font size sliders.

- [ ] **Step 1: Write test for GUI overlay configuration state persistence**

In `src/gui.rs` tests module:
```rust
#[test]
fn test_overlay_state_with_position_and_size() {
    let mut config = AppConfig::default();
    config.overlay.offset_x = 45.0;
    config.overlay.offset_y = 120.0;
    config.overlay.width = 280.0;
    config.overlay.font_size = 12.5;
    assert_eq!(config.overlay.offset_x, 45.0);
    assert_eq!(config.overlay.width, 280.0);
}
```

- [ ] **Step 2: Run test to verify it compiles and passes**

Run: `cargo test gui::tests::test_overlay_state_with_position_and_size`
Expected: PASS.

- [ ] **Step 3: Implement D-Pad arrow controls, numeric inputs, and sliders in Settings**

In `src/gui.rs` `render_view_settings`:
- Add 3x3 D-Pad layout:
  - Arrow buttons: `▲`, `▼`, `◄`, `►`.
  - Holding detection: `if btn.is_pointer_button_down_on() { ... self.overlay_config.offset_x += step; self.last_mail_arrival = Instant::now(); self.save_config(); ctx.request_repaint(); }`
  - Center button `[ RoK ]`: resets offsets to `(16.0, 60.0)`.
  - Numeric `DragValue` inputs for `X Offset` and `Y Offset`.
- Add Sliders:
  - `Overlay Width`: `180.0_f32 ..= 380.0_f32` with `{:.0} px`.
  - `Font Size`: `8.5_f32 ..= 15.0_f32` with `{:.1} pt`.
  - `Background Opacity`: `0.0_f32 ..= 1.0_f32` with custom formatter (`0% (See-Through)` at 0.0, else `{:.0}%`).
- In overlay viewport handling (around line 3677):
  - Pass `&self.overlay_config` into `calculate_overlay_geometry`.

- [ ] **Step 4: Run test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/gui.rs
git commit -m "feat(gui): integrate D-Pad hold controls, numeric offsets, and sizing sliders"
```

---

### Task 5: End-to-End Verification & Release Build

**Files:**
- All files

- [ ] **Step 1: Run full cargo test suite**

Run: `cargo test`
Expected: 99+ tests pass, 0 fail.

- [ ] **Step 2: Run clippy with strict warnings check**

Run: `cargo clippy -- -D warnings`
Expected: 0 warnings.

- [ ] **Step 3: Build release binary and update root executable**

Run:
```powershell
cargo build --release
Copy-Item target\release\rok_watcher.exe .\rok_watcher.exe -Force
```
Expected: Exit code 0, binary updated.

- [ ] **Step 4: Commit and push**

```bash
git add -u
git commit -m "chore(release): verify all tests and update release binary with overlay controls"
```
