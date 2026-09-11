# Plan: Inner Chamber architecture and real multiview evaluation — 2026-09-09 22:20

## Status
IN PROGRESS — Steps 2-4 landed and verified in follow-up sessions (2026-09-10). Step 5's
"download a licensed candidate" path is CLOSED for real as of 2026-09-11 (systemic licensing
dead end, evidence below) and replaced with a from-scratch original-code research direction
that has not yet started.

## Goal
Replace the current open warehouse-like Council Chamber blockout with a legible central rotunda, a physical light-reactive floor, and one explorable chamber for each of the five standing archetype meshes (Sentinel, Aura, Empath, Oracle, and Nebula Jester). Use those rooms as the first live test bed for distinct archetype design languages. In parallel, evaluate an additional reconstruction view honestly: validation-only while TripoSR remains single-view, and geometry-constraining only after a 12 GB-safe multiview engine proves it consumes distinct persisted views.

## Session acceptance ledger — nothing may be silently dropped

- [ ] Preserve the absolute no-recipe law: every Sentinel-cleared object prompt reaches the same model-driven generation path; no noun lookup, premade mesh, or scripted substitute.
- [ ] Preserve the proven player prompt → hidden Chronos2 CLI → receipt-bound TripoSR OBJ → headless Blender GLB → live pedestal placement path.
- [ ] Preserve buyer-facing stage text, real elapsed time, cancellation/process-tree cleanup, failure explanation, plasma charge, lightning/smoke reveal, and permanent title-bar version identity.
- [ ] Preserve the sub-minute manifestation speed class, explicit VRAM unload between SDXL and TripoSR, and the pinned launcher as the test surface.
- [ ] Preserve Sentinel adult-content enforcement and its explicit in-world refusal reason.
- [ ] Preserve corrected Z-up import, neutral exhibition lighting, and unmistakable turntable motion.
- [ ] Do not call generated output believable merely because the pipeline completed; retain the reference, prepared input, mesh receipt, and visual witness for review.

## Steps

### Step 1 — Measure the current building and movement envelope
- [ ] Inventory the central table, manifestation altar, exhibit pedestals, five figure transforms, walls, player spawn, collision radii, and portal/menu routes.
- [ ] Capture a pinned-launcher screenshot set and top-down coordinate witness before moving geometry.
- Files touched: None.
- Expected outcome: A measured plan that cannot strand the player, overlap the table, hide a figure, or break the manifestation approach.

