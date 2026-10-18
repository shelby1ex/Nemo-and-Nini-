/* =============================================================================
   THEMES  —  src/theme.js
   -----------------------------------------------------------------------------
   The look of the island and of Settings: a catalogue of named themes, and the
   set of tokens each one turns into.

   Both the catalogue and the derivation come from the "themes and settings"
   reference project (its static/js/theme-tokens.js catalog and the derivation
   in static/js/theme.js), taken as *styles* only — the palettes, and the way one
   palette fans out into background / surface / border / text / accent tokens.
   Nothing of that project's structure, pages or settings came with them.

   Every theme carries BOTH a dark and a light palette, which is what keeps
   "dark mode" from fighting the theme: a theme is never asked to be light, it
   is asked for the palette that suits the mode currently in force.

   The tokens are written as inline custom properties on <html>. Inline styles
   outrank any stylesheet declaration, so they override each page's own defaults
   without either page having to be rewritten around them — the pages keep the
   variable names they already had, and those names are pointed at the tokens.
   ========================================================================== */

/* Display order for the gallery. */
export const ORDER = [
  "shelby", "vertex", "matrix", "cyber", "ocean", "crimson", "sakura",
  "emerald", "obsidian", "frost", "terminal", "neon", "midnight",
  "glass", "graphite", "techno", "sunset",
  "blue", "red", "green", "purple", "pink", "cyan", "white",
];

/* Names that were used for the same palette at some point. */
const ALIASES = {
  black: "shelby",
  gray: "graphite",
  grey: "graphite",
  tech: "techno",
  hellokitty: "sakura",
  "flowers-pink": "sakura",
  minimal: "glass",
  modern: "neon",
};

/* The palettes. `accent`/`bg`/`surface` are the dark-mode brand;
   the *Light fields are the light-mode brand. `font` is an optional twist
   (a monospace theme is a different mood, so it is kept). */
