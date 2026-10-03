# Builds the kernel library tce_kernel.dll (ADR-0005) and publishes it with its PDB, its C header
# and a manifest, for the Unreal plugin or any other host:
#
#   tools\build-kernel-dll.ps1                  # into dist\tce_kernel\<target>\
#   tools\build-kernel-dll.ps1 -Dest <folder>   # into a folder (for example a plugin's Binaries)
#
# It first checks that the committed header matches the library's exports. Needs Rust with the
# MSVC toolchain (rustup reads rust-toolchain.toml).
param(
    [string]$Dest = ""
)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location (Join-Path $root "kernel")
$target = (rustc -vV | Select-String '^host: (.+)$').Matches[0].Groups[1].Value
if (-not $Dest) {
    $Dest = Join-Path $root "dist\tce_kernel\$target"
}
$header = "crates\civ-ffi\include\tce_kernel.h"

Write-Host "Checking the C header matches the exports..."
cargo test --locked --profile dll -p civ-ffi --test header --quiet
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Building tce_kernel.dll for $target (profile dll)..."
cargo build --locked --profile dll -p civ-ffi
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item "target\dll\tce_kernel.dll", "target\dll\tce_kernel.pdb", $header -Destination $Dest

$text = Get-Content $header -Raw
$abiMajor = [regex]::Match($text, '#define TCE_ABI_MAJOR (\d+)').Groups[1].Value
$abiMinor = [regex]::Match($text, '#define TCE_ABI_MINOR (\d+)').Groups[1].Value
$revision = (git -C $root rev-parse HEAD 2>$null)
if (-not $revision) { $revision = "unknown" }
$changed = @(git -C $root status --porcelain 2>$null).Count
$manifest = [ordered]@{
    library             = "tce_kernel.dll"
    symbols             = "tce_kernel.pdb"
    abi                 = "$abiMajor.$abiMinor"
    target              = $target
    profile             = "dll"
    revision            = $revision
    uncommitted_changes = $changed
    sha256              = (Get-FileHash (Join-Path $Dest "tce_kernel.dll") -Algorithm SHA256).Hash.ToLower()
    built               = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}
$manifest | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $Dest "tce_kernel.json")
Write-Host "Published tce_kernel.dll, tce_kernel.pdb, tce_kernel.h and tce_kernel.json to $Dest"
