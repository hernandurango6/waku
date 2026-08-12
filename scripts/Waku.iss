#define AppName "Waku"
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef StageDir
  #define StageDir "dist\windows\Waku"
#endif
#ifndef OutputDir
  #define OutputDir "dist\windows"
#endif

[Setup]
AppId={{B8A4A6C9-0D8A-4ED3-8D9D-9B9E5D5F3D4A}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={autopf}\Waku
DefaultGroupName=Waku
OutputDir={#OutputDir}
OutputBaseFilename=Waku-{#AppVersion}-Setup
ArchitecturesInstallIn64BitMode=x64
ArchitecturesAllowed=x64
PrivilegesRequired=lowest
Compression=lzma
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\waku.exe

[Files]
Source: "{#StageDir}\waku.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\waku_js_repl.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Waku"; Filename: "{app}\waku.exe"
Name: "{autodesktop}\Waku"; Filename: "{app}\waku.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\waku.exe"; Description: "Launch Waku"; Flags: nowait postinstall skipifsilent
