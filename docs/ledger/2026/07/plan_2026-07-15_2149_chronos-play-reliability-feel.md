# Plan: chronos-play-reliability-feel — 2026-07-15 21:49

## Status
COMPLETE

## Goal
Make “Chronos must be running” reliable for Archetypes play: fail-closed HTTP readiness for Director+Comfy at launch, verified Ollama VRAM unload before Comfy, live service-truth banners on lore menu + Mecha chat, Oracle restored as a playable menu entry, phased wait copy, soft image reveal, Esc back to menu.

## Steps

### Step 1 — Plan
- [x] Write this plan

### Step 2 — Launcher HTTP fail-closed readiness
- [x] Add ureq; probe Ollama tags, Director `/api/v1/status` readiness=ready, Comfy `/system_stats`
- [x] Chronos+Comfy required unless `ARCHETYPES_ALLOW_WITHOUT_CHRONOS=1` (labeled debug)

### Step 3 — Verified Ollama unload in chronos.rs
- [x] Fail before concept-thumbnail if unload call fails
- [x] Plain-language detail for VRAM/Foundry failures
- [x] Unit-testable helpers where pure

### Step 4 — Engine readiness helper + menu/chat feel
- [x] `services/readiness.rs` probe snapshot
- [x] Lore menu: Oracle available; live footer; activate Oracle
- [x] Mecha: live status, phased wait channel, image reveal, strip jargon, Esc→menu from selector

### Step 5 — Docs + tests + commit + push
- [x] STATUS.md; cargo test --workspace; push main

## Test gate
```
cargo test --workspace
```

## Rollback
Revert the unit commit. Never force-push.
