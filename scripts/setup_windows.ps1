param (
    [switch]$PreflightOnly,
    [switch]$NonInteractive
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

if (-not $PreflightOnly) {
    Write-Host "`nSetup complete. You may need to restart your terminal for PATH changes to take effect."
}
