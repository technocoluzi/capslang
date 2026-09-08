# Microsoft Store listing

Copy for the Partner Center submission, kept here so it is versioned alongside
the package it describes. Paste each section into the matching Partner Center
field.

---

## Product name

    CapsLang

Reserve this in Partner Center first. The reservation is what produces the
`Identity/Name` and `Identity/Publisher` values that go into the manifest —
until then `packaging/msix/AppxManifest.xml` carries placeholders.

## Category

**Utilities & tools**

## Pricing

Free. No in-app purchases, no trial.

---

## Short description

> Give the Caps Lock key a better job: switching your keyboard language.

## Description

> If you type in two alphabets — Hebrew and English, Arabic and English,
> Russian, Greek, Thai — you switch input languages hundreds of times a day.
>
> Windows gives you three ways to do it. Alt+Shift and Ctrl+Shift are
> two-handed chords that collide with the shortcuts of whatever app you are in.
> The third is the ` key, which costs you the backtick — not a trade a
> programmer can make.
>
> Meanwhile Caps Lock sits there: a large, easy-to-hit key that almost nobody
> uses for capitals, and that most people have wanted to repurpose for years.
> Windows has no setting for it.
>
> CapsLang is that setting.
>
> Press Caps Lock, switch language. Hold Shift and press it, and you get the
> old capitals toggle, exactly as before. That is the entire product.
>
> It switches by sending the same shortcut Windows itself is configured to use,
> read from your own settings, so it works wherever that shortcut works rather
> than in a list of supported applications. If an app ignores it, one line in
> the config file switches to a different method.
>
> CapsLang makes no network connections. It reads each key only far enough to
> ask "is this Caps Lock?", records nothing, and stores nothing but your
> preferences. It is open source under the MIT licence, and the keyboard hook
> is one short function you can read for yourself.

## Product features

- Caps Lock switches to your next Windows input language
- Shift+Caps Lock keeps the original capitals toggle
- Works in any application, by using the shortcut Windows already uses
- No network access at all: no telemetry, no analytics, no account
- Pause and resume from the notification area, without uninstalling
- Under one megabyte, with no runtime to install alongside it
- Open source under the MIT licence

## Search terms

    caps lock
    keyboard language
    switch input language
    keyboard layout
    language switcher
    hebrew english keyboard
    bilingual typing

## Screenshots

`screenshots/01-tray-menu.png` (1366x768). Genuine capture of the running
application, composited onto a plain background so no unrelated desktop
content appears.

## Privacy policy URL

    https://github.com/technocoluzi/capslang/blob/main/PRIVACY.md

## Support contact

    https://github.com/technocoluzi/capslang/issues

---

## Submission options: restricted capability justification

`runFullTrust` is a restricted capability, so Partner Center asks why the app
declares it. Paste this:

> CapsLang gives the Caps Lock key the job of switching the Windows input
> language. Doing that requires a system-wide low-level keyboard hook
> (SetWindowsHookEx with WH_KEYBOARD_LL) so the key can be observed and
> suppressed outside the application's own windows. No sandboxed API can do
> this, and the hook is only available to full-trust desktop processes, so
> runFullTrust is what the app fundamentally needs in order to work at all.
>
> The capability is used for nothing else. The hook compares each virtual key
> code against VK_CAPITAL and returns immediately for every other key, which is
> neither recorded nor examined further. The application makes no network
> connections of any kind, contains no telemetry or update check, and writes
> only a small local configuration file holding user preferences.
>
> The complete source is public at https://github.com/technocoluzi/capslang,
> and the hook is a single function in src/hook.rs.

## Age rating

The IARC questionnaire applies. CapsLang collects and shares no data, has no
user-generated content, no purchases, no ads, and no communication features,
which should produce the lowest rating in every market.
