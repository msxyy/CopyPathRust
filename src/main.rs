#![windows_subsystem = "windows"]
#![allow(non_snake_case)]
#![allow(clippy::upper_case_acronyms)]

use std::env;
use std::ffi::{c_void, OsString};
use std::os::windows::process::CommandExt;
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
const CREATE_NO_WINDOW: u32 = 0x08000000;

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

    let target = if first.contains('\\') || first.contains('/') {
        first.to_string()
    } else {
        let dir = Path::new(src).parent().ok_or("无法获取源文件目录")?;
        dir.join(first).display().to_string()
    };

    move_overwrite(src, &target)
}

// ---------- 多平台 ADB 配置 ----------

#[derive(Clone)]
struct Platform {
    key: String,
    display_name: String,
    adb_path: String,
    serial: String,
    paths: Vec<String>,
}

fn board_conf_path() -> Option<std::path::PathBuf> {
    env::current_exe()
        .ok()?
        .parent()
        .map(|p| p.join("board.conf"))
}

fn load_all_platforms() -> Vec<Platform> {
    let conf = match board_conf_path() {
        Some(p) if p.exists() => p,
        _ => return Vec::new(),
    };
    let content = match std::fs::read_to_string(&conf) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut platforms: Vec<Platform> = Vec::new();
    let mut current: Option<Platform> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(p) = current.take() {
                platforms.push(p);
            }
            let name = line[1..line.len() - 1].trim();
            current = Some(Platform {
                key: name.to_string(),
                display_name: name.to_string(),
                adb_path: "adb".into(),
                serial: String::new(),
                paths: Vec::new(),
            });
        } else if let (Some(p), Some((k, v))) = (&mut current, line.split_once('=')) {
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "name" => p.display_name = v.to_string(),
                "adb_path" => p.adb_path = v.to_string(),
                "serial" => p.serial = v.to_string(),
                "paths" => {
                    for part in v.split(';') {
                        let part = part.trim();
                        if !part.is_empty() {
                            p.paths.push(part.to_string());
                        }
                    }
                }
                "remote_dir" if !v.is_empty() => {
                    p.paths.push(v.to_string());
                }
                _ => {}
            }
        }
    }
    if let Some(p) = current.take() {
        platforms.push(p);
    }
    platforms.retain(|p| !p.paths.is_empty());
    platforms
}

fn load_platform(key: &str) -> Result<Platform, String> {
    load_all_platforms()
        .into_iter()
        .find(|p| p.key == key)
        .ok_or_else(|| format!("board.conf 中没有找到平台 [{key}]"))
}

// ---------- ADB 操作 ----------

fn run_adb(cfg: &Platform, args: &[&str]) -> Result<(), String> {
    let mut cmd = std::process::Command::new(&cfg.adb_path);
    if !cfg.serial.is_empty() {
        cmd.arg("-s").arg(&cfg.serial);
    }
    cmd.args(args);
    cmd.creation_flags(CREATE_NO_WINDOW);

    let out = cmd.output().map_err(|e| {
        format!(
            "执行 adb 失败：{e}\n\n\
             请确认 adb 在 PATH 中，或在 board.conf 中配置 adb_path 指向 adb.exe 完整路径。"
        )
    })?;
    if !out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let combined = format!("{}{}", stdout.trim(), stderr.trim());
        return Err(format!("adb 执行失败：\n\n{combined}"));
    }
    Ok(())
}

