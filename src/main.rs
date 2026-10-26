#![windows_subsystem = "windows"]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

use std::env;
use std::ffi::{c_void, OsString};
use std::path::Path;
use std::ptr;

type HANDLE = *mut c_void;
type HGLOBAL = *mut c_void;
type HWND = *mut c_void;
type HINSTANCE = *mut c_void;
type HICON = *mut c_void;
type HCURSOR = *mut c_void;
type HBRUSH = *mut c_void;
type HMENU = *mut c_void;

const GMEM_MOVEABLE: u32 = 0x0002;
const CF_UNICODETEXT: u32 = 13;

const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
const MOVEFILE_COPY_ALLOWED: u32 = 0x00000002;
const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
const INVALID_FILE_ATTRIBUTES: u32 = 0xFFFFFFFF;

const MB_OK: u32 = 0x0000;
const MB_ICONINFORMATION: u32 = 0x40;
const MB_ICONERROR: u32 = 0x10;

const WS_OVERLAPPED: u32 = 0x00000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_TABSTOP: u32 = 0x00010000;
const BS_PUSHBUTTON: u32 = 0x00000000;
const SS_LEFT: u32 = 0x00000000;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;
const WM_SETFONT: u32 = 0x0030;
const WM_CLOSE: u32 = 0x0010;

const SW_SHOW: i32 = 5;
const DEFAULT_GUI_FONT: i32 = 17;
const VK_SHIFT: i32 = 0x10;
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;

const ID_STATUS: isize = 100;
const ID_BTN_INSTALL: isize = 101;
const ID_BTN_UNINSTALL: isize = 102;
const ID_BTN_CLOSE: isize = 103;

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct WNDCLASSW {
    style: u32,
    lpfnWndProc: unsafe extern "system" fn(HWND, u32, usize, isize) -> isize,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
    lPrivate: u32,
}

#[link(name = "user32")]
extern "system" {
    fn OpenClipboard(hwnd: HWND) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(uformat: u32, hmem: HANDLE) -> HANDLE;
    fn GetClipboardData(uformat: u32) -> HANDLE;
    fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, utype: u32) -> i32;
    fn RegisterClassW(lpwndclass: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn ShowWindow(hwnd: HWND, cmd: i32) -> i32;
    fn UpdateWindow(hwnd: HWND) -> i32;
    fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const MSG) -> i32;
    fn DispatchMessageW(msg: *const MSG) -> isize;
    fn PostQuitMessage(code: i32);
    fn DestroyWindow(hwnd: HWND) -> i32;
    fn SendMessageW(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn SetWindowTextW(hwnd: HWND, text: *const u16) -> i32;
    fn GetDlgItem(hwnd: HWND, id: i32) -> HWND;
    fn GetAsyncKeyState(vkey: i32) -> i16;
    fn LoadCursorW(inst: HINSTANCE, name: *const u16) -> HCURSOR;
    fn GetStockObject(idx: i32) -> HANDLE;
    fn AdjustWindowRect(lprect: *mut RECT, dwstyle: u32, bmenu: i32) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(uflags: u32, dwbytes: usize) -> HGLOBAL;
    fn GlobalLock(hmem: HGLOBAL) -> *mut c_void;
    fn GlobalUnlock(hmem: HGLOBAL) -> i32;
    fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL;
    fn Sleep(dwms: u32);
    fn GetModuleHandleW(name: *const u16) -> HINSTANCE;
    fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    fn GetFileAttributesW(name: *const u16) -> u32;
    fn SetFileAttributesW(name: *const u16, attrs: u32) -> i32;
}

// ---------- 通用工具 ----------

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn msg(text: &str, caption: &str, flags: u32) {
    let t = wide(text);
    let c = wide(caption);
    unsafe {
        MessageBoxW(ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_OK | flags);
    }
}

fn is_shift_down() -> bool {
    unsafe { GetAsyncKeyState(VK_SHIFT) < 0 }
}

// ---------- 剪贴板 ----------

