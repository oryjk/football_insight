#!/usr/bin/env node
import { spawn } from "node:child_process";
import path from "node:path";
import process from "node:process";

const command = process.argv[2];
const extraArgs = process.argv.slice(3);
const cliPath = "/Users/carlwang/.local/share/mini-program-ci-cli/index.mjs";
const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);

if (!command) {
  console.error("用法: bun run mp:preview [-- --robot 2 --desc 文案]");
  console.error("   或: bun run mp:upload [-- --robot 2 --version 1.0.1 --desc 文案]");
  process.exit(1);
}

const child = spawn("node", [cliPath, command, projectRoot, ...extraArgs], {
  stdio: "inherit",
});

child.on("exit", (code) => process.exit(code ?? 1));
child.on("error", (error) => {
  console.error(error);
  process.exit(1);
});
