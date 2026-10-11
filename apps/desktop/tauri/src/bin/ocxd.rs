//! 默认关闭的 `ocxd` CLI 客户端。
//!
//! 只连接运行中 GUI 冻结的本机 IPC 端点；Unix 使用 socket，Windows 使用 named pipe。
//! 凭据通过 stdin 输入，
//! 不出现在 argv、日志或审计记录中。
//!
//! 两个平台共用请求、响应和 frame 语义；只有传输端点按平台选择。

use std::collections::BTreeMap;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::PathBuf;
use std::process::ExitCode;
use zeroize::Zeroize;

#[cfg(unix)]
use opencodex_desktop_lib::modules::ipc::CLI_SOCKET_RELATIVE_PATH;
use opencodex_desktop_lib::modules::ipc::{
    cli_json, parse_cli, CommandParseError, IpcCommand, IpcError, IpcErrorCode, IpcRequest,
    IpcResponse, CONTRACT_VERSION,
};

const HELP: &str = "Usage: ocxd <command> [options]\n\nCommands:\n  status\n  start --confirm\n  stop --confirm\n  restart --confirm\n  data-root show\n  data-root switch --target <path> [--migrate] --confirm\n  backup create --confirm\n  backup list\n  export --output <path> --confirm\n  import --input <path> [--password-stdin] --confirm\n  sync run --confirm\n  update check\n\nOptions:\n  --json              Emit schema-v1 JSON\n  --password-stdin    Read the legacy container passphrase from stdin (only for v1 containers)\n  --help              Show this help\n";

#[derive(Debug)]
enum CliOutcome {
    Help,
    Request(IpcRequest),
    Error(IpcErrorCode),
}

fn build_request(
    command: IpcCommand,
    args: &[String],
    request_id: String,
) -> Result<IpcRequest, IpcErrorCode> {
    let mut map = BTreeMap::new();
    let flags: Vec<&String> = args.iter().collect();
    for flag in flags {
        match flag.as_str() {
            "--confirm" | "--json" | "--password-stdin" => {}
            // `--migrate` 此前被直接吞掉：用户要求「迁移数据」，实际只做了
            // 引用切换，而且回执还写「Data root switched」，行为与声明不符。
            // 明确转发给 IPC；数据迁移本身未启用，会得到显式错误而不是静默降级。
            "--migrate" => {
                map.insert("migrate".to_string(), "true".to_string());
            }
            "--target" | "--output" | "--input" => {
                let index = args
                    .iter()
                    .position(|value| value == flag)
                    .expect("flag index");
                let value = args.get(index + 1).ok_or(IpcErrorCode::ValidationFailed)?;
                map.insert(flag.trim_start_matches('-').to_string(), value.clone());
            }
            other if other.starts_with("--") => return Err(IpcErrorCode::UnknownCommand),
            _ => {}
        }
    }
    if matches!(command, IpcCommand::DataRootSwitch) && !map.contains_key("target") {
        return Err(IpcErrorCode::ValidationFailed);
    }
    if matches!(command, IpcCommand::Export) && !map.contains_key("output")
        || matches!(command, IpcCommand::Import) && !map.contains_key("input")
    {
        return Err(IpcErrorCode::ValidationFailed);
    }
    if command.requires_confirm() && !args.iter().any(|value| value == "--confirm") {
        return Err(IpcErrorCode::RequireConfirm);
    }
    Ok(IpcRequest {
        request_id,
        command,
        args: map,
        confirm: args.iter().any(|value| value == "--confirm"),
        contract_version: CONTRACT_VERSION,
        secret: None,
    })
}

fn parse(argv: &[String], request_id: String) -> CliOutcome {
    if argv.iter().any(|value| value == "--help") {
        return CliOutcome::Help;
    }
    let command = match parse_cli(argv) {
        Ok(value) => value,
        Err(CommandParseError::RequireConfirm) => {
            return CliOutcome::Error(IpcErrorCode::RequireConfirm)
        }
        Err(CommandParseError::Unknown) => return CliOutcome::Error(IpcErrorCode::UnknownCommand),
    };
    match build_request(command, argv, request_id) {
        Ok(request) => CliOutcome::Request(request),
        Err(code) => CliOutcome::Error(code),
    }
}

#[cfg(unix)]
fn socket_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    Some(home.join(CLI_SOCKET_RELATIVE_PATH))
}

trait ReadWrite: Read + Write {}

impl<T> ReadWrite for T where T: Read + Write {}

