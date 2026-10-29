//! One bounded, sanitized command record per metadata attempt, including failures
//! before npm starts. Never collect the manager's inherited environment.
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::infrastructure::platform::OpenOptionsModeExt;
use crate::modules::process::EnvironmentPolicy;
use crate::modules::runtime::install::{decode_command_output, Masker};

pub(super) struct QueryLog {
    file: std::fs::File,
    path: PathBuf,
    masker: Masker,
}

impl QueryLog {
    pub(super) fn create(
        command: &std::process::Command,
        directory: &Path,
        environment: &EnvironmentPolicy,
    ) -> Result<Self, AppError> {
        let masker = Masker::new(
            [&environment.http_proxy, &environment.https_proxy]
                .into_iter()
                .flatten()
                .filter_map(|value| reqwest::Url::parse(&value.to_string_lossy()).ok())
                .filter_map(|url| url.password().map(str::to_owned))
                .collect(),
        );
        let path = directory.join(format!("npm-view-{}.log", uuid::Uuid::new_v4()));
        let mut log = Self {
            file: (|| {
                std::fs::create_dir_all(directory)?;
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .create_mode(0o600)
                    .open(&path)
            })()
            .map_err(|error: std::io::Error| AppError::RemoteQuery {
                reason: "npm_view_log".into(),
                detail: masker.apply(&format!(
                    "无法建立 npm view 命令日志：{}；{error}",
                    path.display()
                )),
            })?,
            path,
            masker,
        };
        log.write(serde_json::json!({
            "startedAt": chrono::Utc::now().to_rfc3339(),
            "program": command.get_program().to_string_lossy(),
            "arguments": command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
            "workingDirectory": command.get_current_dir().map(|path| path.to_string_lossy()),
            "environment": command.get_envs().map(|(key, value)| (key.to_string_lossy(), value.map(|value| value.to_string_lossy()))).collect::<std::collections::BTreeMap<_,_>>(),
            "environmentCleared": true,
        }))?;
        Ok(log)
    }

    fn write(&mut self, mut record: serde_json::Value) -> Result<(), AppError> {
        mask_values(&mut record, &self.masker);
        writeln!(self.file, "{record}")
            .and_then(|_| self.file.flush())
            .map_err(|error| self.error("npm_view_log", &format!("npm view 日志写入失败：{error}")))
    }

    pub(super) fn finish(
        &mut self,
        stdout: &Capture,
        stderr: &Capture,
        exit: Option<std::process::ExitStatus>,
        failure: Option<(&str, &str)>,
    ) -> Result<(), AppError> {
        for (stream, capture) in [("stdout", stdout), ("stderr", stderr)] {
            self.write(
                serde_json::json!({
                    "stream": stream,
                    "text": capture.text(),
                    "capturedBytes": capture.bytes.len(),
                    "complete": capture.complete,
                    "partialRecordOmitted": !capture.complete && !capture.bytes.is_empty() && !capture.bytes.ends_with(b"\n"),
                }),
            )?;
        }
        self.write(serde_json::json!({
            "endedAt": chrono::Utc::now().to_rfc3339(),
            "exitCode": exit.and_then(|status| status.code()),
            "reason": failure.map(|(reason, _)| reason),
            "detail": failure.map(|(_, detail)| detail),
        }))
    }

    pub(super) fn error(&self, reason: &str, detail: &str) -> AppError {
        AppError::RemoteQuery {
            reason: reason.into(),
            detail: self.masker.apply(&format!(
                "{detail}\n日志：{}",
                dunce::simplified(&self.path).display()
            )),
        }
    }

    pub(super) fn output_detail(&self, stdout: &Capture, stderr: &Capture) -> String {
        let text = if stderr.bytes.is_empty() {
            stdout.text()
        } else {
            stderr.text()
        };
        // Mask the entire retained text before selecting a tail. A tail must not
        // separate URL userinfo from its scheme and bypass credential masking.
        let masked = self.masker.apply(&text);
        let lines = masked
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>();
        let summary = if lines.len() <= 10 {
            lines.join("\n")
        } else {
            format!(
                "{}\n…\n{}",
                lines[..4].join("\n"),
                lines[lines.len() - 6..].join("\n")
            )
        };
        summary.chars().take(1600).collect()
    }
}

fn mask_values(value: &mut serde_json::Value, masker: &Masker) {
    match value {
        serde_json::Value::String(text) => *text = masker.apply(text),
        serde_json::Value::Array(values) => {
            for value in values {
                mask_values(value, masker);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                mask_values(value, masker);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_output_omits_cut_credentials_and_does_not_drop_previous_records() {
        let capture = Capture {
            bytes: b"npm error before timeout\nhttps://user:partial-password".to_vec(),
            complete: false,
        };
        assert_eq!(capture.text(), "npm error before timeout\n");
        assert_eq!(
            Capture {
                complete: true,
                bytes: b"complete unterminated record".to_vec()
            }
            .text(),
            "complete unterminated record"
        );
    }

    #[test]
    fn masking_json_values_preserves_record_keys_and_json_escaping() {
        let masker = Masker::new(vec!["a".into(), "\"quoted\\secret".into()]);
        let mut record = serde_json::json!({
            "arguments": ["a", "\"quoted\\secret", "https://user:pass@registry.invalid/x"],
            "environment": {"PATH": "a", "HTTPS_PROXY": "https://user:pass@host.invalid"}
        });
        mask_values(&mut record, &masker);
        let roundtrip: serde_json::Value = serde_json::from_str(&record.to_string()).unwrap();
        assert_eq!(roundtrip["arguments"][0], "***");
        assert_eq!(roundtrip["arguments"][1], "***");
        assert_eq!(roundtrip["environment"]["PATH"], "***");
        assert!(!roundtrip.to_string().contains("user:"));
    }
}

#[derive(Default)]
pub(super) struct Capture {
    pub(super) bytes: Vec<u8>,
    pub(super) complete: bool,
}

impl Capture {
    fn text(&self) -> String {
        // Incomplete final records may contain cut credentials. Preserve only
        // complete lines on timeout/overflow; explicitly record that omission.
        let bytes = if self.complete {
            &self.bytes[..]
        } else {
            &self.bytes[..self
                .bytes
                .iter()
                .rposition(|byte| *byte == b'\n')
                .map_or(0, |i| i + 1)]
        };
        decode_command_output(bytes)
    }
}
