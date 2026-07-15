# Plan: Mecha Standard Mode Implementation - 2026-07-15 04:46

## Status
COMPLETED

## Goal
Implement the next Archetypes Standard Mode direction on `main`: preserve the black title boot, fade into a game-feeling council/table menu, and route `GameMode::Standard` into a hard-coded native Bevy Mecha-style selector/chat feature. The implementation must mirror the `C:\mecha\aura-mechanician\frontend\src` source canon without Electron, Node, DOM, or browser storage, keep the old chamber/ritual code as reference, keep Oracle Riddle untouched, and prove desktop-facing behavior with runtime screenshots before claiming completion.

## Steps
### Step 1 - Baseline and source-canon inventory
- [x] Action: Read `AGENTS.md`, refresh/verify `origin/main`, confirm `ddd963b` or newer, read `STATUS.md`, the Standard Mode handoff, the Mecha frontend audit, and the rebuild study before code.
- Files touched: this plan only.
- Expected outcome: Current scope, handoff floor, safety fences, and Mecha source requirements are grounded in checked files.

### Step 2 - Mirror Mecha assets and define registry
- [x] Action: Copy the Mecha image asset set into `assets/mecha/` without dropping source files, then add a Rust registry for all seven archetypes with selector metadata, theme tokens, portrait/icon paths, and known selector-vs-CSS color differences.
- Files touched: `assets/mecha/**`, `crates/engine/src/modes/standard_mecha/**`, module exports.
- Expected outcome: The native game has a single Mecha archetype contract and automated checks that every referenced asset exists.

### Step 3 - Build native Standard Mecha states
- [x] Action: Add `standard_mecha` systems for the in-world main menu, selector, switching state, and archetype chat UI while preserving old chamber/ritual code as reference.
- Files touched: `crates/engine/src/modes/standard_mecha/**`, `crates/engine/src/modes/mod.rs`, Standard routing code.
- Expected outcome: `GameMode::Standard` enters the new Mecha Standard path and the menu/selector/chat surfaces render in Bevy, not as web UI.

### Step 4 - Wire real chat and durable history
- [x] Action: Route chat sends through the existing local LLM service and persist/load per-archetype chat history through a real app-data file or ledger-backed path. Fail visibly if the local LLM path is unavailable.
- Files touched: `crates/engine/src/modes/standard_mecha/**`, possibly `crates/engine/src/services/**` only if a narrow helper is required.
- Expected outcome: Chat is not canned, and any history shown to the player is actually durable under the local Archetypes app-data tree.

### Step 5 - Preserve adjacent modes and boot behavior
- [x] Action: Verify black title boot remains `ARCHETYPES` then `A GAME BY MICHAEL HOLT`; Oracle Riddle remains reachable and behaviorally untouched; Inner Chambers and Living Engine remain locked.
- Files touched: only if routing integration requires narrow fixes.
- Expected outcome: The new Standard path does not regress mode availability, boot cadence, or lock boundaries.

Result: The staged launcher capture includes `00_title_arch.png`, `01_title_subtitle.png`, `02_portal_main_menu.png`, and Oracle Riddle proof frames under `artifacts/visual-proof/mecha-standard-2026-07-15_0446-oracle/`. The main menu still exposes Oracle Riddle and shows Inner Chambers / Living Engine as sealed. Oracle source files were not edited, and the full Oracle test set passed.

### Step 6 - Verification, desktop refresh, and witness captures
- [x] Action: Run registry/asset checks, `cargo test --workspace`, `cargo build -p engine`, refresh `dist/desktop` with `scripts/install_shortcut.ps1` only once the implementation is viable, and capture actual runnable app screenshots for title, subtitle, main menu, selector/chat, chat success or visible LLM failure, Oracle Riddle, and locked modes.
- Files touched: `dist/**` only when ready, screenshot artifacts under the designated visual-proof tree.
- Expected outcome: Runtime gates and visual witness evidence determine whether the unit can be committed and pushed to `origin/main`.

Result: Mecha asset SHA-256 mirror check passed for 19 PNGs. `cargo test --workspace` passed with 43 engine tests and 1 launcher test. `cargo build -p engine` passed. `scripts\install_shortcut.ps1` rebuilt release binaries, refreshed `dist`, verified offline voices/model, and recreated Desktop/Start Menu shortcuts. `dist\launcher.exe` ran with `ARCHETYPES_MECHA_CAPTURE=1` and captured the final witness set in `artifacts/visual-proof/mecha-standard-2026-07-15_0446-oracle/`: boot title, subtitle, portal main menu, Mecha selector, Architect chat, real Architect chat response/history persistence, Oracle chat after switch, Oracle Riddle generating, and Oracle Riddle generated-image guessing state.
