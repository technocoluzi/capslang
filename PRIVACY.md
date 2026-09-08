# CapsLang privacy policy

_Last updated: 8 September 2026_

**CapsLang collects nothing, sends nothing, and stores nothing about you.**

That is the whole policy. The rest of this page explains why you should believe
it, because "a program that watches every key you press" deserves more than a
reassurance.

## What CapsLang does with your keystrokes

CapsLang installs a low-level keyboard hook (`WH_KEYBOARD_LL`). Windows offers
no other way to give the Caps Lock key a different job, and the hook does see
every key press on the way past.

What it does with them:

- It reads the virtual key code and compares it to `VK_CAPITAL` — Caps Lock.
- Every other key returns immediately, untouched and unread.
- Nothing is recorded, buffered, logged, counted, or written down. Not the keys
  themselves, not their timing, not how often you switch languages.

You do not have to take this on trust. The whole program is a few hundred lines
of open source, and the hook is one function in one file:
[`src/hook.rs`](https://github.com/technocoluzi/capslang/blob/main/src/hook.rs).

## Network

CapsLang makes no network connections of any kind. There is no telemetry, no
analytics, no crash reporting, no update check, and no account. It has never
needed the internet and does not ask for it.

The only exception is when *you* choose it: the tray menu's **Project page**
item opens the GitHub repository in your browser, exactly as if you had typed
the address yourself.

## What is stored on your device

One small configuration file, holding your preferences and nothing else —
whether switching is on, which method it uses, and which modifier key preserves
the original Caps Lock. It lives in your user profile and never leaves your
device.

Uninstalling CapsLang removes it.

## Children

CapsLang is a keyboard utility. It collects no data from anyone, of any age.

## Changes

If this policy ever changes, the new version will appear at this address and
the date above will change with it. The change history is public in the
repository.

## Contact

Questions, or something here that does not match what you observe:
<https://github.com/technocoluzi/capslang/issues>
