use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

pub struct Overlay {
    window: gtk4::Window,
    icon_label: gtk4::Label,
    name_label: gtk4::Label,
    volume_bar: gtk4::ProgressBar,
    volume_label: gtk4::Label,
    volume_box: gtk4::Box,
    container: gtk4::Box,
    last_activity: Rc<Cell<Instant>>,
    hide_timeout_ms: Rc<Cell<u64>>,
    hide_timer_active: Rc<Cell<bool>>,
}

impl Overlay {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::Window::builder()
            .application(app)
            .default_width(160)
            .default_height(160)
            .decorated(false)
            .resizable(false)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(-1);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        container.add_css_class("overlay-container");
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        let icon_label = gtk4::Label::new(None);
        icon_label.add_css_class("overlay-icon");

        let name_label = gtk4::Label::new(None);
        name_label.add_css_class("overlay-name");

        let volume_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        volume_box.add_css_class("volume-box");
        volume_box.set_visible(false);

        let volume_bar = gtk4::ProgressBar::new();
        volume_bar.add_css_class("volume-bar");
        volume_bar.set_width_request(200);

        let volume_label = gtk4::Label::new(None);
        volume_label.add_css_class("volume-label");

        volume_box.append(&volume_bar);
        volume_box.append(&volume_label);

        container.append(&icon_label);
        container.append(&name_label);
        container.append(&volume_box);
        window.set_child(Some(&container));

        Self {
            window,
            icon_label,
            name_label,
            volume_bar,
            volume_label,
            volume_box,
            container,
            last_activity: Rc::new(Cell::new(Instant::now())),
            hide_timeout_ms: Rc::new(Cell::new(1500)),
            hide_timer_active: Rc::new(Cell::new(false)),
        }
    }

    fn clear_classes(&self) {
        for class in [
            "mode-hyprscroll", "mode-volume", "mode-appscroll", "mode-zoom",
            "mode-hass-media", "status-connected", "status-disconnected",
        ] {
            self.container.remove_css_class(class);
        }
    }

    fn ensure_hide_timer(&self) {
        self.last_activity.set(Instant::now());

        if self.hide_timer_active.get() {
            return;
        }
        self.hide_timer_active.set(true);

        let window = self.window.clone();
        let volume_box = self.volume_box.clone();
        let last_activity = self.last_activity.clone();
        let hide_timeout_ms = self.hide_timeout_ms.clone();
        let hide_timer_active = self.hide_timer_active.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
            let elapsed = last_activity.get().elapsed().as_millis() as u64;
            if elapsed >= hide_timeout_ms.get() {
                window.set_visible(false);
                volume_box.set_visible(false);
                hide_timer_active.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    pub fn show_status(&self, icon: &str, name: &str, connected: bool, timeout_ms: u64) {
        let css_class = if connected { "status-connected" } else { "status-disconnected" };
        self.clear_classes();
        self.container.add_css_class(css_class);
        self.icon_label.set_text(icon);
        self.name_label.set_text(name);
        self.volume_box.set_visible(false);
        self.hide_timeout_ms.set(timeout_ms);
        self.window.set_opacity(1.0);
        self.window.set_visible(true);
        self.ensure_hide_timer();
    }

    pub fn show_mode(&self, icon: &str, name: &str, css_class: &str, timeout_ms: u64) {
        self.clear_classes();
        self.container.add_css_class(css_class);
        self.icon_label.set_text(icon);
        self.name_label.set_text(name);
        self.volume_box.set_visible(false);
        self.hide_timeout_ms.set(timeout_ms);
        self.window.set_opacity(1.0);
        self.window.set_visible(true);
        self.ensure_hide_timer();
    }

    pub fn show_volume(&self, icon: &str, name: &str, css_class: &str, volume: f64, timeout_ms: u64) {
        let pct = (volume * 100.0).round() as i32;
        self.volume_bar.set_fraction(volume.clamp(0.0, 1.0));
        self.volume_label.set_text(&format!("{pct}%"));
        self.volume_box.set_visible(true);

        if !self.window.is_visible() {
            self.clear_classes();
            self.container.add_css_class(css_class);
            self.icon_label.set_text(icon);
            self.name_label.set_text(name);
            self.window.set_opacity(1.0);
            self.window.set_visible(true);
        }

        self.hide_timeout_ms.set(timeout_ms.max(2500));
        self.ensure_hide_timer();
    }

    pub fn load_css() {
        let provider = gtk4::CssProvider::new();

        let user_css = crate::config::Config::config_dir().join("theme.css");
        if user_css.exists() {
            provider.load_from_path(user_css.to_string_lossy().as_ref());
            log::info!("Loaded user theme from {}", user_css.display());
        } else {
            provider.load_from_string(DEFAULT_CSS);
            log::info!("Using default theme");
        }

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

const DEFAULT_CSS: &str = r#"
window {
    background-color: transparent;
}

.overlay-container {
    background-color: rgba(30, 30, 46, 0.92);
    border-radius: 24px;
    padding: 24px 32px;
    border: 2px solid rgba(137, 180, 250, 0.3);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.overlay-icon {
    font-size: 48px;
    color: #89b4fa;
}

.overlay-name {
    font-size: 14px;
    font-weight: bold;
    color: #cdd6f4;
    letter-spacing: 1px;
    text-transform: uppercase;
}

.volume-box {
    margin-top: 8px;
}

.volume-bar trough {
    min-height: 8px;
    border-radius: 4px;
    background-color: rgba(108, 112, 134, 0.4);
}

.volume-bar progress {
    min-height: 8px;
    border-radius: 4px;
    background-color: #f38ba8;
}

.volume-label {
    font-size: 18px;
    font-weight: bold;
    color: #eff1f5;
}

.mode-hyprscroll .overlay-icon { color: #89b4fa; }
.mode-hyprscroll { border-color: rgba(137, 180, 250, 0.3); }

.mode-volume .overlay-icon { color: #a6e3a1; }
.mode-volume { border-color: rgba(166, 227, 161, 0.3); }

.mode-appscroll .overlay-icon { color: #f9e2af; }
.mode-appscroll { border-color: rgba(249, 226, 175, 0.3); }

.mode-zoom .overlay-icon { color: #cba6f7; }
.mode-zoom { border-color: rgba(203, 166, 247, 0.3); }

.mode-hass-media .overlay-icon { color: #f38ba8; }
.mode-hass-media { border-color: rgba(243, 139, 168, 0.3); }

.status-connected .overlay-icon { color: #a6e3a1; }
.status-connected { border-color: rgba(166, 227, 161, 0.3); }

.status-disconnected .overlay-icon { color: #f38ba8; }
.status-disconnected { border-color: rgba(243, 139, 168, 0.3); }
"#;
