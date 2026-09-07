//! User configuration, persisted as TOML next to the app's data.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How CapsLang asks Windows to move to the next input language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    /// Pick `Hotkey` normally, `Message` if Windows has no switch hotkey set.
    Auto,
    /// Synthesise the system language hotkey (Alt+Shift / Ctrl+Shift / `).
    /// Works in every app that honours the normal Windows shortcut.
    Hotkey,
    /// Post WM_INPUTLANGCHANGEREQUEST straight to the focused window.
    /// Leaves no modifier keys in flight, but some apps ignore it.
    Message,
}

/// Which modifier lets the original Caps Lock behaviour through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Passthrough {
    Shift,
    Ctrl,
    Alt,
    /// Caps Lock can no longer toggle capitals at all.
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Start with switching active. Toggle at runtime from the tray menu.
    pub enabled: bool,
    pub method: Method,
    /// Hold this and press Caps Lock to toggle capitals as usual.
    pub passthrough: Passthrough,
    /// Turning this off leaves `capslang --quit` as the way to stop the app.
    pub show_tray_icon: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            method: Method::Auto,
            passthrough: Passthrough::Shift,
            show_tray_icon: true,
        }
    }
}

const TEMPLATE: &str = "\
# CapsLang configuration - https://github.com/technocoluzi/capslang
#
# Edit, save, then pick \"Reload config\" from the tray menu.

# Start with Caps Lock switching active.
enabled = true

# How to switch: \"auto\", \"hotkey\" (synthesise Alt+Shift / Ctrl+Shift / `),
# or \"message\" (post WM_INPUTLANGCHANGEREQUEST to the focused window).
method = \"auto\"

# Hold this and press Caps Lock for the original capitals toggle:
# \"shift\", \"ctrl\", \"alt\", or \"none\".
passthrough = \"shift\"

# Show the notification-area icon. With this off, stop the app by running
# `capslang --quit`.
show_tray_icon = true
";

/// `%APPDATA%\CapsLang\config.toml`
pub fn path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(appdata).join("CapsLang").join("config.toml"))
}

impl Config {
    /// Reads the config, falling back to defaults. Writes a commented starter
    /// file the first time so the options are discoverable.
    pub fn load() -> Self {
        let Some(p) = path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&p) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => {
                if let Some(dir) = p.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = std::fs::write(&p, TEMPLATE);
                Self::default()
            }
        }
    }

    /// Rewrites the file, preserving nothing but the values. Used when the tray
    /// menu changes a setting so the change survives a restart.
    pub fn save(&self) -> std::io::Result<()> {
        let p = path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no APPDATA directory")
        })?;
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(p, format!("# CapsLang configuration\n\n{body}"))
    }
}
