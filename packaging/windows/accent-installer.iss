; Accent Markdown Editor — Windows 11 User-Level Installer (Inno Setup 6)
; Installs exclusively into the current user's profile (%LOCALAPPDATA%\Programs\Accent)
; and HKEY_CURRENT_USER without requiring Administrator privileges (no UAC prompt).

#define MyAppName "Accent"
#define MyAppFullName "Accent Markdown Editor"
#define MyAppVersion "0.8.1"
#define MyAppPublisher "Accent Developers"
#define MyAppURL "https://github.com/accent-editor/accent"
#define MyAppExeName "accent.exe"

[Setup]
AppId={{8F4E7A2C-5C91-4B8E-9A1D-0700ACCENT07}
AppName={#MyAppFullName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppFullName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
; User-level installation in %LOCALAPPDATA%\Programs\Accent (No Admin/UAC required)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppFullName}
DisableProgramGroupPage=yes
ChangesAssociations=yes
ChangesEnvironment=yes
OutputDir=dist
OutputBaseFilename=Accent-{#MyAppVersion}-win11-x64-user-setup
SetupIconFile=..\..\img\brand\accent_logo.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.19041

[Languages]
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "fileassoc"; Description: "Associa i file Markdown (.md, .markdown) ad Accent"; GroupDescription: "Integrazione Windows 11:"
Name: "addtopath"; Description: "Aggiungi Accent al PATH utente (%LOCALAPPDATA%\Programs\Accent)"; GroupDescription: "Integrazione Windows 11:"

[Files]
Source: "..\..\target\release\accent-ui.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\img\brand\accent_logo.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\img\brand\accent_logo.png"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\VERSION"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppFullName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\accent_logo.ico"; Comment: "Editor e visualizzatore Markdown multipiattaforma"
Name: "{autodesktop}\{#MyAppFullName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\accent_logo.ico"; Tasks: desktopicon

[Registry]
; Per-user (HKCU) ProgID registration for Markdown files on Windows 11
Root: HKA; Subkey: "Software\Classes\AccentMarkdownFile"; ValueType: string; ValueName: ""; ValueData: "Documento Markdown"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\AccentMarkdownFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\accent_logo.ico,0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\AccentMarkdownFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\.md\OpenWithProgids"; ValueType: string; ValueName: "AccentMarkdownFile"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\.markdown\OpenWithProgids"; ValueType: string; ValueName: "AccentMarkdownFile"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
; Register in Windows 11 Applications list
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".md"; ValueData: ""; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".markdown"; ValueData: ""; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppFullName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
