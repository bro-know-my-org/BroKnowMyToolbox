<template>
  <div id="app">
    <n-config-provider :theme="darkTheme" :theme-overrides="themeOverrides">
      <n-message-provider>
        <router-view />
      </n-message-provider>
      <n-global-style />
    </n-config-provider>
  </div>
</template>

<script setup lang="ts">
import { darkTheme, NConfigProvider, NGlobalStyle, NMessageProvider, type GlobalThemeOverrides } from "naive-ui";

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#4fb6bd",
    primaryColorHover: "#67c8cf",
    primaryColorPressed: "#329ea3",
    primaryColorSuppl: "#67c8cf",
    bodyColor: "#10161b",
    cardColor: "#151d24",
    modalColor: "#151d24",
    popoverColor: "#151d24",
    tableColor: "#151d24",
    inputColor: "#0f151b",
    borderColor: "#26333d",
    textColorBase: "#e8eef2",
    textColor1: "#e8eef2",
    textColor2: "#b5c2ca",
    textColor3: "#7f8d96",
  },
  Button: {
    borderRadiusMedium: "6px",
    borderRadiusSmall: "6px",
  },
  Card: {
    borderRadius: "8px",
  },
};
</script>

<style scoped></style>
<style></style>
