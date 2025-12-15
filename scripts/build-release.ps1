# Build Release Package for Whisper Transcribe
# This script builds the release executable and creates a distributable package

param(
    [string]$Version = "0.1.0"
)

Write-Host "Building Whisper Transcribe v$Version..." -ForegroundColor Cyan

# Clean previous builds
Write-Host "`nCleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Build release version
Write-Host "`nBuilding release executable (this may take several minutes)..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    exit 1
}

# Create release directory
$releaseDir = "release\whisper-transcribe-v$Version"
Write-Host "`nCreating release package at: $releaseDir" -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null

# Copy executable
Write-Host "Copying executable..." -ForegroundColor Yellow
Copy-Item "target\release\whisper-transcribe.exe" "$releaseDir\"

# Copy documentation
Write-Host "Copying documentation..." -ForegroundColor Yellow
Copy-Item "README.md" "$releaseDir\"
Copy-Item "LICENSE" "$releaseDir\"
Copy-Item "RELEASE_NOTES.md" "$releaseDir\"

# Copy CUDA DLLs for portable execution
Write-Host "Copying CUDA DLLs..." -ForegroundColor Yellow
$cudaDlls = @("cudart64_13.dll", "cublas64_13.dll", "cublasLt64_13.dll")
$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\x64"
$copiedDlls = 0
foreach ($dll in $cudaDlls) {
    $source = Join-Path $cudaPath $dll
    if (Test-Path $source) {
        Copy-Item $source $releaseDir -Force
        $copiedDlls++
    } else {
        Write-Host "  Warning: $dll not found" -ForegroundColor Yellow
    }
}
if ($copiedDlls -eq $cudaDlls.Count) {
    Write-Host "  Copied $copiedDlls CUDA DLLs for portable GPU support" -ForegroundColor Green
} else {
    Write-Host "  Copied $copiedDlls/$($cudaDlls.Count) CUDA DLLs - app will need CUDA in PATH" -ForegroundColor Yellow
}

# Get file size
$exeSize = (Get-Item "$releaseDir\whisper-transcribe.exe").Length / 1MB
Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host "Executable size: $($exeSize.ToString('0.00')) MB" -ForegroundColor Green
Write-Host "Release package: $releaseDir" -ForegroundColor Green

# Create ZIP archive
Write-Host "`nCreating ZIP archive..." -ForegroundColor Yellow
$zipPath = "release\whisper-transcribe-v$Version-windows-x64.zip"
if (Test-Path $zipPath) {
    Remove-Item $zipPath
}
Compress-Archive -Path "$releaseDir\*" -DestinationPath $zipPath

$zipSize = (Get-Item $zipPath).Length / 1MB
Write-Host "ZIP archive created: $zipPath" -ForegroundColor Green
Write-Host "Archive size: $($zipSize.ToString('0.00')) MB" -ForegroundColor Green

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Release v$Version is ready!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`nPackage contents:"
Get-ChildItem $releaseDir | ForEach-Object {
    $size = if ($_.PSIsContainer) { "DIR" } else { "$($_.Length / 1KB) KB" }
    Write-Host "  - $($_.Name) ($size)"
}

Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Test the executable in $releaseDir"
Write-Host "2. Create a GitHub release and upload $zipPath"
Write-Host "3. Tag the release: git tag v$Version"
Write-Host "4. Push tags: git push --tags"
