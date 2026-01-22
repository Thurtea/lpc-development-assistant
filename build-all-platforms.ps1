# LPC Dev Assistant - Multi-Platform Build Script
# Builds Windows, Linux, and macOS installers
# Prerequisites: Rust toolchain with targets installed

param(
    [ValidateSet('windows', 'linux', 'macos', 'all')]
    [string]$Platform = 'all',
    
    [switch]$NoClean
)

$ErrorActionPreference = 'Stop'
$timestamp = Get-Date -Format "yyyy-MM-dd_HHmmss"
$logDir = "./build-logs-$timestamp"
New-Item -ItemType Directory -Path $logDir -Force | Out-Null

Write-Host "LPC Dev Assistant - Multi-Platform Build" -ForegroundColor Cyan
Write-Host "Platform: $Platform" -ForegroundColor Cyan
Write-Host "Log directory: $logDir" -ForegroundColor Gray
Write-Host ""

# Function to run build command with logging
function Build-Target {
    param(
        [string]$Name,
        [string]$Command,
        [string]$LogFile
    )
    
    Write-Host "Building $Name..." -ForegroundColor Yellow
    Write-Host "Command: $Command" -ForegroundColor Gray
    
    $startTime = Get-Date
    try {
        & cmd /c "$Command 2>&1 | Tee-Object -FilePath $LogFile"
        $duration = (Get-Date) - $startTime
        Write-Host "✓ $Name build completed in $($duration.TotalMinutes.ToString('F2')) minutes" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "✗ $Name build failed!" -ForegroundColor Red
        Write-Host "See log: $LogFile" -ForegroundColor Gray
        return $false
    }
}

$results = @{}

# Windows Build
if ($Platform -eq 'windows' -or $Platform -eq 'all') {
    if (-not $NoClean) {
        Write-Host "Cleaning build artifacts..." -ForegroundColor Gray
        cargo clean
    }
    
    Write-Host ""
    $logFile = Join-Path $logDir "build-windows.log"
    $results['Windows'] = Build-Target -Name "Windows (x86_64)" `
        -Command "cargo tauri build --target x86_64-pc-windows-msvc" `
        -LogFile $logFile
    
    if ($results['Windows']) {
        $msiPath = "target\release\bundle\msi\LPC Dev Assistant_0.1.0_x64_en-US.msi"
        if (Test-Path $msiPath) {
            $size = (Get-Item $msiPath).Length / 1MB
            Write-Host "  MSI created: $size MB" -ForegroundColor Gray
        }
    }
    Write-Host ""
}

# Linux Build
if ($Platform -eq 'linux' -or $Platform -eq 'all') {
    Write-Host "Linux Build Notes:" -ForegroundColor Cyan
    Write-Host "  Requires: Linux dev tools or cross-compilation setup" -ForegroundColor Gray
    Write-Host "  Option 1: Build on Linux/WSL" -ForegroundColor Gray
    Write-Host "  Option 2: Use 'cross' tool for cross-compilation:" -ForegroundColor Gray
    Write-Host "    cargo install cross" -ForegroundColor Gray
    Write-Host "    cross tauri build --target x86_64-unknown-linux-gnu" -ForegroundColor Gray
    Write-Host ""
    
    $logFile = Join-Path $logDir "build-linux.log"
    Write-Host "Attempting Linux build (may fail on Windows without cross-tools)..." -ForegroundColor Yellow
    
    # Try building with cross tool if available
    $crossAvailable = $null -ne (Get-Command cross -ErrorAction SilentlyContinue)
    if ($crossAvailable) {
        Write-Host "  'cross' tool detected, proceeding..." -ForegroundColor Green
        $results['Linux'] = Build-Target -Name "Linux (x86_64)" `
            -Command "cross tauri build --target x86_64-unknown-linux-gnu" `
            -LogFile $logFile
    }
    else {
        Write-Host "  'cross' tool not found, skipping Linux build" -ForegroundColor Yellow
        Write-Host "  Install with: cargo install cross" -ForegroundColor Gray
        $results['Linux'] = $false
    }
    Write-Host ""
}

# macOS Build
if ($Platform -eq 'macos' -or $Platform -eq 'all') {
    Write-Host "macOS Build Notes:" -ForegroundColor Cyan
    Write-Host "  Requires: macOS system or Intel/Apple Silicon cross-tools" -ForegroundColor Gray
    Write-Host "  Best option: Build natively on macOS" -ForegroundColor Gray
    Write-Host ""
    
    $logFile = Join-Path $logDir "build-macos.log"
    Write-Host "Attempting macOS build (requires macOS or cross-tools)..." -ForegroundColor Yellow
    
    $osCheck = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)
    if ($osCheck) {
        # Intel Mac
        Write-Host "  Building for Intel (x86_64)..." -ForegroundColor Green
        $results['macOS-Intel'] = Build-Target -Name "macOS (x86_64)" `
            -Command "cargo tauri build --target x86_64-apple-darwin" `
            -LogFile "$logFile.intel"
        
        # Apple Silicon
        Write-Host "  Building for Apple Silicon (aarch64)..." -ForegroundColor Green
        $results['macOS-ARM'] = Build-Target -Name "macOS (aarch64)" `
            -Command "cargo tauri build --target aarch64-apple-darwin" `
            -LogFile "$logFile.arm"
    }
    else {
        Write-Host "  macOS build only available on macOS system" -ForegroundColor Yellow
        Write-Host "  Skipping macOS build on Windows" -ForegroundColor Gray
        $results['macOS'] = $false
    }
    Write-Host ""
}

# Summary
Write-Host "Build Summary" -ForegroundColor Cyan
Write-Host ("-" * 50)
foreach ($target in $results.Keys | Sort-Object) {
    $status = if ($results[$target]) { "✓ SUCCESS" } else { "✗ FAILED" }
    $color = if ($results[$target]) { "Green" } else { "Red" }
    Write-Host "$target`: $status" -ForegroundColor $color
}
Write-Host ("-" * 50)
Write-Host "Logs saved to: $logDir" -ForegroundColor Gray
Write-Host ""

# Output paths
Write-Host "Output Locations:" -ForegroundColor Cyan
if ($results['Windows']) {
    Write-Host "  Windows MSI: target\release\bundle\msi\LPC Dev Assistant_0.1.0_x64_en-US.msi" -ForegroundColor Green
    Write-Host "  Windows EXE: target\release\bundle\nsis\LPC Dev Assistant_0.1.0_x64-setup.exe" -ForegroundColor Green
}
if ($results['Linux']) {
    Write-Host "  Linux DEB: target\release\bundle\deb\lpc-dev-assistant_*_amd64.deb" -ForegroundColor Green
    Write-Host "  Linux AppImage: target\release\bundle\appimage\LPC_Dev_Assistant_*_x64.AppImage" -ForegroundColor Green
}
if ($results['macOS-Intel'] -or $results['macOS-ARM']) {
    Write-Host "  macOS DMG: target\release\bundle\macos\LPC_Dev_Assistant_*.dmg" -ForegroundColor Green
    Write-Host "  macOS APP: target\release\bundle\macos\LPC Dev Assistant.app" -ForegroundColor Green
}

$successCount = ($results.Values | Where-Object { $_ }).Count
$totalCount = $results.Count
Write-Host ""
Write-Host "Builds completed: $successCount/$totalCount" -ForegroundColor Cyan
