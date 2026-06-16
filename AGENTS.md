# AGENTS.md

This file is for AI coding agents working in this repository. Read it before making changes. It captures project-specific constraints that are easy to miss from code alone.

## Project Summary

This is a Tauri 2 + Vue 3 + Naive UI desktop toolbox. The current product direction is an app with built-in, modular tools.

The main app owns:

- window shell
- layout
- header/sidebar
- router
- built-in tool registration
- native capabilities

Tools are source-code modules inside the app. Do not rebuild the previous hot-pluggable plugin platform unless the user explicitly asks for it.

## Important Project Decisions

- Runtime hot-plug plugins are no longer the current direction.
- Do not add plugin manifest loading, plugin install/uninstall, plugin permission grants, generated plugin bundles, or a WASM plugin runtime unless explicitly requested.
- Dynamic routing can stay, but the source is the built-in tool registry, not external manifest scanning.
- Tools should live under `src/views/tools/<tool-name>/index.vue`.
- Tool metadata should be registered in `src/tools/registry.ts`.
- System capabilities should usually go through explicit Rust commands rather than generic tool/plugin action dispatch.
- Tauri capability config is application-level and may remain while the app-level permission strategy is still being decided.

## Key Files

- `README.md`: user/developer-facing overview and usage.
- `TODO`: pending tasks only. Do not add completed work there.
- `src/tools/registry.ts`: built-in tool metadata, menu generation, and route registration helpers.
- `src/router/index.ts`: base router setup. Tool routes must stay mounted under `RootLayout`.
- `src/views/tools/`: built-in tool pages.
- `src-tauri/src/commands/`: explicit Rust commands exposed to the frontend.
- `src/stores/`: Pinia stores for app-wide state.

## Tool Registration Rules

Register built-in tools in `src/tools/registry.ts`:

```ts
{
  id: "file-creator",
  title: "文件创建器",
  path: "/tools/file-creator",
  routeName: "ToolFileCreator",
  component: () => import("../views/tools/file-creator/index.vue"),
}
```

Routes are added under `RootLayout`. Do not add tool routes as top-level routes if they should keep AppHeader and Sidebar visible.

## Native Capability Rules

- Prefer explicit Rust commands for file, path, process, network, and other high-impact system operations.
- Avoid generic action dispatch such as `execute_plugin_action` for built-in tools.
- The file creator uses `create_file(path, content)`.
- Frontend checks are for UX only. Real validation belongs in the backend command.

## State Management

Use Pinia for app-wide state. Do not add new module-level `ref` singleton stores for state that is shared across pages.

Current stores:

- `src/stores/debug.ts`: debug mode state, persisted to `localStorage` with key `bkm.debug.enabled`.
- `src/stores/settings.ts`: user settings and runtime CSS variable application.

Components should use `storeToRefs` when binding Pinia state with `v-model`.

Runtime-adjustable visual values should be CSS Variables, not SCSS variables. SCSS can stay for style organization, but values like title bar height should be applied through variables such as `--bkm-title-bar-height`.

## Commands

Frontend build:

```bash
pnpm run build
```

Rust check:

```bash
cd src-tauri
cargo check
```

Tauri development:

```bash
pnpm tauri dev
```

## Verification Notes

- Run `pnpm run build` after frontend/router/tool registry changes.
- Run `cargo check` after Rust type or command changes.
- PowerShell sandbox may block Vite/esbuild child processes with `spawn EPERM`; rerun with escalation if needed.

## Editing Rules For Agents

- Preserve user changes. The worktree may be dirty.
- Do not revert unrelated edits.
- Keep README, TODO, and this file aligned when changing project architecture.
- Prefer small, direct changes that preserve the built-in tool architecture.