fn set_clipboard_text(text: &str) -> bool {
    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = utf16.len() * 2;
    for _ in 0..5 {
        unsafe {
            if OpenClipboard(ptr::null_mut()) != 0 {
                let _ = EmptyClipboard();
                let h = GlobalAlloc(GMEM_MOVEABLE, bytes);
                if !h.is_null() {
                    let p = GlobalLock(h);
                    if !p.is_null() {
                        ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, p as *mut u8, bytes);
                        GlobalUnlock(h);
                        if !SetClipboardData(CF_UNICODETEXT, h).is_null() {
                            CloseClipboard();
                            return true;
                        }
                    }
                    GlobalFree(h);
                }
                CloseClipboard();
            }
            Sleep(100);
        }
    }
    false
}

fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(ptr::null_mut()) == 0 {
            return None;
        }
        let h = GetClipboardData(CF_UNICODETEXT);
        if h.is_null() {
            CloseClipboard();
            return None;
        }
        let p = GlobalLock(h);
        if p.is_null() {
            CloseClipboard();
            return None;
        }
        let data = p as *const u16;
        let mut len = 0usize;
        while *data.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(data, len);
        let text = String::from_utf16_lossy(slice);
        GlobalUnlock(h);
        CloseClipboard();
        Some(text)
    }
}

// ---------- 路径处理 ----------

fn get_file_name(p: &str) -> String {
    Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string())
}

fn get_parent_dir(p: &str) -> String {
    Path::new(p)
        .parent()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn quote(p: &str) -> String {
    format!("\"{}\"", p)
}

// ---------- 文件替换 ----------

fn clear_readonly(path: &str) {
    unsafe {
        let attrs = GetFileAttributesW(wide(path).as_ptr());
        if attrs != INVALID_FILE_ATTRIBUTES && (attrs & FILE_ATTRIBUTE_READONLY) != 0 {
            SetFileAttributesW(wide(path).as_ptr(), attrs & !FILE_ATTRIBUTE_READONLY);
        }
    }
}

fn move_overwrite(src: &str, target: &str) -> Result<(), String> {
    if src.eq_ignore_ascii_case(target) {
        return Err("两个文件相同，无法覆盖。".into());
    }
    if !Path::new(target).exists() {
        return Err(format!("目标文件不存在：{target}"));
    }
    clear_readonly(target);
    let ok = unsafe {
        MoveFileExW(
            wide(src).as_ptr(),
            wide(target).as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_COPY_ALLOWED,
        ) != 0
    };
    if ok {
        Ok(())
    } else {
        Err(format!("覆盖失败：{}", std::io::Error::last_os_error()))
    }
}

fn replace_with_clipboard(src: &str) -> Result<(), String> {
    let clip = get_clipboard_text().ok_or("剪贴板为空")?;
    let raw = clip.trim().trim_matches('"');
    if raw.is_empty() {
        return Err("剪贴板内容为空".into());
    }
    let first = raw.lines().next().unwrap_or("").trim();
    if first.is_empty() {
        return Err("剪贴板内容为空".into());
    }

    // 完整路径直接用；纯文件名按 src 同目录解析
    let target = if first.contains('\\') || first.contains('/') {
        first.to_string()
    } else {
        let dir = Path::new(src).parent().ok_or("无法获取源文件目录")?;
        dir.join(first).display().to_string()
    };

    move_overwrite(src, &target)
}

// ---------- 终端检测 ----------

fn detect_pwsh() -> Option<String> {
    let mut candidates: Vec<String> = vec![
        r"C:\Program Files\PowerShell\7\pwsh.exe".to_string(),
        r"C:\Program Files\PowerShell\7-preview\pwsh.exe".to_string(),
    ];
    if let Ok(paths) = env::var("PATH") {
        for raw in paths.split(';') {
            let p = raw.trim_matches('"');
            if p.is_empty() {
                continue;
            }
            let full = Path::new(p).join("pwsh.exe");
            if full.is_file() {
                candidates.push(full.display().to_string());
            }
        }
    }
    candidates.into_iter().find(|c| Path::new(c).is_file())
}

fn detect_cmder() -> Option<String> {
    let mut candidates: Vec<String> = vec![
        r"C:\cmder\Cmder.exe".into(),
        r"C:\tools\cmder\Cmder.exe".into(),
        r"D:\cmder\Cmder.exe".into(),
        r"D:\Tools\cmder\Cmder.exe".into(),
    ];

    if let Ok(paths) = env::var("PATH") {
        for raw in paths.split(';') {
            let p = raw.trim_matches('"');
            if p.is_empty() {
                continue;
            }
            let exe = Path::new(p).join("Cmder.exe");
            if exe.is_file() {
                candidates.insert(0, exe.display().to_string());
            }
        }
    }

    if let Ok(exe) = env::var("CMDER_EXE") {
        candidates.insert(0, exe.trim_matches('"').to_string());
    }
    if let Ok(root) = env::var("CMDER_ROOT") {
        let root = root.trim_matches('"');
        let rp = Path::new(root);
        candidates.insert(0, rp.join("Cmder.exe").display().to_string());
        candidates.insert(0, rp.join("bin").join("Cmder.exe").display().to_string());
    }

    for (k, v) in env::vars() {
        if !k.to_ascii_uppercase().contains("CMDER") {
            continue;
        }
        let v = v.trim_matches('"');
        if v.is_empty() {
            continue;
        }
        let p = Path::new(v);
        if p.is_file() && v.to_ascii_lowercase().ends_with("cmder.exe") {
            candidates.insert(0, v.to_string());
        } else if p.is_dir() {
            candidates.insert(0, p.join("Cmder.exe").display().to_string());
            candidates.insert(0, p.join("bin").join("Cmder.exe").display().to_string());
        }
    }

    if let Ok(user) = env::var("USERPROFILE") {
        candidates.push(
            Path::new(&user)
                .join("cmder")
                .join("Cmder.exe")
                .display()
                .to_string(),
        );
    }

    candidates.into_iter().find(|c| Path::new(c).is_file())
}

// ---------- 注册表 ----------

fn current_exe_path() -> Option<String> {
    env::current_exe().ok().map(|p| p.display().to_string())
}

fn registered_exe_path() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(r"Software\Classes\*\shell\CopyPath.01_path\command")
        .ok()?;
    let cmd: String = key.get_value("").ok()?;
    let start = cmd.find('"')? + 1;
    let end = cmd[start..].find('"')? + start;
    Some(cmd[start..end].to_string())
}

