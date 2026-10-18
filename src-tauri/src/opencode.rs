//! The AI engine: a locally spawned `opencode serve`.
//!
//! The runtime itself is **not** shipped inside the workspace — Settings
//! downloads and updates it (`updater.rs`), and it lives in the app's own data
//! folder. Rust owns the process — it starts `opencode serve` inside an
//! isolated data directory, watches its health, subscribes to the server's SSE
//! stream and re-emits every event to the webviews. The front end never speaks
//! HTTP itself, so there is no CORS story, no port in the UI, and nothing to
//! leak.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use futures_util::StreamExt;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::{self, Settings};

pub const EV: &str = "oc:event";
pub const ST: &str = "oc:status";

const NAMES: [&str; 3] = ["vexis-runtime.prev.exe", "vexis-runtime.exe", "opencode.exe"];

#[derive(Default)]
pub struct Engine {
    child: Option<Child>,
    port: u16,
    running: bool,
    version: String,
    error: String,
    log: Vec<String>,
    #[cfg(windows)]
    job: Option<job::Job>,
}

pub struct EngineState(pub Mutex<Engine>);

impl EngineState {
    pub fn new() -> Self {
        Self(Mutex::new(Engine::default()))
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub running: bool,
    pub port: u16,
    pub version: String,
    pub error: String,
    pub path: String,
    pub log: Vec<String>,
}

/// Kill the child when this process dies, however it dies (Ctrl+C included).
#[cfg(windows)]
mod job {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub struct Job(pub HANDLE);
    // The handle is only ever used from one thread at a time.
    unsafe impl Send for Job {}
    unsafe impl Sync for Job {}

    pub fn create() -> Option<Job> {
        unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if h.is_null() {
                return None;
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if ok == 0 {
                return None;
            }
            Some(Job(h))
        }
    }

    impl Job {
        pub fn assign(&self, child: &Child) -> bool {
            unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) != 0 }
        }
    }
}

#[cfg(not(windows))]
mod job {
    pub struct Job;
    impl Job {
        pub fn create() -> Option<Job> {
            None
        }
        pub fn assign(&self, _c: &std::process::Child) -> bool {
            false
        }
    }
}

// ─────────────────────────────── helpers ───────────────────────────────

fn push_log(g: &mut Engine, line: String) {
    if g.log.len() > 300 {
        g.log.drain(..120);
    }
    g.log.push(line);
}

fn base(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .unwrap_or(41999)
}

