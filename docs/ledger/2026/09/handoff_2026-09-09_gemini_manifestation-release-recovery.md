# Gemini Handoff: Manifestation Pipeline and Broken Release Recovery

**Date:** 2026-09-09  
**Repository:** `C:\archetypes`  
**Required branch:** `main` only  
**Current handoff commit before this document:** `e89ee15`  
**Operator:** Michael Holt  

## Read this first

Read `C:\archetypes\AGENTS.md` in full before editing anything. Its rules are
binding: plan first, never delete, never disguise a stub, use `main` only, run
the risk-matched verification gate, refresh the buyer launcher for player-facing
work, and push every completed unit to `origin/main`.

This is a recovery job. Do not trust prior statements that the manifestation
pipeline or release is complete. Establish truth from files, processes,
receipts, generated artifacts, installed paths, and a foreground buyer launch.

## What Michael intended

The Council Chamber manifestation pedestal must accept an arbitrary player
prompt, send that exact request into Chronos2 Object mode under Sentinel,
receive a newly generated object, convert the returned geometry into a valid
`.glb`, import it into the running game, place it on the manifestation pedestal,
and reveal it with a convincing magical machine effect.

This is governed by an absolute **NO-RECIPE LAW**:

- Chronos2 and Archetypes may not satisfy a manifestation request using a
  keyword recipe, noun classifier, canned mesh generator, premade script,
  primitive substitution, or fallback object.
- A user-supplied premade mesh is allowed only as an explicit import path. It
  must never be confused with generated manifestation.
- Once Sentinel clears an Object-mode prompt, Chronos2 must attempt that exact
  prompt with the required generative model. Service/model unavailability is a
  visible execution failure, never permission to return a recipe.
- Religious or political imagery is not to be silently rejected by an invented
  game-side classifier. Only the real published Sentinel prohibited-use policy
  governs refusal, and the buyer must see the actual refusal reason.

The chamber also needs a materially visible stone floor with relief/bump detail
that reacts to lighting, truthful floating wait indicators, cancellation that
terminates the entire generation process tree, no stray terminal window, a
magical production/reveal sequence, and a permanent version/build identifier in
the native window title.

## Current repository state

Relevant commits, newest first:

```text
e89ee15 build(release): publish signed 1.0.4
f414259 feat(release): display version in native title
b2e47af fix(deploy): verify pinned launcher installation
e0291be fix(manifestation): hide Chronos console and cancel child tree
d2e7397 feat(manifestation): route pedestal through Chronos Object mode
d036dd6 feat(chamber): add lit basalt floor relief
```

### Source work that exists but is not accepted as complete

`crates/engine/src/modes/inner_chambers/manifestation.rs` invokes:

```text
C:\chronos2\target\release\chronos.exe first-light
  --prompt <player prompt>
  --out-dir <unique temporary bundle>
  --geometry-forge
  --void
```

It then expects Chronos output under `engine_mesh`, invokes
`scripts/import_chronos_object.py` to perform a generic OBJ-to-GLB conversion,
and stages the GLB for game loading. Audit the precise receipt/path assumptions
against the current `C:\chronos2` Object-mode implementation. Do not assume the
contract matches merely because both sides compile.

`scripts/manifest_artifact.py` still exists because repository law forbids
deletion. Its old keyword recipes were retired by making its entry point fail
closed. Prove that no reachable runtime path, test helper, fallback, or manifest
still invokes or accepts recipe output. Search both repositories broadly.

The manifestation worker uses Windows `CREATE_NO_WINDOW` and tracks the Chronos
PID. Escape/teardown attempts `taskkill /PID <pid> /T /F`. Verify this against a
real running prompt and confirm no `chronos.exe`, `conhost.exe`, Windows
Terminal, Blender, or child process remains after cancel or game exit.

A post-success smoke/lightning reveal was added, but it has not been accepted
in a successful end-to-end buyer run. Treat it as implemented but unverified.

The waiting HUD currently displays generic copy such as `Chronos2
rendering...` plus elapsed seconds. Chronos actually emits lines like:

```text
[chronos-stage] id=sentinel pct=6 state=done msg=...
[chronos-stage] id=forge_plan pct=52 state=begin msg=...
```

The current Rust worker waits for complete command output rather than streaming
these stages into the game in real time. Therefore the floating indicator and
HUD remain theatre. Replace the buffered process handling with live stdout and
stderr readers, parse the real stage protocol, and drive the visible state from
those events. Do not invent timing percentages.

The floor has a normal map from commit `d036dd6`, but Michael reported that the
visual change was barely visible. Treat the floor goal as failed visual
acceptance. It needs visible raised/recessed stone courses or tessellated/mesh
relief plus a proper PBR normal response under chamber lights. Verify from the
actual player camera, not from code or texture existence.

### Version-title work

`crates/engine/build.rs` reads `installer/version.json` and injects the formal
product version and build serial into Rust. `crates/engine/src/main.rs` now uses
a native title shaped like:

```text
Archetypes 1.0.4 (build 5) — Council Chamber
```

There is a test for this contract. The title behavior has not yet been seen in
a successful buyer launch because the new formal installer fails in the
launcher before the engine starts.

## Critical current failure: 1.0.4 is broken

Do not call `1.0.4` shipped or usable.

The signed installer exists at:

```text
C:\Users\m\Downloads\Archetypes_Setup_1.0.4.exe
```

Its recorded SHA-256 is:

```text
2d9e25b36a1310f6e7a547ce119117679988d27650b30f0084a1bb3a77b7fba5
```

The installer, engine, and launcher have valid Azure Artifact Signing
signatures for `CN=Michael Holt`. The installed engine at
`C:\Program Files\Archetypes\engine.exe` reports file/product version
`1.0.4.5`. The pinned Taskbar shortcut currently targets:

```text
C:\Program Files\Archetypes\launcher.exe
```

Those facts do **not** make the release usable.

The buyer launch at 17:05 produced:

```text
Archetypes voices missing

The offline council voices are not installed and repair failed:
Access is denied. (os error 5)
```

Evidence is at:

```text
%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\last-failure.txt
```

Exact root cause:

1. `installer/archetypes_setup.iss` packages the engine, launcher, icon,
   assets, and `version.json`, but it does not package the `speech` tree.
2. It does not package `scripts/dependencies.json`, which the launcher repair
   path expects beside the installed launcher.
3. `scripts/install_shortcut.ps1` prepares a working `dist\speech` tree, but
   the formal installer lane never includes that tree.
4. When voices are absent, `crates/launcher/src/main.rs` attempts repair into
   `<launcher directory>\speech`. For a formal installation that means
   `C:\Program Files\Archetypes\speech`, which a normal user cannot create or
   modify. Windows returns error 5 before the engine can launch.
5. Therefore the new version title never appears and no manifestation testing
   can occur.

There is also a stale per-user install registration at
`%LOCALAPPDATA%\Programs\Archetypes`. Do not remove it automatically because
repository law says never delete and uninstall scope requires care. Ensure the
pinned shortcut and all acceptance evidence identify the formal Program Files
installation. Document the duplicate installation and propose a recoverable
cleanup path if needed.

## Required recovery sequence

### 1. Start with a new plan and audit both installation lanes

Create a dated plan under `docs/ledger/<YYYY>/<MM>/` before edits. Inspect:

- `installer/build.ps1`
- `installer/archetypes_setup.iss`
- `scripts/setup_windows.ps1`
- `scripts/install_shortcut.ps1`
- `scripts/install_product.ps1`
- `scripts/bind_taskbar_to_formal_install.ps1`
- `scripts/dependencies.json`
- launcher speech discovery/repair code
- engine speech discovery code

Define one canonical speech layout that works in both fast and formal lanes.

### 2. Repair formal speech packaging and writable recovery

The formal signed installer must contain or deterministically install the
declared offline speech runtime and Kokoro model. A practical implementation is
to have the formal build prepare/verify `dist\speech` through the existing
idempotent dependency bootstrap, then package that verified tree and
`scripts/dependencies.json` into the installer. Do not assume this suggestion is
correct without auditing installer size, source paths, and setup behavior.

Defense in depth: launcher repair must target a user-writable canonical data or
runtime directory under `%LOCALAPPDATA%\NeuroCognica\Archetypes`, not Program
Files. Both launcher readiness and engine TTS resolution must recognize the
same path. A missing voice package should show a buyer-facing progress/error
surface, not open raw Notepad as the primary experience.

Add tests for:

- formal package declarations include speech and dependency manifest;
- writable repair root selection for Program Files launches;
- launcher and engine agree on speech-root precedence;
- missing/corrupt speech fails with a buyer-readable message;
- installer payload readiness using the exact installed paths.

### 3. Finish the no-recipe manifestation pipeline

Audit `C:\chronos2` first. Read its `AGENTS.md` and its Object-mode/no-recipe
law. Preserve its dirty tree and use its own required plan and verification
rules. Do not change Chronos2 merely to make Archetypes tests convenient.

Trace one unique nonce-bearing player prompt through:

```text
in-game prompt editor
  -> exact subprocess arguments
  -> Chronos2 Sentinel receipt/decision
  -> real Object-mode model attempt
  -> generated source mesh and generation receipt
  -> generic headless conversion
  -> validated GLB
  -> game asset load
  -> pedestal placement
  -> reveal effect
```

For every arrow, retain evidence: prompt digest/nonce, paths, timestamps,
receipt fields, process exit status, mesh hash, GLB hash, and runtime entity
load result. The output must be demonstrably prompt-specific and newly
generated. No primitive or recipe fallback is permitted under failure.

Search for recipe/fallback/theatre surfaces with at least:

```powershell
rg -n -i "recipe|preset|fallback|primitive|keyword|manifest_artifact|mock|stub" C:\archetypes C:\chronos2
```

