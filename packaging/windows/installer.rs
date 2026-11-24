#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::env;
use std::ffi::c_void;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{null, null_mut};
use std::thread;

const APP_VERSION: &str = env!("ACCENT_VERSION");
const ACCENT_EXE_BYTES: &[u8] = include_bytes!("accent.exe");
const ACCENT_ICO_BYTES: &[u8] = include_bytes!("../../img/brand/accent_logo.ico");

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// Win32 basic types
type HWND = *mut c_void;
type HINSTANCE = *mut c_void;
type HICON = *mut c_void;
type HCURSOR = *mut c_void;
type HBRUSH = *mut c_void;
type HFONT = *mut c_void;
type HMENU = *mut c_void;
type HDC = *mut c_void;
type LPARAM = isize;
type WPARAM = usize;
type LRESULT = isize;
type BOOL = i32;
type COLORREF = u32;

// Window styles & messages
const WS_OVERLAPPED: u32 = 0x0000_0000;
const WS_CAPTION: u32 = 0x00C0_0000;
const WS_SYSMENU: u32 = 0x0008_0000;
const WS_MINIMIZEBOX: u32 = 0x0002_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const WS_CHILD: u32 = 0x4000_0000;
const WS_TABSTOP: u32 = 0x0001_0000;
const WS_BORDER: u32 = 0x0080_0000;
const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;

const BS_PUSHBUTTON: u32 = 0x0000_0000;
const BS_DEFPUSHBUTTON: u32 = 0x0000_0001;
const BS_AUTOCHECKBOX: u32 = 0x0000_0003;
const ES_AUTOHSCROLL: u32 = 0x0000_0080;
const SS_ICON: u32 = 0x0000_0003;
const SS_CENTERIMAGE: u32 = 0x0000_0200;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_SETFONT: u32 = 0x0030;
const WM_COMMAND: u32 = 0x0111;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_CTLCOLORBTN: u32 = 0x0135;
const WM_USER: u32 = 0x0400;
const WM_APP_PROGRESS: u32 = WM_USER + 10;
const WM_APP_DONE: u32 = WM_USER + 11;

const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;

const PBM_SETRANGE: u32 = WM_USER + 1;
const PBM_SETPOS: u32 = WM_USER + 2;
const STM_SETICON: u32 = 0x0170;

const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;

const MB_OK: u32 = 0x0000_0000;
const MB_YESNO: u32 = 0x0000_0004;
const MB_ICONERROR: u32 = 0x0000_0010;
const MB_ICONQUESTION: u32 = 0x0000_0020;
const IDYES: i32 = 6;

const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const IDC_ARROW: usize = 32512;
const IMAGE_ICON: u32 = 1;
const LR_DEFAULTCOLOR: u32 = 0x0000_0000;
const TRANSPARENT: i32 = 1;
const FW_NORMAL: i32 = 400;
const FW_SEMIBOLD: i32 = 600;
const FW_BOLD: i32 = 700;
const DEFAULT_CHARSET: u32 = 1;
const OUT_DEFAULT_PRECIS: u32 = 0;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const CLEARTYPE_QUALITY: u32 = 5;
const DEFAULT_PITCH: u32 = 0;

const ICC_PROGRESS_CLASS: u32 = 0x0000_0020;
const ICC_STANDARD_CLASSES: u32 = 0x0000_4000;
const BIF_RETURNONLYFSDIRS: u32 = 0x0000_0001;
const BIF_NEWDIALOGSTYLE: u32 = 0x0000_0040;

// Control IDs
const ID_BTN_BACK: usize = 1001;
const ID_BTN_NEXT: usize = 1002;
const ID_BTN_CANCEL: usize = 1003;
const ID_EDIT_DIR: usize = 1010;
const ID_BTN_BROWSE: usize = 1011;
const ID_CHK_STARTMENU: usize = 1020;
const ID_CHK_DESKTOP: usize = 1021;
const ID_CHK_ASSOC: usize = 1022;
const ID_CHK_PATH: usize = 1023;
const ID_CHK_LAUNCH: usize = 1024;
const ID_PROGRESS: usize = 1030;

// Amber & Coral palette accents (COLORREF = 0x00BBGGRR)
const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}
const COL_WHITE: COLORREF = rgb(0xFF, 0xFF, 0xFF);
const COL_BG: COLORREF = rgb(0xFA, 0xF8, 0xF5);
const COL_DIVIDER: COLORREF = rgb(0xDD, 0xD6, 0xCC);
const COL_AMBER: COLORREF = rgb(0xC8, 0x5A, 0x17);
const COL_CORAL: COLORREF = rgb(0xC8, 0x28, 0x50);
const COL_TEXT: COLORREF = rgb(0x22, 0x25, 0x2A);
const COL_MUTED: COLORREF = rgb(0x64, 0x69, 0x72);

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HICON,
    h_cursor: HCURSOR,
    hbr_background: HBRUSH,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HICON,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: HDC,
    f_erase: BOOL,
    rc_paint: RECT,
    f_restore: BOOL,
    f_inc_update: BOOL,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct INITCOMMONCONTROLSEX {
    dw_size: u32,
    dw_icc: u32,
}