struct MenuEntry {
    key: String,
    location: &'static str,
    name: String,
    command: String,
    icon: String,
    // None = 不写 MultiSelectModel（背景位置）
    // Some("Player"/"Single") = 写入对应值
    model: Option<&'static str>,
}

fn entry(
    location: &'static str,
    key_suffix: &str,
    name: &str,
    command: String,
    icon: &str,
    model: Option<&'static str>,
) -> MenuEntry {
    MenuEntry {
        key: format!("CopyPath.{}", key_suffix),
        location,
        name: name.to_string(),
        command,
        icon: icon.to_string(),
        model,
    }
}

fn is_background_location(location: &str) -> bool {
    location == "Directory\\Background" || location == "DesktopBackground"
}

fn terminal_entries(location: &'static str, placeholder: &str) -> Vec<MenuEntry> {
    let mut v = Vec::new();
    let model = if is_background_location(location) {
        None
    } else {
        Some("Single")
    };

    if let Some(p) = detect_pwsh() {
        v.push(entry(
            location,
            "05_pwsh",
            "在此打开 PowerShell 7",
            format!(
                r#""{}" -NoExit -Command "Set-Location -LiteralPath '{}'""#,
                p, placeholder
            ),
            &format!("\"{}\",0", p),
            model,
        ));
    }

    v.push(entry(
        location,
        "06_ps",
        "在此打开 Windows PowerShell",
        format!(
            r#"powershell.exe -NoExit -Command "Set-Location -LiteralPath '{}'""#,
            placeholder
        ),
        "powershell.exe,0",
        model,
    ));

    v.push(entry(
        location,
        "07_cmd",
        "在此打开命令提示符",
        format!(r#"cmd.exe /s /k pushd "{}""#, placeholder),
        "cmd.exe,0",
        model,
    ));

    if let Some(c) = detect_cmder() {
        v.push(entry(
            location,
            "08_cmder",
            "在此打开 Cmder",
            format!(r#""{}" /START "{}""#, c, placeholder),
            &format!("\"{}\",0", c),
            model,
        ));
    }

    v
}

fn menu_entries() -> Vec<MenuEntry> {
    let exe = current_exe_path().unwrap_or_default();
    let qexe = format!("\"{}\"", exe);
    let copy_icon = "imageres.dll,-5302";
    let mut v = Vec::new();

    // 文件右键：复制类
    for (sfx, name, mode) in [
        ("01_path", "复制路径", "--copy-path"),
        ("02_name", "复制文件名", "--copy-name"),
        ("03_dir", "复制所在目录", "--copy-dir"),
        ("04_quoted", "复制带引号路径", "--copy-quoted"),
    ] {
        v.push(entry(
            "*",
            sfx,
            name,
            format!("{} {} \"%1\"", qexe, mode),
            copy_icon,
            Some("Player"),
        ));
    }

    // 文件右键：替换为剪贴板中的文件（单选）
    v.push(entry(
        "*",
        "05_replace",
        "替换为剪贴板中的文件",
        format!("{} --replace-clipboard \"%1\"", qexe),
        "imageres.dll,-5304",
        Some("Single"),
    ));

    // 文件夹右键：复制类
    for (sfx, name, mode) in [
        ("01_path", "复制路径", "--copy-path"),
        ("02_name", "复制文件夹名", "--copy-name"),
        ("03_dir", "复制所在目录", "--copy-dir"),
        ("04_quoted", "复制带引号路径", "--copy-quoted"),
    ] {
        v.push(entry(
            "Directory",
            sfx,
            name,
            format!("{} {} \"%1\"", qexe, mode),
            copy_icon,
            Some("Player"),
        ));
    }
    v.extend(terminal_entries("Directory", "%1"));

    // 文件夹空白处
    for (sfx, name, mode) in [
        ("01_path", "复制当前路径", "--copy-path"),
        ("04_quoted", "复制当前路径(带引号)", "--copy-quoted"),
    ] {
        v.push(entry(
            "Directory\\Background",
            sfx,
            name,
            format!("{} {} \"%V\"", qexe, mode),
            copy_icon,
            None,
        ));
    }
    v.extend(terminal_entries("Directory\\Background", "%V"));

    v
}

fn install() -> std::io::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for old in [
        r"Software\Classes\*\shell\CopyFilePath",
        r"Software\Classes\Directory\shell\CopyFolderPath",
        r"Software\Classes\Directory\Background\shell\CopyCurrentPath",
    ] {
        let _ = hkcu.delete_subkey_all(old);
    }

    for e in menu_entries() {
        let path = format!(r"Software\Classes\{}\shell\{}", e.location, e.key);
        let (key, _) = hkcu.create_subkey(&path)?;
        key.set_value("", &e.name)?;
        key.set_value("Icon", &e.icon)?;

        let _ = key.delete_value("MultiSelectModel");
        if let Some(m) = e.model {
            key.set_value("MultiSelectModel", &m.to_string())?;
        }

        let (cmd_key, _) = key.create_subkey("command")?;
        cmd_key.set_value("", &e.command)?;
    }
    Ok(())
}

fn uninstall() -> std::io::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for location in [
        "*",
        "Directory",
        "Directory\\Background",
        "DesktopBackground",
    ] {
        let base = format!(r"Software\Classes\{}\shell", location);
        if let Ok(key) = hkcu.open_subkey_with_flags(&base, KEY_READ) {
            let names: Vec<String> = key.enum_keys().filter_map(|r| r.ok()).collect();
            for n in names {
                if n.starts_with("CopyPath.") {
                    let _ = hkcu.delete_subkey_all(format!(r"{}\{}", base, n));
                }
            }
        }
    }

    for old in [
        r"Software\Classes\*\shell\CopyFilePath",
        r"Software\Classes\Directory\shell\CopyFolderPath",
        r"Software\Classes\Directory\Background\shell\CopyCurrentPath",
    ] {
        let _ = hkcu.delete_subkey_all(old);
    }
    Ok(())
}

