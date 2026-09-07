//! The low-level keyboard hook that claims the Caps Lock key.
//!
//! The callback runs on every keystroke in the session and Windows silently
//! drops hooks that take too long, so it does the minimum: decide, post a
//! message to our window, and return.

use crate::config::Passthrough;
use crate::switching::MARKER;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU8, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CAPITAL, VK_CONTROL, VK_MENU, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

static ENABLED: AtomicBool = AtomicBool::new(false);
static PASSTHROUGH: AtomicU8 = AtomicU8::new(0);
static TARGET: AtomicIsize = AtomicIsize::new(0);
/// Caps Lock repeats while held; only the first press should switch.
static CAPS_HELD: AtomicBool = AtomicBool::new(false);
static HOOK: AtomicIsize = AtomicIsize::new(0);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
    if !on {
        CAPS_HELD.store(false, Ordering::Relaxed);
    }
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_passthrough(p: Passthrough) {
    PASSTHROUGH.store(
        match p {
            Passthrough::Shift => 0,
            Passthrough::Ctrl => 1,
            Passthrough::Alt => 2,
            Passthrough::None => 3,
        },
        Ordering::Relaxed,
    );
}

fn passthrough_held() -> bool {
    let vk = match PASSTHROUGH.load(Ordering::Relaxed) {
        0 => VK_SHIFT,
        1 => VK_CONTROL,
        2 => VK_MENU,
        _ => return false,
    };
    unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && ENABLED.load(Ordering::Relaxed) {
        let info = &*(lparam as *const KBDLLHOOKSTRUCT);
        // Skip the modifiers we inject ourselves, or we would recurse.
        if info.dwExtraInfo != MARKER && info.vkCode == VK_CAPITAL as u32 {
            if passthrough_held() {
                // Let Windows toggle capitals the way it always has.
                CAPS_HELD.store(false, Ordering::Relaxed);
            } else {
                let msg = wparam as u32;
                if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                    if !CAPS_HELD.swap(true, Ordering::Relaxed) {
                        let target = TARGET.load(Ordering::Relaxed);
                        if target != 0 {
                            PostMessageW(target as HWND, crate::WM_APP_SWITCH, 0, 0);
                        }
                    }
                } else if msg == WM_KEYUP || msg == WM_SYSKEYUP {
                    CAPS_HELD.store(false, Ordering::Relaxed);
                }
                // Swallow the key so the capitals state never moves.
                return 1;
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

/// Installs the hook on the calling thread. That thread must keep pumping
/// messages or Windows will stop delivering keystrokes to us.
pub fn install(target: HWND) -> bool {
    TARGET.store(target as isize, Ordering::Relaxed);
    let hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), std::ptr::null_mut(), 0) };
    if hook.is_null() {
        return false;
    }
    HOOK.store(hook as isize, Ordering::Relaxed);
    true
}

pub fn uninstall() {
    let hook = HOOK.swap(0, Ordering::Relaxed);
    if hook != 0 {
        unsafe { UnhookWindowsHookEx(hook as HHOOK) };
    }
}
