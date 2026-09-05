import { createI18n } from "vue-i18n";

import enUS from "./locales/en-US.json";
import zhCN from "./locales/zh-CN.json";

export function builtInLocaleMessages(locale: string): Record<string, string> {
  if (locale === "zh-CN") return { ...zhCN };
  if (locale === "en-US") return { ...enUS };
  return {};
}

export function createAppI18n() {
  return createI18n({
    legacy: false,
    locale: "zh-CN",
    fallbackLocale: "en-US",
    messageResolver: (messages, key) => {
      const value = (messages as Record<string, unknown>)[key];
      return typeof value === "string" ? value : null;
    },
    messages: {
      "en-US": builtInLocaleMessages("en-US"),
      "zh-CN": builtInLocaleMessages("zh-CN"),
    },
  });
}
