import { defineStore } from "pinia";
import { ref } from "vue";

import {
  loadRuntimeDiagnostics,
  type RuntimeDiagnostics,
} from "../api/runtime";

export const useRuntimeStore = defineStore("runtime", () => {
  const diagnostics = ref<RuntimeDiagnostics | null>(null);
  const loadError = ref<string | null>(null);
  const initialized = ref(false);
  let initializePromise: Promise<void> | null = null;

  async function initialize() {
    if (diagnostics.value) return;
    if (!initializePromise) {
      initializePromise = (async () => {
        try {
          diagnostics.value = await loadRuntimeDiagnostics();
          loadError.value = null;
        } catch (error) {
          loadError.value =
            typeof error === "object" &&
            error !== null &&
            "message" in error &&
            typeof error.message === "string"
              ? error.message
              : String(error);
        } finally {
          initialized.value = true;
          initializePromise = null;
        }
      })();
    }
    await initializePromise;
  }

  return { diagnostics, loadError, initialized, initialize };
});
