# CODEX LANE C — Council Table (ornate Flower-of-Life astrolabe)

**Owner: Codex.  Status: READY TO START (unblocked).  Director: Claude (Lane A).**
**Cold-start brief — everything you need is in this file. Do not wait on anyone.**

---

## What you are building
Replace the thin, barely-visible procedural table with an ornate **Flower-of-Life astrolabe council table** in the reference style: a dark circular top with a **plasma-etched Flower-of-Life grid**, a **thick gilded rim carrying engraved glyphs**, an **ornate brass + dark-steel splayed leg assembly**, and the existing living **`Stargate_Portal`** disc at the centre (KEEP IT — the operator already approved the portal).

You are rewriting the table authoring script and regenerating **`assets/scenes/table.glb`**.

## Your file ownership (touch ONLY these)
- REWRITE: `scripts/prepare_table.py` (it already builds `assets/scenes/table.glb` and the `Stargate_Portal` — keep the portal, upgrade everything else).
- REGENERATE: `assets/scenes/table.glb`.
- MAY edit/add: `assets/textures/table/**` (keep the existing `portal_vortex_v2.png`).
- **NEVER touch any `.rs` file, `scripts/author_council_chamber.py`, or `assets/scenes/uiscene1.glb`.** Those belong to other lanes.

## FROZEN CONTRACT — the export MUST preserve these
- The animated centre disc node named exactly **`Stargate_Portal`** must remain, with its emissive vortex material/texture unchanged (operator-approved). The engine spawns the table scene under the Name `PortalTable` and animates/reads `Stargate_Portal` — do not rename either.
- The engine scales this table ×5 and sits it at world `(0, -1.1, 0)`. Author in the SAME local scale as the current `prepare_table.py` (top ~1.0 unit radius) so the existing engine placement still fits. If you change overall scale, print the new bounding box in your handoff so Lane A can adjust the one placement constant.

## Build spec (unambiguous) — reference: the operator's spec sheet (gilded glyph rim, plasma-etched Flower-of-Life grid, ornate brass/steel legs)
1. **Keep** the `Stargate_Portal` disc + its vortex texture/material generation exactly as-is (operator-approved). Everything else is upgraded.
2. **Circular top** (~1.0 unit radius, local): dark polished surface (`base ~0.02, roughness 0.4`).
3. **Flower-of-Life inlay:** thin **emissive cyan** geometry forming the classic overlapping-circles Flower-of-Life pattern, inlaid on the top, contained inside the rim. Make it **brighter and denser than the current sparse concentric rings** so it reads clearly in the dark chamber. (Generate the overlapping circles procedurally: a centre circle + 6 around it + the next ring of 12, as thin tori or as an emissive texture on the top face.)
4. **Gilded rim:** a torus noticeably **thicker** than the current thin rim (`metallic 0.92, roughness 0.2`), carrying a ring of **engraved-glyph geometry** — simple extruded notches/blocks spaced around the rim are fine — so the rim reads ornate, not a plain hoop.
5. **Leg assembly:** an ornate **central pedestal + 4 splayed brass legs** with **dark-steel feet**, visibly substantial (the current legs are too thin/dim). Add a subtle **emissive under-glow ring** beneath the top so the table doesn't vanish into the dark chamber.
6. **Triangle budget:** keep the whole table **under ~200k triangles**.
7. **Materials:** gold/brass `metallic 0.92 roughness 0.2`; dark steel `metallic 0.7 roughness 0.35`; cyan inlay emissive.

## Export
Use the export call already in `prepare_table.py` (GLB, `export_apply=True`, `export_yup=True`) writing to `C:/archetypes/assets/scenes/table.glb`.
Run headless: `blender --background --python scripts/prepare_table.py`.
Blender: `C:\Program Files\Blender Foundation\Blender 4.5\blender.exe`.

## Verification you MUST do before declaring done
Re-import the exported `table.glb` headless and assert/print:
- `Stargate_Portal` node still present,
- total triangle count (< ~200k),
- the overall bounding box (so Lane A can confirm scale/placement).
Print the counts. **You do NOT run the game** — that is Lane A's on-screen check. Hand off the GLB + your import-assertion output + the updated script.

## Hard rules (repo doctrine)
- Plan doc first: create `docs/ledger/2026/07/plan_<date>_flower-table-asset.md` before you touch anything.
- Work only on `main`. Never delete files; keep the raw AI table source preserved/hidden as the current script already does.
- Do not present a stub as real. If you cannot keep `Stargate_Portal` intact, STOP and report.
- Commit + push to `origin/main` when the GLB re-imports clean with `Stargate_Portal` present.
