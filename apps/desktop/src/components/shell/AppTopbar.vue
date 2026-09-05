<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";

import { toolCatalog } from "../../tools/registry";

defineEmits<{ openCommand: []; openSettings: [] }>();
const { t } = useI18n();
const route = useRoute();
const shortcut = /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘ K" : "Ctrl K";
const contextTitle = computed(() => {
  const tool = toolCatalog.all().find((item) => item.routeName === route.name);
  return tool ? t(tool.titleKey) : t("nav.workbench");
});
</script>

<template>
  <header class="app-topbar">
    <strong class="page-context">{{ contextTitle }}</strong>
    <button
      type="button"
      class="command-trigger"
      :aria-label="t('command.title')"
      @click="$emit('openCommand')"
    >
      <span>{{ t("command.placeholder") }}</span>
      <kbd>{{ shortcut }}</kbd>
    </button>
    <button
      type="button"
      class="settings-trigger"
      :aria-label="t('settings.open')"
      @click="$emit('openSettings')"
    >
      {{ t("settings.open") }}
    </button>
  </header>
</template>

<style scoped>
.app-topbar {
  position: sticky;
  top: 0;
  z-index: 15;
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 48px;
  padding: 6px 20px;
  background: var(--bkmt-surface);
  border-bottom: 1px solid var(--bkmt-border);
}
.page-context {
  font-size: 0.85rem;
  white-space: nowrap;
}
.command-trigger,
.settings-trigger {
  min-height: 32px;
  padding: 4px 10px;
  color: var(--bkmt-text-muted);
  background: var(--bkmt-surface);
  border: 1px solid var(--bkmt-border);
  border-radius: 4px;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;
}
.command-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: min(320px, 45vw);
  margin-left: auto;
  text-align: left;
}
.command-trigger span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
kbd {
  font: inherit;
  font-size: 0.7rem;
  white-space: nowrap;
}
.command-trigger:hover,
.settings-trigger:hover {
  color: var(--bkmt-text);
  border-color: var(--bkmt-border-strong);
}
@media (max-width: 640px) {
  .app-topbar {
    padding-inline: 12px;
    gap: 8px;
  }
  .page-context {
    display: none;
  }
  .command-trigger {
    width: auto;
    flex: 1;
    margin-left: 0;
  }
}
</style>
