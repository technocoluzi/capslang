//! Run-at-login, which works two different ways depending on how CapsLang was
//! installed.
//!
//! Installed normally, it is the per-user Run key: no elevation, no scheduled
//! task, and CapsLang owns the setting.
//!
//! Inside an MSIX package that route silently stops working — writes to HKCU
//! are copy-on-write into the package's private hive, so the value lands
//! somewhere only CapsLang can see and Windows never acts on it. Packaged
//! builds declare a `windows.startupTask` in the manifest instead, and there
//! the setting belongs to Windows: it appears in Settings > Apps > Startup,
//! and an app that has been switched off there may not switch itself back on.
//! So rather than keep a checkbox that Windows can overrule, the packaged
//! build sends the user to the setting that actually governs it.

use std::sync::OnceLock;
use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "CapsLang";

/// The Settings page holding the startup task declared in our manifest.
pub const SETTINGS_URI: &str = "ms-settings:startupapps";

/// What came of trying to change the setting.
pub enum Outcome {
    Changed,
    /// A packaged build: Windows owns this, and the user changes it in
    /// Settings > Apps > Startup.
    ManagedByWindows,
    Failed(String),
}

const APPMODEL_ERROR_NO_PACKAGE: u32 = 15700;

/// True when running from inside an MSIX package.
pub fn is_packaged() -> bool {
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        let mut len: u32 = 0;
        // Packaged, this fails with ERROR_INSUFFICIENT_BUFFER instead.
        let rc = unsafe { GetCurrentPackageFullName(&mut len, std::ptr::null_mut()) };
        rc != APPMODEL_ERROR_NO_PACKAGE
    })
}

/// Whether CapsLang has registered itself to start at sign-in. Always false in
/// a packaged build, where the answer is Windows' to give, not ours.
pub fn is_enabled() -> bool {
    if is_packaged() {
        return false;
    }
    let Some(expected) = command() else {
        return false;
    };
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ)
        .and_then(|k| k.get_value::<String, _>(RUN_VALUE))
        .map(|v| v.trim().eq_ignore_ascii_case(expected.trim()))
        .unwrap_or(false)
}

pub fn set(enabled: bool) -> Outcome {
    if is_packaged() {
        return Outcome::ManagedByWindows;
    }
    match run_key_set(enabled) {
        Ok(()) => Outcome::Changed,
        Err(e) => Outcome::Failed(e.to_string()),
    }
}

fn command() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(format!("\"{}\"", exe.display()))
}

fn run_key_set(enabled: bool) -> std::io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_WRITE)?;
    if enabled {
        let cmd = command().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "cannot locate our own exe")
        })?;
        key.set_value(RUN_VALUE, &cmd)
    } else {
        match key.delete_value(RUN_VALUE) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
