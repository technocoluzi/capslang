//! Moving Windows to the next input language.

use crate::config::Method;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_LCONTROL, VK_LMENU,
    VK_LSHIFT, VK_OEM_3,
};
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// Stamped on every key event we synthesise so our own hook ignores them.
pub const MARKER: usize = 0x4341_5053; // "CAPS"

const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;
const INPUTLANGCHANGE_FORWARD: usize = 0x0002;

/// The shortcut Windows itself uses to cycle input languages, as configured
/// under `HKCU\Keyboard Layout\Toggle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemHotkey {
    AltShift,
    CtrlShift,
    Grave,
    NotAssigned,
}

impl SystemHotkey {
    pub fn describe(self) -> &'static str {
        match self {
            Self::AltShift => "Alt+Shift",
            Self::CtrlShift => "Ctrl+Shift",
            Self::Grave => "` (grave accent)",
            Self::NotAssigned => "not assigned",
        }
    }
}

/// Reads the user's configured language-switch shortcut. Windows omits these
/// values while they are at their defaults, which means Alt+Shift.
pub fn detect_system_hotkey() -> SystemHotkey {
    let read = || -> Option<String> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let toggle = hkcu.open_subkey(r"Keyboard Layout\Toggle").ok()?;
        toggle
            .get_value::<String, _>("Language Hotkey")
            .or_else(|_| toggle.get_value::<String, _>("Hotkey"))
            .ok()
    };
    match read().as_deref().map(str::trim) {
        Some("2") => SystemHotkey::CtrlShift,
        Some("3") => SystemHotkey::NotAssigned,
        Some("4") => SystemHotkey::Grave,
        // "1", anything unrecognised, or absent: the Windows default.
        _ => SystemHotkey::AltShift,
    }
}

fn key(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: MARKER,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Taps the system shortcut. The modifier is pressed and released around the
/// second key so Windows sees a complete chord and no stray menu activation.
fn send_hotkey(hotkey: SystemHotkey) {
    match hotkey {
        SystemHotkey::AltShift => send(&[
            key(VK_LMENU, false),
            key(VK_LSHIFT, false),
            key(VK_LSHIFT, true),
            key(VK_LMENU, true),
        ]),
        SystemHotkey::CtrlShift => send(&[
            key(VK_LCONTROL, false),
            key(VK_LSHIFT, false),
            key(VK_LSHIFT, true),
            key(VK_LCONTROL, true),
        ]),
        SystemHotkey::Grave => send(&[key(VK_OEM_3, false), key(VK_OEM_3, true)]),
        SystemHotkey::NotAssigned => post_to_foreground(),
    }
}

/// Asks the focused window to move to the next layout itself. Nothing is
/// injected into the keyboard stream, but apps are free to ignore it.
fn post_to_foreground() {
    unsafe {
        let hwnd = GetForegroundWindow();
        if !hwnd.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                hwnd,
                WM_INPUTLANGCHANGEREQUEST,
                INPUTLANGCHANGE_FORWARD,
                0,
            );
        }
    }
}

pub fn switch(method: Method) {
    match method {
        Method::Hotkey => send_hotkey(detect_system_hotkey()),
        Method::Message => post_to_foreground(),
        // `NotAssigned` already falls through to the message route.
        Method::Auto => send_hotkey(detect_system_hotkey()),
    }
}