/// Work out which binary is the engine. A path in Settings always wins, then
/// the copy the app downloaded from Settings → Engine update, then anything
/// older lying around next to the app (kept so a manually dropped binary still
/// works).
pub fn resolve_runtime(app: &AppHandle, s: &Settings) -> Result<PathBuf, String> {
    let picked = s.runtime_path.trim();
    if !picked.is_empty() {
        let p = PathBuf::from(picked);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("Runtime not found: {}", p.display()));
    }

    let managed = crate::updater::managed_exe(app);
    if managed.is_file() {
        return Ok(managed);
    }

    let mut cands: Vec<PathBuf> = Vec::new();

    if let Ok(dir) = app.path().resource_dir() {
        for n in NAMES {
            cands.push(dir.join(n));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for n in NAMES {
                cands.push(dir.join(n));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = Some(cwd);
        for _ in 0..4 {
            let Some(d) = dir else { break };
            for n in NAMES {
                cands.push(d.join(n));
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }
    // cargo dev: the repo root sits one level above src-tauri
    cands.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("vexis-runtime.prev.exe"),
    );

    for c in cands {
        if c.is_file() {
            return Ok(c);
        }
    }
    Err("The opencode runtime is not installed. Open Settings → AI engine and press Download.".into())
}

pub fn info(app: &AppHandle) -> EngineInfo {
    let s = settings::load(app);
    let path = resolve_runtime(app, &s)
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let st = app.state::<EngineState>();
    let g = st.0.lock().unwrap();
    EngineInfo {
        running: g.running,
        port: g.port,
        version: g.version.clone(),
        error: g.error.clone(),
        path,
        log: g.log.clone(),
    }
}

fn emit(app: &AppHandle) {
    let _ = app.emit(ST, info(app));
}

/// `taskkill` is the only reliable way to take the whole tree down, but we only
/// ever call it on a PID we launched ourselves and recorded.
fn kill_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn stop(app: &AppHandle) {
    let st = app.state::<EngineState>();

    // Take the child out from under the lock, then shut it down *outside* it.
    // Taking the tree down is not instant — `kill` then `wait`, then `taskkill`
    // which waits again — and this same lock is taken by `oc_status`, which runs
    // on the UI thread. Holding it across the shutdown was enough to freeze the
    // whole app for as long as the engine took to die.
    let child = {
        let mut g = st.0.lock().unwrap();
        g.running = false;
        g.port = 0;
        #[cfg(windows)]
        {
            // Dropping the job object kills everything it holds; that is the
            // mechanism, not a side effect.
            g.job = None;
        }
        g.child.take()
    };

    if let Some(mut c) = child {
        let pid = c.id();
        let _ = c.kill();
        let _ = c.wait();
        kill_tree(pid);
    }

    emit(app);
}

fn kill_stale(app: &AppHandle) {
    let p = settings::config_dir(app).join("engine.pid");
    let Ok(txt) = std::fs::read_to_string(&p) else {
        return;
    };
    let _ = std::fs::remove_file(&p);
    let Ok(pid) = txt.trim().parse::<u32>() else {
        return;
    };
    // Guard against PID reuse: only kill it if the image is still ours.
    let out = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output();
    if let Ok(o) = out {
        let txt = String::from_utf8_lossy(&o.stdout).to_lowercase();
        if txt.contains("vexis") || txt.contains("opencode") {
            kill_tree(pid);
        }
    }
}

async fn healthy(port: u16) -> Option<String> {
    let url = format!("{}/global/health", base(port));
    let r = reqwest::Client::new()
        .get(url)
        .timeout(Duration::from_millis(1500))
        .send()
        .await
        .ok()?;
    if !r.status().is_success() {
        return None;
    }
    let v: Value = r.json().await.ok()?;
    Some(v.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string())
}

/// Start the engine and wait until it answers. Takes the process from any
/// previous run down first, so calling this twice is safe.
pub async fn start(app: &AppHandle) -> Result<EngineInfo, String> {
    stop(app);
    kill_stale(app);

    let s = settings::load(app);
    let exe = resolve_runtime(app, &s)?;
    let port = if s.port == 0 { free_port() } else { s.port };

    let (data, config) = settings::engine_dirs(app);
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;

    let cwd = if s.workspace.trim().is_empty() {
        app.path()
            .home_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
    } else {
        PathBuf::from(s.workspace.trim())
    };

    let mut cmd = Command::new(&exe);
    cmd.arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .arg("--hostname")
        .arg("127.0.0.1")
        .current_dir(&cwd)
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", &config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Could not start {}: {e}", exe.display()))?;

    let pid = child.id();
    if let Err(e) = std::fs::write(settings::config_dir(app).join("engine.pid"), pid.to_string()) {
        let _ = e;
    }

    // Drain stderr into the engine log the Settings window shows.
    if let Some(err) = child.stderr.take() {
        let handle = app.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(err);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                let st = handle.state::<EngineState>();
                let mut g = st.0.lock().unwrap();
                push_log(&mut g, line);
            }
        });
    }

    #[cfg(windows)]
    let job = job::create();
    #[cfg(windows)]
    if let Some(j) = &job {
        j.assign(&child);
    }

    {
        let st = app.state::<EngineState>();
        let mut g = st.0.lock().unwrap();
        g.child = Some(child);
        g.port = port;
        g.error.clear();
        push_log(&mut g, format!("starting {} on port {port}", exe.display()));
        #[cfg(windows)]
        {
            g.job = job;
        }
    }

    // Wait for it to come up.
    let mut version = None;
    for _ in 0..60 {
        if let Some(v) = healthy(port).await {
            version = Some(v);
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    {
        let st = app.state::<EngineState>();
        let mut g = st.0.lock().unwrap();
        match version {
            Some(v) => {
                g.running = true;
                g.version = v.clone();
                g.error.clear();
                push_log(&mut g, format!("engine ready (version {v})"));
            }
            None => {
                g.running = false;
                g.error = "The engine did not answer in time.".into();
                push_log(&mut g, "engine failed to answer /global/health".into());
            }
        }
    }

    let out = info(app);
    emit(app);
    if out.running {
        Ok(out)
    } else {
        Err(out.error.clone())
    }
}

fn port_of(app: &AppHandle) -> Result<u16, String> {
    let st = app.state::<EngineState>();
    let g = st.0.lock().unwrap();
    if !g.running || g.port == 0 {
        return Err("The AI engine is not running.".into());
    }
    Ok(g.port)
}

async fn req(app: &AppHandle, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value, String> {
    let port = port_of(app)?;
    let mut rb = reqwest::Client::new()
        .request(method, format!("{}{}", base(port), path));
    if let Some(b) = body {
        rb = rb.json(&b);
    }
    let r = rb.timeout(Duration::from_secs(60)).send().await.map_err(|e| e.to_string())?;
    let status = r.status();
    let text = r.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {status}: {}", text.trim()));
    }
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

// ─────────────────────────────── commands ───────────────────────────────

/// `async` on purpose. Tauri runs non-async commands **on the UI thread**, and
/// this one reads engine state that another thread may be holding; the UI must
/// never be able to park behind that.
#[tauri::command(async)]
pub fn oc_status(app: AppHandle) -> EngineInfo {
    info(&app)
}

#[tauri::command]
pub async fn oc_start(app: AppHandle) -> Result<EngineInfo, String> {
    start(&app).await
}

/// `async` on purpose: stopping the engine kills a process tree and waits for
/// it, which is not something the UI thread should ever be doing.
#[tauri::command(async)]
pub fn oc_stop(app: AppHandle) -> EngineInfo {
    stop(&app);
    info(&app)
}

#[tauri::command]
pub async fn oc_restart(app: AppHandle) -> Result<EngineInfo, String> {
    start(&app).await
}

#[tauri::command]
pub async fn oc_providers(app: AppHandle) -> Result<Value, String> {
    req(&app, reqwest::Method::GET, "/provider", None).await
}

#[tauri::command]
pub async fn oc_config(app: AppHandle) -> Result<Value, String> {
    req(&app, reqwest::Method::GET, "/config", None).await
}

#[tauri::command]
pub async fn oc_agents(app: AppHandle) -> Result<Value, String> {
    req(&app, reqwest::Method::GET, "/agent", None).await
}

#[tauri::command]
pub async fn oc_set_key(app: AppHandle, provider: String, key: String) -> Result<Value, String> {
    req(
        &app,
        reqwest::Method::PUT,
        &format!("/auth/{provider}"),
        Some(json!({ "type": "api", "key": key })),
    )
    .await
}

#[tauri::command]
pub async fn oc_clear_key(app: AppHandle, provider: String) -> Result<Value, String> {
    req(&app, reqwest::Method::DELETE, &format!("/auth/{provider}"), None).await
}

#[tauri::command]
pub async fn oc_set_model(app: AppHandle, model: String) -> Result<Value, String> {
    if model.trim().is_empty() {
        return Ok(Value::Null);
    }
    req(
        &app,
        reqwest::Method::PATCH,
        "/config",
        Some(json!({ "model": model })),
    )
    .await
}

#[tauri::command]
pub async fn oc_set_agent(app: AppHandle, agent: String) -> Result<Value, String> {
    if agent.trim().is_empty() {
        return Ok(Value::Null);
    }
    req(
        &app,
        reqwest::Method::PATCH,
        "/config",
        Some(json!({ "default_agent": agent })),
    )
    .await
}

#[tauri::command]
pub async fn oc_new_session(app: AppHandle, title: Option<String>) -> Result<Value, String> {
    let body = json!({ "title": title.unwrap_or_else(|| "Nemo & Nini".into()) });
    req(&app, reqwest::Method::POST, "/session", Some(body)).await
}

#[tauri::command]
pub async fn oc_messages(app: AppHandle, session: String) -> Result<Value, String> {
    req(
        &app,
        reqwest::Method::GET,
        &format!("/session/{session}/message"),
        None,
    )
    .await
}

#[tauri::command]
pub async fn oc_prompt(
    app: AppHandle,
    session: String,
    text: String,
    model: Option<String>,
    agent: Option<String>,
) -> Result<Value, String> {
    let mut body = json!({ "parts": [{ "type": "text", "text": text }] });
    if let Some(m) = model.filter(|m| !m.trim().is_empty()) {
        if let Some((p, i)) = m.split_once('/') {
            body["model"] = json!({ "providerID": p, "modelID": i });
        }
    }
    if let Some(a) = agent.filter(|a| !a.trim().is_empty()) {
        body["agent"] = json!(a);
    }
    req(
        &app,
        reqwest::Method::POST,
        &format!("/session/{session}/prompt_async"),
        Some(body),
    )
    .await
}

#[tauri::command]
pub async fn oc_abort(app: AppHandle, session: String) -> Result<Value, String> {
    req(
        &app,
        reqwest::Method::POST,
        &format!("/session/{session}/abort"),
        Some(json!({})),
    )
    .await
}

#[tauri::command]
pub async fn oc_permission(app: AppHandle, request: String, reply: String) -> Result<Value, String> {
    req(
        &app,
        reqwest::Method::POST,
        &format!("/permission/{request}/reply"),
        Some(json!({ "reply": reply })),
    )
    .await
}

// ───────────────────────────── event stream ─────────────────────────────

/// One long-lived SSE subscription to the engine, reconnecting whenever it drops.
pub fn spawn_event_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::new();
        loop {
            let port = {
                let st = app.state::<EngineState>();
                let g = st.0.lock().unwrap();
                if g.running { g.port } else { 0 }
            };
            if port == 0 {
                tokio::time::sleep(Duration::from_millis(600)).await;
                continue;
            }

            if let Ok(r) = client.get(format!("{}/event", base(port))).send().await {
                if r.status().is_success() {
                    let mut stream = r.bytes_stream();
                    let mut buf: Vec<u8> = Vec::new();
                    while let Some(Ok(chunk)) = stream.next().await {
                        buf.extend_from_slice(&chunk);
                        while let Some(i) = buf.iter().position(|b| *b == b'\n') {
                            let line: Vec<u8> = buf.drain(..=i).collect();
                            let s = String::from_utf8_lossy(&line);
                            if let Some(rest) = s.trim().strip_prefix("data:") {
                                if let Ok(v) = serde_json::from_str::<Value>(rest.trim()) {
                                    let _ = app.emit(EV, v);
                                }
                            }
                        }
                        if buf.len() > 8_000_000 {
                            buf.clear();
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(800)).await;
        }
    });
}
