use serde_json::json;

pub struct HassMediaMode {
    url: String,
    token: String,
    entity_id: String,
    volume_step: f64,
}

impl HassMediaMode {
    pub fn new(url: String, token: String, entity_id: String, volume_step: f64) -> Self {
        Self {
            url,
            token,
            entity_id,
            volume_step,
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn get_volume(&self) -> Option<f64> {
        let url = format!("{}/api/states/{}", self.url, self.entity_id);
        let resp = ureq::get(&url)
            .set("Authorization", &self.auth_header())
            .call()
            .ok()?;

        let body: serde_json::Value = resp.into_json().ok()?;
        body.get("attributes")?
            .get("volume_level")?
            .as_f64()
    }

    fn set_volume(&self, level: f64) {
        let url = format!("{}/api/services/media_player/volume_set", self.url);
        let payload = json!({
            "entity_id": self.entity_id,
            "volume_level": level,
        });

        let result = ureq::post(&url)
            .set("Authorization", &self.auth_header())
            .set("Content-Type", "application/json")
            .send_json(&payload);

        if let Err(e) = result {
            log::warn!("HA volume_set failed: {e}");
        }
    }

    pub fn on_rotate(&self, delta: i32) {
        let current = match self.get_volume() {
            Some(v) => v,
            None => {
                log::warn!("Failed to get HA media player volume");
                return;
            }
        };

        let new_vol = (current + -delta as f64 * self.volume_step).clamp(0.0, 1.0);
        if (new_vol - current).abs() < 0.001 {
            return;
        }

        log::debug!("HA media: {current:.2} -> {new_vol:.2}");
        self.set_volume(new_vol);
    }

    pub fn name(&self) -> &str {
        "HA Media"
    }

    pub fn icon(&self) -> &str {
        "\u{f008}" // FontAwesome film
    }

    pub fn css_class(&self) -> &str {
        "mode-hass-media"
    }
}
