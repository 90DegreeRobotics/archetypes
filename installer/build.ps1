<#
.SYNOPSIS
  Builds, Azure-signs, packages, verifies, archives, and delivers Archetypes.

.DESCRIPTION
  This is the formal release lane. Routine gameplay iteration continues to use
  scripts\install_shortcut.ps1. By default this command requires a working Azure
  Artifact Signing path and fails closed if any emitted PE is not Authenticode
  Valid. -SkipSign is permitted only when version.json uses channel "internal".
#>
[CmdletBinding()]
param(
    [switch]$SkipCargo,
    [switch]$SkipSign
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$versionPath = Join-Path $PSScriptRoot "version.json"
$historyPath = Join-Path $PSScriptRoot "version-history.json"
$issPath = Join-Path $PSScriptRoot "archetypes_setup.iss"
$outputDir = Join-Path $PSScriptRoot "output"

function Write-Step([string]$Text) {
    Write-Host "`n==> $Text" -ForegroundColor Cyan
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Resolve-SignTool {
    $onPath = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }
    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"),
        (Join-Path $env:ProgramFiles "Windows Kits\10\bin")
    )
    foreach ($root in $roots) {
        if (-not (Test-Path -LiteralPath $root)) { continue }
        $dirs = Get-ChildItem -LiteralPath $root -Directory -ErrorAction SilentlyContinue |
            Sort-Object @{ Expression = {
                $parsed = $null
                if ([version]::TryParse($_.Name, [ref]$parsed)) { $parsed } else { [version]"0.0.0.0" }
            }} -Descending
        foreach ($dir in $dirs) {
            $candidate = Join-Path $dir.FullName "x64\signtool.exe"
            if (Test-Path -LiteralPath $candidate) { return $candidate }
        }
    }
    throw "signtool.exe not found in the Windows SDK."
}

function Resolve-Iscc {
    foreach ($candidate in @(
        "ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:LOCALAPPDATA}\Programs\Inno Setup 6\ISCC.exe"
    )) {
        $command = Get-Command $candidate -ErrorAction SilentlyContinue
        if ($command) { return $command.Source }
        if (Test-Path -LiteralPath $candidate) { return $candidate }
    }
    throw "ISCC.exe not found. Install Inno Setup 6."
}

function Resolve-AzureSigning {
    $dlibCandidates = @(
        $env:ARCHETYPES_AZURE_SIGNING_DLIB,
        (Join-Path $repoRoot "tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll"),
        "C:\chronos2\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll"
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    $metaCandidates = @(
        $env:ARCHETYPES_AZURE_SIGNING_METADATA,
        (Join-Path $PSScriptRoot "signing\azure-metadata.json"),
        "C:\chronos2\installer\signing\azure-metadata.json"
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    $dlib = $dlibCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    $meta = $metaCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $dlib) { throw "Azure signing dlib not found." }
    if (-not $meta) { throw "Azure signing metadata not found." }
    $metadata = Get-Content -Raw -LiteralPath $meta | ConvertFrom-Json
    foreach ($field in @("Endpoint", "CodeSigningAccountName", "CertificateProfileName")) {
        if ([string]::IsNullOrWhiteSpace([string]$metadata.$field)) {
            throw "Azure signing metadata is missing $field."
        }
    }
    [pscustomobject]@{ Dlib = [string]$dlib; Metadata = [string]$meta }
}

function Invoke-AzureSign([string]$FilePath, [string]$SignTool, $Azure) {
    Write-Step "Signing $(Split-Path $FilePath -Leaf)"
    & $SignTool sign /v /fd SHA256 /tr "http://timestamp.acs.microsoft.com/" /td SHA256 /dlib $Azure.Dlib /dmdf $Azure.Metadata $FilePath
    if ($LASTEXITCODE -ne 0) { throw "signtool failed for $FilePath (exit $LASTEXITCODE)." }
    $signature = Get-AuthenticodeSignature -LiteralPath $FilePath
    if ($signature.Status -ne "Valid") { throw "Signature is $($signature.Status) for $FilePath." }
}

function Get-SignatureRecord([string]$FilePath) {
    $signature = Get-AuthenticodeSignature -LiteralPath $FilePath
    [ordered]@{
        path = $FilePath
        status = [string]$signature.Status
        subject = if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { "" }
        thumbprint = if ($signature.SignerCertificate) { $signature.SignerCertificate.Thumbprint } else { "" }
        timestamp_subject = if ($signature.TimeStamperCertificate) { $signature.TimeStamperCertificate.Subject } else { "" }
    }
}

$identity = Get-Content -Raw -LiteralPath $versionPath | ConvertFrom-Json
$productVersion = [string]$identity.product_version
$buildSerial = [int]$identity.build_serial
$channel = [string]$identity.channel
$gitCommit = (& git -C $repoRoot rev-parse --short=12 HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw "Could not resolve git commit." }
$buildId = "$productVersion+build.$buildSerial.$gitCommit"
if ($SkipSign -and $channel -ne "internal") {
    throw "-SkipSign is allowed only for channel 'internal'; '$channel' requires signing."
}

if (-not $SkipCargo) {
    Write-Step "Building release engine and launcher"
    Push-Location $repoRoot
    try {
        & cargo build --release --workspace
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)." }
    } finally { Pop-Location }
}

