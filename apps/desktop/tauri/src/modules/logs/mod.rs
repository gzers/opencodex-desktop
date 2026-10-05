//! FZ-14 / FZ-15 只读日志模块。
//!
//! 只负责最近行读取和读取时脱敏；写入、轮转、保留清理不属于本模块。

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

pub fn default_recent_lines() -> usize {
    crate::modules::runtime_defaults::recent_log_lines()
}
pub const APP_LOG_FILE_NAME: &str = "app.log";
pub const AGENT_LOG_FILE_NAME: &str = "agent.log";
pub const AUDIT_LOG_FILE_NAME: &str = "audit.log";
pub const SYNC_LOG_FILE_NAME: &str = "sync.log";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFileKind {
    App,
    Agent,
    Audit,
    Sync,
}

impl LogFileKind {
    pub fn file_name(self) -> &'static str {
        match self {
            Self::App => APP_LOG_FILE_NAME,
            Self::Agent => AGENT_LOG_FILE_NAME,
            Self::Audit => AUDIT_LOG_FILE_NAME,
            Self::Sync => SYNC_LOG_FILE_NAME,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadRecentResult {
    pub lines: Vec<String>,
    pub redacted_line_count: usize,
    pub truncated: bool,
    pub file_missing: bool,
}

/// 读取前先按尾部切块，避免在读取过程中读入无界内容。
pub fn read_recent(path: impl AsRef<Path>, max_lines: usize) -> Result<ReadRecentResult, AppError> {
    if max_lines == 0 {
        return Ok(ReadRecentResult {
            lines: Vec::new(),
            redacted_line_count: 0,
            truncated: false,
            file_missing: false,
        });
    }

    let path = path.as_ref();
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ReadRecentResult {
                lines: Vec::new(),
                redacted_line_count: 0,
                truncated: false,
                file_missing: true,
            });
        }
        Err(error) => {
            return Err(AppError::FileSystem {
                operation: "read log".to_string(),
                detail: error.to_string(),
            })
        }
    };

    let mut file = File::open(path).map_err(|error| AppError::FileSystem {
        operation: "open log".to_string(),
        detail: error.to_string(),
    })?;
    // Read at most one bounded tail, including one byte to identify a partial
    // first record. Never return a cut fragment that could bypass redaction.
    let limit = crate::modules::runtime_defaults::recent_log_max_bytes() as u64;
    let start = metadata.len().saturating_sub(limit);
    let offset = start.saturating_sub(1);
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| AppError::FileSystem {
            operation: "seek log".into(),
            detail: error.to_string(),
        })?;
    let mut buffer = Vec::new();
    file.take(metadata.len() - offset)
        .read_to_end(&mut buffer)
        .map_err(|error| AppError::FileSystem {
            operation: "read log tail".into(),
            detail: error.to_string(),
        })?;
    let has_incomplete_head = start > 0 && buffer.first() != Some(&b'\n');
    let tail = if start > 0 {
        &buffer[buffer.len().min(1)..]
    } else {
        &buffer[..]
    };
    let raw = String::from_utf8_lossy(tail);
    let mut parsed_lines: Vec<&str> = if has_incomplete_head {
        raw.lines().skip(1).collect()
    } else {
        raw.lines().collect()
    };
    let mut dropped_old_lines = 0_usize;
    if parsed_lines.len() > max_lines {
        dropped_old_lines = parsed_lines.len() - max_lines;
        let skip = parsed_lines.len() - max_lines;
        parsed_lines.drain(..skip);
    }

    let mut lines = Vec::with_capacity(parsed_lines.len());
    let redacted_line_count = 0_usize;
    for line in parsed_lines {
        match sanitize_line(line) {
            Some(line) => lines.push(line),
            None => return Err(AppError::LogSanitization),
        }
    }

    Ok(ReadRecentResult {
        lines,
        redacted_line_count,
        truncated: start > 0 || dropped_old_lines > 0,
        file_missing: false,
    })
}

/// 逐条 FZ-14 脱敏：URL 与家目录、S3；返回 None 表示整行无法安全脱敏。
pub fn sanitize_line(line: &str) -> Option<String> {
    Some(sanitize_s3(&sanitize_urls_and_home(line)))
}

fn sanitize_urls_and_home(line: &str) -> String {
    let home = crate::infrastructure::platform::home_dir();
    let redacted = home
        .as_deref()
        .map(|home| redact_home_path(line, home))
        .unwrap_or_else(|| line.to_string());
    let mut output = String::with_capacity(line.len());
    let mut rest = redacted.as_str();

    while let Some(position) = rest.find('h') {
        let candidate = &rest[position..];
        if candidate.starts_with("http://") || candidate.starts_with("https://") {
            let scheme_len = if candidate.starts_with("http://") {
                "http://".len()
            } else {
                "https://".len()
            };
            let after_scheme = &candidate[scheme_len..];
            let end = after_scheme
                .find(|character: char| character.is_whitespace() || character == '"')
                .unwrap_or(after_scheme.len());
            let target = &after_scheme[..end];
            let authority_end = target.find(['/', '?']).unwrap_or(target.len());
            let after_authority = &target[authority_end..];
            let userinfo_end = target[..authority_end].find('@');
            let authority = match userinfo_end {
                Some(at) => &target[at + 1..authority_end],
                None => &target[..authority_end],
            };

            output.push_str(&rest[..position]);
            output.push_str(&candidate[..scheme_len]);
            output.push_str(authority);
            if let Some(query) = after_authority.find('?') {
                output.push_str(&after_authority[..query]);
            } else {
                output.push_str(after_authority);
            }
            rest = &rest[position + scheme_len + end..];
            continue;
        }

        output.push_str(&rest[..=position]);
        rest = &rest[position + 1..];
    }
    output.push_str(rest);
    output
}

