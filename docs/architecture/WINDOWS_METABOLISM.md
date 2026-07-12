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
  - `%ProgramFiles%\Archetypes\speech\sherpa-onnx-v1.13.4-win-x64-shared-MD-Release\` — pinned native TTS runtime.
  - `%ProgramFiles%\Archetypes\speech\kokoro-en-v0_19\` — pinned Kokoro model, voices, tokens, and phonemizer data.
  
- **Mutable (AppData) Paths:**
  - `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\` — Save states, configurations, and SQLite databases (if utilized).
  - `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\` — Engine and launcher logs.
  - `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\audio_cache\` — generated council WAV cache.

The installer does not pre-create AppData. The launcher dynamically resolves and creates the `%LOCALAPPDATA%` tree on the first run for the specific Windows user.

## Speech Runtime

The canonical dependency manifest pins the sherpa-onnx Windows runtime and Kokoro English model by URL and SHA-256. `setup_windows.ps1` downloads, verifies, extracts, and readiness-checks both under the immutable install tree. The engine never searches the repository or system `PATH` for speech dependencies. Development proof may use both `ARCHETYPES_TTS_EXE` and `ARCHETYPES_TTS_MODEL_DIR` explicitly; setting only one is an error.

The seven signature utterances are packaged application assets for immediate response. Generated verdict speech is synthesized on a worker thread, written to mutable LocalAppData, validated as a non-empty RIFF/WAVE, and then handed to Bevy audio. A missing or failed runtime is visible and non-fatal: text and keyboard operation remain available.
