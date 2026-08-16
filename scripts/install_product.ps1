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
if (-not (Test-Path $Ico)) { $Ico = $Launcher }
$UninstallScript = Join-Path $InstallRoot "scripts\uninstall_product.ps1"
$HelpHtml = Join-Path $InstallRoot "help\index.html"

$WshShell = New-Object -ComObject WScript.Shell
$targets = @(
    [Environment]::GetFolderPath("Desktop"),
    (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs")
)
foreach ($dir in $targets) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $lnk = Join-Path $dir "Archetypes.lnk"
    $shortcut = $WshShell.CreateShortcut($lnk)
    $shortcut.TargetPath = $Launcher
    $shortcut.WorkingDirectory = $InstallRoot
    $shortcut.IconLocation = $Ico
    $shortcut.Description = "Archetypes - Council Chamber"
    $shortcut.Save()
}

$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
if (Test-Path $HelpHtml) {
    $helpLnk = Join-Path $StartMenu "Archetypes Help.lnk"
    $helpShortcut = $WshShell.CreateShortcut($helpLnk)
    $helpShortcut.TargetPath = $HelpHtml
    $helpShortcut.WorkingDirectory = (Join-Path $InstallRoot "help")
    $helpShortcut.Description = "Archetypes — Witness Manual"
    $helpShortcut.Save()
}
if (Test-Path $UninstallScript) {
    $uninstallLnk = Join-Path $StartMenu "Uninstall Archetypes.lnk"
    $uninstallShortcut = $WshShell.CreateShortcut($uninstallLnk)
    $uninstallShortcut.TargetPath = "pwsh.exe"
    $uninstallShortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$UninstallScript`""
    $uninstallShortcut.WorkingDirectory = $InstallRoot
    $uninstallShortcut.Description = "Remove Archetypes"
    $uninstallShortcut.Save()
}

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

Write-Host "Installed. Launch from Desktop Archetypes, or Start Menu. Help and Uninstall are in Start Menu."
