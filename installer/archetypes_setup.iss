#define MyAppName "Archetypes"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#ifndef MyAppBuildSerial
  #define MyAppBuildSerial "1"
#endif
#ifndef MyAppBuildId
  #define MyAppBuildId "1.0.0+build.1"
#endif
#define MyAppPublisher "NeuroCognica"

[Setup]
AppId={{B2B39917-DBAB-48A9-9901-7B665FCE7A71}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
UninstallDisplayName={#MyAppName} {#MyAppVersion}
DefaultDirName={autopf}\Archetypes
DefaultGroupName=NeuroCognica
DisableProgramGroupPage=yes
PrivilegesRequired=admin
#ifdef MyAppSigned
SignTool=archetypes
#endif
SetupMutex=Archetypes_Setup,Global\Archetypes_Setup
AllowCancelDuringInstall=yes
OutputDir=output
OutputBaseFilename=Archetypes_Setup
LicenseFile=EULA.txt
InfoBeforeFile=DEPENDENCY_NOTICE.txt
SetupIconFile=..\assets\icons\archetypes.ico
UninstallDisplayIcon={app}\archetypes.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
MinVersion=10.0.18362
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce

; Player data is intentionally not created by this elevated installer. The game
; creates saves/profiles/config/logs beneath the interactive player's
; %LOCALAPPDATA% identity, and no [UninstallDelete] entry targets that tree.

[Files]
Source: "..\target\release\engine.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\launcher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\icons\archetypes.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
; Repository ledgers and internal audits are deliberately not customer payload.
; Player help already ships beneath assets\help with the runtime asset tree.
Source: "version.json"; DestDir: "{app}\installer"; Flags: ignoreversion

[Icons]
Name: "{group}\Archetypes"; Filename: "{app}\launcher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\archetypes.ico"
Name: "{autodesktop}\Archetypes"; Filename: "{app}\launcher.exe"; WorkingDir: "{app}"; IconFilename: "{app}\archetypes.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\launcher.exe"; Description: "Launch Archetypes"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  ReceiptPath: String;
  ReceiptText: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    ReceiptPath := ExpandConstant('{localappdata}\NeuroCognica\Archetypes_uninstall_receipt.txt');
    ForceDirectories(ExtractFileDir(ReceiptPath));
    ReceiptText := 'Archetypes uninstall completed.' + #13#10 +
      'Removed: immutable application files and shortcuts.' + #13#10 +
      'Preserved: {localappdata}\NeuroCognica\Archetypes player data.' + #13#10;
    SaveStringToFile(ReceiptPath, ReceiptText, False);
  end;
end;
