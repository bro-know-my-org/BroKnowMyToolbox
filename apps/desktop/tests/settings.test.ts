import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, test, vi } from "vitest";

import { useSettingsStore } from "../src/stores/settings";

const api = vi.hoisted(() => ({
  loadAppConfig: vi.fn(),
  saveAppConfig: vi.fn(),
}));

vi.mock("../src/api/config", () => api);

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  api.loadAppConfig.mockResolvedValue({
    schemaVersion: 1,
    locale: "system",
    theme: "system",
    accentColor: "#78a9ff",
    fontScalePercent: 100,
    density: "comfortable",
    reducedMotion: false,
    favoriteToolIds: [],
    recentToolIds: [],
  });
  api.saveAppConfig.mockResolvedValue(undefined);
});

test("settings load from the shared data root and persist favorites", async () => {
  const store = useSettingsStore();

  await store.initialize();
  await store.toggleFavorite("file-generator");

  expect(store.config.favoriteToolIds).toEqual(["file-generator"]);
  expect(api.saveAppConfig).toHaveBeenCalledWith(
    expect.objectContaining({ favoriteToolIds: ["file-generator"] }),
  );
  expect(document.documentElement.style.getPropertyValue("--bkmt-accent")).toBe(
    "#78a9ff",
  );
});

test("recent tools are unique and capped in recency order", async () => {
  const store = useSettingsStore();
  await store.initialize();

  for (const id of ["a", "b", "a", "c"]) {
    await store.markRecent(id);
  }

  expect(store.config.recentToolIds.slice(0, 3)).toEqual(["c", "a", "b"]);
});

test("a settings save failure remains visible to the UI", async () => {
  const store = useSettingsStore();
  await store.initialize();
  api.saveAppConfig.mockRejectedValueOnce(new Error("disk unavailable"));

  await expect(store.update({ theme: "dark" })).rejects.toThrow(
    "disk unavailable",
  );

  expect(store.saveError).toContain("disk unavailable");
});

test("a mutation waits for delayed initialization before saving", async () => {
  let finishLoad:
    | ((value: Awaited<ReturnType<typeof api.loadAppConfig>>) => void)
    | undefined;
  api.loadAppConfig.mockReturnValueOnce(
    new Promise((resolve) => {
      finishLoad = resolve;
    }),
  );
  const store = useSettingsStore();
  const update = store.update({ theme: "dark" });

  expect(api.saveAppConfig).not.toHaveBeenCalled();
  finishLoad?.({
    schemaVersion: 1,
    locale: "en-US",
    theme: "light",
    accentColor: "#123456",
    fontScalePercent: 110,
    density: "compact",
    reducedMotion: true,
    favoriteToolIds: ["file-generator"],
    recentToolIds: [],
  });
  await update;

  expect(api.saveAppConfig).toHaveBeenCalledWith(
    expect.objectContaining({
      locale: "en-US",
      theme: "dark",
      accentColor: "#123456",
      favoriteToolIds: ["file-generator"],
    }),
  );
});

test("failed initialization keeps defaults read-only", async () => {
  api.loadAppConfig.mockRejectedValueOnce(new Error("config is corrupt"));
  const store = useSettingsStore();

  await expect(store.update({ theme: "dark" })).rejects.toThrow(
    "settings are read-only",
  );

  expect(api.saveAppConfig).not.toHaveBeenCalled();
  expect(store.loadError).toContain("config is corrupt");
});

test("serialized Rust errors expose their message", async () => {
  api.loadAppConfig.mockRejectedValueOnce({
    code: "config_storage_failed",
    message: "permission denied",
  });
  const store = useSettingsStore();

  await store.initialize();

  expect(store.loadError).toBe("permission denied");
});
