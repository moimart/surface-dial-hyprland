use crate::config::Config;
use crate::modes::{AppScrollMode, HyprScrollMode, VolumeMode, ZoomMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    HyprScroll,
    Volume,
    AppScroll,
    Zoom,
}

impl ModeKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hyprscroll" => Some(Self::HyprScroll),
            "volume" => Some(Self::Volume),
            "appscroll" => Some(Self::AppScroll),
            "zoom" => Some(Self::Zoom),
            _ => {
                log::warn!("Unknown mode in config: {s}");
                None
            }
        }
    }
}

pub struct ModeManager {
    pub current: usize,
    pub order: Vec<ModeKind>,
    pub hyprscroll: HyprScrollMode,
    pub volume: VolumeMode,
    pub appscroll: AppScrollMode,
    pub zoom: ZoomMode,
}

impl ModeManager {
    pub fn new(config: &Config) -> Self {
        let order: Vec<ModeKind> = config
            .general
            .mode_order
            .iter()
            .filter_map(|s| ModeKind::from_str(s))
            .collect();

        // Fallback if config produced an empty list
        let order = if order.is_empty() {
            vec![
                ModeKind::Volume,
                ModeKind::Zoom,
                ModeKind::AppScroll,
                ModeKind::HyprScroll,
            ]
        } else {
            order
        };

        Self {
            current: 0,
            order,
            hyprscroll: HyprScrollMode::new(config.hyprscroll.pixels_per_tick),
            volume: VolumeMode::new(config.volume.step_percent),
            appscroll: AppScrollMode::new(config.appscroll.speed_multiplier),
            zoom: ZoomMode::new(config.zoom.step),
        }
    }

    fn current_mode(&self) -> ModeKind {
        self.order[self.current]
    }

    pub fn cycle(&mut self) -> ModeKind {
        self.current = (self.current + 1) % self.order.len();
        log::info!("Mode switched to: {}", self.name());
        self.current_mode()
    }

    pub fn on_rotate(&mut self, delta: i32) {
        match self.current_mode() {
            ModeKind::HyprScroll => self.hyprscroll.on_rotate(delta),
            ModeKind::Volume => self.volume.on_rotate(delta),
            ModeKind::AppScroll => self.appscroll.on_rotate(delta),
            ModeKind::Zoom => self.zoom.on_rotate(delta),
        }
    }

    pub fn name(&self) -> &str {
        match self.current_mode() {
            ModeKind::HyprScroll => self.hyprscroll.name(),
            ModeKind::Volume => self.volume.name(),
            ModeKind::AppScroll => self.appscroll.name(),
            ModeKind::Zoom => self.zoom.name(),
        }
    }

    pub fn icon(&self) -> &str {
        match self.current_mode() {
            ModeKind::HyprScroll => self.hyprscroll.icon(),
            ModeKind::Volume => self.volume.icon(),
            ModeKind::AppScroll => self.appscroll.icon(),
            ModeKind::Zoom => self.zoom.icon(),
        }
    }

    pub fn css_class(&self) -> &str {
        match self.current_mode() {
            ModeKind::HyprScroll => self.hyprscroll.css_class(),
            ModeKind::Volume => self.volume.css_class(),
            ModeKind::AppScroll => self.appscroll.css_class(),
            ModeKind::Zoom => self.zoom.css_class(),
        }
    }
}
