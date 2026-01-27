# LPC Development Assistant v1.3.0

**Release Date**: January 27, 2026

## New Features

### Ollama Setup Wizard
- Complete configuration interface for managing Ollama
- Check installation and running status with visual indicators
- Start Ollama server directly from the application
- Pull models from dropdown selection (qwen2.5-coder:7b, glm-4.7-flash, glm-4.7:cloud)
- View list of all installed models
- Save Ollama configuration preferences

### File Staging Workflow
- New Staging tab for safe code review before deployment
- Generated code saves to staging directory first (~/.lpc-dev-assistant/staging/)
- Preview file contents with target project badges (Driver/Library)
- Copy individual files to AMLP projects after review
- Clear staging directory with confirmation prompt
- Automatic staging initialization on first use

### Project Path Configuration
- Centralized management of AMLP driver and library paths
- Configure WSL paths for driver and library roots
- Test path connectivity directly from UI
- Display staging directory location
- Persistent configuration storage

### Safety Features
- Automatic file backups with timestamps (filename.c.bak-YYYYMMDD-HHMMSS)
- Review-before-deploy workflow prevents accidental overwrites
- Parent directory creation for new files
- WSL integration for safe cross-platform file operations

## Improvements

- Removed all non-ASCII characters from UI for better compatibility
- Enhanced button labels ("To Driver" instead of arrow symbols)
- Reorganized Settings tab with logical sections
- Extended configuration structure for future expansion
- Improved error handling and user feedback

## Installation

### Windows 10/11 (64-bit)

1. **Download the installer:**
   - MSI installer: LPC_Dev_Assistant_1.3.0_x64_en-US.msi
   - NSIS installer: LPC_Dev_Assistant_1.3.0_x64-setup.exe

2. **Run the installer:**
   - Double-click the downloaded file
   - Follow the installation wizard
   - Choose installation location (default recommended)

3. **First Launch:**
   - The app will check for Ollama installation
   - Configure Ollama settings in the Settings tab
   - Set up project paths for your AMLP driver and library
   - Start using the AI assistant!

### System Requirements

#### Required
- Windows 10 or Windows 11 (64-bit)
- 500 MB free disk space
- Internet connection (for downloading Ollama models)

#### Optional but Recommended
- Ollama installed (download from https://ollama.ai)
- Windows Subsystem for Linux (WSL) 2 with Ubuntu (for AMLP driver integration)
- At least one Ollama model installed (qwen2.5-coder:7b recommended)

## Upgrade from v1.2.0

Your existing configuration will be preserved and automatically upgraded. New fields will be added with sensible defaults:

- Ollama model: qwen2.5-coder:7b
- Ollama provider: local
- Staging directory: Will be created on first use
- Setup complete flag: Will be set after initial configuration

No manual migration is required.

## Quick Start Guide

### First-Time Setup

1. **Install Ollama** (if not already installed):
   - Go to Settings tab
   - Click "Download Ollama" if status shows not installed
   - Install Ollama from the official website
   - Return to app and click "Refresh Status"

2. **Start Ollama Server**:
   - In Settings tab, click "Start Ollama Server"
   - Wait for status to show "running"

3. **Pull a Model**:
   - Select model from dropdown (qwen2.5-coder:7b recommended)
   - Click "Pull Selected Model"
   - Wait for download to complete (may take several minutes)

4. **Configure Project Paths** (optional, for WSL integration):
   - Enter AMLP Driver Root path (e.g., /home/username/amlp-driver)
   - Enter AMLP Library Root path (e.g., /home/username/amlp-library)
   - Click "Test" buttons to verify connectivity
   - Click "Save Project Paths"

### Using the Staging Workflow

1. **Generate Code**:
   - Go to Assistant tab
   - Enter your LPC development question
   - Click "Ask" or press Ctrl+Enter
   - Review the generated code

2. **Save to Staging**:
   - Click "To Driver" or "To Library" button
   - Code is saved to staging directory
   - App automatically switches to Staging tab

3. **Review and Deploy**:
   - Review the staged file in Staging tab
   - Click "Copy to Driver/Library" when ready
   - Original files are backed up automatically
   - Click "Refresh" to update the list

4. **Clear When Done**:
   - Click "Clear All" to remove staged files
   - Confirm the action when prompted

## Known Issues

- None reported at this time

## Getting Help

- **Documentation**: See README.md in installation directory
- **WSL Setup Guide**: See README_WSL_SETUP.md for detailed WSL configuration
- **Issues**: Report bugs on GitHub Issues page
- **Questions**: Check existing documentation or create a GitHub issue

## Credits

Developed with Rust, Tauri, and Ollama for the LPC MUD development community.

## License

MIT License - See LICENSE file for details
