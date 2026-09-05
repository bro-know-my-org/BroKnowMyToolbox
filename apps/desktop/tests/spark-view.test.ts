import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/vue";
import { afterEach, expect, test, vi } from "vitest";
import { createPinia } from "pinia";

import { createAppI18n } from "../src/i18n";
import SparkAnalyzerToolView from "../src/views/tools/spark-analyzer/index.vue";

const bridge = vi.hoisted(() => ({
  fetchReport: vi.fn(),
  invoke: vi.fn(),
}));

type MockSparkAdapter = {
  fetchReport(input: string): Promise<unknown>;
};

vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke }));

vi.mock("@bro-know-my/spark-analyzer/tauri", () => ({
  createTauriSparkAnalyzerAdapter: () => ({
    fetchReport: bridge.fetchReport,
  }),
}));

vi.mock("@bro-know-my/spark-analyzer", async () => {
  const { defineComponent, h, ref } = await import("vue");
  return {
    SparkAnalyzerView: defineComponent({
      props: {
        adapter: {
          type: Object as () => MockSparkAdapter,
          required: true,
        },
        theme: { type: String, default: undefined },
        preferencesStore: { type: Object, default: undefined },
      },
      setup(props) {
        const status = ref("idle");
        async function run() {
          try {
            await props.adapter.fetchReport("spark-report");
            status.value = "loaded";
          } catch {
            status.value = "blocked";
          }
        }
        return () =>
          h(
            "div",
            { "data-testid": "hosted-spark", "data-theme": props.theme },
            [
              h("button", { onClick: run }, "触发 Spark 网络操作"),
              h("span", status.value),
            ],
          );
      },
    }),
  };
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

test("the embedded Spark page persists consent and retries the blocked IPC flow", async () => {
  bridge.fetchReport
    .mockRejectedValueOnce(
      new Error(
        JSON.stringify({
          code: "consent_required",
          capability: "network:spark",
          message: "user consent is required",
        }),
      ),
    )
    .mockResolvedValueOnce({ reportId: "report-1" });
  bridge.invoke.mockResolvedValue(undefined);

  render(SparkAnalyzerToolView, {
    global: { plugins: [createPinia(), createAppI18n()] },
  });
  expect(screen.getByTestId("hosted-spark").getAttribute("data-theme")).toBe(
    "light",
  );
  await fireEvent.click(
    screen.getByRole("button", { name: "触发 Spark 网络操作" }),
  );

  expect(await screen.findByText("Spark Analyzer 需要授权")).toBeTruthy();
  expect(screen.getByText(/获取远程 Spark 报告/)).toBeTruthy();
  await fireEvent.click(screen.getByRole("button", { name: "允许并继续" }));

  await waitFor(() => expect(bridge.fetchReport).toHaveBeenCalledTimes(2));
  expect(bridge.invoke).toHaveBeenCalledWith("set_spark_consent_command", {
    capability: "network:spark",
    allowed: true,
  });
  expect(await screen.findByText("loaded")).toBeTruthy();
});

test("a pending Spark consent decision cannot be submitted twice", async () => {
  bridge.fetchReport.mockRejectedValue(
    new Error(
      JSON.stringify({
        code: "consent_required",
        capability: "network:spark",
        message: "user consent is required",
      }),
    ),
  );
  let finishDecision: (() => void) | undefined;
  bridge.invoke.mockReturnValue(
    new Promise<void>((resolve) => {
      finishDecision = resolve;
    }),
  );
  render(SparkAnalyzerToolView, {
    global: { plugins: [createPinia(), createAppI18n()] },
  });
  await fireEvent.click(
    screen.getByRole("button", { name: "触发 Spark 网络操作" }),
  );
  const allow = await screen.findByRole("button", { name: "允许并继续" });
  await fireEvent.click(allow);
  await fireEvent.click(allow);

  expect(bridge.invoke).toHaveBeenCalledTimes(1);
  finishDecision?.();
  await waitFor(() => expect(bridge.fetchReport).toHaveBeenCalledTimes(2));
});
