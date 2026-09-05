# Contributing

Read [AGENTS.md](./AGENTS.md), [CONTEXT.md](./CONTEXT.md), and the [tool development guide](./docs/tool-development.md) before changing architecture or adding a built-in tool.

## Development checks

```bash
pnpm install --frozen-lockfile
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run build
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Spark npm and Rust dependencies are pinned to the published `0.1.2` release. No sibling checkout or Spark UI build is required. Preserve the documented keyring features when updating these dependencies.

Tools are compile-time modules. Contributions must not add runtime plugin installation, generic native action dispatch, or external manifest scanning.

## Changes

- Keep behavior changes small and test through public interfaces.
- Preserve stable CLI JSON fields, error codes, and exit codes.
- Update the single source-of-truth document for any changed product or architecture decision.
- Do not claim macOS signing, notarization, Gatekeeper, or real-device verification without corresponding evidence.
