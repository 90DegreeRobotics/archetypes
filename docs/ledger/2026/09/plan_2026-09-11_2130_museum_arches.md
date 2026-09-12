# Plan: Walkable Museum Arches — 2026-09-11 21:30

## Status
PENDING

## Goal

Turn a chosen subset of the castle's wall arches into walkable exhibition chambers, each hanging
real Chronos2 works on its side walls under placards that carry the work's actual provenance.
The arcade already reads as openings you could step into; this makes some of them true.

---

## The arch inventory, counted

| | |
|---|---|
| Bays per gallery storey | **72** (`ARCADE_BAYS_PER_LEVEL`) |
| Gallery storeys | **7** (`GALLERY_LEVELS`) |
| **Total arches in the building** | **504** |
| Clear opening per arch | 7.745m wide, springing at 5.5m, crown at 9.37m |
| Wall thickness behind an arch | 12.0m (`CASTLE_WALL_THICKNESS`) |
| Bay chord (repeat distance) | 9.945m |

`castle::total_arches()` returns this and is pinned by test, so the number does not have to be
recounted by hand as the shell changes.

**All 504 must not become chambers.** At 7.745m wide by 12m deep that is roughly 47,000 m² of
excavated interior, every square metre of it needing geometry, collision and content. The wall
would also stop being a wall: 504 voids through a 12m shell leaves piers standing between holes.

## Proposed allocation

| Tier | Arches | Use |
|---|---|---|
| Gallery 1 (ground-most, y=12) | 12 of 72, every 6th bay | **Museum chambers** — the first and only built tier |
| Galleries 2-7 | 0 | Blind arcade, as now |
| Remaining 60 on gallery 1 | 60 | Blind arcade |

Twelve chambers on one storey, evenly spaced every 60°, is a complete-feeling museum circuit a
player can walk in a few minutes, and it leaves five bays of solid masonry between each opening
so the wall still reads as structure. It also caps content at a number the art library can
actually fill well (see below). Expanding to a second tier later is a data change, not a
redesign, because which bays are chambers will be a list, not a hard-coded assumption.

---

## The art, as it actually exists

Verified in `C:\chronos2\out\first_light` on 2026-09-11:

| | |
|---|---|
| Bundles | **151** |
| PNGs across them | 754 |
| Per bundle | `renders/hero.png`, `front.png`, `side.png`, `top.png` |
| Title source | `human_prompt.txt` — e.g. "whirlpool", "volcano", "mushroom" |
| Provenance available | `manifest.json`, `integrity_report.json`, `governance_report.md`, `decisions.json`, `codex_export.jsonl` |

Chronos2 already has a museum concept to borrow rather than reinvent: `crates/chronos_cli/src/gallery_exhibit.rs`
defines `GalleryWork { image, title }`, derives placard titles from the prompt via
`title_from_prompt`, and deliberately **excludes** runs whose own vista was `museum_wall` so a
photograph of a gallery wall never gets hung on a gallery wall. That exclusion rule must be
carried across, not rediscovered.

**Placards state what the bundle records, and nothing else.** Title from `human_prompt.txt`,
date from the bundle directory name, and the integrity hash from `integrity_report.json`. No
invented artist, no invented year, no generated wall text. If a bundle lacks a field, the
placard omits that line rather than filling it in.

---

## Phase 1 — Make one arch passable

