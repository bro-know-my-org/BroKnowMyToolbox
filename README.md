# BroKnowMyToolbox

Tauri 2 + Vue 3 + Naive UI desktop toolbox.

This repository is the forward-looking refactor line for the toolbox. The product direction is a built-in tools workbench:

- the app owns the window shell, layout, router, menu, settings, and native capabilities;
- tools are source modules under `src/views/tools/<tool-name>/index.vue`;
- tool metadata is registered in `src/tools/registry.ts`;
- high-impact system actions should use explicit Rust commands.

It intentionally does not rebuild the old runtime plugin platform. Do not add external plugin manifest loading, install/uninstall flows, permission grants, generated plugin bundles, or a WASM runtime unless that direction is explicitly reopened.

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
