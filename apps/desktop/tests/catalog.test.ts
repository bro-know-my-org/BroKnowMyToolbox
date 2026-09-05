import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/vue";
import { createPinia } from "pinia";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

import App from "../src/App.vue";
import { createAppI18n } from "../src/i18n";
import { createAppRouter } from "../src/router";
import { useSettingsStore } from "../src/stores/settings";

const native = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: native.invoke }));
beforeEach(() => {
  native.invoke.mockReset();
  native.invoke.mockImplementation(async (command: string) => {
    if (command === "load_app_config_command")
      return {
        schemaVersion: 1,
        locale: "zh-CN",
        theme: "light",
        accentColor: "#78a9ff",
        fontScalePercent: 100,
        density: "comfortable",
        reducedMotion: true,
        favoriteToolIds: [],
        recentToolIds: [],
      };
    if (command === "save_app_config_command") return;
    if (command === "load_locale_override_command") return {};
    if (command === "runtime_diagnostics_command")
      return { dataRoot: "/tmp/toolbox", dataRootSource: "system" };
    if (command === "list_file_templates_command")
      return { templates: [], warnings: [] };
    throw new Error(`Unexpected command: ${command}`);
  });
});
afterEach(cleanup);

async function renderCatalog() {
  const router = createAppRouter();
  const pinia = createPinia();
  const settings = useSettingsStore(pinia);
  await settings.initialize();
  await router.push("/");
  await router.isReady();
  render(App, { global: { plugins: [pinia, router, createAppI18n()] } });
  return {
    router,
    settings,
    catalog: within(screen.getByRole("region", { name: "工具" })),
  };
}

test("favorites can be filtered, searched and removed without leaving the catalog", async () => {
  const { catalog } = await renderCatalog();
  await fireEvent.click(
    catalog.getByRole("button", { name: "收藏工具 · 文件生成器" }),
  );
  await waitFor(() =>
    expect(
      catalog
        .getByRole("button", { name: "取消收藏 · 文件生成器" })
        .getAttribute("aria-pressed"),
    ).toBe("true"),
  );
  await fireEvent.click(catalog.getByRole("button", { name: "收藏" }));
  expect(catalog.getAllByRole("article")).toHaveLength(1);
  expect(catalog.queryByText("Spark Analyzer")).toBeNull();
  await fireEvent.update(catalog.getByRole("searchbox"), "spark");
  expect(catalog.getByRole("status").textContent).toContain("没有匹配的工具");
  await fireEvent.update(catalog.getByRole("searchbox"), "");
  await fireEvent.click(
    catalog.getByRole("button", { name: "取消收藏 · 文件生成器" }),
  );
  await waitFor(() =>
    expect(catalog.getByRole("status").textContent).toContain("还没有收藏"),
  );
});

test("search Enter opens a match and switching tools preserves generator inputs", async () => {
  const { router, settings } = await renderCatalog();
  const search = screen.getByRole("searchbox");
  await fireEvent.update(search, "template");
  await fireEvent.keyDown(search, { key: "Enter", isComposing: true });
  expect(router.currentRoute.value.path).toBe("/");
  await fireEvent.keyDown(search, { key: "Enter" });
  await screen.findByRole("heading", { name: "文件生成器" });
  await fireEvent.update(screen.getByLabelText("目标目录"), "/tmp/draft");
  await fireEvent.update(screen.getByLabelText("name"), "Unfinished work");
  const nav = within(screen.getByRole("navigation"));
  await fireEvent.click(nav.getByRole("link", { name: "工具箱" }));
  await screen.findByRole("heading", { name: "工具" });
  await fireEvent.click(screen.getByRole("button", { name: "最近使用" }));
  const recent = within(screen.getByRole("region", { name: "工具" }));
  expect(recent.getAllByRole("article")).toHaveLength(1);
  await fireEvent.click(recent.getByRole("link", { name: /文件生成器/ }));
  await waitFor(() =>
    expect(screen.getByLabelText("目标目录")).toHaveProperty(
      "value",
      "/tmp/draft",
    ),
  );
  expect(screen.getByLabelText("name")).toHaveProperty(
    "value",
    "Unfinished work",
  );
  await waitFor(() =>
    expect(settings.config.recentToolIds[0]).toBe("file-generator"),
  );
});

test("empty searches never navigate and recent tools retain their usage order", async () => {
  const { router, settings, catalog } = await renderCatalog();
  await settings.markRecent("file-generator");
  await settings.markRecent("spark-analyzer");
  await fireEvent.click(catalog.getByRole("button", { name: "最近使用" }));
  expect(catalog.getAllByRole("link")[0].textContent).toContain(
    "Spark Analyzer",
  );
  await fireEvent.update(catalog.getByRole("searchbox"), "not-a-tool");
  await fireEvent.keyDown(catalog.getByRole("searchbox"), { key: "Enter" });
  expect(router.currentRoute.value.path).toBe("/");
  expect(catalog.getByRole("status").textContent).toContain("没有匹配的工具");
});
