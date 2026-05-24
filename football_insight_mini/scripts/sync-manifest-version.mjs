#!/usr/bin/env node
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import JSON5 from "json5";

const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);
const envPath = path.join(projectRoot, ".env.ci.local");
const manifestPath = path.join(projectRoot, "src", "manifest.json");
const generatedVersionPath = path.join(projectRoot, "src", "config", "generatedMiniProgramVersion.ts");

function parseEnvFile(filePath) {
  if (!existsSync(filePath)) return {};

  const content = readFileSync(filePath, "utf8");
  const env = {};
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;
    const index = line.indexOf("=");
    if (index < 0) continue;
    const key = line.slice(0, index).trim();
    const value = line.slice(index + 1).trim();
    env[key] = value;
  }
  return env;
}

function parseVersionCode(versionName) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(String(versionName || "").trim());
  if (!match) {
    throw new Error(`MINI_PROGRAM_VERSION 格式必须是 x.y.z，实际收到: ${versionName}`);
  }
  const [, major, minor, patch] = match;
  return String(Number(major) * 10000 + Number(minor) * 100 + Number(patch));
}

export function syncManifestVersion(options = {}) {
  const env = parseEnvFile(envPath);
  const versionName = options.versionName || process.env.MINI_PROGRAM_VERSION || env.MINI_PROGRAM_VERSION;
  if (!versionName) {
    return null;
  }

  const manifest = JSON5.parse(readFileSync(manifestPath, "utf8"));
  const versionCode = parseVersionCode(versionName);

  const manifestChanged = !(manifest.versionName === versionName && String(manifest.versionCode) === versionCode);
  if (manifestChanged) {
    manifest.versionName = versionName;
    manifest.versionCode = versionCode;
    writeFileSync(`${manifestPath}`, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  }

  writeFileSync(
    generatedVersionPath,
    `export const MINI_PROGRAM_VERSION = ${JSON.stringify(versionName)};\nexport const MINI_PROGRAM_VERSION_CODE = ${JSON.stringify(versionCode)};\n`,
    "utf8",
  );

  return { versionName, versionCode, changed: manifestChanged };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const result = syncManifestVersion();
  if (result) {
    console.log(
      `[sync-manifest-version] ${result.changed ? "updated" : "kept"} manifest versionName=${result.versionName} versionCode=${result.versionCode}`,
    );
  }
}
