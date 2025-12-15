# Whisper Transcribe v0.1.0

> **GPU-accelerated desktop transcription with OpenAI's Whisper - completely offline**

## What's New

First public release! Whisper Transcribe brings fast, GPU-accelerated audio transcription to your desktop with a clean, modern interface.

### Key Features
- **GPU Acceleration** - CUDA support for NVIDIA GPUs with automatic CPU fallback
- **Modern GUI** - Clean egui interface with drag & drop support
- **No Console Window** - Runs as a native Windows GUI application
- **Universal Format Support** - WAV, MP3, FLAC, OGG, M4A, AAC, WMA, Opus, WebM
- **100% Local** - All processing happens on your machine, no cloud required
- **Easy Export** - Copy to clipboard or save as text file
- **Bundled CUDA DLLs** - GPU acceleration works out-of-the-box

## Download

**Windows 10/11 (64-bit):**
- [whisper-transcribe-v0.1.0-setup.exe](https://github.com/papacasper/whisper-transcribe/releases/download/v0.1.0/whisper-transcribe-v0.1.0-setup.exe) - Installer with bundled CUDA DLLs

**Installer Features:**
- Bundled CUDA DLLs for GPU acceleration (no CUDA Toolkit needed)
- Automatic CUDA detection and setup guidance
- Start Menu shortcuts and desktop icon (optional)
- Clean uninstall support

## Quick Start

### Installation
1. Download the installer above
2. Run the installer (follow the setup wizard)
3. Launch from Start Menu or desktop

### First Use
1. Download a Whisper model from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp/tree/main) (recommend `ggml-base.bin`)
2. In the app, click **Browse** next to "Model" and select your `.bin` file
3. Click **Browse** next to "Audio" and select an audio file
4. Click **Transcribe**
5. Copy or save your transcription

## Recommended Models

Download from: https://huggingface.co/ggerganov/whisper.cpp/tree/main

| Model | Size | Speed | Accuracy | Recommended For |
|-------|------|-------|----------|-----------------|
| ggml-tiny.bin | 75 MB | Fast | Good | Testing |
| **ggml-base.bin** | **142 MB** | **Fast** | **Better** | **Most users** |
| ggml-small.bin | 466 MB | Medium | High | Better accuracy |
| ggml-medium.bin | 1.5 GB | Slow | Very High | High accuracy |
| ggml-large-v3.bin | 2.9 GB | Slowest | Best | Best quality |

## System Requirements

**Minimum (CPU-only):**
- Windows 10/11 64-bit
- 4 GB RAM
- 500 MB free disk space

**Recommended (GPU):**
- NVIDIA GPU (GTX 700 series or newer)
- 8 GB RAM
- Latest NVIDIA drivers

## Building from Source

```powershell
git clone https://github.com/papacasper/whisper-transcribe.git
cd whisper-transcribe
cargo build --release
```

For CPU-only build, edit `Cargo.toml` and remove `features = ["cuda"]` from whisper-rs dependency.

## Known Issues

- GPU initialization may fail on some systems (auto-falls back to CPU)
- Large models require significant memory (8+ GB recommended)
- Very long audio (>1 hour) may cause high memory usage

## Documentation

- [README.md](README.md) - Complete documentation
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - Detailed release notes

## Credits

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Rust bindings
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - C++ implementation
- [egui](https://github.com/emilk/egui) - GUI framework
- OpenAI Whisper model

## What's Next?

Planned for v0.2.0:
- Model download manager
- Batch file processing
- Language selection
- Timestamp display
- SRT/VTT subtitle export
- macOS and Linux support