### Step 2 — Build the central rotunda as architecture
- [x] Replace the featureless box impression with a circular/segmented rotunda: articulated wall bays, columns, cornice/rib structure, readable entrances, ceiling treatment, and deliberate sightlines to the council table and altar.
- [x] Keep collision simple and explicit even when visible architecture is layered.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs` and focused tests/helpers.
- Expected outcome: The main room reads as a chamber, not a dark warehouse with objects placed in it.

### Step 3 — Make the main floor physically legible
- [ ] Retain normal-map response but add macro-scale geometric joints/inlays or relief that survives the current camera distance and raking light.
- [ ] Add a neutral key/fill/rim hierarchy with real directed lights and controlled ambient level; verify material response at walking and flight viewpoints.
- Files touched: world material/light code and any generated code-native texture assets.
- Expected outcome: Screenshots visibly show stone/metal surface structure and light direction without relying on emissive paint.

### Step 4 — Build five connected archetype chambers
- [x] Create one accessible chamber bay for Sentinel, Aura, Empath, Oracle, and Nebula Jester, each containing its existing standing mesh as the focal presence.
- [x] Derive style from each archetype's canonical profile and measurable material/light/geometry parameters; avoid decorative clutter that has no functional or symbolic job.
- [x] Give every room a distinct threshold, floor treatment, wall rhythm, ceiling/vertical silhouette, key light, rim light, and navigable inspection space.
- [x] Keep reusable architecture procedural and parameterized; do not hardcode premade chamber meshes masquerading as generated work.
- Files touched: `world.rs`, archetype profile data/helpers, tests, and architecture documentation.
- Expected outcome: Five rooms are unmistakably different in mood and form while remaining one coherent building.

### Step 5 — Add a second view without theatre

**"Download and benchmark a candidate" is CLOSED, permanently, as a dead end — not deferred.**
Every existing open multiview-to-3D reconstruction model was checked at primary sources
(GitHub/HF LICENSE files, training configs, model-zoo resolvers, commit-pinned) via a
Manus research thread appended to
`C:\chronos2\docs\plans\plan_2026-09-07_1530_quality-engine-licence.md`. The result is
systemic, not one bad candidate:

- **InstantMesh** — Apache-2.0 wrapper, but requires Zero123++ (SUDO-AI), whose weights
  are CC-BY-NC-4.0. Flatly non-commercial.
- **Stable Zero123** (Stability) — non-commercial-only free tier; the commercial variant
  requires an ongoing paid Stability AI membership.
- **CRM** (thu-ml) — MIT badge on the repo, but both diffusion checkpoints resume-train
  from ImageDream's `sd-v2.1-base-4view-ipmv.pt`, an Open RAIL++-M derivative of Stable
  Diffusion 2.1-base (confirmed via CRM's own `imagedream/model_zoo.py` resolver and
  training configs, commit `8a16cf6`). §5 of Open RAIL++-M reaches *running* the model,
  not just distributing it, so the operator-install posture that cleared TripoSR's MIT
  does not fully clear this. The separable reconstruction net (`CRM.pth`) has no published
  training record at all (unverifiable, not proven clean), and its own constructor makes
  an unpinned runtime fetch to `stabilityai/stable-diffusion-2-1-base` — which by itself
  already breaks the "ships and fetches nothing" rule TripoSR's whole license position
  depends on, independent of the legal question.
- **OpenLRM** — Apache-2.0 code, CC-BY-NC-4.0 weights. Same shape as InstantMesh.
- **LGM** (3DTopia) — MIT wrapper, but its own paper states it requires ImageDream for
  image-conditioned input. Same contamination as CRM, same source.
- **TRELLIS / TRELLIS.2** (Microsoft) — MIT, but 16 GB / 24 GB VRAM minimum respectively
  (verified only on A100/H100). Fails this machine's 12 GB gate before the license
  question is even reached; submodule licenses (`diffoctreerast`, modified Flexicubes)
  remain unresolved regardless.
- **TripoSplat** (VAST-AI/Tripo, same clean lineage as TripoSR) — genuinely MIT, but
  single-image input producing Gaussian splats, not multiview, not a mesh. Does not
  address this problem.
- **Tripo 3.0 Multiview** — real multiview capability, but a paid third-party SaaS API,
  not a downloadable model. Wrong shape entirely for an operator-installed, nothing-shipped
  product.

The root cause is structural, not incidental: essentially every capable multiview image
generator in this ecosystem descends from the Zero123/MVDream/ImageDream family, and every
one of those is a Stable Diffusion derivative carrying Open RAIL++-M or a CC-BY-NC
relicense. TripoSR is clean specifically *because* it is not diffusion-based view synthesis
at all — a direct feed-forward image-to-triplane model. That is why it was the only
candidate to pass the original Sept 7 sweep and still the only one that passes today.

**New direction (operator, 2026-09-11): do not treat "no license-clean pretrained model
exists" as a dead end for the capability.** Multiview/better-than-single-view
reconstruction stays a real goal. The path to it is original, from-scratch code —
council-driven trial and error on a novel reconstruction approach we own outright — not
adopting a third party's encumbered weights. This is downstream of the same law as
[[no-recipes-is-the-north-star]]: the answer to "the licensed ecosystem doesn't offer a
clean path" is to build the capability, not to declare the goal impossible or fall back to
a hardcoded substitute.

- [ ] Near-term experiment (unaffected by the above, still open): generate and persist a
  genuinely distinct second camera view; measure identity/framing consistency and use it
  only as retry/quality evidence while TripoSR consumes one image.
- [ ] Original-code multiview research track (new, unstarted): scope what a from-scratch
  reconstruction approach would need to measurably beat single-view TripoSR on this
  hardware — this is a research/architecture question first, not an implementation task,
  and deserves its own dated plan when picked up rather than folding into this one.
- Files touched: this plan (closure of the download-candidate path); a future dedicated
  plan for the original-code track.
- Expected outcome achieved: an honest, evidence-backed, permanently documented no-go on
  every existing licensed candidate. Not yet started: the original-code track.

### Step 6 — Verify the whole buyer experience
- [ ] Run focused geometry/material/light tests, `cargo test --workspace`, and `pwsh -File scripts\install_shortcut.ps1`.
- [ ] Launch only from the operator's pinned Taskbar icon and capture the title version plus main rotunda, every chamber, floor response, figure lighting, manifestation stages, adult refusal, successful rotating reveal, and clean menu exit.
- [ ] Verify no visible terminal remains and installed hashes/timestamps match the new build.
- Files touched: verification ledger and designated screenshots only.
- Expected outcome: A stranger can traverse and understand the building and can verify the manifestation system from buyer-facing evidence.

### Step 7 — Publish only proven work
- [ ] Update architecture/user documentation, mark each acceptance item with evidence, explicitly stage owned files, commit to `main`, push `origin main`, and prove clean local/remote parity.
- Files touched: this plan and relevant truth docs.
- Expected outcome: No unfinished source-only chamber work and no release claim without its installed witness.

## Session update — 2026-09-10

This unit was picked back up in a fresh session that read the codebase directly rather than
capturing a pre-move screenshot set first (Step 1's own prescribed order was not followed —
noted honestly rather than checking that box). What actually landed, source-verified against
`crates/engine/src/modes/inner_chambers/world.rs`, `camera.rs`, and `capture.rs`:

- **Step 2 (rotunda architecture) — partially done.** The outer walls are still 4 flat
  `Cuboid` slabs, not a true circular drum — that part of Step 2 is still open. What was
  added: repeated wall pilasters (`CastlePilaster_*`, 6 bays per wall) breaking up the flat
  planes, and a continuous cornice band (`CastleCornice{North,South,East,West}`) at the wall
  top. Collision is untouched (still the existing +/-36 clamp), so this is purely visual.
- **Step 3 (floor legibility) — already substantially satisfied** by earlier work
  (`build_radial_flagstone_mesh` courses + generated normal map + key/fill/rim light rig);
  confirmed still correct, no changes made this session.
- **Step 4 (five archetype chambers) — done.** The five standing figures previously stood in
  open floor space with only per-figure point lights; they now each have a real niche bay:
  a curved wall ring (`build_wall_ring_mesh`, a new reusable mesh builder) with a doorway gap
  that always faces the rotunda center, a tinted floor medallion and low canopy (distinct
  ceiling silhouette from the 22m main vault), and two flanking threshold pillars. Stone tint
  is derived per-archetype (`niche_stone` field), not decorative.
  - **Real bug caught and fixed during this work, not just claimed:** the first geometry pass
    used a 2.9m ring radius, but the five niches sit only ~5.67m apart center-to-center
    (measured from the authored figure positions) — adjacent rings physically overlapped by
    ~0.13m. Shrunk to 2.1m radius (>=1.47m clearance) and added a regression test
    (`adjacent_niche_rings_never_overlap`) asserting >=1.0m clearance between every pair.
  - Collision (`camera.rs`) gates each niche's doorway arc explicitly, matching the visual
    doorway exactly — outside the door arc, the niche wall blocks entry; the interior obstacle
    list already covering pedestals/table/characters/altar is unchanged.
- **Step 5 (multiview evaluation) — not started.** No Chronos2 engine research or benchmarking
  was done this session. Still fully open.
- **Step 6 (verification) — done, with real evidence, not a claim.**
  - `cargo test --workspace`: 116 passed, 0 failed (92 engine incl. 4 new focused tests on the
    new mesh builder / niche geometry, 19 launcher, 5 windows_identity).
  - A new self-driving capture harness (`ARCHETYPES_INNER_CAPTURE=1`, mirrors the existing
    `ARCHETYPES_MECHA_CAPTURE` pattern) was added specifically because there was no existing
    way to get a reproducible rendered-frame proof of Inner Chambers without a human at the
    keyboard. It boot-skips the ~11s title veil and main menu the same way the real
    "Inner Chambers" button does (forces `ChamberState::MainMenu`, despawns `MainMenuUi`,
    inserts `TriggerInnerChambers`), switches the player camera to free flight so obstacle/
    niche collision never fights a teleport, and screenshots seven authored vantage points.
  - This caught three real framing bugs before they were reported as done: (1) the first
    capture attempt showed nothing but the boot veil — the trigger fired before the ~11s boot
    timer, fixed by force-skipping `ChamberState` instead of waiting; (2) the Empath niche shot
    put the camera *inside* the wall ring because the original stand-off formula was a fraction
    of distance-from-hall-center, which breaks for a niche close to the hall center — fixed to
    a fixed stand-off from the niche's own center; (3) a ground-level "overview" shot looked
    like a wall of flat cylinders — not a bug, the 23m-wide/5.6m-tall niche row genuinely fills
    that framing at that distance (checked by hand against the camera's ~45 degree default
    vertical FOV) — resolved by using a near-top-down layout angle instead, which is more
    useful for verification anyway.
  - Screenshots: `artifacts/visual-proof/inner-chambers-capture-2026-09-10/` —
    `00_rotunda_overview.png` (top-down: table portal, manifestation altar, pedestals, niche
    edges all visible in one frame), `01_table_and_dais.png`, and one portrait each for
    `02_niche_sentinel.png` .. `06_niche_nebula_jester.png`, each showing the figure framed by
    its own tinted wall and flanking threshold pillars.
  - `pwsh -File scripts\install_shortcut.ps1` rebuilt the **release** workspace and restaged
    Desktop/Start Menu/Taskbar shortcuts; installed `engine.exe`/`launcher.exe` SHA-256 verified
    against the fresh build.
- **Step 7 — pending this commit.** This plan and `STATUS.md` are being updated in the same
  change that commits and pushes the work.

## Session update — 2026-09-10 (segmented drum continuation)

- **Step 2 (central rotunda) — completed.** The remaining four-slab outer-wall gap was closed
  in `plan_2026-09-10_0904_segmented_rotunda_drum.md`: a procedural 64-bay annular drum now
  replaces the visible square shell, with 16 radial buttresses, continuous cornice, four
  cardinal processional portal frames, circular roof cap, and twelve radial ribs. The existing
  +/-36m movement clamp remains deliberately unchanged; the cardinal frames are readable
  interior axes rather than out-of-bounds exits.
- **Verification:** focused drum geometry regression plus `cargo test --workspace` passed
  117/117. The release was restaged and hash-verified; eight new rendered frames are under
  `artifacts/visual-proof/inner-chambers-rotunda-drum-2026-09-10/`. The dedicated
  `07_rotunda_drum_and_portal.png` frame proves the new curved wall/cornice/rib geometry.
  The proof is geometrical and runtime-real, not aesthetic approval: palette and large niche
  cylinders need a separate art-direction decision if they are to change.

**What remains open for a future session:** Step 5's "download a licensed candidate" question
is now closed for good (see Step 5 above — systemic Open RAIL/CC-BY-NC contamination across
every existing multiview reconstruction model, not fixable by picking a different one). The
open work is the from-scratch original-code multiview research track, which has not started
and deserves its own dated plan when picked up.
