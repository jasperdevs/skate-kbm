# skate-kbm

Keyboard and mouse to virtual Xbox 360 controller mapper for `skate.` on Windows.

This is CLI-only and not published yet. The intended user build is a single `skate-kbm.exe` folder with the mapper included.

## Requirements

- Windows 10 or 11
- [ViGEmBus](https://github.com/nefarius/ViGEmBus/releases) driver

Windows needs a signed virtual controller driver before any app can create an Xbox controller. This project uses ViGEmBus for that part.

## Use

After a build or downloaded zip, run:

```powershell
.\skate-kbm.exe
```

Keep the terminal open, then launch `skate.` from Steam. Press `Ctrl+C` to stop the mapper.

To change mouse sensitivity:

```powershell
.\skate-kbm.exe --mouse-sensitivity 300
```

## Install the driver

If the app cannot connect a virtual controller, install ViGEmBus:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-vigembus.ps1
```

Accept the Windows driver prompt, then run `skate-kbm.exe` again.

## Build from source

Developers need Bun and the .NET SDK:

```powershell
npm install
npm run build
npm run package
```

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

## Notes

- The app uses OpenTUI for the CLI shell and a small Windows mapper process for keyboard, mouse, and virtual controller output.
- The game still sees a controller, not native keyboard and mouse.
- ViGEmBus is required because Windows needs a driver to expose the virtual Xbox controller.
- This repo is public, but no package or release is published until the first working test is confirmed.

## License

MIT
