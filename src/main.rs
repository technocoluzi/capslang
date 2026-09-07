//! CapsLang — switch keyboard input language with the Caps Lock key.
//!
//! A hidden window owns a low-level keyboard hook and a notification-area
//! icon. The hook swallows Caps Lock and posts a message; the window pumps
//! that message and asks Windows for the next input language.

#![windows_subsystem = "windows"]

mod autostart;
mod config;
mod hook;
mod switching;
mod tray;

use config::{Config, Method};
use std::cell::RefCell;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub const WM_APP_TRAY: u32 = WM_APP + 1;
pub const WM_APP_SWITCH: u32 = WM_APP + 2;

const CLASS_NAME: &str = "CapsLangMainWindow";
const APP_NAME: &str = "CapsLang";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_URL: &str = "https://github.com/technocoluzi/capslang";

const ID_ENABLED: usize = 1;
const ID_AUTOSTART: usize = 2;
const ID_OPEN_CONFIG: usize = 3;
const ID_RELOAD: usize = 4;
const ID_ABOUT: usize = 5;
const ID_PROJECT: usize = 6;
const ID_EXIT: usize = 7;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct App {
    config: Config,
    tray: tray::Tray,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        std::process::exit(run_cli(&args));
    }
    if already_running() {
        // A second launch (installer, Startup entry, double-click) is a no-op.
        return;
    }
    run_app();
}

// ---------------------------------------------------------------------------
// Command line
// ---------------------------------------------------------------------------

const USAGE: &str = "CapsLang — switch keyboard input language with the Caps Lock key.

USAGE:
    capslang                  Start CapsLang (or do nothing if already running)
    capslang --quit           Stop the running instance
    capslang --autostart on   Start CapsLang when you sign in
    capslang --autostart off  Stop starting at sign-in
    capslang --config         Open the configuration file
    capslang --status         Report whether CapsLang is running
    capslang --version        Print the version
    capslang --help           Show this help";

fn run_cli(args: &[String]) -> i32 {
    match args[0].as_str() {
        "--help" | "-h" | "/?" => {
            report(USAGE);
            0
        }
        "--version" | "-V" => {
            report(&format!("{APP_NAME} {VERSION}"));
            0
        }
        "--status" => {
            let running = find_instance().is_some();
            report(if running {
                "CapsLang is running."
            } else {
                "CapsLang is not running."
            });
            i32::from(!running)
        }
        "--quit" => match find_instance() {
            Some(hwnd) => {
                unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) };
                0
            }
            None => {
                report("CapsLang is not running.");
                1
            }
        },
        "--config" => {
            let _ = Config::load(); // creates the file on first run
            match config::path() {
                Some(p) => {
                    open_path(&p.to_string_lossy());
                    0
                }
                None => {
                    report("Could not locate %APPDATA%.");
                    1
                }
            }
        }
        "--autostart" => match args.get(1).map(String::as_str) {
            Some("on") | Some("true") => set_autostart(true),
            Some("off") | Some("false") => set_autostart(false),
            _ => {
                report("Usage: capslang --autostart <on|off>");
                2
            }
        },
        other => {
            report(&format!("Unknown option: {other}\n\n{USAGE}"));
            2
        }
    }
}

fn set_autostart(on: bool) -> i32 {
    match autostart::set(on) {
        Ok(()) => {
            report(if on {
                "CapsLang will start when you sign in."
            } else {
                "CapsLang will no longer start when you sign in."
            });
            0
        }
        Err(e) => {
            report(&format!("Could not change the startup setting: {e}"));
            1
        }
    }
}

/// Prints to the console we were launched from, falling back to a dialog when
/// there is none (double-clicked from Explorer).
fn report(text: &str) {
    use std::io::Write;
    let attached = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) != 0 };
    if attached {
        if let Ok(mut out) = std::fs::OpenOptions::new().write(true).open("CONOUT$") {
            if writeln!(out, "{text}").is_ok() {
                return;
            }
        }
    }
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide(APP_NAME).as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn open_path(path: &str) {
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            wide("open").as_ptr(),
            wide(path).as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL as i32,
        );
    }
}

fn find_instance() -> Option<HWND> {
    let hwnd = unsafe { FindWindowW(wide(CLASS_NAME).as_ptr(), std::ptr::null()) };
    (!hwnd.is_null()).then_some(hwnd)
}

/// A named mutex is the reliable single-instance check: it exists from the
/// moment the process starts, before any window has been created.
fn already_running() -> bool {
    const ERROR_ALREADY_EXISTS: u32 = 183;
    unsafe {
        let name = wide("Local\\CapsLang.SingleInstance");
        let handle = CreateMutexW(std::ptr::null(), 1, name.as_ptr());
        handle.is_null() || windows_sys::Win32::Foundation::GetLastError() == ERROR_ALREADY_EXISTS
    }
}

// ---------------------------------------------------------------------------
// The app
// ---------------------------------------------------------------------------

