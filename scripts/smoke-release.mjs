import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import {
  access,
  cp,
  mkdir,
  mkdtemp,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { clearTimeout, setTimeout } from "node:timers";

const options = Object.fromEntries(
  process.argv.slice(2).reduce((pairs, value, index, values) => {
    if (value.startsWith("--")) pairs.push([value.slice(2), values[index + 1]]);
    return pairs;
  }, []),
);
for (const required of ["root", "platform", "version"]) {
  if (!options[required]) throw new Error(`missing --${required}`);
}
if (!new Set(["linux", "macos", "windows"]).has(options.platform)) {
  throw new Error(`unsupported --platform ${options.platform}`);
}

const stageRoot = path.resolve(options.root);
const portableRoot = path.join(stageRoot, "portable");
const executableSuffix = options.platform === "windows" ? ".exe" : "";
const cli = path.join(portableRoot, `bkmt${executableSuffix}`);
await access(cli);

function runCli(executable, args) {
  return execFileSync(executable, args, {
    encoding: "utf8",
    timeout: 15_000,
    windowsHide: true,
  }).trim();
}

assert.equal(runCli(cli, ["--version"]), `bkmt ${options.version}`);
const tools = JSON.parse(runCli(cli, ["tools", "--json"]));
assert.deepEqual(tools.map((tool) => tool.id).sort(), [
  "file-generator",
  "spark-analyzer",
]);

const temporaryRoot = await mkdtemp(
  path.join(os.tmpdir(), "bkmt-release-smoke-"),
);
try {
  const dryRun = JSON.parse(
    runCli(cli, [
      "file",
      "create",
      "--template",
      "@basic-readme",
      "--destination",
      path.join(temporaryRoot, "generated"),
      "--var",
      "name=Release smoke",
      "--dry-run",
      "--json",
    ]),
  );
  assert.equal(dryRun.status, "planned");
  assert.equal(dryRun.files[0]?.path, "README.md");

  const portableProbe = path.join(temporaryRoot, "portable-probe");
  const probeCli = path.join(portableProbe, path.basename(cli));
  await mkdir(portableProbe, { recursive: true });
  await cp(cli, probeCli);
  await writeFile(path.join(portableProbe, "portable.bkmt"), "", "utf8");
  const diagnostics = JSON.parse(
    runCli(probeCli, ["diagnostics", "data-dir", "--json"]),
  );
  assert.equal(diagnostics.source, "portable");
  assert.equal(
    path.resolve(diagnostics.path),
    path.resolve(portableProbe, "data"),
  );

  let desktop;
  if (options.platform === "macos") {
    const app = (await readdir(portableRoot)).find((entry) =>
      entry.endsWith(".app"),
    );
    assert.ok(app, "portable stage must contain a macOS app bundle");
    const macosDirectory = path.join(portableRoot, app, "Contents", "MacOS");
    const candidates = await readdir(macosDirectory, { withFileTypes: true });
    const executable = candidates.find((entry) => entry.isFile());
    assert.ok(executable, "macOS app bundle must contain an executable");
    desktop = path.join(macosDirectory, executable.name);
  } else {
    desktop = path.join(portableRoot, `bro-know-my-toolbox${executableSuffix}`);
  }
  await access(desktop);

  const direct = options["direct-desktop"] === "true";
  const command =
    options.platform === "linux" && !direct ? "xvfb-run" : desktop;
  const args = command === desktop ? [] : ["-a", desktop];
  const child = spawn(command, args, {
    detached: options.platform !== "windows",
    env: {
      ...process.env,
      BKMT_DATA_DIR: path.join(temporaryRoot, "desktop-data"),
    },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  let stderr = "";
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => {
    stderr = `${stderr}${chunk}`.slice(-8_000);
  });

  await new Promise((resolve, reject) => {
    let survivedStartup = false;
    let forceTimer;
    const stop = (signal) => {
      try {
        if (options.platform === "windows") child.kill(signal);
        else process.kill(-child.pid, signal);
      } catch {
        // The exit handler reports early termination and resolves expected shutdowns.
      }
    };
    const startupTimer = setTimeout(() => {
      survivedStartup = true;
      stop("SIGTERM");
      forceTimer = setTimeout(() => stop("SIGKILL"), 2_000);
    }, 5_000);
    child.once("error", (error) => {
      clearTimeout(startupTimer);
      if (forceTimer) clearTimeout(forceTimer);
      reject(error);
    });
    child.once("exit", (code, signal) => {
      clearTimeout(startupTimer);
      if (forceTimer) clearTimeout(forceTimer);
      if (survivedStartup) resolve();
      else {
        reject(
          new Error(
            `desktop exited during startup (code=${code}, signal=${signal})\n${stderr}`,
          ),
        );
      }
    });
  });
} finally {
  await rm(temporaryRoot, { recursive: true, force: true });
}

process.stdout.write(
  `release smoke passed for ${options.platform} ${options.version}\n`,
);
