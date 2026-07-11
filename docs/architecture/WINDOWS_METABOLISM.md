# Archetypes Windows Metabolism

## The Dual-Binary Contract

Archetypes strictly follows the NeuroCognica dual-binary architecture to ensure system stability, dependency verification, and clean exits.

- `archetypes_launcher.exe`
  - The entry point for the user.
  - Verifies prerequisites (like Ollama).
  - Enforces a single-instance mutex.
  - Spawns and supervises `archetypes_engine.exe`.
- `archetypes_engine.exe`
  - The Bevy 3D simulation.
  - Never run directly by the user; expects to be supervised by the launcher.

## Installed Layout & Mutable State

The application enforces a strict separation between immutable binaries and mutable user data.

- **Immutable (Read-Only) Paths:**
  - `%ProgramFiles%\Archetypes\archetypes_launcher.exe`
  - `%ProgramFiles%\Archetypes\archetypes_engine.exe`
  - `%ProgramFiles%\Archetypes\scripts\`
  
- **Mutable (AppData) Paths:**
  - `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\` — Save states, configurations, and SQLite databases (if utilized).
  - `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\` — Engine and launcher logs.

The installer does not pre-create AppData. The launcher dynamically resolves and creates the `%LOCALAPPDATA%` tree on the first run for the specific Windows user.
