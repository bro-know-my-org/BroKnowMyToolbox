<script setup lang="ts">
defineOptions({ name: "HomeView" });

import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink, useRouter } from "vue-router";

import ToolGlyph from "../../components/shell/ToolGlyph.vue";
import { useSettingsStore } from "../../stores/settings";
import { searchRegisteredTools } from "../../tools/registry";

const { t } = useI18n();
const router = useRouter();
const query = ref("");
const filter = ref<"all" | "favorites" | "recent">("all");
const actionError = ref("");
const settings = useSettingsStore();
const { config } = storeToRefs(settings);
const filters = ["all", "favorites", "recent"] as const;
const tools = computed(() => {
  const favorites = new Set(config.value.favoriteToolIds);
  const recent = config.value.recentToolIds;
  const matches = searchRegisteredTools(query.value, t);
  if (filter.value === "recent") {
    return matches
      .filter((tool) => recent.includes(tool.id))
      .sort((a, b) => recent.indexOf(a.id) - recent.indexOf(b.id));
  }
  if (filter.value === "favorites")
    return matches.filter((tool) => favorites.has(tool.id));
  return [...matches].sort(
    (a, b) => Number(favorites.has(b.id)) - Number(favorites.has(a.id)),
  );
});

function isFavorite(id: string) {
  return config.value.favoriteToolIds.includes(id);
}
function rememberTool(id: string) {
  void settings.markRecent(id).catch(() => undefined);
}
function openFirst(event: KeyboardEvent) {
  if (event.isComposing) return;
  const tool = tools.value[0];
  if (tool) {
    void router.push(tool.route);
    rememberTool(tool.id);
  }
}
async function toggleFavorite(id: string) {
  actionError.value = "";
  try {
    await settings.toggleFavorite(id);
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}
</script>

<template>
  <section class="home-view" :aria-label="t('home.builtin_tools')">
    <header class="catalog-heading">
      <h2>{{ t("home.builtin_tools") }}</h2>
      <span>{{ t("home.tool_count", { count: tools.length }) }}</span>
    </header>
    <div class="catalog-toolbar">
      <div
        class="tool-filters"
        role="group"
        :aria-label="t('home.filter_label')"
      >
        <button
          v-for="value in filters"
          :key="value"
          type="button"
          :aria-pressed="filter === value"
          @click="filter = value"
        >
          {{ t(`home.filter.${value}`) }}
        </button>
      </div>
      <input
        v-model="query"
        class="tool-search"
        type="search"
        :aria-label="t('home.search_label')"
        :placeholder="t('home.search_placeholder')"
        @keydown.enter="openFirst"
      />
    </div>
    <div class="tool-list">
      <article v-for="tool in tools" :key="tool.id" class="tool-row">
        <RouterLink
          class="tool-main"
          :to="tool.route"
          @click="rememberTool(tool.id)"
        >
          <ToolGlyph :tool-id="tool.id" />
          <span class="tool-copy">
            <strong>{{ t(tool.titleKey) }}</strong>
            <small>{{ t(tool.descriptionKey) }}</small>
          </span>
          <span class="open-label">{{ t("home.open_tool") }}</span>
        </RouterLink>
        <button
          class="favorite-button"
          type="button"
          :aria-pressed="isFavorite(tool.id)"
          :aria-label="`${t(isFavorite(tool.id) ? 'home.unfavorite' : 'home.favorite')} · ${t(tool.titleKey)}`"
          @click="toggleFavorite(tool.id)"
        >
          {{ t(isFavorite(tool.id) ? "home.favorited" : "home.favorite") }}
        </button>
      </article>
    </div>
    <p v-if="!tools.length" class="empty-state" role="status">
      {{
        t(
          query.trim()
            ? "home.no_results"
            : filter === "recent"
              ? "home.history_empty"
              : "home.no_favorites",
        )
      }}
    </p>
    <p v-if="actionError" class="action-error" role="alert">
      {{ actionError }}
    </p>
  </section>
</template>

<style scoped>
.home-view {
  width: 100%;
}
.catalog-heading {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 16px;
}
.catalog-heading h2 {
  margin: 0;
  font-size: 1.1rem;
}
.catalog-heading > span {
  color: var(--bkmt-text-muted);
  font-size: 0.75rem;
}
.catalog-toolbar {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 12px;
}
.tool-filters {
  display: flex;
  gap: 4px;
}
.tool-filters button,
.favorite-button {
  padding: 6px 10px;
  border: 1px solid transparent;
  border-radius: 4px;
  color: var(--bkmt-text-muted);
  background: transparent;
  cursor: pointer;
  font-size: 0.8rem;
}
.tool-filters button[aria-pressed="true"] {
  color: var(--bkmt-text);
  background: var(--bkmt-surface);
  border-color: var(--bkmt-border);
  font-weight: 600;
}
.tool-search {
  width: min(320px, 40%);
  min-height: 34px;
  padding: 5px 10px;
  color: var(--bkmt-text);
  background: var(--bkmt-surface);
  border: 1px solid var(--bkmt-border);
  border-radius: 4px;
  font-size: 0.8rem;
}
.tool-list {
  border-top: 1px solid var(--bkmt-border);
}
.tool-row {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--bkmt-border);
}
.tool-row:hover {
  background: var(--bkmt-surface);
}
.tool-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: var(--bkmt-tool-row-height, 68px);
  padding: 12px;
  color: var(--bkmt-text);
  text-decoration: none;
}
.tool-copy {
  display: grid;
  gap: 3px;
  flex: 1;
  min-width: 0;
}
.tool-copy strong {
  font-size: 0.9rem;
}
.tool-copy small {
  color: var(--bkmt-text-muted);
  font-size: 0.78rem;
}
.open-label {
  color: var(--bkmt-text-muted);
  font-size: 0.78rem;
}
.favorite-button {
  margin-right: 8px;
  flex-shrink: 0;
}
.favorite-button:hover,
.tool-filters button:hover {
  background: var(--bkmt-control);
  color: var(--bkmt-text);
}
.favorite-button[aria-pressed="true"] {
  color: var(--bkmt-text);
  border-color: var(--bkmt-border-strong);
}
.empty-state {
  padding: 24px 12px;
  color: var(--bkmt-text-muted);
}
.action-error {
  color: var(--bkmt-error);
}
@media (max-width: 640px) {
  .catalog-toolbar {
    flex-direction: column;
    gap: 10px;
  }
  .tool-search {
    width: 100%;
  }
  .open-label {
    display: none;
  }
  .tool-main {
    padding-left: 4px;
  }
}
</style>
