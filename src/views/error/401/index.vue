<template>
  <div class="error-container">
    <n-tooltip>
      <template #trigger>
        <n-result status="info" title="未授权访问" description="抱歉，您没有权限访问该页面，请登录后重试" size="large">
          <template #footer>
            <n-space justify="center">
              <n-button type="primary" @click="handleLogin" size="large"> 重新登录 </n-button>
              <n-button @click="handleBack" size="large"> 返回首页 </n-button>
            </n-space>
          </template>
        </n-result>
      </template>
      你没权限啊你没权限
    </n-tooltip>
  </div>
</template>

<script setup lang="ts">
import { NResult, NButton, NSpace, NTooltip } from "naive-ui";
import { useRouter } from "vue-router";

const router = useRouter();

const handleLogin = () => {
  router.push("/login");
};

const handleBack = () => {
  router.push("/");
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
