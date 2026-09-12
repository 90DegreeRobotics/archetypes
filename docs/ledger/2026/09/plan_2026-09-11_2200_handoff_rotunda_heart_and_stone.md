# Plan / Handoff: The Rotunda Heart and the Stone — 2026-09-11 22:00

## Status
PENDING — written as a handoff. The previous builder is standing down; this is addressed to
whoever picks it up next.

---

## To the builder taking this over

You are inheriting a castle that is structurally sound and visually half-finished. The geometry,
the collision and the module pipeline all work and are covered by tests; what is missing is
surface treatment and one piece of wiring that was never reconnected. Five things are asked for
here. They are ordered so that nothing you build gets thrown away by a later step.

Read these first, in this order:

1. `crates/engine/src/modes/inner_chambers/castle.rs` — every dimension of the building and all
   of its collision. This is the single source of truth. Nothing else should invent a radius.
2. `scripts/author_arcade_bay.py` — the pattern every kit module follows.
3. `STATUS.md` "Current State" — the last eight entries are this work, newest first.

**Two habits from this repo that are worth keeping, because both caught real bugs:**

- **Every module script re-imports its own export and verifies it against the contract, not
  against intent.** The arcade bay checks that its cornice run still matches the chord; the
  vault checks its span and rise; the stair checks its radial band and that it tops out at
  exactly its rise. Each of those caught a genuine error before it shipped.
- **Verify by walking, not by flying.** `ARCHETYPES_WALK_CAPTURE=1` drives the real locomotion
  system with real key presses and writes a report of position, focus and state at each
  screenshot. The older `ARCHETYPES_INNER_CAPTURE=1` harness forces free flight and teleports,
  which is precisely how a movement bug survived for weeks: a flying camera never touches the
  walking code path. Use the flying one for vantage shots, never for proving a mechanic.

Run both from `dist/` after `pwsh -File scripts/install_shortcut.ps1`, which rebuilds release
and restages the installed build.

---

## What is being asked for

1. Stone textures on everything, varied by level of the arcade.
2. The main floor and rotunda area fixed.
3. Every character mesh removed except the Jester.
4. The old table's spinning portal effect brought back — it was the best thing in the build and
   was never reconnected.
5. The manifestation chamber moved to the centre so that the spinning effect sits **under** it.

Items 3, 4 and 5 are one piece of work: they all converge on the middle of the Council circle.
Do them together, as Phase 2 below.

---

## Phase 1 — Stone textures across the kit

### What exists now

Three Blender kit modules are built, and **every one of them is UV-unwrapped already** — each
script's export check fails if any mesh lacks UVs. Nothing is textured; everything is flat
colour. The modules are:

| Module | Script | Instances | Tris |
|---|---|---|---|
| Arcade bay | `scripts/author_arcade_bay.py` | 72 per storey x 7 | 2,268 |
| Stair flight | `scripts/author_stair_flight.py` | 2 per storey x 7 | 380 |
| Vault fresco | `scripts/author_vault_fresco.py` | 1 | 5,280 |

The engine already renders normal maps — `world.rs` uses `normal_map_texture` in three places,
and `chamber_floor_textures()` generates an albedo *and* a tangent-space normal procedurally.
The vault fresco proves the whole texture path end to end: image → Blender material → GLB with
the texture embedded → Bevy. Texturing the rest is that same path.

### The blocker nobody has dealt with yet

**The kit's UVs are `smart_project`, which has no physical scale.** It packs islands into unit
space per object, so the same stone would appear at one size on a pier and a different size on a
tread. Before any stone texture can look right, the kit scripts need **world-scaled UVs** — a
cube projection sized in metres, so a block reads at a consistent ~0.9m everywhere in the
building. This is a change to `bevel_and_unwrap()` in the bay script and `unwrap()` in the stair
script. Do it first; every texture decision after it depends on it.

### Source material

The operator supplied a reference sheet of nine stone types (in `Downloads`, dated 2026-09-11
21:17): Rough Granite Block, Weathered Limestone Ashlar, Dark Basalt Fortress Stone, Mossy
Medieval Wall, Sandstone Block, Chipped Rubble Stone, Wet Dungeon Stone, Old Mortared
Fieldstone, Worn Courtyard Flagstone.

