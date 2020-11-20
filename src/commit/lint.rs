use regex::Regex;

/// 单条 lint 违规记录。
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// 关联标签（通常为 commit hash 前缀）。
    pub label: String,
    /// 违规说明。
    pub detail: String,
}

/// 按 leetcode.v commit 规范校验完整 message。
pub fn lint_message(raw: &str, label: &str) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let subject = lines.first().map(|s| s.trim()).unwrap_or("");
    let body_joined = if lines.len() > 1 { lines[1..].join("\n") } else { String::new() };
    let body = body_joined.trim();
    let full = if body.is_empty() { subject.to_string() } else { format!("{subject}\n{body}") };

    if subject.is_empty() {
        issues.push(LintIssue { label: label.to_string(), detail: "subject is empty".into() });
        return issues;
    }

    let emoji = Regex::new(r"^\p{Extended_Pictographic}").expect("emoji regex");
    // subject 须以 gitmoji 开头
    if !emoji.is_match(subject) {
        issues.push(LintIssue { label: label.to_string(), detail: "subject must start with a gitmoji character".into() });
    }
    if subject.ends_with('.') {
        issues.push(LintIssue { label: label.to_string(), detail: "subject must not end with a period".into() });
    }
    if full.contains(';') || full.contains('；') {
        issues.push(LintIssue { label: label.to_string(), detail: "must not contain semicolons".into() });
    }
    let milestone = Regex::new(r"(?i)\b(phase\s*[-_]?\s*\d|m\d|s\d|a\d|f\d|gate[- ]?\d)\b").expect("milestone");
    if milestone.is_match(&full) {
        issues.push(LintIssue { label: label.to_string(), detail: "must not contain internal milestone codes".into() });
    }
    let bare_ts = Regex::new(r"\bTS\b(?![a-z])").expect("ts regex");
    if bare_ts.is_match(&full) {
        issues.push(LintIssue { label: label.to_string(), detail: "write TypeScript in full instead of bare TS".into() });
    }
    let version = Regex::new(r"\b0\.\d+\.\d+\b").expect("version regex");
    if version.is_match(&full) {
        issues.push(LintIssue { label: label.to_string(), detail: "avoid version numbers in commit messages".into() });
    }
    issues
}

/// 检测 subject 重复项，返回可读描述行。
pub fn duplicate_subjects(subjects: &[(String, String)]) -> Vec<String> {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (_, subject) in subjects {
        *counts.entry(subject.as_str()).or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(subject, count)| format!("duplicate subject ({count}x): {subject}"))
        .collect()
}
