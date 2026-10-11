//! MOD-12：FZ-34 ~ FZ-40 CLI / IPC 请求契约。
//!
//! 本模块只冻结命令解析、确认、响应、幂等与审计字段；不创建 socket、
//! 不注册 PATH、不执行官方子进程。真实端点由后续 infrastructure 接入。

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod audit;
pub mod endpoint;
pub mod path_registration;
pub mod service;

pub const CONTRACT_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 1;
pub const AUDIT_SCHEMA_VERSION: u32 = 1;
pub const AUDIT_RETENTION_DAYS: i64 = 90;
pub const AUDIT_MAX_RECORDS: usize = 5_000;
pub const AUDIT_LOG_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const AUDIT_LOG_ROTATIONS: usize = 5;
pub const MAX_IPC_FRAME_BYTES: usize = 16 * 1024;
#[cfg(unix)]
pub const CLI_SOCKET_RELATIVE_PATH: &str =
    "Library/Caches/com.gzers.opencodex.desktop/ipc/opencodex.ipc";
pub const CLI_PATH_TARGET: &str = "/usr/local/bin/ocxd";

/// FZ-34 命令全集；未列命令不开放。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    Status,
    Start,
    Stop,
    Restart,
    DataRootShow,
    DataRootSwitch,
    BackupCreate,
    BackupList,
    Export,
    Import,
    SyncRun,
    UpdateCheck,
}

impl IpcCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::DataRootShow => "data-root show",
            Self::DataRootSwitch => "data-root switch",
            Self::BackupCreate => "backup create",
            Self::BackupList => "backup list",
            Self::Export => "export",
            Self::Import => "import",
            Self::SyncRun => "sync run",
            Self::UpdateCheck => "update check",
        }
    }

    pub fn requires_confirm(&self) -> bool {
        matches!(
            self,
            Self::Start
                | Self::Stop
                | Self::Restart
                | Self::DataRootSwitch
                | Self::BackupCreate
                | Self::Export
                | Self::Import
                | Self::SyncRun
        )
    }

    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            Self::Status | Self::DataRootShow | Self::BackupList | Self::UpdateCheck
        )
    }
}

/// FZ-36 错误码；与 FZ-11 包装层语义对应。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcErrorCode {
    UnknownCommand,
    ContractVersionMismatch,
    RequireConfirm,
    InstanceOffline,
    AccessDenied,
    TargetNotFound,
    TargetStateConflict,
    ValidationFailed,
    /// 旧版（v1）加密容器需要导出时的原口令；调用方据此提示输入，而不是当作普通失败。
    PassphraseRequired,
    ExecutionFailed,
    Cancelled,
    Timeout,
    InternalError,
}

impl IpcErrorCode {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::UnknownCommand | Self::ContractVersionMismatch => 2,
            Self::RequireConfirm => 4,
            Self::AccessDenied => 5,
            Self::TargetNotFound => 6,
            Self::TargetStateConflict => 7,
            Self::ValidationFailed => 8,
            Self::PassphraseRequired => 13,
            Self::ExecutionFailed => 9,
            Self::Cancelled => 10,
            Self::Timeout => 11,
            Self::InstanceOffline => 3,
            Self::InternalError => 12,
        }
    }
}

/// FZ-36 IPC 请求。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcRequest {
    pub request_id: String,
    pub command: IpcCommand,
    pub args: BTreeMap<String, String>,
    pub confirm: bool,
    pub contract_version: u32,
    /// 仅旧版（v1）加密容器导入时携带的临时口令。
    ///
    /// 不放进 `args`：`args` 会作为「键名」写进审计，且设计上禁止在其中放口令。
    /// 缺省时不参与序列化，请求形状保持不变；响应与审计都不回显它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

impl IpcRequest {
    pub fn validate(&self) -> Result<(), IpcErrorCode> {
        if self.contract_version != CONTRACT_VERSION {
            return Err(IpcErrorCode::ContractVersionMismatch);
        }
        if self.request_id.trim().is_empty() {
            return Err(IpcErrorCode::ValidationFailed);
        }
        if self.command.requires_confirm() && !self.confirm {
            return Err(IpcErrorCode::RequireConfirm);
        }
        Ok(())
    }
}

