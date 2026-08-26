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

async function collectWxmlFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const path = join(directory.pathname, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectWxmlFiles(new URL(`${entry.name}/`, directory)));
    } else if (extname(entry.name) === ".wxml") {
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

for (const wxmlPath of await collectWxmlFiles(outputDir)) {
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

console.log("All mini-program custom components are registered.");