// ---------- GUI ----------

fn get_status_text() -> String {
    let current = current_exe_path().unwrap_or_default();

    let pwsh = detect_pwsh();
    let cmder = detect_cmder();

    let mut terms = Vec::new();
    if pwsh.is_some() {
        terms.push("PowerShell 7");
    }
    terms.push("Windows PowerShell");
    terms.push("命令提示符");
    if cmder.is_some() {
        terms.push("Cmder");
    }
    let terms = terms.join(" / ");

    let mut diag = String::new();
    if let Some(p) = &pwsh {
        diag.push_str(&format!("  PowerShell 7 → {p}\n"));
    }
    if let Some(p) = &cmder {
        diag.push_str(&format!("  Cmder        → {p}\n"));
    }
    if diag.is_empty() {
        diag.push_str("  （未检测到可选的 PowerShell 7 / Cmder）\n");
    }

    match registered_exe_path() {
        Some(p) if p.eq_ignore_ascii_case(&current) => format!(
            r#"状态：✅ 已安装

路径：{p}

检测到的终端：
{diag}
文件右键：复制路径 / 复制文件名 / 复制所在目录 / 复制带引号路径
　　　　　替换为剪贴板中的文件
文件夹右键：复制路径 / 复制文件夹名 / 复制所在目录 / 复制带引号路径
　　　　　在此打开 {terms}
文件夹空白处右键：复制当前路径 / 复制当前路径(带引号)
　　　　　　　　在此打开 {terms}

替换用法：
  跨目录：右键旧文件 → 复制路径；右键新文件 → 替换为剪贴板中的文件
  同目录：右键旧文件 → 复制文件名；右键新文件 → 替换为剪贴板中的文件

小技巧：按住 Shift 点「复制路径」=「复制带引号路径」。
Windows 11 用户如未见菜单项，请点击「显示更多选项」。"#
        ),
        Some(p) => format!(
            r#"状态：⚠️ 已安装，但 exe 已移动到新位置

旧路径：{p}
新路径：{current}

检测到的终端：
{diag}
菜单项会失效，点「安装」重新注册到当前位置。"#
        ),
        None => format!(
            r#"状态：❌ 未安装

当前路径：{current}

检测到的终端：
{diag}
点击「安装」将添加以下菜单项：
文件右键：复制路径 / 复制文件名 / 复制所在目录 / 复制带引号路径
　　　　　替换为剪贴板中的文件
文件夹右键：复制路径 / 复制文件夹名 / 复制所在目录 / 复制带引号路径
　　　　　在此打开 {terms}
文件夹空白处右键：复制当前路径 / 复制当前路径(带引号)
　　　　　　　　在此打开 {terms}

PowerShell 7 与 Cmder 为自动检测：未安装则不注册该项。"#
        ),
    }
}

