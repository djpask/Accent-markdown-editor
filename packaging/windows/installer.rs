#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

const APP_VERSION: &str = env!("ACCENT_VERSION");
const ACCENT_EXE_BYTES: &[u8] =
    include_bytes!("../../target/x86_64-pc-windows-gnu/release/accent-ui.exe");
const ACCENT_ICO_BYTES: &[u8] = include_bytes!("../../img/brand/accent_logo.ico");

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MB_OK: u32 = 0x0000_0000;
const MB_YESNO: u32 = 0x0000_0004;
const MB_ICONERROR: u32 = 0x0000_0010;
const MB_ICONQUESTION: u32 = 0x0000_0020;
const MB_ICONWARNING: u32 = 0x0000_0030;
const MB_ICONINFORMATION: u32 = 0x0000_0040;
const IDYES: i32 = 6;

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(hwnd: usize, text: *const u16, caption: *const u16, utype: u32) -> i32;
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn msg_box(title: &str, text: &str, flags: u32) -> i32 {
    let w_title = to_wide(title);
    let w_text = to_wide(text);
    unsafe { MessageBoxW(0, w_text.as_ptr(), w_title.as_ptr(), flags) }
}

fn install_dir() -> PathBuf {
    let local_app_data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let user = env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"));
            user.join("AppData\\Local")
        });
    local_app_data.join("Programs").join("Accent")
}

fn run_install(silent: bool) -> Result<(), String> {
    let dir = install_dir();
    let title = format!("Installazione di Accent Markdown Editor v{APP_VERSION}");

    if !silent {
        let prompt = format!(
            "Vuoi installare Accent Markdown Editor v{APP_VERSION} nel tuo profilo utente?\n\n\
             Cartella di installazione:\n{}\n\n\
             • Nessun privilegio di Amministratore richiesto\n\
             • Collegamenti nel Menu Start e sul Desktop\n\
             • Associazione per i file .md e .markdown\n\
             • Disinstallatore registrato in 'App installate' di Windows 11",
            dir.display()
        );
        if msg_box(&title, &prompt, MB_YESNO | MB_ICONQUESTION) != IDYES {
            return Ok(());
        }
    }

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Impossibile creare la cartella {}: {e}", dir.display()))?;

    let target_exe = dir.join("accent.exe");
    let target_ico = dir.join("accent_logo.ico");
    let target_uninst = dir.join("Accent-Uninstall.exe");

    // Terminate running instance if upgrading in-place
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "accent.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    fs::write(&target_exe, ACCENT_EXE_BYTES)
        .map_err(|e| format!("Impossibile scrivere {}: {e}", target_exe.display()))?;
    fs::write(&target_ico, ACCENT_ICO_BYTES)
        .map_err(|e| format!("Impossibile scrivere {}: {e}", target_ico.display()))?;

    if let Ok(self_exe) = env::current_exe() {
        let _ = fs::copy(&self_exe, &target_uninst);
    }

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
$InstallDir = '{dir}'
$TargetExe = Join-Path $InstallDir 'accent.exe'
$TargetIcon = Join-Path $InstallDir 'accent_logo.ico'
$TargetUninst = Join-Path $InstallDir 'Accent-Uninstall.exe'

$StartMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$DesktopDir = [Environment]::GetFolderPath('Desktop')
$WshShell = New-Object -ComObject WScript.Shell

foreach ($LnkPath in @(
    (Join-Path $StartMenuDir 'Accent Markdown Editor.lnk'),
    (Join-Path $DesktopDir 'Accent Markdown Editor.lnk')
)) {{
    $Shortcut = $WshShell.CreateShortcut($LnkPath)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.Description = 'Editor e visualizzatore Markdown multipiattaforma in puro Rust'
    $Shortcut.IconLocation = "$TargetIcon,0"
    $Shortcut.Save()
}}

$ProgIdKey = 'HKCU:\Software\Classes\AccentMarkdownFile'
New-Item -Path "$ProgIdKey\DefaultIcon" -Force | Out-Null
New-Item -Path "$ProgIdKey\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path $ProgIdKey -Name '(Default)' -Value 'Documento Markdown'
Set-ItemProperty -Path "$ProgIdKey\DefaultIcon" -Name '(Default)' -Value "$TargetIcon,0"
Set-ItemProperty -Path "$ProgIdKey\shell\open\command" -Name '(Default)' -Value "`"$TargetExe`" `"%1`""

foreach ($Ext in @('.md', '.markdown')) {{
    $OpenWithKey = "HKCU:\Software\Classes\$Ext\OpenWithProgids"
    New-Item -Path $OpenWithKey -Force | Out-Null
    Set-ItemProperty -Path $OpenWithKey -Name 'AccentMarkdownFile' -Value ''
}}

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $UserPath) {{ $UserPath = '' }}
if (($UserPath -split ';') -notcontains $InstallDir) {{
    $NewUserPath = if ($UserPath) {{ "$UserPath;$InstallDir" }} else {{ $InstallDir }}
    [Environment]::SetEnvironmentVariable('Path', $NewUserPath, 'User')
}}

$UninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AccentMarkdownEditor'
New-Item -Path $UninstallKey -Force | Out-Null
Set-ItemProperty -Path $UninstallKey -Name 'DisplayName' -Value 'Accent Markdown Editor'
Set-ItemProperty -Path $UninstallKey -Name 'DisplayVersion' -Value '{ver}'
Set-ItemProperty -Path $UninstallKey -Name 'Publisher' -Value 'Accent Developers'
Set-ItemProperty -Path $UninstallKey -Name 'InstallLocation' -Value $InstallDir
Set-ItemProperty -Path $UninstallKey -Name 'DisplayIcon' -Value $TargetIcon
Set-ItemProperty -Path $UninstallKey -Name 'UninstallString' -Value "`"$TargetUninst`" --uninstall"
Set-ItemProperty -Path $UninstallKey -Name 'QuietUninstallString' -Value "`"$TargetUninst`" --uninstall --silent"
Set-ItemProperty -Path $UninstallKey -Name 'NoModify' -Value 1 -Type DWord
Set-ItemProperty -Path $UninstallKey -Name 'NoRepair' -Value 1 -Type DWord
"#,
        dir = dir.display().to_string().replace('\'', "''"),
        ver = APP_VERSION
    );

    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    if !silent {
        let done_msg = format!(
            "Accent Markdown Editor v{APP_VERSION} è stato installato con successo!\n\n\
             Percorso: {}\n\n\
             Vuoi avviare Accent adesso?",
            target_exe.display()
        );
        if msg_box(&title, &done_msg, MB_YESNO | MB_ICONINFORMATION) == IDYES {
            let _ = Command::new(&target_exe).spawn();
        }
    }

    Ok(())
}

fn run_uninstall(silent: bool) -> Result<(), String> {
    let dir = install_dir();
    let title = "Disinstallazione di Accent Markdown Editor";

    if !silent {
        let prompt = format!(
            "Vuoi disinstallare completamente Accent Markdown Editor dal tuo profilo utente?\n\n\
             Verranno rimossi:\n\
             • {}\n\
             • Collegamenti nel Menu Start e sul Desktop\n\
             • Associazioni file e voce in 'App installate'",
            dir.display()
        );
        if msg_box(title, &prompt, MB_YESNO | MB_ICONWARNING) != IDYES {
            return Ok(());
        }
    }

    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "accent.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
$InstallDir = '{dir}'
$StartMenuShortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Accent Markdown Editor.lnk'
$DesktopShortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Accent Markdown Editor.lnk'

if (Test-Path $StartMenuShortcut) {{ Remove-Item $StartMenuShortcut -Force }}
if (Test-Path $DesktopShortcut) {{ Remove-Item $DesktopShortcut -Force }}

Remove-Item -Path 'HKCU:\Software\Classes\AccentMarkdownFile' -Recurse -Force
Remove-ItemProperty -Path 'HKCU:\Software\Classes\.md\OpenWithProgids' -Name 'AccentMarkdownFile' -Force
Remove-ItemProperty -Path 'HKCU:\Software\Classes\.markdown\OpenWithProgids' -Name 'AccentMarkdownFile' -Force
Remove-Item -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\AccentMarkdownEditor' -Recurse -Force

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath) {{
    $CleanedPath = ($UserPath -split ';' | Where-Object {{ $_ -and ($_ -ne $InstallDir) }}) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $CleanedPath, 'User')
}}
"#,
        dir = dir.display().to_string().replace('\'', "''")
    );

    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let _ = fs::remove_file(dir.join("accent.exe"));
    let _ = fs::remove_file(dir.join("accent_logo.ico"));

    // Schedule removal of the directory (including Accent-Uninstall.exe after this process exits)
    let cmd_cleanup = format!(
        "ping 127.0.0.1 -n 2 > nul & rmdir /s /q \"{}\"",
        dir.display()
    );
    let _ = Command::new("cmd.exe")
        .args(["/C", &cmd_cleanup])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();

    if !silent {
        msg_box(
            title,
            "Accent Markdown Editor è stato disinstallato correttamente dal sistema.",
            MB_OK | MB_ICONINFORMATION,
        );
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let is_uninstall = exe_name.contains("uninstall")
        || args.iter().any(|a| {
            let l = a.to_ascii_lowercase();
            l == "--uninstall" || l == "/uninstall" || l == "-u"
        });
    let is_silent = args.iter().any(|a| {
        let l = a.to_ascii_lowercase();
        l == "--silent" || l == "/s" || l == "/quiet" || l == "-q"
    });

    let res = if is_uninstall {
        run_uninstall(is_silent)
    } else {
        run_install(is_silent)
    };

    if let Err(err) = res {
        if !is_silent {
            msg_box(
                "Errore — Accent Markdown Editor",
                &err,
                MB_OK | MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}
