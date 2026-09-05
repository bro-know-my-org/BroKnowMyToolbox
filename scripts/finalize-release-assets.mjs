import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  cp,
  mkdir,
  readFile,
  readdir,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const options = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, value, index, values) => {
    if (value.startsWith("--")) pairs.push([value.slice(2), values[index + 1]]);
    return pairs;
  }, []),
);
for (const required of [
  "input",
  "output",
  "version",
  "repository",
  "revision",
]) {
  if (!options[required]) throw new Error(`missing --${required}`);
}

const input = path.resolve(options.input);
const output = path.resolve(options.output);
await mkdir(output, { recursive: true });
const staged = [];
const expectedTargets = new Set([
  "linux:x86_64",
  "macos:aarch64",
  "macos:x86_64",
  "windows:x86_64",
]);
const seenTargets = new Set();

for (const name of await readdir(input)) {
  const root = path.join(input, name);
  const metadataPath = path.join(root, "metadata.json");
  let metadata;
  try {
    metadata = JSON.parse(await readFile(metadataPath, "utf8"));
  } catch {
    // Ignore upload-artifact bookkeeping directories.
    continue;
  }
  if (metadata.schemaVersion !== 1) {
    throw new Error(`unsupported staged metadata schema in ${metadataPath}`);
  }
  if (metadata.version !== options.version) {
    throw new Error(
      `staged version ${metadata.version} does not match ${options.version}`,
    );
  }
  const targetKey = `${metadata.platform}:${metadata.arch}`;
  if (!expectedTargets.has(targetKey)) {
    throw new Error(`unexpected staged release target ${targetKey}`);
  }
  if (seenTargets.has(targetKey)) {
    throw new Error(`duplicate staged release target ${targetKey}`);
  }
  seenTargets.add(targetKey);
  staged.push({ root, ...metadata });
}

const missingTargets = [...expectedTargets].filter(
  (target) => !seenTargets.has(target),
);
if (missingTargets.length > 0) {
  throw new Error(
    `missing staged release targets: ${missingTargets.join(", ")}`,
  );
}

for (const item of staged) {
  const baseName = `bkmt-${options.version}-${item.platform}-${item.arch}-portable`;
  if (item.platform === "windows") {
    execFileSync("zip", ["-qr", path.join(output, `${baseName}.zip`), "."], {
      cwd: path.join(item.root, "portable"),
    });
  } else {
    execFileSync("tar", [
      "-czf",
      path.join(output, `${baseName}.tar.gz`),
      "-C",
      path.join(item.root, "portable"),
      ".",
    ]);
  }
  const installerDirectory = path.join(item.root, "installers");
  for (const installer of await readdir(installerDirectory)) {
    await cp(
      path.join(installerDirectory, installer),
      path.join(output, installer),
    );
  }
}

const files = (await readdir(output)).sort();
const assets = [];
for (const name of files) {
  const file = path.join(output, name);
  if (!(await stat(file)).isFile()) continue;
  const bytes = await readFile(file);
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  assets.push({
    name,
    sha256,
    size: bytes.length,
    url: `https://github.com/${options.repository}/releases/download/v${options.version}/${encodeURIComponent(name)}`,
  });
}
await writeFile(
  path.join(output, "SHA256SUMS"),
  `${assets.map((asset) => `${asset.sha256}  ${asset.name}`).join("\n")}\n`,
);
await writeFile(
  path.join(output, "release-manifest.json"),
  `${JSON.stringify(
    {
      schemaVersion: 1,
      version: options.version,
      revision: options.revision,
      assets,
    },
    null,
    2,
  )}\n`,
);

function requiredAsset(fragment) {
  const asset = assets.find((candidate) => candidate.name.includes(fragment));
  if (!asset) throw new Error(`missing release asset containing ${fragment}`);
  return asset;
}

const windows = requiredAsset("windows-x86_64-portable.zip");
const linux = requiredAsset("linux-x86_64-portable.tar.gz");
const macIntel = requiredAsset("macos-x86_64-portable.tar.gz");
const macArm = requiredAsset("macos-aarch64-portable.tar.gz");
const channel = path.join(output, "channel");
await mkdir(path.join(channel, "scoop"), { recursive: true });
await mkdir(path.join(channel, "homebrew"), { recursive: true });
await mkdir(path.join(channel, "aur"), { recursive: true });
await writeFile(
  path.join(channel, "scoop", "bkmt.json"),
  `${JSON.stringify(
    {
      version: options.version,
      description: "Bro Know My Toolbox desktop app and CLI",
      homepage: `https://github.com/${options.repository}`,
      license: "Apache-2.0",
      url: windows.url,
      hash: windows.sha256,
      bin: ["bkmt.exe", "bro-know-my-toolbox.exe"],
      shortcuts: [["bro-know-my-toolbox.exe", "Bro Know My Toolbox"]],
      persist: "data",
    },
    null,
    2,
  )}\n`,
);
await writeFile(
  path.join(channel, "homebrew", "bkmt.rb"),
  `class Bkmt < Formula\n  desc "Bro Know My Toolbox desktop app and CLI"\n  homepage "https://github.com/${options.repository}"\n  version "${options.version}"\n  license "Apache-2.0"\n\n  on_macos do\n    if Hardware::CPU.arm?\n      url "${macArm.url}"\n      sha256 "${macArm.sha256}"\n    else\n      url "${macIntel.url}"\n      sha256 "${macIntel.sha256}"\n    end\n  end\n\n  def install\n    bin.install "bkmt"\n    prefix.install Dir["Bro Know My Toolbox.app"]\n  end\n\n  def caveats\n    "The desktop app is unsigned. Open it from #{prefix}/Bro Know My Toolbox.app after reviewing the checksum."\n  end\nend\n`,
);
const aurVersion = options.version.replaceAll("-", "_");
await writeFile(
  path.join(channel, "aur", "PKGBUILD"),
  `pkgname=bkmt-bin\npkgver=${aurVersion}\npkgrel=1\npkgdesc='Bro Know My Toolbox desktop app and CLI'\narch=('x86_64')\nurl='https://github.com/${options.repository}'\nlicense=('Apache-2.0')\ndepends=('webkit2gtk-4.1')\nsource=("${linux.url}")\nsha256sums=('${linux.sha256}')\n\npackage() {\n  install -Dm755 "$srcdir/bkmt" "$pkgdir/usr/bin/bkmt"\n  install -Dm755 "$srcdir/bro-know-my-toolbox" "$pkgdir/usr/bin/bro-know-my-toolbox"\n  install -Dm644 "$srcdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"\n}\n`,
);
