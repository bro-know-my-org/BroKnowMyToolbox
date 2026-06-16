<template>
  <ToolPage :title="t('settings.title')" :description="t('settings.description')" :eyebrow="t('settings.eyebrow')">
    <ToolPanel :title="t('settings.interfaceTitle')" :description="t('settings.interfaceDescription')">
      <NForm class="settings-form" label-placement="left" label-width="120px">
        <NFormItem :label="t('settings.language')" :show-feedback="false">
          <NSelect v-model:value="languageDraft" class="language-select" :options="languageOptions" />
        </NFormItem>
        <NFormItem :label="t('settings.titleBarHeight')" :show-feedback="false">
          <div class="setting-control">
            <NSlider v-model:value="titleBarHeightDraft" :min="40" :max="72" :step="1" />
            <NInputNumber v-model:value="titleBarHeightDraft" class="height-input" :min="40" :max="72" :step="1" />
            <NButton @click="settingsStore.resetTitleBarHeight">{{ t("common.reset") }}</NButton>
          </div>
        </NFormItem>
        <NFormItem :label="t('settings.debugMode')" :show-feedback="false">
          <NSwitch :value="debugEnabled" @update:value="debugStore.setEnabled" />
        </NFormItem>
      </NForm>
    </ToolPanel>
  </ToolPage>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { NButton, NForm, NFormItem, NInputNumber, NSelect, NSlider, NSwitch } from "naive-ui";
import { useAppI18n } from "../../i18n/useAppI18n";
import ToolPage from "../../components/app/ToolPage.vue";
import ToolPanel from "../../components/app/ToolPanel.vue";
import { appLanguages, type AppLanguage, useSettingsStore } from "../../stores/settings";
import { useDebugStore } from "../../stores/debug";
import { setI18nLanguage } from "../../i18n";

const { t } = useAppI18n();

const debugStore = useDebugStore();
const { enabled: debugEnabled } = storeToRefs(debugStore);

const settingsStore = useSettingsStore();
const { language, titleBarHeight } = storeToRefs(settingsStore);

const languageOptions = computed(() =>
  appLanguages.map((value) => ({
    label: value === "zh-CN" ? t("settings.languageOptions.zhCN") : t("settings.languageOptions.enUS"),
    value,
  })),
);

const languageDraft = computed({
  get: () => language.value,
  set: (value: AppLanguage) => {
    settingsStore.setLanguage(value);
    setI18nLanguage(value);
  },
});

const titleBarHeightDraft = computed({
  get: () => titleBarHeight.value,
  set: (value: number | null) => {
    if (typeof value === "number") {
      settingsStore.setTitleBarHeight(value);
    }
  },
});
</script>

<style scoped lang="scss">
.settings-form {
  max-width: 820px;
}

:deep(.n-form-item) {
  margin-bottom: 16px;
}

:deep(.n-form-item:last-child) {
  margin-bottom: 0;
}

.setting-control {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) 120px auto;
  gap: 12px;
  align-items: center;
  width: 100%;
}

.height-input {
  width: 120px;
}

.language-select {
  max-width: 220px;
}

@media (max-width: 720px) {
  .setting-control {
    grid-template-columns: 1fr;
  }

  .height-input {
    width: 100%;
  }
}
</style>
