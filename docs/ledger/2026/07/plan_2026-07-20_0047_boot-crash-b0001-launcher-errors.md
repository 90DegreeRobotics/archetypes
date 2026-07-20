# Plan: boot-crash-b0001-launcher-errors — 2026-07-20 00:47

## Status
COMPLETED

## Goal
Fix engine exit 101 on Desktop launch (Bevy B0001 query conflicts) and make launcher crashes fail-visible with a persisted log + Notepad.

## Steps
### Step 1 — Fix B0001
- [x] Merge boot hide/reveal into camera visibility chain
- [x] Disjoint `render_chat_ui` `&mut Node` queries (`ChatWaitingBanner` vs `ChatPortrait`)

### Step 2 — Launcher error surface
- [x] Capture engine stdout/stderr to AppData log; open last-failure.txt in Notepad on failure

### Step 3 — Restage desktop
- [x] cargo test; install_shortcut.ps1; release smoke 6s OK
