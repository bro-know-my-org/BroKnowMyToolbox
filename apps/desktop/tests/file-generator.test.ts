import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/vue";
import { afterEach, expect, test, vi } from "vitest";

import FileGeneratorView from "../src/views/tools/file-generator/index.vue";
import { createAppI18n } from "../src/i18n";
import type { PlannedFile } from "../src/api/file-generator";

const api = vi.hoisted(() => ({
  planFileGeneration: vi.fn(),
  executeFileGeneration: vi.fn(),
  listFileTemplates: vi.fn(),
  saveUserTemplate: vi.fn(),
  setFileGenerationConsent: vi.fn(),
}));

vi.mock("../src/api/file-generator", () => api);

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

api.listFileTemplates.mockResolvedValue([]);

test("users can preview and execute a file generation plan", async () => {
  api.planFileGeneration.mockResolvedValue({
    status: "planned",
    files: [{ path: "README.md", action: "create" }],
  });
  api.executeFileGeneration.mockResolvedValue({
    status: "complete",
    files: [{ path: "README.md", outcome: "created" }],
  });

  render(FileGeneratorView, {
    global: { plugins: [createAppI18n()] },
  });

  await fireEvent.update(screen.getByLabelText("目标目录"), "/tmp/demo");
  await fireEvent.update(screen.getByLabelText("name"), "Demo");
  await fireEvent.click(screen.getByRole("button", { name: "预览生成计划" }));

  await waitFor(() => expect(api.planFileGeneration).toHaveBeenCalledOnce());
  expect(screen.getByText("README.md")).toBeTruthy();
  expect(screen.getByText("创建")).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "执行计划" }));

  await waitFor(() => expect(api.executeFileGeneration).toHaveBeenCalledOnce());
  expect(screen.getByText("已创建")).toBeTruthy();
  expect(screen.getByText("执行完成")).toBeTruthy();
});

test("editing inputs invalidates the preview instead of executing a different request", async () => {
  api.planFileGeneration.mockResolvedValue({
    status: "planned",
    files: [{ path: "README.md", action: "create" }],
  });
  render(FileGeneratorView, {
    global: { plugins: [createAppI18n()] },
  });

  await fireEvent.update(screen.getByLabelText("目标目录"), "/tmp/demo");
  await fireEvent.update(screen.getByLabelText("name"), "Before");
  await fireEvent.click(screen.getByRole("button", { name: "预览生成计划" }));
  await screen.findByRole("button", { name: "执行计划" });

  await fireEvent.update(screen.getByLabelText("name"), "After");

  expect(screen.queryByRole("button", { name: "执行计划" })).toBeNull();
  expect(api.executeFileGeneration).not.toHaveBeenCalled();
});

test("a changed filesystem invalidates the rejected plan and requires a fresh preview", async () => {
  const originalFiles: PlannedFile[] = [
    { path: "README.md", action: "create", targetRevision: null },
  ];
  const refreshedFiles: PlannedFile[] = [
    { path: "README.md", action: "overwrite", targetRevision: "new-revision" },
  ];
  api.planFileGeneration
    .mockResolvedValueOnce({ status: "planned", files: originalFiles })
    .mockResolvedValueOnce({ status: "planned", files: refreshedFiles });
  api.executeFileGeneration
    .mockRejectedValueOnce({
      code: "plan_changed",
      message: "Preview the changed files again",
    })
    .mockResolvedValueOnce({
      status: "complete",
      files: [{ path: "README.md", outcome: "created" }],
    });
  render(FileGeneratorView, { global: { plugins: [createAppI18n()] } });
  await fireEvent.update(screen.getByLabelText("目标目录"), "/tmp/demo");
  await fireEvent.update(screen.getByLabelText("name"), "Demo");
  await fireEvent.click(screen.getByRole("button", { name: "预览生成计划" }));
  await fireEvent.click(
    await screen.findByRole("button", { name: "执行计划" }),
  );

  await screen.findByText("Preview the changed files again");
  expect(screen.queryByRole("button", { name: "执行计划" })).toBeNull();
  expect(screen.queryByText("README.md")).toBeNull();
  expect(api.executeFileGeneration).toHaveBeenCalledTimes(1);

  await fireEvent.click(screen.getByRole("button", { name: "预览生成计划" }));
  await fireEvent.click(
    await screen.findByRole("button", { name: "执行计划" }),
  );
  await waitFor(() =>
    expect(api.executeFileGeneration).toHaveBeenCalledTimes(2),
  );
  expect(api.executeFileGeneration.mock.calls[1][1]).toEqual(refreshedFiles);
  expect(await screen.findByText("执行完成")).toBeTruthy();
});

test("users can explicitly deny a filesystem write consent request", async () => {
  api.planFileGeneration.mockResolvedValue({
    status: "planned",
    files: [{ path: "README.md", action: "create" }],
  });
  api.executeFileGeneration.mockRejectedValue({
    code: "consent_required",
    message: "consent required",
  });
  api.setFileGenerationConsent.mockResolvedValue(undefined);

  render(FileGeneratorView, {
    global: { plugins: [createAppI18n()] },
  });
  await fireEvent.update(screen.getByLabelText("目标目录"), "/tmp/demo");
  await fireEvent.update(screen.getByLabelText("name"), "Demo");
  await fireEvent.click(screen.getByRole("button", { name: "预览生成计划" }));
  await screen.findByText("README.md");
  await fireEvent.click(screen.getByRole("button", { name: "执行计划" }));

  expect(await screen.findByText("需要文件写入授权")).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "拒绝" }));

  await waitFor(() =>
    expect(api.setFileGenerationConsent).toHaveBeenCalledWith(false),
  );
  expect(screen.queryByText("需要文件写入授权")).toBeNull();
});

test("users can persist the edited JSON as a user template", async () => {
  api.saveUserTemplate.mockResolvedValue({
    id: "readme",
    title: "README",
    source: "user",
    templateJson: "{}",
  });

  render(FileGeneratorView, {
    global: { plugins: [createAppI18n()] },
  });
  await fireEvent.click(screen.getByText("编辑模板 JSON"));
  await fireEvent.click(screen.getByRole("button", { name: "保存为用户模板" }));

  await waitFor(() => expect(api.saveUserTemplate).toHaveBeenCalledOnce());
});

test("saving a user template requires consent and retries the save after approval", async () => {
  api.saveUserTemplate
    .mockRejectedValueOnce({
      code: "consent_required",
      message: "consent required",
    })
    .mockResolvedValueOnce({
      id: "readme",
      title: "README",
      source: "user",
      templateJson: "{}",
    });
  api.setFileGenerationConsent.mockResolvedValue(undefined);

  render(FileGeneratorView, {
    global: { plugins: [createAppI18n()] },
  });
  await fireEvent.click(screen.getByText("编辑模板 JSON"));
  await fireEvent.click(screen.getByRole("button", { name: "保存为用户模板" }));
  await screen.findByText("需要文件写入授权");
  await fireEvent.click(screen.getByRole("button", { name: "允许并继续" }));

  await waitFor(() => expect(api.saveUserTemplate).toHaveBeenCalledTimes(2));
  expect(api.setFileGenerationConsent).toHaveBeenCalledWith(true);
  expect(api.executeFileGeneration).not.toHaveBeenCalled();
});
