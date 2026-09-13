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

$ScriptsDst = Join-Path $DistRoot "scripts"
New-Item -ItemType Directory -Force -Path $ScriptsDst | Out-Null
Copy-Item (Join-Path $RepoRoot "scripts\dependencies.json") (Join-Path $ScriptsDst "dependencies.json") -Force
Copy-Item (Join-Path $RepoRoot "scripts\uninstall_product.ps1") (Join-Path $ScriptsDst "uninstall_product.ps1") -Force -ErrorAction SilentlyContinue
# uninstall_product.ps1 dot-sources this from its own directory, so the installed
# tree needs it too or uninstall throws instead of removing the shortcut.
Copy-Item (Join-Path $RepoRoot "scripts\neurocognica_start_menu.ps1") (Join-Path $ScriptsDst "neurocognica_start_menu.ps1") -Force
Copy-Item (Join-Path $RepoRoot "scripts\import_chronos_object.py") (Join-Path $ScriptsDst "import_chronos_object.py") -Force
Copy-Item (Join-Path $RepoRoot "scripts\review_object_gate.py") (Join-Path $ScriptsDst "review_object_gate.py") -Force
Copy-Item (Join-Path $RepoRoot "scripts\full_volume_generator.py") (Join-Path $ScriptsDst "full_volume_generator.py") -Force

$HelpSrc = Join-Path $RepoRoot "assets\help"
$HelpDst = Join-Path $DistRoot "help"
if (Test-Path $HelpSrc) {
    New-Item -ItemType Directory -Force -Path $HelpDst | Out-Null
    Copy-Item (Join-Path $HelpSrc "*") -Destination $HelpDst -Recurse -Force
}

# Stage the product icon. It is NOT generated here.
#
# This block used to rasterise the old architect glyph PNG into a single 256px
# .ico and write it over BOTH the dist copy and the repo copy on every run. That
# is how Archetypes ended up wearing a cyan spider glyph instead of the
# NeuroCognica family mark, and why the mark could not be fixed by dropping a
# correct file into the repo: the next restage overwrote it.
#
# The family icon is generated once, outside every repo:
#   python C:\NeuroCognica_Brand\scripts\build_product_brand.py archetypes
# and its output is committed at assets\icons\archetypes.ico - green plate,
# white dots, all seven Windows sizes. Copy it; never redraw it.
$RepoIco = Join-Path $RepoRoot "assets\icons\archetypes.ico"
if (-not (Test-Path -LiteralPath $RepoIco)) {
    throw "Missing product icon $RepoIco. Regenerate it with the brand kit; do not hand-draw one."
}
$IcoPath = Join-Path $DistRoot "archetypes.ico"
Copy-Item -LiteralPath $RepoIco -Destination $IcoPath -Force
Write-Host "Staged product icon: $IcoPath"

# Developer staging creates the same single family shortcut a product install
# does, pointed at the staged dist tree. Same folder, same name, same icon, so
# a later install_product.ps1 replaces it instead of leaving two Archetypes in
# the Start Menu. C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md.
. (Join-Path $PSScriptRoot "neurocognica_start_menu.ps1")
$LauncherExe = Join-Path $DistRoot "launcher.exe"
Remove-NeuroCognicaLegacyShortcut
New-NeuroCognicaShortcut -TargetPath $LauncherExe -WorkingDirectory $DistRoot -IconLocation $IcoPath | Out-Null
New-NeuroCognicaDesktopShortcut -TargetPath $LauncherExe -WorkingDirectory $DistRoot -IconLocation $IcoPath | Out-Null

# Also automatically synchronize the freshly built product to the user's installed Programs
# root (%LOCALAPPDATA%\Programs\Archetypes) where pinned Taskbar shortcuts point.
$userPrograms = Join-Path $env:LOCALAPPDATA "Programs\Archetypes"
if (Test-Path $userPrograms) {
    Write-Host "Syncing build to installed Programs directory: $userPrograms"
    & (Join-Path $PSScriptRoot "install_product.ps1") -SourceRoot $DistRoot -InstallRoot $userPrograms
}

# Update Windows pinned Taskbar shortcut if present
$taskbarLnk = Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk'
if (Test-Path $taskbarLnk) {
    Write-Host "Refreshing Taskbar pinned shortcut: $taskbarLnk"
    $sh = New-Object -ComObject WScript.Shell
    $taskbarShortcut = $sh.CreateShortcut($taskbarLnk)
    $installedLauncher = Join-Path $userPrograms "launcher.exe"
    if (Test-Path $installedLauncher) {
        $taskbarShortcut.TargetPath = $installedLauncher
        $taskbarShortcut.WorkingDirectory = $userPrograms
        $taskbarShortcut.IconLocation = Join-Path $userPrograms "archetypes.ico"
    } else {
        $taskbarShortcut.TargetPath = $LauncherExe
        $taskbarShortcut.WorkingDirectory = $DistRoot
        $taskbarShortcut.IconLocation = $IcoPath
    }
    $taskbarShortcut.Save()
    # Read the persisted shell-link back.  Saving a COM shortcut object is not
    # evidence that Explorer will launch the intended installed launcher.
    $verifiedTaskbarShortcut = $sh.CreateShortcut($taskbarLnk)
    if ($verifiedTaskbarShortcut.TargetPath -ne $installedLauncher -or
        $verifiedTaskbarShortcut.WorkingDirectory -ne $userPrograms) {
        throw "Taskbar shortcut verification failed. Expected $installedLauncher with working directory $userPrograms; got $($verifiedTaskbarShortcut.TargetPath) / $($verifiedTaskbarShortcut.WorkingDirectory)."
    }
    Write-Host "Verified Taskbar target: $($verifiedTaskbarShortcut.TargetPath)"
}

Update-WindowsIconCache

Write-Host "`nDone. All council voices are installed. Launch Archetypes from Taskbar, Desktop, or Start Menu > NeuroCognica."
