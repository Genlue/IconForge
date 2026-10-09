import { create } from "zustand";

export type TitleBarStyle = "macos" | "windows";

const SETTINGS_KEY = "iconforge.settings.v1";

interface PersistedSettings {
  titleBarStyle: TitleBarStyle;
}

export interface SettingsState {
  titleBarStyle: TitleBarStyle;
  settingsOpen: boolean;
  setTitleBarStyle(style: TitleBarStyle): void;
  setSettingsOpen(open: boolean): void;
}

function loadSettings(): PersistedSettings {
  const fallback: PersistedSettings = { titleBarStyle: "macos" };
  try {
    if (typeof window === "undefined") return fallback;
    const raw = window.localStorage.getItem(SETTINGS_KEY);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw) as Partial<PersistedSettings>;
    return {
      titleBarStyle:
        parsed.titleBarStyle === "windows" ? "windows" : "macos",
    };
  } catch {
    return fallback;
  }
}

function persistSettings(settings: PersistedSettings): void {
  try {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  } catch {
    // storage unavailable: settings stay in memory for the session
  }
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  ...loadSettings(),
  settingsOpen: false,

  setTitleBarStyle: (style) => {
    set({ titleBarStyle: style });
    persistSettings({ titleBarStyle: get().titleBarStyle });
  },

  setSettingsOpen: (open) => set({ settingsOpen: open }),
}));
