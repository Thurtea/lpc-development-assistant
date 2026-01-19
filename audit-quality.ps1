# PowerShell audit script for lpc-development-assistant
# Saves outputs to audit-results/ with timestamped filenames

$ErrorActionPreference = 'Continue'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$resultsDir = Join-Path $scriptDir "audit-results"
if (-Not (Test-Path $resultsDir)) { New-Item -ItemType Directory -Path $resultsDir | Out-Null }

function Check-Tool($name, $exe) {
    if (-Not (Get-Command $exe -ErrorAction SilentlyContinue)) {
        Write-Warning "$name ($exe) not found. Some checks will be skipped or fail. Install it to get full results."
        return $false
    }
    Write-Output "$name found: $exe"
    return $true
}

# Check required tools
$cargoAuditPresent = Check-Tool "cargo-audit" "cargo-audit"
$cargoOutdatedPresent = Check-Tool "cargo-outdated" "cargo-outdated"
$clocPresent = Check-Tool "cloc" "cloc"

# Helper to run a command and tee output to file with try/catch
function Run-Command($cmd, $outfile) {
    Write-Output "\n=== Running: $cmd" | Tee-Object -FilePath $outfile -Append
    try {
        # Use Invoke-Expression so we can include pipes and redirects
        Invoke-Expression "$cmd 2>&1 | Tee-Object -FilePath '$outfile' -Append"
        Write-Output "(finished)" | Tee-Object -FilePath $outfile -Append
    } catch {
        Write-Warning "Command failed: $cmd"
        $_ | Out-String | Tee-Object -FilePath $outfile -Append
    }
}

# 1. cargo clippy unwrap/expect warnings
$clippyOut = Join-Path $resultsDir "clippy-unwraps-$timestamp.log"
Run-Command "cargo clippy --all-targets -- -W clippy::unwrap_used -W clippy::expect_used" $clippyOut

# 2. cargo test --workspace
$testOut = Join-Path $resultsDir "cargo-test-$timestamp.log"
Run-Command "cargo test --workspace" $testOut

# 3. cargo audit (if present)
$auditOut = Join-Path $resultsDir "cargo-audit-$timestamp.json"
if ($cargoAuditPresent) {
    try { Invoke-Expression "cargo audit --json > '$auditOut'" } catch { Write-Warning "cargo audit failed" }
} else {
    '{"error":"cargo-audit not installed"}' | Out-File -FilePath $auditOut -Encoding utf8
}

# 4. cargo outdated
$outdatedOut = Join-Path $resultsDir "cargo-outdated-$timestamp.txt"
if ($cargoOutdatedPresent) {
    Run-Command "cargo outdated --depth=1 > '$outdatedOut'" $outdatedOut
} else {
    "cargo-outdated not installed" | Out-File -FilePath $outdatedOut -Encoding utf8
}

# 5. cloc for Rust files
$clocOut = Join-Path $resultsDir "cloc-src-$timestamp.txt"
if ($clocPresent) {
    Run-Command "cloc src/ src-tauri/ --by-file --include-lang=Rust > '$clocOut'" $clocOut
} else {
    "cloc not installed" | Out-File -FilePath $clocOut -Encoding utf8
}

# 6. npm audit/outdated if UI/package.json exists
$uiPath = Join-Path $scriptDir "UI"
$npmAuditOut = Join-Path $resultsDir "npm-audit-$timestamp.json"
$npmOutdatedOut = Join-Path $resultsDir "npm-outdated-$timestamp.json"
if (Test-Path (Join-Path $uiPath "package.json")) {
    Write-Output "UI/package.json found, running npm audit & npm outdated"
    Push-Location $uiPath
    try {
        Invoke-Expression "npm audit --json > '$npmAuditOut'" 2>$null
    } catch {
        Write-Warning "npm audit failed"
        '{"error":"npm audit failed"}' | Out-File -FilePath $npmAuditOut -Encoding utf8
    }
    try {
        Invoke-Expression "npm outdated --json > '$npmOutdatedOut'" 2>$null
    } catch {
        Write-Warning "npm outdated failed"
        '{"error":"npm outdated failed"}' | Out-File -FilePath $npmOutdatedOut -Encoding utf8
    }
    Pop-Location
} else {
    Write-Output "UI/package.json not found; skipping npm audit/outdated"
    "{}" | Out-File -FilePath $npmAuditOut -Encoding utf8
    "{}" | Out-File -FilePath $npmOutdatedOut -Encoding utf8
}

# Summary
Write-Output "\n=== Audit complete. Results saved in: $resultsDir\n"
Get-ChildItem -Path $resultsDir -File | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize

Write-Output "\nFiles created (full paths):"
Get-ChildItem -Path $resultsDir -File | ForEach-Object { $_.FullName }

Write-Output "\nIf you want me to analyze results, paste the contents of the log files or attach them."