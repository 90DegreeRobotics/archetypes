# Archetypes Status

**Last Updated: 2026-09-13 (Proven Manifester baseline restored)**

This document tracks time-sensitive status, current blockers, and recent test runs.

## Current State
- **Proven Manifester baseline restored (2026-09-13):** the surviving real `ice cream cone`
  bundle at `%TEMP%\segfix_proto\live_test2` establishes the last acceptable configuration:
  Chronos2's default compact Flux 768px reference lane, TripoSR MC 256, decoder chunk 65,536,
  foreground ratio 0.85, fitted-plane background segmentation, one complete retained component,
  smooth shading, 19,956 vertices and 39,854 faces. Its multi-angle renders show a coherent
  three-dimensional cone and soft serve. Archetypes now uses that reference lane, MC 256 and
  topology-preserving smooth shading again.
- **The claimed full-volume lane in commit `1e1096f` is retracted:** its generator is an unused
  three-recipe primitive experiment, not the arbitrary-prompt runtime. Its optional, unversioned,
  unbound `inspection.json` could return a stale PASS for a nonexistent model and approved a
  visibly broken capped "goblet." The source remains as rejected audit evidence, but those tools
  are no longer packaged or consumed by the buyer path.
- **Historical object-quality lab (2026-09-12; geometry override conclusion retracted):** `scripts/manifest_quality_lab.py` is a
  repeatable loop — generate a spread of subjects once, then re-render the *same* meshes after
  each change, so a tuning result is isolated from generation variance. Seven subjects run
  (astrolabe, ceramic owl, candelabra, crystal decanter, copper kettle, granite lion, plus the
  live relic). **The dominant variable is subject extraction, not reconstruction.** Where the
  image handed to TripoSR is clean the mesh is recognisable; where extraction collapses the mesh
  is noise. Measured `subject_coverage`: 0.4456 / 0.1298 / 0.0947 for good results against
  **0.0006** for the white owl and **0.0006** for the crystal decanter — both subjects close in
  colour to their own backdrop. There is no overlap between the groups.
  Three changes were shipped here; the EdgeSplit and MC-384 conclusions are now superseded by
  the stronger all-angle ice-cream witness:
  (1) `import_chronos_object.py` no longer force-smooths every polygon. Hard edges above 30
  degrees survive via an EdgeSplit that bakes into the geometry so it holds through glTF; the
  plinth's flat faces and gold band now read as edges instead of melting. It also writes an
  explicit material carrying the reconstruction's vertex colour — the export previously carried
  `COLOR_0` but **no material at all**, leaving every consumer to invent one.
  (2) `CHRONOS_TRIPOSR_MC_RESOLUTION` raised from Chronos2's default 256 to **384**, measured on
  the same cached reference: 141,269 faces to 320,670 for five seconds more, and after both were
  decimated to the same 75k budget the finer source kept the knurled case rim and dial the
  coarse one lost.
  (3) A coverage floor of 0.02 now **refuses** a reconstruction built from a blank frame and
  reports the measurement, rather than staging noise on the altar and calling it the player's
  work. Deliberately not gated on `subject_match.json`: it scored 0.260 for the destroyed owl and
  0.259 for the good astrolabe, so it does not discriminate.
  Ruled out with evidence, not assumption: the vertex-colour round trip is **exactly correct**
  (GLB `COLOR_0` equals `srgb_to_linear(OBJ)` to four decimals), so there is no gamma fault.
  **Historical extraction finding:** the collapse happens in
  `C:\chronos2	ools	riposr_mesh_emitter.py::subject_mask`, where GrabCut is seeded only by
  colour distance (`distance >= 0.10`) and then its result is accepted unconditionally. A guard
  keeping the flood mask when the refinement collapses is written up in
  `docs/ledger/2026/09/plan_2026-09-12_1930_object_quality.md`; it needs an operator decision
  because it edits a sibling product. Evidence:
  `artifacts/visual-proof/quality-2026-09-12/` and `artifacts/quality-lab/`.
- **The manifestation pipeline was never broken; the capture harness was (2026-09-12):** It was
  reported here as "no stage output in 25 minutes from inside the game, 5 minutes on the CLI."
  That was wrong. `drive_manifest_capture` was registered with **no system ordering**, while
  `drive_walk_capture` is ordered `.before(InnerInteractionSet::Resolve)` — with a comment
  stating exactly why: `just_pressed` lives one frame and Bevy clears it in `PreUpdate`, so a
  synthesized press issued after `resolve_actions` has run is wiped before anything reads it.
  The harness pressed `E` **once**, lost it, never opened the prompt, never spawned Chronos2,
  and sat in `Idle` until its timeout — then printed "TIMEOUT waiting for manifestation to leave
  Manifesting phase", asserting a phase it had never reached. The evidence was already on disk
  and was misread: **no manifestation out-dir had ever been created**, which only happens if
  Chronos2 was never invoked. Fixed by ordering the harness before the resolver, retrying `E`
  while the phase is still `Idle`, and reporting the phase actually reached on timeout. A live
  in-game run now completes end to end: prompt submitted at 4.1s, every `[chronos-stage]` line
  through `complete pct=100`, `Succeeded: Manifested 'sacred celestial relic' atop the altar!`,
  the painting on the cushion with the object above it, the `[E] Take it from the altar` prompt
  live, and the content-addressed library written —
  `manifested/47cc55ca-2802-416a-b2f7-a118bbaea2c8.glb`. Evidence:
  `artifacts/visual-proof/manifest-live-2026-09-12/`, including the complete unfiltered log.
- **Objects the player owns: taken, carried, placed, duplicated (2026-09-12):** The blocker was
  never hands. Manifestation wrote one fixed path and overwrote it every run, and Bevy caches by
  asset path — so two creations in the world were two views of whatever was made last, and a
  third silently changed both. `services/artifacts.rs` gives each manifestation its own file
  (`assets/manifested/<id>.glb`) plus an append-only library and placement ledger, the third
  instance of the pattern `build_intent.rs` and `encounter_memory.rs` already use: folded from
  disk on read, and a pick-up recorded as a **withdrawal** rather than by deleting the row.
  `modes/inner_chambers/objects.rs` adds carry (a held anchor parented to the camera, no arm
  mesh), place (`F`), and duplicate (`R`, which keeps the object in hand so a room can be
  filled by pressing one key). Placement needs no physics: `castle_surface_y` already answers
  the floor height anywhere in the building and `clamp_inside_wall` answers whether something
  may be there, so an object cannot be put anywhere the player could not have walked.
  `E` at the altar now takes whatever is standing on the cushion — the altar's own handler
  returns early while something is there, so one press cannot both take the object and reopen
  the prompt. Three defects found by walking it: the pick-up range was measured in 3D from a
  2.85m eye to a floor-level object, so the vertical alone exhausted it and nothing could ever
  be picked up; the seeding tool mirrored `NeuroCognica/Archetypes` while `app_data_root()` ends
  in `data`, so it wrote a ledger the game never read; and all three duplicates landed at one
  offset, stacked inside each other. Evidence:
  `artifacts/visual-proof/objects-2026-09-12/`, whose report records
  `focus=PlacedObject` -> `carrying=yes` -> `objects_standing=4` from one pick-up and three
  copies, on a real walk.
- **Authored sound effects and a real SFX bus (2026-09-12):** Fifteen cues synthesised from
  noise, decaying partials and filtered transients by `scripts/author_sound_effects.py` — no
  sample pack, no licence, deterministic. `services/sfx.rs` mixes them at `master * sfx` times a
  per-cue trim, and the settings menu's "(no SFX bus yet)" qualifier is gone. Footsteps are
  spaced by ground actually covered *after* collision, not by a timer, so they stop when the
  player is pressed against a wall. `SfxTally` is written into the walk report because a sound
  that does not fire is indistinguishable from one that fires silently.
- **Sixty Chronos2 works hang in the chambers with provenance placards (2026-09-12):**
  `scripts/stage_museum_art.py` copies selected works into `assets/museum/` at author time —
  **nothing reads `C:\chronos2` at runtime**, because the launcher already treats Chronos as
  optional and a buyer's machine has no such directory. Of 147 run directories: 14 have no
  usable image, 7 are blank or all-black renders (a mechanical variation test, not a judgement
  about subject), 126 eligible, 60 staged — one per hanging position. Chronos2's own
  `title_from_prompt` and its `museum_wall` exclusion are carried across rather than
  rediscovered; note the exclusion had to read `manifest.json`'s `render_context.vista`, since
  `scene.inspection.json` carries no `vista` key in this library and reading only the documented
  location would have excluded nothing while appearing to work. Placards are rendered to images
  at author time and state only what the bundle recorded — 58 of 60 carry a provenance hash and
  the other 2 omit that line rather than inventing one. `scripts/author_art_frame.py` is the
  fifth kit module (652 tris): gilt moulding, backing board, a 4:3 canvas and a placard plate,
  with the canvas and placard deliberately **excluded** from the kit's world-scaled unwrap so
  their 0..1 UVs map an image onto them once. Two defects its own export check caught: the
  canvas came out 1.6x1.0 instead of 4:3 because the plane's unused Z axis was being scaled
  instead of its Y, and both image planes faced *into* the wall — `R_x(-90)` sends the normal to
  +Y, so every painting was backface-culled and each frame showed its own backing board. A third
  was caught by a new test: the back-wall hanging's yaw had `+ PI` added to it, facing it into
  the masonry. Evidence: `artifacts/visual-proof/museum-art-2026-09-12/`.
