# Layout Fixer

[Русская версия](README.md)

[![Download for Windows x64](https://img.shields.io/badge/Download-Windows%20x64-0078D4?logo=windows)](https://github.com/sosuchpak228/layout-fixer/releases/download/v0.1.0-preview.6/layout-fixer.exe)

A small, local Windows tool for correcting text typed in the wrong English/Russian keyboard layout. Select text and press **Ctrl+Alt+L**. For example, `ghbdtn` becomes `привет`, and `руддщ` becomes `hello`.

This version **never copies selected text to the clipboard**. It reads the selection through Windows UI Automation or a standard Win32 `Edit` control, then replaces it with Unicode keyboard input. Preview.5 also tries focused browser editors that expose a writable UI Automation value or TextEdit pattern alongside a text selection. If an editor exposes neither interface, the tool leaves the text unchanged.

## Use

1. For a portable test, download `layout-fixer.exe` from [Releases](https://github.com/sosuchpak228/layout-fixer/releases) and run it.
2. For a full per-user installation, download the ZIP, extract it, and run `install.cmd`. It copies the tool to `%LOCALAPPDATA%\Programs\LayoutFixer`, adds current-user startup, and registers an entry in Windows installed apps.
3. Select text in an editable field and press **Ctrl+Alt+L**. The tray menu has **Exit Layout Fixer** and **Uninstall Layout Fixer**. You can also uninstall it from **Settings → Apps → Installed apps**.

The EXE is portable; the release ZIP also includes a per-user installer. No administrator privileges, network connection, service, or clipboard access is required. Letter case is preserved by default: `Ghbdtn` becomes `Привет`, `GHBDTN` becomes `ПРИВЕТ`, and `HF,JNFTN` becomes `РАБОТАЕТ`. Use `--lowercase` only if you prefer the old all-lowercase behavior.

The source is under the [MIT license](LICENSE), which is separate from executable signing. The EXE is not Authenticode-signed yet; signing requires a trusted publisher certificate and does not guarantee immediate SmartScreen reputation.

To try it alongside another layout tool, run `layout-fixer.exe --test-hotkey` and use **Ctrl+Alt+F12**. The normal hotkey can only belong to one application at a time. The application does not silently stop or reconfigure other layout tools.

To use a custom shortcut, pass `--hotkey` followed by a combination, for example:

```powershell
Start-Process -FilePath .\layout-fixer.exe -ArgumentList '--hotkey', 'Ctrl+Alt+A'
```

Supported modifiers are `Ctrl`, `Alt`, `Shift`, and `Win`; supported keys are `A-Z`, `0-9`, and `F1-F24`. At least one modifier is required. If Windows or another application owns the combination, Layout Fixer shows an error and exits.

For a persistent custom shortcut, create a Windows shortcut to the EXE and append the arguments after the closing quote in its **Target** field:

```text
"C:\Tools\Layout Fixer\layout-fixer.exe" --hotkey Ctrl+Alt+A
```

Run that shortcut to test it, then move it to `shell:startup` if desired. The tray tooltip displays the active combination.

The portable mode does not change startup or the registry. The installer uses only the current user profile and does not require administrator privileges, a service, or a driver. Uninstall removes the installed EXE, its startup shortcut, and its app registration; it does not touch unrelated user files. Do not enable two listeners with the same hotkey.

## Compatibility

- Verified on Windows 10 with the classic Notepad `Edit` field in both directions. The same x64 EXE is intended for Windows 10 and 11, but real Windows 11 validation is pending.
- UI Automation `TextPattern` works only in applications that expose an editable text selection through it. Browser search boxes and contenteditable nodes are experimental; Yandex and Magnific Canvas Space still need real-world confirmation. Telegram, Unreal Editor, terminal emulators, and remote desktops are **not yet verified** for this path.
- Does not interact with password fields, read-only selections, unsupported/custom editor controls, elevated windows, protected desktops, or games that reject synthetic Unicode input. It intentionally has no clipboard fallback.
- A foreground-window change or multiple/disappearing selections aborts replacement. Selection is capped at 16,384 UTF-16 units. Standard `Edit` fallback also requires the selection to end within the first 16,384 units of the control.
- No selected text or clipboard content is saved to a log or sent over the network. The tray icon uses a stock Windows icon.

For a skipped hotkey, run a second copy with `--test-hotkey --diagnostics` and test with **Ctrl+Alt+F12**. Redirect its output to files from PowerShell:

```powershell
Start-Process -FilePath .\layout-fixer.exe -ArgumentList '--test-hotkey', '--diagnostics' -RedirectStandardOutput "$env:TEMP\layout-fixer-test.out.log" -RedirectStandardError "$env:TEMP\layout-fixer-test.err.log"
```

Exit that copy through its tray icon after testing. The optional probe reports only control type, editability, supported UI Automation patterns, and skip reason; it does not log selected text. If the hotkey does nothing, open an issue with the Windows version, app/version, whether the control is elevated, and reproduction steps. **Never include selected text, screenshots of private content, or credentials.** This is an early public preview, not a universal replacement for every editor.

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
