<#
    Product install for Archetypes (Windows lifecycle).

    Copies the staged dist tree into a per-user Programs folder (no admin required)
    or %ProgramFiles%\Archetypes when that path is writable. Registers Add/Remove
    Programs under HKCU, creates Desktop + Start Menu + Help + Uninstall shortcuts.

    Prefer this over asking the Founder to run cargo. Agents may still restage
    the developer dist with install_shortcut.ps1.
#>
param(
    [string]$SourceRoot = (Join-Path $PSScriptRoot "..\dist"),
    [string]$InstallRoot = ""
)

$ErrorActionPreference = "Stop"
$SourceRoot = [System.IO.Path]::GetFullPath($SourceRoot)
if (-not (Test-Path (Join-Path $SourceRoot "launcher.exe"))) {
    throw "No staged product at $SourceRoot. Run scripts\install_shortcut.ps1 first."
}

if (-not $InstallRoot) {
    $programFiles = Join-Path $env:ProgramFiles "Archetypes"
    $userPrograms = Join-Path $env:LOCALAPPDATA "Programs\Archetypes"
    $InstallRoot = $userPrograms
    try {
        New-Item -ItemType Directory -Force -Path $programFiles | Out-Null
        $probe = Join-Path $programFiles ".write-probe"
        Set-Content -LiteralPath $probe -Value "ok" -ErrorAction Stop
        Remove-Item -LiteralPath $probe -Force
        $InstallRoot = $programFiles
    } catch {
        $InstallRoot = $userPrograms
    }
}

Write-Host "Installing Archetypes to $InstallRoot"
New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
Copy-Item (Join-Path $SourceRoot "*") -Destination $InstallRoot -Recurse -Force

$Launcher = Join-Path $InstallRoot "launcher.exe"
$Ico = Join-Path $InstallRoot "archetypes.ico"
if (-not (Test-Path $Ico)) {
    # Falling back to the exe's embedded icon is how a product quietly loses its
    # family mark. The staged dist must carry the green NeuroCognica .ico.
    throw "No product icon at $Ico. Restage with scripts\install_shortcut.ps1."
}
$UninstallScript = Join-Path $InstallRoot "scripts\uninstall_product.ps1"

# One flat .lnk in Programs\NeuroCognica, beside ChronoSophia2, NC Company
# Database and the rest of the family. Help ships inside the installed tree and
# uninstall lives in Add/Remove Programs, registered below; neither earns a
# second Start Menu entry, and neither belongs at the top level, which is where
# all three of them used to land.
# Authority: C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md
. (Join-Path $PSScriptRoot "neurocognica_start_menu.ps1")
Remove-NeuroCognicaLegacyShortcut
New-NeuroCognicaShortcut -TargetPath $Launcher -WorkingDirectory $InstallRoot -IconLocation $Ico | Out-Null
New-NeuroCognicaDesktopShortcut -TargetPath $Launcher -WorkingDirectory $InstallRoot -IconLocation $Ico | Out-Null
Update-WindowsIconCache

$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Archetypes"
New-Item -Path $uninstallKey -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value "Archetypes" -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "Publisher" -Value "NeuroCognica" -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value "0.3.0" -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value $InstallRoot -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "DisplayIcon" -Value $Ico -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "pwsh.exe -NoProfile -ExecutionPolicy Bypass -File `"$UninstallScript`"" -PropertyType String -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "NoModify" -Value 1 -PropertyType DWord -Force | Out-Null
New-ItemProperty -Path $uninstallKey -Name "NoRepair" -Value 1 -PropertyType DWord -Force | Out-Null

Write-Host "Installed. Launch from the Desktop, or from Start Menu > NeuroCognica > Archetypes."
Write-Host "Help: $(Join-Path $InstallRoot 'help\index.html'). Uninstall: Settings > Apps > Archetypes."
