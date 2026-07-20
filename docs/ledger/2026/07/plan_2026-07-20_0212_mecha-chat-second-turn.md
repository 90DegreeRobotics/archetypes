# Plan: mecha-chat-second-turn — 2026-07-20 02:12

## Status
COMPLETED

## Goal
Fix Standard Mecha multi-turn chat so a second (and later) question still shows the Ollama reply and new Comfy image inline.

## Root cause
Backend already wrote turn 2+ replies/images to JSONL. UI broken by:
1. Idle `tick_chat_wait_indicator` mutating ChatBridge every frame → constant transcript rebuild → new images stuck near alpha 0
2. No auto-scroll → newer turns below the fold
3. `install_shortcut.ps1` wiping `assets/standard_mecha/renders`

## Steps
- [x] Trace history (sentinel turn 2/3 complete in JSONL)
- [x] Fingerprint rebuilds + idle tick fix + ScrollPosition jump
- [x] Preserve renders on Desktop staging
- [x] cargo test + install_shortcut
