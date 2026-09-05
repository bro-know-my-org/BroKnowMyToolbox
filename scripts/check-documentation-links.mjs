import { access, readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

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

function headingSlug(value) {
  return value
    .trim()
    .toLowerCase()
    .replace(/!?(?:\[([^\]]*)\])(?:\([^)]*\)|\[[^\]]*\])/gu, "$1")
    .replace(/<[^>]*>/g, "")
    .replace(/[*~]/g, "")
    .replace(/[^\p{Letter}\p{Number}\s_-]/gu, "")
    .replace(/\s+/g, "-");
}

function visibleMarkdownLines(markdown) {
  const links = [];
  const headings = [];
  let fence = null;
  let inComment = false;
  for (const original of markdown.split(/\r?\n/u)) {
    const fenceMatch = /^(?: {0,3})(`{3,}|~{3,})(.*)$/u.exec(original);
    if (fenceMatch) {
      const marker = fenceMatch[1][0];
      if (
        fence &&
        fence.marker === marker &&
        fenceMatch[1].length >= fence.length &&
        fenceMatch[2].trim() === ""
      ) {
        fence = null;
      } else if (!fence) {
        fence = { marker, length: fenceMatch[1].length };
      }
      links.push("");
      headings.push("");
      continue;
    }
    if (fence) {
      links.push("");
      headings.push("");
      continue;
    }
    let line = original;
    const codeSpans = [];
    line = line.replace(/(`+)(.*?)\1/gu, (_match, _ticks, content) => {
      const token = `BKMT_CODE_TOKEN_${codeSpans.length}_END`;
      codeSpans.push(content);
      return token;
    });
    if (inComment) {
      const end = line.indexOf("-->");
      if (end === -1) {
        links.push("");
        headings.push("");
        continue;
      }
      line = line.slice(end + 3);
      inComment = false;
    }
    while (line.includes("<!--")) {
      const start = line.indexOf("<!--");
      const end = line.indexOf("-->", start + 4);
      if (end === -1) {
        line = line.slice(0, start);
        inComment = true;
        break;
      }
      line = `${line.slice(0, start)}${line.slice(end + 3)}`;
    }
    const restoreCode = (value) =>
      value.replace(
        /BKMT_CODE_TOKEN_(\d+)_END/gu,
        (_match, index) => codeSpans[Number(index)] ?? "",
      );
    headings.push(restoreCode(line));
    links.push(line.replace(/BKMT_CODE_TOKEN_\d+_END/gu, ""));
  }
  return { links, headings };
}

function headingFragments(lines) {
  const fragments = new Set();
  const add = (heading) => {
    const base = headingSlug(heading);
    let slug = base;
    let suffix = 0;
    while (fragments.has(slug)) slug = `${base}-${++suffix}`;
    fragments.add(slug);
  };
  for (const [index, line] of lines.entries()) {
    const atx = /^(?: {0,3})#{1,6}\s+(.+?)\s*#*\s*$/u.exec(line);
    if (atx) {
      add(atx[1]);
      continue;
    }
    if (
      index > 0 &&
      /^ {0,3}(?:=+|-+)\s*$/u.test(line) &&
      lines[index - 1].trim()
    ) {
      add(lines[index - 1].trim());
    }
  }
  return fragments;
}

function destinationValue(raw) {
  const destination = raw.trim();
  if (destination.startsWith("<")) {
    const end = destination.indexOf(">");
    return end === -1 ? destination : destination.slice(1, end);
  }
  return destination.split(/\s+["']/u, 1)[0];
}

function markdownLinks(lines) {
  const references = new Map();
  for (const line of lines) {
    const definition = /^ {0,3}\[([^\]]+)\]:\s*(.+?)\s*$/u.exec(line);
    if (definition) {
      references.set(
        definition[1].trim().toLowerCase(),
        destinationValue(definition[2]),
      );
    }
  }
  const links = [];
  for (const [lineIndex, line] of lines.entries()) {
    if (/^ {0,3}\[[^\]]+\]:/u.test(line)) continue;
    const inline =
      /!?\[[^\]]*\]\((<[^>]*>|(?:[^()\s]|\([^()]*\))+)(?:\s+["'][^"']*["'])?\)/gu;
    for (const match of line.matchAll(inline)) {
      links.push({ line: lineIndex + 1, target: destinationValue(match[1]) });
    }
    const reference = /!?\[([^\]]+)\]\[([^\]]*)\]/gu;
    for (const match of line.matchAll(reference)) {
      const id = (match[2] || match[1]).trim().toLowerCase();
      if (references.has(id))
        links.push({ line: lineIndex + 1, target: references.get(id) });
    }
    const shortcut = /(^|[^!\\\]])\[([^\]]+)\](?![([])/gu;
    for (const match of line.matchAll(shortcut)) {
      const id = match[2].trim().toLowerCase();
      if (references.has(id))
        links.push({ line: lineIndex + 1, target: references.get(id) });
    }
  }
  return links;
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
  const fragmentCache = new Map();

  for (const source of await markdownFiles(root)) {
    const relativeSource = path.relative(root, source);
    const markdown = await readFile(source, "utf8");
    const lines = visibleMarkdownLines(markdown);
    for (const link of markdownLinks(lines.links)) {
      if (externalSchemes.test(link.target) || link.target.startsWith("//"))
        continue;
      const [encodedPath = "", encodedFragment] = link.target.split("#", 2);
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

      let fragments = fragmentCache.get(target);
      if (!fragments) {
        fragments = headingFragments(
          visibleMarkdownLines(await readFile(target, "utf8")).headings,
        );
        fragmentCache.set(target, fragments);
      }
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
