const SECRET_KEY_MARKERS: &[&str] = &[
    "PASSWORD",
    "PASSWD",
    "SECRET",
    "TOKEN",
    "API_KEY",
    "APIKEY",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "AUTH_KEY",
    "CREDENTIAL",
];

const SECRET_FLAGS: &[&str] = &[
    "--password",
    "--passwd",
    "--secret",
    "--token",
    "--api-key",
    "--apikey",
    "--access-token",
    "--auth-token",
];

/// Normalizes a command before persistence.
/// Returns `None` for empty commands or commands starting with whitespace
/// (following the `HIST_IGNORE_SPACE` convention).
pub fn prepare_command(raw: &str) -> Option<String> {
    if raw.trim().is_empty() || raw.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }

    Some(redact_secrets(raw))
}

fn is_secret_key(key: &str) -> bool {
    let upper = key.trim_matches(|c| c == '"' || c == '\'').to_ascii_uppercase();
    SECRET_KEY_MARKERS.iter().any(|marker| upper.contains(marker))
}

fn redact_assignment(token: &str) -> Option<String> {
    let eq_idx = token.find('=')?;
    let (key, rest) = token.split_at(eq_idx);
    let value = &rest[1..];
    if key.is_empty() || value.is_empty() {
        return None;
    }

    if key.starts_with('-') {
        let lower = key.to_ascii_lowercase();
        if SECRET_FLAGS.contains(&lower.as_str()) {
            return Some(format!("{key}=[REDACTED]"));
        }
        return None;
    }

    let valid_env_name = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if valid_env_name && is_secret_key(key) {
        return Some(format!("{key}=[REDACTED]"));
    }

    None
}

fn redact_secrets(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }

    let mut out = Vec::with_capacity(parts.len());
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];
        let lower = part.to_ascii_lowercase();

        if SECRET_FLAGS.contains(&lower.as_str()) {
            out.push(part.to_string());
            if i + 1 < parts.len() && !parts[i + 1].starts_with('-') {
                out.push("[REDACTED]".to_string());
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if lower == "bearer" && i + 1 < parts.len() {
            out.push(part.to_string());
            let next = parts[i + 1];
            let suffix = if next.ends_with('"') {
                "\""
            } else if next.ends_with('\'') {
                "'"
            } else {
                ""
            };
            out.push(format!("[REDACTED]{suffix}"));
            i += 2;
            continue;
        }

        if let Some(redacted) = redact_assignment(part) {
            out.push(redacted);
        } else {
            out.push(part.to_string());
        }

        i += 1;
    }

    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::prepare_command;

    #[test]
    fn skips_empty_and_leading_space_commands() {
        assert_eq!(prepare_command(""), None);
        assert_eq!(prepare_command("   "), None);
        assert_eq!(prepare_command(" secret-command"), None);
        assert_eq!(prepare_command("\texport FOO=bar"), None);
    }

    #[test]
    fn keeps_normal_commands_and_flags_intact() {
        assert_eq!(
            prepare_command("docker run -p 8080:80 nginx"),
            Some("docker run -p 8080:80 nginx".to_string())
        );
        assert_eq!(
            prepare_command("python eval.py --token-count 512"),
            Some("python eval.py --token-count 512".to_string())
        );
    }

    #[test]
    fn redacts_env_secrets_flags_and_bearer_tokens() {
        assert_eq!(
            prepare_command("OPENAI_API_KEY=sk-123456 python app.py"),
            Some("OPENAI_API_KEY=[REDACTED] python app.py".to_string())
        );
        assert_eq!(
            prepare_command("login --password hunter2 --user alice"),
            Some("login --password [REDACTED] --user alice".to_string())
        );
        assert_eq!(
            prepare_command("login --token=ghp_abc123"),
            Some("login --token=[REDACTED]".to_string())
        );
        assert_eq!(
            prepare_command("curl -H \"Authorization: Bearer my-secret-jwt\" https://example.com"),
            Some("curl -H \"Authorization: Bearer [REDACTED]\" https://example.com".to_string())
        );
    }
}