fn send_request<T: ReadWrite>(
    mut stream: T,
    mut request: IpcRequest,
    password_stdin: bool,
) -> Result<IpcResponse<serde_json::Value>, IpcErrorCode> {
    // 只有显式给出 `--password-stdin` 时才读取标准输入：新版明文容器不需要口令。
    //
    // 口令必须随请求送进「正在运行的那个进程」：CLI 与 GUI 是不同进程，
    // 此前只在 CLI 自己的环境里设 OCXD_PASSPHRASE，应用侧永远读不到，
    // 于是旧版容器根本无法经 CLI 导入。
    if password_stdin && matches!(request.command, IpcCommand::Import) {
        let mut passphrase = String::new();
        std::io::stdin()
            .read_line(&mut passphrase)
            .map_err(|_| IpcErrorCode::ValidationFailed)?;
        let cleaned = passphrase.trim_end_matches(['\n', '\r']).to_string();
        passphrase.zeroize();
        request.secret = Some(cleaned);
    }
    let payload = serde_json::to_vec(&request).map_err(|_| IpcErrorCode::InternalError)?;
    stream
        .write_all(&(payload.len() as u64).to_be_bytes())
        .map_err(|_| IpcErrorCode::InternalError)?;
    stream
        .write_all(&payload)
        .map_err(|_| IpcErrorCode::InternalError)?;
    drop(payload);
    stream.flush().map_err(|_| IpcErrorCode::InternalError)?;
    let mut length = [0u8; 8];
    stream
        .read_exact(&mut length)
        .map_err(|_| IpcErrorCode::InternalError)?;
    let length =
        usize::try_from(u64::from_be_bytes(length)).map_err(|_| IpcErrorCode::InternalError)?;
    if length > 1024 * 1024 {
        return Err(IpcErrorCode::InternalError);
    }
    let mut response_bytes = vec![0u8; length];
    stream
        .read_exact(&mut response_bytes)
        .map_err(|_| IpcErrorCode::InternalError)?;
    serde_json::from_slice(&response_bytes).map_err(|_| IpcErrorCode::InternalError)
}

#[cfg(unix)]
fn connect_stream() -> Result<UnixStream, IpcErrorCode> {
    let path = socket_path().ok_or(IpcErrorCode::InstanceOffline)?;
    UnixStream::connect(path).map_err(|_| IpcErrorCode::InstanceOffline)
}

#[cfg(windows)]
fn connect_stream() -> Result<std::fs::File, IpcErrorCode> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(opencodex_desktop_lib::modules::ipc::endpoint::pipe_name())
        .map_err(|_| IpcErrorCode::InstanceOffline)
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let request_id = format!("req_{}", uuid::Uuid::new_v4());
    match parse(&argv, request_id) {
        CliOutcome::Help => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        CliOutcome::Error(code) => {
            eprintln!("{code:?}");
            ExitCode::from(code.exit_code() as u8)
        }
        CliOutcome::Request(request) => {
            let command = request.command;
            let password_stdin = argv.iter().any(|value| value == "--password-stdin");
            match connect_stream().and_then(|stream| send_request(stream, request, password_stdin))
            {
                Ok(response) => {
                    if argv.iter().any(|value| value == "--json") {
                        let payload = cli_json(
                            command,
                            response.data,
                            response.error.clone(),
                            chrono::Utc::now(),
                        );
                        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
                    } else if let Some(error) = &response.error {
                        eprintln!("{}", error.message);
                    } else if let Some(data) = &response.data {
                        println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
                    }
                    match response.error {
                        Some(error) => ExitCode::from(error.code.exit_code() as u8),
                        None => ExitCode::SUCCESS,
                    }
                }
                Err(code) => {
                    let error = IpcError {
                        code,
                        message: opencodex_desktop_lib::modules::ipc::error_message(code)
                            .to_string(),
                    };
                    if argv.iter().any(|value| value == "--json") {
                        let payload = cli_json::<serde_json::Value>(
                            command,
                            None,
                            Some(error),
                            chrono::Utc::now(),
                        );
                        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
                    } else {
                        eprintln!("{}", error.message);
                    }
                    ExitCode::from(code.exit_code() as u8)
                }
            }
        }
    }
}

#[cfg(all(unix, test))]
mod tests {
    use super::*;

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn request(values: &[&str]) -> IpcRequest {
        build_request(
            IpcCommand::DataRootSwitch,
            &argv(values),
            "req_00000000-0000-0000-0000-000000000001".to_string(),
        )
        .expect("request")
    }

    /// 回归：`--migrate` 必须转发给 IPC，否则「迁移数据」会静默降级成引用切换。
    #[test]
    fn migrate_flag_is_forwarded_to_the_ipc_request() {
        let request = request(&[
            "data-root",
            "switch",
            "--target",
            "/tmp/target-root",
            "--migrate",
            "--confirm",
        ]);
        assert_eq!(
            request.args.get("migrate").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            request.args.get("target").map(String::as_str),
            Some("/tmp/target-root")
        );
        assert!(request.confirm);
    }

    #[test]
    fn switch_without_migrate_keeps_reference_only_semantics() {
        let request = request(&[
            "data-root",
            "switch",
            "--target",
            "/tmp/target-root",
            "--confirm",
        ]);
        assert!(!request.args.contains_key("migrate"));
    }
}
