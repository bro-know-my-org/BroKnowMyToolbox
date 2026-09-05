import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/vue";
import { afterEach, expect, test } from "vitest";
import { createPinia } from "pinia";

import App from "../src/App.vue";
import { createAppI18n } from "../src/i18n";
import { createAppRouter } from "../src/router";
import { useSettingsStore } from "../src/stores/settings";
import { registeredTools } from "../src/tools/registry";

afterEach(cleanup);

test.each(registeredTools)(
  "$id loader resolves a Vue component, not a module namespace",
  async (tool) => {
    const component = await tool.component();
    expect(component).toHaveProperty("setup", expect.any(Function));
    expect(component).not.toHaveProperty("default");
  },
);

async function renderApp(path = "/") {
  const router = createAppRouter();
  const pinia = createPinia();
  const settings = useSettingsStore(pinia);
  settings.preview({
    ...settings.config,
    locale: "zh-CN",
    favoriteToolIds: [...settings.config.favoriteToolIds],
    recentToolIds: [...settings.config.recentToolIds],
  });
  await router.push(path);
  await router.isReady();

  return {
    ...render(App, {
      global: {
        plugins: [pinia, router, createAppI18n()],
      },
    }),
    router,
  };
}

test("the desktop shell identifies the product", async () => {
  await renderApp();

  expect(
    screen.getByRole("link", { name: "Bro Know My Toolbox" }),
  ).toBeTruthy();
  expect(screen.getAllByText("文件生成器").length).toBeGreaterThan(0);
  expect(screen.getAllByText("Spark Analyzer").length).toBeGreaterThan(0);
  expect(screen.getByRole("navigation", { name: "主导航" })).toBeTruthy();
  expect(screen.getByRole("heading", { name: "工具" })).toBeTruthy();
});

test("registered tools are reachable through derived routes", async () => {
  await renderApp("/tools/file-generator");

  expect(screen.getByRole("heading", { name: "文件生成器" })).toBeTruthy();
  expect(
    screen.getByText("从模板生成文件前，先创建可审查的生成计划。"),
  ).toBeTruthy();
  expect(screen.getByRole("navigation", { name: "主导航" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "命令面板" })).toBeTruthy();
});

test("tool search filters the catalog by registered keywords", async () => {
  await renderApp();

  await fireEvent.update(screen.getByRole("searchbox"), "template");

  const catalog = within(screen.getByRole("region", { name: "工具" }));
  expect(catalog.getByText("文件生成器")).toBeTruthy();
  expect(catalog.queryByText("Spark Analyzer")).toBeNull();
});

test("the global command palette opens from the keyboard", async () => {
  await renderApp("/tools/file-generator");

  await fireEvent.keyDown(window, { key: "k", ctrlKey: true });

  const dialog = screen.getByRole("dialog");
  const palette = within(dialog);
  await fireEvent.update(
    palette.getByRole("combobox", { name: "全局搜索工具" }),
    "spark",
  );
  expect(palette.getByText("Spark Analyzer")).toBeTruthy();
  expect(palette.queryByText("文件生成器")).toBeNull();
});

test("the command palette navigates even when recent-history persistence is unavailable", async () => {
  const { router } = await renderApp();

  await fireEvent.keyDown(window, { key: "k", ctrlKey: true });
  const palette = within(screen.getByRole("dialog"));
  await fireEvent.click(palette.getByText("文件生成器"));

  await waitFor(() =>
    expect(router.currentRoute.value.path).toBe("/tools/file-generator"),
  );
});

test("the command palette supports arrow-key selection and Enter navigation", async () => {
  const { router } = await renderApp();

  await fireEvent.keyDown(window, { key: "k", ctrlKey: true });
  const palette = within(screen.getByRole("dialog"));
  const search = palette.getByRole("combobox", { name: "全局搜索工具" });
  await fireEvent.keyDown(search, { key: "ArrowDown" });
  await fireEvent.keyDown(search, { key: "Enter" });

  await waitFor(() =>
    expect(router.currentRoute.value.path).toBe("/tools/spark-analyzer"),
  );
});

test("the command palette explains empty results and keeps the current route", async () => {
  const { router } = await renderApp();
  await fireEvent.keyDown(window, { key: "k", ctrlKey: true });
  const palette = within(screen.getByRole("dialog"));
  const search = palette.getByRole("combobox", { name: "全局搜索工具" });
  await fireEvent.update(search, "unknown-tool");
  expect(palette.getByRole("status").textContent).toContain("没有匹配的工具");
  await fireEvent.keyDown(search, { key: "Enter" });
  expect(router.currentRoute.value.path).toBe("/");
});
