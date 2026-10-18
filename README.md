# Nemo & Nini

Two AI companions living on your real Windows desktop — a transparent,
always-on-top island that sits over the actual taskbar and reacts to your real
windows, with a local **opencode** engine behind them. Everything stays on your
machine.

```
┌─────────────────────────────────────────────────────────────┐
│  [ window ]  [ window ]                                     │
│                                                             │
│              ☺ Nemo      ✦ Nini        ← the island         │
│  ───────────────────────────────                            │
│  ▔▔▔ real taskbar ▔▔▔                                      │
└─────────────────────────────────────────────────────────────┘
```

## What it does

- **A real overlay, not a mock.** The island is a frameless, transparent,
  click-through window parked above the real taskbar. It reads the real
  desktop — open windows, the taskbar, the clock — and the two companions walk
  along it, climb onto title bars, and react to what you're doing.
- **A local AI engine.** Rust spawns `opencode serve` in an isolated data
  directory, watches its health, and re-emits every SSE event to the webviews —
  the frontend never speaks HTTP itself. Chat lives in one scrolling thread
  whose history survives restarts.
- **The engine is fetched, not bundled.** Nothing binary ships inside the
  workspace: **Settings → AI engine → Engine update** downloads the opencode
  runtime into the app's own data folder and updates it later from the same
  card (check → download → verify → swap → automatic rollback if the new
  engine doesn't come up).
- **Agent Hub.** Every line the engine emits — tool calls, reasoning, approvals
  — appears live in the hub with short status labels (Thinking, WebFetch,
  Search, Shell, Edit, …). Pending tool permissions are approved from there.
- **Free models only.** The model picker lists every provider's models whose
  cost is actually zero, grouped by provider and searchable.
- **Themes, dark/light, and real language switching.** The island and the
  Settings window share one theme system (dark and light for both) and switch
  between English, Français and العربية — including direction.

## Tech

| Layer    | What                                                              |
| -------- | ----------------------------------------------------------------- |
| Shell    | [Tauri 2](https://tauri.app) (Rust) — overlay window, tray, engine |
| Engine   | [opencode](https://opencode.ai) — downloaded/updated from Settings |
| Frontend | Vite + vanilla HTML / CSS / JS — no framework                      |
| State    | `settings.json` in the app config dir; chat persisted per session  |

## Project layout

```
index.html            the island overlay (companions, chat, Agent Hub)
settings.html         the frameless Settings window
src/
  theme.js            theme catalogue + tokens
  i18n.js             en / fr / ar dictionaries (keys = English source text)
  appearance.js       shared boot: paints theme + language on both windows
nemo and nini.html    frozen original prototype (reference only)
src/prototype/mock/   frozen prototype mock (reference only)
src-tauri/src/
  desktop.rs          overlay geometry, real-window/taskbar polling
  opencode.rs         engine process, health, SSE → Tauri events
  updater.rs          download / verify / swap / rollback of the runtime
  settings.rs         settings.json
  tray.rs             tray icon (exit / Settings)
```

## Develop

Prerequisites: [Rust](https://rustup.rs), [Node.js](https://nodejs.org).

```sh
npm install
npm run tauri dev     # vite + cargo, launches the overlay
npm run tauri build   # production bundle
```

On first launch open **Settings → AI engine → Engine update** and press
**Download** to fetch the opencode runtime — then pick a free model and talk.

## Notes

- The engine's data, keys and runtime live in the app's data folder
  (`%APPDATA%\com.nemonini.app`), never in this repository or your projects.
- `nemo and nini.html` and `src/prototype/mock/index.html` are frozen
  prototypes kept as a visual reference — don't edit or delete them.
