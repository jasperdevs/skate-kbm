<h1 align="center">🛹 skate-kbm</h1>

<p align="center">Keyboard and mouse to virtual Xbox 360 controller mapper for Windows games.</p>

This is CLI-only and currently Windows-only.

## Install

Install globally with Bun:

```powershell
bun add -g skate-kbm
skate-kbm --help
```

Requirements:

- Windows 10 or 11
- [Bun](https://bun.sh/)
- [ViGEmBus](https://github.com/nefarius/ViGEmBus/releases) driver

Windows needs a signed virtual controller driver before games can see a fake Xbox controller. If the app says it cannot connect, install the driver from the source repo with `bun run driver`.

## Use

```powershell
skate-kbm
```

Keep the terminal open, then launch your game. Press `Ctrl+C` to stop the mapper.

To change mouse sensitivity:

```powershell
skate-kbm --mouse-sensitivity 300
```

## Build from source

<details>
<summary>Developer setup</summary>

Developers also need [Rust](https://www.rust-lang.org/tools/install) to rebuild the native Windows mapper. OpenTUI is currently Bun-first, so Bun is the right runtime for the CLI shell.

```powershell
bun install
bun run build
bun run driver
bun start
```

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

- The app uses OpenTUI for the CLI shell and a Rust Windows mapper process for keyboard, mouse, and virtual controller output.
- The game still sees a controller, not native keyboard and mouse.
- ViGEmBus is required because Windows needs a driver to expose the virtual Xbox controller.
- ViGEmBus is archived, so it is treated as the current compatibility backend rather than a forever dependency.

</details>

## License

MIT