fn redact_home_path(line: &str, home: &Path) -> String {
    let home = home.to_string_lossy();
    if home.is_empty() {
        return line.to_string();
    }
    let mut output = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(position) = rest.find(home.as_ref()) {
        let before = &rest[..position];
        let after = &rest[position + home.len()..];
        let boundary_before = before.is_empty()
            || before.ends_with(|ch: char| {
                ch.is_whitespace() || matches!(ch, '=' | '"' | '\'' | '[' | '(')
            });
        let boundary_after = after.is_empty()
            || after.starts_with(|ch: char| {
                ch.is_whitespace() || matches!(ch, '/' | '\\' | '"' | '\'' | ']' | ')' | ',')
            });
        output.push_str(before);
        output.push_str(if boundary_before && boundary_after {
            "~"
        } else {
            home.as_ref()
        });
        rest = after;
    }
    output.push_str(rest);
    output
}

fn sanitize_s3(line: &str) -> String {
    // 单遍游标替换：替换结果不会作为新的搜索输入，避免替换文本再次命中 needle。
    let mut output = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = find_case_insensitive(rest, "s3://") {
        output.push_str(&rest[..start]);
        let after = &rest[start + "s3://".len()..];
        let end = after
            .find(|character: char| character.is_whitespace() || character == '"')
            .unwrap_or(after.len());
        output.push_str("[REDACTED]");
        rest = &after[end..];
    }
    output.push_str(rest);

    let mut result = String::with_capacity(output.len());
    let mut rest = output.as_str();
    while let Some(start) = find_case_insensitive(rest, "bucket=") {
        result.push_str(&rest[..start]);
        let after = &rest[start + "bucket=".len()..];
        let end = after
            .find(|character: char| character.is_whitespace() || character == '"')
            .unwrap_or(after.len());
        result.push_str("bucket=[REDACTED]");
        rest = &after[end..];
    }
    result.push_str(rest);
    result
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_home_redaction_handles_windows_paths_and_keeps_other_users() {
        let home = Path::new(r"C:\Users\tester");
        assert_eq!(
            redact_home_path(r"cwd=C:\Users\tester\AppData file", home),
            r"cwd=~\AppData file"
        );
        assert_eq!(
            redact_home_path(r"[C:\Users\tester] C:\Users\tester2\file", home),
            r"[~] C:\Users\tester2\file"
        );
        assert_eq!(
            redact_home_path("home=/Users/tester/config", Path::new("/Users/tester")),
            "home=~/config"
        );
    }

    #[test]
    fn redacts_s3_uri_and_bucket_values() {
        assert_eq!(
            sanitize_line("upload s3://private-bucket/path failed").as_deref(),
            Some("upload [REDACTED] failed")
        );
        assert_eq!(
            sanitize_line("target Bucket=private-bucket done").as_deref(),
            Some("target bucket=[REDACTED] done")
        );
    }

    #[test]
    fn redacts_url_query_and_userinfo() {
        assert_eq!(
            sanitize_line("call https://user:token@api.example.com/v1?api_key=secret ok")
                .as_deref(),
            Some("call https://api.example.com/v1 ok")
        );
    }

    #[test]
    fn preserves_safe_text() {
        assert_eq!(
            sanitize_line("normal log line").as_deref(),
            Some("normal log line")
        );
    }
    #[test]
    fn bounded_tail_drops_partial_sensitive_and_utf8_records() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("large.log");
        let limit = crate::modules::runtime_defaults::recent_log_max_bytes();
        std::fs::write(
            &path,
            format!("secret={}\n完整记录\nlast record\n", "x".repeat(limit + 13)),
        )
        .unwrap();
        let result = read_recent(&path, 100).unwrap();
        assert!(result.truncated);
        assert_eq!(result.lines, vec!["完整记录", "last record"]);
        std::fs::write(&path, "x".repeat(limit + 1)).unwrap();
        let result = read_recent(&path, 100).unwrap();
        assert!(result.truncated);
        assert!(result.lines.is_empty());
        std::fs::write(&path, format!("old\n{}", "a".repeat(limit))).unwrap();
        assert_eq!(
            read_recent(&path, 100).unwrap().lines[0].len(),
            limit,
            "newline boundary keeps full record"
        );
    }
}
