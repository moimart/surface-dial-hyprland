use crate::hypr_ipc;
use std::time::{Duration, Instant};

pub struct HyprScrollMode {
    pixels_per_tick: i32,
    last_dispatch: Option<Instant>,
}

impl HyprScrollMode {
    pub fn new(pixels_per_tick: i32) -> Self {
        Self {
            pixels_per_tick,
            last_dispatch: None,
        }
    }

    pub fn on_rotate(&mut self, delta: i32) {
        // Layout-specific throttle: let Hyprland's scroll animation finish
        let now = Instant::now();
        if let Some(last) = self.last_dispatch {
            if now.duration_since(last) < Duration::from_millis(350) {
                return;
            }
        }
        self.last_dispatch = Some(now);

        if self.pixels_per_tick <= 0 {
            let dir = if delta > 0 { "move -col" } else { "move +col" };
            log::debug!("Hyprland: layoutmsg {dir}");
            if let Err(e) = hypr_ipc::hypr_dispatch("layoutmsg", dir) {
                log::warn!("Hyprland scroll failed: {e}");
            }
        } else {
            let pixels = -delta * self.pixels_per_tick;
            let sign = if pixels > 0 { "+" } else { "" };
            let args = format!("move {sign}{pixels}");
            log::debug!("Hyprland: layoutmsg {args}");
            if let Err(e) = hypr_ipc::hypr_dispatch("layoutmsg", &args) {
                log::warn!("Hyprland scroll failed: {e}");
            }
        }
    }

    pub fn name(&self) -> &str {
        "Scroll Layout"
    }

    pub fn icon(&self) -> &str {
        "\u{f07e}"
    }

    pub fn css_class(&self) -> &str {
        "mode-hyprscroll"
    }
}