- **Twelve arches are now doorways into walkable chambers (2026-09-12):** Per
  `docs/ledger/2026/09/plan_2026-09-11_2130_museum_arches.md`, 12 of the ground gallery's 72
  bays (every 6th, 30 degrees apart) open into rooms cut 9m into the 12m wall, leaving 3m of
  masonry behind each. `castle.rs` gained `MUSEUM_BAYS`/`museum_bay_bearing`, a chamber-shaped
  exemption in `clamp_inside_wall`, `museum_chamber_surface_y` (without which the chambers were
  a hole in the world — `gallery_surface_y` only answers between the gallery radii, so a player
  who walked through an arch would have fallen to the abyss), and `museum_hanging_positions`.
  **The exemption stays conditional**: a test walks every one of the other 60 bays and asserts
  the wall still stops the player. A second test pins that strafing into a chamber's side wall
  clamps to that wall rather than ejecting the player back into the hall — the shape of bug
  that survived for weeks in the room walls. `scripts/author_museum_chamber.py` is the fourth
  kit module (1,632 tris, 115KB): floor, two hanging walls with plinth courses, back wall, and
  a barrel vault springing from the same 5.5m line as the arch outside, so the vault inside and
  the arch head outside are one piece of geometry. Two defects found by walking it: the bay
  module's near-black `Bay_BlindPanel` is a wall across a real opening and is now hidden at
  those twelve bays, and the gallery sconces sat on `index % 6` — exactly the museum stride —
  so every chamber had a glowing sphere hanging in its doorway. A third found on the way: the
  player could walk around with the settings menu open, because W/S both navigate the menu and
  drive locomotion. Evidence: `artifacts/visual-proof/museum-arches-2026-09-12/`, whose walk
  report records a real walking player crossing the 114m wall face to 118.42m inside a chamber.
- **The settings menu exists (2026-09-12):** Specified in full in
  `GAMEPAD_AND_SETTINGS_GUIDE.md` and never built; two prior units left it. `Esc` (or `B`, or
  `Start`) now opens it — the HUD had been advertising "Esc/B: Menu" the whole time `Esc`
  actually ejected the player out of the castle, so the label was lying and a mis-hit dumped
  you out of the world. Leaving is now an explicit `Leave the Inner Castle` row, and
  `extraction.rs` watches that request rather than the key, so the key that opens the menu can
  never also close the mode in the same frame. Ten rows: seven sliders, reset, resume, leave;
  keyboard and gamepad; every slider's range pinned by test against `GameSettings::clamped` so
  a value the menu allows cannot be silently clamped away on the next load. **Two of the four
  volume sliders now reach real audio**: music was a hard-coded 0.22 that ignored the setting
  and is now `master x music` against that reference mix, applied live rather than on next
  launch; Council voices likewise. There is no sound-effects bus in the castle at all, so that
  row is labelled `Sound effects volume (no SFX bus yet)` in game rather than pretending — a
  test holds the qualifier until a bus exists. Also fixed: the menu was first drawn with block
  and arrow glyphs the bundled font does not carry, which render as empty boxes (the same
  defect is visible in the HUD's own decorative marks in every screenshot this project has
  taken); every string it renders is now ASCII and pinned by test. Evidence:
  `artifacts/visual-proof/settings-menu-2026-09-12/`, whose walk report shows `Esc` opening the
  menu with `mode_state=Navigating` (so it did not eject), four Down presses landing on
  `Ambient music volume`, and the value moving 0.80 -> 1.00 and **coming back at 1.00 on the
  next run**, which is the persistence proving itself end to end.
- **Authored stone across the kit, world-scaled (2026-09-12):** The castle was UV'd but flat
  colour everywhere, which is the whole of the "looks fake" complaint. Two prerequisites, both
  now done. (1) **World-scaled UVs**: `author_arcade_bay.py` and `author_stair_flight.py` now
  cube-project at 4m per tile instead of `smart_project`, which packs islands into unit space
  per object and has no physical scale at all — the bay script now fails its own export check
  if the corona's U span stops matching its width in tiles. (2) **Authored seamless tiles**:
  `scripts/author_stone_tiles.py` writes eight stones as albedo + normal + roughness, periodic
  by construction (wrapped value-noise lattice, wrapped Worley deltas, whole numbers of block
  courses) rather than healed at the edges — every tile's measured seam step came out *below*
  its own interior step. The operator's reference sheet was correctly diagnosed as unusable as
  an asset: one 1254px image holding nine stones is ~38 px/m on a 9.9m bay. Normal maps are the
  part that matters — the hall casts no shadows, so surface normals are the only thing carrying
  relief. One bay GLB serves all seven storeys; `modes/inner_chambers/stone.rs` swaps the
  material per level by authored node name, and deliberately leaves `Bay_BlindPanel` near-black
  so arches keep reading as openings. Floor paving UVs moved from an arbitrary 0.1 to the same
  4m tile. Tests: **214 passing**. Evidence: `artifacts/visual-proof/stone-2026-09-12/` and
  `stone-tiles-2026-09-12/`.
- **The painted image now sits under the object it became (2026-09-12):** Operator: "The
  manifester is supposed to create a 3d object and place the painted image underneath." It was
  spawning the panel at `p.z - 2.8` — beside the object, not under it; the commit that added it
  says "beside" in its own subject line. Centring the altar this morning made that placement
  actively wrong as well, landing the panel on top of the spinning vortex disc. The panel now
  lies flat on the cushion with the object directly above it. Three further defects found while
  proving it: `MANIFESTATION_CUSHION_HEIGHT` was 1.62 against a cushion actually built at 1.34;
  `MANIFESTATION_HOVER_Y` was an **absolute** world y of 2.40, so while the altar stood on a
  tabletop at 3.17 the idle diamond, hourglass and failure X all floated 2.1m *below* their own
  cushion, inside the furniture; and the object spawned at scale 1.15, taking a 1.4m authored
  mesh to 1.61m on a 1.60m cushion, so every manifestation overhung its own altar. All three are
  now derived rather than hard-coded. Evidence:
  `artifacts/visual-proof/manifest-under-2026-09-12/`.
- **The arcade ring floors were being culled away, not merely dark (2026-09-12):**
  `build_radial_flagstone_mesh` emitted its stone tops and both rim faces with reversed
  winding. Bevy culls by winding, not by the normal attribute, so every stone top in the castle
  was invisible from above while still shipping a `+Y` normal claiming otherwise — the normal
  data lied about the geometry, which is why this survived a stone-colour pass, a height pass
  and a joint-width pass without being found. Exactly 3 of the 5 quads per stone were affected
  (768 of 1280 triangles at deck resolution): top, outer rim, inner rim. Only the two radial
  joint faces were wound correctly, so a player standing on a gallery saw 0.55m joint walls
  every 5m with the void showing between them. The Council floor concealed the same defect
  because it has a solid slab cylinder underneath for the culled paving to show through to.
  Fixed by winding those three faces to match the normals they ship, and pinned by a test that
  compares every triangle's geometric winding against its own normal attribute rather than
  trusting either alone. Evidence: `artifacts/visual-proof/gallery-deck-2026-09-12/`.