#[repr(C)]
struct BROWSEINFOW {
    hwnd_owner: HWND,
    pidl_root: *const c_void,
    psz_display_name: *mut u16,
    lpsz_title: *const u16,
    ul_flags: u32,
    lpfn: *const c_void,
    l_param: LPARAM,
    i_image: i32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: HWND,
        h_menu: HMENU,
        h_instance: HINSTANCE,
        lp_param: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    fn ShowWindow(hwnd: HWND, n_cmd_show: i32) -> BOOL;
    fn UpdateWindow(hwnd: HWND) -> BOOL;
    fn EnableWindow(hwnd: HWND, b_enable: BOOL) -> BOOL;
    fn DestroyWindow(hwnd: HWND) -> BOOL;
    fn PostQuitMessage(n_exit_code: i32);
    fn GetMessageW(lp_msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> BOOL;
    fn IsDialogMessageW(h_dlg: HWND, lp_msg: *mut MSG) -> BOOL;
    fn TranslateMessage(lp_msg: *const MSG) -> BOOL;
    fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
    fn SendMessageW(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    fn PostMessageW(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> BOOL;
    fn SetWindowTextW(hwnd: HWND, lp_string: *const u16) -> BOOL;
    fn GetWindowTextW(hwnd: HWND, lp_string: *mut u16, n_max_count: i32) -> i32;
    fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, utype: u32) -> i32;
    fn LoadCursorW(h_instance: HINSTANCE, lp_cursor_name: *const u16) -> HCURSOR;
    fn LoadImageW(
        h_inst: HINSTANCE,
        name: *const u16,
        img_type: u32,
        cx: i32,
        cy: i32,
        fu_load: u32,
    ) -> *mut c_void;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn BeginPaint(hwnd: HWND, lp_paint: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(hwnd: HWND, lp_paint: *const PAINTSTRUCT) -> BOOL;
    fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn InvalidateRect(hwnd: HWND, lp_rect: *const RECT, b_erase: BOOL) -> BOOL;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> HFONT;
    fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    fn DeleteObject(ho: *mut c_void) -> BOOL;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
}

#[link(name = "comctl32")]
unsafe extern "system" {
    fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> BOOL;
}

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoInitializeEx(pv_reserved: *mut c_void, dw_co_init: u32) -> i32;
    fn CoTaskMemFree(pv: *mut c_void);
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn SHBrowseForFolderW(lpbi: *mut BROWSEINFOW) -> *mut c_void;
    fn SHGetPathFromIDListW(pidl: *const c_void, psz_path: *mut u16) -> BOOL;
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn set_text(hwnd: HWND, text: &str) {
    let w = to_wide(text);
    unsafe {
        SetWindowTextW(hwnd, w.as_ptr());
    }
}

fn get_text(hwnd: HWND) -> String {
    let mut buf = vec![0u16; 1024];
    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len as usize])
}

fn is_checked(hwnd: HWND) -> bool {
    unsafe { SendMessageW(hwnd, BM_GETCHECK, 0, 0) as usize == BST_CHECKED }
}

fn set_checked(hwnd: HWND, checked: bool) {
    unsafe {
        SendMessageW(
            hwnd,
            BM_SETCHECK,
            if checked { BST_CHECKED } else { BST_UNCHECKED },
            0,
        );
    }
}

fn show_ctrl(hwnd: HWND, visible: bool) {
    unsafe {
        ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
    }
}

fn enable_ctrl(hwnd: HWND, enabled: bool) {
    unsafe {
        EnableWindow(hwnd, if enabled { 1 } else { 0 });
    }
}

fn msg_box(owner: HWND, title: &str, text: &str, flags: u32) -> i32 {
    let w_title = to_wide(title);
    let w_text = to_wide(text);
    unsafe { MessageBoxW(owner, w_text.as_ptr(), w_title.as_ptr(), flags) }
}

fn default_install_dir() -> PathBuf {
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

fn detect_installed_dir() -> PathBuf {
    // If launched from the installed Accent-Uninstall.exe, use its parent directory
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("accent.exe").exists() {
                return parent.to_path_buf();
            }
        }
    }
    // Otherwise check HKCU Uninstall InstallLocation
    let ps_query = "(Get-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\AccentMarkdownEditor' -ErrorAction SilentlyContinue).InstallLocation";
    if let Ok(out) = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            ps_query,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return PathBuf::from(s);
        }
    }
    default_install_dir()
}

fn browse_folder(owner: HWND, title: &str) -> Option<PathBuf> {
    let mut display = vec![0u16; 260];
    let w_title = to_wide(title);
    let mut bi = BROWSEINFOW {
        hwnd_owner: owner,
        pidl_root: null(),
        psz_display_name: display.as_mut_ptr(),
        lpsz_title: w_title.as_ptr(),
        ul_flags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
        lpfn: null(),
        l_param: 0,
        i_image: 0,
    };
    unsafe {
        let pidl = SHBrowseForFolderW(&mut bi);
        if pidl.is_null() {
            return None;
        }
        let mut path_buf = vec![0u16; 1024];
        let ok = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr()) != 0;
        CoTaskMemFree(pidl);
        if ok {
            let end = path_buf.iter().position(|&c| c == 0).unwrap_or(path_buf.len());
            let s = String::from_utf16_lossy(&path_buf[..end]);
            if !s.is_empty() {
                return Some(PathBuf::from(s));
            }
        }
    }
    None
}

#[derive(Clone)]
struct InstallOptions {
    dir: PathBuf,
    create_start_menu: bool,
    create_desktop: bool,
    associate_markdown: bool,
    add_to_path: bool,
}

fn post_progress(hwnd: HWND, percent: usize, msg: &str) {
    if hwnd.is_null() {
        return;
    }
    let boxed = Box::into_raw(Box::new(msg.to_string())) as LPARAM;
    unsafe {
        PostMessageW(hwnd, WM_APP_PROGRESS, percent, boxed);
    }
}

fn post_done(hwnd: HWND, res: Result<String, String>) {
    if hwnd.is_null() {
        return;
    }
    let boxed = Box::into_raw(Box::new(res)) as LPARAM;
    unsafe {
        PostMessageW(hwnd, WM_APP_DONE, 0, boxed);
    }
}