export const THEMES = {
  shelby: {
    label: "Shelby",
    accent: "#e8e8ea", accent2: "#c0c0c4", bg: "#0a0a0c", surface: "#16161a",
    accentLight: "#2b2b30", accent2Light: "#55555c", bgLight: "#f7f7f8", surfaceLight: "#ffffff",
  },
  vertex: {
    label: "Vertex",
    accent: "#8a6bff", accent2: "#b18cff", bg: "#0b0a14", surface: "#16131f",
    accentLight: "#6a4ae8", accent2Light: "#8a6bff", bgLight: "#f8f7ff", surfaceLight: "#ffffff",
  },
  matrix: {
    label: "Matrix",
    accent: "#00ff66", accent2: "#39ffb0", bg: "#000807", surface: "#001009",
    accentLight: "#00a343", accent2Light: "#39ffb0", bgLight: "#ecfff4", surfaceLight: "#ffffff",
    font: "mono",
  },
  cyber: {
    label: "Cyber",
    accent: "#00e5ff", accent2: "#7cffec", bg: "#04060f", surface: "#0b0f1f",
    accentLight: "#0096b3", accent2Light: "#57e6ff", bgLight: "#f3fbff", surfaceLight: "#ffffff",
    font: "space",
  },
  ocean: {
    label: "Ocean",
    accent: "#4db8ff", accent2: "#7fd0ff", bg: "#041018", surface: "#0a1c2b",
    accentLight: "#0087e0", accent2Light: "#4db8ff", bgLight: "#f2faff", surfaceLight: "#ffffff",
  },
  crimson: {
    label: "Crimson",
    accent: "#ff5c6c", accent2: "#ff8a95", bg: "#120609", surface: "#1d0d12",
    accentLight: "#e03a4e", accent2Light: "#ff5c6c", bgLight: "#fff7f8", surfaceLight: "#ffffff",
  },
  sakura: {
    label: "Sakura",
    accent: "#ff9ecb", accent2: "#ffc0de", bg: "#140b12", surface: "#21141d",
    accentLight: "#e852ab", accent2Light: "#ff9ecb", bgLight: "#fff8fb", surfaceLight: "#ffffff",
  },
  emerald: {
    label: "Emerald",
    accent: "#34d399", accent2: "#5eead4", bg: "#021310", surface: "#06251e",
    accentLight: "#06996b", accent2Light: "#34d399", bgLight: "#f4fffb", surfaceLight: "#ffffff",
  },
  obsidian: {
    label: "Obsidian",
    accent: "#a78bfa", accent2: "#c4b5fd", bg: "#08070d", surface: "#121020",
    accentLight: "#8432e0", accent2Light: "#a78bfa", bgLight: "#faf9ff", surfaceLight: "#ffffff",
  },
  frost: {
    label: "Frost",
    accent: "#8fd0ff", accent2: "#bfe6ff", bg: "#071018", surface: "#0f1c2a",
    accentLight: "#0a85dd", accent2Light: "#8fd0ff", bgLight: "#f4f9ff", surfaceLight: "#ffffff",
  },
  terminal: {
    label: "Terminal",
    accent: "#b8ffa0", accent2: "#e0ffd0", bg: "#060a05", surface: "#0d130b",
    accentLight: "#479936", accent2Light: "#b8ffa0", bgLight: "#f7fff3", surfaceLight: "#ffffff",
    font: "mono",
  },
  neon: {
    label: "Neon",
    accent: "#ff2eff", accent2: "#00f0ff", bg: "#0b0510", surface: "#180c20",
    accentLight: "#c218d8", accent2Light: "#00b8c9", bgLight: "#fef4ff", surfaceLight: "#ffffff",
  },
  midnight: {
    label: "Midnight",
    accent: "#94a3ff", accent2: "#b8c4ff", bg: "#070a18", surface: "#0f1428",
    accentLight: "#4353d6", accent2Light: "#5666e8", bgLight: "#f5f7ff", surfaceLight: "#ffffff",
  },
  glass: {
    label: "Glass",
    accent: "#d6d6e0", accent2: "#ffffff", bg: "#0b0b10", surface: "rgba(255,255,255,0.06)",
    accentLight: "#3a3a44", bgLight: "#eef0f6", surfaceLight: "rgba(255,255,255,0.7)",
  },
  graphite: {
    label: "Graphite",
    accent: "#9aa0a6", accent2: "#c2c7cc", bg: "#0c0d0e", surface: "#141517",
    accentLight: "#55585c", accent2Light: "#787d83", bgLight: "#f3f4f5", surfaceLight: "#ffffff",
  },
  techno: {
    label: "Techno",
    accent: "#7ffbb0", accent2: "#57e6ff", bg: "#041209", surface: "#0a1f12",
    accentLight: "#2fd4a5", accent2Light: "#57e6ff", bgLight: "#f2fff8", surfaceLight: "#ffffff",
    font: "space",
  },
  sunset: {
    label: "Sunset",
    accent: "#ffb25e", accent2: "#ff7aa2", bg: "#14090f", surface: "#201118",
    accentLight: "#d84f73", accent2Light: "#e8914f", bgLight: "#fff9f2", surfaceLight: "#ffffff",
  },
  blue: {
    label: "Blue",
    accent: "#3b82f6", accent2: "#60a5fa", bg: "#060a14", surface: "#0b1424",
    accentLight: "#1d4ed8", accent2Light: "#3b82f6", bgLight: "#f5f9ff", surfaceLight: "#ffffff",
  },
  red: {
    label: "Red",
    accent: "#ef4444", accent2: "#f87171", bg: "#140607", surface: "#210a0d",
    accentLight: "#b91c1c", accent2Light: "#ef4444", bgLight: "#fff7f7", surfaceLight: "#ffffff",
  },
  green: {
    label: "Green",
    accent: "#22c55e", accent2: "#4ade80", bg: "#04120a", surface: "#0a1f12",
    accentLight: "#15803d", accent2Light: "#22c55e", bgLight: "#f4fff8", surfaceLight: "#ffffff",
  },
  purple: {
    label: "Purple",
    accent: "#a855f7", accent2: "#c084fc", bg: "#0d0713", surface: "#180d24",
    accentLight: "#9333ea", accent2Light: "#a855f7", bgLight: "#faf6ff", surfaceLight: "#ffffff",
  },
  pink: {
    label: "Pink",
    accent: "#ec4899", accent2: "#f472b6", bg: "#13060d", surface: "#200b17",
    accentLight: "#be185d", accent2Light: "#ec4899", bgLight: "#fff7fb", surfaceLight: "#ffffff",
  },
  cyan: {
    label: "Cyan",
    accent: "#06b6d4", accent2: "#22d3ee", bg: "#041014", surface: "#091c24",
    accentLight: "#0e7490", accent2Light: "#06b6d4", bgLight: "#f3fcff", surfaceLight: "#ffffff",
  },
  white: {
    label: "White",
    accent: "#e6e6ea", accent2: "#b8bcc6", bg: "#ffffff", surface: "#f4f5f7",
    accentLight: "#111114", accent2Light: "#3a3a44", bgLight: "#ffffff", surfaceLight: "#ffffff",
  },
};

