import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { access, mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import test from "node:test";

test("release finalizer creates portable archives, checksums, and channel metadata", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-release-test-"));
  const input = path.join(root, "input");
  const output = path.join(root, "output");
  for (const [platform, arch] of [
    ["windows", "x86_64"],
    ["linux", "x86_64"],
    ["macos", "x86_64"],
    ["macos", "aarch64"],
  ]) {
    const staged = path.join(input, `${platform}-${arch}`);
    await mkdir(path.join(staged, "portable"), { recursive: true });
    await mkdir(path.join(staged, "installers"), { recursive: true });
    await writeFile(path.join(staged, "portable", "bkmt"), "binary");
    await writeFile(
      path.join(staged, "installers", `${platform}-${arch}-installer.bin`),
      "bundle",
    );
    await writeFile(
      path.join(staged, "metadata.json"),
      JSON.stringify({ schemaVersion: 1, platform, arch, version: "0.1.0" }),
    );
  }

  execFileSync(
    process.execPath,
    [
      "scripts/finalize-release-assets.mjs",
      "--input",
      input,
      "--output",
      output,
      "--version",
      "0.1.0",
      "--repository",
      "bro-know-my-org/BroKnowMyToolbox",
      "--revision",
      "0123456789abcdef",
    ],
    { cwd: path.resolve(import.meta.dirname, "..") },
  );

  const manifest = JSON.parse(
    await readFile(path.join(output, "release-manifest.json"), "utf8"),
  );
  assert.equal(manifest.version, "0.1.0");
  assert.equal(
    manifest.assets.filter((asset) => asset.name.includes("portable")).length,
    4,
  );
  assert.match(
    await readFile(path.join(output, "SHA256SUMS"), "utf8"),
    /windows-x86_64/,
  );
  assert.match(
    await readFile(path.join(output, "channel", "scoop", "bkmt.json"), "utf8"),
    /Apache-2.0/,
  );
  assert.match(
    await readFile(path.join(output, "channel", "homebrew", "bkmt.rb"), "utf8"),
    /on_macos/,
  );
  assert.match(
    await readFile(path.join(output, "channel", "aur", "PKGBUILD"), "utf8"),
    /pkgname=bkmt-bin/,
  );
});

test("release staging adds the portable marker and keeps installer names target-specific", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-stage-test-"));
  const release = path.join(
    root,
    "target",
    "x86_64-pc-windows-msvc",
    "release",
  );
  await mkdir(path.join(release, "bundle", "msi"), { recursive: true });
  await writeFile(path.join(root, "LICENSE"), "Apache-2.0");
  await writeFile(path.join(release, "bkmt.exe"), "cli");
  await writeFile(path.join(release, "bro-know-my-toolbox.exe"), "desktop");
  await writeFile(
    path.join(release, "bundle", "msi", "Toolbox.msi"),
    "installer",
  );

  execFileSync(
    process.execPath,
    [
      path.resolve(import.meta.dirname, "stage-release.mjs"),
      "--target",
      "x86_64-pc-windows-msvc",
      "--platform",
      "windows",
      "--arch",
      "x86_64",
      "--version",
      "0.1.0",
      "--output",
      "release-stage",
    ],
    { cwd: root },
  );

  await access(path.join(root, "release-stage", "portable", "portable.bkmt"));
  await access(path.join(root, "release-stage", "portable", "bkmt.exe"));
  await access(
    path.join(
      root,
      "release-stage",
      "installers",
      "windows-x86_64-Toolbox.msi",
    ),
  );
});

test("release staging rejects a build that produced no installer", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-empty-stage-test-"));
  const release = path.join(
    root,
    "target",
    "x86_64-unknown-linux-gnu",
    "release",
  );
  await mkdir(path.join(release, "bundle"), { recursive: true });
  await writeFile(path.join(root, "LICENSE"), "Apache-2.0");
  await writeFile(path.join(release, "bkmt"), "cli");
  await writeFile(path.join(release, "bro-know-my-toolbox"), "desktop");

  const result = spawnSync(
    process.execPath,
    [
      path.resolve(import.meta.dirname, "stage-release.mjs"),
      "--target",
      "x86_64-unknown-linux-gnu",
      "--platform",
      "linux",
      "--arch",
      "x86_64",
      "--version",
      "0.1.0",
      "--output",
      "release-stage",
    ],
    { cwd: root, encoding: "utf8" },
  );

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /no installer bundles found/);
});

test("release finalization rejects stale staged metadata from another version", async () => {
  const root = await mkdtemp(
    path.join(os.tmpdir(), "bkmt-stale-release-test-"),
  );
  const input = path.join(root, "input");
  for (const [platform, arch] of [
    ["windows", "x86_64"],
    ["linux", "x86_64"],
    ["macos", "x86_64"],
    ["macos", "aarch64"],
  ]) {
    const staged = path.join(input, `${platform}-${arch}`);
    await mkdir(path.join(staged, "portable"), { recursive: true });
    await mkdir(path.join(staged, "installers"), { recursive: true });
    await writeFile(path.join(staged, "portable", "bkmt"), "binary");
    await writeFile(
      path.join(staged, "installers", `${platform}-${arch}.bin`),
      "bundle",
    );
    await writeFile(
      path.join(staged, "metadata.json"),
      JSON.stringify({
        schemaVersion: 1,
        platform,
        arch,
        version: "0.0.9",
      }),
    );
  }

  const result = spawnSync(
    process.execPath,
    [
      "scripts/finalize-release-assets.mjs",
      "--input",
      input,
      "--output",
      path.join(root, "output"),
      "--version",
      "0.1.0",
      "--repository",
      "bro-know-my-org/BroKnowMyToolbox",
      "--revision",
      "0123456789abcdef",
    ],
    { cwd: path.resolve(import.meta.dirname, ".."), encoding: "utf8" },
  );

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /staged version 0\.0\.9 does not match 0\.1\.0/);
});
