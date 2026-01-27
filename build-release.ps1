# LPC Dev Assistant - Release Build Script v1.3.0
# This script helps build and package the release installers

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "LPC Dev Assistant - Release Build v1.3.0" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Clean previous builds
Write-Host "[1/7] Cleaning previous builds..." -ForegroundColor Yellow
if (Test-Path "target/release") {
    Remove-Item -Path "target/release" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  Removed target/release" -ForegroundColor Green
}
cargo clean
Write-Host "  Cargo clean completed" -ForegroundColor Green
Write-Host ""

# Step 2: Verify version numbers
Write-Host "[2/7] Verifying version numbers..." -ForegroundColor Yellow
$cargoVersion = (Get-Content "Cargo.toml" | Select-String "version = " | Select-Object -First 1) -replace '.*version = "([^"]+)".*', '$1'
$tauriVersion = (Get-Content "tauri.conf.json" | Select-String '"version":' | Select-Object -First 1) -replace '.*"version":\s*"([^"]+)".*', '$1'

if ($cargoVersion -eq "1.3.0" -and $tauriVersion -eq "1.3.0") {
    Write-Host "  Cargo.toml version: $cargoVersion" -ForegroundColor Green
    Write-Host "  tauri.conf.json version: $tauriVersion" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Version mismatch!" -ForegroundColor Red
    Write-Host "  Cargo.toml: $cargoVersion" -ForegroundColor Red
    Write-Host "  tauri.conf.json: $tauriVersion" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 3: Check for non-ASCII characters
Write-Host "[3/7] Checking for non-ASCII characters..." -ForegroundColor Yellow
$nonAscii = Select-String -Path "ui/index.html" -Pattern "[^\x00-\x7F]" -ErrorAction SilentlyContinue
if ($nonAscii) {
    Write-Host "  WARNING: Found non-ASCII characters in ui/index.html" -ForegroundColor Red
    $nonAscii | ForEach-Object { Write-Host "    Line $($_.LineNumber): $($_.Line)" -ForegroundColor Red }
} else {
    Write-Host "  No non-ASCII characters found" -ForegroundColor Green
}
Write-Host ""

# Step 4: Test compilation
Write-Host "[4/7] Testing release compilation..." -ForegroundColor Yellow
Write-Host "  This may take several minutes..." -ForegroundColor Gray
$buildOutput = cargo build --release 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Release build successful!" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Build failed!" -ForegroundColor Red
    Write-Host $buildOutput -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 5: Build Tauri installers
Write-Host "[5/7] Building Tauri installers..." -ForegroundColor Yellow
Write-Host "  This will take 10-20 minutes..." -ForegroundColor Gray
Write-Host "  Creating MSI and NSIS installers..." -ForegroundColor Gray
$tauriBuildOutput = cargo tauri build 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Tauri build successful!" -ForegroundColor Green
} else {
    Write-Host "  ERROR: Tauri build failed!" -ForegroundColor Red
    Write-Host $tauriBuildOutput -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 6: Verify installers
Write-Host "[6/7] Verifying installers..." -ForegroundColor Yellow
$msiPath = "target/release/bundle/msi/LPC Dev Assistant_1.3.0_x64_en-US.msi"
$nsisPath = "target/release/bundle/nsis/LPC Dev Assistant_1.3.0_x64-setup.exe"

if (Test-Path $msiPath) {
    $msiSize = (Get-Item $msiPath).Length / 1MB
    Write-Host "  MSI installer found: $([math]::Round($msiSize, 2)) MB" -ForegroundColor Green
} else {
    Write-Host "  WARNING: MSI installer not found at $msiPath" -ForegroundColor Red
}

if (Test-Path $nsisPath) {
    $nsisSize = (Get-Item $nsisPath).Length / 1MB
    Write-Host "  NSIS installer found: $([math]::Round($nsisSize, 2)) MB" -ForegroundColor Green
} else {
    Write-Host "  WARNING: NSIS installer not found at $nsisPath" -ForegroundColor Red
}
Write-Host ""

# Step 7: Create release package
Write-Host "[7/7] Creating release package..." -ForegroundColor Yellow
$releaseDir = "release-v1.3.0"
if (Test-Path $releaseDir) {
    Remove-Item -Path $releaseDir -Recurse -Force
}
New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null

if (Test-Path $msiPath) {
    Copy-Item $msiPath $releaseDir
    Write-Host "  Copied MSI installer" -ForegroundColor Green
}

if (Test-Path $nsisPath) {
    Copy-Item $nsisPath $releaseDir
    Write-Host "  Copied NSIS installer" -ForegroundColor Green
}

Copy-Item "RELEASE_NOTES.md" $releaseDir
Write-Host "  Copied RELEASE_NOTES.md" -ForegroundColor Green
Write-Host ""

# Summary
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "BUILD COMPLETE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Release files created in: $releaseDir" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test the installers on a clean Windows machine" -ForegroundColor White
Write-Host "  2. Verify setup wizard works correctly" -ForegroundColor White
Write-Host "  3. Test Ollama integration" -ForegroundColor White
Write-Host "  4. Test staging workflow" -ForegroundColor White
Write-Host "  5. Create GitHub release and upload installers" -ForegroundColor White
Write-Host ""
Write-Host "Git commands to tag and push:" -ForegroundColor Yellow
Write-Host "  git add ." -ForegroundColor Gray
Write-Host "  git commit -m 'Release v1.3.0: Ollama setup and staging workflow'" -ForegroundColor Gray
Write-Host "  git tag v1.3.0" -ForegroundColor Gray
Write-Host "  git push origin main" -ForegroundColor Gray
Write-Host "  git push origin v1.3.0" -ForegroundColor Gray
Write-Host ""
