# Plan: Playable Council Build — Real Routes, Camera, Panels, Comfy, Launch — 2026-07-12

## Status
COMPLETED (2026-07-12). Claude implemented and visually verified the unit; Codex took
over after rate limiting, reran the final workspace gates, repaired the shortcut script's
conflict with the repository's never-delete law, rebuilt the release staging and shortcuts,
then committed and pushed the carried-forward work. Full design plan:
`C:\Users\m\.claude\plans\crystalline-rolling-puffin.md`.

## Origin
Director request after auditing the recent Codex overhaul (temple, vessel panels, Kokoro
TTS): make the game genuinely launchable and playable with **real** routes to the
archetypes (no stubs), fix the camera, spin the panels in focus, clearer onboarding, and
Comfy-only images. Decisions: launch = playable shortcut + title screen; dialogue = direct
Ollama; images = Comfy-only; council = full multi-voice.

## Audit findings (what the overhaul had left dormant/stubbed)
1. Vessel panels baked in the GLB but the engine had no animation code → they never spun.
2. Camera disabled the authored camera and did an approach-zoom → never framed a speaker;
   the hardcoded `(12,10,16)` sat at the temple wall (radius 21) looking at dark.
3. Deliberation was a hardcoded `architect_verdict()` template; the real Kokoro voice spoke
   canned text; Ollama (up, with `qwen2.5:7b-instruct`) was unused for dialogue.
4. Only the Architect was ever reached.
5. Launcher was a `println!` stub; `setup_windows.ps1` skipped the model pull; no shortcut,
   no title/loading screen.
6. Comfy unwired (game only sent `quick:true` = Blender).

## What was built and verified
- **Real council (`chamber/council.rs`)** — background Ollama `/api/chat`, three in-character
  members (framer/counter/deepener, all seven surfacing over time) + Witness verdict; fails
  visibly if Ollama is down. Verified: logged transcript showed three distinct in-character
  voices + a synthesized verdict; on-screen in `05_council_speaking`.
- **Speaking choreography** — new `CouncilSpeaking` state walks the transcript; each speaker
  drives camera + voice + panel + environment tint. States refactored
  (`FocusArchetype`/`ArchitectInterior` → `CouncilSpeaking`).
- **Camera (`chamber/camera.rs`)** — establishing = the authored `Witness_Camera` transform
  (read from the scene); per-speaker = azimuth-based framing (fixed radius/height, inside the
  temple, above the floor). Verified: establishing shows the whole council; speaking frames
  the speaker with the star beyond.
- **Panels (`chamber/panels.rs`)** — spin the focused archetype's `*_PanelSpinner` clockwise
  only while it speaks. Verified: panel turned between `05` and `05b`.
- **Comfy-only (`chamber/ritual.rs`)** — `render-still` with `quick:false` + `comfyui_url`
  (:8000) + install dir + checkpoint; fail closed. Verified reaching `ArtifactPending` and
  submitting; **blocked** by the Director's `codex.db` error (Chronos-side, see below).
- **Title/loading screen (`chamber/boot.rs`)** — `Booting` default state, full-screen title
  until all seven vessels bind. Verified: `00_title`.
- **Launcher (`crates/launcher`)** — real supervisor (single-instance TCP guard, readiness
  checks, launches the engine). **`scripts/install_shortcut.ps1`** builds release, stages
  `dist/`, makes an `.ico`, and creates Desktop + Start Menu shortcuts. Verified: shortcuts
  created. `setup_windows.ps1` now pulls the Ollama model.
- **Onboarding copy** rewritten to explain the name and why (you are the Witness the council
  reports to). Verified: `01_onboarding`.
- Tests + build were green after each increment (last full green build before the trivial
  `comfyui_checkpoint` addition; that addition still to be recompiled + tests re-run).

## Known blocker (Chronos-side, not the engine)
The live Comfy render fails with `Error: opening codex at "data/codex.db"` from the Chronos
Director. The engine's request is correct and reaches `ArtifactPending`. The Director likely
needs to be restarted from `C:\chronos`. Documented in `STATUS.md`.

## Final verification
- `cargo test --workspace`: 11 passed, 0 failed after the `comfyui_checkpoint` addition.
- `cargo build --workspace`: passed.
- `cargo build --release --workspace`: passed; `dist/` and Desktop/Start Menu shortcuts rebuilt.
- Both PowerShell scripts parse cleanly.
- `install_shortcut.ps1` now stages assets additively and does not delete the prior asset tree.