Be honest with the operator about what that sheet is: **one 1254x1254 image containing all
nine**, so each tile is roughly 378x358 pixels. On a 9.9m bay that is about 38 px/m and will
read as mush. It is an excellent *look* reference and an unusable *asset*. Two routes:

- **Source proper seamless PBR sets** (albedo / normal / roughness / AO at 2K) matching those
  nine looks. Higher quality, needs sourcing and licence checking. See
  `memory: build-it-dont-wait-for-a-clean-license` — if everything available is encumbered, that
  closes "adopt", not the capability; author originals rather than softening the goal.
- **Upscale and heal the operator's tiles into seamless 1K.** Faster, and good enough to judge
  UV scale and normal strength before anything is sourced.

Either way the tiles must be **seamless**, or a visible grid repeats every few metres across
716m of circumference. And albedo alone will not fix flatness: a flat photo of stone on a flat
pier still reads flat because nothing responds to the light. **Generate a normal map from each
albedo** (luminance → height → normal); `chamber_floor_textures()` in `world.rs` is the existing
precedent in this repo.

### Varying stone by level

The operator wants different stone at different levels of the arcade. `castle.rs` already gives
you everything needed: `ARCADE_BAYS_PER_LEVEL` (72), `GALLERY_LEVELS` (7), and
`arcade_bay_position(level, index)`.

The cheap implementation is wrong: exporting seven bay GLBs, one per stone, multiplies a 151KB
asset by seven and forks the module. **Export one bay and swap the material at spawn time** —
Bevy can override a scene's materials, or the bay can carry a named material slot the engine
rebinds per level. Suggested progression, heaviest at the base, which is both how masonry is
actually built and how the eye reads height:

| Level | Deck y | Stone |
|---|---|---|
| 1 | 12.5 | Dark Basalt Fortress Stone |
| 2 | 24.5 | Rough Granite Block |
| 3 | 36.5 | Old Mortared Fieldstone |
| 4 | 48.5 | Chipped Rubble Stone |
| 5 | 60.5 | Weathered Limestone Ashlar |
| 6 | 72.5 | Sandstone Block |
| 7 | 84.5 | Weathered Limestone Ashlar, lightest tint |

That is a proposal, not a decision. **The operator is the sole judge of the look** — see
`memory: never-self-approve-visuals`. Report mechanical facts, show the frame, and never call it
good yourself.

Wet Dungeon Stone and Mossy Medieval Wall do not belong on a lit ceremonial rotunda; hold them
for the under-castle and the abyss level, which is currently bare.

---

## Phase 2 — The rotunda heart

This is the emotional centre of the request. Read all of it before touching anything, because
the three asks are entangled.

### 2a. What is actually at the centre right now

`world.rs` line ~258, in the live `setup_inner_world`:

- A flat `Cylinder::new(3.15, 0.08)` at y=0.44 named `CouncilPortalFloorInlay_Spin`, with three
  `Cuboid` spokes, all carrying `ChronosExhibitTurntable { speed: 0.16 }`, and one point light.

That is a flush floor inlay *imitating* the portal. It is not the effect the operator means.

### 2b. The real effect exists and still works — it was never plugged in

`crates/engine/src/chamber/portal.rs` is intact and `PortalPlugin` **is registered**
(`chamber/mod.rs:72`). It binds any entity named `Stargate_Portal` and, every frame, spins it on
its local Y at 0.16 rad/s while pulsing its emissive through
`LinearRgba::new(0.18 * pulse, 0.48 * pulse, pulse, 1.0)` with `pulse = 2.4 + sin(t * 1.7) * 0.8`.
The disc itself is authored inside `assets/scenes/table.glb`; the engine only animates it.

**The reason it is not in the castle:** `table.glb` is spawned at `world.rs` line ~852, which is
inside `setup_legacy_rotunda_world` — a function marked `#[allow(dead_code)]` and never
registered as a system. Only `setup_inner_world` runs. So the table, and with it the portal disc,
simply never enters the live Seed-of-Life world. Nothing is broken; it was left behind in the
rewrite.

