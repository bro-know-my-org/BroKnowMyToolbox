<script setup lang="ts">
import { NButton, NInput, NModal } from "naive-ui";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { useSettingsStore } from "../../stores/settings";
import { searchRegisteredTools } from "../../tools/registry";
import ToolGlyph from "./ToolGlyph.vue";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ "update:show": [value: boolean] }>();
const { t } = useI18n();
const router = useRouter();
const settings = useSettingsStore();
const query = ref("");
const activeIndex = ref(0);
const results = computed(() => searchRegisteredTools(query.value, t));
const activeOptionId = computed(() =>
  results.value[activeIndex.value]
    ? `command-option-${results.value[activeIndex.value].id}`
    : undefined,
);

watch(results, () => {
  activeIndex.value = 0;
});
watch(
  () => props.show,
  (show) => {
    if (!show) query.value = "";
    activeIndex.value = 0;
  },
);

function handleKeydown(event: KeyboardEvent) {
  if (event.isComposing || results.value.length === 0) return;
  if (event.key === "ArrowDown") {
    event.preventDefault();
    activeIndex.value = (activeIndex.value + 1) % results.value.length;
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    activeIndex.value =
      (activeIndex.value - 1 + results.value.length) % results.value.length;
  } else if (event.key === "Enter") {
    event.preventDefault();
    const tool = results.value[activeIndex.value];
    if (tool) void openTool(tool.id, tool.route);
  }
}

async function openTool(id: string, route: string) {
  emit("update:show", false);
  query.value = "";
  await router.push(route);
  void settings.markRecent(id).catch(() => undefined);
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :title="t('command.title')"
    class="command-palette"
    :style="{ width: 'min(680px, calc(100vw - 32px))' }"
    @update:show="$emit('update:show', $event)"
  >
    <div class="palette-label">
      <span>{{ t("command.prompt_label") }}</span
      ><kbd>ESC</kbd>
    </div>
    <NInput
      v-model:value="query"
      autofocus
      size="large"
      :placeholder="t('command.placeholder')"
      :input-props="{
        type: 'search',
        role: 'combobox',
        'aria-controls': 'command-results',
        'aria-expanded': show,
        'aria-activedescendant': activeOptionId,
        'aria-label': t('command.search_label'),
      }"
      @keydown="handleKeydown"
    />
    <div id="command-results" class="command-results" role="listbox">
      <NButton
        v-for="(tool, index) in results"
        :id="`command-option-${tool.id}`"
        :key="tool.id"
        text
        block
        class="command-result"
        role="option"
        :class="{ 'command-result-active': index === activeIndex }"
        :aria-selected="index === activeIndex"
        @mouseenter="activeIndex = index"
        @click="openTool(tool.id, tool.route)"
      >
        <span class="result-icon"><ToolGlyph :tool-id="tool.id" /></span>
        <span class="result-copy">
          <strong>{{ t(tool.titleKey) }}</strong>
          <small>{{ t(tool.descriptionKey) }}</small>
        </span>
      </NButton>
    </div>
    <p v-if="!results.length" role="status">{{ t("home.no_results") }}</p>
    <div class="palette-hints">
      <span><kbd>↑↓</kbd> {{ t("command.navigate_hint") }}</span>
      <span><kbd>↵</kbd> {{ t("command.open_hint") }}</span>
    </div>
  </NModal>
</template>

<style scoped>
.palette-label,
.palette-hints {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.palette-label {
  margin-bottom: 9px;
  color: var(--bkmt-text-muted);
  letter-spacing: 0.12em;
  font-size: 0.6rem;
}

kbd {
  padding: 2px 5px;
  color: var(--bkmt-text-muted);
  background: var(--bkmt-control);
  border: 1px solid var(--bkmt-border);
  border-radius: 4px;
  font-family: inherit;
  font-size: 0.58rem;
}

.command-results {
  display: grid;
  gap: 4px;
  margin-top: 15px;
}

.command-result {
  min-height: 52px;
  padding: 8px 10px;
  border: 1px solid transparent;
  border-radius: 4px;
}

.command-result-active {
  background: var(--bkmt-control);
  border-color: var(--bkmt-border-strong);
  outline: 2px solid color-mix(in srgb, var(--bkmt-accent) 36%, transparent);
  outline-offset: 1px;
}

.result-icon {
  display: grid;
  place-items: center;
  width: 24px;
}

.result-copy {
  display: grid;
  flex: 1;
  gap: 2px;
  min-width: 0;
  margin-left: 12px;
  text-align: left;
}

.result-copy strong {
  color: var(--bkmt-text);
  font-size: 0.8rem;
}

.result-copy small {
  overflow: hidden;
  color: var(--bkmt-text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.66rem;
}

.palette-hints {
  justify-content: flex-start;
  gap: 16px;
  margin-top: 14px;
  color: var(--bkmt-text-muted);
  font-size: 0.61rem;
}

.palette-hints span {
  display: flex;
  gap: 5px;
  align-items: center;
}
</style>
