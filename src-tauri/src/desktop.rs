//! Real-desktop watcher (Windows).
//!
//! The web layer has no access to the OS, so this module reads the real
//! desktop with Win32 and hands it to the front end as CSS-pixel coordinates
//! relative to the primary monitor's top-left corner.
//!
//! It reports three things:
//!   * the taskbar height (0 when auto-hidden or docked somewhere else)
//!   * the list of real top-level windows (rect + z-order + flags)
//!   * the mouse cursor position
//!
//! All three are polled from background threads in `lib.rs`.

use std::ffi::c_void;

use serde::Serialize;
use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowLongW,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
};

/// Shell / hidden windows we never want to treat as a "real" app window.
const SKIP_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Windows.UI.Core.CoreWindow",
    "ApplicationManager_ImmersiveShellWindow",
    "XamlExplorerHostIslandWindow",
    "Microsoft.UI.Content.DesktopChildSiteBridge",
    "TaskListThumbnailWnd",
];

#[derive(Serialize, Clone, Default)]
pub struct WinDto {
    pub id: i64,
    pub title: String,
    /// Position/size in CSS px, relative to the primary monitor origin.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// 0 = topmost.
    pub z: i32,
    pub minimized: bool,
    pub focused: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct DeskDto {
    pub screen_w: f64,
    pub screen_h: f64,
    /// Height of the taskbar sticking up from the bottom, in CSS px. 0 = none.
    pub taskbar_h: f64,
    pub windows: Vec<WinDto>,
}

struct Ctx {
    out: Vec<WinDto>,
    pid: u32,
    fg: HWND,
    scale: f64,
    ox: i32,
    oy: i32,
    sw: i32,
    sh: i32,
    idx: i32,
}

extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
  unsafe {
    let ctx = &mut *(lparam as *mut Ctx);

    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    // Never report our own overlay.
    let mut wpid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut wpid);
    if wpid == ctx.pid {
        return 1;
    }
    if (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW) != 0 {
        return 1;
    }
    // Cloaked = a suspended UWP window still exists but is not on screen.
    let mut cloaked: u32 = 0;
    let _ = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as u32,
        &mut cloaked as *mut u32 as *mut c_void,
        4,
    );
    if cloaked != 0 {
        return 1;
    }

    let mut cbuf = [0u16; 256];
    let cn = GetClassNameW(hwnd, cbuf.as_mut_ptr(), cbuf.len() as i32);
    let cls = String::from_utf16_lossy(&cbuf[..cn.max(0) as usize]);
    if SKIP_CLASSES.iter().any(|c| *c == cls) {
        return 1;
    }

    let tlen = GetWindowTextLengthW(hwnd);
    if tlen <= 0 {
        return 1;
    }
    let mut tbuf = vec![0u16; (tlen + 1) as usize];
    let tn = GetWindowTextW(hwnd, tbuf.as_mut_ptr(), tbuf.len() as i32);
    let title = String::from_utf16_lossy(&tbuf[..tn.max(0) as usize]);

    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    if GetWindowRect(hwnd, &mut r) == 0 {
        return 1;
    }
    let w = r.right - r.left;
    let h = r.bottom - r.top;
    if w < 60 || h < 60 {
        return 1;
    }
    // Must intersect the primary monitor (iconic windows sit at -32000).
    if r.right <= ctx.ox
        || r.bottom <= ctx.oy
        || r.left >= ctx.ox + ctx.sw
        || r.top >= ctx.oy + ctx.sh
    {
        return 1;
    }

    ctx.out.push(WinDto {
        id: hwnd as isize as i64,
        title,
        x: (r.left - ctx.ox) as f64 / ctx.scale,
        y: (r.top - ctx.oy) as f64 / ctx.scale,
        w: w as f64 / ctx.scale,
        h: h as f64 / ctx.scale,
        z: ctx.idx,
        minimized: IsIconic(hwnd) != 0,
        focused: hwnd == ctx.fg,
    });
    ctx.idx += 1;
    1
  }
}

fn taskbar_height(oy: i32, sh: i32, scale: f64) -> f64 {
    unsafe {
        let cls: Vec<u16> = "Shell_TrayWnd\0".encode_utf16().collect();
        let hwnd = FindWindowW(cls.as_ptr(), std::ptr::null());
        if hwnd.is_null() {
            return 0.0;
        }
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        if GetWindowRect(hwnd, &mut r) == 0 {
            return 0.0;
        }
        let screen_bottom = oy + sh;
        let visible = screen_bottom - r.top;
        // Auto-hidden (pushed to the edge) or docked top/left/right: no floor.
        if visible < 6 || r.top < oy + sh / 2 {
            return 0.0;
        }
        visible as f64 / scale
    }
}

/// One full read of the desktop.
pub fn snapshot(ox: i32, oy: i32, sw: i32, sh: i32, scale: f64) -> DeskDto {
    let mut ctx = Ctx {
        out: Vec::with_capacity(32),
        pid: std::process::id(),
        fg: unsafe { GetForegroundWindow() },
        scale,
        ox,
        oy,
        sw,
        sh,
        idx: 0,
    };
    unsafe {
        EnumWindows(Some(enum_proc), &mut ctx as *mut Ctx as isize);
    }
    DeskDto {
        screen_w: sw as f64 / scale,
        screen_h: sh as f64 / scale,
        taskbar_h: taskbar_height(oy, sh, scale),
        windows: ctx.out,
    }
}

/// Cursor position in CSS px, relative to the monitor origin.
pub fn cursor(ox: i32, oy: i32, scale: f64) -> Option<(f64, f64)> {
    let mut p = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut p) == 0 {
            return None;
        }
    }
    Some(((p.x - ox) as f64 / scale, (p.y - oy) as f64 / scale))
}
