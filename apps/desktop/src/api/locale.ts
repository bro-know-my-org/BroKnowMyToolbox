import { invoke } from "@tauri-apps/api/core";

export function loadLocaleOverride(
  locale: string,
): Promise<Record<string, string>> {
  return invoke("load_locale_override_command", { locale });
}
