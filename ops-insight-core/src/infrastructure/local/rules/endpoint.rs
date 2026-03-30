use super::rule::LocalRule;
use crate::domain::entities::{Issue, Priority, Severity, Suggestion};
use crate::domain::ports::AnalysisInput;
use regex::Regex;
use std::collections::HashMap;

pub struct EndpointStatsRule {
    /// 慢请求阈值（毫秒），超过此值标记为慢请求
    pub slow_threshold_ms: u64,
    /// 报告中显示的 Top N 接口数量
    pub top_n: usize,
}

impl Default for EndpointStatsRule {
    fn default() -> Self {
        Self {
            slow_threshold_ms: 1000,
            top_n: 10,
        }
    }
}

struct EndpointStat {
    count: u64,
    error_count: u64,
    durations_ms: Vec<u64>,
}

impl EndpointStat {
    fn new() -> Self {
        Self {
            count: 0,
            error_count: 0,
            durations_ms: Vec::new(),
        }
    }

    fn avg_duration_ms(&self) -> Option<u64> {
        if self.durations_ms.is_empty() {
            return None;
        }
        Some(self.durations_ms.iter().sum::<u64>() / self.durations_ms.len() as u64)
    }

    fn error_rate(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.error_count as f64 / self.count as f64
    }
}

fn aggregate_stats(input: &AnalysisInput) -> HashMap<String, EndpointStat> {
    let http_re = Regex::new(
        r"(?i)(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+(/[^\s]*)\s+(\d{3})(?:\s+(\d+)ms)?",
    )
    .unwrap();

    let aspnet_re = Regex::new(
        r"(?i)Request finished[^\n]*?(GET|POST|PUT|DELETE|PATCH)\s+(/[^\s]*)[^\n]*?(\d{3})[^\n]*?(\d+(?:\.\d+)?)ms"
    ).unwrap();

    let mut stats: HashMap<String, EndpointStat> = HashMap::new();

    for log in &input.logs {
        let message = &log.message;

        if let Some(caps) = http_re.captures(message) {
            let method = caps[1].to_uppercase();
            let path = caps[2].to_string();
            let status: u16 = caps[3].parse().unwrap_or(0);
            let duration_ms: Option<u64> = caps.get(4).and_then(|m| m.as_str().parse().ok());

            let key = format!("{} {}", method, path);
            let entry = stats.entry(key).or_insert_with(EndpointStat::new);
            entry.count += 1;
            if status >= 400 {
                entry.error_count += 1;
            }
            if let Some(ms) = duration_ms {
                entry.durations_ms.push(ms);
            }
            continue;
        }

        if let Some(caps) = aspnet_re.captures(message) {
            let method = caps[1].to_uppercase();
            let path = caps[2].to_string();
            let status: u16 = caps[3].parse().unwrap_or(0);
            let duration_ms: Option<u64> = caps
                .get(4)
                .and_then(|m| m.as_str().parse::<f64>().ok().map(|d| d as u64));

            let key = format!("{} {}", method, path);
            let entry = stats.entry(key).or_insert_with(EndpointStat::new);
            entry.count += 1;
            if status >= 400 {
                entry.error_count += 1;
            }
            if let Some(ms) = duration_ms {
                entry.durations_ms.push(ms);
            }
        }
    }

    stats
}

impl LocalRule for EndpointStatsRule {
    fn name(&self) -> &str {
        "endpoint_stats"
    }

    fn check(&self, input: &AnalysisInput) -> Vec<Issue> {
        let stats = aggregate_stats(input);
        if stats.is_empty() {
            return vec![];
        }

        let mut issues = Vec::new();

        // 高频接口 — top N by request count
        if stats.len() >= 5 {
            let mut by_count: Vec<(&String, &EndpointStat)> = stats.iter().collect();
            by_count.sort_by(|a, b| b.1.count.cmp(&a.1.count));
            by_count.truncate(self.top_n);

            let description = by_count
                .iter()
                .map(|(endpoint, stat)| format!("{}: {} 次请求", endpoint, stat.count))
                .collect::<Vec<_>>()
                .join("\n");

            issues.push(Issue {
                severity: Severity::Low,
                title: format!("接口请求量 Top {}", self.top_n),
                description,
                affected_hosts: vec![],
                occurrence_count: by_count.iter().map(|(_, s)| s.count).sum(),
            });
        }

        // 慢接口 — avg duration > slow_threshold_ms
        let mut slow_endpoints: Vec<(&String, u64)> = stats
            .iter()
            .filter_map(|(endpoint, stat)| {
                stat.avg_duration_ms()
                    .filter(|&avg| avg > self.slow_threshold_ms)
                    .map(|avg| (endpoint, avg))
            })
            .collect();
        slow_endpoints.sort_by(|a, b| b.1.cmp(&a.1));

        if !slow_endpoints.is_empty() {
            let description = slow_endpoints
                .iter()
                .map(|(endpoint, avg_ms)| format!("{}: 平均响应 {}ms", endpoint, avg_ms))
                .collect::<Vec<_>>()
                .join("\n");

            issues.push(Issue {
                severity: Severity::Medium,
                title: "发现慢接口".to_string(),
                description,
                affected_hosts: vec![],
                occurrence_count: slow_endpoints.len() as u64,
            });
        }

        // 高错误率接口 — error_count / count > 10%
        let mut high_error_endpoints: Vec<(&String, &EndpointStat)> = stats
            .iter()
            .filter(|(_, stat)| stat.error_rate() > 0.1)
            .collect();
        high_error_endpoints.sort_by(|a, b| {
            b.1.error_rate()
                .partial_cmp(&a.1.error_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if !high_error_endpoints.is_empty() {
            let description = high_error_endpoints
                .iter()
                .map(|(endpoint, stat)| {
                    format!(
                        "{}: 错误率 {:.1}% ({}/{})",
                        endpoint,
                        stat.error_rate() * 100.0,
                        stat.error_count,
                        stat.count
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            issues.push(Issue {
                severity: Severity::High,
                title: "接口错误率过高".to_string(),
                description,
                affected_hosts: vec![],
                occurrence_count: high_error_endpoints
                    .iter()
                    .map(|(_, s)| s.error_count)
                    .sum(),
            });
        }

        issues
    }

    fn suggestions(&self, input: &AnalysisInput) -> Vec<Suggestion> {
        let stats = aggregate_stats(input);
        if stats.is_empty() {
            return vec![];
        }

        let has_slow = stats.values().any(|stat| {
            stat.avg_duration_ms()
                .map(|avg| avg > self.slow_threshold_ms)
                .unwrap_or(false)
        });

        let has_high_error = stats.values().any(|stat| stat.error_rate() > 0.1);

        let mut suggestions = Vec::new();

        if has_slow {
            suggestions.push(Suggestion {
                priority: Priority::High,
                title: "优化慢接口响应时间".to_string(),
                detail: "存在平均响应时间超过阈值的接口，建议排查数据库查询是否缺少索引或存在 N+1 查询，并对热点数据引入缓存层以降低响应延迟。".to_string(),
            });
        }

        if has_high_error {
            suggestions.push(Suggestion {
                priority: Priority::High,
                title: "降低接口错误率".to_string(),
                detail: "存在错误率超过 10% 的接口，建议审查错误处理逻辑、完善入参校验，并为下游依赖服务引入熔断器以防止级联故障。".to_string(),
            });
        }

        suggestions
    }
}