/// 执行 adb 并把 stdout+stderr 合并返回，不检查退出码
/// 用于需要"读取输出内容判断"的场景，例如 ls 验证目录
fn run_adb_check(cfg: &Platform, args: &[&str]) -> Result<String, String> {
    let mut cmd = std::process::Command::new(&cfg.adb_path);
    if !cfg.serial.is_empty() {
        cmd.arg("-s").arg(&cfg.serial);
    }
    cmd.args(args);
    cmd.creation_flags(CREATE_NO_WINDOW);

    let out = cmd.output().map_err(|e| {
        format!(
            "执行 adb 失败：{e}\n\n\
             请确认 adb 在 PATH 中，或在 board.conf 中配置 adb_path 指向 adb.exe 完整路径。"
        )
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    Ok(format!("{}{}", stdout, stderr))
}

/// 判断目标路径是否需要 root 才能写
/// /system 和 /vendor 是只读分区，必须 root + remount 才能写
fn needs_root(remote_dir: &str) -> bool {
    remote_dir.starts_with("/system") || remote_dir.starts_with("/vendor")
}

fn adb_push(platform_key: &str, remote_dir: &str, path: &str) -> Result<(), String> {
    let cfg = load_platform(platform_key)?;

    // 方案 B：只有 /system 和 /vendor 才自动 root + remount
    // 失败不阻塞——用户版固件可能不支持 root，或 remount 触发 dm-verity
    // 但不影响 /sdcard、/tmp 等本来就无需 root 的路径
    if needs_root(remote_dir) {
        let _ = run_adb_check(&cfg, &["root"]);
        let _ = run_adb_check(&cfg, &["remount"]);
    }

    // 推送前验证远程目录存在
    let check = run_adb_check(&cfg, &["shell", "ls", "-d", remote_dir])?;
    let low = check.to_lowercase();

    if low.contains("no such file") || low.contains("not found") {
        return Err(format!(
            "远程目录不存在：{remote_dir}\n\n\
             请检查 board.conf 中的路径是否正确，或先在板子上创建该目录。\n\n\
             adb 输出：{}",
            check.trim()
        ));
    }
    if low.contains("permission denied") {
        return Err(format!(
            "远程目录无读取权限：{remote_dir}\n\n\
             自动执行 adb root && adb remount 后仍然失败，可能原因：\n\
             1. 固件不支持 root（production build）\n\
             2. dm-verity 未禁用，remount 无效\n\
             3. 分区挂载为只读\n\n\
             可尝试手动执行：\n\
             adb root\n\
             adb disable-verity\n\
             adb reboot\n\
             adb root\n\
             adb remount\n\n\
             adb 输出：{}",
            check.trim()
        ));
    }

    run_adb(&cfg, &["push", path, remote_dir])
}

fn adb_pull(platform_key: &str, remote_dir: &str, path: &str) -> Result<(), String> {
    let cfg = load_platform(platform_key)?;
    let name = Path::new(path)
        .file_name()
        .ok_or("无效的本地路径")?
        .to_string_lossy()
        .into_owned();
    let remote = format!("{}/{}", remote_dir.trim_end_matches('/'), name);
    let parent = Path::new(path)
        .parent()
        .ok_or("无法获取本地目录")?
        .display()
        .to_string();
    run_adb(&cfg, &["pull", &remote, &parent])
}

fn adb_pull_dir(platform_key: &str, remote_dir: &str, dir: &str) -> Result<(), String> {
    let cfg = load_platform(platform_key)?;
    let remote = format!("{}/.", remote_dir.trim_end_matches('/'));
    run_adb(&cfg, &["pull", &remote, dir])
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

struct SubItem {
    key: String,
    display_name: String,
    command: String,
    children: Option<Vec<SubItem>>,
}

enum MenuKind {
    Command(String),
    Cascade(Vec<SubItem>),
}

struct MenuEntry {
    key: String,
    location: &'static str,
    name: String,
    kind: MenuKind,
    icon: String,
    model: Option<&'static str>,
}

fn entry_cmd(
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
        kind: MenuKind::Command(command),
        icon: icon.to_string(),
        model,
    }
}

fn entry_cascade(
    location: &'static str,
    key_suffix: &str,
    name: &str,
    subs: Vec<SubItem>,
    icon: &str,
    model: Option<&'static str>,
) -> MenuEntry {
    MenuEntry {
        key: format!("CopyPath.{}", key_suffix),
        location,
        name: name.to_string(),
        kind: MenuKind::Cascade(subs),
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
        v.push(entry_cmd(
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

    v.push(entry_cmd(
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

    v.push(entry_cmd(
        location,
        "07_cmd",
        "在此打开命令提示符",
        format!(r#"cmd.exe /s /k pushd "{}""#, placeholder),
        "cmd.exe,0",
        model,
    ));

    if let Some(c) = detect_cmder() {
        v.push(entry_cmd(
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

fn adb_cascade_entries(
    location: &'static str,
    key_suffix: &str,
    label: &str,
    action: &str,
    placeholder: &str,
    model: Option<&'static str>,
    platforms: &[Platform],
) -> Option<MenuEntry> {
    if platforms.is_empty() {
        return None;
    }
    let exe = current_exe_path().unwrap_or_default();
    let qexe = format!("\"{}\"", exe);

    let subs: Vec<SubItem> = platforms
        .iter()
        .map(|p| {
            let path_children: Vec<SubItem> = p
                .paths
                .iter()
                .enumerate()
                .map(|(i, dir)| SubItem {
                    key: format!("path_{}", i),
                    display_name: dir.clone(),
                    command: format!(
                        "{} {} \"{}\" \"{}\" \"{}\"",
                        qexe, action, p.key, dir, placeholder
                    ),
                    children: None,
                })
                .collect();

            if path_children.len() == 1 {
                let child = &path_children[0];
                SubItem {
                    key: p.key.clone(),
                    display_name: p.display_name.clone(),
                    command: child.command.clone(),
                    children: None,
                }
            } else {
                SubItem {
                    key: p.key.clone(),
                    display_name: p.display_name.clone(),
                    command: String::new(),
                    children: Some(path_children),
                }
            }
        })
        .collect();

    Some(entry_cascade(
        location,
        key_suffix,
        label,
        subs,
        "shell32.dll,167",
        model,
    ))
}

fn menu_entries() -> Vec<MenuEntry> {
    let exe = current_exe_path().unwrap_or_default();
    let qexe = format!("\"{}\"", exe);
    let copy_icon = "imageres.dll,-5302";
    let platforms = load_all_platforms();
    let mut v = Vec::new();

    for (sfx, name, mode) in [
        ("01_path", "复制路径", "--copy-path"),
        ("02_name", "复制文件名", "--copy-name"),
        ("03_dir", "复制所在目录", "--copy-dir"),
        ("04_quoted", "复制带引号路径", "--copy-quoted"),
    ] {
        v.push(entry_cmd(
            "*",
            sfx,
            name,
            format!("{} {} \"%1\"", qexe, mode),
            copy_icon,
            Some("Player"),
        ));
    }

    v.push(entry_cmd(
        "*",
        "05_replace",
        "替换为剪贴板中的文件",
        format!("{} --replace-clipboard \"%1\"", qexe),
        "imageres.dll,-5304",
        Some("Single"),
    ));

    if let Some(e) = adb_cascade_entries(
        "*",
        "06_adb_push",
        "推送到板子",
        "--adb-push",
        "%1",
        Some("Single"),
        &platforms,
    ) {
        v.push(e);
    }
    if let Some(e) = adb_cascade_entries(
        "*",
        "07_adb_pull",
        "从板子拉取",
        "--adb-pull",
        "%1",
        Some("Single"),
        &platforms,
    ) {
        v.push(e);
    }

    for (sfx, name, mode) in [
        ("01_path", "复制路径", "--copy-path"),
        ("02_name", "复制文件夹名", "--copy-name"),
        ("03_dir", "复制所在目录", "--copy-dir"),
        ("04_quoted", "复制带引号路径", "--copy-quoted"),
    ] {
        v.push(entry_cmd(
            "Directory",
            sfx,
            name,
            format!("{} {} \"%1\"", qexe, mode),
            copy_icon,
            Some("Player"),
        ));
    }
    v.extend(terminal_entries("Directory", "%1"));

    if let Some(e) = adb_cascade_entries(
        "Directory",
        "09_adb_push",
        "推送此文件夹到板子",
        "--adb-push",
        "%1",
        Some("Single"),
        &platforms,
    ) {
        v.push(e);
    }
    if let Some(e) = adb_cascade_entries(
        "Directory",
        "10_adb_pull",
        "从板子拉取到此文件夹",
        "--adb-pull-dir",
        "%1",
        Some("Single"),
        &platforms,
    ) {
        v.push(e);
    }

    for (sfx, name, mode) in [
        ("01_path", "复制当前路径", "--copy-path"),
        ("04_quoted", "复制当前路径(带引号)", "--copy-quoted"),
    ] {
        v.push(entry_cmd(
            "Directory\\Background",
            sfx,
            name,
            format!("{} {} \"%V\"", qexe, mode),
            copy_icon,
            None,
        ));
    }
    v.extend(terminal_entries("Directory\\Background", "%V"));

    if let Some(e) = adb_cascade_entries(
        "Directory\\Background",
        "09_adb_push",
        "推送当前文件夹到板子",
        "--adb-push",
        "%V",
        None,
        &platforms,
    ) {
        v.push(e);
    }
    if let Some(e) = adb_cascade_entries(
        "Directory\\Background",
        "10_adb_pull",
        "从板子拉取到当前文件夹",
        "--adb-pull-dir",
        "%V",
        None,
        &platforms,
    ) {
        v.push(e);
    }

    v
}

fn install_subitem(parent: &winreg::RegKey, sub: &SubItem) -> std::io::Result<()> {
    let (sub_key, _) = parent.create_subkey(&sub.key)?;
    sub_key.set_value("MUIVerb", &sub.display_name)?;
    let _ = sub_key.delete_value("");

    if let Some(children) = &sub.children {
        let _ = sub_key.delete_subkey_all("command");
        sub_key.set_value("SubCommands", &"")?;
        let (shell_key, _) = sub_key.create_subkey("shell")?;
        for child in children {
            install_subitem(&shell_key, child)?;
        }
    } else {
        let _ = sub_key.delete_subkey_all("shell");
        let _ = sub_key.delete_value("SubCommands");
        let (cmd_key, _) = sub_key.create_subkey("command")?;
        cmd_key.set_value("", &sub.command)?;
    }
    Ok(())
}

fn install() -> std::io::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let _ = uninstall();

    for e in menu_entries() {
        let path = format!(r"Software\Classes\{}\shell\{}", e.location, e.key);
        let (key, _) = hkcu.create_subkey(&path)?;
        key.set_value("Icon", &e.icon)?;

        let _ = key.delete_value("MultiSelectModel");
        if let Some(m) = e.model {
            key.set_value("MultiSelectModel", &m.to_string())?;
        }

        match &e.kind {
            MenuKind::Command(cmd) => {
                key.set_value("", &e.name)?;
                let _ = key.delete_value("MUIVerb");
                let _ = key.delete_value("SubCommands");
                let _ = key.delete_subkey_all("shell");
                let (cmd_key, _) = key.create_subkey("command")?;
                cmd_key.set_value("", cmd)?;
            }
            MenuKind::Cascade(subs) => {
                let _ = key.delete_value("");
                key.set_value("MUIVerb", &e.name)?;
                let _ = key.delete_subkey_all("command");
                key.set_value("SubCommands", &"")?;
                let (shell_key, _) = key.create_subkey("shell")?;
                for sub in subs {
                    install_subitem(&shell_key, sub)?;
                }
            }
        }
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

fn adb_status_lines() -> String {
    let platforms = load_all_platforms();
    if platforms.is_empty() {
        return "ADB 平台：❌ 未配置（需要在 exe 旁放 board.conf）\n".to_string();
    }
    let mut out = String::from("ADB 平台：\n");
    for p in platforms {
        let serial = if p.serial.is_empty() {
            "未指定 serial".to_string()
        } else {
            format!("serial={}", p.serial)
        };
        out.push_str(&format!("  {}（{}）：", p.display_name, serial));
        for (i, dir) in p.paths.iter().enumerate() {
            if i > 0 {
                out.push_str(" / ");
            }
            out.push_str(dir);
        }
        out.push('\n');
    }
    out
}

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

    let adb_lines = adb_status_lines();

    match registered_exe_path() {
        Some(p) if p.eq_ignore_ascii_case(&current) => format!(
            r#"状态：✅ 已安装

路径：{p}

检测到的终端：
{diag}
{adb_lines}
文件右键：复制路径 / 复制文件名 / 复制所在目录 / 复制带引号路径
　　　　　替换为剪贴板中的文件
　　　　　推送到板子 ▶ / 从板子拉取 ▶
文件夹右键：复制路径 / 复制文件夹名 / 复制所在目录 / 复制带引号路径
　　　　　在此打开 {terms}
　　　　　推送此文件夹到板子 ▶ / 从板子拉取到此文件夹 ▶
文件夹空白处右键：复制当前路径 / 复制当前路径(带引号)
　　　　　　　　在此打开 {terms}
　　　　　　　　推送当前文件夹到板子 ▶ / 从板子拉取到当前文件夹 ▶

ADB 配置示例（board.conf）：
  [ingenic]
  name=君正
  paths=/system/config/;/system/bin/;/system/lib/modules/;/system/init/;/system/etc/sensor/;/tmp/

  [rockchip]
  name=瑞芯微
  paths=/sdcard/;/vendor/etc/camera/rkisp2/;/vendor/etc/camera/;/vendor/bin/

  多个路径用分号分隔；修改 board.conf 后需要重新点「安装」才能刷新菜单。

推送到 /system 或 /vendor 时会自动执行 adb root && adb remount；
推送到 /sdcard、/tmp 等不需要 root 的路径不执行任何额外操作。
推送前会验证远程目录存在，不存在会弹错误提示，不会静默失败。

小技巧：按住 Shift 点「复制路径」=「复制带引号路径」。
Windows 11 用户如未见菜单项，请点击「显示更多选项」。"#
        ),
        Some(p) => format!(
            r#"状态：⚠️ 已安装，但 exe 已移动到新位置

旧路径：{p}
新路径：{current}

检测到的终端：
{diag}
{adb_lines}
菜单项会失效，点「安装」重新注册到当前位置。"#
        ),
        None => format!(
            r#"状态：❌ 未安装

当前路径：{current}

检测到的终端：
{diag}
{adb_lines}
点击「安装」将添加以下菜单项：
文件右键：复制路径 / 复制文件名 / 复制所在目录 / 复制带引号路径
　　　　　替换为剪贴板中的文件
　　　　　推送到板子 ▶ / 从板子拉取 ▶
文件夹右键：复制路径 / 复制文件夹名 / 复制所在目录 / 复制带引号路径
　　　　　在此打开 {terms}
　　　　　推送此文件夹到板子 ▶ / 从板子拉取到此文件夹 ▶
文件夹空白处右键：复制当前路径 / 复制当前路径(带引号)
　　　　　　　　在此打开 {terms}
　　　　　　　　推送当前文件夹到板子 ▶ / 从板子拉取到当前文件夹 ▶

ADB 配置（第一次使用必看）：
  在 exe 同目录新建 board.conf，示例：
  [ingenic]
  name=君正
  paths=/system/config/;/system/bin/;/system/lib/modules/;/system/init/;/system/etc/sensor/;/tmp/

  [rockchip]
  name=瑞芯微
  paths=/sdcard/;/vendor/etc/camera/rkisp2/;/vendor/etc/camera/;/vendor/bin/

  多个路径用分号分隔。修改后重新点「安装」即可生效。"#
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
                640,
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
                    680,
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
            bottom: 750,
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
        "--adb-push" | "--adb-pull" | "--adb-pull-dir" => {
            if args.len() < 4 {
                msg(
                    &format!(
                        "{} 参数不足。\n\n\
                         用法：<exe> {} <platform> <remote_dir> <path>\n\
                         通常由右键菜单自动调用，不需要手动执行。",
                        mode, mode
                    ),
                    "CopyPath",
                    MB_ICONERROR,
                );
                return;
            }
            let platform = args[1].to_string_lossy().into_owned();
            let remote_dir = args[2].to_string_lossy().into_owned();
            let path = args[3].to_string_lossy().into_owned();
            let res = match mode.as_str() {
                "--adb-push" => adb_push(&platform, &remote_dir, &path),
                "--adb-pull" => adb_pull(&platform, &remote_dir, &path),
                _ => adb_pull_dir(&platform, &remote_dir, &path),
            };
            if let Err(e) = res {
                msg(&e, "CopyPath - ADB 操作失败", MB_ICONERROR);
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
