<#
    Rebind the existing Archetypes Taskbar pin to a verified formal installation.

    Windows keeps a pin's target when an application is reinstalled, which can
    leave the buyer launching an old per-user developer copy.  This script is
    deliberately explicit: it refuses an unsigned or incomplete formal root,
    persists the Program Files target, then reads the shell link back.
#>
[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:ProgramFiles "Archetypes")
)

$ErrorActionPreference = "Stop"
$InstallRoot = [System.IO.Path]::GetFullPath($InstallRoot)
$launcher = Join-Path $InstallRoot "launcher.exe"
$engine = Join-Path $InstallRoot "engine.exe"
$icon = Join-Path $InstallRoot "archetypes.ico"

foreach ($required in @($launcher, $engine, $icon)) {
    if (-not (Test-Path -LiteralPath $required)) {
        throw "Formal installation is incomplete: missing $required"
    }
}
foreach ($binary in @($launcher, $engine)) {
    $signature = Get-AuthenticodeSignature -LiteralPath $binary
    if ($signature.Status -ne "Valid") {
        throw "Refusing to bind the Taskbar to an unsigned formal binary: $binary ($($signature.Status))."
    }
}

$taskbarLink = Join-Path $env:APPDATA "Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk"
if (-not (Test-Path -LiteralPath $taskbarLink)) {
    throw "No existing Archetypes Taskbar pin exists at $taskbarLink. Pin the installed application once, then rerun this verifier."
}

$shell = New-Object -ComObject WScript.Shell
$link = $shell.CreateShortcut($taskbarLink)
$link.TargetPath = $launcher
$link.WorkingDirectory = $InstallRoot
$link.IconLocation = $icon
$link.Save()

$verified = $shell.CreateShortcut($taskbarLink)
if ($verified.TargetPath -ne $launcher -or $verified.WorkingDirectory -ne $InstallRoot) {
    throw "Taskbar formal-install verification failed. Expected $launcher / $InstallRoot; got $($verified.TargetPath) / $($verified.WorkingDirectory)."
}

Write-Host "Verified pinned Taskbar target: $($verified.TargetPath)"
Write-Host "Verified pinned Taskbar working directory: $($verified.WorkingDirectory)"
