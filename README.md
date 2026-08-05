# BroKnowMyToolbox

Tauri 2 + Vue 3 + Naive UI desktop toolbox.

This repository is the forward-looking refactor line for the toolbox. The product direction is a built-in tools workbench:

- the app owns the window shell, layout, router, menu, settings, and native capabilities;
- tools are source modules under `src/views/tools/<tool-name>/index.vue`;
- tool metadata is registered in `src/tools/registry.ts`;
- high-impact system actions should use explicit Rust commands.

It intentionally does not rebuild the old runtime plugin platform. Do not add external plugin manifest loading, install/uninstall flows, permission grants, generated plugin bundles, or a WASM runtime unless that direction is explicitly reopened.

## Spark Analyzer Integration

Spark Analyzer is a built-in Toolbox page, but its implementation is shared with the standalone BroKnowMySparkAnalyzer application:

```text
@bro-know-my/spark-analyzer   Vue UI and Tauri adapter
bkmsa-tauri                   native Tauri plugin
bkmsa-core / bkmsa-agent      parsing, tools, diagnostics and AI agent
```

Toolbox registers `bkmsa_tauri::init()` and grants `bkmsa-tauri:default`. The frontend creates its adapter through `@bro-know-my/spark-analyzer/tauri`. Do not recreate report parsing, AI transport, or analyzer state under `src-tauri/src/commands`; changes to analysis semantics belong in the BroKnowMySparkAnalyzer Rust crates.

The npm UI and Rust plugin use matching release versions. For example:

```json
"@bro-know-my/spark-analyzer": "^0.1.0"
```

```toml
bkmsa-tauri = "0.1.0"
```

Version `0.1.0` is published to both npm and crates.io, so a normal dependency install resolves the shared frontend adapter and native plugin without repository-local links.

## UI Direction

The initial shell borrows the dense dark workbench style from `BroKnowMySparkAnalyzer`:

- compact custom title bar;
- dark workspace background;
- left navigation;
- reusable tool page and panel components in `src/components/app`;
- built-in tool cards on the home page.

## Common Commands

```bash
pnpm install
pnpm run build
pnpm tauri dev
```

Rust check:

```bash
cd src-tauri
cargo check
```

When changing the embedded analyzer integration, run both the frontend build and Rust check. The standalone Analyzer repository owns the shared SDK release pipeline.

## Adding A Built-In Tool

1. Create `src/views/tools/<tool-name>/index.vue`.
2. Register metadata in `src/tools/registry.ts`.
3. Prefer shared shell components such as `ToolPage` and `ToolPanel`.
4. Add explicit Rust commands for file, path, process, network, or other high-impact native operations.

## Internationalization

The app uses `vue-i18n` with centralized locale files:

```text
src/i18n/locales/zh-CN.ts
src/i18n/locales/en-US.ts
```

User-visible text should use translation keys instead of hard-coded strings. Tool metadata should use `titleKey` and `descriptionKey` in `src/tools/registry.ts`.

Suggested key shape:

```text
app.*
nav.*
common.*
settings.*
tools.<toolId>.*
```

Language switching is hot and persisted through `src/stores/settings.ts`.
