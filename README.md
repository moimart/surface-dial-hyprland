# Surface Dial Daemon for Linux

A lightweight daemon that maps the Microsoft Surface Dial to three cycling modes on Hyprland (Wayland):

- **Scroll Layout** — navigate columns in Hyprland's scrolling layout
- **Volume** — adjust system volume via PipeWire/WirePlumber
- **App Scroll** — scroll the focused application window

Click the dial to cycle between modes. A floating overlay indicator shows the active mode.

## Requirements

- Linux with Wayland + Hyprland
- Bluetooth (for Surface Dial pairing)
- PipeWire + WirePlumber (`wpctl`)
- Rust toolchain
- System libraries: `gtk4`, `gtk4-layer-shell`, `libevdev`, `libudev`

### Arch / CachyOS

```sh
sudo pacman -S gtk4 gtk4-layer-shell libevdev
```

### Fedora

```sh
sudo dnf install gtk4-devel gtk4-layer-shell-devel libevdev-devel systemd-devel
```

## Setup

### 1. Install udev rules

Grants your user access to the Surface Dial input device and `/dev/uinput` (for virtual scroll injection):

```sh
sudo cp udev/10-surface-dial.rules /etc/udev/rules.d/
sudo udevadm control --reload
```

### 2. Pair the Surface Dial

```sh
bluetoothctl
> agent on
> default-agent
> scan on
# Hold the dial's button until it shows up
> pair <MAC>
> connect <MAC>
> trust <MAC>
> exit
```

### 3. Install

```sh
make install    # builds release binary, installs to ~/.local/bin, copies default config
make enable     # enables and starts the systemd user service
```

This installs:
- `~/.local/bin/surface-dial-daemon` — the binary
- `~/.config/systemd/user/surface-dial.service` — systemd unit
- `~/.config/surface-dial/config.toml` — config (won't overwrite existing)
- `~/.config/surface-dial/theme.css` — theme (won't overwrite existing)

To run manually instead:

```sh
cargo run --release
# or with debug logging:
RUST_LOG=debug cargo run --release
```

### Uninstall

```sh
make uninstall  # stops service, removes binary and unit file (keeps config)
```

### config.toml

```toml
[general]
overlay_timeout_ms = 1500     # How long the mode indicator stays visible

[hyprscroll]
# 0 = column mode (focus r/l, smooth with Hyprland animations)
# >0 = pixel mode (move +/-N pixels per tick)
pixels_per_tick = 0

[volume]
step_percent = 2              # Volume change per dial tick

[appscroll]
speed_multiplier = 1          # Scroll notches per dial tick
```

### theme.css

The overlay is styled with GTK4 CSS. The default theme uses Catppuccin Mocha colors with per-mode accents:

| Mode | Accent Color |
|------|-------------|
| Scroll Layout | Blue `#89b4fa` |
| Volume | Green `#a6e3a1` |
| App Scroll | Yellow `#f9e2af` |

Override by editing `~/.config/surface-dial/theme.css`. See `theme.css` in this repo for the full default.

## Hyprland setup

The Scroll Layout mode works with Hyprland's built-in scrolling layout. Enable it per-workspace in `hyprland.conf`:

```
workspace = 2, layout:scrolling

scrolling {
    column_width = 0.33
}
```

## Architecture

```
surface-dial-daemon
├── dial.rs          evdev reader thread (udev discovery, REL_DIAL + BTN_0)
├── main.rs          GTK4 app, glib event loop, accumulator-based rotation
├── mode.rs          Mode manager (click cycles modes)
├── modes/
│   ├── hyprscroll   layoutmsg move +col/-col via hyprctl
│   ├── volume       wpctl set-volume
│   ├── appscroll    uinput virtual scroll device
│   ├── zoom         cursor:zoom_factor via hyprctl
│   └── hass_media   Home Assistant media_player volume via REST API
├── overlay.rs       GTK4 + wlr-layer-shell floating indicator
├── hypr_ipc.rs      hyprctl subprocess wrapper
└── config.rs        TOML config with defaults
```

The daemon reads Surface Dial events (`REL_DIAL` for rotation, `BTN_0` for click) via evdev in a background thread. Events are sent to the GTK4 glib main loop where rotation is accumulated over 120ms windows to filter jitter, then dispatched to the active mode. A 350ms per-mode throttle on the scroll layout ensures smooth animated transitions.

## License

[MIT](LICENSE)
