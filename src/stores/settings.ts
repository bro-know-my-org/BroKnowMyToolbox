import { defineStore } from "pinia";

const TITLE_BAR_HEIGHT_KEY = "bkm.settings.titleBarHeight";
const LANGUAGE_KEY = "bkm.settings.language";
const DEFAULT_TITLE_BAR_HEIGHT = 48;
const MIN_TITLE_BAR_HEIGHT = 40;
const MAX_TITLE_BAR_HEIGHT = 72;

export type AppLanguage = "zh-CN" | "en-US";
export const DEFAULT_LANGUAGE: AppLanguage = "zh-CN";
export const appLanguages: AppLanguage[] = ["zh-CN", "en-US"];

function clampTitleBarHeight(value: number): number {
  return Math.min(MAX_TITLE_BAR_HEIGHT, Math.max(MIN_TITLE_BAR_HEIGHT, Math.round(value)));
}

function readTitleBarHeight(): number {
  try {
    const stored = Number(localStorage.getItem(TITLE_BAR_HEIGHT_KEY));

    if (Number.isFinite(stored)) {
      return clampTitleBarHeight(stored);
    }
  } catch {
    // Ignore storage errors and fall back to the default value.
  }

  return DEFAULT_TITLE_BAR_HEIGHT;
}

function saveTitleBarHeight(value: number) {
  try {
    localStorage.setItem(TITLE_BAR_HEIGHT_KEY, String(value));
  } catch {
    // Ignore storage errors; settings still work for the current session.
  }
}

function applyTitleBarHeight(value: number) {
  document.documentElement.style.setProperty("--bkm-title-bar-height", `${value}px`);
}

function readLanguage(): AppLanguage {
  try {
    const stored = localStorage.getItem(LANGUAGE_KEY);

    if (stored && appLanguages.includes(stored as AppLanguage)) {
      return stored as AppLanguage;
    }
  } catch {
    // Ignore storage errors and fall back to the default language.
  }

  return DEFAULT_LANGUAGE;
}

function saveLanguage(value: AppLanguage) {
  try {
    localStorage.setItem(LANGUAGE_KEY, value);
  } catch {
    // Ignore storage errors; settings still work for the current session.
  }
}

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    titleBarHeight: readTitleBarHeight(),
    language: readLanguage(),
  }),

  actions: {
    applySettings() {
      applyTitleBarHeight(this.titleBarHeight);
    },

    setTitleBarHeight(value: number) {
      const nextValue = clampTitleBarHeight(value);

      this.titleBarHeight = nextValue;
      saveTitleBarHeight(nextValue);
      applyTitleBarHeight(nextValue);
    },

    resetTitleBarHeight() {
      this.setTitleBarHeight(DEFAULT_TITLE_BAR_HEIGHT);
    },

    setLanguage(value: AppLanguage) {
      if (!appLanguages.includes(value)) {
        return;
      }

      this.language = value;
      saveLanguage(value);
    },
  },
});
