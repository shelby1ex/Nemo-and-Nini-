mod desktop;
mod opencode;
mod settings;
mod tray;
mod updater;

use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use opencode::EngineState;
use settings::Settings;

// ── mouse pass-through, with a heart-beat ───────────────────────────────────
//
// The overlay decides *when* it wants the mouse: only while the cursor is over
// Nemo, Nini or the island. While it is holding the mouse it says so a few times
// a second. We keep the last report, and we notice when the reports stop — a
// frozen page, a crashed webview, a click whose `pointerup` never arrived — and
// then take the mouse back.
//
// The release is done with raw Win32 from this thread, deliberately *not* through
// Tauri. A Tauri call has to be serviced by the main thread, so if the main
// thread is the thing that is wedged the release would never happen and the
// desktop would stay frozen with it. Touching the window's own extended style
// works from any thread, so one hung thread can never cost the user their mouse.
static CLOCK: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
/// When the overlay last told us it wants to keep the mouse.
static CAPTURE_BEAT: AtomicU64 = AtomicU64::new(0);
/// `true` while the overlay is holding the mouse (clicks do *not* pass through).
static CAPTURE: AtomicBool = AtomicBool::new(false);
/// The overlay's window handle, resolved once on the main thread so the watchdog
/// never has to ask the main thread for anything.
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

/// How long the overlay may go quiet while holding the mouse before we take it
/// back. Short enough to rescue the desktop, long enough to ride out a slow
/// frame or a garbage-collection pause.
const CAPTURE_GRACE_MS: u64 = 1_500;

fn now_ms() -> u64 {
    CLOCK.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Raw-Win32 pass-through. See the note above for why this is not a Tauri call.
#[cfg(windows)]
fn apply_ignore(hwnd: isize, ignore: bool) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
    };

    if hwnd == 0 {
        return;
    }
    unsafe {
        let h = hwnd as HWND;
        let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
        let ex = if ignore {
            ex | WS_EX_TRANSPARENT
        } else {
            ex & !WS_EX_TRANSPARENT
        };
        SetWindowLongPtrW(h, GWL_EXSTYLE, ex as isize);
    }
}
#[cfg(not(windows))]
fn apply_ignore(_hwnd: isize, _ignore: bool) {}

/// The overlay's handle, cached. Safe to call from the main thread; the watchdog
/// uses the cached value instead.
fn overlay_hwnd(app: &AppHandle) -> isize {
    let cached = MAIN_HWND.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let h = app
        .get_webview_window("main")
        .and_then(|w| w.hwnd().ok())
        .map(|h| h.0 as isize)
        .unwrap_or(0);
    if h != 0 {
        MAIN_HWND.store(h, Ordering::Relaxed);
    }
    h
}

/// Turn mouse pass-through on/off. `ignore = true` means "let every click reach
/// whatever is underneath". Called by the overlay whenever the answer changes,
/// and as a heart-beat while it is holding the mouse.
#[tauri::command]
fn set_click_through(app: AppHandle, ignore: bool) {
    CAPTURE.store(!ignore, Ordering::Relaxed);
    CAPTURE_BEAT.store(now_ms(), Ordering::Relaxed);
    apply_ignore(overlay_hwnd(&app), ignore);
}

