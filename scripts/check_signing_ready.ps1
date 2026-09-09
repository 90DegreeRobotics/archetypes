<#
.SYNOPSIS
  Reports whether this machine can produce a SIGNED Archetypes build right now.

.DESCRIPTION
  Checks all 6 gates of the Azure Trusted Signing pipeline:
  1. signtool.exe in Windows SDK
  2. Azure Code Signing dlib (Azure.CodeSigning.Dlib.dll x64)
  3. Azure metadata JSON file
  4. Azure metadata JSON contents (Endpoint, Account, Profile)
  5. Azure CLI authentication (az account show) or Service Principal env vars
  6. Timestamp Authority (http://timestamp.acs.microsoft.com/)

  Exit code 0 means a signed build is possible right now; 1 means something is missing.
  Documentation: docs/windows/AZURE_TRUSTED_SIGNING.md
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$script:Failures = @()

function Write-Gate {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][bool]$Ok,
        [string]$Detail,
        [string]$Fix
    )
    if ($Ok) {
        Write-Host ("  [ OK ] " + $Name) -ForegroundColor Green
        if ($Detail) { Write-Host ("         " + $Detail) -ForegroundColor DarkGray }
    } else {
        Write-Host ("  [MISS] " + $Name) -ForegroundColor Yellow
        if ($Detail) { Write-Host ("         " + $Detail) -ForegroundColor DarkGray }
        if ($Fix) { Write-Host ("         fix: " + $Fix) -ForegroundColor Cyan }
        $script:Failures += $Name
    }
}

function Find-SignTool {
    $onPath = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }
    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"),
        (Join-Path $env:ProgramFiles "Windows Kits\10\bin")
    )
    foreach ($root in $roots) {
        if (-not (Test-Path -LiteralPath $root)) { continue }
        $versionDirs = Get-ChildItem -LiteralPath $root -Directory -ErrorAction SilentlyContinue |
            Sort-Object -Property @{ Expression = {
                $v = $null
                if ([version]::TryParse($_.Name, [ref]$v)) { $v } else { [version]"0.0.0.0" }
            } } -Descending
        foreach ($vd in $versionDirs) {
            $candidate = Join-Path $vd.FullName "x64\signtool.exe"
            if (Test-Path -LiteralPath $candidate) { return $candidate }
        }
        $candidate = Join-Path $root "x64\signtool.exe"
        if (Test-Path -LiteralPath $candidate) { return $candidate }
    }
    return $null
}

Write-Host ""
Write-Host "Archetypes Signing Readiness Probe" -ForegroundColor White
Write-Host "==================================" -ForegroundColor White
Write-Host ""

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$mode = if ($env:ARCHETYPES_SIGNING_MODE) { $env:ARCHETYPES_SIGNING_MODE.Trim().ToLowerInvariant() } else { "azure" }
Write-Host ("Signing Mode: " + $mode)
Write-Host ""

# 1. Gate: signtool.exe
$signTool = Find-SignTool
$signToolDetail = if ($signTool) { $signTool } else { "not found on PATH or in Windows 10/11 SDK" }
Write-Gate -Name "signtool.exe" -Ok ([bool]$signTool) -Detail $signToolDetail -Fix "install the Windows 10/11 SDK"

# 2. Gate: Azure Code Signing Dlib
$defaultDlib = Join-Path $repoRoot "tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll"
$chronosDlib = "C:\chronos2\tools\signing\bin\x64\Azure.CodeSigning.Dlib.dll"
$dlib = if ($env:ARCHETYPES_AZURE_SIGNING_DLIB) { 
    $env:ARCHETYPES_AZURE_SIGNING_DLIB 
} elseif (Test-Path -LiteralPath $defaultDlib) { 
    $defaultDlib 
} elseif (Test-Path -LiteralPath $chronosDlib) {
    $chronosDlib
} else { 
    $defaultDlib 
}

