# Surface Dial Daemon — Product Requirements

## Goal

A daemon for Linux that uses the Microsoft Surface Dial as a modal input device for Hyprland (Wayland). The dial cycles through user-defined modes via clicks, and rotation drives the active mode's action. A floating overlay indicator shows the current mode.

## Reference implementations

- https://github.com/daniel5151/surface-dial-linux
- https://www.reddit.com/r/SurfaceLinux/comments/eqk22k/surface_dial_on_linux/

## Modes

The dial supports five modes, all configurable and reorderable via `mode_order` in the config:

1. **Volume** — System volume via PipeWire/WirePlumber (`wpctl set-volume`)
2. **Zoom** — Hyprland cursor zoom (`hyprctl keyword cursor:zoom_factor`)
3. **App Scroll** — Virtual scroll wheel events injected via uinput, scrolls the focused window
4. **Hyprland Scroll Layout** — Navigate columns in Hyprland's built-in scrolling layout (`layoutmsg move +col/-col`)
5. **Home Assistant Media** — Control a remote media player's volume via the Home Assistant REST API

Clicking the dial cycles to the next mode. The mode list is configurable — users can reorder, omit, or include any subset.

## Visual feedback

A floating overlay window shows the active mode:

- **Centered** on screen via wlr-layer-shell (overlay layer, no input/keyboard interaction)
- **Per-mode accent colors** (Catppuccin Mocha palette by default)
- **Mode-specific content**:
  - Most modes show an icon + name
  - HA Media additionally renders a volume bar with percentage
  - Connection status (connected/disconnected) shows briefly when the dial pairs/unpairs
- **Auto-hides** after a configurable timeout (default 1.5s)
- **Themeable** via external GTK4 CSS file (`~/.config/surface-dial/theme.css`)

## Configuration

TOML config at `~/.config/surface-dial/config.toml`:

- `general.overlay_timeout_ms` — how long the overlay stays visible
- `general.mode_order` — list of modes to cycle through (and their order)
- Per-mode tuning (volume step, zoom step, scroll speed, HA URL/token/entity, etc.)

CSS theme at `~/.config/surface-dial/theme.css` overrides the default styling.

## Installation

- `make install` builds the release binary, copies it to `~/.local/bin`, installs the systemd user unit, and seeds default config files
- `make enable` / `disable` / `restart` / `status` / `logs` manage the systemd user service
- udev rule grants the user access to the Surface Dial input device and `/dev/uinput`
- `make uninstall` removes the binary and unit (preserves user config)

## Technical requirements

- **Wayland + Hyprland** as the target compositor
- **Bluetooth** for Surface Dial pairing (standard `bluetoothctl` flow)
- **Performant and portable** — minimal system dependencies, single static binary
- **Daemon** — runs in the background as a systemd user service

## Implementation

- **Language**: Rust
- **Input**: `evdev` crate reads `REL_DIAL` (rotation) and `BTN_0` (click) from the Surface Dial
- **Hotplug**: `udev` crate discovers the device and watches for connect/disconnect
- **Overlay**: GTK4 + `gtk4-layer-shell` (wlr-layer-shell protocol)
- **Hyprland IPC**: `hyprctl` subprocess for layout and zoom commands
- **Volume**: `wpctl` subprocess for PipeWire control
- **App scroll**: Virtual `uinput` device emitting `REL_WHEEL` / `REL_WHEEL_HI_RES`
- **HA Media**: Non-blocking HTTP via `ureq` on background threads, with cached volume state

### Rotation handling

The Surface Dial sends `REL_DIAL` events with variable magnitude (proportional to rotation speed) and significant jitter (occasional opposite-direction events). To get smooth, intuitive control:

- A **fast poll loop** (4ms) drains all dial events and accumulates `delta.signum()` values
- A **rotation tick** (120ms) reads the accumulator and dispatches the *dominant* direction to the active mode (jitter cancels out via summing)
- The hyprscroll mode adds an additional **350ms throttle** so each scroll animation can complete before the next column move

This gives a 1:1 feeling for nudges (one tick = one column / volume step / zoom step) while filtering out hardware jitter.

## Out of scope

- Long-press handling (only single click supported)
- Haptic feedback configuration via hidraw
- Multi-monitor overlay positioning
- GUI configuration tool (config is TOML-only)