Classify each match rather than blindly deleting it. Add fail-closed tests that
prevent reintroduction of reachable recipe behavior.

### 4. Make progress, cancellation, and effects real

Stream Chronos stage events while the child is running. Drive the floating
icons and context bubble from real `id`, `pct`, `state`, and `msg` values.
Required buyer-visible states include at least request accepted, Sentinel
review, Sentinel refusal with reason, service/model preparation, view/image
generation, geometry generation, conversion, import, placement, reveal,
failure, timeout, and cancellation. Only show states Chronos actually reports
or Archetypes itself actually performs.

Tune the icon animation against real timestamps. Do not use a decorative timer
as evidence of progress. Provide a clear cancel action and verify the entire
child tree exits. Closing the game must also terminate active work without
leaving a console or terminal window.

Build a coherent manifestation effect within Bevy's current capabilities:
charging pedestal light, energy/plasma buildup, controlled lightning strike,
volumetric-looking smoke/cloud particles, then smoke dissipation that reveals
the imported object. Effects must be tied to actual pipeline stages and must
remain performant on the RTX 3060 target. The object should not appear before
the successful import/placement event.

### 5. Make the floor visibly physical

Retain light-reactive normal detail, but add geometry/material contrast strong
enough to read from the normal chamber camera. Capture before/after screenshots
under identical camera and lighting. Michael's visual acceptance is required;
a texture file and passing unit test are not sufficient.

### 6. Cut a new formal release only after the installed runtime passes

Do not reuse or overwrite 1.0.4. Increment both semantic version and
`build_serial`—the likely next identity is `1.0.5` / build `6`, but verify the
current ledger immediately before editing.

Commit and push the complete tested source **before** building the formal
installer so `release.json.git_commit` names the source that actually produced
the binary. Then:

1. run `cargo test --workspace`;
2. run the dependency/payload readiness tests;
3. run `scripts/check_signing_ready.ps1`;
4. run `installer/build.ps1` with signing enabled;
5. verify the Downloads artifact exists and matches `release.json` SHA-256;
6. verify installer, engine, launcher, and uninstaller signatures are valid and
   timestamped;
7. install the exact Downloads installer as a buyer;
8. verify installed version, signature, speech payload, dependencies manifest,
   registry version, and Taskbar target;
9. launch from the existing pinned Taskbar icon;
10. capture a screenshot whose native title visibly shows the new version and
    build;
11. verify a voice plays;
12. run a real unique manifestation prompt to successful GLB placement and
    reveal;
13. cancel a second real prompt and prove no child process remains;
14. quit through the menu and prove no engine, launcher, Chronos, Blender,
    conhost, or Windows Terminal process remains.

If any step fails, stop the release, record the failure, and do not advance the
history entry to `package_available: true`.

## Required final evidence

Gemini's completion report must distinguish:

- source implemented;
- unit/integration tested;
- signed artifact produced;
- exact artifact installed;
- pinned Taskbar launch witnessed;
- real Chronos generation witnessed;
- GLB imported and placed;
- visual quality accepted by Michael.

Include exact commands, test counts, commits, hashes, signature subject,
version/build, installed paths, process evidence, receipt paths, GLB path/hash,
and screenshots. Do not collapse these categories into the word “done.”

## Canonical files to read

```text
C:\archetypes\AGENTS.md
C:\archetypes\docs\architecture\CHRONOS_TO_GAME_ASSET_PIPELINE.md
C:\archetypes\docs\windows\VERSIONING_AND_RELEASES.md
C:\archetypes\docs\windows\AZURE_TRUSTED_SIGNING.md
C:\archetypes\docs\ledger\2026\09\plan_2026-09-09_1440_real-manifestation-bridge.md
C:\archetypes\docs\ledger\2026\09\plan_2026-09-09_1700_versioned-signed-title-release.md
C:\archetypes\docs\ledger\audits\audit_2026-09-09_deployment-claim-failure.md
C:\archetypes\crates\engine\src\modes\inner_chambers\manifestation.rs
C:\archetypes\crates\engine\src\modes\inner_chambers\world.rs
C:\archetypes\crates\engine\src\main.rs
C:\archetypes\crates\engine\build.rs
C:\archetypes\crates\launcher\src\main.rs
C:\archetypes\installer\build.ps1
C:\archetypes\installer\archetypes_setup.iss
C:\archetypes\scripts\setup_windows.ps1
C:\archetypes\scripts\dependencies.json
C:\archetypes\scripts\import_chronos_object.py
C:\archetypes\scripts\manifest_artifact.py
C:\chronos2\AGENTS.md
```

## Bottom line

The current formal release is signed but broken at startup. The manifestation
bridge has meaningful source work but no accepted end-to-end proof. The wait UI
is not driven by live stages, the reveal has not been witnessed after a real
generation, and the floor failed visual acceptance. Fix the installed buyer
runtime first, then prove the complete no-recipe manifestation chain before
cutting another signed release.
