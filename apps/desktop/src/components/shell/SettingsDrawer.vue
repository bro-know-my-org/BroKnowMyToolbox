<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  NButton,
  NColorPicker,
  NDrawer,
  NDrawerContent,
  NSelect,
  NSlider,
  NSwitch,
} from "naive-ui";
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useRuntimeStore } from "../../stores/runtime";
import { useSettingsStore } from "../../stores/settings";
import type { AppConfig } from "../../api/config";

interface UpdateCheckResult {
  available: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseUrl: string;
}

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ "update:show": [value: boolean] }>();
const { t } = useI18n();
const settings = useSettingsStore();
const runtime = useRuntimeStore();
const { config, loadError, saveError } = storeToRefs(settings);
const { diagnostics, loadError: diagnosticsError } = storeToRefs(runtime);
const updateChecking = ref(false);
const updateResult = ref<UpdateCheckResult | null>(null);
const updateError = ref("");
const draft = ref<AppConfig | null>(null);
const saving = ref(false);

function cloneConfig(): AppConfig {
  return {
    ...config.value,
    favoriteToolIds: [...config.value.favoriteToolIds],
    recentToolIds: [...config.value.recentToolIds],
  };
}

watch(
  () => props.show,
  (show) => {
    if (show) draft.value = cloneConfig();
    else if (!saving.value) {
      settings.cancelPreview();
      draft.value = null;
    }
  },
);

watch(
  draft,
  (value) => {
    if (value) settings.preview(value);
  },
  { deep: true },
);

function updateShow(show: boolean) {
  if (saving.value) return;
  if (!show) settings.cancelPreview();
  emit("update:show", show);
}

async function saveSettings() {
  if (saving.value || !draft.value) return;
  saving.value = true;
  try {
    if (!draft.value) return;
    await settings.update({
      locale: draft.value.locale,
      theme: draft.value.theme,
      accentColor: draft.value.accentColor,
      fontScalePercent: draft.value.fontScalePercent,
      density: draft.value.density,
      reducedMotion: draft.value.reducedMotion,
    });
    settings.cancelPreview();
    emit("update:show", false);
  } catch {
    // Preserve the draft so the user can retry after a failed save.
    if (draft.value) settings.preview(draft.value);
  } finally {
    saving.value = false;
  }
}

async function checkForUpdates() {
  updateChecking.value = true;
  updateError.value = "";
  try {
    updateResult.value = await invoke<UpdateCheckResult>(
      "check_for_update_command",
    );
  } catch (error) {
    updateResult.value = null;
    updateError.value = error instanceof Error ? error.message : String(error);
  } finally {
    updateChecking.value = false;
  }
}

async function openUpdateDownload() {
  if (!updateResult.value?.available) return;
  updateError.value = "";
  try {
    await openUrl(updateResult.value.releaseUrl);
  } catch (error) {
    updateError.value =
      error instanceof Error
        ? error.message
        : typeof error === "object" &&
            error !== null &&
            "message" in error &&
            typeof error.message === "string"
          ? error.message
          : String(error);
  }
}
</script>