$payloads = @(
    (Join-Path $repoRoot "target\release\engine.exe"),
    (Join-Path $repoRoot "target\release\launcher.exe")
)
foreach ($payload in $payloads) {
    if (-not (Test-Path -LiteralPath $payload)) { throw "Required payload missing: $payload" }
}

Write-Step "Verifying offline council voices in dist\speech"
$distDir = Join-Path $repoRoot "dist"
& (Join-Path $repoRoot "scripts\setup_windows.ps1") -InstallRoot $distDir -NonInteractive
if ($LASTEXITCODE -ne 0) { throw "Speech bootstrap failed for dist\speech." }
$speechRuntime = Join-Path $distDir "speech\sherpa-onnx-v1.13.4-win-x64-shared-MD-Release\bin\sherpa-onnx-offline-tts.exe"
$speechModel = Join-Path $distDir "speech\kokoro-en-v0_19\model.onnx"
if (-not (Test-Path -LiteralPath $speechRuntime) -or -not (Test-Path -LiteralPath $speechModel)) {
    throw "Required speech runtime or model missing in $distDir\speech."
}
$depManifest = Join-Path $repoRoot "scripts\dependencies.json"
$importScript = Join-Path $repoRoot "scripts\import_chronos_object.py"
if (-not (Test-Path -LiteralPath $depManifest)) { throw "Required script missing: $depManifest" }
if (-not (Test-Path -LiteralPath $importScript)) { throw "Required script missing: $importScript" }

$signTool = $null
$azure = $null
if (-not $SkipSign) {
    Write-Step "Checking Azure Artifact Signing readiness"
    & (Join-Path $repoRoot "scripts\check_signing_ready.ps1")
    if ($LASTEXITCODE -ne 0) { throw "Signing readiness probe failed." }
    $signTool = Resolve-SignTool
    $azure = Resolve-AzureSigning
    foreach ($payload in $payloads) { Invoke-AzureSign $payload $signTool $azure }
}

New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$iscc = Resolve-Iscc
$isccArgs = @(
    "/DMyAppVersion=$productVersion",
    "/DMyAppBuildSerial=$buildSerial",
    "/DMyAppBuildId=$buildId"
)
if (-not $SkipSign) {
    $signCommand = '$q{0}$q sign /v /fd SHA256 /tr http://timestamp.acs.microsoft.com/ /td SHA256 /dlib $q{1}$q /dmdf $q{2}$q $f' -f $signTool, $azure.Dlib, $azure.Metadata
    $isccArgs += "-sarchetypes=$signCommand"
    $isccArgs += "/DMyAppSigned=1"
}
$isccArgs += $issPath
Write-Step "Compiling Inno Setup package"
& $iscc @isccArgs
if ($LASTEXITCODE -ne 0) { throw "ISCC failed (exit $LASTEXITCODE)." }

