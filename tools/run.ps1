# Starts The Civilization Engine's observer: builds the web shell if it is missing or older than
# its sources, then runs civ-host (release build) and opens the browser.
#
#   tools\run.ps1                 # http://127.0.0.1:7420/
#   tools\run.ps1 --port 7421     # any `civ-host serve` option
#
# Needs Rust (rustup reads rust-toolchain.toml) and Node.js 22 with npm.
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$web = Join-Path $root "web"
$index = Join-Path $web "dist\index.html"

$stale = -not (Test-Path $index)
if (-not $stale) {
    $built = (Get-Item $index).LastWriteTime
    $newer = Get-ChildItem -Path (Join-Path $web "src"), (Join-Path $web "index.html") -Recurse -File |
        Where-Object { $_.LastWriteTime -gt $built } |
        Select-Object -First 1
    $stale = [bool]$newer
}

if ($stale) {
    Write-Host "Building the web shell..."
    Push-Location $web
    try {
        if (-not (Test-Path "node_modules")) {
            npm ci
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        npm run build
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    finally {
        Pop-Location
    }
}

Set-Location (Join-Path $root "kernel")
cargo build --release --locked -p civ-host
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Run the binary itself, not through `cargo run`: cargo keeps its child in a job object that
# Windows kills as soon as the window closes, before the host can save the world.
$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
& (Join-Path $target "release\civ-host.exe") serve --open @args
exit $LASTEXITCODE
