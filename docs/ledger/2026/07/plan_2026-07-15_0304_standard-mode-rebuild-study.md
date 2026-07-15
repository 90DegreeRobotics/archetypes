# Plan: Standard Mode Rebuild Study — 2026-07-15 03:04

## Status
COMPLETED

## Goal
Study the current Archetypes scene failure and the older local council/chat interface code before deciding whether to keep patching Standard Mode or rebuild it around a real full-scale chamber. The output should answer what broke, what should be rebuilt, what can be reused from prior interfaces, and what the new scene contract must be.

## Steps
### Step 1 — Name the current failure honestly
- [x] Action: Inspect the current Standard Mode rendering code and summarize why the council frame became disconnected: lifted spheres, panel billboards, oversized crystal, noisy sky, and weak environment grounding.
- Files touched: this plan only.
- Expected outcome: The failure is tied to concrete systems, not vague taste.

Finding: the current Standard Mode frame is not failing because one number is wrong. It is failing because the scene has no single authored visual hierarchy. `Council_Assembly` is pinned to `COUNCIL_CENTER` in `spheres.rs`, but the `*_PanelSpinner` roots are separate top-level nodes and are lifted by a second name-matching rule. `panels.rs` then billboards those panel roots to the camera, `star.rs` creates a runtime `SolidStar` from the current constellation bounds, `sky.rs` generates a bright procedural star cubemap, and `camera.rs` swings toward the active speaker. Each system is locally understandable. Together they produce the screenshot failure: detached portraits, dark floating vessels, noisy sky, a giant flat blue star/crystal blocking the subject, and no grounded council-room composition.

The direct answer to "how did it get decided to move the sphere up and forget to move the panels" is: the code made that class of mistake easy. The spheres and portrait panels were not one authored object with one parent root. They were patched back together at runtime with string-name rules. That should stop.

### Step 2 — Study prior council/chat interfaces
- [x] Action: Inspect the local legacy directories the operator named: `C:\AURA-1\archive\frontends\archived_20260130_080910\frontend\src` and `C:\mecha\aura-mechanician`.
- Files touched: none.
- Expected outcome: Identify reusable interaction patterns, archetype persona/theme handling, council selection, and any UI motion ideas worth carrying into Archetypes.

Reusable patterns:
- `C:\AURA-1\archive\frontends\archived_20260130_080910\frontend\src\morphing\types.ts` has an `ARCHETYPE_REGISTRY` with stable ids, display names, colors, symbolism, elements, and geometry types. Archetypes should get one canonical metadata registry in Archetypes instead of scattered color/name constants.
- `C:\AURA-1\archive\frontends\archived_20260130_080910\frontend\src\morphing\archetype-shapes.ts` gives each archetype a visual grammar and transition profile. Do not port OS-window morphing directly, but do reuse the concept for chamber portals, speaker highlights, and per-archetype room signatures.
- `C:\AURA-1\archive\frontends\archived_20260130_080910\frontend\src\clients\councilClient.ts`, `aiClient.ts`, and `C:\AURA-1\backend\src\council_verdict.rs` show the right event boundary: typed council envelopes and token/delta streams. The Bevy scene should react to explicit state like `speaker_changed`, `archetype_changed`, `verdict`, or `deliberation`, not infer meaning from mesh transforms.
- `C:\mecha\aura-mechanician\frontend\src\utils\ThemeManager.js` and `AssetLoader.js` centralize theme switching and portrait/icon assets. Archetypes should do the same: one asset lookup path for portraits/icons/reference geometry and one theme state per active/speaking archetype.
- `C:\mecha\aura-mechanician\backend\main.py` already had useful surface concepts: `/archetypes`, `/archetypes/{id}`, `/chat`, and `/council`. Those map cleanly to: inspect an archetype chamber, ask a single archetype, and convene the full council.

### Step 3 — Decide patch versus rebuild
- [x] Action: Compare the cost/risk of continuing to patch the current GLB/runtime hybrid against rebuilding the world from ground/sky upward.
- Files touched: this plan only.
- Expected outcome: A clear recommendation, with upstream/downstream risks and stop rules.

Recommendation: rebuild Standard Mode's visual world. Do not keep layering runtime patches over the current GLB/runtime hybrid.

Why: a patch can shrink the star, lower the spheres, hide the noisy sky, or reduce billboards, but that preserves the root problem: game meaning is split across authored meshes, generated meshes, detached panels, camera hacks, and name-string transforms. This will keep breaking whenever the table, camera, speaker, or game mode changes.

