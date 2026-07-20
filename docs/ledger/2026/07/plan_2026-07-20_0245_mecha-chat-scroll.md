# Plan: mecha-chat-scroll — 2026-07-20 02:45

## Status
COMPLETED

## Goal
Make the Standard Mecha chat transcript scrollable with mouse wheel (and drag), without fighting auto-scroll-to-bottom on new turns.

## Steps
- [x] Wire Bevy 0.18 hover+MouseWheel → ChatUiScroll observer → ScrollPosition
- [x] Click-drag scroll on transcript root
- [x] Auto-jump scroll only when history grows
- [x] cargo test --workspace; install_shortcut.ps1
