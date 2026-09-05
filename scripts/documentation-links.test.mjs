import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { checkDocumentationLinks } from "./check-documentation-links.mjs";

test("documentation link checker accepts files, directories, and heading fragments", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(path.join(root, "docs"));
  await writeFile(
    path.join(root, "README.md"),
    "[guide](./docs/guide.md#首版完成条件) [docs](./docs/)\n",
  );
  await writeFile(
    path.join(root, "docs", "guide.md"),
    "# Guide\n\n## 首版完成条件\n",
  );

  assert.deepEqual(await checkDocumentationLinks(root), []);
});

test("documentation link checker reports missing targets and fragments", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    "[missing](./missing.md) [heading](#not-here)\n",
  );

  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:1: missing target ./missing.md",
    "README.md:1: missing fragment #not-here in README.md",
  ]);
});

test("documentation link checker understands Markdown syntax and repository boundaries", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(path.join(root, "docs"));
  await writeFile(
    path.join(root, "README.md"),
    "`[ignored](missing.md)`\n```md\n[ignored](missing.md)\n```\n[guide][g]\n[balanced](./docs/a(b).md)\n[escape](../outside.md)\n[bad](bad%.md)\n\n[g]: ./docs/guide.md#setext-heading\n",
  );
  await writeFile(
    path.join(root, "docs", "guide.md"),
    "Setext heading\n--------------\n",
  );
  await writeFile(path.join(root, "docs", "a(b).md"), "# Balanced\n");

  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:7: target escapes repository ../outside.md",
    "README.md:8: malformed target escape bad%.md",
  ]);
});

test("documentation link checker keeps code and CommonMark reference syntax isolated", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    "````md\n```md\n[ignored](missing.md)\n```\n````\n`<!--` [real](./guide.md#run-cargo-test)\n[shortcut]\n\n[shortcut]: ./missing.md\n",
  );
  await writeFile(path.join(root, "guide.md"), "## Run `cargo test`\n");

  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:7: missing target ./missing.md",
  ]);
});

test("documentation link checker models occupied heading slugs and fragment case", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    "# foo\n# foo-1\n# foo\n[valid](#foo-2) [wrong-case](#Foo)\n",
  );

  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:4: missing fragment #Foo in README.md",
  ]);
});
