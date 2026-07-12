# Plan: ZeroVOX speech lane — 2026-07-12 05:45

## Status
COMPLETED

## Goal
Read the attached ZeroVOX proposal, verify its technical and licensing claims against current upstream sources and the Archetypes Windows Metabolism, and carry out the requested repository work only where the evidence and operator scope authorize it.

## Steps
### Step 1 — Read and classify the request
- [x] Action: Read the complete attached text and identify the requested outcome, assumptions, and external dependencies.
- Files touched: This plan only.
- Expected outcome: Exact task scope without inferring authority beyond the attachment.

### Step 2 — Verify ZeroVOX and repository compatibility
- [x] Action: Check upstream source, license, releases, Windows support, model/runtime footprint, API surface, and fit with the current engine and installer contract.
- Files touched: None unless implementation is authorized and viable.
- Expected outcome: Evidence-backed go, conditional-go, or stop verdict.

### Step 3 — Execute the authorized lane
- [x] Action: Implement, document, test, and publish the requested unit if viable; otherwise record the concrete blocker and smallest safe next move.
- Files touched: Determined by the verified request.
- Expected outcome: Honest repository state with no undeclared sidecar or unverified speech claim.

## Verdict

- ZeroVOX source and current published models are Apache-2.0, resolving the copyleft concern raised by maintained Piper.
- The current PyPI package classifies its operating system as POSIX/Linux, not Windows, and its public model cards explicitly label the models early alpha and not for production use or distribution.
- The upstream project advertises real-time embedded use, but that positioning is not equivalent to an Archetypes-ready Windows runtime, installer, readiness probe, stable API, or measured Forge latency.
- ZeroVOX must not become a required Archetypes sidecar in its present state. It belongs on a monitored research watchlist, with an optional isolated benchmark only after a native Windows installation path exists.
- The six-month product lane remains: implement Whisper.cpp/CPAL push-to-talk first, preserve a provider-neutral TTS boundary, and select a distributable TTS engine only after Windows, quality, latency, licensing, and installer proof all pass.
- Voice cloning additionally requires explicit consent, provenance, revocation, and anti-impersonation policy before reference samples or embeddings are stored. The Witness voice must never be cloned implicitly.
