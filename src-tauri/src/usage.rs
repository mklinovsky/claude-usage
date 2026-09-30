use chrono::{DateTime, Utc};
use serde::{de::IgnoredAny, Deserialize};

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
    model: Option<IgnoredAny>,
}

impl Usage {
    pub fn model_limits(&self) -> impl Iterator<Item = &Limit> {
        self.limits.iter().flatten().filter(|limit| {
            limit.kind == "weekly_scoped"
                && limit.scope.as_ref().is_some_and(|scope| scope.model.is_some())
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
