use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, EventType, InputEvent, RelativeAxisType};

pub struct AppScrollMode {
    pub speed_multiplier: i32,
    device: Option<VirtualDevice>,
}

impl AppScrollMode {
    pub fn new(speed_multiplier: i32) -> Self {
        let device = match Self::create_virtual_device() {
            Ok(d) => {
                log::info!("Created virtual scroll device");
                Some(d)
            }
            Err(e) => {
                log::error!("Failed to create virtual scroll device: {e}");
                None
            }
        };
        Self {
            speed_multiplier,
            device,
        }
    }

    fn create_virtual_device() -> Result<VirtualDevice, Box<dyn std::error::Error>> {
        let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
        rel_axes.insert(RelativeAxisType::REL_WHEEL);
        rel_axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);

        let device = VirtualDeviceBuilder::new()?
            .name("Surface Dial Virtual Scroll")
            .with_relative_axes(&rel_axes)?
            .build()?;

        Ok(device)
    }

    pub fn on_rotate(&mut self, delta: i32) {
        let Some(ref mut device) = self.device else {
            return;
        };

        // Normalize to ±1 — dial sends variable magnitude but frequency conveys speed
        let direction = delta.signum() * self.speed_multiplier;
        let hires_value = direction * 120;

        let events = [
            InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_WHEEL.0, direction),
            InputEvent::new(
                EventType::RELATIVE,
                RelativeAxisType::REL_WHEEL_HI_RES.0,
                hires_value,
            ),
        ];

        if let Err(e) = device.emit(&events) {
            log::warn!("Failed to emit scroll event: {e}");
        }
    }

    pub fn name(&self) -> &str {
        "App Scroll"
    }

    pub fn icon(&self) -> &str {
        "\u{f0dc}" // FontAwesome sort (up-down arrows)
    }

    pub fn css_class(&self) -> &str {
        "mode-appscroll"
    }
}