<template>
  <NDrawer :show="show" :width="'min(380px, 100vw)'" @update:show="updateShow">
    <NDrawerContent :title="t('settings.title')">
      <div v-if="draft" class="settings-form">
        <section class="settings-group">
          <span class="group-label">{{ t("settings.experience_group") }}</span>
          <label>
            <span>{{ t("settings.language") }}</span>
            <NSelect
              v-model:value="draft.locale"
              :disabled="saving"
              :aria-label="t('settings.language')"
              :options="[
                { label: t('settings.system'), value: 'system' },
                { label: '简体中文', value: 'zh-CN' },
                { label: 'English', value: 'en-US' },
              ]"
            />
          </label>
          <label>
            <span>{{ t("settings.theme") }}</span>
            <NSelect
              v-model:value="draft.theme"
              :disabled="saving"
              :aria-label="t('settings.theme')"
              :options="[
                { label: t('settings.system'), value: 'system' },
                { label: t('settings.light'), value: 'light' },
                { label: t('settings.dark'), value: 'dark' },
              ]"
            />
          </label>
          <label>
            <span>{{ t("settings.accent") }}</span>
            <NColorPicker
              v-model:value="draft.accentColor"
              :disabled="saving"
              :aria-label="t('settings.accent')"
              :show-alpha="false"
              :modes="['hex']"
            />
          </label>
          <label>
            <span>
              {{ t("settings.font_scale") }} · {{ draft.fontScalePercent }}%
            </span>
            <NSlider
              v-model:value="draft.fontScalePercent"
              :disabled="saving"
              :aria-label="t('settings.font_scale')"
              :min="85"
              :max="130"
              :step="5"
            />
          </label>
          <label>
            <span>{{ t("settings.density") }}</span>
            <NSelect
              v-model:value="draft.density"
              :disabled="saving"
              :aria-label="t('settings.density')"
              :options="[
                { label: t('settings.comfortable'), value: 'comfortable' },
                { label: t('settings.compact'), value: 'compact' },
              ]"
            />
          </label>
          <label class="switch-row">
            <span>{{ t("settings.reduced_motion") }}</span>
            <NSwitch
              v-model:value="draft.reducedMotion"
              :disabled="saving"
              :aria-label="t('settings.reduced_motion')"
            />
          </label>
        </section>

        <section class="settings-group update-section">
          <span class="group-label">{{ t("update.title") }}</span>
          <NButton :loading="updateChecking" @click="checkForUpdates">
            {{ t("update.check") }}
          </NButton>
          <span v-if="updateResult?.available">
            {{ t("update.available", { version: updateResult.latestVersion }) }}
          </span>
          <span v-else-if="updateResult">{{ t("update.current") }}</span>
          <span v-if="updateError" class="settings-error">{{
            updateError
          }}</span>
          <NButton
            v-if="updateResult?.available"
            type="primary"
            @click="openUpdateDownload"
          >
            {{ t("update.download") }}
          </NButton>
        </section>

        <section v-if="diagnostics" class="settings-group data-root-section">
          <span class="group-label">{{ t("settings.data_root") }}</span>
          <code>{{ diagnostics.dataRoot }}</code>
          <small>
            {{ t(`settings.data_root_source.${diagnostics.dataRootSource}`) }}
          </small>
        </section>

        <span v-if="loadError" class="settings-error">
          {{ t("settings.load_failed") }}: {{ loadError }}
        </span>
        <span v-if="diagnosticsError" role="alert" class="settings-error">
          {{ t("settings.data_root_failed") }}: {{ diagnosticsError }}
        </span>
        <span v-if="saveError" class="settings-error">
          {{ t("settings.save_failed") }}: {{ saveError }}
        </span>
        <NButton
          type="primary"
          :loading="saving"
          :disabled="saving"
          @click="saveSettings"
        >
          {{ t("settings.save") }}
        </NButton>
      </div>
    </NDrawerContent>
  </NDrawer>
</template>

<style scoped>
.settings-form,
.settings-group,
.settings-group label {
  display: grid;
}

.settings-form {
  gap: 12px;
}

.settings-group {
  gap: 12px;
  padding: 12px;
  background: var(--bkmt-control);
  border: 1px solid var(--bkmt-border);
  border-radius: 4px;
}

.settings-group label {
  gap: 7px;
}

.group-label {
  color: var(--bkmt-text-muted);
  letter-spacing: 0.09em;
  text-transform: uppercase;
  font-size: 0.65rem;
  font-weight: 700;
}

.switch-row {
  grid-template-columns: 1fr auto;
  align-items: center;
}

.data-root-section {
  gap: 5px;
  overflow-wrap: anywhere;
}

.data-root-section code {
  color: var(--bkmt-text);
  white-space: normal;
}

.data-root-section small {
  color: var(--bkmt-text-muted);
}

.settings-error {
  color: var(--n-color-error);
}
</style>
