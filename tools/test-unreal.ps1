param(
    [ValidateSet('Smoke','Spike','Night')][string]$Scenario = 'Smoke',
    [int]$TimeoutSeconds = 240
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'dist\Unreal\Windows\TCE.exe'
$name = "M2$Scenario"
$log = Join-Path $root "dist\Unreal\Windows\TCE\Saved\Logs\$name.log"
if (-not (Test-Path $exe)) { throw 'Build the Windows package first with tools/build-unreal.ps1 -Package.' }
$run = Join-Path $root ("recordings\m2-checks\" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run -Force | Out-Null
$arguments = @('-windowed','-ResX=2560','-ResY=1440','-unattended',"-log=$name.log","-TCESaves=$run\worlds")
$marker = 'TCE_SMOKE_OK'
if ($Scenario -eq 'Smoke') { $arguments += '-TCESmoke' }
if ($Scenario -eq 'Spike') {
    Push-Location (Join-Path $root 'kernel')
    try {
        cargo run --locked --profile dll -p civ-sim --example m2_terrain_fixture --target-dir "$root\unreal\Intermediate\Kernel" -- $run
        if ($LASTEXITCODE -ne 0) { throw 'Kernel terrain fixture failed.' }
    } finally { Pop-Location }
    $arguments += @('-TCESpike',"-TCESpikeDir=$run")
    $marker = 'TCE_SPIKE_OK'
}
if ($Scenario -eq 'Night') {
    $arguments += @('-TCEDemo','-TCEDemoRun','-TCEDemoExit','-TCEDemoImage=M2Night.png')
    $marker = 'TCE_DEMO_FRAME_MS'
}
$started = Get-Date
$quotedArguments = $arguments | ForEach-Object { '"' + $_ + '"' }
$process = Start-Process -FilePath $exe -ArgumentList $quotedArguments -WindowStyle Hidden -PassThru
try {
    Wait-Process -Id $process.Id -Timeout $TimeoutSeconds -ErrorAction Stop
} catch {
    Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
    throw "Unreal $Scenario exceeded $TimeoutSeconds seconds. Inspect $log."
}
$process.Refresh()
if ($process.ExitCode -ne 0 -or -not (Test-Path $log)) { throw "Unreal $Scenario failed. Inspect $log." }
if ((Get-Item $log).LastWriteTime -lt $started -or -not (Select-String -Path $log -Pattern $marker -Quiet)) {
    throw "Unreal did not confirm $Scenario. Inspect $log."
}
if ($Scenario -eq 'Night') {
    $timing = (Select-String -Path $log -Pattern 'TCE_DEMO_FRAME_MS' | Select-Object -Last 1).Line
    $value = [regex]::Match($timing,'p99=([0-9.]+)').Groups[1].Value
    if (-not $value -or [double]::Parse($value,[Globalization.CultureInfo]::InvariantCulture) -gt 16.667) {
        throw "Night village exceeded the 60 fps p99 budget. Inspect $log."
    }
}
if (Test-Path (Join-Path $run 'worlds\.session.json')) { throw 'Unreal left an active session marker after closing.' }
Copy-Item -LiteralPath $log -Destination (Join-Path $run "$name.log")
Select-String -Path $log -Pattern 'TCE_(SMOKE_OK|SPIKE_RESULT|DEMO_FRAME_MS)' | ForEach-Object { Write-Host $_.Line }
Write-Host "Packaged $Scenario passed. Evidence: $run"
exit 0
