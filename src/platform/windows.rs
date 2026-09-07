use crate::brain::movement::{MonitorBounds, Point};
use crate::render::Frame;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
    GetMonitorInfoW, ReleaseDC, SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, HMONITOR,
    MONITORINFO,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMessageW, LoadCursorW, LoadImageW, MessageBoxW, PostMessageW, PostQuitMessage,
    RegisterClassExW, SetForegroundWindow, SetTimer, ShowWindow, TrackPopupMenu, TranslateMessage,
    UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, IMAGE_ICON, LR_DEFAULTSIZE,
    LR_LOADFROMFILE, MB_ICONINFORMATION, MB_OK, MF_STRING, MSG, SW_SHOWNOACTIVATE,
    TPM_RIGHTBUTTON, ULW_ALPHA, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_NULL,
    WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::Win32::UI::WindowsAndMessaging::HICON;

const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_ID: u32 = 1;
const ID_TRAY_ABOUT: usize = 1001;
const ID_TRAY_QUIT: usize = 1002;

const ABOUT_TEXT: PCWSTR = windows::core::w!(
    "DeskFrog v1.0\n\nCopyright (c) 2026 Dominic Lenz\n\nOpen Source software licensed under GNU GPL v3\n\nhttps://github.com/Zongonaut/deskfrog/\n\nFont8x8 by Daniel Hepper (Public Domain)"
);
const ABOUT_TITLE: PCWSTR = windows::core::w!("About DeskFrog");

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Loads the icon shipped in the source tree's `assets/` folder. Baking in the
/// manifest-relative path (rather than an embedded resource ID) keeps this
/// simple while the project has no packaging/install step yet — revisit if
/// DeskFrog ever ships as a relocated standalone binary.
fn load_frog_icon() -> HICON {
    let path = to_wide(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/frog.ico"));
    unsafe {
        match LoadImageW(None, PCWSTR(path.as_ptr()), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE) {
            Ok(handle) => HICON(handle.0),
            Err(_) => HICON::default(),
        }
    }
}

/// Current mouse position in virtual-desktop coordinates, regardless of which
/// window (if any) has focus — works even though FrogWindow is click-through.
pub fn cursor_pos() -> Point {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    Point { x: p.x, y: p.y }
}

/// Enumerates all monitors in virtual-desktop coordinates (which may include
/// negative x/y when a monitor extends left of or above the primary display).
pub fn enumerate_monitors() -> Vec<MonitorBounds> {
    let mut monitors: Vec<MonitorBounds> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut Vec<MonitorBounds> as isize),
        );
    }
    monitors
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorBounds>) };
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(hmonitor, &mut info) }.as_bool() {
        let rc = info.rcWork;
        monitors.push(MonitorBounds {
            x: rc.left,
            y: rc.top,
            width: rc.right - rc.left,
            height: rc.bottom - rc.top,
        });
    }
    BOOL(1)
}

const CLASS_NAME: PCWSTR = windows::core::w!("DeskFrogWindowClass");

/// Owns the layered window and the DIB section backing its pixel buffer.
/// The window is click-through, topmost, and never appears in the taskbar/Alt-Tab.
pub struct FrogWindow {
    hwnd: HWND,
    width: u32,
    height: u32,
    mem_dc: windows::Win32::Graphics::Gdi::HDC,
    bitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
    pixels: *mut u8,
    tray_icon: Option<HICON>,
}

impl FrogWindow {
    pub fn new(width: u32, height: u32) -> windows::core::Result<Self> {
        unsafe {
            let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                lpszClassName: CLASS_NAME,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let ex_style = WS_EX_LAYERED
                | WS_EX_TRANSPARENT
                | WS_EX_TOPMOST
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE;

            let hwnd = CreateWindowExW(
                ex_style,
                CLASS_NAME,
                windows::core::w!("DeskFrog"),
                WS_POPUP,
                100,
                100,
                width as i32,
                height as i32,
                None,
                None,
                Some(hinstance.into()),
                None,
            )?;

            let screen_dc = GetDC(None);
            let mem_dc = CreateCompatibleDC(Some(screen_dc));

            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // negative = top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            };

            let mut bits_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(
                Some(screen_dc),
                &bmi,
                DIB_RGB_COLORS,
                &mut bits_ptr,
                None,
                0,
            )?;
            ReleaseDC(None, screen_dc);

            let old_bitmap = SelectObject(mem_dc, bitmap.into());

            Ok(Self {
                hwnd,
                width,
                height,
                mem_dc,
                bitmap,
                old_bitmap,
                pixels: bits_ptr as *mut u8,
                tray_icon: None,
            })
        }
    }

