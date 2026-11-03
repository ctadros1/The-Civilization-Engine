param([int]$TimeoutSeconds = 120)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'dist\Unreal\Windows\TCE.exe'
$log = Join-Path $root 'dist\Unreal\Windows\TCE\Saved\Logs\UnrealSmoke.log'
if (-not (Test-Path $exe)) { throw 'Build the Windows package first with tools/build-unreal.ps1 -Package.' }
$started = Get-Date
$process = Start-Process -FilePath $exe -ArgumentList @('-TCESmoke','-windowed','-ResX=1600','-ResY=900','-unattended','-log=UnrealSmoke.log') -WindowStyle Hidden -PassThru
try {
    Wait-Process -Id $process.Id -Timeout $TimeoutSeconds -ErrorAction Stop
} catch {
    Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
    throw "Unreal smoke exceeded $TimeoutSeconds seconds. Inspect $log."
}
$process.Refresh()
if ($process.ExitCode -ne 0 -or -not (Test-Path $log)) { throw "Unreal smoke failed. Inspect $log." }
if ((Get-Item $log).LastWriteTime -lt $started -or -not (Select-String -Path $log -Pattern 'TCE_SMOKE_OK' -Quiet)) {
    throw "Unreal did not confirm the world, terrain and people. Inspect $log."
}
Write-Host 'Unreal packaged smoke passed: kernel loaded, world created, terrain and people received, clean shutdown.'
