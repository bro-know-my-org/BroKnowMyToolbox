import { builtInToolCatalog, type ToolDefinition } from "@bkmt/tool-contract";
import type { Component } from "vue";

export interface RegisteredTool extends ToolDefinition {
  readonly component: () => Promise<Component>;
}

const components: Readonly<Record<string, () => Promise<Component>>> = {
  "file-generator": () =>
    import("../views/tools/file-generator/index.vue").then(
      (module) => module.default,
    ),
  "spark-analyzer": () =>
    import("../views/tools/spark-analyzer/index.vue").then(
      (module) => module.default,
    ),
};

for (const id of Object.keys(components)) {
  if (!builtInToolCatalog.get(id)) {
    throw new Error(`Desktop adapter registered unknown tool "${id}"`);
  }
}

export const registeredTools: readonly RegisteredTool[] = Object.freeze(
  builtInToolCatalog.all().map((tool) => {
    const component = components[tool.id];
    if (!component) {
      throw new Error(`Desktop adapter is missing tool "${tool.id}"`);
    }
    return Object.freeze({ ...tool, component });
  }),
);

export function searchRegisteredTools(
  query: string,
  translate: (key: string) => string,
): readonly RegisteredTool[] {
  const term = query.trim().toLowerCase();
  if (!term) return registeredTools;
  return registeredTools.filter((tool) =>
    [
      tool.id,
      tool.cliNamespace,
      translate(tool.titleKey),
      translate(tool.descriptionKey),
      ...tool.keywords,
    ].some((value) => value.toLowerCase().includes(term)),
  );
}

export const toolCatalog = builtInToolCatalog;