/// Ask Windows for rounded corners and the dark border treatment, so the
/// frameless Settings window matches the island instead of looking pasted on.
/// Harmless on Windows 10, where the attribute is simply ignored.
#[cfg(windows)]
fn round_corners(w: &tauri::WebviewWindow) {
    use std::ffi::c_void;
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: i32 = 2;

    let Ok(h) = w.hwnd() else { return };
    let hwnd = h.0 as _;
    unsafe {
        let dark: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );
        let round: i32 = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &round as *const i32 as *const c_void,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

// ─────────────────────────── the app itself ───────────────────────────

/// Open (or focus) the Settings window.
///
/// Safe to call from any thread. The work is always handed to a worker thread
/// (`open_settings_now`) so that callers on the UI thread — the tray menu, the
/// tray icon — take exactly the same path as the island's gear.
pub(crate) fn open_settings(app: &AppHandle) {
    let h = app.clone();
    std::thread::spawn(move || open_settings_now(&h));
}

/// The actual work.
///
/// It is a real, frameless, island-styled window instead of a panel drawn in the
/// overlay. Two rules keep it from taking the app down with it, and both are
/// about *when* the work happens rather than what it is:
///
///   * the window is built from a worker thread, never from inside the UI
///     thread's own message dispatch. Creating a window spins up a whole
///     WebView2, and WebView2 creation has to wait — on the main thread — for
///     messages that only the message loop can deliver. Do that nested inside
///     the very dispatch it is waiting on and the wait can never be answered.
///     That was the freeze: click Settings and the app stopped answering, with
///     the window never appearing.
///   * exactly one thread ever shows or focuses it. Two threads racing to show
///     and focus the same window is the same kind of trouble.
///
/// Dark from the first frame, so there is no white WebView2 surface to see even
/// if the page is slow: the window's own background is set dark below, and the
/// page behind it is dark too.
fn open_settings_now(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }

    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Nemo & Nini — Settings")
        .inner_size(1060.0, 760.0)
        .min_inner_size(880.0, 560.0)
        .decorations(false)
        .transparent(false)
        .background_color(tauri::webview::Color(9, 11, 17, 255))
        .center()
        .resizable(true)
        .build();

    match built {
        Ok(w) => {
            // Say the geometry once more now that there is a real window to say it
            // to. A frameless window still carries an invisible resize border, and
            // centring is only meaningful once the frame exists, so this is where
            // the final size and position are settled.
            let _ = w.set_size(tauri::LogicalSize::new(1060.0, 760.0));
            let _ = w.center();
            #[cfg(windows)]
            round_corners(&w);
        }
        Err(e) => eprintln!("[settings] could not create the Settings window: {e}"),
    }
}

/// The frameless window's own minimise button. `async` so the resize work the
/// message goes on to do cannot block the UI thread's own dispatch.
#[tauri::command(async)]
fn minimize_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.minimize();
    }
}

pub(crate) fn quit(app: &AppHandle) {
    // Take the engine down on a worker thread, then leave. Shutting it down
    // means `kill` + `wait` + `taskkill`, and the UI thread should not be the one
    // sitting there waiting for it — that is what made Quit feel like a freeze.
    let h = app.clone();
    std::thread::spawn(move || {
        opencode::stop(&h);
        h.exit(0);
    });
}

pub(crate) fn restart_engine(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = opencode::start(&handle).await;
    });
}

/// Called by the island's gear. `async` so that nothing at all about opening a
/// window happens on the UI thread; the window itself is built on a worker
/// thread either way. See `open_settings_now` for why that matters.
#[tauri::command(async)]
fn open_settings_cmd(app: AppHandle) {
    open_settings(&app);
}

/// Close the Settings window.
///
/// `async` for the same reason `open_settings_now` is off the UI thread: tearing
/// a WebView2 down has to wait for messages the message loop delivers, so it
/// must not happen nested inside the UI thread's own dispatch.
#[tauri::command(async)]
fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}

