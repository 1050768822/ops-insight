use regex::Regex;
use std::sync::OnceLock;

struct RedactionRule {
    regex: Regex,
    replacement: &'static str,
}

fn rules() -> &'static [RedactionRule] {
    static RULES: OnceLock<Vec<RedactionRule>> = OnceLock::new();
    RULES
        .get_or_init(|| {
            vec![
                RedactionRule {
                    regex: Regex::new(r"sk-ant-[A-Za-z0-9\-]{20,}")
                        .expect("valid anthropic key regex"),
                    replacement: "[REDACTED_ANTHROPIC_KEY]",
                },
                RedactionRule {
                    regex: Regex::new(r"sk-[A-Za-z0-9]{20,}").expect("valid openai key regex"),
                    replacement: "[REDACTED_OPENAI_KEY]",
                },
                RedactionRule {
                    regex: Regex::new(r"NRAK-[A-Za-z0-9]{20,}").expect("valid new relic key regex"),
                    replacement: "[REDACTED_NEWRELIC_KEY]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(bearer\s+)[A-Za-z0-9\-._~+/]+=*")
                        .expect("valid bearer regex"),
                    replacement: "${1}[REDACTED_TOKEN]",
                },
                RedactionRule {
                    regex: Regex::new(r"eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+")
                        .expect("valid jwt regex"),
                    replacement: "[REDACTED_JWT]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(password\s*[=:]\s*)\S+")
                        .expect("valid password regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(passwd\s*[=:]\s*)\S+").expect("valid passwd regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(pwd\s*[=:]\s*)\S+").expect("valid pwd regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(secret\s*[=:]\s*)\S+").expect("valid secret regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(api[_-]?key\s*[=:]\s*)\S+")
                        .expect("valid api key regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(connectionstring\s*[=:]\s*)\S+")
                        .expect("valid connection string regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"(?i)(Server=[^;]{1,100};(?:[^;]*;){0,5}Password=)[^;\\s]+")
                        .expect("valid database password regex"),
                    replacement: "${1}[REDACTED]",
                },
                RedactionRule {
                    regex: Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}")
                        .expect("valid email regex"),
                    replacement: "[REDACTED_EMAIL]",
                },
            ]
        })
        .as_slice()
}

pub fn redact_text(input: &str) -> String {
    let mut redacted = input.to_string();
    for rule in rules() {
        redacted = rule
            .regex
            .replace_all(&redacted, rule.replacement)
            .into_owned();
    }
    redacted
}

pub fn redact_for_display(input: &str, max_chars: usize) -> String {
    let redacted = redact_text(input);
    let truncated: String = redacted.chars().take(max_chars).collect();
    if redacted.chars().count() > max_chars {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::{redact_for_display, redact_text};

    #[test]
    fn redacts_common_secret_patterns() {
        let source = "Authorization: Bearer eyJabc.def.ghi password=abc123 api_key=sk-abcdefghijklmnopqrstuvwxyz";
        let redacted = redact_text(source);
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("eyJabc.def.ghi"));
        assert!(!redacted.contains("sk-abcdefghijklmnopqrstuvwxyz"));
        assert!(redacted.contains("Bearer [REDACTED_TOKEN]"));
    }

    #[test]
    fn redacts_and_truncates_for_display() {
        let source = "user=alice@example.com password=abc123";
        let display = redact_for_display(source, 24);
        assert!(display.contains("[REDACTED_EMAI"));
        assert!(!display.contains("abc123"));
    }
}
