pub fn extract_json_payload(content: &str) -> &str {
    let trimmed = content.trim();

    if let Some(stripped) = trimmed.strip_prefix("```json") {
        return stripped
            .trim()
            .strip_suffix("```")
            .map(str::trim)
            .unwrap_or_else(|| stripped.trim());
    }

    if let Some(stripped) = trimmed.strip_prefix("```") {
        return stripped
            .trim()
            .strip_suffix("```")
            .map(str::trim)
            .unwrap_or_else(|| stripped.trim());
    }

    trimmed
}
