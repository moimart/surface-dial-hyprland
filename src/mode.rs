use crate::config::Config;
use crate::modes::{AppScrollMode, HyprScrollMode, VolumeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeKind {
    HyprScroll,
    Volume,
    AppScroll,
}

impl ModeKind {
    pub fn next(self) -> Self {
        match self {
            Self::HyprScroll => Self::Volume,
            Self::Volume => Self::AppScroll,
            Self::AppScroll => Self::HyprScroll,
        }
    }
}

pub struct ModeManager {
    pub current: ModeKind,
    pub hyprscroll: HyprScrollMode,
    pub volume: VolumeMode,
    pub appscroll: AppScrollMode,
}

impl ModeManager {
    pub fn new(config: &Config) -> Self {
        Self {
            current: ModeKind::HyprScroll,
            hyprscroll: HyprScrollMode::new(config.hyprscroll.pixels_per_tick),
            volume: VolumeMode::new(config.volume.step_percent),
            appscroll: AppScrollMode::new(config.appscroll.speed_multiplier),
        }
    }

    pub fn cycle(&mut self) -> ModeKind {
        self.current = self.current.next();
        log::info!("Mode switched to: {}", self.name());
        self.current
    }

    pub fn on_rotate(&mut self, delta: i32) {
        match self.current {
            ModeKind::HyprScroll => self.hyprscroll.on_rotate(delta),
            ModeKind::Volume => self.volume.on_rotate(delta),
            ModeKind::AppScroll => self.appscroll.on_rotate(delta),
        }
    }

    pub fn name(&self) -> &str {
        match self.current {
            ModeKind::HyprScroll => self.hyprscroll.name(),
            ModeKind::Volume => self.volume.name(),
            ModeKind::AppScroll => self.appscroll.name(),
        }
    }

    pub fn icon(&self) -> &str {
        match self.current {
            ModeKind::HyprScroll => self.hyprscroll.icon(),
            ModeKind::Volume => self.volume.icon(),
            ModeKind::AppScroll => self.appscroll.icon(),
        }
    }

    pub fn css_class(&self) -> &str {
        match self.current {
            ModeKind::HyprScroll => self.hyprscroll.css_class(),
            ModeKind::Volume => self.volume.css_class(),
            ModeKind::AppScroll => self.appscroll.css_class(),
        }
    }
}
