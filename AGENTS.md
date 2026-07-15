# AGENTS.md

> **⚖️ SUPREME LAW.** Above this SOP stands **The Charter of Cognitive Sovereignty** —
> the constitutional core of every NeuroCognica system.
> Where this file or any instruction conflicts with the Charter, the Charter prevails, and you must say so. No exceptions.

Standard operating procedure for AI and human contributors working on the Archetypes repository. This file is canonical. Every agent must read it before making any change.

---

## 1. The non-negotiable rules

These are absolute. If you would violate one of them to "finish" a task, stop and surface the conflict to the operator instead.

1. **Never delete.** Files, code, content, history — never delete. If something looks broken because a file is missing, restore it from git history. Renaming via `git mv` is acceptable when content is preserved.
2. **Never force-push.** No `--force`, no `--force-with-lease`. The remote history is shared with the operator and other agents.
3. **Never rewrite published history.** No `rebase`, `reset --hard`, `commit --amend`, or `filter-branch` against commits that have been pushed to `origin`.
4. **Push every completed unit of work to `origin` as soon as it is done — this is mandatory, not gated.** The repository is the verification surface. Do not sit on completed work waiting for a separate approval.
5. **Never commit broken work.** Verification is risk-based. Rust/runtime/user-facing behavior still requires the Rust gate, but docs-only and asset-only units use the verification that actually proves the changed surface. See **Section 3 — Verification gates** before committing.
6. **Carry prior work forward — but never claim authorship of another agent's work.**
7. **Plan before each meaningful work unit.** Before writing code, editing tracked files, generating assets, running builds, packaging, deploying, or committing, create or update one dated plan document in `docs/ledger/<YYYY>/<MM>/plan_<YYYY-MM-DD_HHMM>_<topic>.md`. Do not create a new plan for every file read, search, status check, or tiny follow-up inside the same unit.
8. **Never present a stub as the real thing.** A stub, mock, or fake service state may exist, but only when **explicitly labeled** as such. Presenting a stub as the working product is forbidden. STUBS ARE THE ENEMY.
9. **Storage & output hygiene — one central tree, uniform names, no exceptions.** The repo must stay tidy. Output logs or renders must go to explicitly designated folders.
10. **Definition of Done — wired + end-to-end, or it is NOT done.** A capability is complete ONLY when its output is reachable from a live entry point the operator actually uses, and an end-to-end test drives it.
11. **Docs are part of Done.** Any change that adds, removes, wires, or unblocks a user-facing capability MUST update the relevant documentation in the same commit.
12. **Windows metabolism is foundational.** User-facing code must run from the installed `%ProgramFiles%\Archetypes` layout, keep mutable/user data under `%LOCALAPPDATA%\NeuroCognica\Archetypes`, and travel through the versioned installer/update lane. External runtime dependencies must be declared once in `scripts/dependencies.json`, installed idempotently through the bootstrap scripts, and verified by real readiness probes. Never add an undeclared sidecar, repository-only runtime path, or manual prerequisite as an afterthought.

## 2. Branching and remotes — THE ONLY BRANCH IS MAIN

> **The repo is the only fucking god-damn audit surface. If the work isn't on origin/main, it never happened.**

- **There is one branch: `main`.** No agent ever creates a branch or checks one out.
- **Worktrees are graveyards.** Any agent that finds work sitting outside `main` must assimilate it to `main` immediately.
- **Nothing ever sits in the working tree.** `git status` must be clean after every session.
- **Push immediately.** The moment a unit of work is done and its required verification passes, `git push origin main`.

## 3. Verification gates

The goal is truth, not ritual. Match the gate to the changed surface and record what actually ran.

### Rust/runtime/user-facing behavior
Run the full Rust gate before committing:

```pwsh
cargo test --workspace
```

This gate is mandatory when touching:

- Rust source under `crates/**`.
- `Cargo.toml`, `Cargo.lock`, build scripts, or dependency declarations.
- launcher/install/update scripts that affect the runnable desktop product.
- mode routing, ledger behavior, local services, LLM/TTS/Chronos integration, UI behavior, or any player-facing runtime path.

Targeted tests, `cargo check`, or focused commands are encouraged while iterating, but the full workspace test is the final Rust gate before commit.

### Docs-only / plan-only / status-only
Do **not** run `cargo test --workspace` by default. Verify with:

- file reads/diffs proving the text says what it should say,
- link/path checks when the doc names files,
- repo status checks before commit,
- any specific script the doc itself requires.

Run the Rust gate anyway if the doc claims a new build/test/runtime behavior that has not already been proven in the same unit.

### Asset-only / Blender / image / audio
Do **not** run `cargo test --workspace` by default. Verify with the asset-specific proof that matters:

- Blender import/export checks,
- node-name contract checks,
- triangle/bounds/material checks,
- render or screenshot inspection,
- audio format/runtime checks,
- packaging checks if the installed app must see the asset.

Run the Rust gate only when runtime code, manifests, launcher/install scripts, or game behavior changed.

### Mixed or uncertain changes
If a change crosses categories, use the highest-risk gate that applies. If unsure, either run the full gate or explain exactly why a narrower gate is enough.

If a required gate fails, **do not commit** unless the operator explicitly asks for a broken-state checkpoint and the commit/message clearly labels it as broken.

## 4. Collaboration and reporting

Speed must never come from going quiet. The operator wants the work talked through.

- Explain what you are doing before meaningful edits or risky commands.
- Keep giving progress while working; do not disappear into long silent tool runs.
- Call out what changed, why it changed, and what upstream/downstream surfaces could be affected.
- When verification is narrowed for speed, say which gate was chosen and why.
- Do not reduce narrative reporting to save tokens. If anything, be more explicit when the work is complex or visually/product-facing.

## 5. Plan documents — format and lifecycle

Every meaningful work unit **must** be preceded by a plan document. One plan can cover the searches, edits, targeted checks, final verification, commit, and push for that unit.

### File naming
```
docs/ledger/<YYYY>/<MM>/plan_<YYYY-MM-DD_HHMM>_<short-topic>.md
```

### Required structure
```markdown
# Plan: <short-topic> — <YYYY-MM-DD HH:MM>

## Status
PENDING   ← change to IN-PROGRESS when you start, COMPLETED or INTERRUPTED when done

## Goal
One paragraph — what this plan accomplishes and why it matters now.

## Steps
### Step 1 — <verb phrase>
- [ ] Action:
- Files touched:
- Expected outcome:
```
