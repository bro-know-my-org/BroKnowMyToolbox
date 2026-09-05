import { afterEach, expect, test, vi } from "vitest";

import { createSparkPreferencesStore } from "../src/api/spark-preferences";

const bridge = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke }));

afterEach(() => vi.clearAllMocks());

test("Spark preferences are mapped to the Toolbox data-root commands", async () => {
  bridge.invoke.mockResolvedValueOnce({
    providerId: "openai",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1-mini",
    temperature: 0.3,
  });
  const store = createSparkPreferencesStore();

  await expect(store.load()).resolves.toEqual({
    providerId: "openai",
    base_url: "https://api.openai.com/v1",
    model: "gpt-4.1-mini",
    temperature: 0.3,
  });
  await store.save({
    providerId: "deepseek",
    base_url: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    temperature: 0.2,
  });

  expect(bridge.invoke).toHaveBeenNthCalledWith(
    1,
    "load_spark_preferences_command",
  );
  expect(bridge.invoke).toHaveBeenNthCalledWith(
    2,
    "save_spark_preferences_command",
    {
      preferences: {
        providerId: "deepseek",
        baseUrl: "https://api.deepseek.com/v1",
        model: "deepseek-chat",
        temperature: 0.2,
      },
    },
  );
});
