# Sentinel Adoption Status

Product: `Archetypes`
Repository: `C:\archetypes`
Canonical Sentinel plan source: `C:\NRI\Sentinel\SENTINEL_IMPERVIOUS_PROTOCOL_MASTER_PLAN.md`
Local plan copy: `docs/security/SENTINEL_IMPERVIOUS_PROTOCOL_MASTER_PLAN.md`
Protected action inventory: `docs/security/SENTINEL_PROTECTED_ACTIONS.md`
Certification report path: `docs/security/SENTINEL_CERTIFICATION_REPORT.md`
Required release mode: `enforce`
Certification readiness: candidate

## Current State

Status: Candidate. `sentinel certify --strict` **PASS** on 2026-08-16 against a clean tree. Launch and in-engine protected work are Sentinel-mediated. Not release-signed. Do not read PASS as **certified**.

Implemented footholds:

- Launcher requires Chronos Director readiness and Sentinel authority in enforce mode.
- Launcher owns a durable local Ed25519 Sentinel client key under `%LOCALAPPDATA%\NeuroCognica\Archetypes\sentinel\launcher_client.seed`.
- Launch intent is written through the guarded Chronos Codex append path before `engine.exe`.
- The engine reuses that keystore. `chat.respond`, `game.respond` (via chat), `model.generate` (via Chronos artifact), `artifact.register`, `memory.write`, `file.write` (via memory/profile persist), and `profile.generate` are mediated before they run.
- Local deny-all paralysis tests cover all 40 canonical protected actions. Unknown actions deny even under the mediated-runtime policy.
- Legacy `ARCHETYPES_ALLOW_WITHOUT_CHRONOS` handling remains absent from launcher and engine source.

Open stop-ship findings for a **certified** (not candidate) release:

- Admin-signed key lifecycle and revocation ceremony are not complete.
- Release artifact signing and policy signing are not complete.
- Unused protected actions remain fail-closed deny rather than Chronos-policy certified paths.

## Required Certification Command

```powershell
cargo run -p sentinel_cli --bin sentinel -- certify --repo C:\archetypes --product Archetypes --strict --output-dir C:\archetypes\docs\security
```
