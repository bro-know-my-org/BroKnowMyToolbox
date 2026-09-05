# Bro Know My Toolbox

[简体中文](./README.md)

Bro Know My Toolbox is a cross-platform desktop toolbox with a first-class `bkmt` command-line application. It starts with a small set of built-in tools and keeps desktop and CLI behavior consistent through shared core modules.

> Status: version-one features and release automation are being finalized. A formal release remains blocked on the Spark Analyzer keyring fix passing three-platform CI and being published.

## Version-one scope

- File Generator: templates, variables, batch generation, execution previews, and safe overwrite rules.
- Spark Analyzer: reuse the separately released Spark Analyzer npm package and Rust crates.
- Build and startup smoke checks on Windows, Linux, and macOS.
- Installed and explicit portable modes.
- Simplified Chinese, English, and user language-pack overrides.

Clipboard management, runtime plugin installation, telemetry, and silent self-updates are outside version one.

## Design principles

- Tools are built-in modules compiled with the application, not runtime plugins.
- Business behavior is implemented once; desktop and CLI are adapters over shared cores.
- Native capabilities pass through explicit Rust commands, minimal Tauri capabilities, and persisted consent checks.
- System data directories are the default; the portable package writes to `data/` only when explicitly marked.
- Machine interfaces use stable error codes and fields; translation happens at presentation edges.

## Documentation

The internal design documents are maintained in Chinese. Start with [architecture](./docs/architecture.md), [implementation plan](./docs/plan.md), [release procedure](./docs/releasing.md), and [domain language](./CONTEXT.md).

## License

This project is licensed under the [Apache License 2.0](./LICENSE).