#[tauri::command]
fn open_path(app: AppHandle, which: String) -> Result<(), String> {
    let dir = match which.as_str() {
        "config" => settings::config_dir(&app),
        "logs" => settings::log_dir(&app),
        _ => return Err("Unknown folder".into()),
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new("explorer");
    cmd.arg(dir);
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn settings_get(app: AppHandle) -> Settings {
    settings::load(&app)
}

#[tauri::command]
fn settings_save(app: AppHandle, mut settings: Settings) -> Result<(), String> {
    // The Settings window has no control for the chat session, so it must not be
    // able to wipe it: whatever is stored wins, and the chat thread keeps its
    // place across a settings save.
    settings.chat_session = settings::load(&app).chat_session;
    settings::save(&app, &settings)?;
    let _ = app.emit("settings:changed", &settings);
    Ok(())
}

#[tauri::command]
fn settings_reset(app: AppHandle) -> Result<Settings, String> {
    let fresh = Settings::default();
    settings::save(&app, &fresh)?;
    let _ = app.emit("settings:changed", &fresh);
    Ok(fresh)
}

/// Remember which engine session the island's chat thread is using, so the
/// conversation is still there after a restart.
#[tauri::command]
fn chat_session_set(app: AppHandle, session: String) -> Result<(), String> {
    let mut s = settings::load(&app);
    if s.chat_session == session {
        return Ok(());
    }
    s.chat_session = session;
    settings::save(&app, &s)
}

#[tauri::command]
fn app_quit(app: AppHandle) {
    quit(&app);
}

/// (origin x, origin y, width, height, scale) of the primary monitor.
fn monitor_info(app: &tauri::AppHandle) -> (i32, i32, i32, i32, f64) {
    if let Some(win) = app.get_webview_window("main") {
        if let Ok(Some(m)) = win.primary_monitor() {
            let p = m.position();
            let s = m.size();
            return (p.x, p.y, s.width as i32, s.height as i32, m.scale_factor());
        }
    }
    (0, 0, 1920, 1080, 1.0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(EngineState::new())
        .manage(updater::UpdaterState::new())
        .invoke_handler(tauri::generate_handler![
            set_click_through,
            open_settings_cmd,
            minimize_settings,
            close_settings,
            open_path,
            settings_get,
            settings_save,
            settings_reset,
            chat_session_set,
            app_quit,
            desktop::focus_window,
            desktop::capture_screen,
            desktop::clipboard_text,
            opencode::oc_status,
            opencode::oc_start,
            opencode::oc_stop,
            opencode::oc_restart,
            opencode::oc_providers,
            opencode::oc_config,
            opencode::oc_agents,
            opencode::oc_set_key,
            opencode::oc_clear_key,
            opencode::oc_set_model,
            opencode::oc_set_agent,
            opencode::oc_new_session,
            opencode::oc_messages,
            opencode::oc_prompt,
            opencode::oc_abort,
            opencode::oc_permission,
            updater::oc_update_status,
            updater::oc_update_check,
            updater::oc_update_run,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Turn the main window into a full-screen transparent overlay over
            // the primary monitor.
            let win = app.get_webview_window("main").expect("main window missing");
            if let Ok(Some(m)) = win.primary_monitor() {
                let p = m.position();
                let s = m.size();
                let _ = win.set_position(tauri::PhysicalPosition::new(p.x, p.y));
                let _ = win.set_size(tauri::PhysicalSize::new(s.width, s.height));
            }
            let _ = win.set_always_on_top(true);
            let _ = win.set_ignore_cursor_events(true);
            // Same thing again, straight through Win32, so the overlay starts out
            // transparent to the mouse no matter what the toolkit did or didn't do.
            apply_ignore(overlay_hwnd(&handle), true);
            let _ = win.show();

            // The notification-area icon: exit, settings, send them out.
            tray::build(&handle)?;

            // Never let the overlay hold the mouse hostage. See the note at the
            // top of this file. Nothing in this loop goes through Tauri, so it
            // keeps working even if the main thread is wedged.
            {
                let mine = overlay_hwnd(&handle);
                std::thread::spawn(move || loop {
                    std::thread::sleep(Duration::from_millis(250));
                    if !CAPTURE.load(Ordering::Relaxed) {
                        continue;
                    }
                    if now_ms().saturating_sub(CAPTURE_BEAT.load(Ordering::Relaxed))
                        <= CAPTURE_GRACE_MS
                    {
                        continue;
                    }
                    // The overlay went quiet while holding the mouse. Take it back.
                    CAPTURE.store(false, Ordering::Relaxed);
                    apply_ignore(mine, true);
                });
            }

            // One long-lived subscription to the engine's event stream.
            opencode::spawn_event_loop(handle.clone());

            // Bring the AI engine up if the user asked for it.
            if settings::load(&handle).auto_start {
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = opencode::start(&h).await;
                });
            }

            // Desktop geometry (taskbar + windows) at ~10 Hz.
            let h1 = handle.clone();
            std::thread::spawn(move || loop {
                let (ox, oy, sw, sh, scale) = monitor_info(&h1);
                let dto = desktop::snapshot(ox, oy, sw, sh, scale);
                let _ = h1.emit("desktop:update", &dto);
                std::thread::sleep(Duration::from_millis(100));
            });

            // Cursor at ~33 Hz, so hover / click-through feels live.
            let h2 = handle.clone();
            std::thread::spawn(move || loop {
                let (ox, oy, _, _, scale) = monitor_info(&h2);
                if let Some((x, y)) = desktop::cursor(ox, oy, scale) {
                    let _ = h2.emit("desktop:cursor", serde_json::json!({ "x": x, "y": y }));
                }
                std::thread::sleep(Duration::from_millis(30));
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                opencode::stop(app);
            }
        });
}
