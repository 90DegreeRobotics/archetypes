# Handoff: Archetypes Standard Mode / Mecha Main Menu Rewrite - 2026-07-15

## Read This First
You are starting fresh in `C:\archetypes`.

The current task is **not** to polish the rejected old Standard Mode. The current task is to build the next Standard Mode direction:

- Boot: black fade in `ARCHETYPES`.
- Then fade in `A GAME BY MICHAEL HOLT`.
- Then a slow fade into a real, game-feeling main menu.
- Main menu should show the beautiful council scene/table, likely from a slightly higher table angle, with the menu living in the world rather than pasted on top.
- Standard Mode must become a hard-coded Bevy game feature where the player can chat with archetypes.
- The chat/selector/visual behavior must carry forward from `C:\mecha\aura-mechanician\frontend\src`.
- The old Standard Mode must be preserved for reference, not deleted.

Base state verified before this handoff:

- Repo: `C:\archetypes`
- Branch: `main`
- Remote: `origin/main`
- Latest commit at handoff creation: `a9ef5e8 Adopt risk-based verification gates`
- Working tree at handoff creation: clean

## Required Reading
Read these files before making code changes:

1. `AGENTS.md`
   - Root repo law. Main branch only. No deletion. Push completed units. Risk-based verification gates.

2. `STATUS.md`
   - Current truth surface. Standard Mode is a rejected baseline. Mecha audit is the next source-canon path.

3. `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`
   - Full audit of `C:\mecha\aura-mechanician\frontend\src`. This is the source canon for selector/chat/theme/assets.

4. `docs/ledger/2026/07/plan_2026-07-15_0304_standard-mode-rebuild-study.md`
   - Explains why the current Standard Mode failed structurally and why patching is the wrong move.

5. `docs/ledger/2026/07/plan_2026-07-15_0412_speed-rules-risk-based-verification.md`
   - Explains the new verification policy. For the actual implementation, Rust/runtime changes still require `cargo test --workspace`.

## Operator Intent
Michael's current direction, translated into implementation language:

- The intro already moved away from the candle video. Keep it black and ceremonial.
- After `ARCHETYPES` and `A GAME BY MICHAEL HOLT`, the game should fade into a main menu that feels like a video game.
- The main menu should use the beautiful new scene, not a flat overlay.
- Use a slightly higher angle on the Flower-of-Life / portal table so the table reads as beautiful and important while the chamber still reads as a place.
- Menu choices should feel physically tied to the table/portal/chamber.
- Archetypes should let the player chat with archetypes.
- When chatting with an archetype, it should behave and look like the old Mecha frontend.
- Do not decide that only a small subset of Mecha matters. The Mecha folder represents prior work Michael does not want forgotten.

## Main Menu Visual Target
When the black title veil fades out, the player should see:

- A stable hero shot of the council chamber.
- The Flower-of-Life / portal table in the foreground or midground.
- Camera slightly higher than the current low table angle, pitched down enough to reveal the table design.
- Chamber arches and council seating visible behind the table.
- No giant runtime star/crystal blocking the view.
- No loose portrait rectangles floating detached from their archetypes.
- No debug-style HUD clutter.

The menu should live in the world:

- Primary action: `ENTER COUNCIL` or the eventual Mecha Standard entry, centered in or just above the portal glow.
- Secondary action: `ORACLE RIDDLE`, placed as a table-ring glyph or secondary option.
- Locked modes: dim sealed glyphs, unlit archways, or cold inactive table marks. They should look intentional, not broken.
- Settings/quit, if present, should be visually low priority.

Video-game feel should come from:

- subtle idle camera drift,
- portal/table pulse,
- torch/light flicker,
- hover/selection glow,
- menu input sound/visual response,
- locked-mode feedback that feels sealed rather than dead,
- smooth fade from title veil into the scene.

## What Is Wrong With Current Standard Mode
Do not spend the next pass tuning one number unless it directly supports the rebuild.

Current failure summary from the rebuild study:

- `Council_Assembly` is pinned to `COUNCIL_CENTER`, but `*_PanelSpinner` roots are separate top-level nodes.
- Runtime string-name rules move some things but not all associated visual pieces.
- `panels.rs` billboards portrait roots to camera.
- `star.rs` creates a runtime `SolidStar` from current constellation bounds.
- `sky.rs` generates a noisy procedural cubemap.
- `camera.rs` swings toward active speakers.
- Together these systems produced the rejected screenshot: giant flat blue crystal/star blocking the subject, dark lifted spheres, detached portrait billboards, noisy sky, weak chamber/table grounding.