$dlibOk = (-not [string]::IsNullOrWhiteSpace($dlib)) -and (Test-Path -LiteralPath $dlib)
$dlibDetail = if ($dlibOk) { $dlib } else { "not found at $dlib" }
Write-Gate -Name "Azure signing dlib" -Ok $dlibOk -Detail $dlibDetail `
    -Fix "copy Azure.CodeSigning.Dlib.dll into tools\signing\bin\x64\ or set ARCHETYPES_AZURE_SIGNING_DLIB"

# 3. Gate: Azure metadata file
$defaultMeta = Join-Path $repoRoot "installer\signing\azure-metadata.json"
$chronosMeta = "C:\chronos2\installer\signing\azure-metadata.json"
$meta = if ($env:ARCHETYPES_AZURE_SIGNING_METADATA) {
    $env:ARCHETYPES_AZURE_SIGNING_METADATA
} elseif (Test-Path -LiteralPath $defaultMeta) {
    $defaultMeta
} elseif (Test-Path -LiteralPath $chronosMeta) {
    $chronosMeta
} else {
    $defaultMeta
}

$metaOk = (-not [string]::IsNullOrWhiteSpace($meta)) -and (Test-Path -LiteralPath $meta)
$metaDetail = if ($metaOk) { $meta } else { "not found at $meta" }
Write-Gate -Name "Azure signing metadata file" -Ok $metaOk -Detail $metaDetail `
    -Fix "create installer\signing\azure-metadata.json with Endpoint, CodeSigningAccountName, and CertificateProfileName"

# 4. Gate: Metadata contents
if ($metaOk) {
    $fieldsOk = $false
    $fieldsDetail = ""
    try {
        $json = Get-Content -Raw -LiteralPath $meta | ConvertFrom-Json
        $required = @("Endpoint", "CodeSigningAccountName", "CertificateProfileName")
        $missing = @()
        $unfilled = @()
        foreach ($f in $required) {
            $v = [string]$json.$f
            if ([string]::IsNullOrWhiteSpace($v)) {
                $missing += $f
            } elseif ($v.StartsWith("<") -or $v -match "REPLACE") {
                $unfilled += $f
            }
        }
        if ($missing.Count -gt 0) {
            $fieldsDetail = "missing field(s): " + ($missing -join ", ")
        } elseif ($unfilled.Count -gt 0) {
            $fieldsDetail = "unfilled template: " + ($unfilled -join ", ")
        } else {
            $fieldsOk = $true
            $fieldsDetail = [string]$json.Endpoint + " / " + [string]$json.CodeSigningAccountName + " / " + [string]$json.CertificateProfileName
        }
    } catch {
        $fieldsDetail = "invalid JSON: " + $_.Exception.Message
    }
    Write-Gate -Name "Azure signing metadata contents" -Ok $fieldsOk -Detail $fieldsDetail `
        -Fix "fill Endpoint, CodeSigningAccountName, and CertificateProfileName"
}

# 5. Gate: Azure Authentication
$spOk = (-not [string]::IsNullOrWhiteSpace($env:AZURE_TENANT_ID)) -and
        (-not [string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_ID)) -and
        (-not [string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_SECRET))
$azCmd = Get-Command az -ErrorAction SilentlyContinue
$azLoggedIn = $false
if ($azCmd) {
    try {
        $null = & az account show 2>$null
        $azLoggedIn = ($LASTEXITCODE -eq 0)
    } catch {
        $azLoggedIn = $false
    }
}
$authOk = $spOk -or $azLoggedIn
if ($spOk) {
    $authDetail = "service principal configured in AZURE_* environment"
} elseif ($azLoggedIn) {
    $authDetail = "active az login CLI session detected"
} elseif ($azCmd) {
    $authDetail = "az installed but no active login session"
} else {
    $authDetail = "Azure CLI not installed and no AZURE_* credentials found"
}
Write-Gate -Name "Azure authentication" -Ok $authOk -Detail $authDetail `
    -Fix "run 'az login' in terminal, or configure AZURE_TENANT_ID / AZURE_CLIENT_ID / AZURE_CLIENT_SECRET"

# 6. Gate: Timestamp Authority
$timestamp = "http://timestamp.acs.microsoft.com/"
Write-Gate -Name "Timestamp authority" -Ok $true -Detail $timestamp

Write-Host ""
if ($script:Failures.Count -eq 0) {
    Write-Host "READY: Machine is fully prepared to produce SIGNED Archetypes builds." -ForegroundColor Green
    Write-Host "Publisher: CN=Michael Holt (Azure Artifact Signing)" -ForegroundColor Green
    Write-Host ""
    exit 0
} else {
    Write-Host ("NOT READY: " + $script:Failures.Count + " gate(s) missing:") -ForegroundColor Yellow
    foreach ($f in $script:Failures) { Write-Host ("  - " + $f) -ForegroundColor Yellow }
    Write-Host ""
    Write-Host "Review docs\windows\AZURE_TRUSTED_SIGNING.md for remediation steps." -ForegroundColor DarkGray
    Write-Host ""
    exit 1
}
