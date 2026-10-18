/* =============================================================================
   APPEARANCE  —  src/appearance.js
   -----------------------------------------------------------------------------
   The one place both windows agree on how things look and which language they
   say them in.

   It reads the saved settings, paints the theme and the language onto the
   document, then stays alive for the `settings:changed` event the Rust side
   fires whenever the Settings window saves — so changing a theme or a language
   doesn't just work for the window you're looking at, it works for the island
   too, immediately.

   Both pages import this. The island's own script is a plain (non-module)
   script, so the two modules are also hung off `window.NNTheme` / `window.NNi18n`
   for anything that isn't part of a module graph.
   ========================================================================== */

import * as Theme from './theme.js';
import * as I18n from './i18n.js';

/* ── state the pages can ask for ────────────────────────────────────────── */

/** Last settings we painted from. Pages can read it; appearance.js owns writes. */
let last = null;

export function applied() {
  return last ? Object.assign({}, last) : null;
}

/* ── painting ───────────────────────────────────────────────────────────── */

/**
 * Paint theme + language. Both are optional: pass only what you have and the
 * other keeps whatever it already was, which is what lets a partial settings
 * object be pushed at it mid-flight.
 */
export function paint(settings) {
  last = Object.assign({}, last, settings || {});
  Theme.apply(last.theme || Theme.DEFAULT_THEME, last.mode || Theme.DEFAULT_MODE);
  I18n.set(last.language || 'en');
}

/* ── booting ────────────────────────────────────────────────────────────── */

let started = false;

/**
 * Read the saved settings and paint them, then follow any later change.
 *
 * This runs *before* anything else waits on the engine, so the window never
 * flashes its default look and then swaps to the chosen one.
 */
export async function init() {
  const T = window.__TAURI__;
  if (!T) return;                       // plain browser preview: nothing to do
  const invoke = (cmd, args) => T.core.invoke(cmd, args || {});

  try {
    paint(await invoke('settings_get'));
  } catch (_) {
    /* First run, or the file isn't there yet: the defaults in theme.js apply. */
    paint({});
  }

  if (!started && T.event) {
    started = true;
    T.event.listen('settings:changed', e => {
      const s = (e && e.payload) || {};
      paint(s);
      /* Let the page do anything theme- or language-specific of its own. */
      try {
        window.dispatchEvent(new CustomEvent('nn:settings', { detail: s }));
      } catch (_) { /* no DOM */ }
    });
  }

  Theme.watchSystem();
}

/* ── what the plain scripts see ─────────────────────────────────────────── */

window.NNTheme = Theme;
window.NNi18n = I18n;
window.NNApp = { init, paint, applied };

/* Start as soon as the document is parsed enough to have an <html>. Module
   scripts are deferred, so this is already safe — but the try keeps it from
   being the reason a page fails to load. */
try {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => init());
  } else {
    init();
  }
} catch (_) {
  /* no document (a test harness): the page will call init() itself */
}

export default { init, paint, applied };