unsafe fn refresh_status(hwnd: HWND) {
    let label = GetDlgItem(hwnd, ID_STATUS as i32);
    if !label.is_null() {
        let t = wide(&get_status_text());
        SetWindowTextW(label, t.as_ptr());
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(ptr::null());
            let font = GetStockObject(DEFAULT_GUI_FONT);

            let status_text = wide(&get_status_text());
            let status = CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                status_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | SS_LEFT,
                20,
                20,
                760,
                480,
                hwnd,
                ID_STATUS as HMENU,
                hinst,
                ptr::null_mut(),
            );
            SendMessageW(status, WM_SETFONT, font as usize, 1);

            for (id, text, x) in [
                (ID_BTN_INSTALL, "安装", 20),
                (ID_BTN_UNINSTALL, "卸载", 160),
                (ID_BTN_CLOSE, "关闭", 300),
            ] {
                let t = wide(text);
                let btn = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    t.as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON,
                    x,
                    520,
                    120,
                    34,
                    hwnd,
                    id as HMENU,
                    hinst,
                    ptr::null_mut(),
                );
                SendMessageW(btn, WM_SETFONT, font as usize, 1);
            }
            0
        }
        WM_COMMAND => {
            match (wparam & 0xFFFF) as isize {
                ID_BTN_INSTALL => {
                    let _ = install();
                    refresh_status(hwnd);
                }
                ID_BTN_UNINSTALL => {
                    let _ = uninstall();
                    refresh_status(hwnd);
                }
                ID_BTN_CLOSE => {
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn run_gui() {
    unsafe {
        let hinst = GetModuleHandleW(ptr::null());
        let class_name = wide("CopyPathInstallerWindow");
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: wndproc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: ptr::null_mut(),
            hCursor: LoadCursorW(ptr::null_mut(), 32512usize as *const u16),
            hbrBackground: 16usize as HBRUSH,
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: 800,
            bottom: 590,
        };
        AdjustWindowRect(&mut rc, style, 0);

        let title = wide("CopyPath - 右键复制路径助手");
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            style | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rc.right - rc.left,
            rc.bottom - rc.top,
            ptr::null_mut(),
            ptr::null_mut(),
            hinst,
            ptr::null_mut(),
        );
        if hwnd.is_null() {
            return;
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut m: MSG = std::mem::zeroed();
        loop {
            let r = GetMessageW(&mut m, ptr::null_mut(), 0, 0);
            if r <= 0 {
                break;
            }
            TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

// ---------- 入口 ----------

fn main() {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    if args.is_empty() {
        run_gui();
        return;
    }

    let mode = match args[0].to_str() {
        Some(s) => s.to_string(),
        None => return,
    };

    match mode.as_str() {
        "--install" => {
            match install() {
                Ok(_) => msg(
                    "安装成功！\n\nWindows 11 用户请点击「显示更多选项」查看菜单。",
                    "CopyPath",
                    MB_ICONINFORMATION,
                ),
                Err(e) => msg(&format!("安装失败：{e}"), "CopyPath", MB_ICONERROR),
            }
            return;
        }
        "--uninstall" => {
            match uninstall() {
                Ok(_) => msg("已卸载右键菜单项。", "CopyPath", MB_ICONINFORMATION),
                Err(e) => msg(&format!("卸载失败：{e}"), "CopyPath", MB_ICONERROR),
            }
            return;
        }
        "--replace-clipboard" => {
            if let Some(path) = args.get(1) {
                let src = path.to_string_lossy().into_owned();
                if let Err(e) = replace_with_clipboard(&src) {
                    msg(&e, "CopyPath - 替换失败", MB_ICONERROR);
                }
            }
            return;
        }
        _ => {}
    }

    let paths: Vec<String> = args
        .iter()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .collect();
    if paths.is_empty() {
        return;
    }

    let shift = is_shift_down();
    let text = match mode.as_str() {
        "--copy-path" => {
            if shift {
                paths
                    .iter()
                    .map(|p| quote(p))
                    .collect::<Vec<_>>()
                    .join("\r\n")
            } else {
                paths.join("\r\n")
            }
        }
        "--copy-quoted" => paths
            .iter()
            .map(|p| quote(p))
            .collect::<Vec<_>>()
            .join("\r\n"),
        "--copy-name" => paths
            .iter()
            .map(|p| get_file_name(p))
            .collect::<Vec<_>>()
            .join("\r\n"),
        "--copy-dir" => paths
            .iter()
            .map(|p| get_parent_dir(p))
            .collect::<Vec<_>>()
            .join("\r\n"),
        _ => return,
    };
    set_clipboard_text(&text);
}
