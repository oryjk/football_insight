import { readdir, readFile } from "node:fs/promises";
import { extname, join } from "node:path";

const outputDir = new URL("../dist/build/mp-weixin/", import.meta.url);

const builtInTags = new Set([
  "ad-custom",
  "camera",
  "channel-live",
  "channel-video",
  "checkbox-group",
  "cover-image",
  "cover-view",
  "keyboard-accessory",
  "live-player",
  "live-pusher",
  "match-media",
  "movable-area",
  "movable-view",
  "navigation-bar",
  "official-account",
  "open-data",
  "page-container",
  "page-meta",
  "picker-view",
  "picker-view-column",
  "radio-group",
  "rich-text",
  "root-portal",
  "scroll-view",
  "swiper-item",
  "web-view",
]);

async function collectFiles(directory, extension) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const path = join(directory.pathname, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectFiles(new URL(`${entry.name}/`, directory), extension));
    } else if (extname(entry.name) === extension) {
      files.push(path);
    }
  }

  return files;
}

async function readUsingComponents(wxmlPath) {
  const jsonPath = `${wxmlPath.slice(0, -5)}.json`;
  try {
    const json = JSON.parse(await readFile(jsonPath, "utf8"));
    return new Set(Object.keys(json.usingComponents || {}));
  } catch (error) {
    if (error?.code === "ENOENT") return new Set();
    throw error;
  }
}

const appJson = JSON.parse(await readFile(new URL("app.json", outputDir), "utf8"));
const globalComponents = new Set(Object.keys(appJson.usingComponents || {}));
const failures = [];

for (const wxmlPath of await collectFiles(outputDir, ".wxml")) {
  const wxml = await readFile(wxmlPath, "utf8");
  const tags = new Set([...wxml.matchAll(/<([a-z][a-z0-9-]*)\b/g)].map((match) => match[1]));
  const localComponents = await readUsingComponents(wxmlPath);

  for (const tag of tags) {
    if (!tag.includes("-") || builtInTags.has(tag)) continue;
    if (localComponents.has(tag) || globalComponents.has(tag)) continue;
    failures.push(`${wxmlPath}: <${tag}>`);
  }
}

if (failures.length > 0) {
  console.error("Unregistered mini-program components:\n");
  for (const failure of failures.sort()) console.error(`- ${failure}`);
  process.exit(1);
}

const unresolvedBabelHelpers = [];
const babelHelperPattern = /@babel\/runtime\/[^\s"'`)]+|regeneratorValues/g;
for (const jsPath of await collectFiles(outputDir, ".js")) {
  const code = await readFile(jsPath, "utf8");
  for (const match of code.matchAll(babelHelperPattern)) {
    unresolvedBabelHelpers.push(`${jsPath}: ${match[0]}`);
  }
}

if (unresolvedBabelHelpers.length > 0) {
  console.error("Unresolved @babel/runtime helper references in mp-weixin bundle:\n");
  for (const item of unresolvedBabelHelpers.sort()) console.error(`- ${item}`);
  process.exit(1);
}

console.log("All mini-program custom components are registered and no unresolved @babel/runtime references exist.");
