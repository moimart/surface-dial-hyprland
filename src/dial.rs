use evdev::{Device, InputEventKind, RelativeAxisType};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const DEVICE_NAME: &str = "Surface Dial System Multi Axis";
const BTN_0: u16 = 256;

#[derive(Debug, Clone)]
pub enum DialEvent {
    Click,
    Rotate(i32),
    Connected,
    Disconnected,
}

pub fn spawn_reader(tx: mpsc::Sender<DialEvent>) {
    thread::spawn(move || loop {
        match find_device() {
            Some(mut device) => {
                log::info!(
                    "Surface Dial connected: {}",
                    device.name().unwrap_or("unknown")
                );
                let _ = tx.send(DialEvent::Connected);

                if let Err(e) = device.grab() {
                    log::warn!("Failed to grab device (non-fatal): {e}");
                }

                loop {
                    match device.fetch_events() {
                        Ok(events) => {
                            for event in events {
                                match event.kind() {
                                    InputEventKind::RelAxis(RelativeAxisType::REL_DIAL) => {
                                        let _ = tx.send(DialEvent::Rotate(event.value()));
                                    }
                                    InputEventKind::Key(_) if event.code() == BTN_0 => {
                                        if event.value() == 1 {
                                            let _ = tx.send(DialEvent::Click);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Device read error: {e}");
                            let _ = tx.send(DialEvent::Disconnected);
                            break;
                        }
                    }
                }
            }
            None => {
                log::debug!("Surface Dial not found, retrying in 2s...");
                thread::sleep(Duration::from_secs(2));
            }
        }
    });
}

fn find_device() -> Option<Device> {
    let mut enumerator = udev::Enumerator::new().ok()?;
    enumerator.match_subsystem("input").ok()?;

    for udev_device in enumerator.scan_devices().ok()? {
        let devnode = match udev_device.devnode() {
            Some(node) if node.to_string_lossy().contains("/event") => node.to_path_buf(),
            _ => continue,
        };

        if let Ok(device) = Device::open(&devnode) {
            if device.name() == Some(DEVICE_NAME) {
                return Some(device);
            }
        }
    }

    None
}
