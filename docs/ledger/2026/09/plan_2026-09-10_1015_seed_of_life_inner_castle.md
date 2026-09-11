# Plan: seed-of-life-inner-castle — 2026-09-10 10:15

## Status
IN-PROGRESS — SCALE RESET

## Goal
Replace the failed gallery/rotunda arrangement with the operator-approved Seed-of-Life
castle plan: one sovereign Council/Witness circle at the center, six equal elevated
outer archetype rooms, and long bridged approaches over a visible under-castle abyss.
The world must read as an enormous traversable ancient castle, not a computer scene,
prototype, or an AI-concept painting copied without spatial truth.

The first rendered Seed-of-Life attempt proved the initial 19m outer-ring radius and
7.35m room radius are spatially cramped and visually inadequate. A later 92m-perimeter
draft remained a set-piece rather than a castle. The operator rejected both directions on
2026-09-10. No screenshot from either attempt is acceptance evidence.

The live target is a dense castle-complex, not set-dressing scale or a fortress-town: a
36m Council island, six 48m archetype wings centered 140m from the Council, ~56m bridge
journeys, and a 210m outer perimeter (approximately 420m across). Each Heaven is a ~1.3km perimeter
promenade/overlook. This requires chunked procedural architecture, distance-aware detail,
and deliberate destinations; naïvely spawning thousands of individual primitive entities is
not an acceptable substitute for world design.

## Evidence and constraints

- The operator supplied the governing plan drawing: a center circle with six tangent outer
  circles inside an enclosing castle perimeter. It supersedes the prior crescent-niche layout.
- Center remains the Witness/Council rotunda; Jester is the Inner Chambers host at the center
  without superseding Witness sovereignty. Outer rooms are Architect, Sentinel, Mentor,
  Explorer, Oracle, and Empath.
- The operator identified additional Downloads GLBs. Audit found AURA, Sentinel, Empath,
  Mentor, Oracle, and Nebula Jester. AURA is the temporary central embodiment of all archetypes.
  Architect and Explorer embodiments were supplied after the original audit and now enter the
  Blender intake lane alongside Mentor. AURA remains the temporary central embodiment.
- Preserve the existing catalog/persistence identifiers and manifestation route. Remove runtime
  exhibition staging only; retain source GLB assets and a reusable ambient-turntable helper for
  later intentional placement.

## Steps

### Step 1 — Rebuild the floor plan at dense castle-complex scale
- [ ] Action: Replace the rejected set-piece dimensions with the stated 36m/48m/140m/56m/210m
  macro layout. Build it as chunked procedural world geometry with distance-aware detail,
  not as an unbounded count of separate cubes; scale abyss, navigation, collision, camera,
  and capture framing coherently.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`, collision/capture helpers.
- Expected outcome: The world reads as a dense, 420m ancient fortress: sustained journeys,
  distinct wing districts, long sightlines, and no visual crowding.

### Step 2 — Replace prototype massing with authored castle architecture
- [ ] Action: Generate code-native cobblestone wall treatment, large stone floor slabs, brick
  edge trim, doorway surrounds, simple shelves/counters/furniture, and room-specific material,
  light, silhouette, and furnishing parameters from the canonical theme docs.
- Files touched: `world.rs`, focused tests, architecture documentation.
- Expected outcome: Every outer room has a distinct functional visual grammar and a navigable,
  materially legible ancient-stone feel without generic gothic clutter or flat-box furniture.

### Step 3 — Place real available figures honestly
- [ ] Action: Stage the operator-supplied Architect, Explorer, and Mentor GLBs through the
  existing asset lane, then place AURA at center and every outer archetype in a room whose
  sight line, floor, backing architecture, and furniture suit its current unposed mesh.
- Files touched: `world.rs`, capture harness, truth docs.
- Expected outcome: Existing unposed meshes are framed by room architecture suited to their
  current poses, never treated as posed actors or invented models.

### Step 4 — Build the Seven Heavens ascent
- [ ] Action: Add seven materially walkable circular galleries on the inner face of the far
  enclosing castle wall, from floor to upper vault, joined by an outward-spiraling stone stair.
  Make its geometry, collision/grounding, lighting, and visual hierarchy real; do not represent
  the seven levels as UI labels or non-walkable set dressing.
- Files touched: `world.rs`, `camera.rs`, capture helpers, focused topology tests.
- Expected outcome: The castle carries a readable vertical journey and a reason to walk the
  full perimeter: every Heaven is an overlook with different downward views of the Council,
  bridges, and six rooms. Its circulation wall reserves deliberate bays for future art, carpets,
  banners, shelves, and collectible/interactive objects rather than treating decoration as
  arbitrary filler.

### Step 5 — Build the circulation-wall grammar
- [ ] Action: Create repeatable architectural bays along every Heaven gallery: stone parapet,
  sightline openings, banner/art recesses, carpet lanes, and shelf/display attachment points.
  Instantiate only durable ancient-castle elements now; retain named sockets for future art and
  gameplay objects rather than inventing fake content.
- Files touched: `world.rs`, future asset/catalog documentation.
- Expected outcome: The outer wall reads as an explorable castle promenade with room for the
  game to grow, not as empty circumference around an otherwise isolated scene.

### Step 6 — Make distance worth traversing
- [ ] Action: Define durable location categories and placement rules for view overlooks,
  art/sigil bays, carpets, banners, shelves, manifestation discoveries, shortcuts, and future
  interactive mechanics across bridges, wings, and Seven Heavens. Implement only real
  architectural supports/landmarks now; record future content as unbuilt rather than fake it.
- Files touched: architecture documentation, `world.rs`, future gameplay catalogs.
- Expected outcome: Scale creates exploration choices and meaningful destinations, not dead air.

### Step 7 — Verify the buyer-facing structure
- [ ] Action: Add topology/material tests, run `cargo test --workspace`, restage the pinned
  launcher, capture/inspect the new center and all six rooms, update status, commit and push.
- Files touched: capture proof, ledger, `STATUS.md`.
- Expected outcome: A real installed render proves the layout and names remaining model gaps.
