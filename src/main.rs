#![windows_subsystem = "windows"]

use std::{mem::size_of, thread, time::Duration};

use layout_fixer::{Direction, convert};
mod tray;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    },
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
            IUIAutomationValuePattern, UIA_EditControlTypeId, UIA_TextPatternId,
            UIA_ValuePatternId,
        },
        Controls::EM_GETSEL,
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
            KEYEVENTF_UNICODE, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey, SendInput,
            UnregisterHotKey, VK_CONTROL, VK_F12, VK_L, VK_MENU,
        },
        WindowsAndMessaging::{
            ES_PASSWORD, ES_READONLY, GWL_STYLE, GetClassNameW, GetForegroundWindow, GetMessageW,
            GetWindowLongPtrW, IsChild, MB_ICONERROR, MB_OK, MSG, MessageBoxW, SMTO_ABORTIFHUNG,
            SendMessageTimeoutW, WM_GETTEXT, WM_GETTEXTLENGTH, WM_HOTKEY,
        },
    },
};
use windows::core::{HSTRING, w};

const MAX_SELECTION: i32 = 16_384;
const HOTKEY_ID: i32 = 0x4c46;

fn message(hwnd: HWND, code: u32, wparam: usize, lparam: isize) -> Result<usize, String> {
    let mut result = 0usize;
    let status = unsafe {
        SendMessageTimeoutW(
            hwnd,
            code,
            WPARAM(wparam),
            LPARAM(lparam),
            SMTO_ABORTIFHUNG,
            500,
            Some(&mut result),
        )
    };
    if status.0 == 0 {
        Err("standard Edit control did not answer within 500 ms".to_string())
    } else {
        Ok(result)
    }
}

fn standard_edit_selection(
    focused: &IUIAutomationElement,
    foreground: HWND,
) -> Result<Option<String>, String> {
    let hwnd = unsafe { focused.CurrentNativeWindowHandle() }
        .map_err(|e| format!("editor window: {e}"))?;
    if !unsafe { IsChild(foreground, hwnd) }.as_bool() {
        return Err("focused editor is not in foreground window".to_string());
    }
    let mut class = [0u16; 64];
    let count = unsafe { GetClassNameW(hwnd, &mut class) };
    if count == 0
        || !String::from_utf16_lossy(&class[..count as usize]).eq_ignore_ascii_case("Edit")
    {
        return Err("focused editor does not expose UIA TextPattern or standard Edit".to_string());
    }
    if unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & (ES_PASSWORD | ES_READONLY) as isize != 0 {
        return Ok(None);
    }
    let mut start = 0u32;
    let mut end = 0u32;
    message(
        hwnd,
        EM_GETSEL,
        (&mut start as *mut u32) as usize,
        (&mut end as *mut u32) as isize,
    )?;
    if start >= end || end - start > MAX_SELECTION as u32 || end > MAX_SELECTION as u32 {
        return Ok(None);
    }
    let length = message(hwnd, WM_GETTEXTLENGTH, 0, 0)?;
    if end as usize > length {
        return Err("selection changed while reading editor".to_string());
    }
    let mut buffer = vec![0u16; end as usize + 1];
    let copied = message(hwnd, WM_GETTEXT, buffer.len(), buffer.as_mut_ptr() as isize)?;
    if copied < end as usize {
        return Err("editor returned truncated text".to_string());
    }
    let mut latest_start = 0u32;
    let mut latest_end = 0u32;
    message(
        hwnd,
        EM_GETSEL,
        (&mut latest_start as *mut u32) as usize,
        (&mut latest_end as *mut u32) as isize,
    )?;
    if (latest_start, latest_end) != (start, end) {
        return Err("selection changed while reading editor".to_string());
    }
    String::from_utf16(&buffer[start as usize..end as usize])
        .map(Some)
        .map_err(|_| "editor selection contains invalid UTF-16".to_string())
}

fn selection(
    ui: &IUIAutomation,
    foreground: HWND,
) -> Result<Option<(String, &'static str)>, String> {
    // UIA only reads the active element; it never alters the global clipboard.
    let focused = unsafe { ui.GetFocusedElement() }.map_err(|e| format!("focused element: {e}"))?;
    if unsafe { focused.CurrentIsPassword() }
        .map_err(|e| format!("password check: {e}"))?
        .as_bool()
    {
        return Ok(None);
    }
    if unsafe { focused.CurrentControlType() }.map_err(|e| format!("editor type: {e}"))?
        != UIA_EditControlTypeId
    {
        return Ok(None);
    }
    if let Ok(value) =
        unsafe { focused.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) }
        && unsafe { value.CurrentIsReadOnly() }
            .map(|v| v.as_bool())
            .unwrap_or(true)
    {
        return Ok(None);
    }
    let pattern: IUIAutomationTextPattern =
        match unsafe { focused.GetCurrentPatternAs(UIA_TextPatternId) } {
            Ok(pattern) => pattern,
            Err(_) => {
                return standard_edit_selection(&focused, foreground)
                    .map(|selected| selected.map(|text| (text, "Win32 Edit")));
            }
        };
    let ranges = unsafe { pattern.GetSelection() }.map_err(|e| format!("selection: {e}"))?;
    if unsafe { ranges.Length() }.map_err(|e| format!("selection count: {e}"))? != 1 {
        return Ok(None);
    }
    let range = unsafe { ranges.GetElement(0) }.map_err(|e| format!("text range: {e}"))?;
    let value = unsafe { range.GetText(MAX_SELECTION + 1) }
        .map_err(|e| format!("selected text: {e}"))?
        .to_string();
    if value.is_empty() || value.encode_utf16().count() > MAX_SELECTION as usize {
        return Ok(None);
    }
    Ok(Some((value, "UI Automation")))
}

