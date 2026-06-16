import { useI18n } from "vue-i18n";
import { useDebugStore } from "../stores/debug";

export function useAppI18n() {
  const i18n = useI18n();
  const debugStore = useDebugStore();

  function t(key: string, ...args: unknown[]): string {
    if (debugStore.enabled && debugStore.altPressed) {
      return key;
    }

    return i18n.t(key, ...(args as []));
  }

  return {
    ...i18n,
    t,
  };
}
