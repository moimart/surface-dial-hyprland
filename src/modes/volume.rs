use std::process::Command;

pub struct VolumeMode {
    pub step_percent: u32,
}

impl VolumeMode {
    pub fn new(step_percent: u32) -> Self {
        Self { step_percent }
    }

    pub fn on_rotate(&self, delta: i32) {
        let direction = if delta.signum() > 0 {
            format!("{}%+", self.step_percent)
        } else {
            format!("{}%-", self.step_percent)
        };

        let result = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &direction])
            .output();

        match result {
            Ok(output) if !output.status.success() => {
                log::warn!(
                    "wpctl failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => log::warn!("Failed to run wpctl: {e}"),
            _ => {}
        }
    }

    pub fn name(&self) -> &str {
        "Volume"
    }

    pub fn icon(&self) -> &str {
        "\u{f028}" // FontAwesome volume-high
    }

    pub fn css_class(&self) -> &str {
        "mode-volume"
    }
}
