<h1 align="center">🛹 skate-kbm</h1>

<p align="center">Keyboard and mouse to virtual Xbox 360 controller mapper for Windows games.</p>

This is CLI-only and not published yet. Use it from source until the first working package is confirmed.

## Requirements

- Windows 10 or 11
- [Bun](https://bun.sh/)
- [ViGEmBus](https://github.com/nefarius/ViGEmBus/releases) driver

Windows needs a signed virtual controller driver before normal games can see a fake Xbox controller. This project uses ViGEmBus for that part.

## Setup

```powershell
bun install
bun run build
bun run driver
```

Accept the Windows driver prompt if it appears.

## Use

```powershell
bun start
```

Keep the terminal open, then launch `skate.` from Steam. Press `Ctrl+C` to stop the mapper.

To change mouse sensitivity:

```powershell
bun start -- --mouse-sensitivity 300
```

## Install the driver

If the app cannot connect a virtual controller, install the driver:

```powershell
bun run driver
```

Accept the Windows driver prompt, then run `bun start` again.

## Build from source

<details>
<summary>Developer setup</summary>

Developers also need the [.NET 9 SDK](https://dotnet.microsoft.com/) to rebuild the Windows mapper. OpenTUI is currently Bun-first, so Bun is the right runtime for the CLI shell.

```powershell
bun install
bun run build
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

- The app uses OpenTUI for the CLI shell and a small Windows mapper process for keyboard, mouse, and virtual controller output.
- The game still sees a controller, not native keyboard and mouse.
- ViGEmBus is required because Windows needs a driver to expose the virtual Xbox controller.
- This repo is public, but no package or release is published until the first working test is confirmed.

</details>

## License

MIT
