# Plan: versioned-signed-title-release — 2026-09-09 17:00

## Status
COMPLETED

## Goal

Make the running game identify its formal release in the native window title at
all times, then cut and deliver a new Azure-signed, versioned installer through
the formal release lane. The Taskbar launch path must be brought to that signed
release and verified rather than inferred from an earlier developer staging
copy.

## Steps

### Step 1 — Embed release identity in the executable and title
- [x] Action: Read `installer/version.json` during the engine build, expose the
  product version and build serial to Rust, use them in the native window title,
  and add a test for the title contract.
- Files touched: engine build metadata and startup code.
- Expected outcome: Every screenshot contains `Archetypes <version> (build <n>)`.

### Step 2 — Serialize a committed-source release
- [x] Action: Advance the release identity and append its pending audit entry
  without altering historical entries.
- Files touched: installer version identity/history.
- Expected outcome: A unique version and serial exist before packaging.
- Correction: 1.0.3/build 4 was signed and installed but built while the title
  source was uncommitted, so its manifest identifies the previous commit.
  It is retained as a real signed artifact, not presented as the clean handoff.
  Release 1.0.4/build 5 supersedes it from a committed source state.

### Step 3 — Build, sign, package, deliver, and install
- [x] Action: Run the signing readiness gate, full Rust tests, formal installer
  build with signing enabled, validate signature/hash/release manifest and the
  Downloads artifact, then install the signed package.
- Files touched: generated release history/manifest only.
- Expected outcome: `Archetypes_Setup_1.0.4.exe` is signed, delivered to
  Downloads, and installed as the buyer artifact.
- Result: The 206,696,416-byte installer was emitted to Downloads with SHA-256
  `6dff04299b89df7c9466933e9dea7ad67739088f5931b8b925c0bd7594a6fcfd`.
  Installer, engine, and launcher signatures are `Valid` for `CN=Michael Holt`.
  The installer was then run silently and installed its signed payload to
  `C:\Program Files\Archetypes`. That 1.0.3 artifact is retained but
  superseded. 1.0.4/build 5 was then emitted from committed source
  `f414259452e4`; its Downloads installer SHA-256 is
  `2d9e25b36a1310f6e7a547ce119117679988d27650b30f0084a1bb3a77b7fba5`.
  The output and Downloads hashes match, all signatures are valid, and the
  installer completed into `C:\Program Files\Archetypes`.

### Step 4 — Rebind and witness the Taskbar path
- [x] Action: Verify the installed signed executable identity and the pinned
  Taskbar shortcut target; launch only through the normal buyer path when the
  installer has completed.
- Files touched: deployment audit/plan only if needed.
- Expected outcome: The next screenshot identifies the formal release in its
  native title bar.
- Result: The installed engine PE reports file/product version `1.0.3.4` and a
  valid `CN=Michael Holt` signature. The superseding installed engine PE
  reports file/product version `1.0.4.5` with the same valid signature. The
  existing pinned Taskbar link was
  rebound and reread as `C:\Program Files\Archetypes\launcher.exe` with
  `C:\Program Files\Archetypes` as its working directory. A foreground click
  from that pin remains the final visual witness.

### Step 5 — Commit and publish
- [x] Action: Review generated metadata, commit named source/release records,
  push `main`, and record what was actually witnessed versus what still needs
  player confirmation.
- Files touched: named files from this plan.
- Expected outcome: Origin is the auditable handoff surface.
