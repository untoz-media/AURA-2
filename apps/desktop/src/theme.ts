export type AuraTheme = "default" | "midnight" | "oled" | "aurora" | "light";

const THEME_STORAGE_KEY = "aura.desktop.theme";

export const auraThemes: Array<{
  id: AuraTheme;
  name: string;
  description: string;
}> = [
  {
    id: "default",
    name: "AURA",
    description: "The core blue and violet AURA identity.",
  },
  {
    id: "midnight",
    name: "Midnight",
    description: "Deeper navy surfaces with restrained blue light.",
  },
  {
    id: "oled",
    name: "OLED",
    description: "True-black surfaces for dark rooms and OLED displays.",
  },
  {
    id: "aurora",
    name: "Aurora",
    description: "A brighter cyan, blue and violet spectrum.",
  },
  {
    id: "light",
    name: "Light",
    description: "A clean bright theme while keeping the AURA spectrum.",
  },
];

function isAuraTheme(value: string | null): value is AuraTheme {
  return auraThemes.some((theme) => theme.id === value);
}

export function readAuraTheme(): AuraTheme {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    return isAuraTheme(stored) ? stored : "default";
  } catch {
    return "default";
  }
}

export function applyAuraTheme(theme: AuraTheme) {
  document.documentElement.dataset.auraTheme = theme;

  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // Theme persistence is best effort. The current session still updates.
  }
}

export function initializeAuraTheme() {
  const theme = readAuraTheme();
  applyAuraTheme(theme);
  return theme;
}
