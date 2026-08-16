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

$WshShell = New-Object -ComObject WScript.Shell
$shortcutDirs = @(
    [Environment]::GetFolderPath("Desktop"),
    (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs")
)
$shortcutNames = @("Archetypes.lnk", "Archetypes Help.lnk", "Uninstall Archetypes.lnk")
foreach ($dir in $shortcutDirs) {
    foreach ($name in $shortcutNames) {
        $lnk = Join-Path $dir $name
        if (Test-Path $lnk) {
            Remove-Item -LiteralPath $lnk -Force
            Write-Host "Removed $lnk"
        }
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