The fix is a coherent authored/game-owned presentation hierarchy, not more detached runtime patching.

## Mecha Source Canon
Source folder:

`C:\mecha\aura-mechanician\frontend\src`

The audit found 36 files:

- `index.html` - live Electron UI shell: splash, chat, portrait panel, health monitor, docs modals, archetype switcher.
- `mindplane.js` - live consciousness selector: `uxbacklayer.png`, seven nodes, hover glow/cards, QSIC pulse, platonic solids, selection transition.
- `main.js` - Electron shell. Do not port Electron; translate behavior into native Bevy.
- `utils\ThemeManager.js` - dynamic theme switching.
- `utils\AssetLoader.js` - portrait/icon/logo/council asset loading.
- `components\*.tsx` - reference React mindplane/provenance components. `MindplaneOverlay.tsx` is syntactically damaged; preserve design intent, not raw code.
- `themes\*.css` - per-archetype visual signatures.
- `assets\*.png` - 19 image assets including portraits, icons, logo, council reference images, and `uxbacklayer.png`.

Important: `mindplane.js` selector colors and CSS theme colors disagree for Mentor, Oracle, and Jester. Do not silently normalize that. Treat it as a source fact and make an explicit choice in implementation notes.

## Existing Archetypes Code Map
Likely files to inspect before implementation:

- `crates/engine/src/main.rs`
- `crates/engine/src/chamber/mod.rs`
- `crates/engine/src/chamber/boot.rs`
- `crates/engine/src/chamber/camera.rs`
- `crates/engine/src/chamber/ritual.rs`
- `crates/engine/src/chamber/council.rs`
- `crates/engine/src/chamber/panels.rs`
- `crates/engine/src/chamber/portal.rs`
- `crates/engine/src/chamber/spheres.rs`
- `crates/engine/src/chamber/star.rs`
- `crates/engine/src/chamber/sky.rs`
- `crates/engine/src/modes/game_mode.rs`
- `crates/engine/src/modes/mod.rs`
- `crates/engine/src/modes/oracle_riddle/*`
- `crates/engine/src/services/llm.rs`
- `crates/engine/src/services/ledger.rs`
- `crates/engine/src/services/paths.rs`
- `crates/engine/src/theme/constants.rs`
- `scripts/install_shortcut.ps1`

Current facts:

- `GameMode::Standard` is available.
- `OracleRiddle` is available and must not be broken.
- `InnerChambers` and `LivingEngine` are locked and must remain locked.
- Local Ollama calls already exist through `services::llm`.
- Hash-chained ledger exists through `services::ledger`.
- Desktop shortcut launches `C:\archetypes\dist\launcher.exe`, not the live repo binary.

## Recommended Implementation Shape
Create a focused implementation plan first, then build.

Recommended plan filename:

`docs/ledger/2026/07/plan_<timestamp>_mecha-standard-mode-implementation.md`

Recommended architecture:

1. Add a new hard-coded Standard implementation under:
   `crates/engine/src/modes/standard_mecha/`

2. Add a Bevy-native state machine for:
   - `MainMenuScene`
   - `MechaSelector`
   - `MechaChat`
   - `MechaSwitching`

3. Keep old chamber/ritual code in repo as reference. Do not delete it.

4. Route `GameMode::Standard` into the new Standard/Mecha state path once the new path exists.

5. Mirror the Mecha asset set into the Archetypes asset tree, likely under:
   `assets/mecha/`

6. Add an explicit Rust registry equivalent to Mecha's metadata:
   - id,
   - display name,
   - subtitle,
   - element,
   - selector coordinate,
   - selector role,
   - selector color,
   - CSS/theme colors,
   - portrait path,
   - icon path,
   - platonic solid id.

7. Add tests proving all seven archetypes are represented and all referenced assets exist.

8. Build the main menu as a table/chamber scene presentation, not a flat button panel.

9. Build archetype chat with real local LLM calls and fail-visible errors. No canned stub responses.

10. Persist chat history through a real local store or ledger-backed path under `%LOCALAPPDATA%\NeuroCognica\Archetypes`. Do not present memory as durable unless it is actually durable.

## Scope Fences
Do not:

- create a branch,
- delete old Standard Mode code,
- remove Mecha source references from the plan,
- unlock Inner Chambers or Living Engine,
- change Oracle Riddle scoring/gameplay,
- silently weaken ledger hash-chain behavior,
- depend on Electron, Node, DOM APIs, or browser `localStorage`,
- claim desktop behavior until `dist` is refreshed and the desktop launch path is witnessed,
- present stubs as complete,
- call a visually weak result done because tests pass.

Do:

- work on `main`,
- keep the user updated while working,
- explain upstream/downstream impact before risky changes,
- commit and push completed verified units,
- use screenshots as the product gate for visual work,
- preserve the old implementation as reference until Michael explicitly approves cleanup.

## Verification Gates
Because the actual implementation will touch Rust/runtime/player-facing behavior, the final implementation gate must include:

```pwsh
cargo test --workspace
```

Likely additional gates:

- targeted tests for new registry/chat/history code,
- asset existence checks for all Mecha portraits/icons/logo/uxbacklayer,
- `cargo build -p engine`,
- `scripts\install_shortcut.ps1` when ready to refresh desktop,
- launch from the Desktop shortcut or `dist\launcher.exe`,
- screenshot proof from the actual runnable app.

Required screenshots/witness frames:

- black title intro with `ARCHETYPES`,
- subtitle with `A GAME BY MICHAEL HOLT`,
- faded-in main menu hero table shot,
- hover/selection state on primary menu action,
- Mecha-style selector or archetype selection surface,
- at least two archetype chat views,
- archetype switch state,
- real chat send success or honest local-LLM failure,
- Oracle Riddle still reachable and working,
- locked modes still locked.

Docs-only work can use the new targeted gate, but this implementation cannot skip cargo or visual witnessing.

## Stop Rules
Stop and report if:

- the Mecha asset source is missing,
- copying assets would overwrite unrelated existing assets,
- Standard routing requires deleting old chamber code,
- Oracle Riddle becomes affected,
- desktop launch path cannot be refreshed or witnessed,
- LLM/chat is not real,
- the visible result looks like debug UI rather than a game menu.

## Copyable Prompt For Fresh Instance
Paste this into the fresh instance if needed:

```text
You are working in C:\archetypes on main. Read AGENTS.md first. Do not create branches. Do not delete old Standard Mode. Latest base commit should be a9ef5e8 or newer; verify git status and origin/main before editing.

Goal: implement the next Archetypes Standard Mode direction. Boot is black title: ARCHETYPES, then A GAME BY MICHAEL HOLT, then slow fade into a real game-feeling main menu. The menu should show the beautiful council/table scene from a slightly higher table angle, with the menu living in the portal/table/chamber rather than flat HUD buttons. Standard Mode must become a hard-coded Bevy feature where the player can chat with archetypes. When chatting with an archetype it should behave/look like C:\mecha\aura-mechanician\frontend\src. That Mecha folder is source canon, not optional inspiration.

Required reads before code:
- STATUS.md
- docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md
- docs/ledger/2026/07/plan_2026-07-15_0304_standard-mode-rebuild-study.md
- docs/ledger/2026/07/handoff_codex_2026-07-15_standard-mode-mecha-main-menu.md

First create a dated implementation plan in docs/ledger/2026/07/. Then implement in focused units. Recommended architecture: new crates/engine/src/modes/standard_mecha/ module; mirror Mecha assets into assets/mecha/; create a Rust archetype registry with Mecha metadata/theme/asset paths; route GameMode::Standard into the new Mecha Standard state path once viable; preserve existing chamber/ritual code as reference.

Do not touch Oracle Riddle behavior. Keep Inner Chambers and Living Engine locked. Do not depend on Electron/Node/DOM/localStorage. Chat must use real local LLM/service behavior or fail visibly. History must be real if presented as persistent.

Verification for runtime implementation: cargo test --workspace, build engine, verify assets, refresh dist/desktop only when ready, and provide screenshots from the actual runnable app. Visual quality is a gate; passing tests alone is not enough.
```

## Final Response Expectations For The Implementing Instance
When the implementation unit is done, report:

- files changed,
- what behavior changed,
- what Mecha source surfaces were ported,
- what was intentionally deferred,
- upstream/downstream impact,
- exact verification commands and results,
- desktop/app witness status,
- commit hash and push status.

Do not say "done" unless the desktop-facing behavior was actually witnessed or the response clearly says it was not.