/// FZ-36 IPC 响应；不新增顶层字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcResponse<T> {
    pub request_id: String,
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<IpcError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IpcError {
    pub code: IpcErrorCode,
    pub message: String,
}

/// FZ-35 --json 输出契约。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CliJson<T> {
    pub schema_version: u32,
    pub command: IpcCommand,
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<IpcError>,
    pub generated_at: String,
}

/// 审计来源；只记录来源分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSource {
    Cli,
    Ipc,
    Gui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Succeeded,
    Failed,
    Cancelled,
}

/// FZ-40 审计记录；args 只保留键名。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditRecord {
    pub schema_version: u32,
    pub event_id: String,
    pub request_id: String,
    pub source: AuditSource,
    pub peer_uid: Option<u32>,
    pub command: IpcCommand,
    pub arg_keys: Vec<String>,
    pub confirm: bool,
    pub result: AuditResult,
    pub error_code: Option<IpcErrorCode>,
    pub started_at: String,
    pub finished_at: String,
}

impl AuditRecord {
    pub fn from_request(
        request: &IpcRequest,
        source: AuditSource,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        result: AuditResult,
        error_code: Option<IpcErrorCode>,
    ) -> Self {
        Self {
            schema_version: AUDIT_SCHEMA_VERSION,
            event_id: format!("evt_{}", uuid::Uuid::new_v4()),
            request_id: request.request_id.clone(),
            source,
            peer_uid: None,
            command: request.command,
            arg_keys: request.args.keys().cloned().collect(),
            confirm: request.confirm,
            result,
            error_code,
            started_at: started_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            finished_at: finished_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        }
    }
}

/// 简单命令解析器；只接受 FZ-34 命令表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandParseError {
    Unknown,
    RequireConfirm,
}

fn allowed_cli_flags(command: IpcCommand) -> (&'static [&'static str], &'static [&'static str]) {
    match command {
        IpcCommand::Status
        | IpcCommand::DataRootShow
        | IpcCommand::BackupList
        | IpcCommand::UpdateCheck => (&["--json"], &[]),
        IpcCommand::Start | IpcCommand::Stop | IpcCommand::Restart | IpcCommand::SyncRun => {
            (&["--confirm", "--json"], &[])
        }
        IpcCommand::BackupCreate => (&["--confirm", "--json"], &[]),
        IpcCommand::DataRootSwitch => (&["--confirm", "--json", "--migrate"], &["--target"]),
        IpcCommand::Export => (&["--confirm", "--json", "--password-stdin"], &["--output"]),
        IpcCommand::Import => (&["--confirm", "--json", "--password-stdin"], &["--input"]),
    }
}

pub fn parse_cli(argv: &[String]) -> Result<IpcCommand, CommandParseError> {
    let mut index = 1;
    let command = match argv.first().map(String::as_str) {
        Some("status") => IpcCommand::Status,
        Some("start") => IpcCommand::Start,
        Some("stop") => IpcCommand::Stop,
        Some("restart") => IpcCommand::Restart,
        Some("export") => IpcCommand::Export,
        Some("import") => IpcCommand::Import,
        Some("data-root") => match argv.get(1).map(String::as_str) {
            Some("show") => {
                index = 2;
                IpcCommand::DataRootShow
            }
            Some("switch") => {
                index = 2;
                IpcCommand::DataRootSwitch
            }
            _ => return Err(CommandParseError::Unknown),
        },
        Some("backup") => match argv.get(1).map(String::as_str) {
            Some("create") => {
                index = 2;
                IpcCommand::BackupCreate
            }
            Some("list") => {
                index = 2;
                IpcCommand::BackupList
            }
            _ => return Err(CommandParseError::Unknown),
        },
        Some("sync") => match argv.get(1).map(String::as_str) {
            Some("run") => {
                index = 2;
                IpcCommand::SyncRun
            }
            _ => return Err(CommandParseError::Unknown),
        },
        Some("update") if argv.get(1).map(String::as_str) == Some("check") => {
            index = 2;
            IpcCommand::UpdateCheck
        }
        _ => return Err(CommandParseError::Unknown),
    };

    let (flags, value_flags) = allowed_cli_flags(command);
    let mut confirmed = false;
    while index < argv.len() {
        let flag = argv[index].as_str();
        if flags.contains(&flag) {
            if flag == "--confirm" {
                confirmed = true;
            }
            index += 1;
        } else if value_flags.contains(&flag) {
            if argv.get(index + 1).is_none() || argv[index + 1].starts_with("--") {
                return Err(CommandParseError::Unknown);
            }
            index += 2;
        } else {
            return Err(CommandParseError::Unknown);
        }
    }
    if command.requires_confirm() && !confirmed {
        return Err(CommandParseError::RequireConfirm);
    }
    Ok(command)
}