The legacy spawn, for reference — note these numbers are for the **old** castle scale and must be
re-derived, not copied:

```rust
SceneRoot(asset_server.load("scenes/table.glb#Scene0")),
Transform::from_xyz(0.0, 1.523, 0.0).with_scale(Vec3::splat(1.56)),
Name::new("RotundaCouncilTable"),
// plus PointLight { intensity: 22_000.0, range: 9.0, color: srgb(0.25, 0.75, 1.0) } at y=1.74
```

The comment above it records the derivation: feet authored at local z = -0.784, so resting on a
dais top at y = 0.30 needs origin y = 0.30 + 0.784 * 1.56 = 1.523. **Redo that arithmetic for the
current floor height** (`castle::GROUND_Y` = 0.4) and whatever scale is chosen. The Council
circle is now `COUNCIL_RADIUS` = 24m, against 16m when those numbers were written, so the table
will likely want to be larger, not the same.

### 2c. Moving the manifestation chamber onto it

`MANIFESTATION_PEDESTAL_POS` (`manifestation.rs:34`) is `Vec3::new(0.0, 0.30, 3.4)` — offset 3.4m
from the centre. Everything the altar spawns derives from that one constant (`manifestation.rs`
lines 186, 457, 510 all read `let p = MANIFESTATION_PEDESTAL_POS`), so moving the constant moves
the whole altar. That is the good news.

The asked-for arrangement is: **table and spinning portal at the exact centre, manifestation
altar directly above it, so the vortex turns underneath the altar.** Concretely that means
`MANIFESTATION_PEDESTAL_POS` becomes `(0.0, <height above the portal disc>, 0.0)`, and the altar
needs to sit high enough that the disc is visible spinning beneath rather than being covered.

**Do not forget these three, or the change will look right and behave wrong:**

1. `camera.rs::character_obstacles()` hard-codes a manifestation pedestal collider at
   `(Vec2::new(0.0, 3.4), 1.25)`. Move it to `(0.0, 0.0)` or the player will walk through the
   altar and bump into thin air 3.4m away. **Derive it from `MANIFESTATION_PEDESTAL_POS` rather
   than retyping the number** — two colliders in that same list were previously transcribed with
   a flipped sign and sat 10m from the thing they were meant to block.
2. `interaction.rs` resolves the altar by distance to `MANIFESTATION_PEDESTAL_POS` with
   `ALTAR_INTERACTION_RANGE` = 3.2. At the centre the player approaches from any bearing, so
   check the range still reaches from the floor: the altar origin is low and eye height is 3.25,
   which already costs about 2.95m of that 3.2m budget vertically. It may need raising.
3. The existing `CouncilPortalFloorInlay_Spin` cylinder and its three spokes are the stand-in
   being replaced. Delete them, or the real portal will spin inside a fake one.

### 2d. Removing the character meshes

Remove every `.glb` character except `nebula_jester.glb`:

| Mesh | Where | Action |
|---|---|---|
| `aura.glb` | `world.rs:296`, Council circle at (0, 0.42, -8.5) | **Remove** |
| `nebula_jester.glb` | `world.rs:302`, at (10.2, 0.42, 6.2) | **Keep** |
| `architect` / `sentinel` / `explorer` / `empath` / `mentor` / `oracle` `.glb` | the `SeedRoom` table, `world.rs:309-314`, `asset: Some(...)` | **Remove** — set `asset: None` |

**This has a consequence that must go back to the operator before you ship it.** The six room
figures each carry the `ArchetypeEmbodiment` component, and that component is the *only* thing
the conversation system targets:

- `interaction.rs:134` queries `(&Transform, &ArchetypeEmbodiment)` to resolve focus;
- `encounters.rs` opens a conversation against `InteractionTarget::Archetype(embodiment)`.

Delete the figures and the six archetype conversations become unreachable. The Remember/Forget
memory system, the local Ollama reply path and the TTS reply path all still work — they simply
have nothing to talk to. Three honest options, for the operator to choose:

