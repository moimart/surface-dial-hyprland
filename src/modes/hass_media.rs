use serde_json::json;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct HassMediaMode {
    url: String,
    token: String,
    entity_id: String,
    volume_step: f64,
    cached_volume: Arc<Mutex<Option<f64>>>,
}

impl HassMediaMode {
    pub fn new(url: String, token: String, entity_id: String, volume_step: f64) -> Self {
        let mode = Self {
            url,
            token,
            entity_id,
            volume_step,
            cached_volume: Arc::new(Mutex::new(None)),
        };
        // Fetch initial volume in background
        mode.sync_volume();
        mode
    }

    fn sync_volume(&self) {
        let url = format!("{}/api/states/{}", self.url, self.entity_id);
        let auth = format!("Bearer {}", self.token);
        let cache = self.cached_volume.clone();

        thread::spawn(move || {
            let vol = fetch_volume(&url, &auth);
            if let Some(v) = vol {
                *cache.lock().unwrap() = Some(v);
                log::debug!("HA media: synced volume = {v:.2}");
            }
        });
    }

    fn send_volume(&self, level: f64) {
        let url = format!("{}/api/services/media_player/volume_set", self.url);
        let auth = format!("Bearer {}", self.token);
        let entity_id = self.entity_id.clone();

        thread::spawn(move || {
            let payload = json!({
                "entity_id": entity_id,
                "volume_level": level,
            });
            let result = ureq::post(&url)
                .set("Authorization", &auth)
                .set("Content-Type", "application/json")
                .send_json(&payload);
            if let Err(e) = result {
                log::warn!("HA volume_set failed: {e}");
            }
        });
    }

    pub fn on_rotate(&mut self, delta: i32) -> Option<f64> {
        let mut cache = self.cached_volume.lock().unwrap();
        let current = match *cache {
            Some(v) => v,
            None => {
                drop(cache);
                self.sync_volume();
                return None;
            }
        };

        let new_vol = (current + -delta as f64 * self.volume_step).clamp(0.0, 1.0);
        if (new_vol - current).abs() < 0.001 {
            return Some(current);
        }

        *cache = Some(new_vol);
        drop(cache);

        log::debug!("HA media: {current:.2} -> {new_vol:.2}");
        self.send_volume(new_vol);
        Some(new_vol)
    }

    pub fn name(&self) -> &str {
        "HA Media"
    }

    pub fn icon(&self) -> &str {
        "\u{f008}"
    }

    pub fn css_class(&self) -> &str {
        "mode-hass-media"
    }
}

fn fetch_volume(url: &str, auth: &str) -> Option<f64> {
    let resp = ureq::get(url)
        .set("Authorization", auth)
        .call()
        .ok()?;
    let body: serde_json::Value = resp.into_json().ok()?;
    body.get("attributes")?
        .get("volume_level")?
        .as_f64()
}
