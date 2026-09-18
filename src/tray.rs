use std::mem::size_of;

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
                DestroyWindow, GetCursorPos, IDI_APPLICATION, LoadIconW, MF_STRING,
                PostQuitMessage, RegisterClassW, SetForegroundWindow, TPM_RIGHTBUTTON,
                TrackPopupMenu, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_COMMAND, WM_DESTROY,
                WM_RBUTTONUP, WNDCLASSW,
            },
        },
    },
    core::w,
};

const TRAY_MESSAGE: u32 = WM_APP + 1;
const EXIT_ID: usize = 1;

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        TRAY_MESSAGE if lparam.0 as u32 == WM_RBUTTONUP => {
            if let Ok(menu) = unsafe { CreatePopupMenu() } {
                unsafe {
                    let _ = AppendMenuW(menu, MF_STRING, EXIT_ID, w!("Exit Layout Fixer"));
                    let mut point = POINT::default();
                    if GetCursorPos(&mut point).is_ok() {
                        let _ = SetForegroundWindow(hwnd);
                        let _ = TrackPopupMenu(
                            menu,
                            TPM_RIGHTBUTTON,
                            point.x,
                            point.y,
                            None,
                            hwnd,
                            None,
                        );
                    }
                    let _ = DestroyMenu(menu);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND if wparam.0 & 0xffff == EXIT_ID => {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub struct Tray {
    hwnd: HWND,
    icon: NOTIFYICONDATAW,
}

impl Tray {
    pub fn new(hotkey: &str) -> Result<Self, String> {
        let module = unsafe { GetModuleHandleW(None) }.map_err(|e| e.to_string())?;
        let instance = HINSTANCE(module.0);
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: w!("LayoutFixerTrayWindow"),
            ..Default::default()
        };
        if unsafe { RegisterClassW(&class) } == 0 {
            return Err(format!(
                "could not register tray window: {}",
                std::io::Error::last_os_error()
            ));
        }
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("LayoutFixerTrayWindow"),
                w!("Layout Fixer"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance),
                None,
            )
        }
        .map_err(|e| format!("tray window: {e}"))?;
        let mut icon = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: TRAY_MESSAGE,
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }.map_err(|e| e.to_string())?,
            ..Default::default()
        };
        for (target, unit) in icon
            .szTip
            .iter_mut()
            .zip(format!("Layout Fixer: Ctrl+Alt+{hotkey}").encode_utf16())
        {
            *target = unit;
        }
        if !unsafe { Shell_NotifyIconW(NIM_ADD, &icon) }.as_bool() {
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return Err("could not register tray icon".to_string());
        }
        Ok(Self { hwnd, icon })
    }

    pub fn window(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.icon);
            if windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(self.hwnd)).as_bool() {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}