fn perform_install(opts: &InstallOptions, hwnd: HWND) -> Result<String, String> {
    let dir = &opts.dir;
    post_progress(hwnd, 12, "Chiusura eventuali istanze di Accent in esecuzione...");
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "accent.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    post_progress(hwnd, 28, "Creazione della cartella di installazione...");
    fs::create_dir_all(dir)
        .map_err(|e| format!("Impossibile creare la cartella {}:\n{e}", dir.display()))?;

    let target_exe = dir.join("accent.exe");
    let target_ico = dir.join("accent_logo.ico");
    let target_uninst = dir.join("Accent-Uninstall.exe");

    post_progress(hwnd, 52, "Estrazione di Accent Markdown Editor...");
    fs::write(&target_exe, ACCENT_EXE_BYTES)
        .map_err(|e| format!("Impossibile scrivere {}:\n{e}", target_exe.display()))?;
    fs::write(&target_ico, ACCENT_ICO_BYTES)
        .map_err(|e| format!("Impossibile scrivere {}:\n{e}", target_ico.display()))?;

    post_progress(hwnd, 68, "Installazione del disinstallatore...");
    if let Ok(self_exe) = env::current_exe() {
        if self_exe != target_uninst {
            let _ = fs::copy(&self_exe, &target_uninst);
        }
    }

    post_progress(
        hwnd,
        84,
        "Configurazione collegamenti, associazioni file e registro utente...",
    );

    let dir_ps = dir.display().to_string().replace('\'', "''");
    let create_start_ps = if opts.create_start_menu { "$true" } else { "$false" };
    let create_desktop_ps = if opts.create_desktop { "$true" } else { "$false" };
    let assoc_md_ps = if opts.associate_markdown { "$true" } else { "$false" };
    let add_path_ps = if opts.add_to_path { "$true" } else { "$false" };

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
$InstallDir = '{dir_ps}'
$TargetExe = Join-Path $InstallDir 'accent.exe'
$TargetIcon = Join-Path $InstallDir 'accent_logo.ico'
$TargetUninst = Join-Path $InstallDir 'Accent-Uninstall.exe'

$WshShell = New-Object -ComObject WScript.Shell
if ({create_start_ps}) {{
    $StartMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    $Lnk = Join-Path $StartMenuDir 'Accent Markdown Editor.lnk'
    $Shortcut = $WshShell.CreateShortcut($Lnk)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.Description = 'Editor e visualizzatore Markdown multipiattaforma in puro Rust'
    $Shortcut.IconLocation = "$TargetIcon,0"
    $Shortcut.Save()
}}

if ({create_desktop_ps}) {{
    $DesktopDir = [Environment]::GetFolderPath('Desktop')
    $Lnk = Join-Path $DesktopDir 'Accent Markdown Editor.lnk'
    $Shortcut = $WshShell.CreateShortcut($Lnk)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $env:USERPROFILE
    $Shortcut.Description = 'Editor e visualizzatore Markdown multipiattaforma in puro Rust'
    $Shortcut.IconLocation = "$TargetIcon,0"
    $Shortcut.Save()
}}

if ({assoc_md_ps}) {{
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
}}

if ({add_path_ps}) {{
    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $UserPath) {{ $UserPath = '' }}
    if (($UserPath -split ';') -notcontains $InstallDir) {{
        $NewUserPath = if ($UserPath) {{ "$UserPath;$InstallDir" }} else {{ $InstallDir }}
        [Environment]::SetEnvironmentVariable('Path', $NewUserPath, 'User')
    }}
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

    post_progress(hwnd, 100, "Installazione completata!");
    Ok(target_exe.display().to_string())
}

fn perform_uninstall(dir: &Path, hwnd: HWND) -> Result<String, String> {
    post_progress(hwnd, 18, "Chiusura eventuali istanze di Accent in esecuzione...");
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "accent.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    post_progress(
        hwnd,
        48,
        "Rimozione collegamenti, associazioni file e voce di registro...",
    );
    let dir_ps = dir.display().to_string().replace('\'', "''");
    let ps_script = format!(
        r#"
$ErrorActionPreference = 'SilentlyContinue'
$InstallDir = '{dir_ps}'
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
"#
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

    post_progress(hwnd, 80, "Rimozione dei file dell'applicazione...");
    let _ = fs::remove_file(dir.join("accent.exe"));
    let _ = fs::remove_file(dir.join("accent_logo.ico"));

    post_progress(hwnd, 100, "Disinstallazione completata!");
    Ok(dir.display().to_string())
}

fn schedule_self_cleanup(dir: &Path) {
    let cmd_cleanup = format!(
        "ping 127.0.0.1 -n 2 > nul & rmdir /s /q \"{}\"",
        dir.display()
    );
    let _ = Command::new("cmd.exe")
        .args(["/C", &cmd_cleanup])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WizardPage {
    Welcome,
    Options,
    Progress,
    Finish,
}

struct WizardState {
    is_uninstall: bool,
    page: WizardPage,
    target_dir: PathBuf,
    installed_exe: PathBuf,
    finished_ok: bool,
    busy: bool,

    // GDI resources
    h_font_title: HFONT,
    h_font_sub: HFONT,
    h_font_body: HFONT,
    h_font_bold: HFONT,
    h_brush_white: HBRUSH,
    h_brush_bg: HBRUSH,
    h_brush_divider: HBRUSH,
    h_brush_amber: HBRUSH,
    h_brush_coral: HBRUSH,

    // Header controls
    lbl_header_title: HWND,
    lbl_header_sub: HWND,
    img_icon: HWND,

    // Page: Welcome
    lbl_welcome_title: HWND,
    lbl_welcome_body: HWND,

    // Page: Options (Install only)
    lbl_dir_caption: HWND,
    edit_dir: HWND,
    btn_browse: HWND,
    lbl_opts_caption: HWND,
    chk_start_menu: HWND,
    chk_desktop: HWND,
    chk_assoc: HWND,
    chk_path: HWND,

    // Page: Progress
    lbl_progress_status: HWND,
    progress_bar: HWND,

    // Page: Finish
    lbl_finish_title: HWND,
    lbl_finish_body: HWND,
    chk_launch: HWND,

    // Footer buttons
    btn_back: HWND,
    btn_next: HWND,
    btn_cancel: HWND,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            is_uninstall: false,
            page: WizardPage::Welcome,
            target_dir: default_install_dir(),
            installed_exe: PathBuf::new(),
            finished_ok: false,
            busy: false,
            h_font_title: null_mut(),
            h_font_sub: null_mut(),
            h_font_body: null_mut(),
            h_font_bold: null_mut(),
            h_brush_white: null_mut(),
            h_brush_bg: null_mut(),
            h_brush_divider: null_mut(),
            h_brush_amber: null_mut(),
            h_brush_coral: null_mut(),
            lbl_header_title: null_mut(),
            lbl_header_sub: null_mut(),
            img_icon: null_mut(),
            lbl_welcome_title: null_mut(),
            lbl_welcome_body: null_mut(),
            lbl_dir_caption: null_mut(),
            edit_dir: null_mut(),
            btn_browse: null_mut(),
            lbl_opts_caption: null_mut(),
            chk_start_menu: null_mut(),
            chk_desktop: null_mut(),
            chk_assoc: null_mut(),
            chk_path: null_mut(),
            lbl_progress_status: null_mut(),
            progress_bar: null_mut(),
            lbl_finish_title: null_mut(),
            lbl_finish_body: null_mut(),
            chk_launch: null_mut(),
            btn_back: null_mut(),
            btn_next: null_mut(),
            btn_cancel: null_mut(),
        }
    }
}

