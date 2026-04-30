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
let renderTimer: ReturnType<typeof setInterval> | undefined;
let renderer: Awaited<ReturnType<typeof createRenderer>> | undefined;

process.on("SIGINT", stop);
process.on("SIGTERM", stop);

startMapper();
renderer = await createRenderer();
renderTimer = setInterval(() => {
  renderer?.update(lastLine, stateLine);
}, 250);

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
  return createRenderer();
}

async function createRenderer() {
  try {
    const { createCliRenderer, TextRenderable } = await import("@opentui/core");
    const ui = await createCliRenderer({
      screenMode: "alternate-screen",
      consoleMode: "disabled",
      externalOutputMode: "capture-stdout",
      clearOnShutdown: true,
      exitOnCtrlC: false,
      targetFps: 12,
      maxFps: 12,
    });

    const title = new TextRenderable(ui, {
      id: "title",
      content: "skate-kbm",
      position: "absolute",
      left: 2,
      top: 1,
      fg: "#7dd3fc",
    });
    const body = new TextRenderable(ui, {
      id: "body",
      content: screenContent("starting...", ""),
      position: "absolute",
      left: 2,
      top: 3,
      fg: "#e5e7eb",
    });

    ui.root.add(title);
    ui.root.add(body);
    ui.start();

    return {
      update(status: string, state: string) {
        body.content = screenContent(status, state);
        ui.requestRender();
      },
      destroy() {
        ui.destroy();
      },
    };
  } catch {
    process.stdout.write(screenContent(lastLine, stateLine));
    return {
      update() {},
      destroy() {},
    };
  }
}

function screenContent(status: string, state: string) {
  return `Virtual Xbox 360 mapper for keyboard and mouse.

Controls
  WASD                 left stick
  Mouse                right stick
  Shift / Space        A
  Esc                  B
  E                    X
  R                    Y
  Left click           RT
  Right click          LT

Status
  ${status}
  ${state || "state: waiting for input"}

Keep this running, then launch your game. Press Ctrl+C to stop.`;
}

function stop() {
  if (renderTimer) clearInterval(renderTimer);
  if (mapperProcess && !mapperProcess.killed) {
    mapperProcess.kill("SIGINT");
  }
  renderer?.destroy();
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
