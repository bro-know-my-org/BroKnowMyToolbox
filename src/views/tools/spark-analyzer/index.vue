<template>
  <div class="spark-tool">
    <div class="spark-tool__status">
      <span>{{ t(`tools.sparkAnalyzer.status.${statusKey}`) }}</span>
    </div>
    <SparkAnalyzerView
      :adapter="sparkAnalyzerAdapter"
      :language="sparkLanguage"
      :debug="debugEnabled"
      embedded
      @status-change="statusKey = $event.key"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { SparkAnalyzerView } from "@bro-know-my/spark-analyzer";
import "@bro-know-my/spark-analyzer/style.css";
import { useAppI18n } from "../../../i18n/useAppI18n";
import { useSettingsStore } from "../../../stores/settings";
import { useDebugStore } from "../../../stores/debug";
import { sparkAnalyzerAdapter } from "./adapter";

type SparkStatusKey =
  | "waiting"
  | "parsing"
  | "loaded"
  | "parseFailed"
  | "fetching"
  | "fetchFailed"
  | "textLoaded"
  | "analyzing"
  | "done"
  | "failed";

const { t } = useAppI18n();
const settingsStore = useSettingsStore();
const { language } = storeToRefs(settingsStore);
const debugStore = useDebugStore();
const { enabled: debugEnabled } = storeToRefs(debugStore);

const statusKey = ref<SparkStatusKey>("waiting");
const sparkLanguage = computed(() => (language.value === "en-US" ? "en" : "zh"));
</script>

<style scoped>
.spark-tool {
  height: calc(100vh - var(--bkm-title-bar-height) - 36px);
  min-height: min(720px, calc(100vh - var(--bkm-title-bar-height) - 36px));
  overflow: auto;
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
}

.spark-tool__status {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  min-height: 36px;
  padding: 0 12px;
  border-bottom: 1px solid var(--bkm-border);
  background: rgba(18, 26, 33, 0.96);
  backdrop-filter: blur(12px);
}

.spark-tool__status span {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  border-radius: 999px;
  padding: 0 10px;
  color: #b7fff6;
  background: rgba(79, 182, 189, 0.18);
  font-size: 12px;
  font-weight: 650;
}
</style>
