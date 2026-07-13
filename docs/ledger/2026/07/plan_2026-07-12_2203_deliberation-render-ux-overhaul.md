# Plan: Deliberation/render UX overhaul — 2026-07-12 22:03

## Status
IN-PROGRESS

## Goal
The operator played the freshly-built desktop shortcut (`dist/engine.exe`, staged 21:11 today — confirmed not stale) and dictated a full punch list of what's actually wrong on screen, contradicting several "COMPLETED" claims in `STATUS.md`. This plan fixes, in order: the deliberation-screen camera/text/panel mess, the voice/text sync jar, the post-verdict jargon and static image, the default Comfy art style, and the table geometry — using the operator's own words as the spec. Root causes below were traced in the actual source before writing this plan, not guessed.

## Root causes traced before coding
- **Voice/text sync jar**: `crates/engine/src/chamber/council.rs::advance_speakers` advances `transcript.cursor` the instant the previous line's audio finishes; `crates/engine/src/chamber/ritual.rs::render_ritual_ui` reads `council.lines.get(council.cursor)` and reveals the bubble/transcript text for the new line immediately. The new line's TTS (`speech.rs::request_council_line`) only *starts* a background `sherpa-onnx-offline-tts.exe` process at that same moment — model load + synthesis takes real wall-clock time — so the text is on screen well before `CouncilVoiceState::phase` reaches `Playing`. Fix: gate text reveal on the voice phase, not the cursor.
- **Scattered text**: `spawn_ritual_ui` creates two independent absolutely-positioned text nodes — `TranscriptDrawer` at `left:24,top:24` and `RitualPrompt` at `right:22,bottom:18` — plus a separate `SpeakerBubble`. That's the upper-left/lower-right split the operator is describing. Needs consolidation into one top panel.
- **Jargon**: `render_ritual_ui`'s `ArtifactPending`/`ArtifactResult` arms literally say "Chronos is painting locally…" and dump `Artifact:` / `PNG:` / `Proof:` fields — internal debug language with no translation for a player.
- **Panel orientation**: `panels.rs::keep_panels_upright` recomputes the `_PanelSpinner` root's rotation every frame via `look_at(camera) + rotate_local_x(-FRAC_PI_2)`, fully overriding the Blender-authored rotation. The two faces are fixed local children (`Icon` at local `+Z`, `Portrait` at local `-Z`, per `scripts/author_temple_overhaul.py::author_panel`). The current formula happens to expose the `Icon` face on all seven spheres instead of the `Portrait` (the actual archetype artwork) — a single sign/offset error, wrong identically everywhere, matching the operator's report.
- **Off-kilter star during deliberation**: `camera.rs::drive_camera` has no dedicated case for `ChamberState::Deliberating` — it falls into the catch-all `_ => establishing`, the same wide Director establishing shot used for the main menu backdrop, not a composed deliberation frame.
- **Static artifact image**: `present_artifact_image`/`position_artifact_image` in `ritual.rs` place the returned PNG at a fixed `ARTIFACT_DISPLAY_SIZE` with no animation.
- **Locked gothic-sketch art style — found the actual bug**: `crates/engine/src/chamber/ritual.rs::request_chronos_artifact` sends `"fidelity": "final"` to Chronos's `pipeline/concept-thumbnail`. But `chronos_director::storyboard_prompt_from_req` (`C:\chronos\crates\chronos_director\src\main.rs:2555`) only special-cases `fidelity.eq_ignore_ascii_case("refined")`; any other value — including `"final"` — silently falls through to `"fast rough storyboard sketch, low-detail, gesture-first"`. Every single artifact render has been getting the rough-sketch treatment regardless of the `style` field. **This is fixable entirely inside Archetypes** — no Chronos repo edit needed for the style default.
- **Table geometry**: `assets/scenes/table.glb` (built 21:01 today, matches `dist/`) is the output of `scripts/prepare_table.py`'s procedural rebuild from `plan_2026-07-12_1945` (marked COMPLETED). Its thin gold torus rims and dark cylinder read as "barely visible outline" in the chamber's near-black lighting exactly as the operator describes — this is what's live, not a stale build. The `Stargate_Portal` disc (confirmed excellent by the operator) is generated in the same script and must be preserved untouched. Rebuilding the physical table shell needs the operator's reference image (mentioned but not yet attached).

