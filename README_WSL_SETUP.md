# WSL Setup Guide for LPC Dev Assistant

## 1) Check WSL availability
- In PowerShell:
  ```powershell
  wsl.exe --status
  ```
- Expected: reports default distro. If missing, install:
  ```powershell
  wsl --install
  ```
  Then reboot and run `wsl.exe --status` again.

## 2) Required paths
- Driver root (default): `/home/<user>/amlp-driver`
- Library root (default): `/home/<user>/amlp-library`
- Binary path: `/home/<user>/amlp-driver/build/driver`

Create directories if missing:
```bash
wsl.exe -e bash -lc "mkdir -p /home/<user>/amlp-driver /home/<user>/amlp-library"
```

## 3) Obtain/build the driver
- If you have source checked out in WSL:
  ```bash
  wsl.exe -e bash -lc "cd /home/<user>/amlp-driver && make"  # or `cargo build --release`
  ```
- Verify binary exists and is executable:
  ```bash
  wsl.exe -e bash -lc "test -x /home/<user>/amlp-driver/build/driver && echo OK"
  ```

## 4) Library expectations
- Place runtime/library files under `/home/<user>/amlp-library` (e.g., `master.c`, `simul_efun.c`).
- Create the library directory if absent:
  ```bash
  wsl.exe -e bash -lc "mkdir -p /home/<user>/amlp-library"
  ```

## 5) Manual smoke tests
- List driver and tests:
  ```bash
  wsl.exe -e bash -lc "ls -la /home/<user>/amlp-driver/build && ls -la /home/<user>/amlp-driver/tests/lpc"
  ```
- Compile sample:
  ```bash
  wsl.exe -e bash -lc "cd /home/<user>/amlp-driver && ./build/driver compile /home/<user>/amlp-driver/tests/lpc/simple.c -v"
  ```
- Run sample (if runtime files exist):
  ```bash
  wsl.exe -e bash -lc "cd /home/<user>/amlp-driver && ./build/driver run /home/<user>/amlp-driver/tests/lpc/simple.c -v"
  ```

## 6) Using the app
- Open Settings tab → set WSL username/paths if different.
- Click "Run Validation" to confirm WSL, driver dir, driver binary, and library dir.
- If a component is missing, use the suggested fix command shown in the app.
