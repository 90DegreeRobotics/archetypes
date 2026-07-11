# Archetypes — Agent Orientation

**Last updated: 2026-07-11**

Read this after `README.md` and before touching any file. `README.md` is now the primary repo truth surface; this document is the agent-orientation companion that explains where the work happens and what rules still matter in practice.

## System Architecture

Archetypes is a focused Rust backend serving a single primary function. We have consciously chosen a lighter architecture compared to the heavy multi-crate Chronos system.

- **Backend**: Single Rust crate API (`cargo build` in root).
- **Database**: SQLite (or simple flat files/JSONL) instead of the heavy RocksDB + Merkle Tree "Forever Law" system, given the focused scope of the application.

## Key Rules (Do Not Violate)

- **THE UI IS THE ONLY GREEN CHECK.** A change is NOT done, NOT tested, NOT a checked box until the capability is wired into the app and demonstrated working. "Works in the terminal" is reported as exactly that — terminal-only, UI-pending — never as done.
- **Plan before every action — no exceptions.** Create a dated plan document `plan_<YYYY-MM-DD_HHMM>_<topic>.md`.
- **There is one branch: `main`. Always. No exceptions.** No agent ever creates a branch, checks one out, or does work anywhere but `main`. Worktrees are graveyards.
- **Push completed work to `origin` immediately.** The repo must receive new work as soon as it is done.
- **Never present a stub as the real surface.** STUBS ARE THE ENEMY.
- **One central output tree, uniform names.** No scattered output files.

See `AGENTS.md` and `PROTOCOL.md` for full doctrinal rules.
