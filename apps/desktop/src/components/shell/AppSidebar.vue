<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";

import { useSettingsStore } from "../../stores/settings";
import { registeredTools } from "../../tools/registry";
import ToolGlyph from "./ToolGlyph.vue";

const { t } = useI18n();
const settings = useSettingsStore();

function rememberTool(id: string) {
  void settings.markRecent(id).catch(() => undefined);
}
</script>

<template>
  <aside class="app-sidebar">
    <RouterLink class="sidebar-brand" to="/" :aria-label="t('app.title')">
      <ToolGlyph />
      <strong>BKMT</strong>
    </RouterLink>
    <nav :aria-label="t('nav.main')">
      <RouterLink class="sidebar-link" to="/">
        <ToolGlyph />
        <span>{{ t("nav.workbench") }}</span>
      </RouterLink>
      <RouterLink
        v-for="tool in registeredTools"
        :key="tool.id"
        class="sidebar-link"
        :to="tool.route"
        @click="rememberTool(tool.id)"
      >
        <ToolGlyph :tool-id="tool.id" />
        <span>{{ t(tool.titleKey) }}</span>
      </RouterLink>
    </nav>
  </aside>
</template>

<style scoped>
.app-sidebar {
  position: sticky;
  top: 0;
  align-self: start;
  height: 100dvh;
  overflow-y: auto;
  padding: 0 8px 12px;
  color: var(--bkmt-text);
  background: var(--bkmt-surface);
  border-right: 1px solid var(--bkmt-border);
}
.sidebar-brand,
.sidebar-link {
  display: flex;
  align-items: center;
  gap: 10px;
  color: inherit;
  text-decoration: none;
}
.sidebar-brand {
  height: 48px;
  padding: 0 10px;
  font-size: 0.9rem;
  border-bottom: 1px solid var(--bkmt-border);
  margin-bottom: 8px;
}
nav {
  display: grid;
  gap: 3px;
}
.sidebar-link {
  min-height: 36px;
  padding: 7px 10px;
  border-radius: 4px;
  font-size: 0.85rem;
}
.sidebar-link:hover {
  background: var(--bkmt-control);
}
.sidebar-link.router-link-exact-active {
  background: var(--bkmt-control);
  box-shadow: inset 3px 0 var(--bkmt-accent);
  font-weight: 600;
}
@media (max-width: 640px) {
  .app-sidebar {
    position: static;
    height: auto;
    padding: 0 12px 8px;
    border-right: 0;
    border-bottom: 1px solid var(--bkmt-border);
  }
  .sidebar-brand {
    height: 40px;
    margin-bottom: 4px;
  }
  nav {
    display: flex;
    flex-wrap: wrap;
  }
  .sidebar-link {
    font-size: 0.8rem;
  }
}
</style>
