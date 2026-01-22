# LPC Dev Assistant - Release Instructions

## For Maintainers: Creating GitHub Releases

### Step 1: Build All Platforms
```powershell
# On Windows
.\build-all-platforms.ps1 -Platform windows

# On macOS (for macOS/Intel builds)
./build-all-platforms.ps1 -Platform macos

# On Linux (for Linux builds)
./build-all-platforms.ps1 -Platform linux
```

### Step 2: Locate Built Installers

**Windows:**
```
target/release/bundle/msi/LPC Dev Assistant_0.1.0_x64_en-US.msi
target/release/bundle/nsis/LPC Dev Assistant_0.1.0_x64-setup.exe
```

**macOS:**
```
target/release/bundle/macos/LPC Dev Assistant.dmg
target/release/bundle/macos/LPC_Dev_Assistant_*.app.tar.gz
```

**Linux:**
```
target/release/bundle/deb/lpc-dev-assistant_*.deb
target/release/bundle/appimage/LPC_Dev_Assistant_*_x64.AppImage
```

### Step 3: Create GitHub Release

1. Go to [GitHub Releases](https://github.com/Thurtea/lpc-development-assistant/releases)
2. Click "Draft a new release"
3. Tag: `v1.2.0` (or current version from `tauri.conf.json`)
4. Title: "LPC Dev Assistant v1.2.0"
5. Description:
   ```markdown
   ## Changes
   - [List your changes here]
   
   ## Installation
   
   ### Windows
   Download `LPC Dev Assistant_0.1.0_x64_en-US.msi` and run it.
   
   ### macOS
   Download `.dmg`, drag to Applications.
   
   ### Linux
   Download `.deb` (Debian/Ubuntu) or `.AppImage` (universal).
   ```

6. **Upload Files:**
   - ✅ `LPC Dev Assistant_0.1.0_x64_en-US.msi` (Windows MSI)
   - ✅ `LPC Dev Assistant_0.1.0_x64-setup.exe` (Windows NSIS)
   - ✅ `LPC Dev Assistant.dmg` (macOS)
   - ✅ `lpc-dev-assistant_*.deb` (Linux DEB)
   - ✅ `LPC_Dev_Assistant_*_x64.AppImage` (Linux AppImage)

7. Click "Publish release"

### Step 4: Update Version

When releasing a new version:
1. Update `tauri.conf.json` version: `"version": "1.3.0"`
2. Build all platforms
3. Create GitHub Release with tag matching version
4. Assets will automatically have new version in filenames

## For End Users: Installation

### Option A: Pre-Built (Recommended)
1. Go to [Releases page](https://github.com/Thurtea/lpc-development-assistant/releases)
2. Download installer for your OS
3. Run installer
4. Launch app

### Option B: Build from Source
1. Clone repo: `git clone https://github.com/Thurtea/lpc-development-assistant.git`
2. Install Rust and Ollama
3. Run: `cargo tauri build`
4. Installers in `target/release/bundle/`

## Version Numbers

- Config version in `tauri.conf.json`: Controls installer version
- GitHub Release tag: Marks the release point
- Keep them in sync!

Example:
```json
{
  "version": "1.2.0"  // in tauri.conf.json
}
```
→ Build → Creates `LPC Dev Assistant_0.1.0_x64_en-US.msi`
→ GitHub Release tag: `v1.2.0`

