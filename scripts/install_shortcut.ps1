<#
    Build the Archetypes release, stage a self-contained dist folder, and create a
    Desktop + Start Menu shortcut (with an app icon) that launches the game through the
    supervising launcher. This is the "a proper desktop icon to a working game" path;
    run scripts\setup_windows.ps1 first to install the runtime dependencies.
#>
param(
    [string]$DistRoot = (Join-Path $PSScriptRoot "..\dist"),
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

if (-not $SkipBuild) {
    Write-Host "Building release binaries (engine + launcher)..."
    Push-Location $RepoRoot
    try {
        & cargo build --release --workspace
        if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
    } finally {
        Pop-Location
    }
}

$DistRoot = [System.IO.Path]::GetFullPath($DistRoot)
New-Item -ItemType Directory -Force -Path $DistRoot | Out-Null

Copy-Item (Join-Path $RepoRoot "target\release\engine.exe") $DistRoot -Force
Copy-Item (Join-Path $RepoRoot "target\release\launcher.exe") $DistRoot -Force

# The release engine loads ./assets relative to its working directory.
$AssetsDst = Join-Path $DistRoot "assets"
New-Item -ItemType Directory -Force -Path $AssetsDst | Out-Null
Get-ChildItem (Join-Path $RepoRoot "assets") | Copy-Item -Destination $AssetsDst -Recurse -Force
Write-Host "Staged runtime to $DistRoot"

# Build an .ico from an archetype icon so the shortcut has real art.
$IcoPath = Join-Path $DistRoot "archetypes.ico"
$IconPng = Join-Path $RepoRoot "assets\icons\architect-icon.png"
try {
    Add-Type -AssemblyName System.Drawing
    $src = [System.Drawing.Image]::FromFile($IconPng)
    $bmp = New-Object System.Drawing.Bitmap $src, 256, 256
    $hicon = $bmp.GetHicon()
    $icon = [System.Drawing.Icon]::FromHandle($hicon)
    $stream = [System.IO.File]::Create($IcoPath)
    $icon.Save($stream)
    $stream.Close(); $icon.Dispose(); $bmp.Dispose(); $src.Dispose()
    Write-Host "Icon written to $IcoPath"
} catch {
    Write-Warning "Could not build .ico ($_). The shortcut will use the launcher's default icon."
    $IcoPath = Join-Path $DistRoot "launcher.exe"
}

$LauncherExe = Join-Path $DistRoot "launcher.exe"
$WshShell = New-Object -ComObject WScript.Shell
$targets = @(
    [Environment]::GetFolderPath("Desktop"),
    (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs")
)
foreach ($dir in $targets) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $lnk = Join-Path $dir "Archetypes.lnk"
    $shortcut = $WshShell.CreateShortcut($lnk)
    $shortcut.TargetPath = $LauncherExe
    $shortcut.WorkingDirectory = $DistRoot
    $shortcut.IconLocation = $IcoPath
    $shortcut.Description = "Archetypes - Council Chamber"
    $shortcut.Save()
    Write-Host "Shortcut created: $lnk"
}

Write-Host "`nDone. Launch Archetypes from your Desktop or Start Menu."
