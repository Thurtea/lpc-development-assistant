# Start both Vite (port 1420) and Tauri dev in separate PowerShell windows
$root = Split-Path -Parent $MyInvocation.MyCommand.Definition
Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location 'E:\\Work\\AMLP\\lpc-dev-assistant'; npx vite --port 1420 --host 0.0.0.0"
Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location 'E:\\Work\\AMLP\\lpc-dev-assistant'; cargo tauri dev --features with_tauri"
