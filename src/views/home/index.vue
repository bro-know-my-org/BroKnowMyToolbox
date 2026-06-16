<template>
  <ToolPage :title="t('home.title')" :description="t('home.description')" :eyebrow="t('home.eyebrow')">
    <section class="tool-grid">
      <RouterLink v-for="tool in tools" :key="tool.id" class="tool-card" :to="tool.path">
        <div class="tool-card__icon">
          <NIcon v-if="tool.icon" :component="tool.icon" />
        </div>
        <div>
          <h2>{{ t(tool.titleKey) }}</h2>
          <p>{{ t(tool.descriptionKey) }}</p>
        </div>
      </RouterLink>
    </section>

    <ToolPanel :title="t('home.directionTitle')" :description="t('home.directionDescription')">
      <div class="principles">
        <div>
          <strong>{{ t("home.principles.builtInTitle") }}</strong>
          <span>{{ t("home.principles.builtInBody") }}</span>
        </div>
        <div>
          <strong>{{ t("home.principles.shellTitle") }}</strong>
          <span>{{ t("home.principles.shellBody") }}</span>
        </div>
        <div>
          <strong>{{ t("home.principles.commandsTitle") }}</strong>
          <span>{{ t("home.principles.commandsBody") }}</span>
        </div>
      </div>
    </ToolPanel>
  </ToolPage>
</template>

<script setup lang="ts">
import { NIcon } from "naive-ui";
import { RouterLink } from "vue-router";
import { useAppI18n } from "../../i18n/useAppI18n";
import ToolPage from "../../components/app/ToolPage.vue";
import ToolPanel from "../../components/app/ToolPanel.vue";
import { tools } from "../../tools/registry";

const { t } = useAppI18n();
</script>

<style scoped lang="scss">
.tool-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 14px;
}

.tool-card {
  min-height: 132px;
  display: flex;
  gap: 14px;
  color: inherit;
  text-decoration: none;
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--bkm-panel), #121a21);
  padding: 16px;
  transition:
    border-color 0.16s ease,
    transform 0.16s ease,
    background 0.16s ease;
}

.tool-card:hover {
  border-color: var(--bkm-accent);
  background: linear-gradient(180deg, #19242c, #121b22);
  transform: translateY(-1px);
}

.tool-card__icon {
  width: 38px;
  height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  border-radius: 8px;
  color: var(--bkm-accent);
  background: var(--bkm-accent-soft);
  font-size: 22px;
}

h2 {
  margin: 0;
  color: var(--bkm-text);
  font-size: 16px;
  line-height: 1.3;
}

p {
  margin: 8px 0 0;
  color: var(--bkm-text-muted);
  font-size: 13px;
  line-height: 1.55;
}

.principles {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.principles div {
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.025);
}

.principles strong,
.principles span {
  display: block;
}

.principles strong {
  color: var(--bkm-text);
  font-size: 13px;
}

.principles span {
  margin-top: 6px;
  color: var(--bkm-text-muted);
  font-size: 12px;
  line-height: 1.5;
}

@media (max-width: 860px) {
  .principles {
    grid-template-columns: 1fr;
  }
}
</style>