/** What the island and Settings look like before anyone picks anything: the
 *  monochrome black-and-white pair they were built with. */
export const DEFAULT_THEME = "shelby";
export const DEFAULT_MODE = "dark";

const MONO = 'ui-monospace,"Cascadia Mono",Consolas,"SF Mono",Menlo,monospace';

/* ── small colour helpers ───────────────────────────────────────────────── */

function strip(hex) {
  let h = String(hex == null ? "" : hex).trim().replace("#", "");
  if (h.length === 3) h = h.split("").map((c) => c + c).join("");
  return h;
}

/** Is this something we can do arithmetic on? (glass' surface is not.) */
function isHex(v) {
  return /^[0-9a-f]{6}$/i.test(strip(v));
}

export function rgba(hex, alpha) {
  const h = strip(hex);
  if (h.length !== 6) return `rgba(0,0,0,${alpha})`;
  const n = parseInt(h, 16);
  if (Number.isNaN(n)) return `rgba(0,0,0,${alpha})`;
  return `rgba(${(n >> 16) & 255},${(n >> 8) & 255},${n & 255},${alpha})`;
}

/** Readable text on a given background — the reference's threshold, kept. */
export function textOn(bg) {
  const h = strip(bg);
  const n = parseInt(h, 16);
  if (h.length !== 6 || Number.isNaN(n)) return "#ffffff";
  const r = (n >> 16) & 255, g = (n >> 8) & 255, b = n & 255;
  const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return lum > 0.55 ? "#111111" : "#ffffff";
}

/* ── looking themes up ──────────────────────────────────────────────────── */

export function canonical(id) {
  const s = String(id == null ? "" : id).trim().toLowerCase();
  if (!s) return DEFAULT_THEME;
  const alias = ALIASES[s];
  if (alias) return alias;
  return THEMES[s] ? s : DEFAULT_THEME;
}

export function get(id) {
  return THEMES[canonical(id)];
}

export function list() {
  return ORDER.map((id) => Object.assign({ id }, THEMES[id]));
}

/** "dark" | "light" — resolving "system" against Windows. */
export function resolveMode(mode) {
  const m = String(mode == null ? "" : mode).toLowerCase();
  if (m === "dark" || m === "light") return m;
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  } catch (_) {
    return DEFAULT_MODE;
  }
}

/** A theme's brand palette for one mode. */
export function paletteFor(id, mode) {
  const t = get(id);
  const dark = resolveMode(mode) === "dark";
  return {
    id: canonical(id),
    label: t.label,
    font: t.font || "",
    accent: dark ? t.accent : (t.accentLight || t.accent),
    accent2: dark ? (t.accent2 || t.accent) : (t.accent2Light || t.accent2 || t.accent),
    bg: dark ? t.bg : (t.bgLight || t.bg),
    surface: dark ? t.surface : (t.surfaceLight || t.surface),
    dark,
  };
}

/* ── the token set ──────────────────────────────────────────────────────── */

/**
 * One palette turned into the full set of tokens both pages are styled from.
 * The spread of shades (and which ones exist at all) mirrors the reference's
 * `setPaletteVars`, so a theme reads the same here as it does there.
 */
