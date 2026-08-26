<#
    The NeuroCognica Start Menu family rule, in one place.

    Authority: C:\NeuroCognica_Brand\docs\START_MENU_FAMILY.md

    One flat .lnk per product in Programs\NeuroCognica. Not a nested product
    folder, not a top-level Programs\Archetypes.lnk, and not a second shortcut
    for help or uninstall - uninstall is reached through Add/Remove Programs,
    which install_product.ps1 registers.

    Archetypes shipped three top-level shortcuts before 2026-08-25, which is why
    Remove-NeuroCognicaLegacyShortcut exists and is called on install as well as
    on uninstall: a buyer who upgrades must not be left with both.

    Dot-source it:  . (Join-Path $PSScriptRoot "neurocognica_start_menu.ps1")
#>

$script:NeuroCognicaProductName = "Archetypes"

function Get-NeuroCognicaFamilyDir {
    Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\NeuroCognica"
}

function Get-NeuroCognicaShortcutPath {
    Join-Path (Get-NeuroCognicaFamilyDir) "$script:NeuroCognicaProductName.lnk"
}

function New-NeuroCognicaShortcut {
    param(
        [Parameter(Mandatory = $true)][string]$TargetPath,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string]$IconLocation,
        [string]$Description = "Archetypes - Council Chamber"
    )

    if (-not (Test-Path -LiteralPath $TargetPath)) {
        throw "Refusing to create a shortcut to a missing target: $TargetPath"
    }
    if (-not (Test-Path -LiteralPath $IconLocation)) {
        throw "Refusing to create a shortcut with a missing icon: $IconLocation"
    }

    $FamilyDir = Get-NeuroCognicaFamilyDir
    New-Item -ItemType Directory -Force -Path $FamilyDir | Out-Null

    $Lnk = Get-NeuroCognicaShortcutPath
    $Shell = New-Object -ComObject WScript.Shell
    $Shortcut = $Shell.CreateShortcut($Lnk)
    $Shortcut.TargetPath = $TargetPath
    $Shortcut.WorkingDirectory = $WorkingDirectory
    $Shortcut.IconLocation = $IconLocation
    $Shortcut.Description = $Description
    $Shortcut.Save()
    Write-Host "Start Menu: $Lnk"
    return $Lnk
}

function New-NeuroCognicaDesktopShortcut {
    param(
        [Parameter(Mandatory = $true)][string]$TargetPath,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string]$IconLocation,
        [string]$Description = "Archetypes - Council Chamber"
    )

    $Lnk = Join-Path ([Environment]::GetFolderPath("Desktop")) "$script:NeuroCognicaProductName.lnk"
    $Shell = New-Object -ComObject WScript.Shell
    $Shortcut = $Shell.CreateShortcut($Lnk)
    $Shortcut.TargetPath = $TargetPath
    $Shortcut.WorkingDirectory = $WorkingDirectory
    $Shortcut.IconLocation = $IconLocation
    $Shortcut.Description = $Description
    $Shortcut.Save()
    Write-Host "Desktop: $Lnk"
    return $Lnk
}

function Remove-NeuroCognicaLegacyShortcut {
    <#
        The top-level names Archetypes used to write. Removing them is not
        optional cleanup: leaving them is how the buyer ends up with the product
        listed twice, one of the two pointing at a dev checkout.
    #>
    $Programs = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    $Legacy = @(
        (Join-Path $Programs "Archetypes.lnk"),
        (Join-Path $Programs "Archetypes Help.lnk"),
        (Join-Path $Programs "Uninstall Archetypes.lnk"),
        (Join-Path $Programs "Archetypes")
    )
    foreach ($Path in $Legacy) {
        if (Test-Path -LiteralPath $Path) {
            Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "Removed legacy Start Menu entry: $Path"
        }
    }
}

function Update-WindowsIconCache {
    <#
        Windows caches shortcut icons by path. A new .ico at an old path is a
        new icon nobody sees until the shell is told.
    #>
    try {
        Add-Type -Namespace NcShell -Name Api -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("shell32.dll")]
public static extern void SHChangeNotify(int eventId, uint flags, System.IntPtr item1, System.IntPtr item2);
"@ -ErrorAction Stop
        # SHCNE_ASSOCCHANGED, SHCNF_IDLIST
        [NcShell.Api]::SHChangeNotify(0x08000000, 0x0000, [System.IntPtr]::Zero, [System.IntPtr]::Zero)
    } catch {
        Write-Host "  (icon cache refresh unavailable: $_)"
    }
}
