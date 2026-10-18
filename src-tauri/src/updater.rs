//! Getting the opencode runtime in and out of date, straight from Settings.
//!
//! The app does not ship the engine inside the workspace: the binary lives in
//! the app's own data folder and Settings fetches it — **Download** when it is
//! missing, **Update engine** when GitHub has a newer release. The flow follows
//! the reference gateway's updater step for step:
//!
//! 1. check `anomalyco/opencode`'s latest release,
//! 2. download the `opencode-windows-x64.zip` asset into a hidden staging
//!    folder (streamed, with progress events),
//! 3. extract the binary and verify it actually runs (`--version`),
//! 4. swap it in — Windows lets a running exe be renamed, so the old engine
//!    keeps its image and the new version takes effect on the next start,
//! 5. if the engine was running, restart it; if the new binary does not come
//!    up, roll back to the previous one and say so.
//!
//! Every step re-emits an `oc:update` event so the Settings window can show
//! exactly where it is without polling.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings;

/// Progress/state events for the update flow.
pub const EV: &str = "oc:update";

const RELEASE_API: &str = "https://api.github.com/repos/anomalyco/opencode/releases/latest";
/// How long one check stays valid, so opening Settings never hammers GitHub.
const CHECK_TTL: Duration = Duration::from_secs(600);
/// States in which the UI should disable the update buttons.
const BUSY: [&str; 5] = ["checking", "downloading", "verifying", "swapping", "restarting"];

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    /// idle | checking | downloading | verifying | swapping | restarting |
    /// uptodate | applied | error | rolled_back
    pub state: String,
    /// Installed version, empty when nothing is installed (or not probed yet).
    pub current: String,
    /// Latest version GitHub reports, empty until the first check lands.
    pub latest: String,
    /// Raw detail for error states (English; the UI translates the frame).
    pub message: String,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub busy: bool,
    /// Is there a runtime the engine could be started with right now?
    pub installed: bool,
}

struct Release {
    version: String,
    url: String,
}

pub struct Inner {
    st: UpdateStatus,
    /// `current` has been probed at least once (or the runtime is absent).
    probed: bool,
    release: Option<Release>,
    checked_at: Option<Instant>,
}

pub struct UpdaterState(pub Mutex<Inner>);

impl UpdaterState {
    pub fn new() -> Self {
        Self(Mutex::new(Inner {
            st: UpdateStatus {
                state: "idle".into(),
                ..Default::default()
            },
            probed: false,
            release: None,
            checked_at: None,
        }))
    }
}

// ─────────────────────────────── paths ───────────────────────────────

/// Where the app-managed copy of the engine lives — outside the workspace,
/// next to the engine's own data, so nothing is ever written into your repo.
pub fn runtime_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| settings::config_dir(app))
        .join("runtime")
}

pub fn managed_exe(app: &AppHandle) -> PathBuf {
    runtime_dir(app).join("opencode.exe")
}

fn prev_of(exe: &Path) -> PathBuf {
    exe.with_extension("prev.exe")
}

fn failed_of(exe: &Path) -> PathBuf {
    exe.with_extension("failed.exe")
}

// ─────────────────────────────── state ───────────────────────────────

/// Mutate the status and fan the new snapshot out to every window.
fn with(app: &AppHandle, f: impl FnOnce(&mut UpdateStatus)) {
    let st = app.state::<UpdaterState>();
    let snap = {
        let mut g = st.0.lock().unwrap();
        f(&mut g.st);
        g.st.clone()
    };
    let _ = app.emit(EV, snap);
}

fn busy(app: &AppHandle) -> bool {
    let st = app.state::<UpdaterState>();
    let g = st.0.lock().unwrap();
    g.st.busy || BUSY.contains(&g.st.state.as_str())
}

/// The current snapshot, probing the installed version the first time it is
/// asked for (and only then — running the binary on every poll would be silly).
pub fn status(app: &AppHandle) -> UpdateStatus {
    let need = {
        let st = app.state::<UpdaterState>();
        let g = st.0.lock().unwrap();
        !g.probed
    };
    if need {
        let installed = crate::opencode::resolve_runtime(app, &settings::load(app)).is_ok();
        let current = installed
            .then(|| probe_current(app))
            .flatten()
            .unwrap_or_default();
        let st = app.state::<UpdaterState>();
        let mut g = st.0.lock().unwrap();
        if !g.probed {
            g.st.current = current;
            g.st.installed = installed;
            g.probed = true;
        }
    }
    let st = app.state::<UpdaterState>();
    let g = st.0.lock().unwrap();
    g.st.clone()
}

