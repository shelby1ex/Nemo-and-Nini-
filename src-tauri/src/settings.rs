//! Everything the Settings window edits, in one JSON file.
//!
//! The file lives in the app's config directory (`settings.json`). Nothing
//! secret is stored here: provider API keys go straight to opencode's own auth
//! store, which we keep in an isolated directory next to our data.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    // ── AI / opencode ──────────────────────────────────────────────────
    /// Empty = the copy the app manages (downloaded from Settings). Only set
    /// this to point the engine at a different binary.
    pub runtime_path: String,
    /// 0 = pick a free port every launch.
    pub port: u16,
    pub auto_start: bool,
    /// "providerID/modelID". Empty = whatever opencode defaults to.
    pub model: String,
    /// "build", "plan", … Empty = opencode's default agent.
    pub agent: String,
    /// Working directory the engine runs in. Empty = the user's home folder.
    pub workspace: String,
    /// The opencode session the island's chat thread is using. Kept here so the
    /// conversation survives a restart; the messages themselves live in the
    /// engine's own store and are re-read from it.
    pub chat_session: String,

    // ── General ────────────────────────────────────────────────────────
    pub launch_at_startup: bool,
    pub sound: bool,
    pub volume: f32,
    pub language: String,

    // ── Companions ─────────────────────────────────────────────────────
    pub comp_size: f32,
    pub walk_speed: f32,
    pub both_out_on_launch: bool,

    // ── Desktop & taskbar ──────────────────────────────────────────────
    pub click_through: bool,
    pub perch_windows: bool,
    pub floor_offset: f32,

    // ── Appearance ─────────────────────────────────────────────────────
    /// Theme id from the island's catalogue ("shelby", "ocean", …). Empty = the
    /// default black-and-white.
    pub theme: String,
    /// "dark", "light" or "system" (follow Windows).
    pub mode: String,
    pub island_top: f32,
    pub island_opacity: f32,
    pub comp_opacity: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            runtime_path: String::new(),
            port: 0,
            auto_start: true,
            model: String::new(),
            agent: String::new(),
            workspace: String::new(),
            chat_session: String::new(),

            launch_at_startup: false,
            sound: true,
            volume: 0.7,
            language: "en".into(),

            comp_size: 1.0,
            walk_speed: 1.0,
            both_out_on_launch: false,

            click_through: true,
            perch_windows: true,
            floor_offset: 0.0,

            island_top: 12.0,
            island_opacity: 1.0,
            comp_opacity: 1.0,

            theme: "shelby".into(),
            mode: "dark".into(),
        }
    }
}

fn base_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn file(app: &AppHandle) -> PathBuf {
    base_dir(app).join("settings.json")
}

pub fn load(app: &AppHandle) -> Settings {
    fs::read_to_string(file(app))
        .ok()
        .and_then(|t| serde_json::from_str::<Settings>(&t).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let p = file(app);
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let txt = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(&p, txt).map_err(|e| e.to_string())
}

/// The engine's own little world: an isolated data + config directory so it can
/// never read or migrate the user's own opencode database.
pub fn engine_dirs(app: &AppHandle) -> (PathBuf, PathBuf) {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| base_dir(app))
        .join("engine");
    (base.join("data"), base.join("config"))
}

pub fn config_dir(app: &AppHandle) -> PathBuf {
    base_dir(app)
}

pub fn log_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_log_dir()
        .unwrap_or_else(|_| base_dir(app).join("logs"))
}
