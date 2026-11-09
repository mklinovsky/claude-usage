use chrono::{DateTime, Utc};
use tauri::tray::TrayIcon;

use crate::usage::{FetchError, Usage, UsageWindow};

// Rounding down so 99.6% never shows as 100% while quota remains.
fn percent(value: Option<f64>) -> String {
    value.map_or("–".into(), |p| format!("{:.0}%", p.floor()))
}

fn window_percent(window: &Option<UsageWindow>) -> String {
    percent(window.as_ref().and_then(|w| w.utilization))
}

fn time_left(resets_at: DateTime<Utc>) -> String {
    let seconds = (resets_at - Utc::now()).num_seconds().max(0) as u64;
    let minutes = seconds.div_ceil(60);
    if minutes < 60 {
        format!("{minutes}m")
    } else {
        format!("{}h", minutes.div_ceil(60))
    }
}

fn five_hour(window: &Option<UsageWindow>) -> String {
    let used = window_percent(window);
    match window.as_ref().and_then(|w| w.resets_at) {
        Some(resets_at) => format!("{used} ({})", time_left(resets_at)),
        None => used,
    }
}

fn format_usage(usage: &Usage) -> String {
    let mut title = format!(
        "{} · {}",
        five_hour(&usage.five_hour),
        window_percent(&usage.seven_day)
    );
    for limit in usage.model_limits() {
        title.push_str(&format!(" · {}", percent(limit.percent)));
    }
    title
}

fn describe(error: &FetchError) -> String {
    match error {
        FetchError::RateLimited {
            retry_at: Some(retry_at),
        } => format!("Rate limited, retrying in {}", time_left(*retry_at)),
        FetchError::RateLimited { retry_at: None } => "Rate limited".into(),
        FetchError::Failed(message) => message.clone(),
    }
}

pub struct UsageTray {
    tray: TrayIcon,
    usage: Option<Usage>,
    error: Option<FetchError>,
}

impl UsageTray {
    pub fn new(tray: TrayIcon) -> Self {
        Self {
            tray,
            usage: None,
            error: None,
        }
    }

    pub fn update(&mut self, result: Result<Usage, FetchError>) {
        match result {
            Ok(usage) => {
                self.usage = Some(usage);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn render(&self) {
        let title = match (&self.usage, &self.error) {
            (Some(usage), None) => format_usage(usage),
            (Some(usage), Some(_)) => format!("{} ⚠", format_usage(usage)),
            (None, Some(_)) => "Claude ⚠".into(),
            (None, None) => "Claude …".into(),
        };
        let _ = self.tray.set_title(Some(title));
        let _ = self.tray.set_tooltip(self.error.as_ref().map(describe));
    }
}