What can be changed safely now: visual scene assets, Standard Mode chamber/camera/panel/star systems, and docs. What must not be broken: Oracle Riddle scoring and ledger sealing, locked Inner Chambers state, desktop launch path, local LLM/TTS readiness, and the existing council transcript/audio data flow.

Negative system effect check:
- Risk: a full scene rebuild touches the same `chamber/*` systems that Standard Mode relies on. Mitigation: keep the ritual state machine and ledger services intact while replacing visual ownership in phases.
- Risk: new GLBs can silently break node binding. Mitigation: freeze a node-name contract and run headless GLB verification before any runtime wiring.
- Risk: desktop can remain stale. Mitigation: only claim user-visible completion after `scripts\install_shortcut.ps1` refreshes `dist` and the desktop path is witnessed.
- Risk: art passes can pass tests while still looking bad. Mitigation: screenshots/render review are mandatory gates, not optional evidence.

### Step 4 — Draft the new scene contract
- [x] Action: Define a rebuilt Standard Mode contract: ground, sky, star chamber, rounded table/council layout, archetype chambers/archways, speaking highlight behavior, and what must stay in view.
- Files touched: this plan only, or a follow-up rebuild brief if needed.
- Expected outcome: A cold-start build direction that prevents the same detached-transform failure.

## Rebuild Contract

### Build Order
1. Ground and sky first. No council table, no star chamber dressing, no game-mode polish until heaven/earth reads correctly in a screenshot.
2. Star chamber second. Full-scale circular chamber with stable floor, horizon, lighting, and archway rhythm.
3. Council table third. Rounded table in front of the Witness, with all seven archetypes visible from the main council camera.
4. Archetype chambers fourth. Seven archways lead to seven chamber entrances or separate chamber scenes. A player can enter a chamber to explore or ask a specific archetype.
5. Game-mode interactivity last. Modes consume the chamber contract; they do not invent their own visual hierarchy.

### Authored Hierarchy Rule
Every archetype's visible seat/portrait/light/chamber entrance must live under one authored root:

`Archetype_<id>_Root`

No more top-level portrait spinners that need to be chased after the sphere root moves. If an archetype moves, the portrait, speaker light, label anchor, portal marker, and interaction volume move with it.

### Proposed Node Contract
- `WorldRoot`
- `Ground_Earth`
- `Sky_Heaven`
- `StarChamber_Root`
- `StarChamber_Floor`
- `StarChamber_ArchwayRing`
- `CouncilTable_Root`
- `CouncilSeats_Root`
- `WitnessCamera_Home`
- `WitnessCamera_Council`
- `WitnessCamera_Table`
- `Archetype_<id>_Root`
- `Archetype_<id>_Seat`
- `Archetype_<id>_Portrait`
- `Archetype_<id>_SpeakerLight`
- `Archetype_<id>_ChamberEntrance`
- `Archetype_<id>_InteractionVolume`
- `Archetype_<id>_Camera`

Accepted ids: `architect`, `sentinel`, `mentor`, `explorer`, `oracle`, `empath`, `jester`.

### Council Presentation
- All archetypes stay in view during council deliberation.
- Speaking does not trigger a wild orbit. It triggers material/light emphasis on the speaking archetype and, at most, a slight dolly or focal-length change from the stable council camera.
- The highlight should be readable without motion: rim light, portrait frame glow, table sigil glow, or chamber-entrance glow.
- The star/crystal is either authored as a deliberate object with scale limits or omitted from the first rebuild. No runtime-generated star should be allowed to block the council.
- Portraits/images can stay, but they must look seated/framed in the chamber, not pasted onto loose rectangles in space.

### Scene Asset Strategy
- Start with a Blender-authored `ground_sky` or replacement `uiscene1.glb` proof that contains only ground, sky/environment, scale references, and camera markers.
- Add chamber geometry after ground/sky approval.
- Add table after chamber approval.
- Add archetype chambers as separate GLBs or authored sub-roots only after the council room is stable.

### Stop Rules
- No claim of visual completion without screenshots from the runnable app.
- No node contract changes without updating verification.
- No new mode unlock while Standard Mode is visually broken.
- No runtime relocation of separate archetype parts by string-name patching.
- No self-approval: a passing `cargo test` is necessary for commits, but screenshots are the product gate.
