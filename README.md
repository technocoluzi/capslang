<div align="center">

<img src="assets/capslang.ico" width="96" height="96" alt="">

# CapsLang

**Switch keyboard input language with the Caps Lock key, on Windows.**

[![CI](https://github.com/technocoluzi/capslang/actions/workflows/ci.yml/badge.svg)](https://github.com/technocoluzi/capslang/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

---

If you type in two alphabets — Hebrew and English, Arabic and English, Russian
and English, Greek, Thai — you switch languages hundreds of times a day.
Windows gives you three ways to do it: `Alt+Shift`, `Ctrl+Shift`, or the
`` ` `` key. The first two are two-handed chords that collide with application
shortcuts; the third costs you the backtick, which is not a trade a programmer
can make.

Caps Lock is a big, easy-to-hit key that almost nobody uses for capitals.
CapsLang gives it the job it should have had.

Windows has no built-in setting for this, so CapsLang is a small tray app that
installs a keyboard hook and does it for you.

## Install

Download the installer from the [latest release](https://github.com/technocoluzi/capslang/releases/latest)
and run it. It installs into your user profile — no administrator prompt — and
offers to start CapsLang when you sign in.

To install without any prompts:

```
CapsLang-setup.exe /VERYSILENT
```

That installs it, registers it to start at sign-in, and launches it. Add
`/TASKS=""` if you would rather it not start at sign-in.

Or build it yourself — see [Building](#building).

## Use

| Key | Does |
| --- | --- |
| `Caps Lock` | Switch to the next input language |
| `Shift` + `Caps Lock` | The original Caps Lock — toggle capitals |

Right-click the tray icon to pause CapsLang, toggle start-at-login, open the
config file, or quit. Double-clicking the icon pauses and resumes.

Installed from the Microsoft Store, start-at-login belongs to Windows rather
than to CapsLang — the menu opens *Settings → Apps → Startup* instead of
carrying its own checkbox, and `--autostart` reports that and does nothing.
A packaged app cannot honestly own that switch: Windows can overrule it.

There is also a command line, useful for scripts and for the installer:

```
capslang                  Start CapsLang (does nothing if already running)
capslang --quit           Stop the running instance
capslang --autostart on   Start CapsLang when you sign in
capslang --autostart off  Stop starting at sign-in
capslang --config         Open the configuration file
capslang --status         Report whether CapsLang is running
capslang --version        Print the version
capslang --help           Show this help
```

## Configure

The config file lives at `%APPDATA%\CapsLang\config.toml` and is created on
first run. Edit it, then pick **Reload config** from the tray menu.

```toml
# Start with Caps Lock switching active.
enabled = true

# How to switch languages.
#   "auto"    — hotkey, falling back to message when Windows has no shortcut set
#   "hotkey"  — synthesise the system shortcut (Alt+Shift / Ctrl+Shift / `)
#   "message" — post WM_INPUTLANGCHANGEREQUEST to the focused window
method = "auto"

# Hold this and press Caps Lock for the original capitals toggle.
#   "shift" | "ctrl" | "alt" | "none"
passthrough = "shift"

# Show the notification-area icon. With this off, stop CapsLang by running
# `capslang --quit`.
show_tray_icon = true
```

## How it works

A `WH_KEYBOARD_LL` hook watches for Caps Lock. When it sees one — and your
passthrough modifier is not held — it swallows the key so the capitals state
never moves, and posts a message to CapsLang's hidden window. The hook itself
does nothing else: Windows silently drops hooks whose callback runs long, and
that is the usual reason these tools "stop working after a while".

The window then switches the language one of two ways:

- **`hotkey`** synthesises whatever shortcut Windows itself is configured to
  use, read from `HKCU\Keyboard Layout\Toggle`. Because it is the real system
  shortcut, it works anywhere the shortcut works.
- **`message`** posts `WM_INPUTLANGCHANGEREQUEST` straight to the focused
  window. Nothing touches the keyboard stream, but an application is free to
  ignore the message, and some do.

`auto` picks `hotkey`, unless you have set the Windows shortcut to *Not
assigned*, in which case there is nothing to synthesise and it uses `message`.

CapsLang cycles forward through every input language you have installed. With
the usual two, that is a straight toggle.

## Troubleshooting

**"Windows protected your PC" when running the installer.** CapsLang is not
code-signed yet. Click *More info* → *Run anyway*. If you would rather not,
build from source.

**"Smart App Control has blocked this app".** Smart App Control hard-blocks
unsigned software with no way to bypass it per-app. It is only on for a
minority of machines — clean installs of Windows 11 22H2 or later that passed
its evaluation period. You can turn it off in *Windows Security → App &
browser control → Smart App Control*, and recent Windows updates let you turn
it back on afterwards.

**Caps Lock does nothing in one particular app.** If that app runs elevated and
CapsLang does not, Windows will not deliver its keystrokes to our hook. Run
CapsLang as administrator too, or leave that app alone.

**Caps Lock switches, but the app ignores the new language.** Try
`method = "message"`, or the reverse if you are already on it.

**Caps Lock stopped working after a while.** Check that `capslang.exe` is still
running (`capslang --status`). If it is not, and it disappears repeatedly, open
an issue with the app you were using when it stopped.

## Building

Requires the [Rust toolchain](https://rustup.rs/). On Windows with the MSVC
toolchain (the default), you also need the Visual Studio Build Tools.

```
cargo build --release
```

The binary lands at `target/release/capslang.exe` and needs no other files.

To build the installer as well, install [Inno Setup 6](https://jrsoftware.org/isinfo.php)
and run:

```
ISCC.exe /DAppVersion=0.1.0 installer/capslang.iss
```

The icons are generated from code, so they can be reviewed rather than taken on
trust. Regenerate them with `powershell -File assets/generate-icons.ps1`.

### The Store package

```
powershell -File packaging/build-msix.ps1 -Version 0.1.0.0
```

That stages `dist/msix-layout` and, with the Windows SDK installed, packs
`dist/CapsLang-<version>.msix`. The package is deliberately left unsigned: the
Store signs submissions with a Microsoft-trusted certificate, which is the
point of shipping there — it is what clears SmartScreen and Smart App Control.

To run the package locally without signing anything, turn on Developer Mode
(*Settings → System → For developers*) and register the layout in place:

```
Add-AppxPackage -Register dist\msix-layout\AppxManifest.xml
Remove-AppxPackage (Get-AppxPackage *CapsLang*).PackageFullName
```

`Identity/Name` and `Identity/Publisher` in the manifest are placeholders until
the name is reserved in Partner Center; pass the real values to the script.

## License

MIT — see [LICENSE](LICENSE).
