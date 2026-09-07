; CapsLang installer — Inno Setup 6
;
; Installs per-user so no elevation prompt appears. A keyboard hook does not
; need administrator rights; the only thing it buys is the ability to see keys
; in elevated windows, which is a trade-off left to the user.
;
; Build with:  ISCC.exe /DBinDir=..\target\x86_64-pc-windows-msvc\release installer\capslang.iss

#define AppName      "CapsLang"
#define AppExe       "capslang.exe"
#define AppPublisher "technocoluzi"
#define AppUrl       "https://github.com/technocoluzi/capslang"

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

#ifndef BinDir
  #define BinDir "..\target\release"
#endif

[Setup]
AppId={{9C1E0B27-2C4B-4C8E-9A2E-2E6E2B1E5A31}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppUrl}
AppSupportURL={#AppUrl}/issues
AppUpdatesURL={#AppUrl}/releases
VersionInfoVersion={#AppVersion}

; "lowest" keeps the whole install inside the user's profile: no UAC prompt,
; and {autopf} resolves to %LOCALAPPDATA%\Programs.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
DisableDirPage=auto
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
LicenseFile=..\LICENSE

OutputDir=..\dist
OutputBaseFilename=CapsLang-{#AppVersion}-setup
SetupIconFile=..\assets\capslang.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; Offer to close a running CapsLang instead of forcing a reboot.
CloseApplications=yes
CloseApplicationsFilter=*.exe
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "startup"; Description: "Start {#AppName} when I sign in"; GroupDescription: "Additional options:"

[Files]
Source: "{#BinDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md";        DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE";          DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"

[Run]
; Order matters: register autostart before launching, so the tray menu shows
; the right check mark straight away.
Filename: "{app}\{#AppExe}"; Parameters: "--autostart on"; Tasks: startup; Flags: runhidden waituntilterminated
; No skipifsilent: an unattended install should leave CapsLang running, which
; is the entire point of installing it.
Filename: "{app}\{#AppExe}"; Description: "Start {#AppName} now"; Flags: nowait postinstall

[UninstallRun]
Filename: "{app}\{#AppExe}"; Parameters: "--quit";          Flags: runhidden waituntilterminated; RunOnceId: "StopCapsLang"
Filename: "{app}\{#AppExe}"; Parameters: "--autostart off"; Flags: runhidden waituntilterminated; RunOnceId: "ClearAutostart"

[UninstallDelete]
; The config lives outside {app}; remove the folder we created for it.
Type: filesandordirs; Name: "{userappdata}\{#AppName}"

[Code]
function InitializeSetup(): Boolean;
begin
  Result := True;
end;
