import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { useDebugStore } from "./stores/debug";
import { useSettingsStore } from "./stores/settings";
import { initToolRoutes } from "./tools/registry";
import { i18n, setI18nLanguage } from "./i18n";
import "./layout/styles/variables.scss";
import "./styles/app.scss";

const app = createApp(App);
const pinia = createPinia();

function bootstrap() {
  app.use(pinia);
  const debugStore = useDebugStore();
  const settingsStore = useSettingsStore();
  settingsStore.applySettings();
  setI18nLanguage(settingsStore.language);
  window.addEventListener("keydown", (event) => debugStore.setAltPressed(event.altKey));
  window.addEventListener("keyup", (event) => debugStore.setAltPressed(event.altKey));
  window.addEventListener("blur", () => debugStore.setAltPressed(false));

  initToolRoutes(router);

  app.use(i18n);
  app.use(router);
  app.mount("#app");
}

bootstrap();
