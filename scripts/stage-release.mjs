import { cp, mkdir, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const options = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, value, index, values) => {
    if (value.startsWith("--")) pairs.push([value.slice(2), values[index + 1]]);
    return pairs;
  }, []),
);
for (const required of ["target", "platform", "arch", "version", "output"]) {
  if (!options[required]) throw new Error(`missing --${required}`);
}

const workspace = process.cwd();
const releaseRoot = path.join(workspace, "target", options.target, "release");
const output = path.resolve(options.output);
const portable = path.join(output, "portable");
const installers = path.join(output, "installers");
await mkdir(portable, { recursive: true });
await mkdir(installers, { recursive: true });

const executableSuffix = options.platform === "windows" ? ".exe" : "";
await cp(
  path.join(releaseRoot, `bkmt${executableSuffix}`),
  path.join(portable, `bkmt${executableSuffix}`),
);

if (options.platform === "macos") {
  const appDirectory = path.join(releaseRoot, "bundle", "macos");
  const app = (await readdir(appDirectory)).find((entry) =>
    entry.endsWith(".app"),
  );
  if (!app) throw new Error(`no macOS app bundle found in ${appDirectory}`);
  await cp(path.join(appDirectory, app), path.join(portable, app), {
    recursive: true,
  });
} else {
  await cp(
    path.join(releaseRoot, `bro-know-my-toolbox${executableSuffix}`),
    path.join(portable, `bro-know-my-toolbox${executableSuffix}`),
  );
}

await cp(path.join(workspace, "LICENSE"), path.join(portable, "LICENSE"));
await writeFile(path.join(portable, "portable.bkmt"), "", "utf8");
await writeFile(
  path.join(portable, "README-portable.txt"),
  "This unsigned portable build stores ordinary data in the data/ directory beside the launcher or executable. Back up that directory before replacing the package.\n",
  "utf8",
);

let installerCount = 0;
const installerPatterns = {
  linux: /\.(?:AppImage|deb|rpm)$/i,
  macos: /\.dmg$/i,
  windows: /\.(?:exe|msi)$/i,
};
const installerPattern = installerPatterns[options.platform];
if (!installerPattern)
  throw new Error(`unsupported --platform ${options.platform}`);
async function collectBundleFiles(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const source = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (entry.name.endsWith(".app")) continue;
      await collectBundleFiles(source);
    } else if (installerPattern.test(entry.name)) {
      const destination = `${options.platform}-${options.arch}-${entry.name}`;
      await cp(source, path.join(installers, destination));
      installerCount += 1;
    }
  }
}

await collectBundleFiles(path.join(releaseRoot, "bundle"));
if (installerCount === 0) {
  throw new Error(`no installer bundles found for ${options.target}`);
}
await writeFile(
  path.join(output, "metadata.json"),
  `${JSON.stringify(
    {
      schemaVersion: 1,
      platform: options.platform,
      arch: options.arch,
      target: options.target,
      version: options.version,
    },
    null,
    2,
  )}\n`,
  "utf8",
);