The wall is currently a closed shell: `castle::clamp_inside_wall` holds the player at
`castle_inner_face() - 0.6`. A chamber needs a doorway exemption, exactly as the archetype rooms
already have one (`ROOM_DOOR_HALF_ARC` exempts a wedge of each room's ring).

- [ ] Add `MUSEUM_BAYS: [usize; 12]` and `museum_bay_bearing(index)` to `castle.rs`.
- [ ] Add a chamber-shaped exemption to `clamp_inside_wall`: inside a museum bay's angular
      wedge, the player may pass the wall face and travel out to the chamber's back wall.
- [ ] Add `museum_chamber_surface_y` so the chamber floor is continuous with the gallery deck.
- [ ] **Tests:** a player walking at a museum bay's bearing passes the wall face and stops at the
      back wall; a player one bay over is still stopped at the face; the chamber floor matches
      the deck height so there is no step at the threshold.
- **Gate:** a *walking* capture entering one arch and standing inside it.

## Phase 2 — The chamber module (Blender)

`scripts/author_museum_chamber.py`, third kit module, same contract discipline as the bay:

- [ ] Interior 7.745m wide (matching the arch opening), 9.0m deep, 9.37m to a barrel-vaulted
      ceiling springing at the same 5.5m line as the arch outside, so the vault continues the
      geometry rather than contradicting it. That leaves 3m of the 12m wall as solid backing.
- [ ] Two side walls carrying hanging positions, a back wall, a floor, and a reveal that meets
      the bay's arch with no seam.
- [ ] Bevelled and UV'd throughout; a flat wall panel with proper UVs on each hanging position
      so the artwork texture maps 1:1 without stretching.
- [ ] Export verification: named nodes, no cameras, UVs present, opening width matching the
      arch, depth within the wall thickness.

## Phase 3 — Hanging works and placards

- [ ] New `services/museum.rs`: an `Exhibit { work_image, title, bundle_id, created, integrity_hash }`
      read from a manifest, never by scanning `C:\chronos2` at runtime.
- [ ] **Assets are copied into the repo at author time, not read from `C:\chronos2` at runtime.**
      The game must not depend on a sibling product's working directory existing on a buyer's
      machine — the launcher already treats Chronos as optional. A script stages selected
      `hero.png` files into `assets/museum/` and writes `assets/museum/manifest.json`.
- [ ] Frame + canvas + placard as a small Blender module, instanced per hanging position, with
      the artwork applied as a material texture at runtime from the manifest.
- [ ] Placard text rendered from the manifest fields listed above.
- [ ] **Tests:** the manifest round-trips; a missing image is skipped with a visible gap rather
      than a fabricated placeholder; a `museum_wall` vista bundle is excluded.

## Phase 4 — Approach and reading

- [ ] Chamber lighting: the hall's tiered lamps do not reach inside a 9m recess, so each chamber
      needs its own warm source, and the artwork needs to be readable without being emissive
      (an emissive painting looks like a lightbox, not a painting).
- [ ] A prompt at the threshold, through the existing `HintRequest` priority resource.
- [ ] Optional: a `[Remember]` on an exhibit, reusing `encounter_memory`'s consent model rather
      than inventing a second one.

---

## Also outstanding — visual work this plan does not cover

These are separate modules, listed so they are not mistaken for done:

- **Stair kit module.** The stair reads correctly now that it is limestone instead of dark
  basalt, and its treads are countable, but its open side is a raw stepped silhouette with no
  stringer or parapet, and it has a balustrade on one side only. It is still primitives.
- **Room drums.** Six flat-coloured cylinders; they work at eye level and read as tanks from
  above. No exterior articulation.
- **Stone texture.** Every kit module is UV'd but untextured flat colour. A shared trim sheet
  would serve the arcade, chambers and stairs at once.
- **No shadow casting anywhere.** Every module so far has had to bake its depth read into
  material value instead, which is why the arch recess is near-black rather than merely dark.

## Risks

1. **Content quantity.** 12 chambers with, say, 6 works each is 72 hangings against 151 bundles —
   comfortable. Expanding to a second tier doubles the demand and would start repeating works.
2. **Texture memory.** 72 artwork textures at full resolution is significant; they should be
   downscaled at staging time to the size a wall panel actually needs.
3. **The wall is load-bearing to the fiction as well as the geometry.** Cutting too many holes
   makes the building read as scaffolding. Twelve is deliberately conservative.
