<template>
  <section class="tool-page">
    <header class="tool-page__header">
      <div>
        <p v-if="eyebrow" class="tool-page__eyebrow">{{ eyebrow }}</p>
        <h1>{{ title }}</h1>
        <p v-if="description" class="tool-page__description">{{ description }}</p>
      </div>
      <div v-if="$slots.actions" class="tool-page__actions">
        <slot name="actions" />
      </div>
    </header>

    <div class="tool-page__body">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  description?: string;
  eyebrow?: string;
}>();
</script>

<style scoped lang="scss">
.tool-page {
  width: 100%;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tool-page__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.tool-page__eyebrow {
  margin: 0 0 4px;
  color: var(--bkm-accent);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0;
}

h1 {
  margin: 0;
  color: var(--bkm-text);
  font-size: 24px;
  line-height: 1.2;
  font-weight: 700;
}

.tool-page__description {
  max-width: 760px;
  margin: 8px 0 0;
  color: var(--bkm-text-muted);
  font-size: 13px;
  line-height: 1.6;
}

.tool-page__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.tool-page__body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

@media (max-width: 720px) {
  .tool-page__header {
    flex-direction: column;
  }
}
</style>