## Steps

### Step 1 — Fix the voice/text sync jar
- [x] Action: In `speech.rs`, expose when audio has actually started (`CouncilVoicePhase::Playing`, first frame the `ArchetypeVoice` entity exists). In `ritual.rs::render_ritual_ui`, do not reveal a `CouncilSpeaking` line's bubble text or append it to the transcript drawer until that line's voice phase is `Playing` — show a neutral "forming…" state before that (already exists as `speech.line`). Add/adjust a test asserting the transcript never shows a line ahead of its voice phase.
- Files touched: `crates/engine/src/chamber/speech.rs`, `crates/engine/src/chamber/ritual.rs`.
- Expected outcome: the Witness never sees text before hearing the matching voice begin.

### Step 2 — One consolidated top HUD panel, plain language, bigger text
- [x] Action: Replace the two separate `TranscriptDrawer` (top-left) / `RitualPrompt` (bottom-right) nodes with a single top-anchored panel: larger font, a defined bordered container with the existing `Overflow::clip_y` scroll clipping made visually obvious (so future long transcripts read as scrollable, not cut off). Rewrite all player-facing copy to plain language: "Chronos is painting locally…" → something like "Turning the verdict into an image…"; drop the raw `Artifact:`/`PNG:`/`Proof:` id dump from the player-facing text (keep those in logs/`info!` only).
- Files touched: `crates/engine/src/chamber/ritual.rs` (`spawn_ritual_ui`, `render_ritual_ui`).
- Expected outcome: one legible panel, top of screen, in plain English, at every ritual stage.

### Step 3 — Correct archetype panel orientation
- [x] Action: Fix `keep_panels_upright` in `panels.rs` so the `Portrait` face (the actual archetype artwork) faces the Witness camera, not the `Icon` glyph face, on all seven spheres, remaining always upright. Verify empirically with a rebuilt binary + capture screenshot comparison (per this repo's own static-check convention), not by inspection alone.
- Files touched: `crates/engine/src/chamber/panels.rs`.
- Expected outcome: every sphere shows its archetype portrait, right-side-up, facing the Witness.

### Step 4 — Give deliberation its own composed camera frame
- [x] Action: Add a dedicated `ChamberState::Deliberating` camera pose in `camera.rs::drive_camera` (centered on the star, not the wide main-menu establishing shot). While rebuilding/capturing, also watch for the "flashing/buggy" artifact the operator described across a short run of consecutive frames (reuse the existing pixel-diff static-check technique) and fix if it reproduces.
- Files touched: `crates/engine/src/chamber/camera.rs`.
- Expected outcome: entering deliberation composes the star deliberately in frame; no flicker.

### Step 5 — Slow zoom-in on the returned artifact
- [x] Action: Animate the artifact image's displayed scale from a smaller starting size up to `ARTIFACT_DISPLAY_SIZE` over a couple of seconds after `present_artifact_image` fires, instead of appearing at full size instantly.
- Files touched: `crates/engine/src/chamber/ritual.rs`.
- Expected outcome: the reveal draws the eye in rather than popping in static.

### Step 6 — Fix the art-style bug and shift the default toward modern/realistic
- [x] Action: Change `request_chronos_artifact`'s request body from `"fidelity": "final"` to `"fidelity": "refined"` (the value Chronos actually recognizes) and rewrite `artifact_prompt`/the `style` field away from the accidental rough-sketch result toward modern, realistic, readable imagery — no gothic/pen-sketch bias — while keeping it a single restrained artifact-grade image with no text.
- Files touched: `crates/engine/src/chamber/ritual.rs`.
- Expected outcome: returned artifacts are modern and realistic by default, not dark gothic sketches — confirmed by an actual generated image, not just the prompt text.

### Step 7 — Table geometry pass (blocked on operator reference image)
- [ ] Action: Once the operator resupplies the reference image, rebuild `scripts/prepare_table.py`'s physical table shell (top, rims, legs) to actually read clearly in the chamber's dark lighting, while leaving `Stargate_Portal`'s generation and animation untouched (operator confirmed it's excellent).
- Files touched: `scripts/prepare_table.py`, `assets/scenes/table.glb`.
- Expected outcome: table matches the reference and is clearly visible; portal untouched.

