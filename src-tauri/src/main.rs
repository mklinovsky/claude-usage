mod credentials;
mod tray;
mod usage;

use std::{sync::mpsc, thread, time::Duration};

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    ActivationPolicy,
};

const POLL_INTERVAL: Duration = Duration::from_secs(120);

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
                loop {
                    usage_tray.render(usage::fetch_usage(&client));
                    let _ = refresh_rx.recv_timeout(POLL_INTERVAL);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
