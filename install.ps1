param(
    [switch]$NoStartup
)

$ErrorActionPreference = 'Stop'
$source = Split-Path -Parent $MyInvocation.MyCommand.Path
$installDir = Join-Path $env:LOCALAPPDATA 'Programs\LayoutFixer'
$targetExe = Join-Path $installDir 'layout-fixer.exe'
$startupDir = [Environment]::GetFolderPath('Startup')
$startupShortcut = Join-Path $startupDir 'Layout Fixer.lnk'
$uninstaller = Join-Path $installDir 'uninstall.ps1'

if (-not (Test-Path (Join-Path $source 'layout-fixer.exe') -PathType Leaf)) {
    throw 'layout-fixer.exe must be next to install.ps1. Extract the release ZIP first.'
}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
$running = Get-CimInstance Win32_Process -Filter "Name = 'layout-fixer.exe'" |
    Where-Object { $_.ExecutablePath -eq $targetExe }
foreach ($process in $running) {
    Stop-Process -Id $process.ProcessId -Force -ErrorAction SilentlyContinue
}
Start-Sleep -Milliseconds 250
Copy-Item (Join-Path $source 'layout-fixer.exe') $targetExe -Force
Copy-Item (Join-Path $source 'uninstall.ps1') $uninstaller -Force

if (-not $NoStartup) {
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($startupShortcut)
    $shortcut.TargetPath = $targetExe
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Save()
}

$uninstallCommand = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$uninstaller`""
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\LayoutFixer'
New-Item -Path $key -Force | Out-Null
New-ItemProperty -Path $key -Name DisplayName -Value 'Layout Fixer' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $key -Name DisplayVersion -Value '0.1.0-preview.6' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $key -Name Publisher -Value 'Layout Fixer contributors' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $key -Name InstallLocation -Value $installDir -PropertyType String -Force | Out-Null
New-ItemProperty -Path $key -Name UninstallString -Value $uninstallCommand -PropertyType String -Force | Out-Null
New-ItemProperty -Path $key -Name NoModify -Value 1 -PropertyType DWord -Force | Out-Null
New-ItemProperty -Path $key -Name NoRepair -Value 1 -PropertyType DWord -Force | Out-Null

Start-Process -FilePath $targetExe -WorkingDirectory $installDir
Write-Host "Installed Layout Fixer to $installDir"
