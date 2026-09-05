<script setup lang="ts">
defineProps<{ toolId?: string }>();
</script>

<template>
  <span class="glyph-placeholder" aria-hidden="true" />
</template>

<style scoped>
.glyph-placeholder {
  display: inline-block;
  flex: 0 0 16px;
  width: 16px;
  height: 16px;
  border: 1px dashed var(--bkmt-text-muted);
  border-radius: 2px;
  opacity: 0.55;
}
</style>
