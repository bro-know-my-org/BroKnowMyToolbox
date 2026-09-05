# Distribution channels

Release assets and channel metadata are generated from one versioned build rather than edited by hand.

`scripts/stage-release.mjs` collects the desktop binary/app bundle, `bkmt`, `LICENSE`, and `portable.bkmt` for each CI target. `scripts/finalize-release-assets.mjs` then creates:

- portable archives for Windows x64, Linux x64, macOS x64, and macOS arm64;
- `SHA256SUMS` and `release-manifest.json`;
- `channel/scoop/bkmt.json`;
- `channel/homebrew/bkmt.rb` for a Homebrew Tap;
- `channel/aur/PKGBUILD`.

The Release workflow uploads these files to a draft GitHub Release. Publishing remains a deliberate maintainer action after checking the unsigned artifacts and the macOS limitations documented in [testing-release.md](../docs/testing-release.md).

Channel repositories should copy the generated file from the published release. They must not substitute a version or checksum manually.