fn mark_installed(app: &AppHandle, version: &str) {
    let st = app.state::<UpdaterState>();
    let mut g = st.0.lock().unwrap();
    g.st.current = version.to_string();
    // An empty version means the rollback restored "nothing" — a fresh install
    // that failed, so the app is back to not having a runtime.
    g.st.installed = !version.is_empty();
    g.probed = true;
}

// ─────────────────────────────── versions ───────────────────────────────

/// The first `x.y.z`-shaped run in the text, tolerating a leading `v`.
fn parse_version(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            let mut last_digit = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                if chars[i].is_ascii_digit() {
                    last_digit = i;
                }
                i += 1;
            }
            let cand: String = chars[start..=last_digit].iter().collect();
            if cand.contains('.') {
                return Some(cand);
            }
        } else {
            i += 1;
        }
    }
    None
}

fn tuple_of(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|p| p.trim().parse::<u32>().ok()).collect()
}

/// Is `b` strictly newer than `a`? `""` (nothing installed) means yes.
fn newer(a: &str, b: &str) -> bool {
    if a.is_empty() {
        return true;
    }
    let (av, bv) = (tuple_of(a), tuple_of(b));
    for i in 0..av.len().max(bv.len()) {
        let x = av.get(i).copied().unwrap_or(0);
        let y = bv.get(i).copied().unwrap_or(0);
        if x != y {
            return y > x;
        }
    }
    false
}

/// Run `<runtime> --version` and pull the version out of whatever it prints.
/// Waits at most 20s and kills a binary that hangs instead of parking the UI.
fn run_version(path: &Path) -> Option<String> {
    let mut cmd = Command::new(path);
    cmd.arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().ok()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(60));
            }
            Err(_) => return None,
        }
    }
    let out = child.wait_with_output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_version(&text)
}

fn probe_current(app: &AppHandle) -> Option<String> {
    let exe = crate::opencode::resolve_runtime(app, &settings::load(app)).ok()?;
    run_version(&exe)
}

// ─────────────────────────────── GitHub ───────────────────────────────

async fn fetch_release() -> Result<Release, String> {
    let client = reqwest::Client::new();
    let r = client
        .get(RELEASE_API)
        .header("User-Agent", "nemo-nini-updater")
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("could not reach GitHub: {e}"))?;
    if !r.status().is_success() {
        return Err(format!("GitHub answered HTTP {}", r.status()));
    }
    let v: serde_json::Value = r.json().await.map_err(|e| e.to_string())?;
    let tag = v
        .get("tag_name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let version = parse_version(&tag)
        .ok_or_else(|| format!("no version in release tag: {tag}"))?;

    let assets = v.get("assets").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let mut url = String::new();
    for a in &assets {
        if a.get("name").and_then(|x| x.as_str()) == Some("opencode-windows-x64.zip") {
            url = a.get("browser_download_url").and_then(|x| x.as_str()).unwrap_or("").into();
            break;
        }
    }
    if url.is_empty() {
        for a in &assets {
            let n = a.get("name").and_then(|x| x.as_str()).unwrap_or("");
            if n.contains("windows-x64") && !n.contains("desktop") && n.ends_with(".zip") {
                url = a.get("browser_download_url").and_then(|x| x.as_str()).unwrap_or("").into();
                break;
            }
        }
    }
    if url.is_empty() {
        return Err(format!("release {tag} has no windows-x64 zip"));
    }
    Ok(Release { version, url })
}

/// The latest release, cached for CHECK_TTL so status paints never re-fetch.
async fn latest_release(app: &AppHandle) -> Result<Release, String> {
    let cached = {
        let st = app.state::<UpdaterState>();
        let g = st.0.lock().unwrap();
        match (&g.release, g.checked_at) {
            (Some(r), Some(at)) if at.elapsed() < CHECK_TTL => Some(Release {
                version: r.version.clone(),
                url: r.url.clone(),
            }),
            _ => None,
        }
    };
    if let Some(r) = cached {
        return Ok(r);
    }
    let rel = fetch_release().await?;
    {
        let st = app.state::<UpdaterState>();
        let mut g = st.0.lock().unwrap();
        g.release = Some(Release {
            version: rel.version.clone(),
            url: rel.url.clone(),
        });
        g.checked_at = Some(Instant::now());
        g.st.latest = rel.version.clone();
    }
    Ok(rel)
}

