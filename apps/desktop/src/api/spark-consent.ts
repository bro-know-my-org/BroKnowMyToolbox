import type { SparkAnalyzerAdapter } from "@bro-know-my/spark-analyzer";

export type SparkConsentCapability =
  "credentials:ai" | "filesystem:write" | "network:spark";
export type SparkConsentCode = "consent_required" | "consent_denied";
export type RequestSparkConsent = (
  capability: SparkConsentCapability,
  code: SparkConsentCode,
) => Promise<boolean>;

type ConsentError = {
  code: SparkConsentCode;
  capability: SparkConsentCapability;
};

function parseConsentError(error: unknown): ConsentError | null {
  const raw = error instanceof Error ? error.message : String(error);
  try {
    const parsed = JSON.parse(raw) as Partial<ConsentError>;
    if (
      (parsed.code === "consent_required" ||
        parsed.code === "consent_denied") &&
      (parsed.capability === "credentials:ai" ||
        parsed.capability === "filesystem:write" ||
        parsed.capability === "network:spark")
    ) {
      return parsed as ConsentError;
    }
  } catch {
    return null;
  }
  return null;
}

export function createConsentAwareSparkAdapter(
  base: SparkAnalyzerAdapter,
  requestConsent: RequestSparkConsent,
): SparkAnalyzerAdapter {
  return new Proxy(base, {
    get(target, property, receiver) {
      const value = Reflect.get(target, property, receiver);
      if (typeof value !== "function") return value;
      return async (...args: unknown[]) => {
        const granted = new Set<SparkConsentCapability>();
        for (;;) {
          try {
            return await Reflect.apply(value, target, args);
          } catch (error) {
            const consent = parseConsentError(error);
            if (!consent || granted.has(consent.capability)) throw error;
            if (!(await requestConsent(consent.capability, consent.code)))
              throw error;
            granted.add(consent.capability);
          }
        }
      };
    },
  });
}
