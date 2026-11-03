param(
    [string]$Engine = "D:\Program Files\Epic Games\UE_5.8",
    [switch]$Package,
    [switch]$SkipKernel,
    [switch]$Rebuild
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$project = Join-Path $root 'unreal\TCE.uproject'
if (-not (Test-Path (Join-Path $Engine 'Engine\Build\BatchFiles\Build.bat'))) {
    throw "Unreal Engine not found at $Engine. Pass -Engine <installation>."
}
$version = Get-Content (Join-Path $Engine 'Engine\Build\Build.version') -Raw | ConvertFrom-Json
if ($version.MajorVersion -ne 5 -or $version.MinorVersion -ne 8 -or $version.PatchVersion -ne 3) {
    throw 'This milestone is pinned to Unreal Engine 5.8.3.'
}
if (-not $SkipKernel) {
    & (Join-Path $PSScriptRoot 'build-kernel-dll.ps1') -TargetDir (Join-Path $root 'unreal\Intermediate\Kernel')
    if ($LASTEXITCODE -ne 0) { throw 'Kernel build failed' }
    Push-Location (Join-Path $root 'web')
    try {
        npm ci
        if ($LASTEXITCODE -ne 0) { throw 'Web dependencies failed' }
        npm run build
        if ($LASTEXITCODE -ne 0) { throw 'Web build failed' }
    } finally { Pop-Location }
}
$buildArgs = @()
if ($Rebuild) { $buildArgs = @('-Rebuild','-ForceRulesCompile','-NoUBTMakefiles') }
$kernelLibrary = Join-Path $root 'dist\tce_kernel\x86_64-pc-windows-msvc\tce_kernel.dll'
$binaryDirectory = Join-Path $root 'unreal\Binaries\Win64'
New-Item -ItemType Directory -Force -Path $binaryDirectory | Out-Null
# Copy explicitly: a clock adjustment must not leave an older library beside a fresh client.
Copy-Item -LiteralPath $kernelLibrary -Destination (Join-Path $binaryDirectory 'tce_kernel.dll') -Force
& (Join-Path $Engine 'Engine\Build\BatchFiles\Build.bat') TCEEditor Win64 Development "-Project=$project" -WaitMutex -NoHotReloadFromIDE @buildArgs
if ($LASTEXITCODE -ne 0) { throw 'Unreal editor build failed' }
& (Join-Path $Engine 'Engine\Binaries\Win64\UnrealEditor-Cmd.exe') $project -run=pythonscript "-script=$root\unreal\Tools\build_assets.py" -unattended -nosplash -nullrhi
if ($LASTEXITCODE -ne 0) { throw 'Unreal asset generation failed' }
if ($Package) {
    if ($Rebuild) {
        & (Join-Path $Engine 'Engine\Build\BatchFiles\Build.bat') TCE Win64 Development "-Project=$project" -WaitMutex @buildArgs
        if ($LASTEXITCODE -ne 0) { throw 'Unreal game rebuild failed' }
    }
    & (Join-Path $Engine 'Engine\Build\BatchFiles\RunUAT.bat') BuildCookRun "-project=$project" -noP4 -platform=Win64 -clientconfig=Development -build -cook -stage -pak -archive "-archivedirectory=$root\dist\Unreal"
    if ($LASTEXITCODE -ne 0) { throw 'Unreal packaging failed' }
}
