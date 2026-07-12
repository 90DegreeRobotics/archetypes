param (
    [switch]$PreflightOnly,
    [switch]$NonInteractive,
    [string]$InstallRoot = (Join-Path $env:ProgramFiles "Archetypes")
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ManifestPath = Join-Path $ScriptDir "dependencies.json"

if (-not (Test-Path $ManifestPath)) {
    Write-Error "Dependency manifest not found at $ManifestPath"
}

$Manifest = Get-Content $ManifestPath | ConvertFrom-Json

Write-Host "Archetypes Windows Dependency Bootstrap"
Write-Host "---------------------------------------"

foreach ($pkg in $Manifest.winget_packages) {
    if ($pkg.required) {
        Write-Host "Checking for $($pkg.name)..."
        $check = winget list --id $($pkg.id) --exact 2>&1
        
        if ($check -match $pkg.id) {
            Write-Host "  -> Already installed."
        } else {
            if ($PreflightOnly) {
                Write-Host "  -> Missing. (Preflight only, skipping install)"
            } else {
                Write-Host "  -> Installing via Winget..."
                $args = @("install", "--id", $pkg.id, "--exact", "--accept-source-agreements", "--accept-package-agreements")
                if ($NonInteractive) { $args += "--silent" }
                & winget $args
            }
        }
    }
}

foreach ($artifact in $Manifest.download_artifacts) {
    if (-not $artifact.required) { continue }
    $destination = Join-Path $InstallRoot $artifact.destination
    $expectedName = if ($artifact.name -match "sherpa-onnx") {
        "sherpa-onnx-v1.13.4-win-x64-shared-MD-Release\bin\sherpa-onnx-offline-tts.exe"
    } else {
        "kokoro-en-v0_19\model.onnx"
    }
    $readinessPath = Join-Path $destination $expectedName
    Write-Host "Checking for $($artifact.name)..."
    if (Test-Path -LiteralPath $readinessPath) {
        Write-Host "  -> Ready at $readinessPath"
        continue
    }
    if ($PreflightOnly) {
        Write-Host "  -> Missing. (Preflight only, skipping install)"
        continue
    }

    New-Item -ItemType Directory -Force -Path $destination | Out-Null
    $archive = Join-Path $env:TEMP ("archetypes-$($artifact.sha256).tar.bz2")
    Write-Host "  -> Downloading pinned artifact..."
    & curl.exe -L --fail --output $archive $artifact.url
    if ($LASTEXITCODE -ne 0) { throw "Download failed for $($artifact.name)" }
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash
    if ($actual -ne $artifact.sha256) {
        throw "Hash mismatch for $($artifact.name): expected $($artifact.sha256), got $actual"
    }
    & tar.exe -xf $archive -C $destination
    if ($LASTEXITCODE -ne 0) { throw "Extraction failed for $($artifact.name)" }
    if (-not (Test-Path -LiteralPath $readinessPath)) {
        throw "Artifact extracted but readiness path is absent: $readinessPath"
    }
    Write-Host "  -> Installed and verified."
}

if (-not $PreflightOnly) {
    Write-Host "`nSetup complete. You may need to restart your terminal for PATH changes to take effect."
}
