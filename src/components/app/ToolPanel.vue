<template>
  <section class="tool-panel">
    <header v-if="title || $slots.actions" class="tool-panel__header">
      <div>
        <h2 v-if="title">{{ title }}</h2>
        <p v-if="description">{{ description }}</p>
      </div>
      <div v-if="$slots.actions" class="tool-panel__actions">
        <slot name="actions" />
      </div>
    </header>
    <slot />
  </section>
</template>

<script setup lang="ts">
defineProps<{
  title?: string;
  description?: string;
}>();
</script>

<style scoped lang="scss">
.tool-panel {
  width: 100%;
  border: 1px solid var(--bkm-border);
  border-radius: 8px;
  background: linear-gradient(180deg, var(--bkm-panel), #121a21);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
  padding: 16px;
}

.tool-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}

h2 {
  margin: 0;
  color: var(--bkm-text);
  font-size: 16px;
  line-height: 1.3;
  font-weight: 650;
}

p {
  margin: 6px 0 0;
  color: var(--bkm-text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.tool-panel__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
