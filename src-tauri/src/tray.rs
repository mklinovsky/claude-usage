use chrono::Local;
use tauri::tray::TrayIcon;

use crate::usage::{Usage, UsageWindow};

// Rounding down so 99.6% never shows as 100% while quota remains.
fn percent(value: Option<f64>) -> String {
    value.map_or("–".into(), |p| format!("{:.0}%", p.floor()))
}

fn window_percent(window: &Option<UsageWindow>) -> String {
    percent(window.as_ref().and_then(|w| w.utilization))
}

fn five_hour(window: &Option<UsageWindow>) -> String {
    let used = window_percent(window);
    match window.as_ref().and_then(|w| w.resets_at) {
        Some(resets_at) => format!("{used} ({})", resets_at.with_timezone(&Local).format("%H:%M")),
        None => used,
    }
}

fn format_usage(usage: &Usage) -> String {
    let mut title = format!(
        "5h {} · 7d {}",
        five_hour(&usage.five_hour),
        window_percent(&usage.seven_day)
    );
    for (name, limit) in usage.model_limits() {
        title.push_str(&format!(" · {name} {}", percent(limit.percent)));
    }
    title
}

pub struct UsageTray {
    tray: TrayIcon,
    last_title: Option<String>,
}

impl UsageTray {
    pub fn new(tray: TrayIcon) -> Self {
        Self { tray, last_title: None }
    }

    pub fn render(&mut self, usage: Result<Usage, String>) {
        match usage {
            Ok(usage) => {
                let title = format_usage(&usage);
                let _ = self.tray.set_title(Some(&title));
                let _ = self.tray.set_tooltip(None::<&str>);
                self.last_title = Some(title);
            }
            Err(error) => {
                let title = self
                    .last_title
                    .as_ref()
                    .map_or("Claude ⚠".into(), |title| format!("{title} ⚠"));
                let _ = self.tray.set_title(Some(title));
                let _ = self.tray.set_tooltip(Some(error));
            }
        }
    }
}
