import { createI18n } from "vue-i18n";
import zhCN from "./locales/zh-CN";
import enUS from "./locales/en-US";
import { DEFAULT_LANGUAGE, type AppLanguage } from "../stores/settings";

export const messages = {
  "zh-CN": zhCN,
  "en-US": enUS,
};

export const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LANGUAGE,
  fallbackLocale: "zh-CN",
  messages,
});

export function setI18nLanguage(language: AppLanguage) {
  i18n.global.locale.value = language;
  document.documentElement.lang = language;
}
