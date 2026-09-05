<script setup lang="ts">
defineOptions({ name: "FileGeneratorView" });

import { computed, onMounted, reactive, ref, watch } from "vue";
import {
  NAlert,
  NButton,
  NCard,
  NCheckbox,
  NInput,
  NSelect,
  NSpace,
} from "naive-ui";
import { useI18n } from "vue-i18n";

import {
  executeFileGeneration,
  listFileTemplates,
  planFileGeneration,
  saveUserTemplate,
  setFileGenerationConsent,
  type CommandError,
  type FileGenerationPlan,
  type FileGenerationReport,
  type FileGenerationRequest,
  type FileTemplateEntry,
} from "../../../api/file-generator";

const { t } = useI18n();

const templateJson = ref(`{
  "schemaVersion": 1,
  "id": "readme",
  "title": "README",
  "variables": [
    { "name": "name", "required": true, "default": null }
  ],
  "files": [
    { "path": "README.md", "content": "# {{name}}\\n" }
  ]
}`);
const destination = ref("");
const overwrite = ref(false);
const variables = reactive<Record<string, string>>(Object.create(null));
const plan = ref<FileGenerationPlan | null>(null);
const plannedRequest = ref<FileGenerationRequest | null>(null);
const report = ref<FileGenerationReport | null>(null);
const error = ref<CommandError | null>(null);
const consentRequired = ref(false);
const pendingConsentAction = ref<"save" | "preview" | "execute" | null>(null);
const busy = ref(false);
const templates = ref<FileTemplateEntry[]>([]);
const selectedTemplateId = ref<string | null>(null);
let templateRevision = 0;
let templateRequest = 0;

interface VariableDefinition {
  name: string;
  required?: boolean;
  default?: string | null;
}

let lastVariableDefinitions: VariableDefinition[] = [];
const variableDefinitions = computed<VariableDefinition[]>(() => {
  try {
    const parsed = JSON.parse(templateJson.value) as {
      variables?: unknown;
    };
    if (!Array.isArray(parsed.variables)) {
      lastVariableDefinitions = [];
      return lastVariableDefinitions;
    }
    const definitions = parsed.variables.filter(
      (value): value is VariableDefinition =>
        typeof value === "object" &&
        value !== null &&
        "name" in value &&
        typeof value.name === "string" &&
        value.name.length > 0,
    );
    if (definitions.length !== parsed.variables.length)
      return lastVariableDefinitions;
    lastVariableDefinitions = definitions;
    return lastVariableDefinitions;
  } catch {
    return lastVariableDefinitions;
  }
});

watch(
  variableDefinitions,
  (definitions) => {
    const names = new Set(definitions.map((definition) => definition.name));
    for (const name of Object.keys(variables)) {
      if (!names.has(name)) delete variables[name];
    }
    for (const definition of definitions) {
      if (!(definition.name in variables)) {
        variables[definition.name] = definition.default ?? "";
      }
    }
  },
  { immediate: true },
);

watch(selectedTemplateId, (id) => {
  const selected = templates.value.find((template) => template.id === id);
  if (selected) templateJson.value = selected.templateJson;
});
watch(templateJson, () => {
  templateRevision += 1;
});

async function refreshTemplates(preferredId?: string) {
  const requestId = ++templateRequest;
  const startingRevision = templateRevision;
  try {
    const nextTemplates = await listFileTemplates();
    if (requestId !== templateRequest) return;
    templates.value = nextTemplates;
    if (templateRevision === startingRevision) {
      selectedTemplateId.value =
        templates.value.find((template) => template.id === preferredId)?.id ??
        templates.value[0]?.id ??
        null;
    }
  } catch (value) {
    if (requestId !== templateRequest) return;
    error.value = commandError(value);
  }
}

async function persistTemplate() {
  busy.value = true;
  error.value = null;
  try {
    const saved = await saveUserTemplate(templateJson.value);
    await refreshTemplates(saved.id);
  } catch (value) {
    const mapped = commandError(value);
    error.value = mapped;
    consentRequired.value =
      mapped.code === "consent_required" || mapped.code === "consent_denied";
    pendingConsentAction.value = consentRequired.value ? "save" : null;
  } finally {
    busy.value = false;
  }
}

