param(
    [string]$WslUser = $env:USERNAME,
    [string]$DriverRoot = $null,
    [string]$LibraryRoot = $null
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $DriverRoot -or [string]::IsNullOrWhiteSpace($DriverRoot)) {
    $DriverRoot = "/home/$WslUser/amlp-driver"
}
if (-not $LibraryRoot -or [string]::IsNullOrWhiteSpace($LibraryRoot)) {
    $LibraryRoot = "/home/$WslUser/amlp-library"
}

function Invoke-WslCommand {
    param([string]$Command)
    Write-Host "\n>>> wsl.exe -e bash -lc \"$Command\"" -ForegroundColor Yellow
    wsl.exe -e bash -lc "$Command"
}

Write-Host "Using DriverRoot=$DriverRoot" -ForegroundColor Cyan
Write-Host "Using LibraryRoot=$LibraryRoot" -ForegroundColor Cyan

# 1) List driver and test files
Invoke-WslCommand "ls -la $DriverRoot/build && ls -la $DriverRoot/tests/lpc"

# 2) Compile with verbose logging
Invoke-WslCommand "cd $DriverRoot && ./build/driver compile $DriverRoot/tests/lpc/simple.c -v"

# 3) AST with verbose logging
Invoke-WslCommand "cd $DriverRoot && ./build/driver ast $DriverRoot/tests/lpc/simple.c -v"

# 4) Bytecode with verbose logging
Invoke-WslCommand "cd $DriverRoot && ./build/driver bytecode $DriverRoot/tests/lpc/simple.c -v"

# 5) Run with combined stdout/stderr captured to /tmp/driver_run.log
Invoke-WslCommand "cd $DriverRoot && ./build/driver run $DriverRoot/tests/lpc/simple.c -v > /tmp/driver_run.log 2>&1; cat /tmp/driver_run.log"

# 6) Check for runtime files (efuns/simul_efuns/master)
Invoke-WslCommand "ls -la $LibraryRoot || true"
Invoke-WslCommand "find $LibraryRoot -maxdepth 2 -type f \( -name 'simul_efun*' -o -name 'master.c' -o -name 'std*.c' \) | head -n 40"