fn modifiers_released() -> bool {
    for _ in 0..120 {
        let down = unsafe {
            GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 || GetAsyncKeyState(VK_MENU.0 as i32) < 0
        };
        if !down {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    false
}

fn unicode_key(ch: u16, released: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: Default::default(),
                wScan: ch,
                dwFlags: KEYEVENTF_UNICODE
                    | if released {
                        KEYEVENTF_KEYUP
                    } else {
                        Default::default()
                    },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn replace_selection(text: &str) -> Result<(), String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut events = Vec::with_capacity(normalized.len() * 2);
    for unit in normalized.encode_utf16() {
        events.push(unicode_key(unit, false));
        events.push(unicode_key(unit, true));
    }
    let sent = unsafe { SendInput(&events, size_of::<INPUT>() as i32) };
    if sent != events.len() as u32 {
        return Err(format!(
            "input incomplete ({sent}/{} events): {}; target may be elevated or may reject synthetic Unicode",
            events.len(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn on_hotkey(ui: &IUIAutomation, lowercase: bool) -> Result<&'static str, String> {
    let foreground = unsafe { GetForegroundWindow() };
    if !modifiers_released() {
        return Err("release Ctrl and Alt, then retry".to_string());
    }
    let (source, method) = match selection(ui, foreground)? {
        Some(value) => value,
        None => return Ok("no supported selection; unchanged"),
    };
    let replacement = convert(&source, Direction::Auto, lowercase);
    if replacement == source {
        return Ok("selection unchanged by mapping");
    }
    if unsafe { GetForegroundWindow() } != foreground {
        return Err("foreground changed; selection left untouched".to_string());
    }
    replace_selection(&replacement)?;
    Ok(if method == "UI Automation" {
        "converted via UI Automation; clipboard untouched"
    } else {
        "converted via Win32 Edit; clipboard untouched"
    })
}

fn run(hotkey: u32, lowercase: bool) -> Result<(), String> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .map_err(|e| format!("COM initialization: {e}"))?;
    let result = (|| {
        let ui: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
                .map_err(|e| format!("UI Automation: {e}"))?;
        let hotkey_name = if hotkey == VK_L.0 as u32 { "L" } else { "F12" };
        let tray = tray::Tray::new(hotkey_name)?;
        unsafe {
            RegisterHotKey(
                Some(tray.window()),
                HOTKEY_ID,
                MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                hotkey,
            )
        }
        .map_err(|e| format!("hotkey unavailable (another app may own it): {e}"))?;
        println!(
            "Ready: Ctrl+Alt+{}. Press Ctrl+C here to stop.",
            hotkey_name
        );
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                match on_hotkey(&ui, lowercase) {
                    Ok(status) => println!("{status}"),
                    Err(message) => eprintln!("Skipped: {message}"),
                }
            }
        }
        unsafe { UnregisterHotKey(Some(tray.window()), HOTKEY_ID) }.ok();
        Ok(())
    })();
    unsafe { CoUninitialize() };
    result
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Layout Fixer 0.1.0-preview.3\n  (no arguments)      Listen on Ctrl+Alt+L in the tray; preserve letter case\n  --test-hotkey        Listen on Ctrl+Alt+F12 alongside an existing fixer\n  --lowercase          Convert all output to lowercase\n  --help               Show this message\nNo clipboard access, network, service, or autorun."
        );
        return;
    }
    if args
        .iter()
        .any(|a| a != "--test-hotkey" && a != "--lowercase")
    {
        eprintln!("Unknown argument; see --help.");
        std::process::exit(2);
    }
    let hotkey = if args.iter().any(|a| a == "--test-hotkey") {
        VK_F12.0
    } else {
        VK_L.0
    };
    if let Err(error) = run(hotkey as u32, args.iter().any(|a| a == "--lowercase")) {
        eprintln!("Layout Fixer: {error}");
        if !args.iter().any(|a| a == "--test-hotkey") {
            unsafe {
                MessageBoxW(
                    None,
                    &HSTRING::from(error),
                    w!("Layout Fixer"),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
        std::process::exit(1);
    }
}
