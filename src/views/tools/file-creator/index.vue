<template>
  <ToolPage :title="t('tools.fileCreator.title')" :description="t('tools.fileCreator.description')" eyebrow="Built-in Tool">
    <ToolPanel :title="t('tools.fileCreator.panelTitle')" :description="t('tools.fileCreator.panelDescription')">
      <NForm :model="form" label-placement="top">
        <div class="form-grid">
          <NFormItem :label="t('tools.fileCreator.path')">
            <NInput v-model:value="form.path" :placeholder="t('tools.fileCreator.pathPlaceholder')" />
          </NFormItem>

          <NFormItem :label="t('tools.fileCreator.filename')">
            <NInput v-model:value="form.filename" :placeholder="t('tools.fileCreator.filenamePlaceholder')" />
          </NFormItem>
        </div>

        <NFormItem :label="t('tools.fileCreator.content')">
          <NInput v-model:value="form.content" type="textarea" :rows="12" :placeholder="t('tools.fileCreator.contentPlaceholder')" />
        </NFormItem>

        <div class="action-row">
          <NButton type="primary" :loading="creating" @click="handleCreate">{{ t("tools.fileCreator.create") }}</NButton>
          <NText depth="3">{{ previewPath || t("common.waitingForInput") }}</NText>
        </div>
      </NForm>
    </ToolPanel>
  </ToolPage>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { NButton, NForm, NFormItem, NInput, NText, useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useAppI18n } from "../../../i18n/useAppI18n";
import ToolPage from "../../../components/app/ToolPage.vue";
import ToolPanel from "../../../components/app/ToolPanel.vue";

const { t } = useAppI18n();
const message = useMessage();
const creating = ref(false);

const form = ref({
  path: "",
  filename: "",
  content: "",
});

const previewPath = computed(() => joinPath(form.value.path, form.value.filename));

function joinPath(dir: string, filename: string): string {
  const cleanDir = dir.trim().replace(/[\\/]+$/, "");
  const cleanFilename = filename.trim().replace(/^[\\/]+/, "");

  if (!cleanDir) {
    return cleanFilename;
  }

  return `${cleanDir}/${cleanFilename}`;
}

async function handleCreate() {
  creating.value = true;
  try {
    await invoke("create_file", {
      path: previewPath.value,
      content: form.value.content,
    });

    message.success(t("tools.fileCreator.createSuccess"));
  } catch (error) {
    message.error(t("tools.fileCreator.createFailed", { message: String(error) }));
  } finally {
    creating.value = false;
  }
}
</script>

<style scoped lang="scss">
.form-grid {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) minmax(180px, 320px);
  gap: 12px;
}

.action-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

@media (max-width: 760px) {
  .form-grid {
    grid-template-columns: 1fr;
  }
}
</style>
