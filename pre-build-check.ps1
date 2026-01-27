# Pre-Build Test Script
# Quick verification before full build

Write-Host "Running pre-build checks..." -ForegroundColor Cyan
Write-Host ""

# Check Rust version
Write-Host "Rust version:" -ForegroundColor Yellow
rustc --version
cargo --version
Write-Host ""

# Check for Tauri CLI
Write-Host "Tauri CLI check:" -ForegroundColor Yellow
$tauriCheck = cargo tauri --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host $tauriCheck -ForegroundColor Green
} else {
    Write-Host "Tauri CLI not found. Installing..." -ForegroundColor Yellow
    cargo install tauri-cli
}
Write-Host ""

# Quick syntax check
Write-Host "Running cargo check..." -ForegroundColor Yellow
cargo check --all-targets
if ($LASTEXITCODE -eq 0) {
    Write-Host "Syntax check passed!" -ForegroundColor Green
} else {
    Write-Host "Syntax check failed!" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Check for clippy warnings
Write-Host "Running clippy..." -ForegroundColor Yellow
cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -eq 0) {
    Write-Host "Clippy passed - no warnings!" -ForegroundColor Green
} else {
    Write-Host "Clippy found issues. Fix before building." -ForegroundColor Yellow
}
Write-Host ""

# Verify critical files exist
Write-Host "Checking critical files..." -ForegroundColor Yellow
$criticalFiles = @(
    "Cargo.toml",
    "tauri.conf.json",
    "ui/index.html",
    "src-tauri/src/main.rs",
    "src-tauri/src/config.rs",
    "src-tauri/src/commands/staging.rs",
    "templates",
    "mud-references/index.jsonl"
)

$allExist = $true
foreach ($file in $criticalFiles) {
    if (Test-Path $file) {
        Write-Host "  [OK] $file" -ForegroundColor Green
    } else {
        Write-Host "  [MISSING] $file" -ForegroundColor Red
        $allExist = $false
    }
}
Write-Host ""

if ($allExist) {
    Write-Host "All pre-build checks passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Ready to build. Run:" -ForegroundColor Cyan
    Write-Host "  .\build-release.ps1" -ForegroundColor White
} else {
    Write-Host "Pre-build checks failed. Fix issues before building." -ForegroundColor Red
    exit 1
}
