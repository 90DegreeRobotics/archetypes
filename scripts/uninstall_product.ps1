<#
    Uninstall Archetypes from the product install root.

    Removes binaries, shortcuts, and the Add/Remove Programs entry.
    Leaves %LOCALAPPDATA%\NeuroCognica\Archetypes (Witness profile, ledger, logs)
    unless -RemoveAppData is passed.
#>
param(
    [switch]$RemoveAppData
)

$ErrorActionPreference = "Stop"

$InstallRoot = $null
$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Archetypes"
if (Test-Path $uninstallKey) {
    $InstallRoot = (Get-ItemProperty -Path $uninstallKey -Name InstallLocation -ErrorAction SilentlyContinue).InstallLocation
}
if (-not $InstallRoot) {
    $InstallRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
}

# Remove this product's shortcut and every legacy name it ever wrote. The
# NeuroCognica folder itself is NEVER removed: its other members are still
# installed. C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md.
. (Join-Path $PSScriptRoot "neurocognica_start_menu.ps1")
$FamilyLnk = Get-NeuroCognicaShortcutPath
if (Test-Path -LiteralPath $FamilyLnk) {
    Remove-Item -LiteralPath $FamilyLnk -Force
    Write-Host "Removed $FamilyLnk"
}
Remove-NeuroCognicaLegacyShortcut

foreach ($name in @("Archetypes.lnk", "Archetypes Help.lnk", "Uninstall Archetypes.lnk")) {
    $lnk = Join-Path ([Environment]::GetFolderPath("Desktop")) $name
    if (Test-Path -LiteralPath $lnk) {
        Remove-Item -LiteralPath $lnk -Force
        Write-Host "Removed $lnk"
    }
}

if (Test-Path $uninstallKey) {
    Remove-Item -Path $uninstallKey -Recurse -Force
}

if ($InstallRoot -and (Test-Path $InstallRoot)) {
    $protected = @(
        (Join-Path $env:USERPROFILE "Desktop"),
        $env:SystemRoot,
        $env:ProgramFiles
    )
    $normalized = [System.IO.Path]::GetFullPath($InstallRoot).TrimEnd('\')
    $ok = $normalized -match '\\Archetypes$'
    if (-not $ok) {
        throw "Refusing to delete unexpected install root: $normalized"
    }
    Remove-Item -LiteralPath $normalized -Recurse -Force
    Write-Host "Removed $normalized"
}

if ($RemoveAppData) {
    $appData = Join-Path $env:LOCALAPPDATA "NeuroCognica\Archetypes"
    if (Test-Path $appData) {
        Remove-Item -LiteralPath $appData -Recurse -Force
        Write-Host "Removed $appData"
    }
}

Write-Host "Archetypes uninstall complete."
