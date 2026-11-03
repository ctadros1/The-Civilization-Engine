param([string]$FFmpeg = 'ffmpeg', [string]$SourceVideo = '', [int]$TimeoutSeconds = 240)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'dist\Unreal\Windows\TCE.exe'
if (-not $SourceVideo -and -not (Test-Path $exe)) { throw 'Build the Windows package first.' }
$run = Join-Path $root ('recordings\m2-capture\' + [guid]::NewGuid().ToString('N'))
$frames = Join-Path $run 'frames'
New-Item -ItemType Directory -Path $frames -Force | Out-Null
if ($SourceVideo) {
    $inputArguments = @('-i',(Resolve-Path -LiteralPath $SourceVideo).Path)
} else {
$arguments = @('-windowed','-ResX=1920','-ResY=1080','-unattended','-TCEDemo','-TCEDemoRun','-TCEDemoExit',
    '-TCEDemoImage=M2Capture.png','-log=M2Capture.log',"-TCEDemoFrames=$frames","-TCESaves=$run\worlds")
$quotedArguments = $arguments | ForEach-Object { '"' + $_ + '"' }
$process = Start-Process -FilePath $exe -ArgumentList $quotedArguments -WindowStyle Hidden -PassThru
try { Wait-Process -Id $process.Id -Timeout $TimeoutSeconds -ErrorAction Stop }
catch { Stop-Process -Id $process.Id -ErrorAction SilentlyContinue; throw 'Unreal capture timed out.' }
$process.Refresh()
if ($process.ExitCode -ne 0) { throw 'Unreal capture failed.' }
$images = @(Get-ChildItem -LiteralPath $frames -Filter 'frame*.jpg' | Sort-Object Name)
if ($images.Count -lt 60) { throw 'Unreal did not produce enough capture frames.' }
# Preserve the observed wall-clock pacing. Capture is separate from the performance gate.
$concat = [Collections.Generic.List[string]]::new()
$concat.Add('ffconcat version 1.0')
for ($i=0; $i -lt $images.Count; $i++) {
    $concat.Add("file '$($images[$i].Name)'")
    $duration = .1
    if ($i+1 -lt $images.Count) { $duration = [Math]::Max(.001,($images[$i+1].LastWriteTimeUtc-$images[$i].LastWriteTimeUtc).TotalSeconds) }
    $concat.Add('duration ' + $duration.ToString('F6',[Globalization.CultureInfo]::InvariantCulture))
}
$concat.Add("file '$($images[-1].Name)'")
$concat | Set-Content -LiteralPath (Join-Path $frames 'frames.txt') -Encoding utf8NoBOM
$inputArguments = @('-f','concat','-safe','0','-i','frames.txt')
}
Copy-Item -LiteralPath 'C:\Windows\Fonts\seguisb.ttf' -Destination (Join-Path $frames 'font.ttf')
$output = Join-Path $root 'demos\m2\m2-first-light.mp4'
$rawOutput = Join-Path $root 'demos\m2\m2-first-light-raw.mp4'
$filter = "fps=30,scale=1920:1080,drawtext=fontfile=font.ttf:text='M2 | First light in Unreal':x=40:y=h-100:fontsize=34:fontcolor=white:box=1:boxcolor=0x101c25@0.8:boxborderw=12:enable='lt(t,3)',drawtext=fontfile=font.ttf:text='A kernel-lived village after 90 days':x=40:y=h-100:fontsize=28:fontcolor=white:box=1:boxcolor=0x101c25@0.8:boxborderw=12:enable='between(t,3,6)',drawtext=fontfile=font.ttf:text='Follow camera | Hearth light uses household fuel':x=40:y=h-100:fontsize=28:fontcolor=white:box=1:boxcolor=0x101c25@0.8:boxborderw=12:enable='gt(t,6)'"
Push-Location $frames
try {
    if (-not $SourceVideo) {
        & $FFmpeg -y -hide_banner -loglevel warning @inputArguments -vf 'fps=30,scale=1920:1080' -c:v libx264 -preset medium -crf 18 -pix_fmt yuv420p -movflags +faststart $rawOutput
        if ($LASTEXITCODE -ne 0) { throw 'Source video encoding failed.' }
    }
    & $FFmpeg -y -hide_banner -loglevel warning @inputArguments -vf $filter -c:v libx264 -preset medium -crf 19 -pix_fmt yuv420p -movflags +faststart $output
    if ($LASTEXITCODE -ne 0) { throw 'Video encoding failed.' }
} finally { Pop-Location }
if (-not $SourceVideo) { Copy-Item -LiteralPath (Join-Path $root 'dist\Unreal\Windows\TCE\Saved\Logs\M2Capture.log') -Destination $run }
Write-Host "Video: $output. Working files: $run"
