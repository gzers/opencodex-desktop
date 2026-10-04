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
    let mut buffer = Vec::new();
    let mut start = metadata.len();
    let mut has_incomplete_head = false;

    while buffer.iter().filter(|byte| **byte == b'\n').count() < max_lines && start > 0 {
        let chunk_size = start.min(64 * 1024);
        start -= chunk_size;
        let mut chunk = vec![0_u8; chunk_size as usize];
        file.seek(SeekFrom::Start(start))
            .map_err(|error| AppError::FileSystem {
                operation: "seek log".to_string(),
                detail: error.to_string(),
            })?;
        file.read_exact(&mut chunk)
            .map_err(|error| AppError::FileSystem {
                operation: "read log chunk".to_string(),
                detail: error.to_string(),
            })?;

        // 起点在文件中间时，首个片段可能是不完整记录，必须丢弃。
        let search_from = if start == 0 { 0 } else { 1 };
        let confirmed_chunk = &chunk[search_from..];
        if search_from == 1 && !confirmed_chunk.starts_with(b"\n") {
            has_incomplete_head = true;
        }

        let mut combined = Vec::with_capacity(confirmed_chunk.len() + buffer.len());
        combined.extend_from_slice(confirmed_chunk);
        combined.extend_from_slice(&buffer);
        buffer = combined;

        let newline_count = buffer.iter().filter(|byte| **byte == b'\n').count();
        if newline_count >= max_lines {
            break;
        }
    }

    let raw = String::from_utf8_lossy(&buffer);
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
        truncated: has_incomplete_head || dropped_old_lines > 0,
        file_missing: false,
    })
}

/// 逐条 FZ-14 脱敏：URL 与家目录、S3；返回 None 表示整行无法安全脱敏。
pub fn sanitize_line(line: &str) -> Option<String> {
    Some(sanitize_s3(&sanitize_urls_and_home(line)))
}

fn sanitize_urls_and_home(line: &str) -> String {
    let home = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from);
    let mut output = String::with_capacity(line.len());
    let mut rest = line;

    while let Some(position) = rest.find('h') {
        let candidate = &rest[position..];
        if let Some(home_path) = home.as_ref() {
            let home_text = home_path.to_string_lossy();
            if candidate.starts_with(home_text.as_ref())
                && rest[..position].ends_with(|character: char| {
                    character.is_whitespace()
                        || character == '"'
                        || character == '\''
                        || character == '['
                })
            {
                output.push('~');
                rest = &rest[home_text.len()..];
                continue;
            }
        }

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
}
