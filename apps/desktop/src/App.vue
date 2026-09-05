<script setup lang="ts">
import { NConfigProvider, darkTheme, useOsTheme } from "naive-ui";
import { storeToRefs } from "pinia";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { RouterView } from "vue-router";

import { loadLocaleOverride } from "./api/locale";
import AppSidebar from "./components/shell/AppSidebar.vue";
import AppTopbar from "./components/shell/AppTopbar.vue";
import CommandPalette from "./components/shell/CommandPalette.vue";
import SettingsDrawer from "./components/shell/SettingsDrawer.vue";
import { builtInLocaleMessages } from "./i18n";
import { useRuntimeStore } from "./stores/runtime";
import { useSettingsStore } from "./stores/settings";

const { locale, setLocaleMessage } = useI18n();
const settings = useSettingsStore();
const runtime = useRuntimeStore();
const { config } = storeToRefs(settings);
const settingsOpen = ref(false);
const commandOpen = ref(false);
const osTheme = useOsTheme();
const systemLocale = ref(resolveSystemLocale());

function resolveSystemLocale() {
  return navigator.language.toLowerCase().startsWith("zh") ? "zh-CN" : "en-US";
}

function handleLanguageChange() {
  systemLocale.value = resolveSystemLocale();
}

const resolvedLocale = computed(() => {
  if (config.value.locale !== "system") return config.value.locale;
  return systemLocale.value;
});
const activeTheme = computed(() =>
  config.value.theme === "dark" ||
  (config.value.theme === "system" && osTheme.value === "dark")
    ? darkTheme
    : null,
);
const themeOverrides = computed(() => ({
  common: {
    primaryColor: config.value.accentColor,
    primaryColorHover: config.value.accentColor,
    primaryColorPressed: config.value.accentColor,
  },
}));

watch(
  resolvedLocale,
  async (value, _previous, onCleanup) => {
    let cancelled = false;
    onCleanup(() => {
      cancelled = true;
    });
    locale.value = value;
    document.documentElement.lang = value;
    setLocaleMessage(value, builtInLocaleMessages(value));
    try {
      const overrides = await loadLocaleOverride(value);
      if (!cancelled) {
        setLocaleMessage(value, {
          ...builtInLocaleMessages(value),
          ...overrides,
        });
      }
    } catch {
      // Built-in messages remain available without an override file.
    }
  },
  { immediate: true },
);

function handleShortcut(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    commandOpen.value = true;
  }
}

onMounted(() => {
  void settings.initialize();
  void runtime.initialize();
  window.addEventListener("keydown", handleShortcut);
  window.addEventListener("languagechange", handleLanguageChange);
});
onUnmounted(() => {
  window.removeEventListener("keydown", handleShortcut);
  window.removeEventListener("languagechange", handleLanguageChange);
});
</script>

<template>
  <NConfigProvider :theme="activeTheme" :theme-overrides="themeOverrides">
    <div class="app-workbench">
      <AppSidebar />
      <section class="app-stage">
        <AppTopbar
          @open-command="commandOpen = true"
          @open-settings="settingsOpen = true"
        />
        <main class="app-content">
          <RouterView v-slot="{ Component }">
            <KeepAlive include="FileGeneratorView">
              <component :is="Component" />
            </KeepAlive>
          </RouterView>
        </main>
      </section>
    </div>

    <SettingsDrawer
      :show="settingsOpen && settings.initialized"
      @update:show="settingsOpen = $event"
    />
    <CommandPalette v-model:show="commandOpen" />
  </NConfigProvider>
</template>

<style scoped>
.app-workbench {
  display: grid;
  grid-template-columns: 184px minmax(0, 1fr);
  min-height: 100vh;
  color: var(--bkmt-text);
  background: var(--bkmt-background);
}

.app-stage {
  min-width: 0;
}

.app-content {
  min-width: 0;
  padding: 20px 24px 32px;
}

@media (max-width: 960px) {
  .app-workbench {
    grid-template-columns: 168px minmax(0, 1fr);
  }

  .app-content {
    padding-inline: 16px;
  }
}
@media (max-width: 640px) {
  .app-workbench {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto 1fr;
  }
  .app-content {
    padding: 16px 12px 24px;
  }
}
</style>
