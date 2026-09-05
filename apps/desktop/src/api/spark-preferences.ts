import { invoke } from "@tauri-apps/api/core";
import type {
  SparkAnalyzerPreferences,
  SparkAnalyzerPreferencesStore,
} from "@bro-know-my/spark-analyzer";

type StoredSparkPreferences = {
  providerId: string;
  baseUrl: string;
  model: string;
  temperature: number;
};

export function createSparkPreferencesStore(): SparkAnalyzerPreferencesStore {
  return {
    async load() {
      const stored = await invoke<StoredSparkPreferences | null>(
        "load_spark_preferences_command",
      );
      if (!stored) return null;
      return {
        providerId: stored.providerId,
        base_url: stored.baseUrl,
        model: stored.model,
        temperature: stored.temperature,
      };
    },
    async save(preferences: SparkAnalyzerPreferences) {
      await invoke("save_spark_preferences_command", {
        preferences: {
          providerId: preferences.providerId ?? "custom",
          baseUrl: preferences.base_url ?? "",
          model: preferences.model ?? "",
          temperature: preferences.temperature ?? 0.2,
        } satisfies StoredSparkPreferences,
      });
    },
  };
}