thread_local! {
    static STATE: RefCell<WizardState> = RefCell::new(WizardState::default());
}

unsafe fn create_font(height: i32, weight: i32) -> HFONT {
    let face = to_wide("Segoe UI");
    unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH,
            face.as_ptr(),
        )
    }
}

unsafe fn make_ctrl(
    parent: HWND,
    h_inst: HINSTANCE,
    class_name: &str,
    text: &str,
    ex_style: u32,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: usize,
    font: HFONT,
) -> HWND {
    let cls = to_wide(class_name);
    let txt = to_wide(text);
    unsafe {
        let ctrl = CreateWindowExW(
            ex_style,
            cls.as_ptr(),
            txt.as_ptr(),
            WS_CHILD | style,
            x,
            y,
            w,
            h,
            parent,
            id as HMENU,
            h_inst,
            null_mut(),
        );
        if !font.is_null() && !ctrl.is_null() {
            SendMessageW(ctrl, WM_SETFONT, font as WPARAM, 1);
        }
        ctrl
    }
}

fn update_page_ui(hwnd: HWND) {
    STATE.with(|cell| {
        let s = cell.borrow();
        let is_welcome = s.page == WizardPage::Welcome;
        let is_options = s.page == WizardPage::Options;
        let is_progress = s.page == WizardPage::Progress;
        let is_finish = s.page == WizardPage::Finish;

        // Header text per step
        if s.is_uninstall {
            match s.page {
                WizardPage::Welcome => {
                    set_text(s.lbl_header_title, "Disinstallazione di Accent");
                    set_text(
                        s.lbl_header_sub,
                        "Rimozione guidata dal profilo utente (nessun privilegio amministrativo)",
                    );
                }
                WizardPage::Progress => {
                    set_text(s.lbl_header_title, "Disinstallazione in corso...");
                    set_text(
                        s.lbl_header_sub,
                        "Attendere mentre Accent viene rimosso dal sistema.",
                    );
                }
                WizardPage::Finish => {
                    set_text(s.lbl_header_title, "Disinstallazione completata");
                    set_text(
                        s.lbl_header_sub,
                        "Accent Markdown Editor è stato rimosso correttamente.",
                    );
                }
                WizardPage::Options => {}
            }
        } else {
            match s.page {
                WizardPage::Welcome => {
                    set_text(
                        s.lbl_header_title,
                        &format!("Accent Markdown Editor v{APP_VERSION}"),
                    );
                    set_text(
                        s.lbl_header_sub,
                        "Installazione guidata per l'utente corrente (senza privilegi di amministratore)",
                    );
                }
                WizardPage::Options => {
                    set_text(s.lbl_header_title, "Cartella e opzioni di installazione");
                    set_text(
                        s.lbl_header_sub,
                        "Scegli dove installare Accent e quali integrazioni attivare su Windows.",
                    );
                }
                WizardPage::Progress => {
                    set_text(s.lbl_header_title, "Installazione in corso...");
                    set_text(
                        s.lbl_header_sub,
                        "Attendere il completamento dell'installazione di Accent.",
                    );
                }
                WizardPage::Finish => {
                    set_text(s.lbl_header_title, "Installazione completata");
                    set_text(
                        s.lbl_header_sub,
                        &format!("Accent Markdown Editor v{APP_VERSION} è pronto all'uso."),
                    );
                }
            }
        }

        // Welcome controls
        show_ctrl(s.lbl_welcome_title, is_welcome);
        show_ctrl(s.lbl_welcome_body, is_welcome);

        // Options controls
        show_ctrl(s.lbl_dir_caption, is_options);
        show_ctrl(s.edit_dir, is_options);
        show_ctrl(s.btn_browse, is_options);
        show_ctrl(s.lbl_opts_caption, is_options);
        show_ctrl(s.chk_start_menu, is_options);
        show_ctrl(s.chk_desktop, is_options);
        show_ctrl(s.chk_assoc, is_options);
        show_ctrl(s.chk_path, is_options);

        // Progress controls
        show_ctrl(s.lbl_progress_status, is_progress);
        show_ctrl(s.progress_bar, is_progress);

        // Finish controls
        show_ctrl(s.lbl_finish_title, is_finish);
        show_ctrl(s.lbl_finish_body, is_finish);
        show_ctrl(s.chk_launch, is_finish && !s.is_uninstall && s.finished_ok);

        // Footer buttons
        match s.page {
            WizardPage::Welcome => {
                show_ctrl(s.btn_back, false);
                show_ctrl(s.btn_next, true);
                show_ctrl(s.btn_cancel, true);
                enable_ctrl(s.btn_next, true);
                enable_ctrl(s.btn_cancel, true);
                set_text(
                    s.btn_next,
                    if s.is_uninstall { "Disinstalla" } else { "Avanti ›" },
                );
            }
            WizardPage::Options => {
                show_ctrl(s.btn_back, true);
                show_ctrl(s.btn_next, true);
                show_ctrl(s.btn_cancel, true);
                enable_ctrl(s.btn_back, true);
                enable_ctrl(s.btn_next, true);
                enable_ctrl(s.btn_cancel, true);
                set_text(s.btn_next, "Installa");
            }
            WizardPage::Progress => {
                show_ctrl(s.btn_back, false);
                show_ctrl(s.btn_next, true);
                show_ctrl(s.btn_cancel, true);
                enable_ctrl(s.btn_next, false);
                enable_ctrl(s.btn_cancel, false);
                set_text(
                    s.btn_next,
                    if s.is_uninstall { "Disinstallazione..." } else { "Installazione..." },
                );
            }
            WizardPage::Finish => {
                show_ctrl(s.btn_back, false);
                show_ctrl(s.btn_next, true);
                show_ctrl(s.btn_cancel, false);
                enable_ctrl(s.btn_next, true);
                set_text(s.btn_next, "Fine");
            }
        }
    });
    unsafe {
        InvalidateRect(hwnd, null(), 1);
    }
}

