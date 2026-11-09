use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{blocking::Response, header::RETRY_AFTER, StatusCode};
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

pub enum FetchError {
    RateLimited { retry_at: Option<DateTime<Utc>> },
    Failed(String),
}

fn failed(error: reqwest::Error) -> FetchError {
    FetchError::Failed(error.to_string())
}

fn retry_at(response: &Response) -> Option<DateTime<Utc>> {
    let seconds: i64 = response.headers().get(RETRY_AFTER)?.to_str().ok()?.parse().ok()?;
    Some(Utc::now() + TimeDelta::seconds(seconds))
}

pub fn fetch_usage(client: &reqwest::blocking::Client) -> Result<Usage, FetchError> {
    let token = credentials::read_token().map_err(FetchError::Failed)?;
    let response = client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .map_err(failed)?;
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(FetchError::RateLimited {
            retry_at: retry_at(&response),
        });
    }
    response
        .error_for_status()
        .and_then(|response| response.json())
        .map_err(failed)
}
