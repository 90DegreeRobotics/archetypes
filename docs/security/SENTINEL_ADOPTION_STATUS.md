# Sentinel Adoption Status

Product: `Archetypes`
Repository: `C:\archetypes`
Canonical Sentinel plan source: `C:\NRI\Sentinel\SENTINEL_IMPERVIOUS_PROTOCOL_MASTER_PLAN.md`
Local plan copy: `docs/security/SENTINEL_IMPERVIOUS_PROTOCOL_MASTER_PLAN.md`
Protected action inventory: `docs/security/SENTINEL_PROTECTED_ACTIONS.md`
Certification report path: `docs/security/SENTINEL_CERTIFICATION_REPORT.md`
Required release mode: `enforce`
Certification readiness: blocked

## Current State

Status: Implementing, not release-certified.

Implemented footholds:

- Launcher requires Chronos Director readiness and Sentinel authority in enforce mode.
- Launcher owns a durable local Ed25519 Sentinel client key under `%LOCALAPPDATA%\NeuroCognica\Archetypes\sentinel\launcher_client.seed`.
- Launch intent is written through the guarded Chronos Codex append path before `engine.exe`.
- The guarded launch append carries a client-signed Sentinel authority envelope bound to the normalized `codex_append` request digest.
- Legacy `ARCHETYPES_ALLOW_WITHOUT_CHRONOS` handling has been removed from launcher source.

Open stop-ship findings:

- Chronos client-key bootstrap is still a local authority registration path; admin-signed key lifecycle and revocation ceremony are not complete.
- In-engine Sentinel client is not complete.
- Runtime game response mediation is not complete.
- Save/export/share mediation is not complete.
- Player profile and memory mediation are not complete.
- Deny-all paralysis test for launcher plus engine is not complete.
- Release artifact signing and policy signing are not complete.

## Required Certification Command

```powershell
cargo run -p sentinel_cli --bin sentinel -- certify --repo C:\archetypes --product Archetypes --strict --output-dir C:\archetypes\docs\security
```