- **(a)** Attach `ArchetypeEmbodiment` to something else in each room — the plinth, a niche
  marker, the workshop bench — so the rooms still hold conversations without a figure standing
  there. Cheapest, keeps the feature alive.
- **(b)** Accept that conversation moves to the Jester alone, and give the Jester all seven
  archetype identities to speak as. A real design change, not a deletion.
- **(c)** Remove the figures now and accept conversations are dark until new meshes arrive.

Also delete the six room-figure entries from `camera.rs::character_obstacles()` when the figures
go, or the rooms keep six invisible 1.5m pillars standing where nobody is. That array is derived
from `castle::room_centres()` and `EMBODIMENT_RADIAL_OFFSET`, so it is one edit, not six.

---

## Phase 3 — The main floor and the rotunda area

The operator's words were that the main floor and rotunda area "must be fixed". Two concrete
defects are already known; walk the floor before assuming they are the only ones.

1. **The Council and room platforms still carry metre-wide flagstone gaps.** `spawn_castle_platform`
   calls `build_radial_flagstone_mesh(..., 16, 0.055)`. That gap is an **angle**, so at the
   Council circle's 24m radius it is a **1.32m hole** between stones, and at a room's 26m radius
   a **1.43m** one. The gallery decks had exactly this bug and were fixed on 2026-09-11 by
   specifying joints in metres through the new `joint_angle(metres, radius)` helper in `world.rs`.
   Apply the same helper here. The wide gaps currently read as a radial spoke pattern from
   overhead, which is why they were not caught earlier — confirm with the operator whether they
   want the pattern kept as decoration before flattening it to mortar lines.
2. **The same mesh raises its stones above its own origin.** The gallery decks were floating
   0.55m above the height collision used, so players crossed them buried to the shin in their own
   floor. Decks and the promenade were dropped by the stone thickness; **`spawn_castle_platform`
   has not been checked for this**. Verify what height the Council floor's stone tops actually sit
   at versus `castle::GROUND_Y` (0.4), and drop the mesh if they disagree.

Beyond those two, the floor is untextured flat colour and will be picked up by Phase 1.

---

## Sequencing, and why

1. **Phase 2 first** (heart: characters out, table in, altar centred). It is the operator's
   priority, it is self-contained, and it changes what stands on the floor — which you want
   settled before you texture that floor.
2. **Phase 3 next** (floor and platform fixes). Small, mechanical, and it makes Phase 1's results
   legible.
3. **Phase 1 last** (stone). Start with world-scaled UVs, prove one texture on one module, get an
   operator verdict, then roll out by level.

## Verification each phase must pass

- `cargo test --workspace` — currently **199 passing** (175 engine, 19 launcher, 5
  windows_identity). Do not let that number fall.
- `pwsh -File scripts/install_shortcut.ps1` — rebuilds release and restages the installed build
  with SHA-verified binaries.
- A **walking** capture (`ARCHETYPES_WALK_CAPTURE=1`) for anything touching movement, collision
  or interaction, with its `walk_report.txt` committed as evidence.
- Frames under `artifacts/visual-proof/<topic>-<date>/`, and an explicit statement of what they
  do **and do not** prove. Geometry proof is not aesthetic approval.

## Known-good things not to break

- `castle.rs` is the only place dimensions live. The Blender scripts carry matching contract
  constants and fail their own export checks when the two drift; keep that property.
- The wall is a shell with a doorway exemption only for the archetype rooms. The museum plan
  (`plan_2026-09-11_2130_museum_arches.md`) depends on adding a second exemption for chosen bays;
  do not make `clamp_inside_wall` unconditional.
- Writes report honestly: a journal row that lands while the ledger seal fails reports "kept, but
  unsealed" with the reason, rather than claiming nothing was written. See `WriteReceipt` in
  `services/build_intent.rs`.

## Open item inherited, not caused by this plan

**The local audit ledger chain is broken at line 34** and every ledger seal in the app has been
failing since. It is logged under Blockers in `STATUS.md` with the cause and two options
(quarantine the file, or leave it). It needs an operator decision, not an agent's. Gameplay
writes still succeed and say so honestly, so it is not urgent — but it should not be discovered
a third time by someone wondering why nothing seals.
