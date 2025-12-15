; Whisper Transcribe Installer Script for Inno Setup
; Builds a small installer that checks for CUDA and guides installation

#define MyAppName "Whisper Transcribe"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "papacasper"
#define MyAppURL "https://github.com/papacasper/whisper-transcribe"
#define MyAppExeName "whisper-transcribe.exe"

[Setup]
; App information
AppId={{8F3D5A8B-4C2E-4F1A-9E3D-7B2C8A5D4E9F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\LICENSE
OutputDir=..\release
OutputBaseFilename=whisper-transcribe-v{#MyAppVersion}-setup
SetupIconFile=compiler:SetupClassicIcon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesInstallIn64BitMode=x64compatible
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\release\whisper-transcribe-v0.1.0-minimal\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0-minimal\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0-minimal\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0-minimal\INSTALL.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0-minimal\RELEASE_NOTES.md"; DestDir: "{app}"; Flags: ignoreversion
; Include CUDA DLLs for portable execution
Source: "..\release\whisper-transcribe-v0.1.0\cudart64_13.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0\cublas64_13.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\release\whisper-transcribe-v0.1.0\cublasLt64_13.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
var
  CudaCheckPage: TOutputMsgMemoWizardPage;
  CudaInstalled: Boolean;

function CheckCudaInstalled: Boolean;
var
  CudaPath: String;
begin
  Result := False;
  
  // Check if CUDA_PATH environment variable exists
  if RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'CUDA_PATH', CudaPath) then
  begin
    Result := DirExists(CudaPath);
  end;
  
  // Also check common CUDA installation path
  if not Result then
  begin
    Result := DirExists(ExpandConstant('{commonpf}\NVIDIA GPU Computing Toolkit\CUDA\v13.0'));
  end;
end;

function CheckCudaDllsInPath: Boolean;
var
  PathValue: String;
begin
  Result := False;
  
  if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'PATH', PathValue) then
  begin
    Result := Pos('CUDA\v13.0\bin\x64', PathValue) > 0;
  end;
end;

procedure InitializeWizard;
begin
  CudaCheckPage := CreateOutputMsgMemoPage(wpSelectDir,
    'GPU Acceleration Check', 'Checking for NVIDIA CUDA support',
    'The installer is checking if CUDA is installed for GPU acceleration.',
    '');
end;

procedure CurPageChanged(CurPageID: Integer);
var
  StatusText: String;
begin
  if CurPageID = CudaCheckPage.ID then
  begin
    CudaInstalled := CheckCudaInstalled;
    
    if CudaInstalled then
    begin
      StatusText := '✓ CUDA Toolkit detected!' + #13#10#13#10;
      
      if CheckCudaDllsInPath then
      begin
        StatusText := StatusText + '✓ CUDA DLLs are in PATH - GPU acceleration ready!' + #13#10#13#10;
        StatusText := StatusText + 'The application will use your NVIDIA GPU for faster transcription.';
      end
      else
      begin
        StatusText := StatusText + '⚠ CUDA DLLs not in PATH' + #13#10#13#10;
        StatusText := StatusText + 'To enable GPU acceleration, add CUDA to PATH:' + #13#10;
        StatusText := StatusText + '1. Open PowerShell' + #13#10;
        StatusText := StatusText + '2. Run this command:' + #13#10#13#10;
        StatusText := StatusText + '$cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\bin\x64"' + #13#10;
        StatusText := StatusText + '[Environment]::SetEnvironmentVariable("PATH", [Environment]::GetEnvironmentVariable("PATH", "User") + ";$cudaPath", "User")' + #13#10#13#10;
        StatusText := StatusText + '3. Restart your computer';
      end;
    end
    else
    begin
      StatusText := '⚠ CUDA Toolkit not detected' + #13#10#13#10;
      StatusText := StatusText + 'The application will run in CPU-only mode (slower transcription).' + #13#10#13#10;
      StatusText := StatusText + 'To enable GPU acceleration with NVIDIA GPUs:' + #13#10;
      StatusText := StatusText + '1. Open PowerShell as Administrator' + #13#10;
      StatusText := StatusText + '2. Run: winget install Nvidia.CUDA --version 13.0' + #13#10;
      StatusText := StatusText + '3. Follow the PATH setup instructions above' + #13#10;
      StatusText := StatusText + '4. Restart your computer' + #13#10#13#10;
      StatusText := StatusText + 'Visit: https://github.com/papacasper/whisper-transcribe#gpu-acceleration';
    end;
    
    CudaCheckPage.RichEditViewer.Text := StatusText;
  end;
end;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  Result := False;
end;