- **The table is gone; the vortex lies in the floor (2026-09-12):** Operator directive was
  "the table goes away. only the animated spinning disk gets put on the floor. the manifester
  sits on the floor in the middle of the spinning disk. there is no table." The 2026-09-12
  00:08 unit had read the earlier handoff as *restore the table* and put the altar on a
  tabletop at y=3.17; that reading was wrong and is now corrected. A new Blender module
  `scripts/author_portal_disc.py` exports `assets/scenes/portal_disc.glb` carrying exactly one
  node, `Stargate_Portal` (142 tris, 7.2m across, flat, UV'd) — authored standalone rather than
  spawning `table.glb` and hiding its other meshes, so nothing ships as an invisible mesh.
  `chamber/portal.rs` is unmodified and still finds it by name. The disc lies at
  `GROUND_Y + 0.008` (the 8mm clears the paving; coplanar would z-fight) and the altar stands
  at `(0, GROUND_Y, 0)` in the middle of it. Collision dropped from the old 2.65m table
  silhouette to a 1.45m altar footprint, and the altar's interaction anchor moved to its cushion
  — measuring 3D distance from a 3.25m eye down to a floor-level base would have spent almost
  the whole 3.2m range climbing. Also fixed here: the Council paving carried *both* defects the
  gallery decks were fixed for on 2026-09-11 and had never been checked — its joint was an
  angle (0.055 rad = a **1.32m hole** at the 24m radius, which is the radial spoke pattern in
  the operator's screenshot) and its stones sat 0.062m proud of the collision surface. Tests:
  **204 passing** (180 engine, 19 launcher, 5 windows identity), up from 201. Evidence:
  `artifacts/visual-proof/portal-disc-2026-09-12/` and
  `artifacts/visual-proof/portal-disc-walk-2026-09-12/`, whose `walk_report.txt` records a real
  walking approach stopping at 1.57m with `focus=ManifestationAltar`, and a frame taken from
  3.23m — on the disc — looking down at the altar. Geometry and reachability proof; not
  aesthetic approval.
- **Rotunda heart restored (2026-09-12):** The live Seed-of-Life castle now spawns the authored
  `table.glb` at the Council centre instead of the procedural inlay stand-in. Its real child
  `Stargate_Portal` is again found and animated by the already-registered `PortalPlugin`. The
  manifestation altar is centred directly on the measured portal plane (table feet local
  `z=-0.766`, disc local `z=0.300`, 2.6x scale, current floor y=0.4), so the blue vortex remains
  visible as a ring beneath the altar base. A visual inspection caught the first collision
  radius covering only the altar, which let a walking player enter the wider Council table; it
  now covers the complete 2.6m-scale table silhouette. Source regressions pin the shared altar
  and collision position. Tests: 201 passing; Desktop/Taskbar installed build restaged and
  hash-verified. Evidence: `artifacts/visual-proof/rotunda-heart-2026-09-12/` (the free-flight
  `01_council_floor_inlay.png` establishes the portal/altar geometry) and
  `artifacts/visual-proof/rotunda-heart-walk-2026-09-12-final/` (the walking report establishes
  collision at `(0.00, 3.25, 2.65)` and `focus=ManifestationAltar`). The close walking frame is
  interaction/collision evidence, not aesthetic approval.
- **Satellite rooms and non-Jester figures hidden (2026-09-11):** Per operator directive, the six
  satellite archetype rooms (platforms, cobblestone walls, doorways, bridges over the abyss, room
  furniture, drafting bench, and threshold lights) and the central AURA figure are cleanly hidden,
  preserving the Jester as the active host in the Council circle (`(10.2, 0.42, 6.2)`). All underlying
  code and assets remain 100% intact (Rule 1: Never delete) for potential future relocation into
  arcade archways. Character obstacle colliders were adjusted so no phantom collision blocks the
  open floor, while `canonical_room_figure_obstacles()` preserves the regression contracts.
  Both `ARCHETYPES_INNER_CAPTURE` and `ARCHETYPES_WALK_CAPTURE` were executed on the installed
  release build, producing 17 verified 4K frames and a complete walking log confirming smooth ground
  locomotion, Jester presence, and stair climbing. Evidence:
  `artifacts/visual-proof/rotunda-clean-2026-09-11/` and `rotunda-walk-2026-09-11/`.
  Tests: 199 passing; installed Desktop/Taskbar build synced.
- **HANDOFF to the next builder (2026-09-11):** The previous builder stood down; the next unit of
  work is written up in full at
  `docs/ledger/2026/09/plan_2026-09-11_2200_handoff_rotunda_heart_and_stone.md`. It covers five
  operator asks — stone textures varied by arcade level, the main floor and rotunda fixed, every
  character mesh removed except the Jester, the old table's spinning `Stargate_Portal` effect
  reconnected, and the manifestation altar moved to the centre so the vortex turns beneath it.
  Two findings worth knowing without reading the plan: the portal effect is **not broken** —
  `chamber/portal.rs` works and `PortalPlugin` is registered, but `table.glb` is only spawned
  inside `setup_legacy_rotunda_world`, which is `#[allow(dead_code)]` and never runs, so the disc
  never enters the live castle. And removing the six room figures removes the only entities
  carrying `ArchetypeEmbodiment`, which is what the conversation system targets — that needs an
  operator decision, not a silent deletion.
- **Stair flight kit module (2026-09-11):** The perimeter ascent is now a Blender module, two
  instances per storey. `scripts/author_stair_flight.py` builds a real flight — stepped treads
  with a nosing, a solid raked soffit under them, a parapet **on both sides** with a coping
  rail, and the landing at its head — genuinely curved at the 108m radius rather than modelled
  straight. 380 triangles, 25KB. The ascent dropped from ~602 entities to 14. A tread cannot
  carry a stringer, a parapet or a handrail, because all three run continuously along a flight
  and a box only knows about itself, which is why the flight is the repeat unit.
  To let one module serve every storey, `gallery_y` is now measured from the promenade
  (`PROMENADE_Y + (level+1) * GALLERY_RISE`): anchoring the first gallery at a fixed 12.0m while
  the promenade sat at 0.5 left the ground flight climbing 11.5m in the same 72 risers every
  other flight used 12.0m for — 159.7mm downstairs, 166.7mm above. Every flight now rises
  exactly 12.0m, and the top bay lands exactly on the wall head at 96.0m.
- **Gallery paving was floating 0.55m above its own collision surface (2026-09-11):**
  `build_radial_flagstone_mesh` raises its stones above the mesh origin, and the decks were
  placed with that origin *at* the walking height. Players crossed every gallery buried to the
  shin in their own floor, sighting along it through the joints — which is what the dark banding
  in the operator's screenshots was. Decks and the promenade are now dropped by the stone
  thickness. Separately, the mesh takes its joint width as an *angle*, so the rescale had turned
  a mortar line into a 1.02m hole between every flagstone; joints are now specified in metres
  via `joint_angle`.
- **Perimeter stone repair + black arch interiors (2026-09-11):** The gallery decks, ground
  promenade and every stair tread were sharing the dark basalt floor material (base 0.34) while
  the Blender arcade standing on them is pale limestone (0.62) — roughly double the value — so
  from any gallery the walkway read as a black slab hung under a pale building. They now use a
  matched `pale_stone`, and cornices/rails a non-metallic `pale_trim` (they were 0.25 metallic
  brown, reading as copper pipework). The arch recess went to near-black so the arcade reads as
  openings rather than panels; the spandrel above each arch went back to masonry, since only the
  opening is a hole. Evidence: `artifacts/visual-proof/stone-and-arches/` and
  `stone-and-arches-walk/`. Still primitives: the stair's open side is a raw stepped silhouette
  with no stringer, and the room drums are flat-coloured cylinders.
- **Museum arches planned (2026-09-11):** The building has **504 arches** (72 bays x 7 storeys,
  `castle::total_arches()`, pinned by test). Plan to make 12 of them walkable exhibition
  chambers hanging real Chronos2 works with provenance-backed placards:
  `docs/ledger/2026/09/plan_2026-09-11_2130_museum_arches.md`. Art source verified as 151
  first-light bundles in `C:\chronos2\outirst_light`, to be staged into the repo rather
  than read from a sibling product at runtime.
- **Painted vault fresco (2026-09-11):** The ceiling is now an operator-supplied circular
  painting on a real saucer dome. `scripts/author_vault_fresco.py` builds a spherical cap with a
  114m base and 30m rise (cut from a 231.6m sphere) and maps the image with polar UVs projected
  orthographically from below, so the painting's border lands exactly on the springing circle and
  its centre burst exactly on the crown, with no wrap and therefore no seam. This replaced a
  procedural `Cone` — whose UVs run around its lateral surface and would have smeared a circular
  painting into a spiral — and the 24 rib bars that cut across it. Source image lives at
  `assets/textures/vault_fresco.png`; module at `assets/scenes/vault_fresco.glb` (5,280 tris).
  UVs are inset to 97.5% because the PNG's transparent corners carry undefined RGB that sampled
  as coloured speckle along the springing line. Evidence: `artifacts/visual-proof/vault-fresco/`.
- **Castle architecture moves to a Blender kit (2026-09-11):** The perimeter arcade is no longer
  procedural Bevy primitives. `scripts/author_arcade_bay.py` authors one arcade bay headlessly
  in Blender 4.5 — pier with plinth/shaft/capital, a real semicircular voussoir arch plus
  archivolt, spandrel, string course and corona — bevelled, UV'd, 2,268 triangles, 151KB, and
  exported to `assets/scenes/arcade_bay.glb`. The engine instances it 72 times per storey across
  7 storeys; `castle.rs` remains the placement layer and gained `arcade_bay_width/bearing/
  position` and `wall_module_yaw`. The script and `castle.rs` both carry the 9.9452m chord and
  each fails independently if the other drifts. Why: an un-bevelled `Cuboid` catches no
  highlight, has no UVs for stone texture, and cannot be an arch — which is what an arcade is
  made of. Evidence: `artifacts/visual-proof/arcade-bay-kit/`. This is the first module; stairs,
  balustrades, vault severies and room drums are still primitives.
- **Architect workshop + castle movement repair (2026-09-11):** The Architect's room has a real
  drafting bench (`world.rs`) that opens a plan workshop: create, sequence, complete, stall and
  close player-owned `BuildIntent` artifacts (`services/build_intent.rs`), every write shown
  verbatim on a confirm screen first, append-only and folded from disk on each read. Getting
  there required fixing four defects the audit found first: (1) the room-wall collision ejected
  any **walking** player ~8.5m backwards the moment they passed a room's centre, making every
  archetype figure and every piece of back-of-room furniture reachable only by flying — it is now
  a shell with real thickness; (2) the Architect and Empath figure colliders were transcribed
  mirrored, 10m onto the wrong side of their rooms, and are now derived from the spawn formula;
  (3) three systems raced to write the single hint line and the proximity prompt always lost — a
  `HintRequest` priority resource now owns it; (4) `Esc` could close a modal *and* exit the mode
  in the same frame — an `InnerActions`/`InnerModalState` arbiter now resolves context keys once
  per frame. A new `ARCHETYPES_WALK_CAPTURE` harness proves all of it by **walking** and pressing
  real keys (the old harness flies and teleports, which is why the movement bug survived).
  Evidence: `artifacts/visual-proof/architect-workshop-2026-09-11/`.
  Plan: `docs/ledger/2026/09/plan_2026-09-11_0035_architect_workshop.md`.
- **Smash Room redesign retired (2026-09-11):** `Game Plan_ Archetypes — The Inner Chambers
  Smash Room.md` (the 2026-09-08 physics/prop-destruction redesign) is marked SUPERSEDED by
  operator decision. It was never built (no physics crate, no `WorldProp`/`SmashSim`/cards
  modules), and it directly contradicted every load-bearing choice actually shipped since:
  flight locomotion was kept, archetypes remained embodied conversational partners (now with
  a Remember/Forget memory system), and the manifestation altar generates 2D concept art, not
  smashable 3D props. Do not resume work from that document without a fresh operator decision.
- **Gamepad input + persisted settings (2026-09-11):** Real Xbox/gilrs input (confirmed
  against a physically connected controller) now drives Inner Castle movement, look,
  jump/fly, and interact/cancel alongside keyboard/mouse (`services/gamepad_input.rs`,
  wired into `modes/inner_chambers/camera.rs`, `encounters.rs`, `manifestation.rs`). A new
  `GameSettings` resource (`services/settings.rs`) persists sensitivity/deadzone/volume to
  `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`. Still open: an in-game
  settings menu screen and wiring volume to actual audio buses; voice/STT steps of
  `docs/ledger/2026/09/plan_2026-09-10_2350_gamepad_voice_runtime.md` untouched.
- **Inner Castle consentful encounter memory (2026-09-11):** Encounters no longer log
  unconditionally as recallable memory. Every turn opens `Transient`
  (`services/encounter_memory.rs`, an append-only ledger-sealed journal); the player presses
  F5 Remember / F6 Forget / F7 View record, and only remembered turns are recall-eligible,
  recomputed fresh from disk every time. A Forget is an auditable withdrawal, not a delete.
  Steps 2-9 of `docs/ledger/2026/09/plan_2026-09-11_0015_inner_castle_grounded_capabilities.md`
  (Oracle archive, Mentor reading room, Architect workshop, local STT) remain PENDING.
- **Inner Chambers archetype niche chambers (2026-09-10):** The five standing archetype
  figures (Sentinel, Aura, Empath, Oracle, Nebula Jester) previously stood in open floor space
  with only per-figure point lights. Each now has a real niche bay: a curved wall ring
  (`build_wall_ring_mesh`) with a doorway gap that always faces the rotunda center, a tinted
  floor medallion and low canopy giving a ceiling silhouette distinct from the 22m main vault,
  and two flanking threshold pillars — stone tint derived per-archetype, not decorative. The
  outer castle walls also gained repeated pilasters and a cornice band breaking up the flat
  slabs (purely visual; collision unchanged). Caught and fixed a real geometry bug in the
  process: the first 2.9m niche ring radius overlapped adjacent niches by ~0.13m (the five
  bays sit only ~5.67m apart center-to-center); shrunk to 2.1m and added a regression test
  asserting >=1.0m clearance between every pair. Added a new `ARCHETYPES_INNER_CAPTURE`
  self-driving screenshot harness (mirrors the existing Mecha capture pattern) since there was
  previously no reproducible way to get a rendered-frame proof of Inner Chambers without a
  human at the keyboard; it caught two more framing bugs (a too-early boot-veil trigger, and a
  camera stand-off formula that put the Empath shot inside its own wall) before they shipped.
  Still open: the outer wall is still 4 flat slabs, not a true circular drum, and the Step 5
  multiview-reconstruction evaluation from the same plan was not touched this session. Plan:
  `docs/ledger/2026/09/plan_2026-09-09_2220_inner_chamber_architecture_and_multiview.md`.
- **Inner Chambers Castle Rotunda & Animated Council Table (2026-09-09):** Inner Chambers was overhauled from an empty open void into a massive enclosed castle great hall ($76\text{m} \times 76\text{m}$, $22\text{m}$ high). Features a dark polished stone floor at $y = 0.0$, a two-tiered raised stone dais ($R = 6.0\text{m}$ step, $R = 4.8\text{m}$ platform at $y = 0.30\text{m}$), 4 solid enclosing perimeter walls with stone buttress pillars, an enclosed vaulted ceiling with dark iron/timber cross-beams, 8 perimeter wall sconce braziers, and dual overhead spotlights. Centered on the dais is the animated Flower-of-Life Council Table (`assets/scenes/table.glb`, scale 2.6, origin $y = 2.34\text{m}$) with its rotating, pulsing `Stargate_Portal` cyan vortex. The player spectator camera spawns elevated looking directly at the table with smooth 6DOF flight (WASD fly, Space rise, Shift/C descend, Mouse 360° look). All legacy placeholder primitives (cubes, lines, spheres) were purged from the room. Fixed table visibility gating in `crates/engine/src/chamber/camera.rs` so exclusive modes never hide non-ritual scene elements. Verified with 100/100 workspace tests and staged to Desktop via `pwsh -File scripts\install_shortcut.ps1`.
- **NeuroCognica Start Menu identity + family icon (2026-08-25):** Archetypes now installs one flat
  `Programs\NeuroCognica\Archetypes.lnk` beside ChronoSophia2, NC Company Database, EOAI-MGS and Chirox, per
  `C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md`. It previously wrote three top-level shortcuts (`Archetypes`,
  `Archetypes Help`, `Uninstall Archetypes`) outside the family folder, and the Start Menu entry pointed at the
  developer checkout `C:\archetypes\dist\launcher.exe` while the product was installed under
  `%LOCALAPPDATA%\Programs\Archetypes`. Help now lives at `help\index.html` in the install root and uninstall is
  Add/Remove Programs; neither gets a Start Menu entry.
  The product icon is the green NeuroCognica family mark (six white dots on an opaque green plate, all seven Windows
  sizes), generated by the brand kit and committed at `assets/icons/archetypes.ico`. `install_shortcut.ps1` used to
  rasterise the old architect glyph into a single 256px `.ico` over both the dist copy **and** the repo copy on every
  restage, so the real mark could not be committed; that block is gone and the committed file is copied instead.
  `crates/engine/build.rs` and `crates/launcher/build.rs` now emit `cargo:rerun-if-changed` for the icon — without it
  cargo never re-ran the build script, and a corrected `.ico` left a stale mark embedded in the exe.
  Verified: `cargo test -p launcher --test windows_identity` (5 passed), `cargo build --release --workspace`,
  `scripts\install_product.ps1` run against the live install (legacy shortcuts removed, family shortcut created),
  and `C:\NeuroCognica_Brand\scripts\check_start_menu_family.ps1` reports Archetypes resolving with a 7-size icon.
  Still open: Archetypes has no signed single-file installer. `install_product.ps1` is an operator install and the
  binaries are unsigned, against NeuroCognica doctrine that every versioned executable is signed on compile.
- **Honest RC (2026-08-16):** Version **0.3.0**. Sentinel `--strict` certify **PASS**; `Certification readiness: candidate` (not certified — key lifecycle and release signing remain open). HKCU Add/Remove Programs `Uninstall\Archetypes` proven (DisplayName, DisplayVersion 0.3.0, UninstallString, DisplayIcon) at per-user `C:\Users\m\AppData\Local\Programs\Archetypes`; uninstall without `-RemoveAppData` removed ARP/binaries/shortcuts and kept Witness AppData. Live click surface: Desktop `Archetypes.lnk` → `C:\archetypes\dist\launcher.exe`. Launch evidence: `artifacts/visual-proof/honest-rc-2026-08-16_1239/` (black veil → menu; `last-failure.txt` absent after success). Council TTS is CLI + WAV cache; in-process sherpa C API is opt-in only (`ARCHETYPES_TTS_CAPI`) because the pinned ORT 1.17.1 sidecar AVs the engine. Sidecar Director start now passes `--sentinel-mode enforce`.
- **Product depth (2026-08-16):** Version 0.3.0. In-engine Sentinel mediation covers chat, artifacts, ledger/world memory, and Witness profile. The launcher starts installed Ollama / Chronos Director / ComfyUI when they are down. CouncilSpeaking flies to a hexagram axis and crosses into that archetype's interior law. Inner Chambers is a seven-mind hub. Accepted artifacts persist as world-memory tokens on the table.
- **Product rebuild (2026-08-16):** Desktop Standard Mode is the AURA council ritual again. Consciousness is the former Mecha 1:1 chat, with ledger id `consciousness`. Inner Chambers is a playable Architect blueprint interior (not cubes). Living Engine is a playable 3-sphere metabolic prototype with tested sim. HELP is an in-game overlay plus `assets/help/index.html`. Launcher fail-visible covers duplicate instance, Ollama, TTS, Chronos, and engine miss; TTS repairs from `dist/scripts/dependencies.json`. Comfy output is no longer hardcoded to one operator profile. Install/uninstall scripts exist (`scripts/install_product.ps1`, `scripts/uninstall_product.ps1`). Version 0.2.0.
- **Sentinel launch gate (2026-07-20):** Desktop launch still requires Chronos Director `readiness: ready` with Sentinel authority in `enforce` mode. `ARCHETYPES_ALLOW_WITHOUT_CHRONOS` remains absent.
- **Seed-of-Life Standard Mode UI (2026-07-20):** Operator plate is canon. Selector is pure black with spaced gold `A R C H E T Y P E S`, hairline+glint, and seven thin gold rings (Sentinel center; Architect/Mentor/Explorer/Oracle/Empath/Jester around). Mecha `uxbacklayer` is no longer used in Standard Mode. Chat/switching use the same black + gold hairline language and Cinzel (`assets/fonts/Cinzel-Regular.ttf`). Canon ref: `assets/standard_mecha/canon/seed-of-life-selector.png`.
- **Maximized window / hidden console / taskbar icon (2026-07-20):** Release `engine.exe` and `launcher.exe` use `windows_subsystem = "windows"` (no console flash). Engine Startup calls `Window::set_maximized(true)`. Both exes embed `assets/icons/archetypes.ico` via winres; Desktop/Start Menu shortcuts point at the same `.ico`. Launcher also passes `CREATE_NO_WINDOW` when spawning the engine. Desktop restaged via `scripts\install_shortcut.ps1`.
- **Mecha chat scroll (2026-07-20):** Channel transcript now scrolls with mouse wheel/trackpad (Bevy hover map → `ChatUiScroll` observer) and click-drag. Auto-jump to bottom only on new history turns so manual scrolling is not fought during wait refreshes.
- **Mecha multi-turn chat UI (2026-07-20):** Second+ questions already produced Ollama text and Comfy images in JSONL, but the channel UI failed to show them. Idle wait-tick was dirtying `ChatBridge` every frame (constant transcript rebuild / stuck image alpha), there was no auto-scroll to the newest turn, and Desktop restage wiped `assets/standard_mecha/renders`. Fixed fingerprint-gated rebuilds, scroll-to-bottom, render-folder preserve in `install_shortcut.ps1`.
- **Boot crash fix + launcher errors (2026-07-20):** Desktop launch was dying immediately with Bevy `B0001` (conflicting `Visibility`/`Camera` queries: boot gating vs camera systems). Boot world hide/reveal is now owned solely by `camera::gate_boot_and_table_visibility`. The launcher captures engine stdout/stderr to `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\last-engine.log` and on failure writes `last-failure.txt` and opens it in Notepad so Desktop launches are never silent.
- **Boot veil / chat wait / desktop dist (2026-07-20):** Fixed a first-frame flash where the chamber-backed menu world could appear before the pure-black title veil (ClearColor was forced to ceremonial navy during Booting; lore chamber now starts `Visibility::Hidden` and the Witness camera is inactive until boot ends). Standard Mecha chat shows an honest `WORKING (Ns) — …` banner above the input plus matching status/transcript lines while Ollama and Chronos/Comfy run. Portraits use fixed native-aspect pixel boxes; Comfy artifacts stay inline in the left channel. **Root cause of “changes didn’t show on Desktop”:** Desktop `Archetypes.lnk` launches stale `dist\launcher.exe` — source edits from 2026-07-19 were never staged. Rule 13 in `AGENTS.md` and `.cursor/rules/desktop-launcher-surface.mdc` now require `scripts\install_shortcut.ps1` after every player-facing change.
- **Mecha chat image aspect + in-channel artifacts (2026-07-19):** Standard Mecha chat no longer stretches sidebar portraits or Comfy renders into forced boxes. Portraits keep native aspect (`1024x1536` tall / `1024x1024` square) via `NodeImageMode::Auto` and per-archetype `portrait_aspect`. Completed Chronos/Comfy artifacts are embedded inline under each archetype reply in the left channel (scrollable transcript rows) for every archetype; the right panel keeps the portrait, bio, and short image/service status only — no separate “LATEST COMFY RENDER” strip. Soft reveal targets the newest in-chat artifact.
- **Chronos play reliability + chamber feel (2026-07-15):** The launcher fail-closes unless Ollama, offline TTS, Chronos Director (`readiness: ready` on `:7777`), and Comfy (`/system_stats` on `:8000`) are all up; as of 2026-07-20 the old `ARCHETYPES_ALLOW_WITHOUT_CHRONOS` debug flag is ignored for launch safety. Engine concept-thumbnail calls verify Ollama VRAM unload before painting. The lore main menu restores Oracle Riddle as a playable entry (no STANDBY lie), shows a live Ollama/Director/Comfy footer, and Standard Mode chat shows the same service truth, phased wait copy (Ollama answering → Chronos painting), soft image reveal, player-facing image status without artifact/proof jargon, and Esc from the consciousness selector back to the lore menu.
- **Standard chat Comfy response images (2026-07-15):** Standard Mode is exposed again from the default Bevy main menu. Every new submitted archetype-chat statement now extracts local keywords, sends the archetype response through real local Ollama, sends the image request through the real Chronos Director `concept-thumbnail` / ComfyUI lane with that archetype's hard-coded signature art style, displays the returned image inline in the chat channel, and persists the turn to both per-archetype JSONL history and the global hash-chained ledger. Failures are visible and recorded; no placeholder image is substituted. The main menu also has a real `QUIT GAME` option that sends Bevy `AppExit::Success`.
- **Lore chamber launcher integration (2026-07-15):** Desktop launch now shows the black intro (`ARCHETYPES`, then `A GAME BY MICHAEL HOLT`) and fades into a new lore-compliant council chamber GLB at `assets/scenes/lore_chamber.glb`. The chamber is generated by `scripts/author_lore_chamber.py` from repo canon: the seven council seats use the real Archetype ids/theme colors from `crates/engine/src/theme/constants.rs`; Codex/Lexis and Viren remain real theme nodes but not seven-seat council members. The default main menu is chamber-backed, launches Standard Mode and Oracle Riddle, keeps Inner Chambers / Living Engine locked, and includes `QUIT GAME`. The rejected legacy chamber/table/portal/sky/star stack remains preserved behind `ARCHETYPES_LEGACY_CHAMBER`, not the default desktop path.
- **Native Mecha Standard Mode (2026-07-15):** The native Bevy Mecha-style Standard implementation under `crates/engine/src/modes/standard_mecha/` is now the active Standard path, with mirrored 19 Mecha assets under `assets/mecha/`, a Rust archetype registry, per-archetype chat persona and art style, real local Ollama chat, Chronos/Comfy response images, and real JSONL history under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\standard_mecha\chat_history\`.
- **Fresh Standard Mode handoff (2026-07-15):** A cold-start handoff for the next instance now lives at `docs/ledger/2026/07/handoff_codex_2026-07-15_standard-mode-mecha-main-menu.md`. It points the next agent at the current base commit, required repo-law reads, Mecha source-canon audit, Standard Mode rebuild study, main-menu visual target, implementation scope, safety fences, and verification gates for the Mecha-style Standard Mode rewrite.
- **Risk-based verification rules (2026-07-15):** Root `AGENTS.md` now distinguishes Rust/runtime gates from docs-only and asset-only gates. `cargo test --workspace` remains mandatory for Rust/runtime/user-facing behavior and mixed high-risk changes, but docs-only/status-only and asset-only work may use targeted proof instead of paying the full Rust test tax. The same change also clarifies that planning is one plan per meaningful work unit, not one plan per search/status read, and explicitly preserves the operator's preference for more collaboration/reporting rather than quieter execution.
- **Mecha Standard Mode source-canon audit (2026-07-15):** `C:\mecha\aura-mechanician\frontend\src` has been fully inventoried and classified as the source canon for the Standard Mode rewrite. The audit is `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`. It records all 36 files, all 19 image assets with dimensions, the splash/selector/chat/theme/history/health/provenance behavior, known source bugs, and the upstream/downstream safety boundary. No game code changed in this audit unit; the next implementation unit is a hard-coded Bevy Standard Mode that lets the player chat with archetypes in the Mecha visual/behavioral language while preserving the old Standard Mode as reference.
- **Standard Mode rebuild decision (2026-07-15):** The operator supplied a fresh desktop screenshot that invalidates the current Standard Mode visual direction. This is not a tuning-only problem. The current scene is split across authored GLBs, runtime-generated star geometry, separate top-level portrait panel roots, camera-facing billboards, and camera swing choreography. Result: a giant flat blue star/crystal blocks the council, archetype portraits read as loose pasted rectangles, the spheres are dark and detached, the starfield reads as white noise, and the table/chamber no longer grounds the experience. The active plan is now `docs/ledger/2026/07/plan_2026-07-15_0304_standard-mode-rebuild-study.md`: rebuild from ground/sky first, then star chamber, then table, then archetype chambers. The current desktop visual is a rejected baseline, not an accepted milestone.
- **Inner Chambers safety gate (2026-07-15):** The uncommitted Lane B work was not accepted as a playable mode. The code is now parked behind the existing locked menu entry while its unsafe lifecycle was corrected: exit cleanup runs when entering `InnerChambersState::Exiting`, chamber/table reload reuses the canonical table transform, and extraction requires proximity to an Architect truth node instead of accepting `E` anywhere. New pure tests cover node proximity and ledger payload shape. This is not a visual approval and was not desktop-staged as a new playable mode.
- **Lane 0 (Spine) implemented and audit-corrected (2026-07-13):** The foundation for multilane modes is laid down. Introduced a real `GameMode` registry and selector. Standard Mode is playable, Oracle Riddle became playable in Lane A, and Inner Chambers / Living Engine remain visible as locked future contracts only. Extracted domain services (`services::{llm, chronos, paths, ledger}`), centralized app-data paths, and wired the Standard Mode loop to seal profile/offering/artifact events into the local hash-chained JSONL ledger. `WitnessProfile` is now public with `mode_stats` and `arrow_signal` accumulator. Added `Difficulty` type for visual clue complexity.
- **Oracle Riddle fairness correction (2026-07-14):** The riddle prompt pool no longer uses abstract/impossible triples such as `Signal Ash Garden`. Rounds now use concrete visual clues, tell the player to guess any order, score with exact/alias/semantic matching, fall back to lexical scoring if embeddings are unavailable, and award player-facing Insight tiers (`Perfect Read`, `Clear Read`, `Partial Read`, `Faint Echo`) sealed into the Oracle ledger payload.
- **Intro changed (2026-07-14):** The runtime startup no longer plays the `blackflame`/candle video frames or audio. Boot is now a plain black ceremonial veil: `ARCHETYPES` fades in first, `A GAME BY MICHAEL HOLT` fades in after it, the veil holds long enough to feel deliberate and to let the chamber load, then it slowly fades into the mode selector.
- **Desktop launch truth (2026-07-14):** The Desktop shortcut launches `C:\archetypes\dist\launcher.exe` with working directory `C:\archetypes\dist`; it does not run directly from the live repo. If desktop visuals do not match `main`, rerun `scripts\install_shortcut.ps1` after the source/build is verified so `dist\engine.exe`, `dist\launcher.exe`, and `dist\assets\` are refreshed.
- **Canon alignment (2026-07-13):** Reconciled `theme_codex()` to use the Lexis name and perfect fifth C-G harmonic signature, and verified Viren Flamebearer obedience to the Ember Covenant timing.
- **Phase 0:** Repository and dual-binary Rust workspace initialized with NeuroCognica governance protocols.
- **Chamber prototype:** The engine loads `assets/scenes/uiscene1.glb`, exported from the isolated Blender working copy `uiscene1.codex-temple.blend` without modifying the operator's original. It contains an enclosed basalt temple and vault, altar rings, a fixed cyan/magenta glass star tetrahedron with warm metal edges, glTF-compatible authored lighting, the Witness, and seven enlarged council vessels locked to the star's tips.
- **Archetype vessels (new 2026-07-12):** All seven council spheres use deliberately art-directed translucent glass. Each encloses a fixed double-faced icon/portrait panel built from repository artwork. Panel roots face the Witness camera with world-up locked, so they remain upright and never spin or invert.
- **Camera framing (new 2026-07-12):** The runtime Witness camera now adopts the Director's authored `Witness_Camera` transform as its establishing frame (read from the scene, so it tracks any re-authoring), and when a council member speaks it glides to that sphere's compass bearing at a fixed radius/height and looks at it — the speaking vessel to the fore with the star beyond. (The full hexagram-alignment flight is still a future refinement.)
- **TTS engine proven (new 2026-07-12):** Kokoro-82M through the native sherpa-onnx Windows runtime was selected after a real Forge spike. Seven distinct archetype audition WAVs were generated offline on CPU with no Python, cloud, CUDA, GPL runtime, or voice cloning; warm synthesis measured 0.654-0.873 RTF for the audition lines. The samples are in `artifacts/audio-proof/kokoro/`; the live integration is recorded below.
- **Live archetype voices (new 2026-07-12):** Each time the council yields the floor to a new speaker (`CurrentFocus` changes during `CouncilSpeaking`) Bevy plays that member's pre-rendered Kokoro signature. `WitnessVerdict` submits the generated verdict to a non-blocking sherpa-onnx worker, validates its returned RIFF/WAVE, and plays it through an in-memory Bevy `AudioSource`. Speech status and failures are visible in the ritual UI. The engine and model are pinned by URL and SHA-256 in `scripts/dependencies.json`; mutable generated WAVs live under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\audio_cache`.
- **Living council — real, multi-voice (new 2026-07-12):** Deliberation is no longer a templated stub. On an offering, `chamber/council.rs` queries a local Ollama model (`qwen2.5:7b-instruct`) on a background thread for three council members chosen so all seven surface over time (a framer, a counter, a deepener), each answering **in character** from a constitutional persona, then collapses the exchange into a Witness verdict. If Ollama is unreachable the deliberation fails visibly — never canned text.
- **Speaking choreography (new 2026-07-12):** `CouncilSpeaking` walks the transcript speaker by speaker; each turn moves `CurrentFocus`, drives the camera and voice, tints the environment, and keeps every fixed panel upright and readable.
- **Launch (new 2026-07-12; revised 2026-07-15):** A `Booting` title/loading screen covers everything through a time-gated black intro; default launch no longer waits on the legacy authored vessels/stargate because those visuals are parked. The `launcher` crate is a real supervisor (single-instance guard, readiness checks for Ollama/TTS/Chronos-Foundry, then starts the engine). `scripts/install_shortcut.ps1` builds the release, stages a `dist/`, and creates Desktop + Start Menu shortcuts with an app icon; `scripts/setup_windows.ps1` now also pulls the required Ollama model.
- **Desktop voice repair (2026-07-12):** The install lane now places the pinned sherpa-onnx runtime and Kokoro 11-speaker model under the installed bundle's `speech/` directory. Launcher and engine resolve that portable sibling before Program Files, while still supporting explicit and Program Files roots. All seven archetype speaker assignments are available from the one local multi-speaker model. A real Desktop shortcut launch started `C:\archetypes\dist\engine.exe` after readiness passed.
- **Authored startup and menu (updated 2026-07-15):** Startup is `Booting → MainMenu`. The old `blackflame` video-derived frames/audio remain on disk as preserved assets but are not used by the current intro. `ARCHETYPES` fades in over black, `A GAME BY MICHAEL HOLT` follows, and the black veil slowly fades on a time gate rather than waiting for the old vessels/stargate. The menu is chamber-backed: Standard Mode and Oracle Riddle dispatch into their native paths, Inner Chambers and Living Engine are locked, and `QUIT GAME` gracefully closes the app. A live readiness footer reports Ollama / Chronos Director / Comfy truth.
- **Locked choreography and input (2026-07-12):** Opening/menu/onboarding show the table with the Merkaba hidden. Submission hides the table and reveals the star. ArtifactResult restores the table, returns the camera downward, and pins the image to the stargate. The text editor supports Space, cursor arrows, Home/End, Backspace, Delete, and insertion at the visible caret.
- **Ritual loop:** One continuous council-world ritual runs through the live app — Booting (title) → Onboarding (persistent Witness profile) → table offering → deliberation → the council speaks (multi-voice) → Witness verdict → Chronos/Comfy artifact return. Typing and `Enter` drive it.
- **Portal table (new 2026-07-12):** The Director-supplied raw table was reduced from 1.42M to roughly 120k triangles and 55 MB to 10.6 MB, upgraded with metallic/roughness and blue-region emission, and integrated under the council star. Its authored `Stargate_Portal` disc rotates and pulses; Onboarding/Idle intent text follows the portal in screen space, and submission sweeps the camera upward into the council view. JPEG support is enabled for the embedded albedo.
- **World encapsulation (new 2026-07-12):** The `ArchetypeTheme` registry is no longer dead code. On entering the Architect interior the world inverts to the *Luminous Blueprint* environment — the ceremonial dark void lerps to the archetype's luminous void and the global ambient light floods to structural clarity — driven generically by `CurrentFocus`.
- **Artifact return — direct canvas image (new 2026-07-12):** The game calls Chronos `pipeline/concept-thumbnail`, applying the museum-canvas pattern without its wrapper: generate one standalone painting and return that PNG directly. The live endpoint uses Chronos's bounded SDXL workflow so it can share the RTX 3060 with Bevy; full Flux remains available to Chronos's offline museum lane. No wall, easel, or Blender render is created. Failures remain fail-closed.
- **Council text presentation (new 2026-07-12):** The former full-width text wall is replaced by a closable transcript drawer (`Tab`), a concise theme-colored bubble projected beside the speaking sphere, and a separate low-priority prompt/status surface. New speaker bubbles fly in and fade over 350 ms. Functional verification is automated; visual approval belongs to the operator.
- **Direct-image E2E proof (2026-07-12):** The deterministic live app run released the completed council model from VRAM, submitted concept `2b2ec8d1-eb6e-4c1a-b958-6e86a1ad8171`, received completion event `e3e99f9a-503c-4367-9691-e0f32c73b4de`, verified and staged the 2,188,686-byte PNG, and displayed it in frame `artifacts/visual-proof/council-text-ui-final/08_artifact_result.png`. The speaking UI candidate is `05_council_speaking.png`; neither frame is self-approved aesthetically.
- **Runtime behavior:** Bevy owns the active Witness camera and binds gameplay identities by exported node name. The star tetrahedron and every sphere are held fixed at their authored positions — the star does not rotate and no sphere ever moves. Focus is expressed by camera movement and environment change, not by relocating geometry.
- **Viren:** Ember Covenant palette, no-ambient-glow law, 140/220/480 ms motion timings, and single struck E-flat signature are encoded in the theme registry. Viren is a catalytic subnode, not one of the seven council spheres.
- **Operator playtest audit (2026-07-12 evening):** The operator launched the freshly-built desktop shortcut and dictated a punch list that contradicted several claims above. Root causes were traced in source and fixed (`docs/ledger/2026/07/plan_2026-07-12_2203_deliberation-render-ux-overhaul.md`):
  - The "double-faced icon/portrait panel" claim above (line 10) was upright and non-spinning as stated, but was showing the **Icon** glyph on every sphere, not the **Portrait** artwork — the two faces are coplanar with backface culling disabled, so whichever is nearer the camera wins the depth test regardless of face-normal direction. Fixed: Portrait is now pinned to the near side.
  - The council transcript/bubble used to reveal a line's text the instant the speaking turn advanced, well before that line's TTS had actually started playing (each line's synthesis spawns a fresh `sherpa-onnx-offline-tts.exe` process, which is not instant). Text now only appears once the voice is actually audible.
  - The former split HUD (a transcript box top-left, a separate status line bottom-right) is now one consolidated top panel; jargon copy ("Chronos is painting locally…", raw artifact/PNG/proof-receipt ids) was replaced with plain witness-facing language everywhere except logs.
  - `Deliberating` used to reuse the wide main-menu establishing camera shot (off-center, no clear subject); it now gets its own centered frame on the star.
  - The returned artifact image now eases in from a smaller size instead of popping in at full size.
  - **Found and fixed a real bug, not a style choice:** every Comfy artifact request was silently sending `fidelity: "final"`, a value Chronos's `chronos_director::storyboard_prompt_from_req` does not recognize (it only special-cases `"refined"`), so every render fell through to Chronos's rough-sketch fallback prompt regardless of the `style` field sent. This is why every artifact came back as a dark, low-detail sketch. Fixed the value and shifted the default style toward modern/realistic; confirmed against a real generated image.
  - The portal table's physical shell (not the `Stargate_Portal` disc, which the operator confirmed is excellent) still reads as thin/sparse in the chamber's dark lighting and needs a further pass once the operator resupplied a reference image — addressed in the multilane rebuild below.
- **Multilane visual rebuild (2026-07-13):** Directed as three file-disjoint lanes with a frozen node-name contract (`docs/ledger/2026/07/plan_2026-07-13_0540_multilane-chamber-star-table.md`; Codex briefs `CODEX_LANE_B_*`, `CODEX_LANE_C_*`). All three landed and were verified together on screen:
  - **Lane A (engine):** a procedural **starfield skybox + environment map** replaces the absolute-black void (glass/gold now catch starlight); the "1980s wireframe" `Star_Tetra` is force-hidden and replaced by an engine-authored **solid glowing stellated-octahedron crystal** (tilted to a 3/4 view, deep metallic sapphire) gated to the deliberation/council/verdict states; and per the operator's "just don't show it" directive, all ritual HUD text is suppressed except the input panel at onboarding/table.
  - **Lane B (Codex):** the boxy temple is replaced by a **circular arched, torch-lit chamber** (`assets/scenes/uiscene1.glb`), open to the starfield.
  - **Lane C (Codex):** the sparse table is replaced by an **ornate Flower-of-Life astrolabe table** with a gilded glyph rim, preserving the approved `Stargate_Portal` (`assets/scenes/table.glb`).
  - Integration verified: all 7 vessels bind, panels stay upright, the solid star auto-centres on the new geometry, the portal is intact. The Desktop/Start-Menu shortcut was rebuilt to this integrated bundle.
- **Lane A (Oracle Riddle) implemented and audit-corrected (2026-07-13; fairness corrected 2026-07-14):** The Oracle Riddle reverse-prompt mode is now playable from the mode selector. It runs its own isolated state machine in `modes::oracle_riddle` without breaking the Standard Mode ritual. Players are presented with a generated Chronos image derived from a hidden concrete 3-word visual prompt, then may guess the three clues in any order. Scoring gives exact and alias credit first, can use embeddings for softer semantic matches, and still produces a lexical score if embeddings are unavailable. Results show per-clue matches, total score, and an Insight reward tier; completion/failure records are sealed to the local Lane 0 ledger. Inner Chambers and Living Engine remain visibly locked.

## Blockers
- **The local audit ledger chain is broken at line 34** (`%LOCALAPPDATA%\NeuroCognica\Archetypes\data\ledger.jsonl`,
  `inner_castle_encounter_created`). Caused on 2026-09-11 by `encounter_memory` unit tests that
  appended to the one real `ledger.jsonl` from parallel test threads and forked the chain. The
  tests were fixed the same day (they now use scratch files); the damaged file was deliberately
  **not** rewritten, because repairing a tamper-evident audit trail is an operator decision.
  Effect until decided: `last_hash()` verifies the whole chain before every append, so **every**
  ledger seal now fails — gameplay writes still succeed and are reported honestly as "kept, but
  unsealed" with the reason. Operator decision needed: quarantine the file (rename aside, fresh
  chain starts, old file preserved as evidence) or leave it as is.
- **Sentinel certified (not candidate) release** still needs admin-signed key lifecycle, revocation ceremony, and release/policy signing. Strict certify is **PASS**; adoption remains **candidate**.
- Chronos Foundry and ComfyUI remain sibling products. The launcher starts them when they are installed and down; it does not download Chronos.
- Lore-chamber seated figures remain generated placeholders until an operator-supplied GLB replaces them.
- Portal-table composition and prompt placement have functional capture proof but await operator visual approval.
- Oracle competitive shells (Daily/Speed/Hardcore/Infinite/Versus) remain unbuilt.
- No ambient/music layer. World memory now persists artifact lineage and table tokens; it is not yet Chronos mutation/world-growth.
- **Table geometry:** the physical table shell still reads as thin against near-black lighting. Blocked on the operator's reference image.

## Verification
- **Segmented circular Inner Chambers rotunda drum (2026-09-10):** The former four visible
  square-wall slabs are replaced in `world.rs` by a procedural 64-bay annular drum at 37.5m
  radius, with 16 radial buttresses, annular cornice, four cardinal processional portal frames,
  circular roof cap, and 12 radial ribs. The conservative +/-36m camera clamp remains unchanged;
  the portal frames are interior visual/processional axes, not unsafe exits. Focused mesh test
  proves all 64 closed bays remain in the 37.5–39m annulus; `cargo test --workspace` passed
  117/117 (93 engine, 19 launcher, 5 Windows identity). `scripts\\install_shortcut.ps1` rebuilt
  and restaged the release, SHA-verified the installed engine/launcher, and refreshed the Taskbar
  target. Eight fresh real frames from the staged installed engine are under
  `artifacts/visual-proof/inner-chambers-rotunda-drum-2026-09-10/`, with
  `07_rotunda_drum_and_portal.png` specifically showing the curved shell, cornice, portal frame,
  and radial roof ribs. This is geometry/render proof, not an aesthetic approval: the palette and
  large niche cylinders remain visually austere. The shell-launched capture command did not yield
  a visible process/log witness in this noninteractive session, so launcher E2E capture is not
  claimed beyond installed staging and Taskbar-target verification.
- **Inner Chambers archetype niche chambers (2026-09-10):** `cargo test --workspace` passed
  116/116 (92 engine incl. 4 new focused tests on the new mesh builder and niche geometry, 19
  launcher, 5 windows_identity). `pwsh -File scripts\install_shortcut.ps1` rebuilt the release
  workspace and restaged Desktop/Start Menu/Taskbar; installed `engine.exe`/`launcher.exe`
  SHA-256 verified against the fresh build. New `ARCHETYPES_INNER_CAPTURE=1` self-driving
  capture produced 7 real rendered screenshots from the actual rebuilt binary under
  `artifacts/visual-proof/inner-chambers-capture-2026-09-10/`: a top-down layout shot showing
  the table portal, manifestation altar, exhibit pedestals, and all five niche edges in one
  frame; the table/dais establishing shot; and one portrait per archetype niche showing the
  figure framed by its own tinted wall and flanking threshold pillars. This is a rendered-frame
  proof, not a self-approval of the visual direction — the operator has not reviewed it.
- **Inner Chambers Castle Rotunda & Animated Council Table (2026-09-09):** `cargo test --workspace` passed 100/100 tests (79 engine, 16 launcher, 5 windows_identity). Desktop restaged via `scripts\install_shortcut.ps1` with fresh `dist\engine.exe` release binary and staged `assets\scenes\table.glb`. Table visibility unblocked in `chamber/camera.rs`, legacy primitive shapes purged from `world.rs`, 6DOF flight verified.
- **Honest RC (2026-08-16):** `cargo test --workspace` passed (79 engine + 16 launcher). Sentinel `certify --strict` **PASS**; readiness **candidate**. ARP install/uninstall proven (HKCU, per-user Programs\Archetypes, AppData kept). Desktop restage; capture `00_title_arch.png` / `01_title_subtitle.png` / `02_lore_main_menu.png`. PE FileVersion 0.3.0.0.
- **Product depth (2026-08-16):** `cargo test --workspace` passed (78 engine + 15 launcher). Sentinel `adoption_readiness` PASS (candidate). Hexagram camera, seven Inner Chambers, council interior crossing, world-memory lineage, and launcher sidecar start are covered by unit tests. Desktop restage via `scripts\install_shortcut.ps1`.
- **Product rebuild (2026-08-16):** `cargo test --workspace` passed (68 engine + 13 launcher). Identity: Standard = council ritual, Consciousness = 1:1 chat. Inner Chambers and Living Engine unlocked with tested loops. Launcher fail-visible + TTS sidecar repair covered by unit tests for hash/manifest path. Desktop restage via `scripts\install_shortcut.ps1`.
- **Seed-of-Life Standard Mode UI (2026-07-20):** `cargo test --workspace` (55 engine + 3 launcher). Seed slot geometry + Cinzel/canon asset asserts. `scripts\install_shortcut.ps1` restaged Desktop. Operator confirm: selector matches Seed-of-Life plate; chat is black+gold.
- **Maximized window / hidden console / taskbar icon (2026-07-20):** `cargo test --workspace` (54 engine + 3 launcher). `scripts\install_shortcut.ps1` release-staged dist. PE: `dist\engine.exe` and `dist\launcher.exe` are `WINDOWS_GUI` with embedded `RT_GROUP_ICON`. Brief `dist\engine.exe` smoke stayed up 4s. Operator visual confirm: Desktop launch → maximized game only, no console, Archetypes icon on taskbar/shortcut.
- **Mecha chat scroll (2026-07-20):** `cargo test --workspace` + `scripts\install_shortcut.ps1`. Wheel/drag scroll wired in `standard_mecha` from Bevy 0.18 scroll example pattern; release smoke OK.
- **Mecha multi-turn chat UI (2026-07-20):** `cargo test --workspace` passed (54 engine + 3 launcher). Desktop restaged via `scripts\install_shortcut.ps1` (preserves `assets/standard_mecha/renders`). Unit coverage for transcript fingerprint detecting turn 2 without idle rebuilds. Confirmed prior sentinel.jsonl already contained complete turn 2/3 assistant text + image metadata (backend OK; UI was the break). Release `dist\engine.exe` smoke-ran 6s OK.
- **Boot crash fix + launcher errors (2026-07-20):** Reproduced Desktop exit 101 as Bevy B0001 — first from boot/camera Visibility conflict, then from `render_chat_ui` dual `&mut Node` queries (`ChatWaitingBanner` vs `ChatPortrait`) without `Without<>` disjoint filters. Both fixed. Launcher now writes `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\last-engine.log` and opens `last-failure.txt` in Notepad on engine failure. `cargo test --workspace` passed (53 engine + 3 launcher). `scripts\install_shortcut.ps1` restaged Desktop; release `dist\engine.exe` smoke-ran 6s without crash.
- **Boot veil / chat wait / desktop dist (2026-07-20):** `cargo test --workspace` passed with 53 engine tests and 2 launcher tests. Desktop product refreshed with `scripts\install_shortcut.ps1` so `dist\engine.exe` / `dist\launcher.exe` and Desktop/Start Menu `Archetypes.lnk` match this unit. Waiting copy asserts `WORKING (Ns) — …` format.
- **Mecha chat image aspect + in-channel artifacts (2026-07-19):** `cargo test --workspace` passed with 53 engine tests and 2 launcher tests. Focused `standard_mecha` tests cover portrait aspect constants matching source asset shapes, history/image JSONL, jargon-free player image lines, and phased wait copy. UI change is in `crates/engine/src/modes/standard_mecha/mod.rs` (scrollable `ChatTranscriptRoot` with inline `ChatArtifactImage`, Auto-fit portraits).
- **Chronos play reliability + chamber feel (2026-07-15):** `cargo test --workspace` passed with 52 engine tests and 2 launcher tests. Launcher probes Director readiness + Comfy `/system_stats` and fails closed without Chronos unless the debug override is set. Engine readiness banner, Oracle menu restore, phased chat wait, soft image reveal, and jargon-free player image lines are covered by unit tests in this commit.
- **Standard chat Comfy response images (2026-07-15):** `cargo test --workspace` passed with 47 engine tests and 1 launcher test. `cargo build -p engine` passed. `scripts\install_shortcut.ps1` completed the release build but timed out before staging; the fresh release binaries were then staged manually into `dist\engine.exe` / `dist\launcher.exe`, `dist\assets\` was refreshed, and the Desktop + Start Menu shortcuts were verified to target `C:\archetypes\dist\launcher.exe` with working directory `C:\archetypes\dist`. Runtime prerequisites were live-checked: Chronos Director `:7777` reported `readiness: ready`, ComfyUI `:8000` reported RTX 3060 CUDA availability, and Ollama `:11434` listed the required local models. The staged `dist\launcher.exe` was run with `ARCHETYPES_MECHA_CAPTURE=1`; real rendered frames landed under `artifacts/visual-proof/standard-chat-comfy-2026-07-15_1818/`, including `05_architect_chat_result_or_failure.png`, which shows the Architect response with a completed Comfy image. The persisted turn in `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\standard_mecha\chat_history\architect.jsonl` records artifact `c790b503-3f6f-412d-a788-f3b103b25a26`, proof receipt `e752ae1a-578a-4727-8066-54edae172d6f`, source PNG `C:\Users\m\Documents\ComfyUI\output\chronos_concept_f79ab5c12de7439aa7375154031fbe48_00001_.png`, and staged asset `standard_mecha/renders/architect-c790b503-3f6f-412d-a788-f3b103b25a26-1784161338662.png`; the global hash-chained ledger sealed matching `mecha_chat_user` and `mecha_chat_assistant` events.
- **Lore chamber launcher integration (2026-07-15):** Blender 4.5 ran `scripts\author_lore_chamber.py` headlessly, exported `assets/scenes/lore_chamber.glb`, rendered `artifacts/visual-proof/lore-chamber-2026-07-15_0539/blender_lore_chamber.png`, and re-imported the GLB with `objects=121`, `meshes=104`, `lights=17`, `cameras=0`, `point_lights=16`, `triangles=35074`, `missing=[]`. The runtime GLB intentionally exports no cameras so Bevy owns a single live `RuntimeWitnessCamera`. `cargo test --workspace` passed with 45 engine tests and 1 launcher test. `cargo build -p engine` passed. `scripts\install_shortcut.ps1` rebuilt release binaries, staged `dist`, verified the Ollama model and offline TTS assets, and refreshed the Desktop / Start Menu shortcuts. The staged `dist\launcher.exe` was run with `ARCHETYPES_LORE_CAPTURE=1`; it reported Ollama, TTS, Chronos Director, and ComfyUI ready, then captured the witnessed frames under `artifacts/visual-proof/lore-chamber-runtime-2026-07-15_0539/`: `00_title_arch.png` (black title), `01_title_subtitle.png` (black creator credit), and `02_lore_main_menu.png` (chamber-backed main menu).
- **Blank-slate launcher reset (2026-07-15):** `cargo test --workspace` passed with 44 engine tests and 1 launcher test. `cargo build -p engine` passed. `scripts\install_shortcut.ps1` rebuilt release binaries, staged `dist`, verified Ollama model and offline voice assets, and refreshed Desktop/Start Menu shortcuts. The staged `dist\launcher.exe` was run with `ARCHETYPES_BLANK_CAPTURE=1`; it reported Ollama, TTS, Chronos Director, and ComfyUI ready, then captured real rendered frames under `artifacts/visual-proof/blank-slate-shell-2026-07-15_0525/`: `00_title_arch.png`, `01_title_subtitle.png`, and `02_blank_main_menu.png`. The witnessed menu frame shows no legacy chamber, table, portal, starfield, Mecha selector, or Oracle UI.
- **Mecha Standard Mode implementation (2026-07-15):** Verified Mecha asset mirroring by SHA-256 hash comparison from `C:\mecha\aura-mechanician\frontend\src\assets` to `assets/mecha` (`mecha_asset_hashes_match=true`, 19 PNGs). `cargo test --workspace` passed with 43 engine tests and 1 launcher test. `cargo build -p engine` passed. `scripts\install_shortcut.ps1` rebuilt release binaries, staged `dist`, verified offline voices/model, and refreshed Desktop/Start Menu shortcuts. The staged `dist\launcher.exe` was run with `ARCHETYPES_MECHA_CAPTURE=1`; it reported Ollama, TTS, Chronos Director, and ComfyUI ready, then captured real rendered frames in `artifacts/visual-proof/mecha-standard-2026-07-15_0446-oracle/`: `00_title_arch.png`, `01_title_subtitle.png`, `02_portal_main_menu.png`, `03_mecha_selector.png`, `04_architect_chat.png`, `05_architect_chat_result_or_failure.png`, `06_oracle_chat_after_switch.png`, `07_oracle_riddle_generating.png`, and `08_oracle_riddle_guessing_or_result.png`. The Architect chat frame shows a real local Ollama response and persisted history; the Oracle Riddle frame reached the generated-image guessing state.
- **Fresh Standard Mode handoff (2026-07-15):** Docs-only handoff unit. Verified by reading source truth anchors, checking the generated handoff content, diff, and repo status. No Rust/runtime code changed, so `cargo test --workspace` is not required under `AGENTS.md` risk-based gates.
- **Risk-based verification rules (2026-07-15):** Docs/rules-only change. Verified by reading `AGENTS.md`, checking the diff, and confirming repo status before commit. Per the new repo rule, no Rust runtime code changed, so `cargo test --workspace` is not required for this unit.
- **Mecha Standard Mode audit (2026-07-15):** Mechanical inventory, asset dimension checks, source reads, endpoint scans, and Archetypes ownership mapping completed for `C:\mecha\aura-mechanician\frontend\src`; recorded in `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`. This was an audit/docs unit only; no desktop build was refreshed.
- **Inner Chambers safety correction (2026-07-15):** `cargo test --workspace` passed with 38 engine tests and 1 launcher test after relocking Inner Chambers, repairing its parked exit lifecycle, adding proximity-gated truth extraction, and adding focused pure tests. `dist` was not refreshed because no new playable desktop behavior was shipped.
- **Lane A Oracle Riddle audit (2026-07-13):** `cargo test --workspace` passed with 30 engine tests and 1 launcher test after the corrective pass. Verified semantic-distance prompt tiers, exact/partial vector scoring without live Ollama, ledger payload shape, Oracle unlocked, and Inner Chambers / Living Engine locked.
- **Lane 0 spine audit correction (2026-07-13):** `cargo test --workspace` passed with 24 engine tests and 1 launcher test, 0 failed. The corrective pass added mode-registry tests and ledger chain/tamper verification tests.
- `cargo test --workspace` passed on 2026-07-12: 9 passed, 0 failed (engine 9, launcher 0).
- `cargo build -p engine` rebuilt the runnable binary on 2026-07-12 before every capture.
  (Lesson recorded: `cargo test` builds test harnesses, **not** the runnable `engine.exe`;
  a prior "static" claim was made against a stale binary and was wrong.)
- **Static star — mechanical proof (2026-07-12):** the re-exported GLB has `animations: 0`
  and no engine code rotates anything. Two Onboarding frames captured 3.5 s apart (camera
  provably fixed in that state) differ on **6 of 3,686,400 pixels**, max delta **2/255** —
  i.e. nothing moves. Rotation would have changed thousands of pixels.
- **Glass retheme (2026-07-12):** the star tetrahedron renders crystal-clear with black
  wireframe edges; the glass spheres are the visual focus (verified in the live window).
- **Earlier end-to-end (prior build, 2026-07-12):** capture mode drove the full ritual and
  the artifact-result frame showed a real ~120 s Chronos/Blender render displayed in-chamber
  (`Status: complete`, verified `C:\chronos\renders\...png`). The interior-inversion and
  artifact-display systems are unchanged by the star/glass work.
- Vulkan validation-layer warnings remain non-fatal on this machine.
- **Temple export proof (2026-07-12):** Blender re-imported the final GLB with 82 objects, 65 meshes, 7 supported lights, 7 panel animation actions, 16 embedded images, 21 named icon/portrait panel nodes, and 41 temple nodes. The Blender visual proof is `artifacts/visual-proof/temple-overhaul-0040.png`.
- **Live voice proof (2026-07-12):** A fresh `engine.exe` deterministic run played the packaged signature WAV on focus, then generated an 808,552-byte verdict WAV from the real `WitnessVerdict` transition; the returned file passed RIFF/WAVE validation. Standalone warm audition lines measured 2.47-3.92 seconds.
- **Playable-build proof (2026-07-12):** A fresh `cargo build -p engine` + deterministic capture walked the whole loop on screen: `00_title` (title/loading screen, nothing of the chamber shown), `01_onboarding` (new "YOU ARE THE WITNESS" copy), `02_table` (authored establishing frame showing the star, vessels, and temple), `05`/`05b_council_speaking` (a real Ollama line with the speaker's sphere framed and its panel turned between the two frames), `06_witness_verdict`. The logged transcript showed three distinct in-character voices (e.g. Explorer/Sentinel/Oracle) plus a synthesized verdict — not templated text. The Comfy artifact reached `ArtifactPending` and submitted a real request; the Director's render then failed on its `codex.db` error (see blocker).
- **Portal-table proof (2026-07-12):** `artifacts/visual-proof/portal-table-final-v2/02_table.png` shows the embedded JPEG albedo, metallic gold ringwork, emissive blue vortex, portal-following intent text, and table camera pose in the live Bevy window. `03_deliberating.png` proves the camera has swept away from the table on submission. These are review candidates, not self-approved visuals.
- **Launch proof (2026-07-12):** `scripts\install_shortcut.ps1` built the release workspace, staged `dist/` (engine + launcher + assets + `archetypes.ico`), and created `Desktop\Archetypes.lnk` and a Start Menu shortcut pointing at the supervising launcher.
- **Operator playtest fixes — verification (2026-07-12 evening):** `cargo test --workspace`: 15 passed, 0 failed. Each fix in the audit above was verified against a rebuilt `target/debug/engine.exe` under `ARCHETYPES_CAPTURE=1` with real Ollama, real Kokoro/sherpa-onnx TTS, and a real Chronos/ComfyUI artifact render in the loop (not just `cargo test`) — screenshots showed all seven spheres displaying their archetype portrait right-side-up, a centered deliberation frame, the consolidated plain-language top panel, and a real generated artifact that is a modern, warm, detailed painting rather than the previous dark low-detail sketch. `scripts\install_shortcut.ps1` then rebuilt the **release** workspace and re-staged `dist/`/the Desktop and Start Menu shortcuts with these fixes; the table-geometry item remains unfixed pending the operator's reference image.
