param(
    [int]$ProcessId = 0
)

$ErrorActionPreference = 'Stop'
$installDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$targetExe = Join-Path $installDir 'layout-fixer.exe'
$startupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'Layout Fixer.lnk'
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\LayoutFixer'

if ($ProcessId -gt 0) {
    $running = @(Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)
} else {
    $running = @(Get-CimInstance Win32_Process -Filter "Name = 'layout-fixer.exe'" |
        Where-Object { $_.ExecutablePath -eq $targetExe } |
        ForEach-Object { Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue })
}
foreach ($process in $running) {
    Wait-Process -Id $process.Id -Timeout 5 -ErrorAction SilentlyContinue
    if (Get-Process -Id $process.Id -ErrorAction SilentlyContinue) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
}

if (Test-Path $startupShortcut) {
    Remove-Item -LiteralPath $startupShortcut -Force
}
if (Test-Path $key) {
    Remove-Item -LiteralPath $key -Recurse -Force
}
if (Test-Path $targetExe) {
    Remove-Item -LiteralPath $targetExe -Force
}

$cleanup = "Start-Sleep -Milliseconds 500; Remove-Item -LiteralPath '$installDir' -Recurse -Force -ErrorAction SilentlyContinue"
Start-Process -FilePath powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', $cleanup -WindowStyle Hidden
Write-Host 'Layout Fixer was removed.'
