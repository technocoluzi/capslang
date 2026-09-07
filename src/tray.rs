//! Notification-area icon.

use crate::wide;
use windows_sys::Win32::Foundation::{HWND, TRUE};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, DestroyIcon, HICON, LR_DEFAULTCOLOR,
};

const ICON_ON: &[u8] = include_bytes!("../assets/capslang.ico");
const ICON_OFF: &[u8] = include_bytes!("../assets/capslang-off.ico");
const ICON_ID: u32 = 1;

/// Pulls the best-matching image out of a `.ico` and turns it into an HICON.
///
/// The icons are embedded rather than compiled into a resource table so this
/// works identically under the MSVC and GNU toolchains.
fn icon_from_ico(bytes: &[u8], desired: i32) -> HICON {
    let count = match bytes.get(4..6) {
        Some(b) => u16::from_le_bytes([b[0], b[1]]) as usize,
        None => return std::ptr::null_mut(),
    };

    let mut best: Option<(i32, usize, usize)> = None; // (width, offset, len)
    for i in 0..count {
        let e = 6 + i * 16;
        let Some(entry) = bytes.get(e..e + 16) else {
            break;
        };
        let width = if entry[0] == 0 { 256 } else { entry[0] as i32 };
        let len = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let off = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        if bytes.len() < off + len {
            continue;
        }
        let better = match best {
            None => true,
            Some((bw, _, _)) => (width - desired).abs() < (bw - desired).abs(),
        };
        if better {
            best = Some((width, off, len));
        }
    }

    let Some((_, off, len)) = best else {
        return std::ptr::null_mut();
    };
    unsafe {
        CreateIconFromResourceEx(
            bytes[off..].as_ptr(),
            len as u32,
            TRUE,
            0x0003_0000, // icon format version, as the API requires
            desired,
            desired,
            LR_DEFAULTCOLOR,
        )
    }
}

pub struct Tray {
    hwnd: HWND,
    on: HICON,
    off: HICON,
    added: bool,
}

impl Tray {
    pub fn new(hwnd: HWND, size: i32) -> Self {
        Self {
            hwnd,
            on: icon_from_ico(ICON_ON, size),
            off: icon_from_ico(ICON_OFF, size),
            added: false,
        }
    }

    fn data(&self, enabled: bool, tip: &str) -> NOTIFYICONDATAW {
        let mut d: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        d.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        d.hWnd = self.hwnd;
        d.uID = ICON_ID;
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        d.uCallbackMessage = crate::WM_APP_TRAY;
        d.hIcon = if enabled { self.on } else { self.off };
        let tip: Vec<u16> = wide(tip);
        let n = tip.len().min(d.szTip.len());
        d.szTip[..n].copy_from_slice(&tip[..n]);
        d.szTip[d.szTip.len() - 1] = 0;
        d
    }

    /// Adds the icon, or updates it in place if it is already there.
    pub fn show(&mut self, enabled: bool, tip: &str) {
        let mut d = self.data(enabled, tip);
        let action = if self.added { NIM_MODIFY } else { NIM_ADD };
        let ok = unsafe { Shell_NotifyIconW(action, &mut d) } != 0;
        if ok {
            self.added = true;
        } else if self.added {
            // Explorer restarted and dropped our icon; add it again.
            self.added = false;
            let mut d = self.data(enabled, tip);
            self.added = unsafe { Shell_NotifyIconW(NIM_ADD, &mut d) } != 0;
        }
    }

    pub fn hide(&mut self) {
        if self.added {
            let mut d: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
            d.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            d.hWnd = self.hwnd;
            d.uID = ICON_ID;
            unsafe { Shell_NotifyIconW(NIM_DELETE, &mut d) };
            self.added = false;
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        self.hide();
        unsafe {
            if !self.on.is_null() {
                DestroyIcon(self.on);
            }
            if !self.off.is_null() {
                DestroyIcon(self.off);
            }
        }
    }
}
