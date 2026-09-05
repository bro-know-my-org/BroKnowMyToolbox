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
import { createAppI18n } from "../src/i18n";
import { createAppRouter } from "../src/router";
import { useSettingsStore } from "../src/stores/settings";

const native = vi.hoisted(() => ({
  invoke: vi.fn(),
  openUrl: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: native.openUrl }));

const initialConfig = {
  schemaVersion: 1,
  locale: "zh-CN",
  theme: "system",
  accentColor: "#78a9ff",
  fontScalePercent: 100,
  density: "comfortable",
  reducedMotion: false,
  favoriteToolIds: [],
  recentToolIds: [],
};

beforeEach(() => {
  vi.clearAllMocks();
  native.invoke.mockImplementation((command: string) => {
    if (command === "load_app_config_command")
      return Promise.resolve(structuredClone(initialConfig));
    if (command === "load_locale_override_command") return Promise.resolve({});
    if (command === "save_app_config_command") return Promise.resolve();
    if (command === "runtime_diagnostics_command")
      return Promise.resolve({
        dataRoot: "/data/bkmt",
        dataRootSource: "system",
      });
    if (command === "check_for_update_command")
      return Promise.resolve({
        available: true,
        currentVersion: "0.1.0",
        latestVersion: "0.2.0",
        releaseUrl:
          "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0",
      });
    return Promise.reject(new Error(`unexpected command ${command}`));
  });
  native.openUrl.mockResolvedValue(undefined);
});

afterEach(cleanup);

async function renderSettings() {
  const router = createAppRouter();
  const pinia = createPinia();
  await router.push("/");
  await router.isReady();
  render(App, {
    global: { plugins: [pinia, router, createAppI18n()] },
  });
  await fireEvent.click(await screen.findByRole("button", { name: "设置" }));
  return useSettingsStore(pinia);
}

test("settings expose every adjustable visual preference as a labelled control", async () => {
  await renderSettings();

  expect(screen.getByLabelText("语言")).toBeTruthy();
  expect(screen.getByLabelText("主题")).toBeTruthy();
  expect(screen.getByLabelText("强调色")).toBeTruthy();
  expect(screen.getByLabelText(/字号/)).toBeTruthy();
  expect(screen.getByLabelText("界面密度")).toBeTruthy();
  expect(screen.getByRole("switch", { name: "减少动画" })).toBeTruthy();
  expect(screen.getByText("/data/bkmt")).toBeTruthy();
});

test("settings persist visual preferences and apply runtime CSS state", async () => {
  const store = await renderSettings();

  await store.update({
    theme: "dark",
    locale: "en-US",
    accentColor: "#ff6600",
    fontScalePercent: 115,
    density: "compact",
    reducedMotion: true,
  });

  expect(native.invoke).toHaveBeenCalledWith("save_app_config_command", {
    config: expect.objectContaining({
      theme: "dark",
      locale: "en-US",
      fontScalePercent: 115,
      density: "compact",
      reducedMotion: true,
    }),
  });
  expect(document.documentElement.dataset.theme).toBe("dark");
  expect(document.documentElement.dataset.density).toBe("compact");
  expect(document.documentElement.classList.contains("reduced-motion")).toBe(
    true,
  );
  expect(
    document.documentElement.style.getPropertyValue("--bkmt-font-scale"),
  ).toBe("1.15");
  await waitFor(() =>
    expect(screen.getByText("Appearance and experience")).toBeTruthy(),
  );
});

test("unsaved visual changes update the application theme immediately", async () => {
  const store = await renderSettings();

  store.preview({
    ...store.config,
    theme: "light",
    favoriteToolIds: [...store.config.favoriteToolIds],
    recentToolIds: [...store.config.recentToolIds],
  });

  await waitFor(() =>
    expect(document.documentElement.dataset.theme).toBe("light"),
  );
});

test("manual update check opens the validated release download page", async () => {
  await renderSettings();

  await fireEvent.click(screen.getByRole("button", { name: "检查更新" }));
  expect(await screen.findByText("发现新版本 0.2.0")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "打开下载页面" }));

  expect(native.openUrl).toHaveBeenCalledWith(
    "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0",
  );
});

test("saving an open settings draft preserves newer navigation state", async () => {
  const store = await renderSettings();
  await store.toggleFavorite("file-generator");

  await fireEvent.click(screen.getByRole("button", { name: "保存设置" }));

  await waitFor(() =>
    expect(native.invoke).toHaveBeenLastCalledWith("save_app_config_command", {
      config: expect.objectContaining({ favoriteToolIds: ["file-generator"] }),
    }),
  );
});

test("download launch failures remain visible in the settings drawer", async () => {
  native.openUrl.mockRejectedValueOnce({ message: "opener unavailable" });
  await renderSettings();
  await fireEvent.click(screen.getByRole("button", { name: "检查更新" }));
  await fireEvent.click(
    await screen.findByRole("button", { name: "打开下载页面" }),
  );

  expect(await screen.findByText("opener unavailable")).toBeTruthy();
});

test("an in-flight settings save blocks duplicate submission and closing", async () => {
  const original = native.invoke.getMockImplementation()!;
  let finishSave!: () => void;
  native.invoke.mockImplementation((command: string, ...args: unknown[]) => {
    if (command === "save_app_config_command")
      return new Promise<void>((resolve) => {
        finishSave = resolve;
      });
    return original(command, ...args);
  });
  await renderSettings();
  const button = screen.getByRole("button", { name: "保存设置" });
  await fireEvent.click(button);
  await waitFor(() => expect(button.hasAttribute("disabled")).toBe(true));
  await fireEvent.click(button);
  await fireEvent.keyDown(document, { key: "Escape" });
  expect(screen.getByLabelText("语言")).toBeTruthy();
  expect(
    native.invoke.mock.calls.filter(
      ([command]) => command === "save_app_config_command",
    ),
  ).toHaveLength(1);
  finishSave();
  await waitFor(() =>
    expect(screen.queryByRole("button", { name: "保存设置" })).toBeNull(),
  );
});

test("failed settings saves keep the draft available for retry", async () => {
  const original = native.invoke.getMockImplementation()!;
  native.invoke.mockImplementation((command: string, ...args: unknown[]) => {
    if (command === "save_app_config_command")
      return Promise.reject(new Error("disk unavailable"));
    return original(command, ...args);
  });
  await renderSettings();
  const motion = screen.getByRole("switch", { name: "减少动画" });
  await fireEvent.click(motion);
  await fireEvent.click(screen.getByRole("button", { name: "保存设置" }));
  await screen.findByText(/disk unavailable/);
  expect(motion.getAttribute("aria-checked")).toBe("true");
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "保存设置" }).hasAttribute("disabled"),
    ).toBe(false),
  );
});

test("structured runtime diagnostic failures remain visible in settings", async () => {
  const original = native.invoke.getMockImplementation()!;
  native.invoke.mockImplementation((command: string, ...args: unknown[]) => {
    if (command === "runtime_diagnostics_command")
      return Promise.reject({
        code: "data_root_not_unicode",
        message: "the data directory cannot be represented as UTF-8",
      });
    return original(command, ...args);
  });
  await renderSettings();
  const alert = await screen.findByRole("alert");
  expect(alert.textContent).toContain("读取实际数据目录失败");
  expect(alert.textContent).toContain("cannot be represented as UTF-8");
  expect(alert.textContent).not.toContain("[object Object]");
});
