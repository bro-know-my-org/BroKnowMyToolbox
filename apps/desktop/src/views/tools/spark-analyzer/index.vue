<script setup lang="ts">
defineOptions({ name: "SparkAnalyzerToolView" });

import { invoke } from "@tauri-apps/api/core";
import { NButton, NCard, NModal, NSpace, NText, useOsTheme } from "naive-ui";
import { storeToRefs } from "pinia";
import { computed, onBeforeUnmount, ref } from "vue";
import { useI18n } from "vue-i18n";
import { SparkAnalyzerView } from "@bro-know-my/spark-analyzer";
import { createTauriSparkAnalyzerAdapter } from "@bro-know-my/spark-analyzer/tauri";
import "@bro-know-my/spark-analyzer/style.css";
import {
  createConsentAwareSparkAdapter,
  type SparkConsentCapability,
  type SparkConsentCode,
} from "../../../api/spark-consent";
import { createSparkPreferencesStore } from "../../../api/spark-preferences";
import { useSettingsStore } from "../../../stores/settings";

type PendingConsent = {
  capability: SparkConsentCapability;
  code: SparkConsentCode;
  resolvers: Array<(allowed: boolean) => void>;
};

const { locale, t } = useI18n();
const settings = useSettingsStore();
const { config } = storeToRefs(settings);
const osTheme = useOsTheme();
const consentQueue: PendingConsent[] = [];
const pendingConsent = ref<PendingConsent | null>(null);
const decisionError = ref("");
const decisionBusy = ref(false);
let disposed = false;

function showNextConsent() {
  if (!pendingConsent.value)
    pendingConsent.value = consentQueue.shift() ?? null;
}

function requestConsent(
  capability: SparkConsentCapability,
  code: SparkConsentCode,
) {
  return new Promise<boolean>((resolve) => {
    if (disposed) {
      resolve(false);
      return;
    }
    const existing = [pendingConsent.value, ...consentQueue].find(
      (pending) => pending?.capability === capability,
    );
    if (existing) existing.resolvers.push(resolve);
    else consentQueue.push({ capability, code, resolvers: [resolve] });
    showNextConsent();
  });
}

async function decideConsent(allowed: boolean) {
  const pending = pendingConsent.value;
  if (!pending || decisionBusy.value) return;
  decisionBusy.value = true;
  decisionError.value = "";
  try {
    await invoke("set_spark_consent_command", {
      capability: pending.capability,
      allowed,
    });
    if (pendingConsent.value !== pending) return;
    pendingConsent.value = null;
    for (const resolve of pending.resolvers) resolve(allowed);
    showNextConsent();
  } catch (error) {
    decisionError.value =
      error instanceof Error ? error.message : String(error);
  } finally {
    decisionBusy.value = false;
  }
}

onBeforeUnmount(() => {
  disposed = true;
  const pending = pendingConsent.value;
  pendingConsent.value = null;
  if (pending) for (const resolve of pending.resolvers) resolve(false);
  for (const pending of consentQueue.splice(0)) {
    for (const resolve of pending.resolvers) resolve(false);
  }
});

const adapter = createConsentAwareSparkAdapter(
  createTauriSparkAnalyzerAdapter(),
  requestConsent,
);
const preferencesStore = createSparkPreferencesStore();
const language = computed(() =>
  locale.value.toLowerCase().startsWith("zh") ? "zh" : "en",
);
const theme = computed<"dark" | "light">(() => {
  if (config.value.theme === "dark") return "dark";
  if (config.value.theme === "light") return "light";
  return osTheme.value === "dark" ? "dark" : "light";
});
const consentMessage = computed(() =>
  pendingConsent.value
    ? t(`spark.consent.capability.${pendingConsent.value.capability}`)
    : "",
);
</script>

<template>
  <SparkAnalyzerView
    :adapter="adapter"
    :language="language"
    :theme="theme"
    :preferences-store="preferencesStore"
    embedded
  />
  <NModal :show="pendingConsent !== null" :mask-closable="false">
    <NCard class="consent-card" :title="t('spark.consent.title')" role="dialog">
      <NSpace vertical>
        <NText>{{ consentMessage }}</NText>
        <NText v-if="decisionError" type="error">{{ decisionError }}</NText>
        <NSpace justify="end">
          <NButton :disabled="decisionBusy" @click="decideConsent(false)">{{
            t("spark.consent.deny")
          }}</NButton>
          <NButton
            type="primary"
            :loading="decisionBusy"
            @click="decideConsent(true)"
          >
            {{ t("spark.consent.allow") }}
          </NButton>
        </NSpace>
      </NSpace>
    </NCard>
  </NModal>
</template>

<style scoped>
.consent-card {
  width: min(32rem, calc(100vw - 2rem));
}
</style>