    /// Adds a tray icon with a right/left-click context menu (About, Quit).
    /// Quit closes this window, which ends `run_message_loop_with`.
    pub fn enable_tray_icon(&mut self, tooltip: &str) {
        let icon = load_frog_icon();
        self.tray_icon = Some(icon);

        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: TRAY_ID,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: icon,
            ..Default::default()
        };
        let tip = to_wide(tooltip);
        let len = tip.len().min(data.szTip.len() - 1);
        data.szTip[..len].copy_from_slice(&tip[..len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &data);
        }
    }

    /// Copies `frame` into the DIB section and atomically repositions + repaints
    /// the window via UpdateLayeredWindow. `frame` must match this window's size.
    pub fn present(&self, frame: &Frame, screen_x: i32, screen_y: i32) {
        debug_assert_eq!(frame.width, self.width);
        debug_assert_eq!(frame.height, self.height);
        unsafe {
            std::ptr::copy_nonoverlapping(
                frame.bgra.as_ptr(),
                self.pixels,
                frame.bgra.len(),
            );

            let screen_dc = GetDC(None);
            let pos = POINT {
                x: screen_x,
                y: screen_y,
            };
            let size = SIZE {
                cx: self.width as i32,
                cy: self.height as i32,
            };
            let src_pos = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            let _ = UpdateLayeredWindow(
                self.hwnd,
                Some(screen_dc),
                Some(&pos),
                Some(&size),
                Some(self.mem_dc),
                Some(&src_pos),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
            ReleaseDC(None, screen_dc);
        }
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    /// Starts a repeating timer that posts WM_TIMER messages into this window's queue.
    pub fn start_timer(&self, interval_ms: u32) {
        unsafe {
            let _ = SetTimer(Some(self.hwnd), 1, interval_ms, None);
        }
    }

    /// Runs the Win32 message loop until WM_QUIT, invoking `on_tick` on every WM_TIMER
    /// and presenting the frame + position it returns. Blocks the calling thread.
    pub fn run_message_loop_with(&self, mut on_tick: impl FnMut() -> (Frame, i32, i32)) {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if msg.message == WM_TIMER {
                    let (frame, x, y) = on_tick();
                    self.present(&frame, x, y);
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for FrogWindow {
    fn drop(&mut self) {
        unsafe {
            if self.tray_icon.is_some() {
                let data = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: TRAY_ID,
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_DELETE, &data);
            }
            SelectObject(self.mem_dc, self.old_bitmap);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.mem_dc);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_TRAYICON => {
            let event = lparam.0 as u32;
            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                unsafe { show_tray_menu(hwnd) };
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            match wparam.0 & 0xFFFF {
                ID_TRAY_ABOUT => unsafe {
                    let _ = MessageBoxW(Some(hwnd), ABOUT_TEXT, ABOUT_TITLE, MB_OK | MB_ICONINFORMATION);
                },
                ID_TRAY_QUIT => unsafe {
                    let _ = DestroyWindow(hwnd);
                },
                _ => {}
            }
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Standard Win32 tray-menu recipe: the owner window must briefly become the
/// foreground window for the popup to behave correctly (dismiss on outside
/// click), and a trailing WM_NULL works around a documented Explorer quirk
/// where the menu can otherwise fail to close.
unsafe fn show_tray_menu(hwnd: HWND) {
    unsafe {
        let Ok(hmenu) = CreatePopupMenu() else { return };
        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_ABOUT, windows::core::w!("About"));
        let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_QUIT, windows::core::w!("Quit"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), hwnd, None);
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(hmenu);
    }
}
