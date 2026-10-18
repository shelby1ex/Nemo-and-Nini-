//! The little icon in the notification area. It is the only way in and out of an
//! app that deliberately has no titlebar and no taskbar button.

use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

/// Keeps the tray icon alive: Tauri removes it when the last handle drops.
#[allow(dead_code)]
pub struct Tray(pub Mutex<Option<TrayIcon>>);

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let settings_i = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
    let toggle_i = MenuItemBuilder::with_id("toggle", "Send them out / Bring them home").build(app)?;
    let restart_i = MenuItemBuilder::with_id("restart", "Restart the AI engine").build(app)?;
    let quit_i = MenuItemBuilder::with_id("quit", "Quit Nemo & Nini").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&settings_i)
        .item(&toggle_i)
        .item(&restart_i)
        .separator()
        .item(&quit_i)
        .build()?;

    let mut builder = TrayIconBuilder::with_id("nemo-nini")
        .tooltip("Nemo & Nini")
        .menu(&menu)
        .show_menu_on_left_click(true);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let tray = builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => crate::open_settings(app),
            "toggle" => {
                let _ = app.emit_to("main", "ui:toggle_out", ());
            }
            "restart" => crate::restart_engine(app),
            "quit" => crate::quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, TrayIconEvent};
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                crate::open_settings(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(Tray(Mutex::new(Some(tray))));
    Ok(())
}