$setupExe = Join-Path $outputDir "Archetypes_Setup.exe"
if (-not (Test-Path -LiteralPath $setupExe)) { throw "Expected installer missing: $setupExe" }
$setupSignature = Get-AuthenticodeSignature -LiteralPath $setupExe
if (-not $SkipSign -and $setupSignature.Status -ne "Valid") {
    throw "Installer signature is $($setupSignature.Status), expected Valid."
}

$hash = Get-Sha256 $setupExe
$size = (Get-Item -LiteralPath $setupExe).Length
$builtUtc = (Get-Date).ToUniversalTime().ToString("o")
$versionedName = "Archetypes_Setup_$productVersion.exe"
$archiveDir = Join-Path $outputDir "archive"
New-Item -ItemType Directory -Force -Path $archiveDir | Out-Null
$copies = @(
    (Join-Path $archiveDir $versionedName),
    (Join-Path $repoRoot $versionedName),
    (Join-Path (Join-Path $env:USERPROFILE "Downloads") $versionedName)
)
foreach ($copy in $copies) {
    $parent = Split-Path $copy -Parent
    if (-not (Test-Path -LiteralPath $parent)) { throw "Required delivery directory missing: $parent" }
    Copy-Item -LiteralPath $setupExe -Destination $copy -Force
    if ((Get-Sha256 $copy) -ne $hash) { throw "Hash mismatch after delivery: $copy" }
}

$payloadRecords = foreach ($payload in $payloads) {
    $sig = Get-SignatureRecord $payload
    [ordered]@{
        filename = Split-Path $payload -Leaf
        sha256 = Get-Sha256 $payload
        size_bytes = (Get-Item -LiteralPath $payload).Length
        signature = $sig
    }
}
$release = [ordered]@{
    schema = "archetypes.release_build.v1"
    product_name = "Archetypes"
    product_version = $productVersion
    build_serial = $buildSerial
    build_id = $buildId
    channel = $channel
    min_os_version = [string]$identity.min_os_version
    git_commit = $gitCommit
    built_utc = $builtUtc
    installer = [ordered]@{
        path = $setupExe
        filename = $versionedName
        sha256 = $hash
        size_bytes = $size
        signed = ($setupSignature.Status -eq "Valid")
        downloads_path = $copies[2]
    }
    signing_status = if ($setupSignature.Status -eq "Valid") { "signed" } else { "unsigned_internal" }
    signatures = @((Get-SignatureRecord $setupExe))
    payload = @($payloadRecords)
}
$release | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $outputDir "release.json") -Encoding UTF8

$historyDoc = Get-Content -Raw -LiteralPath $historyPath | ConvertFrom-Json
$entry = $historyDoc.entries | Where-Object { [int]$_.build_serial -eq $buildSerial } | Select-Object -First 1
if (-not $entry) {
    $newEntry = [ordered]@{
        normalized_version = $productVersion
        build_serial = $buildSerial
        source_commit = $gitCommit
        built_utc = $builtUtc
        installer_sha256 = $hash
        size_bytes = $size
        package_available = $true
        note = "Emitted and verified by installer/build.ps1."
    }
    $historyDoc.entries = @($historyDoc.entries) + [PSCustomObject]$newEntry
} else {
    $entry.source_commit = $gitCommit
    $entry.built_utc = $builtUtc
    $entry.installer_sha256 = $hash
    $entry.size_bytes = $size
    $entry.package_available = $true
    $entry.note = "Emitted and verified by installer/build.ps1."
}
$historyDoc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $historyPath -Encoding UTF8

Write-Host "`nSIGNED RELEASE COMPLETE" -ForegroundColor Green
Write-Host "  Downloads: $($copies[2])"
Write-Host "  SHA256:   $hash"
Write-Host "  Signer:   $($setupSignature.SignerCertificate.Subject)"
Write-Host "  Manifest: $(Join-Path $outputDir 'release.json')"