onMounted(() => refreshTemplates());

function request(): FileGenerationRequest {
  return {
    templateJson: templateJson.value,
    destination: destination.value,
    variables: { ...variables },
    overwrite: overwrite.value,
  };
}

function invalidatePlan() {
  plan.value = null;
  plannedRequest.value = null;
}

watch([templateJson, destination, overwrite, variables], invalidatePlan, {
  deep: true,
});

function commandError(value: unknown): CommandError {
  if (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value
  ) {
    return value as CommandError;
  }
  return { code: "unknown", message: String(value) };
}

async function preview() {
  busy.value = true;
  error.value = null;
  report.value = null;
  consentRequired.value = false;
  const snapshot = request();
  try {
    const nextPlan = await planFileGeneration(snapshot);
    if (JSON.stringify(request()) === JSON.stringify(snapshot)) {
      plannedRequest.value = snapshot;
      plan.value = nextPlan;
    }
  } catch (value) {
    const mapped = commandError(value);
    error.value = mapped;
    consentRequired.value =
      mapped.code === "consent_required" || mapped.code === "consent_denied";
    pendingConsentAction.value = consentRequired.value ? "preview" : null;
  } finally {
    busy.value = false;
  }
}

async function execute() {
  const snapshot = plannedRequest.value;
  if (!snapshot) return;
  const reviewedFiles = plan.value?.files ?? [];
  busy.value = true;
  error.value = null;
  report.value = null;
  consentRequired.value = false;
  try {
    report.value = await executeFileGeneration(snapshot, reviewedFiles);
    invalidatePlan();
  } catch (value) {
    const mapped = commandError(value);
    error.value = mapped;
    consentRequired.value =
      mapped.code === "consent_required" || mapped.code === "consent_denied";
    pendingConsentAction.value = consentRequired.value ? "execute" : null;
  } finally {
    busy.value = false;
  }
}

async function grantAndContinue() {
  busy.value = true;
  try {
    const action = pendingConsentAction.value;
    await setFileGenerationConsent(true);
    consentRequired.value = false;
    pendingConsentAction.value = null;
    if (action === "save") await persistTemplate();
    if (action === "preview") await preview();
    if (action === "execute") await execute();
  } catch (value) {
    error.value = commandError(value);
  } finally {
    busy.value = false;
  }
}

