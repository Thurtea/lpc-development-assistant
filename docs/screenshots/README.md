# Screenshots Guide

## Directory Structure
```
docs/screenshots/
├── README.md (this file)
├── app-main-window.png           # Main Assistant tab
├── app-driver-tab.png            # Driver integration tab
├── code-generation-example.png   # LPC code generation in action
├── compilation-success.png       # Successful compilation output
├── compilation-error.png         # Error handling display
├── settings-modal.png            # Generation settings
└── first-run-setup.png           # Setup wizard
```

## Naming Convention

**Format**: `{feature}-{state}-{variant}.png`

**Examples**:
- `app-main-window.png` - Main application window
- `driver-connection-success.png` - Connection test passed
- `driver-connection-error.png` - Connection test failed
- `compilation-success.png` - Successful LPC compilation
- `compilation-error.png` - Compilation with errors
- `code-generation-streaming.png` - AI generating code in real-time
- `settings-temperature-high.png` - Settings with high temperature
- `rag-search-results.png` - Reference search results

## Recommended Specifications

**Image Format**: PNG (lossless, good for UI screenshots)

**Dimensions**:
- **Full window**: 1200x800px (matches default window size)
- **Focused UI section**: 800x600px or smaller
- **Mobile/compact**: 600x400px

**File Size**:
- Target: < 500KB per image
- Use PNG optimization tools if needed (e.g., `pngquant`, `optipng`)

**DPI/Resolution**: 
- 96 DPI (standard for web display)
- For Retina displays, capture at 2x and scale down

## How to Capture

### Windows
1. Use Windows Snipping Tool (Win + Shift + S)
2. Or use dedicated tools: ShareX, Greenshot, Snagit

### macOS
1. Use built-in Screenshot (Cmd + Shift + 4)
2. For window capture: Cmd + Shift + 4, then Space

### Linux
1. Use Flameshot, Spectacle, or gnome-screenshot
2. Command line: `gnome-screenshot -w` (window only)

## Embedding in README.md

### Basic Embed
```markdown
![Application Main Window](docs/screenshots/app-main-window.png)
```

### With Alt Text and Title
```markdown
![LPC Development Assistant main window showing code generation in progress](docs/screenshots/code-generation-example.png "Code Generation Example")
```

### Side-by-Side Comparison
```markdown
<p align="center">
  <img src="docs/screenshots/compilation-success.png" width="45%" alt="Success" />
  <img src="docs/screenshots/compilation-error.png" width="45%" alt="Error Handling" />
</p>
```

### Feature Showcase Section
```markdown
## Features

### AI-Powered Code Generation
Generate LPC code with context-aware suggestions from your mudlib.

![Code generation with streaming responses](docs/screenshots/code-generation-streaming.png)

### Driver Integration
Compile and test your LPC code directly from the assistant.

![Driver tab showing compilation results](docs/screenshots/compilation-success.png)

### Smart Reference Search
Built-in RAG system searches your driver documentation and mudlib examples.

![Search results from reference corpus](docs/screenshots/rag-search-results.png)
```

## Tips for Good Screenshots

1. **Clean State**: 
   - Hide personal information
   - Use generic usernames/paths if visible
   - Clear browser dev tools if not relevant

2. **Representative Content**:
   - Show realistic LPC code examples
   - Use actual compilation output
   - Demonstrate real features, not placeholders

3. **Highlight Focus**:
   - Add subtle borders or arrows to draw attention
   - Crop to relevant UI section
   - Consider annotating complex screenshots

4. **Consistent Theme**:
   - Use same color scheme across all screenshots
   - Keep window size consistent
   - Same font size/zoom level

5. **Update Regularly**:
   - Re-capture when UI changes significantly
   - Mark outdated screenshots with version tags
   - Keep this README in sync with actual files

## Screenshot Checklist

Before adding to repository:
- [ ] No personal/sensitive information visible
- [ ] Image is optimized (< 500KB)
- [ ] Filename follows naming convention
- [ ] Alt text is descriptive and helpful
- [ ] README.md updated with new screenshot reference
- [ ] Screenshot shows latest UI version