export function tokens(id, mode) {
  const p = paletteFor(id, mode);
  const dark = p.dark;
  const text = textOn(p.bg);
  const surf = isHex(p.surface) ? p.surface : null;

  return {
    "--accent": p.accent,
    "--accent-2": p.accent2,
    "--accent-contrast": textOn(p.accent),
    "--accent-soft": rgba(p.accent, 0.12),
    "--accent-glow": rgba(p.accent, 0.34),

    "--bg": p.bg,
    "--bg-elevated": surf || p.surface,
    "--surface": rgba(p.accent, 0.04),
    "--surface-solid": p.surface,
    "--surface-hover": rgba(p.accent, 0.08),
    "--surface-active": rgba(p.accent, 0.13),

    "--border": rgba(p.accent, 0.12),
    "--border-strong": rgba(p.accent, 0.22),
    "--border-light": rgba(p.accent, 0.06),
    "--hairline": rgba(p.accent, 0.055),

    "--text": text,
    "--text-dim": rgba(text, 0.65),
    "--text-faint": rgba(text, 0.45),

    "--track": rgba(text, 0.14),
    "--track-hover": rgba(text, 0.24),

    "--danger": dark ? "#ff6b81" : "#cc2f46",
    "--success": dark ? "#4ade80" : "#16a34a",
    "--warning": dark ? "#f5b84b" : "#a05a08",
    "--info": dark ? "#64b5f6" : "#1d6fd1",
    "--danger-soft": dark ? "rgba(255,107,129,.12)" : "rgba(204,47,70,.12)",
    "--success-soft": dark ? "rgba(74,222,128,.12)" : "rgba(22,163,74,.12)",
    "--warning-soft": dark ? "rgba(245,184,75,.14)" : "rgba(160,90,8,.14)",

    "--code-bg": dark ? rgba(text, 0.06) : rgba(text, 0.05),
    "--layer": dark ? "rgba(0,0,0,.62)" : "rgba(0,0,0,.32)",
    "--focus-ring": rgba(p.accent, 0.28),
    "--shadow": dark
      ? "0 18px 40px -16px rgba(0,0,0,.9)"
      : "0 18px 40px -16px rgba(20,24,40,.28)",
    "--shadow-sm": dark ? "0 2px 8px rgba(0,0,0,.5)" : "0 2px 8px rgba(20,24,40,.14)",
    "--island-bg": p.bg,
    "--island-sheen": rgba(text, 0.06),
  };
}

/* ── applying it ────────────────────────────────────────────────────────── */

/* What is on screen now, and what was asked for — they differ when the mode is
   "system", and knowing both is what lets us follow Windows later. */
let current = { theme: DEFAULT_THEME, mode: DEFAULT_MODE, requested: DEFAULT_MODE };

export function currentTheme() {
  return Object.assign({}, current);
}

/**
 * Paint a theme onto the document. Everything lands on <html> as inline custom
 * properties, which is why the pages need no knowledge of the theme at all —
 * they just use the variable names they always did.
 */
export function apply(id, mode) {
  const theme = canonical(id);
  const resolved = resolveMode(mode);
  current = {
    theme,
    mode: resolved,
    requested: String(mode == null ? "" : mode).toLowerCase() || DEFAULT_MODE,
  };

  const root = document.documentElement;
  const vars = tokens(theme, resolved);
  for (const k in vars) root.style.setProperty(k, vars[k]);

  root.dataset.theme = theme;
  root.dataset.mode = resolved;

  const font = THEMES[theme].font;
  /* A mono theme replaces the font wholesale, the way it does in the
     reference catalogue — it is part of the theme, not a separate setting. */
  if (font === "mono") root.style.setProperty("--font", MONO);
  else root.style.removeProperty("--font");

  try {
    window.dispatchEvent(new CustomEvent("nn:theme", { detail: currentTheme() }));
  } catch (_) { /* no DOM in a test harness */ }
}

/** While the mode is "system", repaint when Windows changes its mind. */
export function watchSystem() {
  try {
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => {
        if (current.requested === "system") apply(current.theme, "system");
      });
  } catch (_) { /* older engines */ }
}

/* ── the gallery thumbnail ──────────────────────────────────────────────── */

/**
 * One theme drawn as a tiny chat window: a sidebar, a couple of message rows
 * and the accent dot. Straight from the reference's card preview, so a theme is
 * recognised by its shape as well as its name.
 */
export function thumb(id, mode) {
  const p = paletteFor(id, mode);
  const text = textOn(p.bg);
  const side = rgba(text, 0.05);
  const bar = rgba(text, 0.35);
  const msg = rgba(text, 0.2);
  const user = rgba(p.accent, 0.35);
  return `<span class="tk-thumb" style="background:${p.bg}">
      <span class="tk-side" style="background:${side}"></span>
      <span class="tk-main">
        <span class="tk-bar" style="background:${bar}"></span>
        <span class="tk-msg tk-a" style="background:${msg}"></span>
        <span class="tk-msg tk-u" style="background:${user}"></span>
      </span>
      <span class="tk-dot" style="background:${p.accent}"></span>
    </span>`;
}
