$sh = New-Object -ComObject WScript.Shell
$lnkPath = Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk'
if (Test-Path $lnkPath) {
    $lnk = $sh.CreateShortcut($lnkPath)
    Write-Host "Taskbar Lnk Path: $lnkPath"
    Write-Host "TargetPath:       $($lnk.TargetPath)"
    Write-Host "Arguments:        $($lnk.Arguments)"
    Write-Host "WorkingDirectory: $($lnk.WorkingDirectory)"
} else {
    Write-Host "Not found: $lnkPath"
}