/// 生成成功 / 失败 JSON；调用方传入 UTC 时间保证幂等可测。
pub fn cli_json<T>(
    command: IpcCommand,
    data: Option<T>,
    error: Option<IpcError>,
    generated_at: DateTime<Utc>,
) -> CliJson<T> {
    CliJson {
        schema_version: SCHEMA_VERSION,
        command,
        ok: error.is_none(),
        data,
        error,
        generated_at: generated_at.to_rfc3339_opts(SecondsFormat::Secs, true),
    }
}

/// 响应 helper：拒绝 contract_version / confirm / request_id 错误。
pub fn validate_request<T>(request: IpcRequest) -> Result<IpcRequest, IpcResponse<T>> {
    match request.validate() {
        Ok(()) => Ok(request),
        Err(code) => Err(IpcResponse {
            request_id: request.request_id,
            ok: false,
            data: None,
            error: Some(IpcError {
                code,
                message: error_message(code).to_string(),
            }),
        }),
    }
}

pub fn error_message(code: IpcErrorCode) -> &'static str {
    match code {
        IpcErrorCode::UnknownCommand => "Unknown command.",
        IpcErrorCode::ContractVersionMismatch => "Contract version mismatch.",
        IpcErrorCode::RequireConfirm => "This action requires --confirm.",
        IpcErrorCode::InstanceOffline => "The running instance is offline.",
        IpcErrorCode::AccessDenied => "Access denied.",
        IpcErrorCode::TargetNotFound => "Target not found.",
        IpcErrorCode::TargetStateConflict => "Target state conflict.",
        IpcErrorCode::ValidationFailed => "Validation failed.",
        IpcErrorCode::PassphraseRequired => "Legacy container requires the original passphrase.",
        IpcErrorCode::ExecutionFailed => "Execution failed.",
        IpcErrorCode::Cancelled => "Cancelled.",
        IpcErrorCode::Timeout => "Timed out.",
        IpcErrorCode::InternalError => "Internal error.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn args(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn request(command: IpcCommand, confirm: bool) -> IpcRequest {
        IpcRequest {
            request_id: "req_00000000-0000-0000-0000-000000000001".to_string(),
            command,
            args: BTreeMap::new(),
            confirm,
            contract_version: 1,
            secret: None,
        }
    }

    #[test]
    fn frozen_command_set_matches_fz34() {
        let commands = [
            IpcCommand::Status,
            IpcCommand::Start,
            IpcCommand::Stop,
            IpcCommand::Restart,
            IpcCommand::DataRootShow,
            IpcCommand::DataRootSwitch,
            IpcCommand::BackupCreate,
            IpcCommand::BackupList,
            IpcCommand::Export,
            IpcCommand::Import,
            IpcCommand::SyncRun,
            IpcCommand::UpdateCheck,
        ];
        assert_eq!(commands.len(), 12);
        assert!(IpcCommand::Start.requires_confirm());
        assert!(IpcCommand::SyncRun.requires_confirm());
        assert!(IpcCommand::Status.is_read_only());
        assert!(IpcCommand::UpdateCheck.is_read_only());
    }

    #[test]
    fn confirm_false_is_rejected_with_exit_code_four() {
        let invalid = request(IpcCommand::Start, false);
        assert_eq!(invalid.validate(), Err(IpcErrorCode::RequireConfirm));
        assert_eq!(IpcErrorCode::RequireConfirm.exit_code(), 4);

        let valid = request(IpcCommand::Start, true);
        assert_eq!(valid.validate(), Ok(()));
    }

    #[test]
    fn contract_version_must_match_one() {
        let mut invalid = request(IpcCommand::Status, false);
        invalid.contract_version = 2;
        assert_eq!(
            invalid.validate(),
            Err(IpcErrorCode::ContractVersionMismatch)
        );
        assert_eq!(IpcErrorCode::ContractVersionMismatch.exit_code(), 2);
    }

    #[test]
    fn cli_parser_only_accepts_frozen_commands() {
        let argv = vec!["status".to_string()];
        assert_eq!(parse_cli(&argv), Ok(IpcCommand::Status));
        assert_eq!(
            parse_cli(&["start".to_string()]),
            Err(CommandParseError::RequireConfirm)
        );
        assert_eq!(
            parse_cli(&["start".to_string(), "--confirm".to_string()]),
            Ok(IpcCommand::Start)
        );
        assert_eq!(
            parse_cli(&["update".to_string(), "check".to_string()]),
            Ok(IpcCommand::UpdateCheck)
        );
        assert_eq!(
            parse_cli(&["provider".to_string(), "write".to_string()]),
            Err(CommandParseError::Unknown)
        );
    }

    #[test]
    fn cli_json_has_no_extra_top_level_fields() {
        let now = Utc.with_ymd_and_hms(2026, 9, 15, 3, 4, 5).unwrap();
        let response = cli_json::<BTreeMap<String, String>>(
            IpcCommand::Status,
            Some(BTreeMap::new()),
            None,
            now,
        );
        let payload = serde_json::to_value(&response).unwrap();
        assert_eq!(
            payload
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            [
                "command",
                "data",
                "error",
                "generated_at",
                "ok",
                "schema_version"
            ]
        );
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.generated_at, "2026-09-15T03:04:05Z");
    }

    #[test]
    fn failed_response_keeps_sanitized_error_and_exit_code() {
        let rejection = validate_request::<()>(request(IpcCommand::Start, false)).unwrap_err();
        assert!(!rejection.ok);
        assert_eq!(
            rejection.error.as_ref().unwrap().code,
            IpcErrorCode::RequireConfirm
        );
        assert_eq!(
            rejection.error.unwrap().message,
            "This action requires --confirm."
        );
    }

    #[test]
    fn audit_record_never_captures_argument_values() {
        let started = Utc.with_ymd_and_hms(2026, 9, 15, 3, 0, 0).unwrap();
        let finished = started + chrono::Duration::seconds(1);
        let mut request = request(IpcCommand::DataRootSwitch, true);
        request.args = args(&[("target", "/private/fixture/root"), ("password", "secret")]);
        let record = AuditRecord::from_request(
            &request,
            AuditSource::Cli,
            started,
            finished,
            AuditResult::Succeeded,
            None,
        );
        let mut expected_arg_keys = vec!["target", "password"];
        expected_arg_keys.sort_unstable();
        assert_eq!(record.arg_keys, expected_arg_keys);
        assert_eq!(record.schema_version, 1);
        assert_eq!(record.result, AuditResult::Succeeded);
        assert!(record.error_code.is_none());
        let payload = serde_json::to_string(&record).unwrap();
        assert!(!payload.contains("/private/fixture/root"));
        assert!(!payload.contains("secret"));
    }

    #[test]
    fn audit_retention_matches_fz40() {
        assert_eq!(AUDIT_RETENTION_DAYS, 90);
        assert_eq!(AUDIT_MAX_RECORDS, 5_000);
        assert_eq!(AUDIT_LOG_MAX_BYTES, 5 * 1024 * 1024);
        assert_eq!(AUDIT_LOG_ROTATIONS, 5);
    }

    #[test]
    fn ipc_response_shape_is_frozen() {
        let response = IpcResponse {
            request_id: "req".to_string(),
            ok: true,
            data: Some(()),
            error: None,
        };
        let payload = serde_json::to_value(&response).unwrap();
        assert_eq!(
            payload
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            ["data", "error", "ok", "request_id"]
        );
    }
}
