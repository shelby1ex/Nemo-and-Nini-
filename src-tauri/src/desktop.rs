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
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowLongW,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindow, IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
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

/* ── Agent Hub helpers ───────────────────────────────────────────────────
   Three real things the hub asks of the desktop: bring a window forward
   ("click a step to jump to its window"), snap the screen ("+ Screen"
   context), read the clipboard ("+ Clipboard" context). All of it goes
   through Win32 — the web layer never gets raw OS access. */

/// Bring a real window to the front. `id` is the window id the desktop
/// watcher already reports (the same one the hub's boxes are keyed by).
#[tauri::command]
pub fn focus_window(id: i64) -> Result<(), String> {
    use windows_sys::Win32::System::Threading::AttachThreadInput;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };
    unsafe {
        let hwnd = id as isize as HWND;
        if hwnd.is_null() || IsWindow(hwnd) == 0 {
            return Err("that window is gone".into());
        }
        if IsIconic(hwnd) != 0 {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        /* Windows refuses SetForegroundWindow from a process that is not
           foreground. The documented workaround is to share input state with
           the foreground thread for the moment of the switch. */
        let fg = GetForegroundWindow();
        let mut t_fg = 0u32;
        if !fg.is_null() {
            t_fg = GetWindowThreadProcessId(fg, std::ptr::null_mut());
        }
        let t_me = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        if t_fg != 0 {
            AttachThreadInput(t_fg, GetCurrentThreadId(), 1);
        }
        if t_me != 0 && t_me != t_fg {
            AttachThreadInput(t_me, GetCurrentThreadId(), 1);
        }
        let ok = SetForegroundWindow(hwnd);
        if t_fg != 0 {
            AttachThreadInput(t_fg, GetCurrentThreadId(), 0);
        }
        if t_me != 0 && t_me != t_fg {
            AttachThreadInput(t_me, GetCurrentThreadId(), 0);
        }
        let _ = BringWindowToTop(hwnd);
        if ok == 0 && GetForegroundWindow() != hwnd {
            return Err("Windows refused to bring that window forward".into());
        }
        Ok(())
    }
}

/// Snap the primary monitor into a PNG in the app's data folder and hand
/// back its path — the hub attaches it to the next task so the agent can
/// actually look at the screen.
#[tauri::command]
pub fn capture_screen(app: tauri::AppHandle) -> Result<String, String> {
    use windows_sys::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HGDIOBJ, SRCCOPY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    unsafe {
        let w = GetSystemMetrics(SM_CXSCREEN);
        let h = GetSystemMetrics(SM_CYSCREEN);
        if w <= 0 || h <= 0 {
            return Err("no screen to capture".into());
        }
        let screen = GetDC(std::ptr::null_mut());
        if screen.is_null() {
            return Err("no desktop DC".into());
        }
        let mem = CreateCompatibleDC(screen);
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let prev = SelectObject(mem, bmp as HGDIOBJ);
        BitBlt(mem, 0, 0, w, h, screen, 0, 0, SRCCOPY);

        let bmi = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        let mut info: BITMAPINFO = std::mem::zeroed();
        info.bmiHeader = bmi;
        let got = GetDIBits(
            mem,
            bmp,
            0,
            h as u32,
            pixels.as_mut_ptr() as *mut c_void,
            &mut info as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        );

        SelectObject(mem, prev);
        DeleteObject(bmp as HGDIOBJ);
        DeleteDC(mem);
        ReleaseDC(std::ptr::null_mut(), screen);
        if got == 0 {
            return Err("could not read the screen".into());
        }

        /* pixels are BGRA, bottom-up (the DC's own order): flip to RGBA,
           top-down, and write through the png crate. */
        let dir = crate::settings::config_dir(&app).join("screenshots");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let path = dir.join(format!("screen-{stamp}.png"));
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        let mut out = vec![0u8; pixels.len()];
        for y in 0..h as usize {
            let src = y * w as usize * 4;
            for x in 0..w as usize {
                let s = src + x * 4;
                let d = src + x * 4;
                out[d] = pixels[s + 2]; // R
                out[d + 1] = pixels[s + 1]; // G
                out[d + 2] = pixels[s]; // B
                out[d + 3] = 255;
            }
        }
        writer.write_image_data(&out).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    }
}

/// The clipboard's plain text, as the hub's "+ Clipboard" chip. Read once,
/// handed to the agent as part of the next task — nothing is kept anywhere.
#[tauri::command]
pub fn clipboard_text() -> Result<String, String> {
    use windows_sys::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows_sys::Win32::System::Ole::CF_UNICODETEXT;
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("the clipboard is busy".into());
        }
        let out = (|| -> Result<String, String> {
            let h = GetClipboardData(CF_UNICODETEXT.into());
            if h.is_null() {
                return Err("there is no text on the clipboard".into());
            }
            let size = GlobalSize(h);
            if size < 2 {
                return Err("there is no text on the clipboard".into());
            }
            let p = GlobalLock(h) as *const u16;
            if p.is_null() {
                return Err("the clipboard is locked".into());
            }
            let units = size / 2;
            let mut len = 0;
            while len < units && *p.add(len) != 0 {
                len += 1;
            }
            let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
            GlobalUnlock(h);
            Ok(s)
        })();
        CloseClipboard();
        out
    }
}
