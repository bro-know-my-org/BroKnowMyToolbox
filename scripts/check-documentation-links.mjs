import { access, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { parseDocumentationMarkdown } from "./documentation-markdown.mjs";

const ignoredDirectories = new Set([".git", "dist", "node_modules", "target"]);
const externalSchemes = /^[a-z][a-z0-9+.-]*:/i;

async function markdownFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory() && ignoredDirectories.has(entry.name)) continue;
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await markdownFiles(entryPath)));
    if (entry.isFile() && entry.name.endsWith(".md")) files.push(entryPath);
  }
  return files;
}

async function exists(target) {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}

export async function checkDocumentationLinks(rootDirectory) {
  const root = path.resolve(rootDirectory);
  const errors = [];
  const parsedFiles = new Map();
  const parseFile = async (file) => {
    if (!parsedFiles.has(file))
      parsedFiles.set(
        file,
        parseDocumentationMarkdown(await readFile(file, "utf8")),
      );
    return parsedFiles.get(file);
  };

  for (const source of await markdownFiles(root)) {
    const relativeSource = path.relative(root, source);
    for (const link of (await parseFile(source)).links) {
      if (externalSchemes.test(link.target) || link.target.startsWith("//"))
        continue;
      const fragmentStart = link.target.indexOf("#");
      const encodedPath =
        fragmentStart === -1
          ? link.target
          : link.target.slice(0, fragmentStart);
      const encodedFragment =
        fragmentStart === -1 ? undefined : link.target.slice(fragmentStart + 1);
      const encodedTarget = encodedPath.split("?", 1)[0];
      let targetReference;
      try {
        targetReference = decodeURIComponent(encodedTarget);
      } catch {
        errors.push(
          `${relativeSource}:${link.line}: malformed target escape ${encodedTarget}`,
        );
        continue;
      }
      const target = targetReference
        ? path.resolve(path.dirname(source), targetReference)
        : source;
      const relativeTarget = path.relative(root, target);
      if (
        path.isAbsolute(targetReference) ||
        relativeTarget === ".." ||
        relativeTarget.startsWith(`..${path.sep}`)
      ) {
        errors.push(
          `${relativeSource}:${link.line}: target escapes repository ${targetReference}`,
        );
        continue;
      }
      if (!(await exists(target))) {
        errors.push(
          `${relativeSource}:${link.line}: missing target ${targetReference}`,
        );
        continue;
      }
      if (encodedFragment === undefined || encodedFragment === "") continue;
      if (path.extname(target).toLowerCase() !== ".md") continue;

      const { fragments } = await parseFile(target);
      let fragment;
      try {
        fragment = decodeURIComponent(encodedFragment);
      } catch {
        errors.push(
          `${relativeSource}:${link.line}: malformed fragment escape #${encodedFragment}`,
        );
        continue;
      }
      if (!fragments.has(fragment)) {
        errors.push(
          `${relativeSource}:${link.line}: missing fragment #${fragment} in ${path.relative(root, target)}`,
        );
      }
    }
  }
  return errors;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const errors = await checkDocumentationLinks(
    process.argv[2] ?? process.cwd(),
  );
  if (errors.length > 0) {
    for (const error of errors) process.stderr.write(`${error}\n`);
    process.exitCode = 1;
  } else {
    process.stdout.write("documentation links are valid\n");
  }
}
