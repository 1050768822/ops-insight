use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::application::dtos::QueryRange;
use crate::domain::entities::{ErrorEvent, LogEntry, LogLevel};
use crate::domain::ports::DataSource;
use crate::domain::value_objects::SecretKey;

pub struct NewRelicSource {
    api_key: SecretKey,
    account_id: String,
    client: reqwest::Client,
}

impl NewRelicSource {
    pub fn new(api_key: SecretKey, account_id: String) -> Self {
        Self {
            api_key,
            account_id,
            client: reqwest::Client::new(),
        }
    }

    async fn run_nrql<T: for<'de> Deserialize<'de>>(
        &self,
        nrql: &str,
    ) -> anyhow::Result<Vec<T>> {
        let query = format!(
            r#"{{ "query": "{{ actor {{ account(id: {account_id}) {{ nrql(query: \"{nrql}\") {{ results }} }} }} }}" }}"#,
            account_id = self.account_id,
            nrql = nrql.replace('"', "\\\""),
        );

        let resp: serde_json::Value = self
            .api_key
            .use_key("newrelic_nrql_request", |key| {
                self.client
                    .post("https://api.newrelic.com/graphql")
                    .header("Api-Key", key)
                    .header("Content-Type", "application/json")
                    .body(query)
                    .send()
            })
            .await?
            .json()
            .await?;

        let results = resp["data"]["actor"]["account"]["nrql"]["results"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let items = results
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        Ok(items)
    }

    fn hostname_filter(hostnames: &[String]) -> String {
        if hostnames.is_empty() {
            return String::new();
        }
        let list = hostnames
            .iter()
            .map(|h| format!("'{h}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(" AND hostname IN ({list})")
    }
}

#[derive(Deserialize)]
struct RawLog {
    timestamp: i64,
    level: Option<String>,
    message: Option<String>,
    hostname: Option<String>,
    service: Option<String>,
}

#[derive(Deserialize)]
struct RawError {
    #[serde(rename = "earliest")]
    first_seen: i64,
    #[serde(rename = "latest")]
    last_seen: i64,
    #[serde(rename = "count")]
    count: u64,
    message: Option<String>,
    hostname: Option<String>,
    service: Option<String>,
}

#[async_trait]
impl DataSource for NewRelicSource {
    async fn fetch_logs(&self, range: &QueryRange) -> anyhow::Result<Vec<LogEntry>> {
        let from = range.from.timestamp_millis();
        let to = range.to.timestamp_millis();
        let host_filter = Self::hostname_filter(&range.hostnames);

        let nrql = format!(
            "SELECT timestamp, level, message, hostname, service FROM Log \
             WHERE timestamp >= {from} AND timestamp <= {to}{host_filter} \
             LIMIT 1000"
        );

        let raw: Vec<RawLog> = self.run_nrql(&nrql).await?;

        let entries = raw
            .into_iter()
            .map(|r| LogEntry {
                timestamp: DateTime::from_timestamp_millis(r.timestamp)
                    .unwrap_or_else(Utc::now),
                level: parse_level(r.level.as_deref()),
                message: r.message.unwrap_or_default(),
                hostname: r.hostname.unwrap_or_default(),
                service: r.service,
            })
            .collect();

        Ok(entries)
    }

    async fn fetch_errors(&self, range: &QueryRange) -> anyhow::Result<Vec<ErrorEvent>> {
        let from = range.from.timestamp_millis();
        let to = range.to.timestamp_millis();
        let host_filter = Self::hostname_filter(&range.hostnames);

        let nrql = format!(
            "SELECT count(*) AS count, min(timestamp) AS earliest, max(timestamp) AS latest, \
             message, hostname, service FROM Log \
             WHERE level IN ('ERROR','FATAL') \
             AND timestamp >= {from} AND timestamp <= {to}{host_filter} \
             FACET message, hostname, service LIMIT 100"
        );

        let raw: Vec<RawError> = self.run_nrql(&nrql).await?;

        let events = raw
            .into_iter()
            .map(|r| ErrorEvent {
                timestamp: DateTime::from_timestamp_millis(r.last_seen)
                    .unwrap_or_else(Utc::now),
                message: r.message.unwrap_or_default(),
                hostname: r.hostname.unwrap_or_default(),
                service: r.service,
                count: r.count,
                first_seen: DateTime::from_timestamp_millis(r.first_seen)
                    .unwrap_or_else(Utc::now),
                last_seen: DateTime::from_timestamp_millis(r.last_seen)
                    .unwrap_or_else(Utc::now),
            })
            .collect();

        Ok(events)
    }
}

fn parse_level(level: Option<&str>) -> LogLevel {
    match level.map(|s| s.to_uppercase()).as_deref() {
        Some("DEBUG") => LogLevel::Debug,
        Some("INFO") => LogLevel::Info,
        Some("WARN") | Some("WARNING") => LogLevel::Warn,
        Some("ERROR") => LogLevel::Error,
        Some("FATAL") | Some("CRITICAL") => LogLevel::Fatal,
        _ => LogLevel::Info,
    }
}
