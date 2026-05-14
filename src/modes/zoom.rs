use crate::hypr_ipc;
use std::process::Command;

pub struct ZoomMode {
    pub step: f64,
}

impl ZoomMode {
    pub fn new(step: f64) -> Self {
        Self { step }
    }

    fn get_zoom_factor() -> f64 {
        // `hyprctl getoption` works on both 0.54 (hyprlang) and 0.55 (Lua).
        let output = Command::new("hyprctl")
            .args(["getoption", "cursor:zoom_factor"])
            .output()
            .ok();

        output
            .and_then(|o| {
                let text = String::from_utf8_lossy(&o.stdout);
                text.lines()
                    .find(|l| l.contains("float:"))
                    .and_then(|l| l.split_whitespace().last())
                    .and_then(|v| v.parse::<f64>().ok())
            })
            .unwrap_or(1.0)
    }

    fn set_zoom_factor(new_zoom: f64) -> Result<(), String> {
        if hypr_ipc::use_lua() {
            // 0.55+: `hyprctl keyword` is rejected by the Lua parser
            // ("keyword can't work with non-legacy parsers. Use eval.").
            let lua = format!(
                "hl.config({{ cursor = {{ zoom_factor = {new_zoom:.2} }} }})"
            );
            return hypr_ipc::hypr_eval(&lua);
        }
        // 0.54 and earlier: legacy keyword path.
        let arg = format!("cursor:zoom_factor {new_zoom:.2}");
        let output = Command::new("hyprctl")
            .args(["keyword", &arg])
            .output()
            .map_err(|e| format!("Failed to run hyprctl: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "hyprctl keyword failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    pub fn on_rotate(&self, delta: i32) {
        let current = Self::get_zoom_factor();
        let new_zoom = if delta > 0 {
            (current - self.step).max(1.0)
        } else {
            (current + self.step).min(10.0)
        };

        if (new_zoom - current).abs() < 0.01 {
            return;
        }

        match Self::set_zoom_factor(new_zoom) {
            Ok(()) => log::debug!("Zoom: {current:.2} -> {new_zoom:.2}"),
            Err(e) => log::warn!("Zoom set failed: {e}"),
        }
    }

    pub fn name(&self) -> &str {
        "Zoom"
    }

    pub fn icon(&self) -> &str {
        "\u{f00e}" // FontAwesome search-plus
    }

    pub fn css_class(&self) -> &str {
        "mode-zoom"
    }
}
