import { readFile } from "node:fs/promises";
import process from "node:process";

const tagIndex = process.argv.indexOf("--tag");
const tag = tagIndex >= 0 ? process.argv[tagIndex + 1] : undefined;
if (!tag?.startsWith("v"))
  throw new Error("--tag must be a v-prefixed SemVer tag");
const expected = tag.slice(1);
const numericIdentifier = String.raw`(?:0|[1-9]\d*)`;
const prereleaseIdentifier = String.raw`(?:${numericIdentifier}|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)`;
const semver = new RegExp(
  String.raw`^${numericIdentifier}\.${numericIdentifier}\.${numericIdentifier}(?:-${prereleaseIdentifier}(?:\.${prereleaseIdentifier})*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$`,
  "u",
);
if (!semver.test(expected)) {
  throw new Error("--tag must contain a valid SemVer version");
}

const cargo = await readFile("Cargo.toml", "utf8");
const cargoVersion = cargo.match(
  /\[workspace\.package\][\s\S]*?\nversion = "([^"]+)"/,
)?.[1];
const rootPackage = JSON.parse(await readFile("package.json", "utf8"));
const desktopPackage = JSON.parse(
  await readFile("apps/desktop/package.json", "utf8"),
);
const toolContractPackage = JSON.parse(
  await readFile("packages/tool-contract/package.json", "utf8"),
);
const tauriConfig = JSON.parse(
  await readFile("apps/desktop/src-tauri/tauri.conf.json", "utf8"),
);
const versions = {
  "Cargo workspace": cargoVersion,
  "root package": rootPackage.version,
  "desktop package": desktopPackage.version,
  "tool contract package": toolContractPackage.version,
  "Tauri config": tauriConfig.version,
};

const mismatches = Object.entries(versions).filter(
  ([, version]) => version !== expected,
);
if (mismatches.length) {
  throw new Error(
    `tag ${tag} does not match:\n${mismatches
      .map(([source, version]) => `- ${source}: ${version ?? "missing"}`)
      .join("\n")}`,
  );
}

process.stdout.write(`workspace version ${expected} matches ${tag}\n`);
