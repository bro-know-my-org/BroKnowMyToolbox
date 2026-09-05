import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/vue";
import { createPinia } from "pinia";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

import App from "../src/App.vue";
import { createAppI18n, builtInLocaleMessages } from "../src/i18n";
import { createAppRouter } from "../src/router";
import { useSettingsStore } from "../src/stores/settings";

const native = vi.hoisted(() => ({ invoke: vi.fn(), loadPack: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
vi.mock("../src/api/locale", () => ({ loadLocaleOverride: native.loadPack }));

beforeEach(() => {
  native.invoke.mockReset();
  native.loadPack.mockReset().mockResolvedValue({});
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

async function renderLocale(locale = "en-US") {
  const pinia = createPinia();
  const settings = useSettingsStore(pinia);
  const config = {
    ...settings.config,
    locale,
    favoriteToolIds: [],
    recentToolIds: [],
  };
  native.invoke.mockImplementation((command: string) => {
    if (command === "load_app_config_command") return Promise.resolve(config);
    if (command === "save_app_config_command") return Promise.resolve();
    if (command === "runtime_diagnostics_command")
      return Promise.resolve({ dataRoot: "/data", dataRootSource: "system" });
    return Promise.reject(new Error(`unexpected command: ${command}`));
  });
  await settings.initialize();
  const router = createAppRouter();
  await router.push("/");
  await router.isReady();
  const i18n = createAppI18n();
  render(App, { global: { plugins: [pinia, router, i18n] } });
  return { settings, i18n };
}

test("removing a language override restores the built-in messages", async () => {
  native.loadPack.mockResolvedValueOnce({
    "tool.file_generator.name": "Custom Files",
  });
  const { settings, i18n } = await renderLocale();
  expect((await screen.findAllByText("Custom Files")).length).toBeGreaterThan(
    0,
  );
  await settings.update({ locale: "zh-CN" });
  await waitFor(() => expect(document.documentElement.lang).toBe("zh-CN"));
  await settings.update({ locale: "en-US" });
  await waitFor(() =>
    expect(i18n.global.t("tool.file_generator.name")).toBe(
      builtInLocaleMessages("en-US")["tool.file_generator.name"],
    ),
  );
  expect(screen.queryByText("Custom Files")).toBeNull();
});

test("a failed override reload cannot retain stale translations", async () => {
  native.loadPack.mockResolvedValueOnce({
    "tool.file_generator.name": "Custom Files",
  });
  const { settings, i18n } = await renderLocale();
  await waitFor(() =>
    expect(i18n.global.t("tool.file_generator.name")).toBe("Custom Files"),
  );
  await settings.update({ locale: "zh-CN" });
  await waitFor(() => expect(document.documentElement.lang).toBe("zh-CN"));
  native.loadPack.mockRejectedValueOnce(new Error("invalid pack"));
  await settings.update({ locale: "en-US" });
  await waitFor(() =>
    expect(i18n.global.t("tool.file_generator.name")).toBe(
      builtInLocaleMessages("en-US")["tool.file_generator.name"],
    ),
  );
});

test("an old language request cannot overwrite a newer pack after switching back", async () => {
  let finishOld!: (value: Record<string, string>) => void;
  native.loadPack.mockReturnValueOnce(
    new Promise<Record<string, string>>((resolve) => {
      finishOld = resolve;
    }),
  );
  const { settings, i18n } = await renderLocale();
  await settings.update({ locale: "zh-CN" });
  await waitFor(() => expect(document.documentElement.lang).toBe("zh-CN"));
  native.loadPack.mockResolvedValueOnce({
    "tool.file_generator.name": "Fresh Files",
  });
  await settings.update({ locale: "en-US" });
  await waitFor(() =>
    expect(i18n.global.t("tool.file_generator.name")).toBe("Fresh Files"),
  );
  finishOld({ "tool.file_generator.name": "Stale Files" });
  await Promise.resolve();
  await Promise.resolve();
  expect(i18n.global.t("tool.file_generator.name")).toBe("Fresh Files");
});

test("system language changes are reactive but do not override an explicit locale", async () => {
  const language = vi
    .spyOn(navigator, "language", "get")
    .mockReturnValue("en-US");
  const { settings } = await renderLocale("system");
  expect(document.documentElement.lang).toBe("en-US");
  language.mockReturnValue("zh-CN");
  await fireEvent(window, new Event("languagechange"));
  await waitFor(() => expect(document.documentElement.lang).toBe("zh-CN"));
  await settings.update({ locale: "en-US" });
  await fireEvent(window, new Event("languagechange"));
  expect(document.documentElement.lang).toBe("en-US");
});
