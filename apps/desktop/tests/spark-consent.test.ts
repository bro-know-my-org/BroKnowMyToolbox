import type { SparkAnalyzerAdapter } from "@bro-know-my/spark-analyzer";
import { describe, expect, test, vi } from "vitest";

import { createConsentAwareSparkAdapter } from "../src/api/spark-consent";

describe("Spark consent adapter", () => {
  test("an operation waits for host consent and retries after it is allowed", async () => {
    const report = {
      reportId: "report-1",
      kind: "text" as const,
      source: "remote",
      summary: { title: "Report", findings: [] },
    };
    const fetchReport = vi
      .fn()
      .mockRejectedValueOnce(
        new Error(
          JSON.stringify({
            code: "consent_required",
            capability: "network:spark",
            message: "user consent is required",
          }),
        ),
      )
      .mockResolvedValueOnce(report);
    const base = { fetchReport } as unknown as SparkAnalyzerAdapter;
    const requestConsent = vi.fn().mockResolvedValue(true);
    const adapter = createConsentAwareSparkAdapter(base, requestConsent);

    await expect(adapter.fetchReport("spark-key")).resolves.toEqual(report);
    expect(requestConsent).toHaveBeenCalledWith(
      "network:spark",
      "consent_required",
    );
    expect(fetchReport).toHaveBeenCalledTimes(2);
  });

  test("a denied consent keeps the operation blocked", async () => {
    const denial = new Error(
      JSON.stringify({
        code: "consent_denied",
        capability: "credentials:ai",
        message: "user denied this capability",
      }),
    );
    const loadApiKey = vi.fn().mockRejectedValue(denial);
    const base = { loadApiKey } as unknown as SparkAnalyzerAdapter;
    const requestConsent = vi.fn().mockResolvedValue(false);
    const adapter = createConsentAwareSparkAdapter(base, requestConsent);

    await expect(adapter.loadApiKey()).rejects.toBe(denial);
    expect(requestConsent).toHaveBeenCalledWith(
      "credentials:ai",
      "consent_denied",
    );
    expect(loadApiKey).toHaveBeenCalledTimes(1);
  });

  test("an operation can request multiple capabilities in sequence", async () => {
    const analyze = vi
      .fn()
      .mockRejectedValueOnce(
        new Error(
          JSON.stringify({
            code: "consent_required",
            capability: "network:spark",
          }),
        ),
      )
      .mockRejectedValueOnce(
        new Error(
          JSON.stringify({
            code: "consent_required",
            capability: "credentials:ai",
          }),
        ),
      )
      .mockResolvedValueOnce("done");
    const requestConsent = vi.fn().mockResolvedValue(true);
    const adapter = createConsentAwareSparkAdapter(
      { analyze } as unknown as SparkAnalyzerAdapter,
      requestConsent,
    );

    await expect(
      (adapter as unknown as { analyze(): Promise<string> }).analyze(),
    ).resolves.toBe("done");
    expect(requestConsent).toHaveBeenNthCalledWith(
      1,
      "network:spark",
      "consent_required",
    );
    expect(requestConsent).toHaveBeenNthCalledWith(
      2,
      "credentials:ai",
      "consent_required",
    );
  });
});
