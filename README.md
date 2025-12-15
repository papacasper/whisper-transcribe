# Whisper Transcribe

A fast, GPU-accelerated desktop application for transcribing audio files locally using OpenAI's Whisper model via `whisper-rs`.

## Features

- **GPU Acceleration**: CUDA support for NVIDIA GPUs with automatic CPU fallback
- **Modern GUI**: Built with `eframe`/`egui` for a clean, responsive interface
- **No Console Window**: Runs as a native Windows GUI application
- **Universal Audio Support**: Handles WAV, MP3, FLAC, OGG, M4A, AAC, WMA, Opus, and WebM formats via `symphonia`
- **Drag & Drop**: Simply drag model and audio files into the application window
- **Local Processing**: All transcription happens on your machine - no cloud services required
- **Export Options**: Copy transcriptions to clipboard or save as text files
- **Easy Installation**: Installer bundles CUDA DLLs for out-of-the-box GPU support

## Quick Start (End Users)

### Using the Installer (Recommended)
1. Download `whisper-transcribe-v0.1.0-setup.exe` from the [Releases](https://github.com/papacasper/whisper-transcribe/releases) page
2. Run the installer - CUDA DLLs are bundled for GPU acceleration
3. Launch from Start Menu or Desktop
4. Download a Whisper model when prompted, or manually from [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp/tree/main)

### Portable (ZIP)
1. Download and extract the ZIP release
2. Run `whisper-transcribe.exe`
3. For GPU acceleration, ensure CUDA DLLs are in PATH (see Troubleshooting)

## Development Requirements

### Essential
- Rust toolchain (1.70+)
- A Whisper GGML model file (`.bin` format)
- Windows 10/11 (primary target platform)

### For GPU Acceleration (Optional)
- NVIDIA GPU with CUDA compute capability 3.5+
- NVIDIA GPU drivers (latest recommended)
- CUDA Toolkit 13.0 or compatible version

## Installation

### 1. Install Rust
```pwsh
winget install Rustlang.Rust.MSVC
```

### 2. Install CUDA Toolkit (for GPU acceleration)
```pwsh
winget install Nvidia.CUDA --version 13.0
```

**Important**: After installation, you may need to add the CUDA DLL directory to your PATH:
```pwsh
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\x64"
[Environment]::SetEnvironmentVariable("PATH", [Environment]::GetEnvironmentVariable("PATH", "User") + ";$cudaPath", "User")
```

Then restart your terminal or reboot for the PATH changes to take effect.

### 3. Clone and Build
```pwsh
git clone https://github.com/papacasper/whisper-transcribe.git
cd whisper-transcribe
cargo build --release
```

The executable will be at: `target/release/whisper-transcribe.exe`

## Development

### Run in Debug Mode
```pwsh
cargo run
```

### Build Release Version
```pwsh
cargo build --release
```

### Build Without GPU Support
If you don't have an NVIDIA GPU or CUDA installed, edit `Cargo.toml` and change:
```toml
whisper-rs = { version = "0.15", features = ["cuda"] }
```
to:
```toml
whisper-rs = "0.15"
```

## GPU Acceleration

This application is built with CUDA support enabled by default. The application will:
1. **First attempt**: Use your NVIDIA GPU via CUDA for faster transcription
2. **Fallback**: Automatically switch to CPU if GPU initialization fails

### System Requirements for GPU
- **GPU**: NVIDIA GPU with CUDA compute capability 3.5 or higher (e.g., GTX 700 series or newer)
- **Drivers**: Latest NVIDIA GPU drivers
- **CUDA**: CUDA Toolkit 13.0 (installed via the instructions above)
- **Build Tools**: Visual Studio 2019/2022 with C++ build tools (MSVC)

### Verifying GPU Support
Check if your GPU is detected:
```pwsh
nvidia-smi
```

You should see your GPU listed with CUDA Version 13.0 or compatible.

## Usage Guide

### Getting Whisper Models
This application requires Whisper GGML model files (`.bin` format). Download models from the [Hugging Face whisper.cpp repository](https://huggingface.co/ggerganov/whisper.cpp/tree/main).

**Recommended models:**
- `ggml-tiny.bin` (~75 MB) - Fastest, good for testing
- `ggml-base.bin` (~142 MB) - Good balance of speed and accuracy
- `ggml-small.bin` (~466 MB) - Better accuracy, moderate speed
- `ggml-medium.bin` (~1.5 GB) - High accuracy, slower
- `ggml-large-v3.bin` (~2.9 GB) - Best accuracy, slowest

### Running the Application

1. **Launch**: Run `whisper-transcribe.exe` or use `cargo run`
2. **Load Model**: Click **Browse...** next to "Model:" and select your `.bin` model file (or drag & drop)
3. **Load Audio**: Click **Browse...** next to "Audio:" and select your audio file (or drag & drop)
4. **Transcribe**: Click the **Transcribe** button
5. **Wait**: The app will show a spinner while processing ("Loading..." → "Transcribing...")
6. **Export**: 
   - Click **Copy** to copy the transcription to clipboard
   - Click **Save...** to save as a text file
   - Click **Clear** to reset and start over

### Supported Audio Formats
The following formats are automatically decoded:
- **Lossless**: WAV, FLAC
- **Compressed**: MP3, OGG Vorbis, Opus, AAC, M4A
- **Other**: WMA, WebM

All audio is automatically converted to mono 16kHz (Whisper's required format).

## Troubleshooting

### Application Won't Launch

Exit code -1073741515 (0xC0000135): Missing DLL dependencies
- Solution: Add CUDA DLLs to PATH (see Installation step 2)
- Verify CUDA installation: Run `nvidia-smi` to check CUDA is installed and the driver is working
- The CUDA DLLs should be in: `C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\x64\`

Application crashes immediately:
- Try building without CUDA (see "Build Without GPU Support")
- Ensure you have the latest NVIDIA drivers
- Check Windows Event Viewer for details

### Build Errors

"Failed to compile whisper-rs-sys":
- Install Visual Studio Build Tools with C++ support
- Ensure CUDA_PATH is set: `echo $env:CUDA_PATH`
- Clean and rebuild: `cargo clean && cargo build --release`

"LINK : fatal error LNK1181: cannot open input file 'cuda.lib'":
- CUDA Toolkit installation is incomplete
- Reinstall CUDA Toolkit: `winget uninstall Nvidia.CUDA && winget install Nvidia.CUDA --version 13.0`

### Transcription Issues

"Failed to load Whisper model":
- Use a GGML `.bin` model (not PyTorch `.pt`)
- Download from the official whisper.cpp repo
- Re-download if corrupted

"Unsupported audio format":
- Convert to WAV/MP3/FLAC
- FFmpeg example: `ffmpeg -i input.file -ar 16000 -ac 1 output.wav`

"Error: No audio track found":
- File may be corrupt or contain no audio
- Re-encode: `ffmpeg -i input.mp4 -vn -acodec libmp3lame output.mp3`

### Performance Issues

Slow transcription:
- Use a smaller model (`tiny`, `base`, or `small`)
- Build in release mode
- Verify GPU usage (`nvidia-smi`) or Task Manager
- Close other GPU-heavy apps

High memory usage:
- Use a smaller model
- Split very long audio files
- Close other apps

GPU not used:
- Confirm CUDA DLLs in PATH
- App falls back to CPU if GPU init fails

## Technical Details

### Architecture
- Frontend: eframe/egui (Rust GUI)
- Audio: symphonia (decode) + rubato (resample)
- ML: whisper-rs → whisper.cpp (C++)
- GPU: CUDA via cuBLAS

### Dependencies
- whisper-rs v0.15
- eframe v0.30
- symphonia v0.5
- rubato v0.16
- rfd v0.15
- arboard v3
- anyhow v1.0

### Build Process (CUDA)
- whisper-rs-sys builds whisper.cpp with `GGML_CUDA=ON`
- Links against CUDA libs; runtime requires: `cudart64_13.dll`, `cublas64_13.dll`, `cublasLt64_13.dll`

## Contributing
- Fork, create a branch, make changes, test on Windows, then open a PR

## License

MIT License

Copyright (c) 2024 Casper

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