async function denyConsent() {
  busy.value = true;
  try {
    await setFileGenerationConsent(false);
    consentRequired.value = false;
    pendingConsentAction.value = null;
    error.value = {
      code: "consent_denied",
      message: t("file_generator.consent.denied"),
    };
  } catch (value) {
    error.value = commandError(value);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <section class="tool-page">
    <header class="tool-heading">
      <h2>{{ t("tool.file_generator.name") }}</h2>
      <p>{{ t("tool.file_generator.description") }}</p>
    </header>

    <div class="workspace-grid">
      <NCard size="small" :title="t('file_generator.input.title')">
        <div class="form-stack">
          <label>
            <span>{{ t("file_generator.template_catalog") }}</span>
            <NSelect
              v-model:value="selectedTemplateId"
              :options="
                templates.map((template) => ({
                  label: `${template.title} · ${t(`file_generator.template_source.${template.source}`)}`,
                  value: template.id,
                }))
              "
              :placeholder="t('file_generator.template_catalog_empty')"
            />
          </label>
          <label>
            <span>{{ t("file_generator.destination") }}</span>
            <NInput
              v-model:value="destination"
              :input-props="{ 'aria-label': t('file_generator.destination') }"
            />
          </label>
          <label v-for="variable in variableDefinitions" :key="variable.name">
            <span>{{ variable.name }}</span>
            <NInput
              v-model:value="variables[variable.name]"
              :input-props="{ 'aria-label': variable.name }"
            />
          </label>
          <NCheckbox v-model:checked="overwrite">
            {{ t("file_generator.overwrite") }}
          </NCheckbox>
          <NButton
            type="primary"
            :disabled="destination.trim().length === 0"
            :loading="busy"
            @click="preview"
          >
            {{ t("file_generator.preview") }}
          </NButton>
          <details class="template-editor">
            <summary>{{ t("file_generator.edit_template") }}</summary>
            <label>
              <span>{{ t("file_generator.template") }}</span>
              <NInput
                v-model:value="templateJson"
                type="textarea"
                :input-props="{ 'aria-label': t('file_generator.template') }"
                :autosize="{ minRows: 10, maxRows: 20 }"
              />
            </label>
            <NButton :loading="busy" @click="persistTemplate">
              {{ t("file_generator.save_template") }}
            </NButton>
          </details>
        </div>
      </NCard>

      <NCard size="small" :title="t('file_generator.plan.title')">
        <p v-if="!plan" class="empty-state">
          {{ t("file_generator.plan.empty") }}
        </p>
        <template v-else>
          <table>
            <thead>
              <tr>
                <th>{{ t("file_generator.path") }}</th>
                <th>{{ t("file_generator.action") }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="file in plan.files" :key="file.path">
                <td>{{ file.path }}</td>
                <td>{{ t(`file_generator.action.${file.action}`) }}</td>
              </tr>
            </tbody>
          </table>
          <NSpace class="actions">
            <NButton type="success" :loading="busy" @click="execute">
              {{ t("file_generator.execute") }}
            </NButton>
          </NSpace>
        </template>
      </NCard>
    </div>

    <NAlert
      v-if="consentRequired"
      :show-icon="false"
      type="warning"
      :title="t('file_generator.consent.title')"
    >
      <p>{{ t("file_generator.consent.message") }}</p>
      <NSpace>
        <NButton type="warning" :loading="busy" @click="grantAndContinue">
          {{ t("file_generator.consent.allow") }}
        </NButton>
        <NButton :disabled="busy" @click="denyConsent">
          {{ t("file_generator.consent.deny") }}
        </NButton>
      </NSpace>
    </NAlert>

    <NAlert
      v-if="error && !consentRequired"
      :show-icon="false"
      type="error"
      :title="error.code"
    >
      {{ error.message }}
    </NAlert>

    <NCard v-if="report" :title="t(`file_generator.status.${report.status}`)">
      <table>
        <tbody>
          <tr v-for="file in report.files" :key="file.path">
            <td>{{ file.path }}</td>
            <td>{{ t(`file_generator.outcome.${file.outcome}`) }}</td>
            <td v-if="file.error">{{ file.error }}</td>
          </tr>
        </tbody>
      </table>
    </NCard>
  </section>
</template>

<style scoped>
.tool-page {
  display: grid;
  gap: 16px;
}

.tool-page h2,
.tool-page p {
  margin: 0;
}

.workspace-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 16px;
}

.form-stack,
label {
  display: grid;
  gap: 8px;
}

.form-stack {
  gap: 12px;
}

label span,
th {
  font-weight: 600;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  padding: 10px 8px;
  border-bottom: 1px solid rgb(128 128 128 / 25%);
  text-align: left;
}

.actions {
  margin-top: 18px;
}

.empty-state {
  color: var(--bkmt-text-muted);
}

@media (max-width: 760px) {
  .workspace-grid {
    grid-template-columns: 1fr;
  }
}
.tool-heading {
  display: grid;
  gap: 4px;
}
.tool-heading h2 {
  font-size: 1.1rem;
}
.tool-heading p {
  color: var(--bkmt-text-muted);
  font-size: 0.8rem;
}
.workspace-grid {
  align-items: start;
}
.template-editor {
  border-top: 1px solid var(--bkmt-border);
  padding-top: 10px;
}
.template-editor summary {
  cursor: pointer;
  color: var(--bkmt-text-muted);
  font-size: 0.8rem;
}
.template-editor[open] summary {
  margin-bottom: 10px;
}
.template-editor label {
  margin-bottom: 10px;
}
td {
  overflow-wrap: anywhere;
}
</style>
