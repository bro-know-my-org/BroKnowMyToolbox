<template>
  <div class="error-container">
    <n-tooltip trigger="hover">
      <template #trigger>
        <n-result status="404" title="页面未找到" description="抱歉，您访问的页面不存在或已被移除" size="large">
          <template #footer>
            <n-space justify="center">
              <n-button type="primary" @click="handleBack"> 返回首页 </n-button>
              <n-button @click="handleGoBack"> 返回上一页 </n-button>
            </n-space>
          </template>
        </n-result>
      </template>
      棍木棍木棍？
    </n-tooltip>
  </div>
</template>

<script setup lang="ts">
import { NResult, NButton, NSpace, NTooltip } from "naive-ui";
import { useRouter } from "vue-router";

const router = useRouter();

const handleBack = () => {
  router.push("/");
};

const handleGoBack = () => {
  router.back();
};
</script>

<style scoped>
.error-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px;
}
</style>
