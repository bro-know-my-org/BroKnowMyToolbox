<template>
  <aside class="sidebar">
    <n-menu
      v-model:value="activeKey"
      :options="menuOptions"
      :collapsed="collapsed"
      :collapsed-width="64"
      :collapsed-icon-size="18"
      :width="232"
      @update:value="handleMenuSelect"
    />
  </aside>
</template>

<script setup lang="ts">
import { computed, h, ref } from "vue";
import { NMenu, NIcon } from "naive-ui";
import type { Component } from "vue";
import type { MenuOption } from "naive-ui";
import { useRoute, useRouter } from "vue-router";
import { useAppI18n } from "../../../i18n/useAppI18n";
import { Home24Regular, Settings24Regular, Toolbox24Regular } from "@vicons/fluent";
import { createToolMenuItems } from "../../../tools/registry";

const { t } = useAppI18n();
const router = useRouter();
const route = useRoute();
const collapsed = ref(false);

const activeKey = computed(() => route.path);

function renderIcon(component: Component) {
  return () => h(NIcon, { component });
}

const menuOptions = computed<MenuOption[]>(() => [
  {
    label: t("nav.home"),
    key: "/index",
    icon: renderIcon(Home24Regular),
  },
  {
    label: t("nav.tools"),
    key: "tools-group",
    icon: renderIcon(Toolbox24Regular),
    children: createToolMenuItems(t),
  },
  {
    label: t("nav.settings"),
    key: "/settings",
    icon: renderIcon(Settings24Regular),
  },
]);

function handleMenuSelect(key: string) {
  if (key.startsWith("/")) {
    router.push(key);
  }
}
</script>

<style scoped lang="scss">
.sidebar {
  width: 232px;
  flex: 0 0 232px;
  border-right: 1px solid var(--bkm-border);
  background: rgba(18, 26, 33, 0.78);
  overflow: hidden;
}

:deep(.n-menu) {
  height: 100%;
  padding: 10px 8px;
  background: transparent;
}

:deep(.n-menu-item-content) {
  border-radius: 7px;
}

:deep(.n-menu-item-content::before) {
  border-radius: 7px;
}
</style>
