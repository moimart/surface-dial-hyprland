use std::process::Command;

pub struct ZoomMode {
    pub step: f64,
}

impl ZoomMode {
    pub fn new(step: f64) -> Self {
        Self { step }
    }

    fn get_zoom_factor() -> f64 {
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

        let arg = format!("cursor:zoom_factor {new_zoom:.2}");
        let result = Command::new("hyprctl")
            .args(["keyword", &arg])
            .output();

        match result {
            Ok(o) if !o.status.success() => {
                log::warn!("hyprctl zoom failed: {}", String::from_utf8_lossy(&o.stderr));
            }
            Err(e) => log::warn!("Failed to run hyprctl: {e}"),
            _ => log::debug!("Zoom: {current:.2} -> {new_zoom:.2}"),
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