fn run_app() {
    let config = Config::load();
    hook::set_passthrough(config.passthrough);
    hook::set_enabled(config.enabled);

    let hwnd = match create_window() {
        Some(h) => h,
        None => {
            report("CapsLang could not create its message window.");
            return;
        }
    };

    if !hook::install(hwnd) {
        report(
            "CapsLang could not install its keyboard hook.\n\nAnother program may already own the Caps Lock key.",
        );
        return;
    }

    let size = unsafe { GetSystemMetrics(SM_CXSMICON) };
    let mut tray = tray::Tray::new(hwnd, size);
    if config.show_tray_icon {
        tray.show(config.enabled, &tooltip(config.enabled));
    }
    APP.with(|a| *a.borrow_mut() = Some(App { config, tray }));

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    hook::uninstall();
    APP.with(|a| a.borrow_mut().take());
}

fn tooltip(enabled: bool) -> String {
    if enabled {
        format!("{APP_NAME} {VERSION}\nCaps Lock switches language")
    } else {
        format!("{APP_NAME} {VERSION}\nPaused")
    }
}

fn create_window() -> Option<HWND> {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = wide(CLASS_NAME);
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(wnd_proc);
        wc.hInstance = instance;
        wc.lpszClassName = class.as_ptr();
        if RegisterClassW(&wc) == 0 {
            return None;
        }
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            wide(APP_NAME).as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        (!hwnd.is_null()).then_some(hwnd)
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Explorer sends this after it restarts; the icon has to be re-added.
    let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());

    match msg {
        WM_APP_SWITCH => {
            let method = APP.with(|a| {
                a.borrow()
                    .as_ref()
                    .map(|app| app.config.method)
                    .unwrap_or(Method::Auto)
            });
            switching::switch(method);
            0
        }
        WM_APP_TRAY => {
            match lparam as u32 {
                WM_RBUTTONUP | WM_CONTEXTMENU => show_menu(hwnd),
                WM_LBUTTONDBLCLK => toggle_enabled(),
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
        m if m == taskbar_created => {
            let enabled = hook::is_enabled();
            APP.with(|a| {
                if let Some(app) = a.borrow_mut().as_mut() {
                    if app.config.show_tray_icon {
                        app.tray.show(enabled, &tooltip(enabled));
                    }
                }
            });
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn refresh_tray() {
    let enabled = hook::is_enabled();
    APP.with(|a| {
        if let Some(app) = a.borrow_mut().as_mut() {
            if app.config.show_tray_icon {
                app.tray.show(enabled, &tooltip(enabled));
            }
        }
    });
}

fn toggle_enabled() {
    let enabled = !hook::is_enabled();
    hook::set_enabled(enabled);
    APP.with(|a| {
        if let Some(app) = a.borrow_mut().as_mut() {
            app.config.enabled = enabled;
            let _ = app.config.save();
        }
    });
    refresh_tray();
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }
    let checked = |on: bool| MF_STRING | if on { MF_CHECKED } else { MF_UNCHECKED };

    AppendMenuW(
        menu,
        checked(hook::is_enabled()),
        ID_ENABLED,
        wide("&Enabled").as_ptr(),
    );
    AppendMenuW(
        menu,
        checked(autostart::is_enabled()),
        ID_AUTOSTART,
        wide("Start with &Windows").as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(
        menu,
        MF_STRING,
        ID_OPEN_CONFIG,
        wide("Open &config file").as_ptr(),
    );
    AppendMenuW(menu, MF_STRING, ID_RELOAD, wide("&Reload config").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_ABOUT, wide("&About").as_ptr());
    AppendMenuW(menu, MF_STRING, ID_PROJECT, wide("&Project page").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_EXIT, wide("E&xit").as_ptr());

    let mut pt = std::mem::zeroed();
    GetCursorPos(&mut pt);
    // Without this the menu refuses to close when you click elsewhere.
    SetForegroundWindow(hwnd);
    let choice = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(menu);

    match choice as usize {
        ID_ENABLED => toggle_enabled(),
        ID_AUTOSTART => {
            let _ = autostart::set(!autostart::is_enabled());
        }
        ID_OPEN_CONFIG => {
            if let Some(p) = config::path() {
                open_path(&p.to_string_lossy());
            }
        }
        ID_RELOAD => reload_config(),
        ID_ABOUT => show_about(),
        ID_PROJECT => open_path(PROJECT_URL),
        ID_EXIT => {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
        _ => {}
    }
}

fn reload_config() {
    let fresh = Config::load();
    hook::set_passthrough(fresh.passthrough);
    hook::set_enabled(fresh.enabled);
    APP.with(|a| {
        if let Some(app) = a.borrow_mut().as_mut() {
            if !fresh.show_tray_icon {
                app.tray.hide();
            }
            app.config = fresh;
        }
    });
    refresh_tray();
}

fn show_about() {
    let hotkey = switching::detect_system_hotkey();
    let method = APP.with(|a| {
        a.borrow()
            .as_ref()
            .map(|app| app.config.method)
            .unwrap_or(Method::Auto)
    });
    let route = match method {
        Method::Message => "posting WM_INPUTLANGCHANGEREQUEST".to_string(),
        _ => format!("sending {}", hotkey.describe()),
    };
    let path = config::path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unavailable)".into());

    let text = format!(
        "{APP_NAME} {VERSION}\n\nCaps Lock switches to the next input language.\nSwitching by: {route}\nWindows language shortcut: {}\n\nConfig: {path}\n{PROJECT_URL}",
        hotkey.describe()
    );
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(&text).as_ptr(),
            wide(&format!("About {APP_NAME}")).as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
