mod config;
mod dial;
mod hypr_ipc;
mod mode;
mod modes;
mod overlay;

use config::Config;
use dial::DialEvent;
use gtk4::glib;
use gtk4::prelude::*;
use mode::ModeManager;
use overlay::Overlay;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = Config::load();
    let overlay_timeout = config.general.overlay_timeout_ms;

    let app = gtk4::Application::builder()
        .application_id("com.surface-dial.daemon")
        .build();

    app.connect_activate(move |app| {
        Overlay::load_css();
        let overlay = Overlay::new(app);
        let manager = Rc::new(RefCell::new(ModeManager::new(&config)));

        // Show initial mode briefly
        {
            let m = manager.borrow();
            overlay.show_mode(m.icon(), m.name(), m.css_class(), overlay_timeout);
        }

        // Spawn dial reader thread
        let (tx, rx) = mpsc::channel::<DialEvent>();
        dial::spawn_reader(tx);

        // Accumulator for rotation direction between ticks
        let rotate_acc: Rc<RefCell<i32>> = Rc::new(RefCell::new(0));

        // Fast poll: drain channel events, accumulate rotation, handle clicks immediately
        let manager_poll = manager.clone();
        let rotate_acc_poll = rotate_acc.clone();
        glib::timeout_add_local(Duration::from_millis(4), move || {
            while let Ok(event) = rx.try_recv() {
                match event {
                    DialEvent::Rotate(delta) if delta != 0 => {
                        *rotate_acc_poll.borrow_mut() += delta.signum();
                    }
                    DialEvent::Click => {
                        let mut m = manager_poll.borrow_mut();
                        m.cycle();
                        overlay.show_mode(m.icon(), m.name(), m.css_class(), overlay_timeout);
                    }
                    DialEvent::Connected => {
                        log::info!("Surface Dial connected");
                        let m = manager_poll.borrow();
                        overlay.show_mode(m.icon(), m.name(), m.css_class(), overlay_timeout);
                    }
                    DialEvent::Disconnected => {
                        log::warn!("Surface Dial disconnected, waiting for reconnect...");
                    }
                    _ => {}
                }
            }
            glib::ControlFlow::Continue
        });

        // Rotation tick: process accumulated direction every 120ms
        let manager_tick = manager.clone();
        glib::timeout_add_local(Duration::from_millis(120), move || {
            let acc = rotate_acc.replace(0);
            if acc != 0 {
                log::debug!("Rotate tick: acc={acc}, dir={}", acc.signum());
                manager_tick.borrow_mut().on_rotate(acc.signum());
            }
            glib::ControlFlow::Continue
        });
    });

    app.run_with_args::<&str>(&[]);
}
