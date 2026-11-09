mod credentials;
mod tray;
mod usage;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy,
};
use usage::FetchError;

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
                let mut next_fetch = Instant::now();
                let mut not_before = next_fetch;
                loop {
                    if Instant::now() >= next_fetch {
                        let result = usage::fetch_usage(&client);
                        next_fetch = match &result {
                            Err(FetchError::RateLimited {
                                retry_at: Some(retry_at),
                            }) => {
                                let wait = (*retry_at - Utc::now()).to_std().unwrap_or_default();
                                not_before = Instant::now() + wait;
                                not_before
                            }
                            _ => Instant::now() + POLL_INTERVAL,
                        };
                        usage_tray.update(result);
                    }
                    usage_tray.render();
                    let timeout =
                        RENDER_INTERVAL.min(next_fetch.saturating_duration_since(Instant::now()));
                    if refresh_rx.recv_timeout(timeout).is_ok() {
                        next_fetch = not_before;
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
