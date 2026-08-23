# dsx

Minimal DualSense-to-XInput bridge for Windows. Reads a PS5 controller
(USB or Bluetooth) and mirrors it onto a virtual Xbox 360 pad, so games
that only accept XInput — such as Xbox app / Game Pass titles — can use it.

Built to be auditable: no config, no network, no updater, ~300 lines.
The only external component is the [ViGEmBus](https://github.com/nefarius/ViGEmBus/releases)
kernel driver (open source, Microsoft-signed), which provides the virtual pad.

## Requirements

- Windows 10/11
- [ViGEmBus driver](https://github.com/nefarius/ViGEmBus/releases) installed
- A DualSense or DualSense Edge connected via USB or Bluetooth

## Build

Native on Windows: `cargo build --release`

Cross-compile from WSL/Linux (requires `mingw-w64`):

```
cargo build --release --target x86_64-pc-windows-gnu
```

## Use

1. Connect the DualSense.
2. Run `dsx.exe`. It prints `bridging` when active.
3. Start the game. It sees an Xbox 360 controller.
4. Ctrl+C to stop.

## Mapping

Sticks, triggers and d-pad map 1:1. Cross/Circle/Square/Triangle map to
A/B/X/Y. Create maps to Back, Options to Start, PS to Guide. The touchpad,
mute button, rumble, adaptive triggers and lightbar are not bridged.
