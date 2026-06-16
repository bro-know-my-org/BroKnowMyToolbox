<template>
  <div class="layout-container">
    <AppHeader />
    <div class="main-content">
      <Sidebar />
      <main class="content-area">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppHeader from "./components/AppHeader/index.vue";
import Sidebar from "./components/Sidebar/index.vue";
</script>

<style scoped lang="scss">
.layout-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding-top: var(--bkm-title-bar-height);
  box-sizing: border-box;
  background:
    radial-gradient(circle at top left, rgba(79, 182, 189, 0.14), transparent 34rem),
    var(--bkm-bg);
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.content-area {
  flex: 1;
  padding: 18px;
  overflow-y: auto;
  background: transparent;
}
</style>
