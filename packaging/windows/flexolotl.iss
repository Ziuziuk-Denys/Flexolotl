#define Name "Flexolotl"

[Setup]
AppId={{6E0D7C6A-3B7D-4C62-9A86-5C1E0F6B7A41}
AppName={#Name}
AppVersion={#Version}
AppPublisher=Denys Ziuziuk
AppPublisherURL=https://github.com/Ziuziuk-Denys/Flexolotl
DefaultDirName={autopf}\{#Name}
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed={#Architecture}
ArchitecturesInstallIn64BitMode={#Architecture}
SetupIconFile=flexolotl.ico
UninstallDisplayIcon={app}\flexolotl.exe
OutputDir={#Output}
OutputBaseFilename=Flexolotl-{#Version}-win-{#Label}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"

[Files]
Source: "{#Program}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#Name}"; Filename: "{app}\flexolotl.exe"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "{#Name}"; Flags: dontcreatekey uninsdeletevalue

[Run]
Filename: "{app}\flexolotl.exe"; Description: "{cm:LaunchProgram,{#Name}}"; Flags: nowait postinstall skipifsilent
