# Build whisper-transcribe with CUDA on Windows.
# Usage (from repo root):
#   pwsh -File .\scripts\build-cuda.ps1

$ErrorActionPreference = 'Stop'

# Ensure CUDA env vars are available in this session (helps after fresh terminal restart).
$cudaPath = [Environment]::GetEnvironmentVariable('CUDA_PATH', 'Machine')
if (-not $cudaPath) {
  throw 'CUDA_PATH (Machine) is not set. Install NVIDIA CUDA Toolkit first.'
}
$cudaBin = Join-Path $cudaPath 'bin'
if (-not (Test-Path (Join-Path $cudaBin 'nvcc.exe'))) {
  throw "nvcc.exe not found at: $cudaBin"
}

# Ensure nvcc is discoverable in this session (PATH might require a new terminal otherwise).
if (-not (($env:Path -split ';') | Where-Object { $_ -and ($_.TrimEnd('\\') -ieq $cudaBin.TrimEnd('\\')) })) {
  $env:Path = "$cudaBin;$env:Path"
}
$env:CUDA_PATH = $cudaPath

# Build Tools Developer environment
$vsDevCmd = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat'
if (-not (Test-Path $vsDevCmd)) {
  throw "VsDevCmd.bat not found at: $vsDevCmd (is VS Build Tools 2022 installed?)"
}

Write-Host "CUDA_PATH=$env:CUDA_PATH"
Write-Host "Using VS Dev Cmd: $vsDevCmd"
Write-Host 'Building (release)...'

# Run build in a cmd.exe session so VsDevCmd.bat can set up MSVC env vars.
cmd /c "set CUDA_PATH=%CUDA_PATH% && \"$vsDevCmd\" -arch=amd64 -host_arch=amd64 && cargo build --release"
