# Plan: Product rebuild — 2026-08-16 10:45

## Status
COMPLETED

## Goal
Stage a flight-proven rebuild: put the AURA council on the Desktop menu, keep Consciousness (Mecha) as its own mode, finish Windows lifecycle (install/uninstall/help/sidecar/fail-visible), and ship Inner Chambers plus Living Engine as real playable modes — never as unlocked stubs.

Engineering rule: each stage must compile and pass `cargo test --workspace` before the next stage is opened. Do not relabel a locked mode playable until its loop is wired and tested.

## Frozen product identity
- **STANDARD MODE** = AURA council ritual (onboarding → table offering → 3-voice deliberation → speaking choreography → verdict → Chronos artifact). Ledger id `standard`.
- **CONSCIOUSNESS** = 1:1 archetype chat (former default Standard/Mecha path). Ledger id `consciousness`.
- **ORACLE RIDDLE** = reverse-prompt. Unchanged id `oracle_riddle`.
- **INNER CHAMBERS** = Architect navigable interior; extracted truth seeds one Oracle round. Unlocked only after the interior is real.
- **LIVING ENGINE** = metabolic instrument (resonance / aura / Viren / implants). Unlocked only after the sim + playable loop exist.

## Steps
### Step 1 — Stage 1 identity and council flight
- [x] Action: Add `GameMode::Consciousness`. Menu Standard enters the council ritual and loads `uiscene1` + table. Esc returns to lore-chamber menu. Mecha ledger events retag to `consciousness`. Always register council visual plugins; visibility owned solely by `gate_boot_and_table_visibility`.
- Files touched: `game_mode.rs`, `boot.rs`, `chamber/mod.rs`, `camera.rs`, `ritual.rs`, `standard_mecha/mod.rs`, tests.
- Expected outcome: Five-mode registry; two (then four) playable; council reachable without `ARCHETYPES_CAPTURE`.

### Step 2 — Stage 2 Windows metabolism
- [x] Action: Every launcher miss uses `fail_visible`. Auto-repair TTS from pinned `dependencies.json`. Portable Comfy output path. In-game Help + `assets/help/index.html`. Product install/uninstall scripts. Stage help + dependency manifest into `dist`.
- Files touched: `crates/launcher/src/main.rs`, `chronos.rs`, `boot.rs` (Help), `scripts/*`, `assets/help/**`.
- Expected outcome: Founder never needs a console. Missing voices repair themselves. Uninstall exists.

### Step 3 — Stage 3 Inner Chambers
- [x] Action: Replace cube prototype with a Luminous Blueprint interior. Per-node truths. Persist last truth; Oracle consumes it once. Restore lore menu on exit (do not spawn `uiscene1`). Unlock registry.
- Files touched: `modes/inner_chambers/**`, `oracle_riddle/scoring.rs`, `game_mode.rs`.
- Expected outcome: Playable Architect chamber; extraction seeds Oracle.

### Step 4 — Stage 4 Living Engine
- [x] Action: Pure, tested resonance/aura/Viren/implant sim. Playable 3-sphere prototype (Architect, Sentinel, Oracle) + Viren strain + 2 implant slots. Unlock registry.
- Files touched: `modes/living_engine/**`, `modes/mod.rs`, `game_mode.rs`.
- Expected outcome: Headless tests prove the instrument; Desktop menu launches it.

### Step 5 — Verify and restage
- [x] Action: `cargo test --workspace`. Update README, STATUS, lane docs. `scripts\install_shortcut.ps1`.
- Files touched: docs, dist via installer script.
- Expected outcome: Desktop icon runs the new product. Failures remain visible.
