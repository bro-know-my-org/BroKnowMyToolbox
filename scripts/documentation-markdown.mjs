import GithubSlugger from "github-slugger";
import { fromMarkdown } from "mdast-util-from-markdown";

function walk(node, visit) {
  visit(node);
  for (const child of node.children ?? []) walk(child, visit);
}

function headingText(node) {
  if (node.type === "html") return "";
  if (node.type === "image" || node.type === "imageReference")
    return node.alt ?? "";
  if (typeof node.value === "string") return node.value;
  return (node.children ?? []).map(headingText).join("");
}

export function parseDocumentationMarkdown(markdown) {
  const tree = fromMarkdown(markdown);
  const definitions = new Map();
  walk(tree, (node) => {
    if (node.type === "definition" && !definitions.has(node.identifier))
      definitions.set(node.identifier, node.url);
  });
  const links = [];
  const fragments = new Set();
  const slugger = new GithubSlugger();
  walk(tree, (node) => {
    if (node.type === "heading") fragments.add(slugger.slug(headingText(node)));
    let target;
    if (node.type === "link" || node.type === "image") target = node.url;
    if (node.type === "linkReference" || node.type === "imageReference")
      target = definitions.get(node.identifier);
    if (target !== undefined)
      links.push({ line: node.position.start.line, target });
  });
  return { links, fragments };
}
