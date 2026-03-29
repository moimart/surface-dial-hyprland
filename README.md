# Surface Dial Daemon for Linux

A lightweight daemon that maps the Microsoft Surface Dial to cycling modes on Hyprland (Wayland):

- **Volume** — adjust system volume via PipeWire/WirePlumber
- **Zoom** — Hyprland cursor zoom in/out
- **App Scroll** — scroll the focused application window
- **Scroll Layout** — navigate columns in Hyprland's scrolling layout
- **Home Assistant Media** — control a Home Assistant media player's volume via REST API

Click the dial to cycle between modes. A floating overlay indicator shows the active mode with per-mode accent colors and a volume bar for HA media.

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

### Managing the service

```sh
make enable     # enable and start
make disable    # stop and disable
make restart    # restart after config changes
make status     # check if running
make logs       # view live logs
```

### Uninstall

```sh
make uninstall  # stops service, removes binary and unit file (keeps config)
```

## Configuration

Config lives at `~/.config/surface-dial/config.toml`.

```toml
[general]
overlay_timeout_ms = 1500

# Mode cycle order — click the dial to rotate through these.
# Available modes: volume, zoom, appscroll, hyprscroll, hass_media
mode_order = ["volume", "zoom", "appscroll", "hyprscroll"]

[hyprscroll]
# 0 = column mode (focus r/l, smooth with Hyprland animations)
# >0 = pixel mode (move +/-N pixels per tick)
pixels_per_tick = 0

[volume]
step_percent = 2              # Volume change per dial tick

[appscroll]
speed_multiplier = 1          # Scroll notches per dial tick

[zoom]
step = 0.5                    # Zoom factor change per dial tick

[hass_media]
url = "http://homeassistant.local:8123"
token = "your-long-lived-access-token"
entity_id = "media_player.living_room"
volume_step = 0.02            # Volume change per tick (0.0 - 1.0)
```

### Modes

| Mode | What it does | Config section |
|------|-------------|----------------|
| **volume** | System volume via `wpctl` | `[volume]` |
| **zoom** | Hyprland `cursor:zoom_factor` | `[zoom]` |
| **appscroll** | Injects virtual scroll wheel events | `[appscroll]` |
| **hyprscroll** | Navigates Hyprland scrolling layout columns | `[hyprscroll]` |
| **hass_media** | Controls Home Assistant media player volume | `[hass_media]` |

Add or remove modes from `mode_order` to customize which modes are available and in what order. Modes not listed are skipped when cycling.

### Home Assistant Media setup

1. Create a [long-lived access token](https://www.home-assistant.io/docs/authentication/#your-account-profile) in your HA profile
2. Find your media player entity ID (e.g. `media_player.living_room`)
3. Add the `[hass_media]` section to your config and include `"hass_media"` in `mode_order`

The overlay shows a volume bar with percentage when adjusting HA media volume.

### Theme

The overlay is styled with GTK4 CSS. The default theme uses Catppuccin Mocha colors:

| Mode | Accent Color |
|------|-------------|
| Scroll Layout | Blue `#89b4fa` |
| Volume | Green `#a6e3a1` |
| App Scroll | Yellow `#f9e2af` |
| Zoom | Purple `#cba6f7` |
| HA Media | Pink `#f38ba8` |

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
