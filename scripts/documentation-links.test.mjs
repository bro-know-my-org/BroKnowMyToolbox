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

test("escaped labels and destinations follow CommonMark rather than link-shaped text", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(path.join(root, "a(b).md"), "# Guide\n");
  await writeFile(path.join(root, "a&b.md"), "# Guide\n");
  await writeFile(
    path.join(root, "README.md"),
    [
      String.raw`\[not a link](missing.md)`,
      String.raw`[escaped](a\(b\).md)`,
      String.raw`[reference][id]`,
      String.raw`\[literal][id]`,
      "[entity](a&amp;b.md)",
      "",
      String.raw`[id]: a\(b\).md`,
    ].join("\n"),
  );
  assert.deepEqual(await checkDocumentationLinks(root), []);
});

test("parenthesized titles and first reference definitions preserve the actual target", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(path.join(root, "guide.md"), "# Guide\n");
  await writeFile(
    path.join(root, "README.md"),
    [
      "[valid](guide.md (a title)) [missing](missing.md (a title))",
      "[ref] [multi   word]",
      "",
      "[ref]: guide.md (reference title)",
      "[ref]: missing-duplicate.md",
      "[multi word]:",
      "  guide.md",
    ].join("\n"),
  );
  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:1: missing target missing.md",
  ]);
});

test("thematic breaks do not manufacture heading fragments", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    [
      "# Heading",
      "---",
      "",
      "_Real_ heading",
      "===",
      "",
      "[valid](#heading) [setext](#real-heading) [phantom](#heading-1)",
      "[extra](#heading#extra)",
    ].join("\n"),
  );
  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:7: missing fragment #heading-1 in README.md",
    "README.md:8: missing fragment #heading#extra in README.md",
  ]);
});

test("multiline code spans, indented code and nested fences never create links", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    [
      "`code begins",
      "[ignored](missing-span.md)",
      "code ends`",
      "",
      "    [ignored](missing-indent.md)",
      "",
      "> ```md",
      "> [ignored](missing-fence.md)",
      "> ```",
      "",
      "[real](missing-real.md)",
    ].join("\n"),
  );
  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:11: missing target missing-real.md",
  ]);
});

test("multiline links retain their opening source line and nested headings use visible text", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "bkmt-doc-links-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(
    path.join(root, "README.md"),
    [
      "> # _Emphasis_ and `code` &amp; text",
      "",
      "[valid](#emphasis-and-code--text)",
      "[multiline",
      "label](missing.md)",
    ].join("\n"),
  );
  assert.deepEqual(await checkDocumentationLinks(root), [
    "README.md:4: missing target missing.md",
  ]);
});
