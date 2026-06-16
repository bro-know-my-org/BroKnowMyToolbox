import { defineStore } from "pinia";

const DEBUG_STORAGE_KEY = "bkm.debug.enabled";

function readInitialDebugEnabled(): boolean {
  try {
    return localStorage.getItem(DEBUG_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

function saveDebugEnabled(enabled: boolean) {
  try {
    localStorage.setItem(DEBUG_STORAGE_KEY, String(enabled));
  } catch {
    // Ignore storage errors; debug mode still works for the current session.
  }
}

export const useDebugStore = defineStore("debug", {
  state: () => ({
    enabled: readInitialDebugEnabled(),
    altPressed: false,
  }),

  actions: {
    setEnabled(enabled: boolean) {
      this.enabled = enabled;
      saveDebugEnabled(enabled);
    },

    toggle() {
      this.setEnabled(!this.enabled);
    },

    setAltPressed(pressed: boolean) {
      this.altPressed = pressed;
    },
  },
});
