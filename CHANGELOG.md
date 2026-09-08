# Changelog

All notable changes to CapsLang are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- MSIX packaging for the Microsoft Store: manifest, generated logo assets,
  and a build script, wired into CI so a broken manifest fails there rather
  than at submission.
- `--status` now reports how CapsLang was installed, whether it starts at
  sign-in, and which shortcut it sends, so a bug report can start with a paste.
- A privacy policy and the Microsoft Store listing copy.

### Fixed

- Command output goes to a redirected stdout when the caller provides one, so
  `capslang --status > file` and pipelines work. Rust reports a write to a
  null standard handle as a silent success, so the handle is now checked
  directly rather than trusting the write.

### Changed

- Start-at-login now depends on how CapsLang was installed. Unpackaged, the
  Run key, as before. Packaged, Windows owns the setting through the manifest
  startup task, so the tray menu opens Settings > Apps > Startup instead of
  showing a checkbox it cannot honestly keep in sync.

## [0.1.0]

First release.

- Caps Lock switches to the next Windows input language.
- Shift+Caps Lock keeps the original capitals toggle; the modifier is
  configurable, or can be turned off entirely.
- Notification-area icon: pause and resume, toggle start-at-login, open or
  reload the config file.
- Two switching strategies — synthesising the system language shortcut, or
  posting `WM_INPUTLANGCHANGEREQUEST` — with `auto` choosing between them.
- Command line for scripting and for the installer: `--quit`, `--status`,
  `--autostart on|off`, `--config`.
- Per-user installer that needs no administrator rights.
