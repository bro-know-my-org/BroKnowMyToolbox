import { defineStore } from "pinia";
import { computed, readonly, ref, watch } from "vue";

import { loadAppConfig, saveAppConfig, type AppConfig } from "../api/config";

const defaultConfig = (): AppConfig => ({
  schemaVersion: 1,
  locale: "system",
  theme: "system",
  accentColor: "#78a9ff",
  fontScalePercent: 100,
  density: "comfortable",
  reducedMotion: false,
  favoriteToolIds: [],
  recentToolIds: [],
});

function cloneConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    favoriteToolIds: [...config.favoriteToolIds],
    recentToolIds: [...config.recentToolIds],
  };
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return String(error);
}

function accentContrast(color: string): "#000000" | "#ffffff" {
  const channels = [1, 3, 5].map((index) => {
    const value = Number.parseInt(color.slice(index, index + 2), 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  const luminance =
    0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  return luminance > 0.179 ? "#000000" : "#ffffff";
}

export const useSettingsStore = defineStore("settings", () => {
  const committedConfig = ref<AppConfig>(defaultConfig());
  const previewConfig = ref<AppConfig | null>(null);
  const effectiveConfig = computed(
    () => previewConfig.value ?? committedConfig.value,
  );
  const initialized = ref(false);
  const loadError = ref<string | null>(null);
  const saveError = ref<string | null>(null);
  let initializePromise: Promise<void> | null = null;
  let operationQueue: Promise<void> = Promise.resolve();

  function applyRuntime() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--bkmt-accent", effectiveConfig.value.accentColor);
    root.style.setProperty(
      "--bkmt-accent-contrast",
      accentContrast(effectiveConfig.value.accentColor),
    );
    root.style.setProperty(
      "--bkmt-font-scale",
      String(effectiveConfig.value.fontScalePercent / 100),
    );
    root.dataset.theme = effectiveConfig.value.theme;
    root.dataset.density = effectiveConfig.value.density;
    root.classList.toggle(
      "reduced-motion",
      effectiveConfig.value.reducedMotion,
    );
  }

  watch(effectiveConfig, applyRuntime, {
    deep: true,
    immediate: true,
    flush: "sync",
  });

  async function initialize() {
    if (initialized.value && !loadError.value) return;
    if (!initializePromise) {
      initializePromise = (async () => {
        try {
          committedConfig.value = await loadAppConfig();
          loadError.value = null;
        } catch (error) {
          loadError.value = errorMessage(error);
        } finally {
          initialized.value = true;
          initializePromise = null;
        }
      })();
    }
    await initializePromise;
  }

  async function requireLoadedConfig() {
    await initialize();
    if (loadError.value) {
      throw new Error(
        `settings are read-only because loading failed: ${loadError.value}`,
      );
    }
  }

  async function persistLoadedConfig() {
    try {
      await saveAppConfig(cloneConfig(committedConfig.value));
      saveError.value = null;
    } catch (error) {
      saveError.value = errorMessage(error);
      throw error;
    }
  }

  async function mutateAndPersist(mutate: () => void) {
    const operation = operationQueue.then(async () => {
      await requireLoadedConfig();
      const previous = cloneConfig(committedConfig.value);
      mutate();
      try {
        await persistLoadedConfig();
      } catch (error) {
        committedConfig.value = previous;
        throw error;
      }
    });
    operationQueue = operation.catch(() => undefined);
    return operation;
  }

  async function update(patch: Partial<AppConfig>) {
    await mutateAndPersist(() => {
      committedConfig.value = { ...committedConfig.value, ...patch };
    });
    if (previewConfig.value) {
      previewConfig.value = { ...previewConfig.value, ...patch };
    }
  }

  function preview(config: AppConfig) {
    previewConfig.value = cloneConfig(config);
  }

  function cancelPreview() {
    previewConfig.value = null;
  }

  async function toggleFavorite(toolId: string) {
    await mutateAndPersist(() => {
      const favorites = new Set(committedConfig.value.favoriteToolIds);
      if (favorites.has(toolId)) favorites.delete(toolId);
      else favorites.add(toolId);
      committedConfig.value.favoriteToolIds = [...favorites];
    });
  }

  async function markRecent(toolId: string) {
    await mutateAndPersist(() => {
      committedConfig.value.recentToolIds = [
        toolId,
        ...committedConfig.value.recentToolIds.filter((id) => id !== toolId),
      ].slice(0, 10);
    });
  }

  return {
    config: readonly(effectiveConfig),
    initialized,
    loadError,
    saveError,
    initialize,
    preview,
    cancelPreview,
    update,
    toggleFavorite,
    markRecent,
  };
});
