# Whisper Transcribe v0.1.0 - Initial Release

Released: December 15, 2025

## Overview
First public release of Whisper Transcribe - a GPU-accelerated desktop application for local audio transcription using OpenAI's Whisper model.

## Features

### Core Functionality
- **Local Audio Transcription**: Transcribe audio files completely offline using Whisper models
- **GPU Acceleration**: CUDA support for NVIDIA GPUs with automatic CPU fallback
- **Modern GUI**: Clean, intuitive interface built with eframe/egui
- **Drag & Drop**: Easy file loading - just drag model and audio files into the window
- **Multi-Format Support**: WAV, MP3, FLAC, OGG, M4A, AAC, WMA, Opus, WebM
- **Export Options**: Copy transcriptions to clipboard or save as text files

### Technical Highlights
- Automatic audio resampling to 16kHz mono (Whisper's required format)
- Thread-based processing to keep UI responsive during transcription
- Graceful error handling with user-friendly error messages
- Efficient memory management for large audio files
- **No console window**: Runs as a native Windows GUI application without spawning a terminal

### Installer Features
- **Bundled CUDA DLLs**: GPU acceleration works out-of-the-box without installing CUDA Toolkit
- CUDA detection with helpful setup guidance
- Desktop shortcut option
- Clean uninstaller

## System Requirements

### Minimum
- Windows 10/11 (64-bit)
- 4 GB RAM
- 500 MB disk space (excluding models)

### Recommended for GPU
- NVIDIA GPU with CUDA compute capability 3.5+ (GTX 700 series or newer)
- 8 GB RAM
- CUDA Toolkit 13.0
- Latest NVIDIA GPU drivers

### CPU-Only Mode
- Works on any x64 Windows system
- Slower transcription but no GPU required

## Installation

### Using the Installer (Recommended)
1. Download `whisper-transcribe-v0.1.0-setup.exe`
2. Run the installer
3. Launch from Start Menu or Desktop shortcut

The installer bundles the required CUDA DLLs, so GPU acceleration works immediately on systems with NVIDIA GPUs - no need to install CUDA Toolkit separately.

### Manual Installation (ZIP)
1. Download and extract the release package
2. Run `whisper-transcribe.exe`

For GPU acceleration with the ZIP package, you'll need CUDA DLLs in PATH:
```powershell
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\x64"
[Environment]::SetEnvironmentVariable("PATH", [Environment]::GetEnvironmentVariable("PATH", "User") + ";$cudaPath", "User")
```

### CPU-Only
The application automatically falls back to CPU if no GPU is detected - no additional setup needed.

## Getting Models
Download Whisper GGML models from:
https://huggingface.co/ggerganov/whisper.cpp/tree/main

**Recommended models:**
- `ggml-base.bin` (142 MB) - Good balance of speed/accuracy
- `ggml-small.bin` (466 MB) - Better accuracy
- `ggml-medium.bin` (1.5 GB) - High accuracy

## Usage
1. Launch `whisper-transcribe.exe`
2. Load a Whisper model (.bin file)
3. Load an audio file
4. Click "Transcribe"
5. Copy or save the transcription

## Known Issues
- GPU initialization may fail on some systems (app automatically falls back to CPU)
- Large models (medium/large) require significant memory (8+ GB recommended)
- Very long audio files (>1 hour) may cause high memory usage
- Windows Defender may need to scan the executable on first run

## Troubleshooting

### Application won't start (Error -1073741515)
**Solution**: CUDA DLLs not in PATH. Follow installation step 2 above, then restart your terminal.

### "Failed to load Whisper model"
**Solution**: Ensure you're using a GGML `.bin` model file, not PyTorch `.pt` files.

### Transcription is very slow
**Solutions**:
- Use a smaller model (tiny/base instead of medium/large)
- Verify GPU is being used (check Task Manager or run `nvidia-smi`)
- Ensure CUDA DLLs are properly installed

## Building from Source
```powershell
# Clone repository
git clone <repo-url>
cd whisper-transcribe

# Build with GPU support (requires CUDA Toolkit)
cargo build --release

# Or build CPU-only (no CUDA needed)
# Edit Cargo.toml: whisper-rs = "0.15" (remove features = ["cuda"])
cargo build --release
```

## What's Next (v0.2.0 Planning)
- Model download manager
- Batch processing for multiple files
- Language selection
- Timestamp display
- Export to SRT/VTT subtitle formats
- Configuration file for default settings
- macOS and Linux support

## Credits
- Built with [whisper-rs](https://github.com/tazz4843/whisper-rs)
- Based on [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- OpenAI Whisper model
- GUI powered by [egui](https://github.com/emilk/egui)

## License
See LICENSE file for details.

## Support
For issues, bugs, or feature requests, please open an issue on GitHub.

---

Thank you for using Whisper Transcribe!
