#!/usr/bin/env bun
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");
const packagedRoot = dirname(process.execPath);
const packagedMapper = resolve(packagedRoot, "mapper/skate-kbm-mapper.exe");
const devMapper = resolve(root, "dist/app/mapper/skate-kbm-mapper.exe");
const mapper = existsSync(packagedMapper) ? packagedMapper : devMapper;
const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  printHelp();
  process.exit(0);
}

if (!existsSync(mapper)) {
  console.error("mapper binary missing. Run: npm run build");
  process.exit(1);
}

let mapperProcess: ChildProcessWithoutNullStreams | undefined;
let lastLine = "starting...";
let stateLine = "";

process.on("SIGINT", stop);
process.on("SIGTERM", stop);

await renderIntro();
startMapper();

setInterval(() => {
  renderFallback();
}, 500);

function startMapper() {
  mapperProcess = spawn(mapper, args, {
    cwd: existsSync(packagedMapper) ? packagedRoot : root,
    windowsHide: true,
  });

  mapperProcess.stdout.on("data", (chunk) => {
    for (const line of chunk.toString().split(/\r?\n/).filter(Boolean)) {
      if (line.startsWith("state:")) stateLine = line;
      else lastLine = line;
    }
  });

  mapperProcess.stderr.on("data", (chunk) => {
    lastLine = `error: ${chunk.toString().trim()}`;
  });

  mapperProcess.on("exit", (code) => {
    lastLine = `mapper exited with code ${code ?? "unknown"}`;
    mapperProcess = undefined;
  });
}

async function renderIntro() {
  try {
    const { createCliRenderer, TextRenderable } = await import("@opentui/core");
    const renderer = await createCliRenderer();
    renderer.root.add(
      new TextRenderable(renderer, {
        id: "title",
        content: "skate-kbm",
        position: "absolute",
        left: 2,
        top: 1,
        fg: "#7dd3fc",
      }),
    );
    renderer.root.add(
      new TextRenderable(renderer, {
        id: "hint",
        content: "Virtual Xbox 360 mapper for keyboard and mouse. Ctrl+C stops.",
        position: "absolute",
        left: 2,
        top: 3,
        fg: "#e5e7eb",
      }),
    );
    setTimeout(() => renderer.destroy(), 900);
  } catch {
    renderFallback();
  }
}

function renderFallback() {
  process.stdout.write("\x1b[2J\x1b[H");
  console.log("skate-kbm");
  console.log("Virtual Xbox 360 mapper for keyboard and mouse.");
  console.log("");
  console.log("Controls");
  console.log("  WASD                 left stick");
  console.log("  Mouse                right stick");
  console.log("  Shift / Space        A");
  console.log("  Esc                  B");
  console.log("  E                    X");
  console.log("  R                    Y");
  console.log("  Left click           RT");
  console.log("  Right click          LT");
  console.log("");
  console.log(lastLine);
  if (stateLine) console.log(stateLine);
  console.log("");
  console.log("Keep this running, then launch skate. from Steam. Press Ctrl+C to stop.");
}

function stop() {
  if (mapperProcess && !mapperProcess.killed) {
    mapperProcess.kill("SIGINT");
  }
  process.stdout.write("\x1b[?25h");
  process.exit(0);
}

function printHelp() {
  console.log(`skate-kbm

Usage:
  skate-kbm [options]
  bun ./src/cli/index.ts [options]

Options:
  --mouse-sensitivity <number>   Right-stick mouse sensitivity. Default: 220
  -h, --help                     Show help
`);
}