// ─────────────────────────────── file work ───────────────────────────────

async fn download(app: &AppHandle, url: &str, dest: &Path) -> Result<(), String> {
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let client = reqwest::Client::new();
    let r = client
        .get(url)
        .header("User-Agent", "nemo-nini-updater")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("download failed: {e}"))?;
    if !r.status().is_success() {
        return Err(format!("download answered HTTP {}", r.status()));
    }
    let total = r.content_length().unwrap_or(0);
    with(app, |s| {
        s.bytes_total = total;
        s.bytes_done = 0;
    });

    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut stream = r.bytes_stream();
    let mut done: u64 = 0;
    let mut last_emit = Instant::now();
    while let Some(Ok(chunk)) = stream.next().await {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(150) {
            last_emit = Instant::now();
            with(app, |s| s.bytes_done = done);
        }
    }
    let _ = file.flush();
    with(app, |s| s.bytes_done = done);
    Ok(())
}

/// Pull the engine's exe out of the zip: `opencode.exe` if the archive names
/// it exactly, otherwise the first exe that isn't a directory entry.
fn extract_exe(zip_path: &Path, staging: &Path) -> Result<PathBuf, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut pick: Option<String> = None;
    for i in 0..archive.len() {
        let Ok(entry) = archive.by_index(i) else { continue };
        let name = entry.name().to_string();
        if name.ends_with('/') || !name.to_ascii_lowercase().ends_with(".exe") {
            continue;
        }
        let base = name.rsplit(['/', '\\']).next().unwrap_or("").to_ascii_lowercase();
        if base == "opencode.exe" {
            pick = Some(name);
            break;
        }
        if pick.is_none() {
            pick = Some(name);
        }
    }
    let name = pick.ok_or("no .exe inside the downloaded zip")?;
    let mut entry = archive.by_name(&name).map_err(|e| e.to_string())?;

    std::fs::create_dir_all(staging).map_err(|e| e.to_string())?;
    let target = staging.join("opencode.exe");
    if target.exists() {
        let _ = std::fs::remove_file(&target);
    }
    let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    Ok(target)
}

/// Rename across folders when we can (instant), copy when we must (a runtime
/// staged on another volume).
fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    std::fs::copy(from, to).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(from);
    Ok(())
}

/// Put the staged binary in place. The running engine keeps its (renamed)
/// old image, so the swap never needs the engine to be stopped first.
fn swap(staged: &Path, exe: &Path) -> Result<(), String> {
    if let Some(dir) = exe.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let prev = prev_of(exe);
    if prev.exists() {
        let _ = std::fs::remove_file(&prev);
    }
    let mut renamed = false;
    if exe.exists() {
        std::fs::rename(exe, &prev)
            .map_err(|e| format!("could not set the current runtime aside: {e}"))?;
        renamed = true;
    }
    match move_file(staged, exe) {
        Ok(()) => Ok(()),
        Err(e) => {
            if renamed {
                let _ = std::fs::rename(&prev, exe);
            }
            Err(e)
        }
    }
}

/// Undo a swap: current image aside, previous image back, wreckage removed.
fn rollback(exe: &Path) -> Result<(), String> {
    let prev = prev_of(exe);
    if !prev.is_file() {
        return Err("no previous runtime to restore".into());
    }
    let failed = failed_of(exe);
    let mut had_current = false;
    if exe.exists() {
        let _ = std::fs::remove_file(&failed);
        std::fs::rename(exe, &failed)
            .map_err(|e| format!("could not set the failed runtime aside: {e}"))?;
        had_current = true;
    }
    match std::fs::rename(&prev, exe) {
        Ok(()) => {
            if had_current {
                let _ = std::fs::remove_file(failed);
            }
            Ok(())
        }
        Err(e) => {
            if had_current {
                let _ = std::fs::rename(&failed, exe);
            }
            Err(format!("rollback failed: {e}"))
        }
    }
}

// ─────────────────────────────── the flow ───────────────────────────────

