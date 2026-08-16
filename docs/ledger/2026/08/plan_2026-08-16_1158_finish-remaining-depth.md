# Plan: Finish remaining product depth — 2026-08-16 11:58

## Status
COMPLETED

## Goal
Close the six remaining flight items in one session: Sentinel candidate certification with real in-engine mediation, Chronos/Comfy/Ollama as launcher-supervised sidecars, persistent council TTS, hexagram-alignment camera, seven distinct archetype interiors, and persistent world memory with chamber consequence. No stubs. Each stage must compile; the unit ends with `cargo test --workspace` and Desktop restage.

## Frozen rules
- STANDARD MODE remains the AURA council. CONSCIOUSNESS remains 1:1 chat.
- Sentinel unavailable means deny. No bypass flags.
- Chronos Foundry is not vendored into Archetypes; the launcher starts the installed sibling if it is down.
- Certification readiness becomes **candidate** only after mediation is wired and deny-all tests pass. Not release-signed; that stays honest.

## Steps
### Step 1 — Persistent TTS
- [x] Action: Keep Kokoro loaded via sherpa C API (`sherpa-onnx-c-api.dll`) on a single worker; hash-cache WAVs; CLI fallback if the DLL refuses to create an engine. Warm during boot veil.
- Files touched: `crates/engine/src/chamber/speech.rs`, new `tts_runtime.rs`, `Cargo.toml`.
- Expected outcome: Second and later council lines do not respawn `sherpa-onnx-offline-tts.exe` when the warm engine is alive.

### Step 2 — Hexagram camera
- [x] Action: During CouncilSpeaking, fly the Witness camera to the stellated-octahedron 3-fold axis closest to the speaking sphere so the star reads as a hexagram behind the speaker.
- Files touched: `chamber/camera.rs`, `chamber/star.rs`.
- Expected outcome: Pure tests prove axis selection; speaking pose is no longer a simple compass orbit.

### Step 3 — Seven interiors
- [x] Action: Inner Chambers becomes a hub with seven themed rooms (Architect through Jester), each with distinct law/geometry and extractable truths. CouncilSpeaking also crosses into that archetype's interior set around the star.
- Files touched: `modes/inner_chambers/**`, `chamber/interior.rs`, HELP.
- Expected outcome: Extraction records which mind was read; Oracle still consumes one triple.

### Step 4 — World memory
- [x] Action: Accepted Chronos artifacts persist as a hash-chained lineage under AppData and spawn visible chamber tokens on the table. Next session recalls them.
- Files touched: `services/memory.rs`, `ritual.rs`.
- Expected outcome: Artifact return mutates the chamber; memory is recallable.

### Step 5 — Sidecar supervision
- [x] Action: If Ollama, Director, or Comfy are down, the launcher starts the installed binaries (portable discovery, no `C:\Users\m\`) and waits. Fail-visible if they cannot be started.
- Files touched: `crates/launcher/src/main.rs`, `dependencies.json`, HELP.
- Expected outcome: Founder clicks Desktop Archetypes; the launcher brings the stack up.

### Step 6 — Sentinel candidate
- [x] Action: In-engine Sentinel client mediates chat.respond, game.respond, model.generate, artifact.register, memory.write, file.write, profile.generate. Deny-all and unknown-action tests. Update adoption docs to candidate. Re-run `sentinel certify`.
- Files touched: `services/sentinel.rs`, llm/ledger/chronos/memory, `docs/security/**`.
- Expected outcome: `adoption_readiness` PASS as candidate. Inventory still lists all 40 actions.

### Step 7 — Verify and restage
- [x] Action: `cargo test --workspace`. Docs. `scripts\install_shortcut.ps1`.
- Expected outcome: Desktop icon runs 0.3.0 with the finished depth.

## Verification record
- `cargo test --workspace --offline`: 78 engine + 15 launcher, pass.
- Sentinel `adoption_readiness`: PASS (candidate). Strict certify FAIL only on `strict_git_clean` (uncommitted tree; operator has not asked to commit).
- Desktop restage 2026-08-16 12:16: `dist\engine.exe` / `dist\launcher.exe` FileVersion 0.3.0.0. Desktop `Archetypes.lnk` → `C:\archetypes\dist\launcher.exe` cwd `C:\archetypes\dist`. Help and `dist\scripts\dependencies.json` staged.
