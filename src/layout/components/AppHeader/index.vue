<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <n-avatar class="brand__icon" src="/src-tauri/icons/128x128.png" :size="28" />
      <div class="brand__text" data-tauri-drag-region>
        <div class="brand__title">{{ t("app.title") }}</div>
        <div class="brand__subtitle">{{ t("app.subtitle") }}{{ debugEnabled ? " / debug" : "" }}</div>
      </div>
    </div>

    <div class="header-actions">
      <n-button quaternary size="small" @click="openGithub">
        <template #icon>
          <n-icon :component="LogoGithub" />
        </template>
      </n-button>
      <div v-if="isTauri" class="window-controls">
        <button type="button" title="最小化" @click="handleMinimize">
          <n-icon :component="LineHorizontal120Regular" />
        </button>
        <button type="button" title="最大化" @click="handleMaximize">
          <n-icon :component="Maximize20Regular" />
        </button>
        <button type="button" class="window-controls__close" title="关闭" @click="handleClose">
          <n-icon :component="Dismiss20Regular" />
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { NAvatar, NButton, NIcon } from "naive-ui";
import { LogoGithub } from "@vicons/ionicons5";
import { Dismiss20Regular, LineHorizontal120Regular, Maximize20Regular } from "@vicons/fluent";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { useAppI18n } from "../../../i18n/useAppI18n";
import { useDebugStore } from "../../../stores/debug";

const { t } = useAppI18n();
const debugStore = useDebugStore();
const { enabled: debugEnabled } = storeToRefs(debugStore);

const isTauri = "__TAURI_INTERNALS__" in window;
const appWindow = isTauri ? getCurrentWindow() : null;

function openGithub() {
  window.open("https://github.com/bro-know-my-org/bro-know-my-toolbox", "_blank");
}

async function handleMinimize() {
  await appWindow?.minimize();
}

async function handleMaximize() {
  await appWindow?.toggleMaximize();
}

async function handleClose() {
  await appWindow?.close();
}
</script>

<style scoped lang="scss">
.app-header {
  height: var(--bkm-title-bar-height);
  background: rgba(18, 26, 33, 0.94);
  border-bottom: 1px solid var(--bkm-border);
  user-select: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 0 12px;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  -webkit-app-region: drag;
  backdrop-filter: blur(18px);
}

.brand {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  -webkit-app-region: drag;
}

.brand__icon {
  flex: 0 0 auto;
  background: var(--bkm-panel-strong);
}

.brand__text {
  min-width: 0;
}

.brand__title {
  color: var(--bkm-text);
  font-size: 14px;
  line-height: 1.2;
  font-weight: 700;
}

.brand__subtitle {
  color: var(--bkm-text-muted);
  font-size: 11px;
  line-height: 1.3;
}

.header-actions {
  height: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.window-controls {
  height: 100%;
  display: flex;
}

.window-controls button {
  width: 46px;
  height: 100%;
  border: 0;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--bkm-text-muted);
  background: transparent;
  cursor: pointer;
}

.window-controls button:hover {
  color: var(--bkm-text);
  background: var(--bkm-panel-strong);
}

.window-controls__close:hover {
  color: #fff;
  background: var(--bkm-danger) !important;
}
</style>
