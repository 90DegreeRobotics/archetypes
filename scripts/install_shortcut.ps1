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

# Install the pinned offline speech runtime and multi-speaker Kokoro model directly
# beside the installed binaries. The launcher and engine prefer this portable root,
# so a Desktop launch never depends on repository or proof-directory paths.
Write-Host "Installing/verifying offline council voices in $DistRoot..."
& (Join-Path $RepoRoot "scripts\setup_windows.ps1") -InstallRoot $DistRoot -NonInteractive
if ($LASTEXITCODE -ne 0) { throw "offline voice bootstrap failed" }

Copy-Item (Join-Path $RepoRoot "target\release\engine.exe") $DistRoot -Force
Copy-Item (Join-Path $RepoRoot "target\release\launcher.exe") $DistRoot -Force

# The release engine loads ./assets relative to its working directory.
# Preserve runtime-staged Comfy chat renders under standard_mecha/renders so a
# Desktop restage does not wipe the player's in-chat paintings.
$AssetsSrc = Join-Path $RepoRoot "assets"
$AssetsDst = Join-Path $DistRoot "assets"
$RendersRel = "standard_mecha\renders"
$RendersDst = Join-Path $AssetsDst $RendersRel
$RendersBackup = Join-Path $env:TEMP ("archetypes-renders-backup-" + [guid]::NewGuid().ToString("N"))
if (Test-Path $RendersDst) {
    New-Item -ItemType Directory -Force -Path $RendersBackup | Out-Null
    Copy-Item (Join-Path $RendersDst "*") -Destination $RendersBackup -Recurse -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $AssetsDst | Out-Null
Get-ChildItem $AssetsSrc | Copy-Item -Destination $AssetsDst -Recurse -Force
New-Item -ItemType Directory -Force -Path $RendersDst | Out-Null
if (Test-Path $RendersBackup) {
    Copy-Item (Join-Path $RendersBackup "*") -Destination $RendersDst -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $RendersBackup -Recurse -Force -ErrorAction SilentlyContinue
}
Write-Host "Staged runtime to $DistRoot (preserved chat renders under assets\$RendersRel)"

# Build / refresh an .ico so the Desktop shortcut and embedded exe icons stay aligned.
$IcoPath = Join-Path $DistRoot "archetypes.ico"
$IconPng = Join-Path $RepoRoot "assets\icons\architect-icon.png"
$RepoIco = Join-Path $RepoRoot "assets\icons\archetypes.ico"
try {
    Add-Type -AssemblyName System.Drawing
    $src = [System.Drawing.Image]::FromFile($IconPng)
    $bmp = New-Object System.Drawing.Bitmap $src, 256, 256
    $hicon = $bmp.GetHicon()
    $icon = [System.Drawing.Icon]::FromHandle($hicon)
    foreach ($dest in @($IcoPath, $RepoIco)) {
        $stream = [System.IO.File]::Create($dest)
        $icon.Save($stream)
        $stream.Close()
    }
    $icon.Dispose(); $bmp.Dispose(); $src.Dispose()
    Write-Host "Icon written to $IcoPath (and $RepoIco for winres embeds)"
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

Write-Host "`nDone. All council voices are installed. Launch Archetypes from your Desktop or Start Menu."
