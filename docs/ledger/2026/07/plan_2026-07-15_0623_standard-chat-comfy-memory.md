# Plan: Standard Chat Comfy Memory — 2026-07-15 06:23

## Status
COMPLETED

## Goal
Make Standard Mode archetype chat generate a real Chronos/Comfy image for every submitted user statement, display that image alongside the archetype's response, and persist the full turn forever using the Chronos Forever Law contract. The implementation must preserve each archetype's chat voice and add a distinct signature art style per archetype without touching Oracle Riddle behavior or unlocking Inner Chambers / Living Engine.

## Steps
### Step 1 — Trace existing truth
- [x] Action: Read the current Standard Mecha chat code, local history path, Chronos render service code, and the Chronos Forever Law source in `C:\chronos`.
- Files touched: plan only during this step.
- Expected outcome: The implementation targets real existing routes and persistence, not a stub or fake image surface.

### Step 1A — Add a real main-menu exit
- [x] Action: Add a Quit/Exit command to the Bevy main menu that gracefully closes the whole app through Bevy's app-exit event path.
- Files touched: `crates/engine/src/chamber/boot.rs`
- Expected outcome: The desktop-facing main menu has an explicit exit option and no longer traps the user in the launcher shell.

### Step 2 — Model the image turn contract
- [x] Action: Add a typed render-request/result/persistence shape for Standard chat turns, including keyword extraction, archetype id, archetype chat style, archetype art style, prompt, status, output path, and failure text.
- Files touched: `crates/engine/src/modes/standard_mecha/**`, docs/status as needed.
- Expected outcome: Every turn can represent either a completed image or a visible fail-closed image error, and history remains real.

### Step 3 — Wire real Chronos/Comfy generation
- [x] Action: Call the existing local Chronos pipeline for every user statement, using extracted keywords plus archetype-specific art style, while keeping the LLM response real and visibly failing if Chronos/Comfy cannot render.
- Files touched: `crates/engine/src/services/chronos.rs`, `crates/engine/src/modes/standard_mecha/**` as needed.
- Expected outcome: No Electron/Node/DOM/localStorage dependency; image generation uses the same local service lane the launcher already probes.

### Step 4 — Display and persist forever
- [x] Action: Render the generated image in the Standard chat UI beside the response, persist turn records under `%LOCALAPPDATA%\NeuroCognica\Archetypes`, and append immutable Forever Law records so nothing presented as remembered can be forgotten.
- Files touched: Standard Mode UI/history code and status docs.
- Expected outcome: The player sees the archetype response and signature image every turn; restart history is backed by real files, not ephemeral UI state.

### Step 5 — Verify runtime and publish
- [x] Action: Add/adjust focused tests, run `cargo test --workspace`, `cargo build -p engine`, refresh `dist` only after tests pass, run the packaged launcher through Standard chat capture if available, inspect screenshots/output artifacts, update this plan and `STATUS.md`, commit, and push `main`.
- Files touched: docs/status/proof artifacts as needed.
- Expected outcome: `origin/main` contains a verified Standard chat image-generation lane, with honest notes for any live service blocker.

## Verification Notes
- `cargo test -p engine standard_mecha` passed: 7 Standard tests.
- `cargo test -p engine chamber::boot` passed: 3 boot/menu tests.
- `cargo test --workspace` passed: 47 engine tests and 1 launcher test.
- `cargo build -p engine` passed.
- `scripts\install_shortcut.ps1` completed the release build but timed out before staging; the fresh release binaries were then manually copied into `dist`, `dist\assets` was refreshed, and Desktop / Start Menu shortcuts were verified to target `C:\archetypes\dist\launcher.exe` with working directory `C:\archetypes\dist`.
- Chronos Director `:7777`, ComfyUI `:8000`, and Ollama `:11434` were live before runtime capture.
- Packaged runtime witness: `C:\archetypes\dist\launcher.exe` ran with `ARCHETYPES_MECHA_CAPTURE=1` and captured `artifacts/visual-proof/standard-chat-comfy-2026-07-15_1818/05_architect_chat_result_or_failure.png`, showing Architect chat with a completed Comfy image in the response panel.
- Forever Law evidence: `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\standard_mecha\chat_history\architect.jsonl` and `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\ledger.jsonl` contain the matching response image metadata, artifact id `c790b503-3f6f-412d-a788-f3b103b25a26`, and proof receipt `e752ae1a-578a-4727-8066-54edae172d6f`.
