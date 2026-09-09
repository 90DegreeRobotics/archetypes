# Azure Trusted Signing Pipeline — Windows Code Signing Architecture

**Canonical Reference for the Archetypes Repository**  
*Derived from the production-proven signing architecture in `C:\chronos2`*

---

## 1. Executive Summary & Publisher Identity

The operator is a **trusted Microsoft Windows verified author**:
- **Verified Signer:** `CN=Michael Holt, O=Michael Holt, L=Normal, S=il, C=US`
- **Certificate Authority:** `CN=Microsoft ID Verified CS AOC CA 03, O=Microsoft Corporation, C=US`
- **Service Tier:** Azure Artifact Signing (formerly Azure Trusted Signing), Public Trust, Individual Path
- **Hardware Security:** Microsoft FIPS 140-3 Level 3 HSM (no local private keys, no `.pfx` on disk, no USB dongles to lose)
- **Endpoint:** `https://eus.codesigning.azure.net`
- **Signing Account:** `chronosophiasigning`
- **Certificate Profile:** `chronosophiapublic` (Public Trust, program type `None`, home street address omitted for privacy)

Any release executable or installer distributed to end users must pass through this signing pipeline. Handing over unsigned binaries to a buyer triggers immediate OS-level security blocks.

---

## 2. Why Code Signing Is Mandatory (The Commercial Threat Model)

On modern Windows 11 systems, an unsigned executable is treated as hostile:

1. **Windows Defender Heuristic Deletions:**
   Defender's machine learning classifiers (such as `Trojan:Win32/Bearfoos.B!ml`) proactively delete unsigned game installers and executables upon download. The `!ml` tag denotes a heuristic detection triggered by an unknown, unsigned binary extracting and executing processes. A verified publisher identity suppresses this false positive.

2. **SmartScreen Filter:**
   Unsigned binaries have zero reputation, presenting an aggressive "Windows protected your PC" modal. Reputation attaches directly to the code signing certificate identity.

3. **Smart App Control (SAC):**
   Enabled by default on fresh Windows 11 installations. Under enforced SAC, unsigned binaries cannot be run—there is no click-through option or bypass. Only binaries signed by a trusted certificate authority are permitted to execute.

---

## 3. The 6-Step Azure Trusted Signing Pipeline

The signing pipeline consists of six sequential gates. Every gate must be verified before declaring a release build ready.

```
+-------------------------------------------------------------------------------+
|                      THE 6-STEP SIGNING PIPELINE                             |
+-------------------------------------------------------------------------------+
| 1. Resolve SignTool.exe     -> Windows 10/11 SDK x64 build (sorted by version) |
| 2. Locate Azure Dlib        -> Azure.CodeSigning.Dlib.dll (x64) from NuGet    |
| 3. Load Azure Metadata      -> azure-metadata.json (Endpoint/Account/Profile) |
| 4. Verify Authentication    -> Active 'az login' session or Service Principal |
| 5. Configure Timestamp      -> http://timestamp.acs.microsoft.com/ (TSA)      |
| 6. ISCC Compile-Time Sign   -> ISCC -sarchetypes="<cmd>" (signs Setup & Temp)  |
+-------------------------------------------------------------------------------+
```

### Step 1 — Resolve `signtool.exe`
`signtool.exe` does not live on `PATH` by default; it is installed inside versioned directories within the Windows 10/11 SDK (`C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64\signtool.exe`).
- **Resolver Rule:** Never rely on a hardcoded SDK path. Search the SDK directories and sort candidate version numbers numerically as `[version]` (not as strings, where `10.0.9999.0` would incorrectly rank higher than `10.0.26100.0`).
- **Architecture:** Always select the **x64** binary.

### Step 2 — Azure Code Signing Dlib (`Azure.CodeSigning.Dlib.dll`)
`signtool.exe` cannot speak to Azure Artifact Signing natively. It relies on a custom signing provider dlib from the `Microsoft.Trusted.Signing.Client` NuGet package:
- **Location:** `tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll`
- **Crucial Gotcha:** The loader DLL lives under `bin\x64\`, **not** at the root of the unpacked NuGet package. Passing the package root causes `signtool` to fail or fallback silently to an unsigned build.

### Step 3 — Azure Signing Metadata Configuration
The dlib reads an account descriptor JSON file (`azure-metadata.json`):
```json
{
  "Endpoint": "https://eus.codesigning.azure.net",
  "CodeSigningAccountName": "chronosophiasigning",
  "CertificateProfileName": "chronosophiapublic"
}
```
- **Validation:** Build scripts must ensure this file exists and does not contain unexpanded template markers (e.g. `<account-name>` or `REPLACE_ME`).

### Step 4 — Azure Identity & Authentication
The dlib authenticates against Azure to sign using the operator's verified identity:
- **Interactive Operator Session:** An active Azure CLI session via `az login` (verified by `az account show`).
- **Automated Service Principal (Optional CI):** `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, and `AZURE_CLIENT_SECRET`.

### Step 5 — Timestamp Authority (Load-Bearing)
Azure Artifact Signing end-entity certificates rotate daily and have a **72-hour lifetime**.
- **The Threat:** If a binary is signed without a valid timestamp, the signature becomes invalid the moment the 72-hour certificate expires.
- **The Fix:** Every signature must include a countersignature from Microsoft's Public RSA Time Stamping Authority:
  `http://timestamp.acs.microsoft.com/`
