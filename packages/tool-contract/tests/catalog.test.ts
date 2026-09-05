import { describe, expect, test } from "vitest";

import { createToolCatalog, type ToolDefinition } from "../src/index";

const definitions: readonly ToolDefinition[] = [
  {
    id: "file-generator",
    titleKey: "tool.file_generator.name",
    descriptionKey: "tool.file_generator.description",
    route: "/tools/file-generator",
    routeName: "ToolFileGenerator",
    cliNamespace: "file",
    keywords: ["file", "template", "文件", "模板"],
    capabilities: ["filesystem:write"],
  },
  {
    id: "spark-analyzer",
    titleKey: "tool.spark_analyzer.name",
    descriptionKey: "tool.spark_analyzer.description",
    route: "/tools/spark-analyzer",
    routeName: "ToolSparkAnalyzer",
    cliNamespace: "spark",
    keywords: ["spark", "profile", "Minecraft"],
    capabilities: ["filesystem:read", "network:spark"],
  },
];

describe("tool catalog", () => {
  test("callers can discover tools by id, namespace, and keywords", () => {
    const catalog = createToolCatalog(definitions);

    expect(catalog.search("template").map((tool) => tool.id)).toEqual([
      "file-generator",
    ]);
    expect(catalog.search("SPARK").map((tool) => tool.id)).toEqual([
      "spark-analyzer",
    ]);
    expect(catalog.search("file-generator").map((tool) => tool.id)).toEqual([
      "file-generator",
    ]);
  });

  test("duplicate tool ids are rejected at catalog creation", () => {
    expect(() =>
      createToolCatalog([
        ...definitions,
        {
          ...definitions[0],
          route: "/tools/another-file-generator",
          routeName: "ToolAnotherFileGenerator",
          cliNamespace: "another-file",
        },
      ]),
    ).toThrowError('Duplicate tool id "file-generator"');
  });

  test.each([
    ["route", { route: "/tools/file-generator" }],
    ["routeName", { routeName: "ToolFileGenerator" }],
    ["cliNamespace", { cliNamespace: "file" }],
  ] as const)("duplicate tool %s values are rejected", (field, duplicate) => {
    const [duplicateValue] = Object.values(duplicate);
    expect(() =>
      createToolCatalog([
        ...definitions,
        {
          ...definitions[1],
          id: "duplicate-metadata",
          route: "/tools/duplicate-metadata",
          routeName: "ToolDuplicateMetadata",
          cliNamespace: "duplicate-metadata",
          ...duplicate,
        },
      ]),
    ).toThrowError(`Duplicate tool ${field} "${duplicateValue}"`);
  });

  test.each([
    ["id", { id: "" }],
    ["id", { id: "../escape" }],
    ["route", { route: "tools/file-generator" }],
    ["route", { route: "/settings" }],
    ["routeName", { routeName: "tool file" }],
    ["cliNamespace", { cliNamespace: "FILE" }],
    ["capabilities", { capabilities: ["unknown:capability"] }],
  ] as const)("invalid tool %s values are rejected", (_field, invalid) => {
    expect(() =>
      createToolCatalog([
        {
          ...definitions[0],
          ...invalid,
        } as ToolDefinition,
      ]),
    ).toThrowError(/Invalid tool/);
  });
});
