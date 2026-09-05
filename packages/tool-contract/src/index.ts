import catalogJson from "../catalog.json";

export type ToolCapability =
  "credentials:ai" | "filesystem:read" | "filesystem:write" | "network:spark";

export interface ToolDefinition {
  readonly id: string;
  readonly titleKey: string;
  readonly descriptionKey: string;
  readonly route: string;
  readonly routeName: string;
  readonly cliNamespace: string;
  readonly keywords: readonly string[];
  readonly capabilities: readonly ToolCapability[];
}

export interface ToolCatalog {
  all(): readonly ToolDefinition[];
  get(id: string): ToolDefinition | undefined;
  search(query: string): readonly ToolDefinition[];
}

function normalize(value: string): string {
  return value.trim().toLowerCase();
}

function assertUnique(
  definitions: readonly ToolDefinition[],
  field: "cliNamespace" | "id" | "route" | "routeName",
): void {
  const seen = new Set<string>();
  for (const definition of definitions) {
    const value = definition[field];
    if (seen.has(value)) {
      throw new Error(`Duplicate tool ${field} "${value}"`);
    }
    seen.add(value);
  }
}

const VALID_CAPABILITIES = new Set<ToolCapability>([
  "credentials:ai",
  "filesystem:read",
  "filesystem:write",
  "network:spark",
]);

function assertValid(value: unknown): asserts value is ToolDefinition {
  const invalid = (field: string, value: unknown): never => {
    throw new Error(`Invalid tool ${field} ${JSON.stringify(value)}`);
  };
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    invalid("definition", value);
  }
  const definition = value as Record<string, unknown>;
  for (const field of [
    "id",
    "titleKey",
    "descriptionKey",
    "route",
    "routeName",
    "cliNamespace",
  ] as const) {
    if (typeof definition[field] !== "string")
      invalid(field, definition[field]);
  }
  if (
    !Array.isArray(definition.keywords) ||
    [...definition.keywords].some((entry) => typeof entry !== "string")
  ) {
    invalid("keywords", definition.keywords);
  }
  if (
    !Array.isArray(definition.capabilities) ||
    [...definition.capabilities].some((entry) => typeof entry !== "string")
  ) {
    invalid("capabilities", definition.capabilities);
  }
  const typed = definition as unknown as ToolDefinition;
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(typed.id)) {
    invalid("id", typed.id);
  }
  if (!/^\/tools\/[a-z0-9]+(?:-[a-z0-9]+)*$/.test(typed.route)) {
    invalid("route", typed.route);
  }
  if (!/^[A-Z][A-Za-z0-9]*$/.test(typed.routeName)) {
    invalid("routeName", typed.routeName);
  }
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(typed.cliNamespace)) {
    invalid("cliNamespace", typed.cliNamespace);
  }
  if (!typed.titleKey.trim() || !typed.descriptionKey.trim()) {
    invalid("translation key", "empty");
  }
  if (typed.keywords.some((value) => !value.trim())) {
    invalid("keywords", definition.keywords);
  }
  if (
    typed.capabilities.some((value) => !VALID_CAPABILITIES.has(value)) ||
    new Set(typed.capabilities).size !== typed.capabilities.length
  ) {
    invalid("capabilities", typed.capabilities);
  }
}

export function createToolCatalog(
  definitions: readonly ToolDefinition[],
): ToolCatalog {
  for (const definition of definitions) assertValid(definition);
  for (const field of ["id", "route", "routeName", "cliNamespace"] as const) {
    assertUnique(definitions, field);
  }
  const tools = Object.freeze(
    definitions.map((definition) =>
      Object.freeze({
        ...definition,
        keywords: Object.freeze([...definition.keywords]),
        capabilities: Object.freeze([...definition.capabilities]),
      }),
    ),
  );
  const byId = new Map(tools.map((tool) => [tool.id, tool]));

  return Object.freeze({
    all: () => tools,
    get: (id: string) => byId.get(id),
    search: (query: string) => {
      const term = normalize(query);
      if (!term) {
        return tools;
      }
      return tools.filter((tool) =>
        [tool.id, tool.cliNamespace, ...tool.keywords].some((value) =>
          normalize(value).includes(term),
        ),
      );
    },
  });
}

export const builtInToolCatalog = createToolCatalog(
  catalogJson as unknown as readonly ToolDefinition[],
);