async fn run_flow(app: &AppHandle) -> Result<(), String> {
    // 1 — which release, and is it actually newer than what we have?
    let cur = status(app);
    let rel = latest_release(app).await?;
    if cur.installed && !newer(&cur.current, &rel.version) {
        with(app, |s| {
            s.state = "uptodate".into();
            s.latest = rel.version.clone();
            s.busy = false;
        });
        return Ok(());
    }

    // 2 — download into staging.
    let staging = runtime_dir(app).join("staging");
    let zip = staging.join("opencode-update.zip");
    download(app, &rel.url, &zip).await.map_err(|e| {
        let _ = std::fs::remove_file(&zip);
        e
    })?;

    // 3 — extract and prove it runs before anything is replaced.
    with(app, |s| {
        s.state = "verifying".into();
        s.bytes_done = 0;
        s.bytes_total = 0;
    });
    let staged = extract_exe(&zip, &staging);
    let _ = std::fs::remove_file(&zip);
    let staged = staged?;
    let staged_ver = run_version(&staged).ok_or("the downloaded engine does not run (--version failed)")?;

    // 4 — swap it in where the engine actually reads it from. An explicit
    //     runtime path in Settings is updated in place; otherwise the managed
    //     copy (or a fresh install of it) is the target.
    let target = crate::opencode::resolve_runtime(app, &settings::load(app))
        .unwrap_or_else(|_| managed_exe(app));
    with(app, |s| s.state = "swapping".into());
    if let Err(e) = swap(&staged, &target) {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }
    mark_installed(app, &staged_ver);

    // 5 — a running engine still holds the old image; restart it so the new
    //     version takes effect now, and roll back if it does not come up.
    let was_running = crate::opencode::info(app).running;
    if !was_running {
        with(app, |s| {
            s.state = "applied".into();
            s.latest = staged_ver;
            s.busy = false;
            s.message.clear();
        });
        return Ok(());
    }

    with(app, |s| s.state = "restarting".into());
    match crate::opencode::start(app).await {
        Ok(_) => {
            with(app, |s| {
                s.state = "applied".into();
                s.latest = staged_ver;
                s.busy = false;
                s.message.clear();
            });
            Ok(())
        }
        Err(e) => {
            with(app, |s| s.state = "swapping".into());
            let back = rollback(&target);
            mark_installed(app, &cur.current);
            let _ = crate::opencode::start(app).await; // bring the old one back
            let detail = match back {
                Ok(()) => format!("the new engine did not start: {e}"),
                Err(r) => format!("the new engine did not start ({e}) and {r}"),
            };
            Err(detail)
        }
    }
}

// ─────────────────────────────── commands ───────────────────────────────

/// Snapshot for the update card. `async` so the one-off version probe never
/// parks the UI thread.
#[tauri::command(async)]
pub fn oc_update_status(app: AppHandle) -> UpdateStatus {
    status(&app)
}

/// Ask GitHub for the latest release (cached). Leaves state `uptodate` when
/// there is nothing to do, `idle` with `latest` set when there is.
#[tauri::command]
pub async fn oc_update_check(app: AppHandle) -> Result<UpdateStatus, String> {
    if busy(&app) {
        return Ok(status(&app));
    }
    with(&app, |s| {
        s.state = "checking".into();
        s.busy = true;
        s.message.clear();
        s.bytes_done = 0;
        s.bytes_total = 0;
    });
    let cur = status(&app);
    let res = async {
        let rel = latest_release(&app).await?;
        Ok::<_, String>((cur, rel))
    }
    .await;
    match res {
        Ok((cur, rel)) => {
            let up_to_date = cur.installed && !newer(&cur.current, &rel.version);
            with(&app, |s| {
                s.state = if up_to_date { "uptodate" } else { "idle" }.into();
                s.busy = false;
            });
            Ok(status(&app))
        }
        Err(e) => {
            with(&app, |s| {
                s.state = "error".into();
                s.busy = false;
                s.message = e.clone();
            });
            Err(e)
        }
    }
}

/// The whole flow: check, download, verify, swap, restart, roll back on
/// failure. Long-running on purpose — it is an async command, so it never
/// touches the UI thread, and it reports progress through `oc:update`.
#[tauri::command]
pub async fn oc_update_run(app: AppHandle) -> Result<UpdateStatus, String> {
    if busy(&app) {
        return Ok(status(&app));
    }
    with(&app, |s| {
        s.state = "checking".into();
        s.busy = true;
        s.message.clear();
        s.bytes_done = 0;
        s.bytes_total = 0;
    });
    let res = run_flow(&app).await;
    match res {
        Ok(()) => Ok(status(&app)),
        Err(e) => {
            with(&app, |s| {
                s.state = "error".into();
                s.busy = false;
                s.message = e.clone();
            });
            Err(e)
        }
    }
}
