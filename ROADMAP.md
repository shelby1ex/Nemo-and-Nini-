# Nemo & Nini — Roadmap & Bugs

> Written 2026-10-19. All dev processes stopped this session; nothing is running.
> **Locked decisions (user):** the Agent Hub is removed **permanently** — it never comes
> back in any form. The **island carries everything**: chat, voice, approvals, activity.

---

## 🐛 Bugs — current known issues

| # | Bug | Status / notes |
|---|-----|----------------|
| B1 | **Voice agent doesn't work** (hub mic). | Root cause not investigated — the hub dies anyway. Voice is rebuilt **from scratch on the island** (see Roadmap Phase 2). Nothing of the hub's voice is salvaged. |
| B2 | **Agent Hub exists at all.** | Decision: delete entirely — `#axcss`, `#axjs`, the `#axFab` button, hub openers (`d2Hub`/`hdrHub`), its `nn:lang` listener, its `localStorage` key `nn-hub`. Phase 1. |
| B3 | **Engine latency 4–60 s variance.** | **Not our bug** — free-tier queue at the provider. Proven by A/B tests (same model+agent+prompt: 4.9 s → 57.6 s). Mitigations shipped: lean `nemo` chat agent (warm persona), "Still queued — free models can take a minute · 27s" subtitle after 20 s, instant eye animation on send, red stop button. Consistent <5 s needs a paid API key (out of scope while free-only rule stands). |
| B4 | **Aborted turns linger in the engine session.** | Interrupting keeps the partial locally, but the engine session may retain the aborted turn; after a restart `load()` can show a partial/half answer in restored history. Minor; decide policy when touching chat next (trim last turn on abort vs accept). |
| B5 | **`nemotron-3.5-lightning-free` is broken** in the engine catalog (75 s+ timeout on "hello"). | Comes from the engine's own model list; picker shows it. Optionally hard-filter known-dead free models in Settings. |
| B6 | **Everything since commit `6379316` is uncommitted.** | Hub redesign, visibility fix, talking-eyes, wait phase + heartbeat, engine warm-up, **engine start-race fix** (`opencode.rs` START_LOCK), work trail, stop button, `nemo` agent, queued-hint. Commit + push after the next tested round; keep README current. |
| B7 | Engine `engine.pid` lives in app config dir, not `engine/`. | Not a bug — was misread once. Documented so it isn't "fixed" by mistake. |

---

## 🗺 Roadmap

### Phase 1 — Kill the hub, consolidate on the island
1. **Delete the Agent Hub completely**: `#axcss`/`#axjs` blocks, `#axFab`, `d2Hub`/`hdrHub`
   openers, hub `nn:lang` listener, `nn-hub` localStorage. No grave, no toggle.
2. **Rehome what the hub alone made real** (the rest dies with it):
   - **Engine permission requests** → island approval card wired to `oc_permission`
     (real approvals, not the demo ones). The island already has the card vocabulary.
   - Activity = the **work trail in chat** (already shipped, keep).
   - Model/agent pickers stay in Settings only (hub copies die).
3. Remove hub-only i18n keys; run i18n + script checkers; build clean.

### Phase 2 — Voice on the island (the "voice agent" done right)
1. **Mic button on the island composer** — push-to-talk; STT into `#promptIn`.
2. **Speak answers aloud** (TTS) as words **stream in** — never wait for the full reply,
   that's what makes it feel alive despite B3. Nemo's talk-eyes sync with speech.
3. Speaking/idle states for both characters; mute toggle reuses the existing sound setting.
4. Voice must degrade gracefully: engine slow → they look thoughtful and say nothing
   until the first word, subtitle stays honest (existing queued-hint).

### Phase 3 — JARVIS layer (discussion first — two open questions)
- **Walk: show or real?** Do the companions *animate* walking to a window, or *actually*
  control it? (Unanswered — blocks the design.)
- **First command pack**: which of the example PC actions gets built first? (Unanswered.)
- Then: conversation view, window-control skills, reminders, walk-to-and-tap animations.

### Phase 4 — Ship discipline
- Commit each **tested** round, push, README maintained.
- Next commit after user tests: everything listed in B6.

---

### Standing rules (don't regress)
- `nemo and nini.html` + `src\prototype\mock\index.html` are frozen references — never edit.
- No mouth, no new facial parts — eye expressions only, prototype vocabulary.
- Free models only; engine never bundled (Settings → Engine downloads it).
- Theme source = thumbnails/styles only; language must really translate the UI.
- Benign on close: `Chrome_WidgetWin_0 … Error = 1412`.
