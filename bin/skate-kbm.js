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
  let stopping = false;
  let exited = false;
  const child = spawn(mapper, args, {
    cwd: existsSync(packagedMapper) ? packagedRoot : root,
    stdio: ["ignore", "inherit", "inherit"],
    windowsHide: true,
  });

  const stop = () => {
    if (stopping) return;
    stopping = true;
    if (process.platform !== "win32") {
      child.kill("SIGINT");
    }
    setTimeout(() => {
      if (!exited) child.kill("SIGTERM");
    }, 1500).unref();
  };

  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);

  child.on("exit", (code, signal) => {
    exited = true;
    process.off("SIGINT", stop);
    process.off("SIGTERM", stop);

    if (stopping && (signal === "SIGINT" || signal === "SIGTERM")) {
      process.exitCode = 0;
      return;
    }

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
  --no-mouse-capture             Do not hide or recenter the Windows cursor
  -h, --help                     Show help

Keep this running, then launch your game. Press Ctrl+C to stop.`);
}
