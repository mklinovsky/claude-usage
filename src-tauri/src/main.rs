mod credentials;
mod tray;
mod usage;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy,
};

const POLL_INTERVAL: Duration = Duration::from_secs(120);
// The countdown shows minutes, so redraw more often than the API is polled.
const RENDER_INTERVAL: Duration = Duration::from_secs(60);

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.set_activation_policy(ActivationPolicy::Accessory);

            let (refresh_tx, refresh_rx) = mpsc::channel();
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;
            let tray = TrayIconBuilder::new()
                .title("Claude …")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "refresh" => {
                        let _ = refresh_tx.send(());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            thread::spawn(move || {
                let client = reqwest::blocking::Client::new();
                let mut usage_tray = tray::UsageTray::new(tray);
                let mut last_fetch: Option<Instant> = None;
                loop {
                    if last_fetch.is_none_or(|fetched| fetched.elapsed() >= POLL_INTERVAL) {
                        usage_tray.update(usage::fetch_usage(&client));
                        last_fetch = Some(Instant::now());
                    }
                    usage_tray.render();
                    if refresh_rx.recv_timeout(RENDER_INTERVAL).is_ok() {
                        last_fetch = None;
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
