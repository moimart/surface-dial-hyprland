use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub struct Overlay {
    window: gtk4::Window,
    icon_label: gtk4::Label,
    name_label: gtk4::Label,
    container: gtk4::Box,
    epoch: std::cell::Cell<u64>,
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

        // Layer shell setup
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(-1);

        // Center on screen: don't anchor to any edge
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        // Build UI
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        container.add_css_class("overlay-container");
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);

        let icon_label = gtk4::Label::new(None);
        icon_label.add_css_class("overlay-icon");

        let name_label = gtk4::Label::new(None);
        name_label.add_css_class("overlay-name");

        container.append(&icon_label);
        container.append(&name_label);
        window.set_child(Some(&container));

        Self {
            window,
            icon_label,
            name_label,
            container,
            epoch: std::cell::Cell::new(0),
        }
    }

    pub fn show_mode(&self, icon: &str, name: &str, css_class: &str, timeout_ms: u64) {
        // Remove previous mode class
        for class in ["mode-hyprscroll", "mode-volume", "mode-appscroll", "mode-zoom", "mode-hass-media"] {
            self.container.remove_css_class(class);
        }
        self.container.add_css_class(css_class);

        self.icon_label.set_text(icon);
        self.name_label.set_text(name);

        self.window.set_opacity(1.0);
        self.window.set_visible(true);

        // Bump epoch to invalidate any pending hide timeout
        let snap = self.epoch.get().wrapping_add(1);
        self.epoch.set(snap);

        // Schedule hide — the closure captures the epoch and only hides
        // if no newer show_mode call has occurred since
        let window = self.window.clone();
        let epoch = self.epoch.clone();
        glib::timeout_add_local_once(
            std::time::Duration::from_millis(timeout_ms),
            move || {
                if epoch.get() == snap {
                    window.set_visible(false);
                }
            },
        );
    }

    pub fn load_css() {
        let provider = gtk4::CssProvider::new();

        // Try loading user theme first
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
"#;
