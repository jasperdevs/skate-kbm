<h1 align="center">🛹 skate-kbm</h1>

<p align="center">Keyboard and mouse to virtual Xbox 360 controller mapper for Windows games.</p>

This is a plain terminal CLI. It does not use a full-screen TUI or any non-Node JavaScript runtime.

## Install

Install globally with npm:

```powershell
npm install -g skate-kbm
skate-kbm
```

Install adds the `skate-kbm` command and the native mapper binary.

Requirements:

- Windows 10 or 11
- [Node.js](https://nodejs.org/) 20 or newer
- [ViGEmBus](https://github.com/nefarius/ViGEmBus/releases) driver

Windows needs a signed virtual controller driver before games can see a fake Xbox controller. If the app says it cannot connect, install the driver from the source repo with `npm run driver`, or install ViGEmBus manually from its release page.

## Use

```powershell
skate-kbm
```

Keep the terminal open, then launch your game. Press `Ctrl+C` in the terminal or `Ctrl+Alt+Backspace` anywhere to stop the mapper.

The command prints normal terminal output only. It should not switch screens, clear your terminal, or close the terminal window.

To change mouse sensitivity:

```powershell
skate-kbm --mouse-sensitivity 300
```

By default, the mapper uses Windows Raw Input for mouse movement. The visible cursor is only hidden/centered while you hold a mouse button, so you can release the button to use the desktop normally.

Cursor lock modes:

```powershell
skate-kbm --cursor-lock hold
skate-kbm --cursor-lock always
skate-kbm --cursor-lock off
```

If raw input is unavailable, it falls back to cursor capture. To test without hiding or recentering the cursor:

```powershell
skate-kbm --no-mouse-capture
```

To print live input diagnostics:

```powershell
skate-kbm --debug
```

## Build from source

<details>
<summary>Developer setup</summary>

Developers also need [Rust](https://www.rust-lang.org/tools/install) to rebuild the native Windows mapper.

```powershell
npm install
npm run build
npm run driver
npm start
```

`npm run build` compiles the Rust mapper and copies it to `dist/app/mapper/skate-kbm-mapper.exe`, which is the binary shipped in the npm package.

</details>

## Controls

| Input | Controller output |
| --- | --- |
| `WASD` | Left stick |
| Mouse | Right stick |
| `Shift` or `Space` | `A` |
| `Esc` | `B` |
| `E` | `X` |
| `R` | `Y` |
| `Q` | Left bumper |
| `F` | Right bumper |
| Left click | Right trigger |
| Right click | Left trigger |
| Arrow keys | D-pad |
| `Tab` | Back |
| `Enter` | Start |

<details>
<summary>Notes</summary>

- The npm package has no JavaScript runtime dependencies.
- The app uses a plain Node.js CLI wrapper and a Rust Windows mapper process for keyboard, mouse, and virtual controller output.
- The game still sees a controller, not native keyboard and mouse.
- ViGEmBus is required because Windows needs a driver to expose the virtual Xbox controller.
- ViGEmBus is archived, so it is treated as the current compatibility backend rather than a forever dependency.

</details>

## License

MIT