fn start_worker(hwnd: HWND) {
    let (is_uninstall, opts, uninstall_dir) = STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.page = WizardPage::Progress;
        s.busy = true;
        if !s.is_uninstall {
            let typed = get_text(s.edit_dir).trim().to_string();
            if !typed.is_empty() {
                s.target_dir = PathBuf::from(typed);
            }
        }
        let opts = InstallOptions {
            dir: s.target_dir.clone(),
            create_start_menu: is_checked(s.chk_start_menu),
            create_desktop: is_checked(s.chk_desktop),
            associate_markdown: is_checked(s.chk_assoc),
            add_to_path: is_checked(s.chk_path),
        };
        (s.is_uninstall, opts, s.target_dir.clone())
    });

    update_page_ui(hwnd);
    let hwnd_usize = hwnd as usize;
    thread::spawn(move || {
        let hwnd_ptr = hwnd_usize as HWND;
        let result = if is_uninstall {
            perform_uninstall(&uninstall_dir, hwnd_ptr)
        } else {
            perform_install(&opts, hwnd_ptr)
        };
        post_done(hwnd_ptr, result);
    });
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                let h_inst = GetModuleHandleW(null());
                let f_title = create_font(-20, FW_BOLD);
                let f_sub = create_font(-13, FW_NORMAL);
                let f_bold = create_font(-15, FW_SEMIBOLD);
                let f_body = create_font(-14, FW_NORMAL);

                let br_white = CreateSolidBrush(COL_WHITE);
                let br_bg = CreateSolidBrush(COL_BG);
                let br_div = CreateSolidBrush(COL_DIVIDER);
                let br_amber = CreateSolidBrush(COL_AMBER);
                let br_coral = CreateSolidBrush(COL_CORAL);

                let h_icon_big =
                    LoadImageW(h_inst, 1usize as *const u16, IMAGE_ICON, 48, 48, LR_DEFAULTCOLOR);

                let img_icon = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "",
                    0,
                    WS_VISIBLE | SS_ICON | SS_CENTERIMAGE,
                    22,
                    14,
                    48,
                    48,
                    0,
                    null_mut(),
                );
                if !h_icon_big.is_null() {
                    SendMessageW(img_icon, STM_SETICON, h_icon_big as WPARAM, 0);
                }

                let lbl_header_title = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "",
                    0,
                    WS_VISIBLE,
                    82,
                    16,
                    450,
                    26,
                    0,
                    f_title,
                );
                let lbl_header_sub = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "",
                    0,
                    WS_VISIBLE,
                    82,
                    44,
                    450,
                    22,
                    0,
                    f_sub,
                );

                // Welcome page
                let (welcome_title, welcome_body, default_dir) = STATE.with(|cell| {
                    let s = cell.borrow();
                    if s.is_uninstall {
                        (
                            "Conferma disinstallazione di Accent Markdown Editor".to_string(),
                            format!(
                                "Questa procedura guidata rimuoverà Accent Markdown Editor dal tuo profilo utente.\n\n\
                                 Cartella rilevata:\n{}\n\n\
                                 Elementi che verranno rimossi:\n\
                                 • Applicazione e disinstallatore nella cartella di installazione\n\
                                 • Collegamenti nel Menu Start e sul Desktop\n\
                                 • Associazioni file .md / .markdown e voce in 'App installate'\n\n\
                                 I tuoi documenti Markdown personali non verranno toccati.",
                                s.target_dir.display()
                            ),
                            s.target_dir.display().to_string(),
                        )
                    } else {
                        (
                            format!("Benvenuto in Accent Markdown Editor v{APP_VERSION}"),
                            "Questa procedura guidata ti accompagnerà nell'installazione di Accent sul tuo computer.\n\n\
                             • Editor e visualizzatore Markdown locale in puro Rust\n\
                             • Installazione nel tuo profilo utente: nessun privilegio amministrativo richiesto\n\
                             • Supporto offline per KaTeX, Mermaid ed esportazione PDF/DOCX/ODT/LaTeX/EPUB\n\
                             • Disinstallazione guidata integrata in 'App installate' di Windows\n\n\
                             Fai clic su «Avanti» per scegliere cartella e opzioni."
                                .to_string(),
                            s.target_dir.display().to_string(),
                        )
                    }
                });

                let lbl_welcome_title = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    &welcome_title,
                    0,
                    0,
                    28,
                    98,
                    496,
                    24,
                    0,
                    f_bold,
                );
                let lbl_welcome_body = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    &welcome_body,
                    0,
                    0,
                    28,
                    128,
                    496,
                    192,
                    0,
                    f_body,
                );

                // Options page
                let lbl_dir_caption = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "Cartella di destinazione (profilo utente, senza privilegi di amministratore):",
                    0,
                    0,
                    28,
                    98,
                    496,
                    20,
                    0,
                    f_bold,
                );
                let edit_dir = make_ctrl(
                    hwnd,
                    h_inst,
                    "EDIT",
                    &default_dir,
                    WS_EX_CLIENTEDGE,
                    WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL,
                    28,
                    124,
                    398,
                    26,
                    ID_EDIT_DIR,
                    f_body,
                );
                let btn_browse = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Sfoglia...",
                    0,
                    WS_TABSTOP | BS_PUSHBUTTON,
                    434,
                    123,
                    90,
                    28,
                    ID_BTN_BROWSE,
                    f_body,
                );

                let lbl_opts_caption = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "Opzioni di integrazione con Windows:",
                    0,
                    0,
                    28,
                    168,
                    496,
                    20,
                    0,
                    f_bold,
                );
                let chk_start_menu = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Crea collegamento nel Menu Start",
                    0,
                    WS_TABSTOP | BS_AUTOCHECKBOX,
                    34,
                    196,
                    470,
                    22,
                    ID_CHK_STARTMENU,
                    f_body,
                );
                let chk_desktop = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Crea collegamento sul Desktop",
                    0,
                    WS_TABSTOP | BS_AUTOCHECKBOX,
                    34,
                    224,
                    470,
                    22,
                    ID_CHK_DESKTOP,
                    f_body,
                );
                let chk_assoc = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Associa i file Markdown (.md, .markdown) ad Accent",
                    0,
                    WS_TABSTOP | BS_AUTOCHECKBOX,
                    34,
                    252,
                    470,
                    22,
                    ID_CHK_ASSOC,
                    f_body,
                );
                let chk_path = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Aggiungi Accent al PATH dell'utente corrente",
                    0,
                    WS_TABSTOP | BS_AUTOCHECKBOX,
                    34,
                    280,
                    470,
                    22,
                    ID_CHK_PATH,
                    f_body,
                );
                set_checked(chk_start_menu, true);
                set_checked(chk_desktop, true);
                set_checked(chk_assoc, true);
                set_checked(chk_path, true);

                // Progress page
                let lbl_progress_status = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "Preparazione...",
                    0,
                    0,
                    28,
                    154,
                    496,
                    24,
                    0,
                    f_body,
                );
                let progress_bar = make_ctrl(
                    hwnd,
                    h_inst,
                    "msctls_progress32",
                    "",
                    0,
                    0,
                    28,
                    186,
                    496,
                    24,
                    ID_PROGRESS,
                    null_mut(),
                );
                SendMessageW(progress_bar, PBM_SETRANGE, 0, (100 << 16) as LPARAM);
                SendMessageW(progress_bar, PBM_SETPOS, 5, 0);

                // Finish page
                let lbl_finish_title = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "",
                    0,
                    0,
                    28,
                    108,
                    496,
                    26,
                    0,
                    f_bold,
                );
                let lbl_finish_body = make_ctrl(
                    hwnd,
                    h_inst,
                    "STATIC",
                    "",
                    0,
                    0,
                    28,
                    142,
                    496,
                    110,
                    0,
                    f_body,
                );
                let chk_launch = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Avvia Accent Markdown Editor al termine",
                    0,
                    WS_TABSTOP | BS_AUTOCHECKBOX,
                    28,
                    264,
                    460,
                    24,
                    ID_CHK_LAUNCH,
                    f_body,
                );
                set_checked(chk_launch, true);

                // Footer buttons
                let btn_back = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "‹ Indietro",
                    0,
                    WS_TABSTOP | BS_PUSHBUTTON,
                    236,
                    336,
                    92,
                    30,
                    ID_BTN_BACK,
                    f_body,
                );
                let btn_next = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Avanti ›",
                    0,
                    WS_TABSTOP | BS_DEFPUSHBUTTON,
                    336,
                    336,
                    96,
                    30,
                    ID_BTN_NEXT,
                    f_body,
                );
                let btn_cancel = make_ctrl(
                    hwnd,
                    h_inst,
                    "BUTTON",
                    "Annulla",
                    0,
                    WS_TABSTOP | BS_PUSHBUTTON,
                    440,
                    336,
                    88,
                    30,
                    ID_BTN_CANCEL,
                    f_body,
                );

                STATE.with(|cell| {
                    let mut s = cell.borrow_mut();
                    s.h_font_title = f_title;
                    s.h_font_sub = f_sub;
                    s.h_font_body = f_body;
                    s.h_font_bold = f_bold;
                    s.h_brush_white = br_white;
                    s.h_brush_bg = br_bg;
                    s.h_brush_divider = br_div;
                    s.h_brush_amber = br_amber;
                    s.h_brush_coral = br_coral;
                    s.lbl_header_title = lbl_header_title;
                    s.lbl_header_sub = lbl_header_sub;
                    s.img_icon = img_icon;
                    s.lbl_welcome_title = lbl_welcome_title;
                    s.lbl_welcome_body = lbl_welcome_body;
                    s.lbl_dir_caption = lbl_dir_caption;
                    s.edit_dir = edit_dir;
                    s.btn_browse = btn_browse;
                    s.lbl_opts_caption = lbl_opts_caption;
                    s.chk_start_menu = chk_start_menu;
                    s.chk_desktop = chk_desktop;
                    s.chk_assoc = chk_assoc;
                    s.chk_path = chk_path;
                    s.lbl_progress_status = lbl_progress_status;
                    s.progress_bar = progress_bar;
                    s.lbl_finish_title = lbl_finish_title;
                    s.lbl_finish_body = lbl_finish_body;
                    s.chk_launch = chk_launch;
                    s.btn_back = btn_back;
                    s.btn_next = btn_next;
                    s.btn_cancel = btn_cancel;
                });
            }
            update_page_ui(hwnd);
            0
        }
        WM_PAINT => {
            unsafe {
                let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
                let hdc = BeginPaint(hwnd, &mut ps);
                STATE.with(|cell| {
                    let s = cell.borrow();
                    // Top white header banner
                    let rc_header = RECT { left: 0, top: 0, right: 600, bottom: 78 };
                    FillRect(hdc, &rc_header, s.h_brush_white);
                    // Amber-to-Coral accent bar under header
                    let rc_amber = RECT { left: 0, top: 76, right: 280, bottom: 79 };
                    let rc_coral = RECT { left: 280, top: 76, right: 600, bottom: 79 };
                    FillRect(hdc, &rc_amber, s.h_brush_amber);
                    FillRect(hdc, &rc_coral, s.h_brush_coral);
                    // Main warm alabaster body
                    let rc_body = RECT { left: 0, top: 79, right: 600, bottom: 322 };
                    FillRect(hdc, &rc_body, s.h_brush_bg);
                    // Footer divider + crisp white footer bar
                    let rc_div = RECT { left: 0, top: 322, right: 600, bottom: 323 };
                    FillRect(hdc, &rc_div, s.h_brush_divider);
                    let rc_foot = RECT { left: 0, top: 323, right: 600, bottom: 420 };
                    FillRect(hdc, &rc_foot, s.h_brush_white);
                });
                EndPaint(hwnd, &ps);
            }
            0
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            let hdc = w_param as HDC;
            let ctrl = l_param as HWND;
            STATE.with(|cell| {
                let s = cell.borrow();
                unsafe {
                    SetBkMode(hdc, TRANSPARENT);
                    if ctrl == s.lbl_header_title || ctrl == s.lbl_header_sub || ctrl == s.img_icon
                    {
                        SetTextColor(
                            hdc,
                            if ctrl == s.lbl_header_sub { COL_MUTED } else { COL_TEXT },
                        );
                        return s.h_brush_white as LRESULT;
                    }
                    SetTextColor(hdc, COL_TEXT);
                    s.h_brush_bg as LRESULT
                }
            })
        }
        WM_COMMAND => {
            let id = w_param & 0xFFFF;
            match id {
                ID_BTN_BROWSE => {
                    if let Some(chosen) =
                        browse_folder(hwnd, "Seleziona la cartella di installazione per Accent:")
                    {
                        let final_dir = if chosen
                            .file_name()
                            .map(|n| n.to_string_lossy().eq_ignore_ascii_case("accent"))
                            .unwrap_or(false)
                        {
                            chosen
                        } else {
                            chosen.join("Accent")
                        };
                        STATE.with(|cell| {
                            let mut s = cell.borrow_mut();
                            s.target_dir = final_dir.clone();
                            set_text(s.edit_dir, &final_dir.display().to_string());
                        });
                    }
                }
                ID_BTN_BACK => {
                    STATE.with(|cell| {
                        let mut s = cell.borrow_mut();
                        if s.page == WizardPage::Options {
                            s.page = WizardPage::Welcome;
                        }
                    });
                    update_page_ui(hwnd);
                }
                ID_BTN_NEXT => {
                    let action = STATE.with(|cell| {
                        let mut s = cell.borrow_mut();
                        match s.page {
                            WizardPage::Welcome => {
                                if s.is_uninstall {
                                    "start"
                                } else {
                                    s.page = WizardPage::Options;
                                    "refresh"
                                }
                            }
                            WizardPage::Options => "start",
                            WizardPage::Finish => "finish",
                            WizardPage::Progress => "none",
                        }
                    });
                    match action {
                        "refresh" => update_page_ui(hwnd),
                        "start" => start_worker(hwnd),
                        "finish" => {
                            let (launch_exe, cleanup_dir) = STATE.with(|cell| {
                                let s = cell.borrow();
                                let launch = if !s.is_uninstall
                                    && s.finished_ok
                                    && is_checked(s.chk_launch)
                                    && !s.installed_exe.as_os_str().is_empty()
                                {
                                    Some(s.installed_exe.clone())
                                } else {
                                    None
                                };
                                let cleanup = if s.is_uninstall && s.finished_ok {
                                    Some(s.target_dir.clone())
                                } else {
                                    None
                                };
                                (launch, cleanup)
                            });
                            if let Some(exe) = launch_exe {
                                let _ = Command::new(exe).spawn();
                            }
                            if let Some(dir) = cleanup_dir {
                                schedule_self_cleanup(&dir);
                            }
                            unsafe {
                                DestroyWindow(hwnd);
                            }
                        }
                        _ => {}
                    }
                }
                ID_BTN_CANCEL => unsafe {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                },
                _ => {}
            }
            0
        }
        WM_APP_PROGRESS => {
            let percent = w_param;
            let text = if l_param != 0 {
                unsafe { *Box::from_raw(l_param as *mut String) }
            } else {
                String::new()
            };
            STATE.with(|cell| {
                let s = cell.borrow();
                set_text(s.lbl_progress_status, &text);
                unsafe {
                    SendMessageW(s.progress_bar, PBM_SETPOS, percent, 0);
                }
            });
            0
        }
        WM_APP_DONE => {
            let res: Result<String, String> = if l_param != 0 {
                unsafe { *Box::from_raw(l_param as *mut Result<String, String>) }
            } else {
                Err("Errore sconosciuto".to_string())
            };
            STATE.with(|cell| {
                let mut s = cell.borrow_mut();
                s.busy = false;
                s.page = WizardPage::Finish;
                match res {
                    Ok(path_str) => {
                        s.finished_ok = true;
                        if s.is_uninstall {
                            set_text(
                                s.lbl_finish_title,
                                "Accent Markdown Editor è stato disinstallato",
                            );
                            set_text(
                                s.lbl_finish_body,
                                &format!(
                                    "Tutti i file dell'applicazione e le integrazioni di sistema sono stati rimossi.\n\n\
                                     Cartella rimossa:\n{path_str}\n\n\
                                     Fai clic su «Fine» per chiudere la procedura guidata."
                                ),
                            );
                        } else {
                            s.installed_exe = PathBuf::from(&path_str);
                            set_text(
                                s.lbl_finish_title,
                                &format!("Installazione di Accent v{APP_VERSION} completata!"),
                            );
                            set_text(
                                s.lbl_finish_body,
                                &format!(
                                    "Accent Markdown Editor è stato installato nel tuo profilo utente senza privilegi di amministratore.\n\n\
                                     Percorso eseguibile:\n{path_str}\n\n\
                                     Puoi disinstallarlo in qualsiasi momento da Impostazioni › App › App installate."
                                ),
                            );
                        }
                    }
                    Err(err) => {
                        s.finished_ok = false;
                        set_text(
                            s.lbl_finish_title,
                            if s.is_uninstall {
                                "Errore durante la disinstallazione"
                            } else {
                                "Errore durante l'installazione"
                            },
                        );
                        set_text(s.lbl_finish_body, &err);
                    }
                }
            });
            update_page_ui(hwnd);
            0
        }
        WM_CLOSE => {
            let (busy, finished_ok, is_uninstall, dir) = STATE.with(|cell| {
                let s = cell.borrow();
                (s.busy, s.finished_ok, s.is_uninstall, s.target_dir.clone())
            });
            if busy {
                return 0;
            }
            if !finished_ok {
                let confirm = msg_box(
                    hwnd,
                    "Accent Markdown Editor",
                    if is_uninstall {
                        "Vuoi annullare la disinstallazione di Accent Markdown Editor?"
                    } else {
                        "Vuoi annullare l'installazione di Accent Markdown Editor?"
                    },
                    MB_YESNO | MB_ICONQUESTION,
                );
                if confirm != IDYES {
                    return 0;
                }
            } else if is_uninstall {
                schedule_self_cleanup(&dir);
            }
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            STATE.with(|cell| {
                let s = cell.borrow();
                unsafe {
                    for obj in [
                        s.h_font_title as *mut c_void,
                        s.h_font_sub as *mut c_void,
                        s.h_font_body as *mut c_void,
                        s.h_font_bold as *mut c_void,
                        s.h_brush_white as *mut c_void,
                        s.h_brush_bg as *mut c_void,
                        s.h_brush_divider as *mut c_void,
                        s.h_brush_amber as *mut c_void,
                        s.h_brush_coral as *mut c_void,
                    ] {
                        if !obj.is_null() {
                            DeleteObject(obj);
                        }
                    }
                }
            });
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

fn run_wizard_gui(is_uninstall: bool) {
    STATE.with(|cell| {
        let mut s = cell.borrow_mut();
        s.is_uninstall = is_uninstall;
        s.target_dir = if is_uninstall {
            detect_installed_dir()
        } else {
            default_install_dir()
        };
    });

    unsafe {
        let _ = CoInitializeEx(null_mut(), 0x2); // COINIT_APARTMENTTHREADED
        let icc = INITCOMMONCONTROLSEX {
            dw_size: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dw_icc: ICC_PROGRESS_CLASS | ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&icc);

        let h_inst = GetModuleHandleW(null());
        let cls_name = to_wide("AccentSetupWizardWnd");
        let h_icon =
            LoadImageW(h_inst, 1usize as *const u16, IMAGE_ICON, 32, 32, LR_DEFAULTCOLOR) as HICON;
        let h_icon_sm =
            LoadImageW(h_inst, 1usize as *const u16, IMAGE_ICON, 16, 16, LR_DEFAULTCOLOR) as HICON;

        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: Some(wnd_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: h_inst,
            h_icon,
            h_cursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            hbr_background: null_mut(),
            lpsz_menu_name: null(),
            lpsz_class_name: cls_name.as_ptr(),
            h_icon_sm,
        };
        RegisterClassExW(&wc);

        let win_w = 568;
        let win_h = 414;
        let scr_w = GetSystemMetrics(SM_CXSCREEN);
        let scr_h = GetSystemMetrics(SM_CYSCREEN);
        let x = ((scr_w - win_w) / 2).max(20);
        let y = ((scr_h - win_h) / 2).max(20);

        let title = if is_uninstall {
            "Disinstallazione — Accent Markdown Editor".to_string()
        } else {
            format!("Installazione — Accent Markdown Editor v{APP_VERSION}")
        };
        let w_title = to_wide(&title);

        let hwnd = CreateWindowExW(
            0,
            cls_name.as_ptr(),
            w_title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x,
            y,
            win_w,
            win_h,
            null_mut(),
            null_mut(),
            h_inst,
            null_mut(),
        );
        if hwnd.is_null() {
            msg_box(
                null_mut(),
                "Errore — Accent Setup",
                "Impossibile inizializzare la finestra della procedura guidata.",
                MB_OK | MB_ICONERROR,
            );
            return;
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg = std::mem::zeroed::<MSG>();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
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

    if is_silent {
        let res = if is_uninstall {
            let dir = detect_installed_dir();
            let r = perform_uninstall(&dir, null_mut());
            if r.is_ok() {
                schedule_self_cleanup(&dir);
            }
            r
        } else {
            let opts = InstallOptions {
                dir: default_install_dir(),
                create_start_menu: true,
                create_desktop: true,
                associate_markdown: true,
                add_to_path: true,
            };
            perform_install(&opts, null_mut())
        };
        if let Err(err) = res {
            eprintln!("{err}");
            std::process::exit(1);
        }
        return;
    }

    run_wizard_gui(is_uninstall);
}