- **Command Flag:** `/tr http://timestamp.acs.microsoft.com/ /td SHA256`

### Step 6 — Inno Setup (ISCC) Compile-Time Signing (Preventing Error 4551)
Signing the outer installer `Archetypes_Setup.exe` *after* compilation is fatally flawed on modern Windows:
- **The Mechanism:** Inno Setup's setup loader (`SetupLdr`) extracts a temporary self-copy (`Setup.e32`) into `%TEMP%`.
- **The Incident:** Under Smart App Control (SAC) enforced mode, running a post-hoc signed installer throws:
  > *Unable to execute file in the temporary directory. Setup aborted.*  
  > **Error 4551: An Application Control policy has blocked this file.**
- **The Solution:** The signing command must be passed into Inno Setup at compile time:
  ```powershell
  ISCC.exe -sarchetypes="$q$signTool$q sign /v /fd SHA256 /tr $timestamp /td SHA256 /dlib $q$dlib$q /dmdf $q$meta$q $f" /DMyAppSigned=1 Archetypes_setup.iss
  ```
- **Why this works:** When `SignTool` is configured in the `.iss` script (`SignTool=archetypes`), Inno Setup automatically signs:
  1. `uninst.e32.tmp` (the uninstaller `unins000.exe`),
  2. The temporary setup self-copy extracted to `%TEMP%`, and
  3. The final compiled `Archetypes_Setup.exe`.

---

## 4. Exact `signtool` Command Syntax

### Individual Binary / Payload Signing:
```powershell
& $signToolPath sign `
    /v `
    /fd SHA256 `
    /tr "http://timestamp.acs.microsoft.com/" `
    /td SHA256 `
    /dlib "C:\archetypes\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll" `
    /dmdf "C:\archetypes\installer\signing\azure-metadata.json" `
    "target\release\engine.exe"
```

### Inno Setup SignTool Definition:
Inside the Inno Setup script (`.iss`):
```inno
#ifdef MyAppSigned
SignTool=archetypes
#endif
```

Inside the PowerShell build script:
```powershell
$signCmd = '$q{0}$q sign /v /fd SHA256 /tr {1} /td SHA256 /dlib $q{2}$q /dmdf $q{3}$q $f' -f `
    $signToolPath, $timestamp, $dlibPath, $metadataPath

& $isccPath "-sarchetypes=$signCmd" "/DMyAppSigned=1" $issPath
```
*(Note: `$q` is Inno Setup's internal quotation sequence to prevent space-escaping issues on Windows paths).*

---

## 5. Post-Signing Integrity & Payload Re-Hashing

A critical supply-chain defect discovered in `chronos2` must not be repeated:
- If a build script calculates SHA-256 hashes of payload binaries (`engine.exe`, `launcher.exe`) *before* signing them, and then signs them afterwards, the digital signature alters the PE binary.
- This results in a manifest hash that differs from the shipped file, tripping integrity alarms.
- **Rule:** Whenever binaries are signed, they must be **re-hashed on disk immediately after signing**, and those post-signing hashes must be recorded into the release manifest (`release.json`).

---

## 6. Truthful Verification Contract (`release.json`)

Never trust build parameters or exit codes alone. The build pipeline must independently inspect the finished output file using Windows Authenticode cmdlets:
```powershell
$sig = Get-AuthenticodeSignature -LiteralPath "installer\output\Archetypes_Setup.exe"
if ($sig.Status -ne "Valid" -or $sig.SignerCertificate.Subject -notmatch "Michael Holt") {
    throw "Authenticode signature verification failed!"
}
```

The output `release.json` records real, observed state:
```json
{
  "schema": "archetypes.release_build.v1",
  "product_version": "1.0.0",
  "build_serial": 1,
  "signing_status": "signed",
  "signatures": [
    {
      "kind": "authenticode",
      "subject": "CN=Michael Holt, O=Michael Holt, L=Normal, S=il, C=US",
      "thumbprint": "11745D6DCA21E459EAF384E11ED7D213BCB9DD84",
      "signed_file": "Archetypes_Setup.exe",
      "timestamp_authority": "http://timestamp.acs.microsoft.com/"
    }
  ]
}
```

---

## 7. Operational Readiness Checklist (`check_signing_ready.ps1`)

Before running a production release build, execute the readiness probe:
```powershell
$env:ARCHETYPES_SIGNING_MODE = "azure"
$env:ARCHETYPES_AZURE_SIGNING_DLIB = "C:\archetypes\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll"
$env:ARCHETYPES_AZURE_SIGNING_METADATA = "C:\archetypes\installer\signing\azure-metadata.json"
pwsh scripts\check_signing_ready.ps1
```

All 6 gates must report `[ OK ]`:
1. `signtool.exe` resolved from Windows SDK.
2. `Azure.CodeSigning.Dlib.dll` present under `bin\x64`.
3. `azure-metadata.json` present on disk.
4. `azure-metadata.json` valid JSON with populated account and profile names.
5. Azure authentication active (`az account show` exit 0 or `AZURE_*` env vars).
6. Timestamp authority responsive.
