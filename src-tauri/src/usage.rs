use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::credentials;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Deserialize)]
pub struct Usage {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
    limits: Option<Vec<Limit>>,
}

#[derive(Deserialize)]
pub struct UsageWindow {
    pub utilization: Option<f64>,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct Limit {
    kind: String,
    pub percent: Option<f64>,
    scope: Option<LimitScope>,
}

#[derive(Deserialize)]
struct LimitScope {
    model: Option<LimitModel>,
}

#[derive(Deserialize)]
struct LimitModel {
    display_name: Option<String>,
}

impl Usage {
    pub fn model_limits(&self) -> impl Iterator<Item = (&str, &Limit)> {
        self.limits
            .iter()
            .flatten()
            .filter(|limit| limit.kind == "weekly_scoped")
            .filter_map(|limit| {
                let name = limit.scope.as_ref()?.model.as_ref()?.display_name.as_deref()?;
                Some((name, limit))
            })
    }
}

pub fn fetch_usage(client: &reqwest::blocking::Client) -> Result<Usage, String> {
    let token = credentials::read_token()?;
    client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.json())
        .map_err(|e| e.to_string())
}
