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
5. **Never commit broken code.** `cargo test` must pass before every commit. If you cannot run the suite, say so and stop.
6. **Carry prior work forward — but never claim authorship of another agent's work.**
7. **Plan before every action.** Before writing any code, editing any file, or running any build command, you must first create a dated plan document in `docs/ledger/<YYYY>/<MM>/plan_<YYYY-MM-DD_HHMM>_<topic>.md`.
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
- **Push immediately.** The moment a unit of work is done and tests pass, `git push origin main`.

## 3. The test gate

Before every commit:

```pwsh
cargo test
```

If any test fails, **do not commit**.

## 4. Plan documents — format and lifecycle

Every session of meaningful work **must** be preceded by a plan document.

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
