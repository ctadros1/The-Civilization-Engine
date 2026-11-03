param([string]$Engine = "D:\Program Files\Epic Games\UE_5.8", [switch]$Editor)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$packaged = Join-Path $root 'dist\Unreal\Windows\TCE.exe'
if (-not $Editor -and (Test-Path $packaged)) {
    Start-Process -FilePath $packaged -ArgumentList @('-windowed', '-ResX=1600', '-ResY=900') -WindowStyle Normal
    exit 0
}
$editor = Join-Path $Engine 'Engine\Binaries\Win64\UnrealEditor.exe'
if (-not (Test-Path $editor)) { throw "Unreal Engine missing at $Engine" }
Start-Process -FilePath $editor -ArgumentList @(('"' + (Join-Path $root 'unreal\TCE.uproject') + '"'), '-game', '-windowed', '-ResX=1600', '-ResY=900') -WindowStyle Normal
