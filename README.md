# Layout Fixer

[![Download for Windows x64](https://img.shields.io/badge/Download-Windows%20x64-0078D4?logo=windows)](https://github.com/sosuchpak228/layout-fixer/releases/download/v0.1.0-preview.1/layout-fixer.exe)

A small, local Windows tool for correcting text typed in the wrong English/Russian keyboard layout. Select text and press **Ctrl+Alt+L**. For example, `ghbdtn` becomes `привет`, and `руддщ` becomes `hello`.

This version **never copies selected text to the clipboard**. It reads the selection through Windows UI Automation or a standard Win32 `Edit` control, then replaces it with Unicode keyboard input. If an editor exposes neither interface, the tool leaves the text unchanged.

## Use

1. Use the download button above, or get the EXE/ZIP from [Releases](https://github.com/sosuchpak228/layout-fixer/releases). Keep the EXE in a permanent folder if you plan to add it to startup.
2. Run `layout-fixer.exe`. It lives in the notification area; right-click its icon to exit. As an unsigned preview, it may show a Windows SmartScreen warning; check the source and SHA-256 before choosing to run it.
3. Select text in an editable field and press **Ctrl+Alt+L**.

This is a portable preview, not an installer. No administrator privileges, network connection, service, clipboard access, or automatic startup is required. Letter case is preserved by default: `Ghbdtn` becomes `Привет`, `GHBDTN` becomes `ПРИВЕТ`, and `HF,JNFTN` becomes `РАБОТАЕТ`. Use `--lowercase` only if you prefer the old all-lowercase behavior.

To try it alongside another layout tool, run `layout-fixer.exe --test-hotkey` and use **Ctrl+Alt+F12**. The normal hotkey can only belong to one application at a time. The application does not silently stop or reconfigure other layout tools.

To start it on sign-in, first make sure it works in your editors, then place a shortcut to the EXE in your own Windows Startup folder (`shell:startup`). Remove that shortcut to undo startup. Do not enable both this tool and another `Ctrl+Alt+L` listener on startup.

## Compatibility

- Verified on Windows 10 with the classic Notepad `Edit` field in both directions.
- UI Automation `TextPattern` works only in applications that expose an editable text selection through it. Chrome, Telegram, Unreal Editor, terminal emulators, and remote desktops are **not yet verified** for this new path.
- Does not interact with password fields, read-only selections, unsupported/custom editor controls, elevated windows, protected desktops, or games that reject synthetic Unicode input. It intentionally has no clipboard fallback.
- A foreground-window change or multiple/disappearing selections aborts replacement. Selection is capped at 16,384 UTF-16 units. Standard `Edit` fallback also requires the selection to end within the first 16,384 units of the control.
- No selected text or clipboard content is saved to a log or sent over the network. The tray icon uses a stock Windows icon.

If the hotkey does nothing in your editor, open an issue with the Windows version, app/version, whether the control is elevated, and reproduction steps. **Never include the selected text, screenshots of private content, or credentials.** This is an early public preview, not a universal replacement for every editor.

## Build

On Windows with the Rust 1.98 MSVC toolchain:

```powershell
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

The portable executable is `target/release/layout-fixer.exe`. The pure EN/RU mapping lives in `src/lib.rs`; OS selection and input are in `src/main.rs`, and the notification icon is in `src/tray.rs`.

## Safety and limitations

The tool does not hook keystrokes globally: `RegisterHotKey` only notifies it when its own shortcut is pressed. It reads only the current selection at that moment. Windows UI Automation is not supported uniformly by editors, and `SendInput` cannot cross Windows integrity boundaries. It does not switch your actual keyboard language or install a driver. See [design notes](docs/design.md) for the tradeoffs and test plan.

## License

MIT, see [LICENSE](LICENSE).