### Step 8 — End-to-end verification and truth docs
- [ ] Action: `cargo build -p engine`, run `ARCHETYPES_CAPTURE=1` deterministic walkthrough, visually confirm each fix against a fresh screenshot set, rebuild release + reinstall the desktop shortcut, update `STATUS.md`/`README.md` to match what's actually true, commit and push each completed unit as it lands (not batched at the end).
- Files touched: `STATUS.md`, `README.md`, build/install scripts as needed.
- Expected outcome: the live desktop shortcut reflects every fix; docs match reality; tree is clean on `origin/main`.

## Verification
- `cargo test --workspace`: 15 passed (14 engine, 1 launcher), 0 failed.
- Steps 1-6 verified against a rebuilt `target/debug/engine.exe` in `ARCHETYPES_CAPTURE=1` deterministic runs (not against `cargo test`, which only builds test harnesses) — real Ollama (`qwen2.5:7b-instruct`), real Kokoro/sherpa-onnx TTS (pointed at the installed `dist/speech` runtime), and a real Chronos/ComfyUI artifact render all in the loop:
  - A query-conflict panic (Bevy B0001: the `prompt` and `bubble` queries both gained `&mut Node` access with no filter proving them disjoint) was caught by actually running the binary — `cargo check`/`build` do not catch this — and fixed by adding `Without<SpeakerBubble>` to the `prompt` query.
  - Step 3's first attempted fix (a `rotate_local_y` sign flip) did **not** change anything when captured — the real mechanism is that Icon/Portrait are coplanar-ish planes with backface culling disabled, so whichever is physically nearer the camera wins the depth test regardless of face-normal direction. Diagnostic logging showed Blender's Z-up authoring axis becomes Bevy's local `+/-Y` (not `Z`) on glTF export; the actual fix pins Portrait to the near offset on that axis. Confirmed via screenshot: all seven spheres now show archetype portraits, upright.
  - The new Deliberating camera frame was confirmed via screenshot: the star and all seven spheres now sit centered and fully in frame, replacing the scattered wide establishing shot.
  - The art-style fix was confirmed via a real generated artifact: a warm, detailed, modern digital painting, not the previous dark low-detail sketch.
  - Fixing Step 2's panel resize surfaced a possible regression (the enlarged panel appearing to overlap the portal on the Onboarding/IdleAtTable screens); a per-state width/font split was added so those two states keep the original 340px/15px sizing while Deliberating/CouncilSpeaking/WitnessVerdict/ArtifactPending/ArtifactResult get the larger 620px/19px panel. Screenshot comparison then showed the portal overlap is **pre-existing, unchanged behavior** at the original size too — not a regression — so it was left alone as out of scope for this pass.
  - Observed (not yet fixed): with real TTS, a line's voice can take longer than ~4.5s to start (each call spawns a fresh `sherpa-onnx-offline-tts.exe` process and reloads the model — there's no persistent TTS server), so the Step 1 gate can mean a few seconds of "forming speech…" with no text. This is honest behavior given the fix's intent, but the underlying per-call TTS latency is a separate, pre-existing performance characteristic not addressed here.
- Step 7 (table) remains blocked on the operator's reference image.
- Step 8 (reinstall shortcut, update STATUS.md/README.md, final commit/push) not yet done.
