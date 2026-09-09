# Versioning, Serialization, and Release Management

**Canonical Reference for the Archetypes Repository**  
*Aligned with the versioning and release doctrine from `C:\chronos2`*

---

## 1. Principles of Release Management

1. **Every Release Is Serialized and Named:**
   Two distinct binaries must never share the same version string. Every formal release has a semantic version (`product_version`) and a strictly increasing integer (`build_serial`).

2. **The Operator Law of Artifact Delivery:**
   > **Every formal build lands a VERSION-NAMED installer in the operator's `Downloads` folder.**  
   > (`%USERPROFILE%\Downloads\Archetypes_Setup_<version>.exe`)

   The operator tests as a buyer: they uninstall previous builds and execute the new installer themselves. Build scripts must copy the versioned installer to `Downloads` and verify the SHA-256 hash between the output directory and the destination. Never force the operator to hunt through build trees or run CLI commands to install an update.

3. **Reconciling the Two Development Cadences:**
   To maintain rapid progress without sacrificing release integrity, the repository operates on two distinct lanes:

   | Lane | Purpose | Trigger / Command | Artifact / Destination |
   |---|---|---|---|
   | **Fast Inner Loop** | Rapid iteration (gameplay, shaders, 3D assets, camera tuning, UI) | `pwsh scripts\install_shortcut.ps1` | `dist\engine.exe` staged to `%LOCALAPPDATA%\Programs\Archetypes` and reflected on the **Pinned Windows Taskbar Icon** instantly (0–30s cadence). |
   | **Formal Release Lane** | Production distribution, buyer verification, clean-room installs | `pwsh installer\build.ps1` | Fully signed `Archetypes_Setup_<version>.exe` delivered to `%USERPROFILE%\Downloads`, registered in `version-history.json`, and recorded in `release.json`. |

---

## 2. Version Identity Contract (`installer/version.json`)

The single source of truth for current product identity is `installer/version.json`:

```json
{
  "schema": "archetypes.release_identity.v1",
  "product_name": "Archetypes",
  "product_version": "1.0.0",
  "build_serial": 1,
  "channel": "stable",
  "min_os_version": "10.0.18362",
  "distribution": "direct",
  "signing": {
    "status": "signed",
    "authenticode": true,
    "channels_requiring_signature": [
      "stable",
      "beta"
    ],
    "certificate": "Azure Artifact Signing, Public Trust, individual path (CN=Michael Holt).",
    "enforced_by": "ISCC compile-time signing flag -sarchetypes and Get-AuthenticodeSignature verification"
  }
}
```

### Key Fields:
- `product_version`: Semantic version string (`MAJOR.MINOR.PATCH`).
- `build_serial`: Monotonically increasing integer. Never decremented. Every published build increments this number.
- `channel`: `stable` (default public release), `beta` (preview release), or `internal` (developer testing).
- `min_os_version`: Windows build floor (`10.0.18362` = Windows 10 1903 / Windows 11).

---

## 3. Version History Audit Trail (`installer/version-history.json`)

Every completed build is permanently recorded in `installer/version-history.json`:

```json
{
  "schema": "archetypes.version_history.v1",
  "entries": [
    {
      "normalized_version": "1.0.0",
      "build_serial": 1,
      "source_commit": "084f536c4b2a",
      "built_utc": "2026-09-09T12:45:00Z",
      "installer_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "size_bytes": 142589000,
      "package_available": true,
      "note": "Initial castle rotunda council chamber build with animated table"
    }
  ]
}
```

This ensures full auditability across git commits, binary hashes, and released versions.

---

## 4. Build Artifact Output Topology

When `installer/build.ps1` executes, it distributes the output to four standardized locations:

1. **Compiler Output:**  
   `installer\output\Archetypes_Setup.exe` (working build output)
2. **Versioned Archive:**  
   `installer\output\archive\Archetypes_Setup_<version>.exe` (immutable archival copy)
3. **Repository Root Shadow:**  
   `Archetypes_Setup_<version>.exe` (convenience access)
4. **Operator Downloads (MANDATORY):**  
   `%USERPROFILE%\Downloads\Archetypes_Setup_<version>.exe` (verified by SHA-256 match)

---

## 5. Release Metadata (`installer/output/release.json`)

The release pipeline emits `release.json` capturing the verified cryptographic and file attributes:

```json
{
  "schema": "archetypes.release_build.v1",
  "product_name": "Archetypes",
  "product_version": "1.0.0",
  "build_serial": 1,
  "build_id": "1.0.0+build.1.084f536c4b2a",
  "channel": "stable",
  "min_os_version": "10.0.18362",
  "git_commit": "084f536c4b2a",
  "built_utc": "2026-09-09T12:45:00Z",
  "installer": {
    "path": "installer\\output\\Archetypes_Setup.exe",
    "filename": "Archetypes_Setup_1.0.0.exe",
    "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "size_bytes": 142589000,
    "signed": true
  },
  "signing_status": "signed",
  "signatures": [
    {
      "kind": "authenticode",
      "subject": "CN=Michael Holt, O=Michael Holt, L=Normal, S=il, C=US",
      "thumbprint": "11745D6DCA21E459EAF384E11ED7D213BCB9DD84",
      "signed_file": "Archetypes_Setup.exe"
    }
  ]
}
```

---

## 6. How Session Agents Must Operate

1. **During Routine Gameplay/Art Development:**
   - Modify Rust code, shaders, or 3D assets.
   - Run targeted tests and verify behavior.
   - Run `pwsh -File scripts\install_shortcut.ps1` to update `dist\` and sync `%LOCALAPPDATA%\Programs\Archetypes`.
   - The operator launches and verifies immediately from their **pinned Taskbar icon**.

2. **When Preparing a Versioned Release:**
   - Increment `product_version` or `build_serial` in `installer/version.json`.
   - Run `pwsh scripts\check_signing_ready.ps1` to confirm all 6 Azure gates are green.
   - Execute `pwsh installer\build.ps1` (with signing enabled).
   - Verify that `Archetypes_Setup_<version>.exe` exists in the operator's `Downloads` directory and matches `release.json`.
   - Report the SHA-256 hash, Authenticode signer, and file location directly to the operator.
