#!/usr/bin/env node
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packagedRoot = dirname(process.execPath);
const packagedMapper = resolve(packagedRoot, "mapper/skate-kbm-mapper.exe");
const devMapper = resolve(root, "dist/app/mapper/skate-kbm-mapper.exe");
const mapper = existsSync(packagedMapper) ? packagedMapper : devMapper;
const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  printHelp();
  process.exitCode = 0;
} else if (!existsSync(mapper)) {
  console.error("mapper binary missing. Run: npm run build");
  process.exitCode = 1;
} else {
  runMapper();
}

function runMapper() {
  const child = spawn(mapper, args, {
    cwd: existsSync(packagedMapper) ? packagedRoot : root,
    stdio: ["ignore", "inherit", "inherit"],
    windowsHide: true,
  });

  child.on("exit", (code) => {
    process.exitCode = code ?? 1;
  });

  child.on("error", (error) => {
    console.error(`failed to start mapper: ${error.message}`);
    process.exitCode = 1;
  });
}

function printHelp() {
  console.log(`skate-kbm

Usage:
  skate-kbm [options]
  node ./bin/skate-kbm.js [options]

Options:
  --mouse-sensitivity <number>   Right-stick mouse sensitivity. Default: 500
  --cursor-lock <mode>           Cursor lock mode: hold, always, off. Default: hold
  --no-mouse-capture             Alias for --cursor-lock off
  --debug                        Print live input state
  -h, --help                     Show help

Keep this running, then launch your game. Press Ctrl+C or Ctrl+Alt+Backspace to stop.`);
}
