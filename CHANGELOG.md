# Changelog

All notable changes to Bro Know My Toolbox are documented here. The project follows Semantic Versioning; desktop and `bkmt` share one workspace version.

## [Unreleased]

### Added

- Tauri 2 + Vue 3 desktop toolbox with built-in tool discovery, favorites, recent tools, command palette, themes, and language overrides.
- Shared Rust configuration, portable data-root resolution, product consent, and file generation modules.
- File Generator GUI and `bkmt file create` workflows with reviewable plans and capability-based writes.
- Embedded Spark Analyzer GUI and `bkmt spark` commands using the upstream Spark core and agent.
- Manual update checks, portable packages, checksums, release manifest, and generated Scoop/Homebrew/AUR metadata.

### Security

- Spark network, credential, and export operations are enforced by a host authorizer in Rust.
- Spark keyring platform backends and cross-process regression tests are prepared upstream; release remains blocked until the fixed upstream version passes the three-platform CI and is published.

[Unreleased]: https://github.com/bro-know-my-org/BroKnowMyToolbox/compare/v0.0.3...HEAD
