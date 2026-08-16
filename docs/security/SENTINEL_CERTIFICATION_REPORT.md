# Sentinel Certification Report

This report is deterministic by design. It omits timestamps so rerunning certification does not dirty a clean release tree.

Product: `Archetypes`
Repository: `C:\archetypes`
Strict mode: `true`
Result: `FAIL`

## Checks

| Check | Status | Detail |
| --- | --- | --- |
| `repo_exists` | `PASS` | repository path exists |
| `git_repository` | `PASS` | path is inside a Git worktree |
| `strict_git_clean` | `FAIL` | working tree was dirty before report write |
| `master_plan_doc` | `PASS` | required Sentinel security document is present and contains release-critical markers |
| `adoption_status_doc` | `PASS` | required Sentinel security document is present and contains release-critical markers |
| `protected_actions_doc` | `PASS` | required Sentinel security document is present and contains release-critical markers |
| `adoption_readiness` | `PASS` | Sentinel adoption readiness is marked as candidate or certified |
| `protected_action_inventory` | `PASS` | inventory explicitly classifies every canonical Sentinel protected action |
| `tracked_runtime_artifacts` | `PASS` | no tracked env files, service keys, logs, build output, or runtime captures found |
| `tracked_secret_material` | `PASS` | no live-looking credentials or private-key material found in tracked files |
| `source_stub_markers` | `PASS` | no executable source stub markers found |
| `sentinel_bypass_flags` | `PASS` | no Sentinel bypass or shadow-mode flags found in executable source |
| `guard_fail_closed_self_test` | `PASS` | deny-all policy denies every protected action and unknown actions deny even under explicit policy |

## Evidence

### `repo_exists`

- `C:\archetypes`

### `git_repository`

- `C:\archetypes`

### `strict_git_clean`

- ` M Cargo.lock`
- ` M README.md`
- ` M STATUS.md`
- ` M crates/engine/Cargo.toml`
- ` M crates/engine/build.rs`
- ` M crates/engine/src/chamber/boot.rs`
- ` M crates/engine/src/chamber/camera.rs`
- ` M crates/engine/src/chamber/interior.rs`
- ` M crates/engine/src/chamber/mod.rs`
- ` M crates/engine/src/chamber/ritual.rs`
- ` M crates/engine/src/chamber/speech.rs`
- ` M crates/engine/src/chamber/star.rs`
- ` M crates/engine/src/modes/game_mode.rs`
- ` M crates/engine/src/modes/inner_chambers/camera.rs`
- ` M crates/engine/src/modes/inner_chambers/extraction.rs`
- ` M crates/engine/src/modes/inner_chambers/mod.rs`
- ` M crates/engine/src/modes/inner_chambers/world.rs`
- ` M crates/engine/src/modes/mod.rs`
- ` M crates/engine/src/modes/oracle_riddle/scoring.rs`
- ` M crates/engine/src/modes/standard_mecha/mod.rs`
- ` M crates/engine/src/services/chronos.rs`
- ` M crates/engine/src/services/ledger.rs`
- ` M crates/engine/src/services/llm.rs`
- ` M crates/engine/src/services/mod.rs`
- ` M crates/engine/src/theme/mod.rs`
- ` M crates/launcher/Cargo.toml`
- ` M crates/launcher/build.rs`
- ` M crates/launcher/src/main.rs`
- ` M docs/ledger/2026/07/CODEX_LANE_B_INNER_CHAMBERS.md`
- ` M docs/ledger/2026/07/CODEX_LANE_C_LIVING_ENGINE.md`
- ` M docs/ledger/CURRENT_HANDOFF.md`
- ` M docs/security/SENTINEL_ADOPTION_STATUS.md`
- ` M docs/security/SENTINEL_PROTECTED_ACTIONS.md`
- ` M scripts/dependencies.json`
- ` M scripts/install_shortcut.ps1`
- `?? LICENSE`
- `?? assets/help/`
- `?? crates/engine/src/chamber/tts_runtime.rs`
- `?? crates/engine/src/modes/inner_chambers/catalog.rs`
- `?? crates/engine/src/modes/inner_chambers/seed.rs`
- `?? crates/engine/src/modes/living_engine/`
- `?? crates/engine/src/services/memory.rs`
- `?? crates/engine/src/services/sentinel.rs`
- `?? docs/ledger/2026/08/plan_2026-08-16_0710_windows-release-audit.md`
- `?? docs/ledger/2026/08/plan_2026-08-16_1045_product-rebuild.md`
- `?? docs/ledger/2026/08/plan_2026-08-16_1158_finish-remaining-depth.md`
- `?? docs/ledger/audits/`
- `?? scripts/install_product.ps1`
- `?? scripts/uninstall_product.ps1`

### `master_plan_doc`

- `C:\archetypes\docs\security\SENTINEL_IMPERVIOUS_PROTOCOL_MASTER_PLAN.md`

### `adoption_status_doc`

- `C:\archetypes\docs\security\SENTINEL_ADOPTION_STATUS.md`

### `protected_actions_doc`

- `C:\archetypes\docs\security\SENTINEL_PROTECTED_ACTIONS.md`

### `adoption_readiness`

- `C:\archetypes\docs\security\SENTINEL_ADOPTION_STATUS.md`

### `protected_action_inventory`

- `40 actions covered in C:\archetypes\docs\security\SENTINEL_PROTECTED_ACTIONS.md`

### `tracked_runtime_artifacts`

- No additional evidence.

### `tracked_secret_material`

- No additional evidence.

### `source_stub_markers`

- No additional evidence.

### `sentinel_bypass_flags`

- No additional evidence.

### `guard_fail_closed_self_test`

- `40 protected actions tested`

