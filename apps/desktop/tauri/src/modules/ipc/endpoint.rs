//! MOD-12 真实本机 IPC 端点。
//!
//! Unix 使用私有 Unix socket，Windows 使用拒绝远程客户端的 named pipe。
//! 两个平台共享同一套 8 字节大端长度 + JSON frame 协议；本层只负责
//! 本机安全传输，不执行业务命令，也不搜索 PATH。

use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::errors::AppError;
use crate::modules::ipc::{IpcRequest, IpcResponse, MAX_IPC_FRAME_BYTES};

pub const SOCKET_RELATIVE_PATH: &str = "ipc/opencodex.ipc";

/// Windows named pipe 的稳定身份前缀。实际名称还包含当前账户的稳定摘要，
/// 避免不同账户的客户端意外连接到同一个可见名称；访问控制仍由 named pipe
/// 的创建者默认 DACL 和 reject_remote_clients 共同承担。
#[cfg(windows)]
const WINDOWS_PIPE_IDENTITY: &str = "OpenCodeXDesktop";

async fn read_request<R>(stream: &mut R) -> Result<IpcRequest, AppError>
where
    R: AsyncRead + Unpin,
{
    let mut length_bytes = [0u8; 8];
    stream
        .read_exact(&mut length_bytes)
        .await
        .map_err(|error| AppError::FileSystem {
            operation: "read IPC frame length".to_string(),
            detail: error.to_string(),
        })?;
    let length =
        usize::try_from(u64::from_be_bytes(length_bytes)).map_err(|_| AppError::FileSystem {
            operation: "validate IPC frame length".to_string(),
            detail: "IPC frame length cannot fit in memory".to_string(),
        })?;
    if length == 0 || length > MAX_IPC_FRAME_BYTES {
        return Err(AppError::FileSystem {
            operation: "validate IPC frame length".to_string(),
            detail: "IPC frame is empty or exceeds the frozen limit".to_string(),
        });
    }
    let mut payload = vec![0u8; length];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|error| AppError::FileSystem {
            operation: "read IPC frame".to_string(),
            detail: error.to_string(),
        })?;
    serde_json::from_slice::<IpcRequest>(&payload).map_err(|error| AppError::FileSystem {
        operation: "parse IPC request".to_string(),
        detail: error.to_string(),
    })
}

async fn write_response<W>(
    stream: &mut W,
    response: &IpcResponse<serde_json::Value>,
) -> Result<(), AppError>
where
    W: AsyncWrite + Unpin,
{
    let bytes = serde_json::to_vec(response).map_err(|error| AppError::FileSystem {
        operation: "serialize IPC response".to_string(),
        detail: error.to_string(),
    })?;
    if bytes.is_empty() || bytes.len() > MAX_IPC_FRAME_BYTES {
        return Err(AppError::FileSystem {
            operation: "validate IPC response length".to_string(),
            detail: "IPC response exceeds the frozen limit".to_string(),
        });
    }
    stream
        .write_all(&(bytes.len() as u64).to_be_bytes())
        .await
        .map_err(|error| AppError::FileSystem {
            operation: "write IPC response length".to_string(),
            detail: error.to_string(),
        })?;
    stream
        .write_all(&bytes)
        .await
        .map_err(|error| AppError::FileSystem {
            operation: "write IPC response".to_string(),
            detail: error.to_string(),
        })?;
    stream.flush().await.map_err(|error| AppError::FileSystem {
        operation: "flush IPC response".to_string(),
        detail: error.to_string(),
    })
}

async fn handle_stream<T, S>(mut stream: T, service: &mut S) -> Result<(), AppError>
where
    T: AsyncRead + AsyncWrite + Unpin,
    S: super::service::IpcExecutor<serde_json::Value>,
{
    let request = read_request(&mut stream).await?;
    let response = service.execute_ipc(request).await;
    write_response(&mut stream, &response).await
}

#[cfg(unix)]
mod platform {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::io::AsRawFd;
    use tokio::net::{UnixListener, UnixStream};

    pub fn socket_path(cache_root: &Path) -> PathBuf {
        cache_root.join(SOCKET_RELATIVE_PATH)
    }

    fn validate_peer(stream: &UnixStream) -> Result<(), AppError> {
        let raw_fd = stream.as_raw_fd();
        #[cfg(target_vendor = "apple")]
        {
            let mut peer_uid: libc::uid_t = 0;
            let mut peer_gid: libc::gid_t = 0;
            let result = unsafe { libc::getpeereid(raw_fd, &mut peer_uid, &mut peer_gid) };
            let current_uid = unsafe { libc::getuid() };
            if result != 0 || peer_uid != current_uid {
                return Err(AppError::FileSystem {
                    operation: "validate IPC peer identity".to_string(),
                    detail: "peer uid does not match the application owner".to_string(),
                });
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut credentials = libc::ucred {
                pid: 0,
                uid: 0,
                gid: 0,
            };
            let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
            let result = unsafe {
                libc::getsockopt(
                    raw_fd,
                    libc::SOL_SOCKET,
                    libc::SO_PEERCRED,
                    &mut credentials as *mut libc::ucred as *mut libc::c_void,
                    &mut length,
                )
            };
            let current_uid = unsafe { libc::geteuid() };
            if result != 0 || credentials.uid != current_uid {
                return Err(AppError::FileSystem {
                    operation: "validate IPC peer identity".to_string(),
                    detail: "peer uid does not match the application owner".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Unix socket 端点；目录保持 0700，socket 保持 0600。
    pub struct IpcEndpoint {
        listener: UnixListener,
        socket_path: PathBuf,
        pub requests_served: Arc<AtomicU64>,
    }

    impl IpcEndpoint {
        /// 创建私有 socket。已有可用端点会阻断第二次创建。
        pub fn bind(cache_root: &Path) -> Result<Self, AppError> {
            let _storage = crate::infrastructure::storage_writers::global().admit()?;
            let path = socket_path(cache_root);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| AppError::FileSystem {
                    operation: "create IPC directory".to_string(),
                    detail: error.to_string(),
                })?;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).map_err(
                    |error| AppError::FileSystem {
                        operation: "set IPC directory permissions".to_string(),
                        detail: error.to_string(),
                    },
                )?;
            }

            if std::os::unix::net::UnixStream::connect(&path).is_ok() {
                return Err(AppError::InstanceLockConflict);
            }
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(AppError::FileSystem {
                        operation: "remove unavailable IPC socket".to_string(),
                        detail: error.to_string(),
                    })
                }
            }

            let listener = UnixListener::bind(&path).map_err(|error| AppError::FileSystem {
                operation: "bind IPC socket".to_string(),
                detail: error.to_string(),
            })?;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).map_err(
                |error| AppError::FileSystem {
                    operation: "set IPC socket permissions".to_string(),
                    detail: error.to_string(),
                },
            )?;
            Ok(Self {
                listener,
                socket_path: path,
                requests_served: Arc::new(AtomicU64::new(0)),
            })
        }

        pub fn local_path(&self) -> &Path {
            &self.socket_path
        }

        pub async fn accept_once<S>(&mut self, service: &mut S) -> Result<u64, AppError>
        where
            S: crate::modules::ipc::service::IpcExecutor<serde_json::Value>,
        {
            let (stream, _address) =
                self.listener
                    .accept()
                    .await
                    .map_err(|error| AppError::FileSystem {
                        operation: "accept IPC connection".to_string(),
                        detail: error.to_string(),
                    })?;
            validate_peer(&stream)?;
            handle_stream(stream, service).await?;
            Ok(self.requests_served.fetch_add(1, Ordering::SeqCst) + 1)
        }
    }

    impl Drop for IpcEndpoint {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use sha2::{Digest, Sha256};
    use tokio::net::windows::named_pipe::{NamedPipeServer, PipeMode, ServerOptions};

    /// Named pipe 名称不使用 C/D 盘路径；默认 DACL 绑定创建者的 Windows
    /// 安全边界，且服务端拒绝远程客户端。账户摘要只用于稳定区分同机账户。
    pub fn pipe_name() -> String {
        let account = std::env::var_os("USERNAME")
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown-user".to_string());
        let mut digest = Sha256::new();
        digest.update(account.as_bytes());
        let suffix = digest
            .finalize()
            .iter()
            .take(12)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!(r"\\.\pipe\{WINDOWS_PIPE_IDENTITY}-{suffix}")
    }

    fn create_server(name: &str, first: bool) -> std::io::Result<NamedPipeServer> {
        let mut options = ServerOptions::new();
        options
            .pipe_mode(PipeMode::Byte)
            .reject_remote_clients(true)
            .first_pipe_instance(first)
            .max_instances(1);
        options.create(name)
    }

    /// Windows named pipe 端点；每处理一个请求就创建下一个 server instance。
    pub struct IpcEndpoint {
        pipe_name: String,
        server: Option<NamedPipeServer>,
        pub requests_served: Arc<AtomicU64>,
    }

    impl IpcEndpoint {
        pub fn bind(_cache_root: &Path) -> Result<Self, AppError> {
            let pipe_name = pipe_name();
            let server = create_server(&pipe_name, true).map_err(|error| AppError::FileSystem {
                operation: "bind Windows IPC named pipe".to_string(),
                detail: error.to_string(),
            })?;
            Ok(Self {
                pipe_name,
                server: Some(server),
                requests_served: Arc::new(AtomicU64::new(0)),
            })
        }

        pub fn endpoint_name(&self) -> &str {
            &self.pipe_name
        }

        pub async fn accept_once<S>(&mut self, service: &mut S) -> Result<u64, AppError>
        where
            S: crate::modules::ipc::service::IpcExecutor<serde_json::Value>,
        {
            let mut server = self.server.take().ok_or(AppError::InstanceLockConflict)?;
            server
                .connect()
                .await
                .map_err(|error| AppError::FileSystem {
                    operation: "accept Windows IPC named pipe".to_string(),
                    detail: error.to_string(),
                })?;
            let result = handle_stream(server, service).await;
            self.server = Some(create_server(&self.pipe_name, false).map_err(|error| {
                AppError::FileSystem {
                    operation: "rebind Windows IPC named pipe".to_string(),
                    detail: error.to_string(),
                }
            })?);
            result?;
            Ok(self.requests_served.fetch_add(1, Ordering::SeqCst) + 1)
        }
    }
}

#[cfg(windows)]
pub use platform::pipe_name;
#[cfg(unix)]
pub use platform::socket_path;
pub use platform::IpcEndpoint;

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::ipc::service::IpcService;
    use crate::modules::ipc::{IpcCommand, IpcErrorCode, IpcResponse};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn fixture_request() -> IpcRequest {
        IpcRequest {
            request_id: "req_00000000-0000-0000-0000-000000000001".to_string(),
            command: IpcCommand::Status,
            args: BTreeMap::new(),
            confirm: false,
            contract_version: 1,
            secret: None,
        }
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn binds_private_socket_and_round_trips_frozen_response() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().expect("create temporary cache root");
        let mut endpoint = IpcEndpoint::bind(root.path()).expect("bind endpoint");
        let service = Arc::new(tokio::sync::Mutex::new(IpcService::for_tests()));
        let path = endpoint.local_path().to_path_buf();
        let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
            tokio::spawn(async move {
                let mut guard = service.lock().await;
                endpoint
                    .accept_once(&mut *guard)
                    .await
                    .expect("handle request");
                Ok::<(), std::io::Error>(())
            });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        assert_eq!(
            std::fs::metadata(&path)
                .expect("socket metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let response = send_unix_request(path, fixture_request())
            .await
            .expect("response");
        assert!(response.ok);
        assert_eq!(
            response.request_id,
            "req_00000000-0000-0000-0000-000000000001"
        );
        let _ = server.await.expect("server task");
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn rejects_bad_contract_and_too_large_frame() {
        let root = tempfile::tempdir().expect("create temporary cache root");
        let mut endpoint = IpcEndpoint::bind(root.path()).expect("bind endpoint");
        let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
            tokio::spawn(async move {
                let mut service = IpcService::for_tests();
                let _ = endpoint.accept_once(&mut service).await;
                Ok::<(), std::io::Error>(())
            });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let mut invalid = fixture_request();
        invalid.contract_version = 2;
        let response = send_unix_request(root.path().join(SOCKET_RELATIVE_PATH), invalid)
            .await
            .expect("response");
        assert_eq!(
            response.error.map(|error| error.code),
            Some(IpcErrorCode::ContractVersionMismatch)
        );
        let _ = server.await.expect("server task");

        let _endpoint = IpcEndpoint::bind(root.path()).expect("rebind endpoint");
        let mut writer = tokio::net::UnixStream::connect(root.path().join(SOCKET_RELATIVE_PATH))
            .await
            .expect("connect");
        writer
            .write_all(&(MAX_IPC_FRAME_BYTES as u64 + 1).to_be_bytes())
            .await
            .expect("write");
        let mut length = [0u8; 8];
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(120),
            writer.read_exact(&mut length),
        )
        .await;
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn prevents_second_live_endpoint() {
        let root = tempfile::tempdir().expect("create temporary cache root");
        let endpoint = Arc::new(tokio::sync::Mutex::new(
            IpcEndpoint::bind(root.path()).expect("bind endpoint"),
        ));
        let service = Arc::new(tokio::sync::Mutex::new(IpcService::for_tests()));
        let path = endpoint.lock().await.local_path().to_path_buf();
        let server_endpoint = endpoint.clone();
        let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
            tokio::spawn(async move {
                let mut service = service.lock().await;
                let mut endpoint = server_endpoint.lock().await;
                let _ = endpoint.accept_once(&mut *service).await;
                let _ = endpoint.accept_once(&mut *service).await;
                Ok::<(), std::io::Error>(())
            });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let second_bind = IpcEndpoint::bind(root.path());
        assert!(second_bind.is_err());
        let response = send_unix_request(path.clone(), fixture_request())
            .await
            .expect("response");
        assert!(response.ok);
        assert!(path.exists());
        let _ = server.await.expect("server task");
    }

    #[cfg(unix)]
    async fn send_unix_request(
        path: PathBuf,
        request: IpcRequest,
    ) -> Result<IpcResponse<serde_json::Value>, std::io::Error> {
        use tokio::net::UnixStream;
        let payload = serde_json::to_vec(&request).expect("serialize request");
        let mut stream = UnixStream::connect(path).await?;
        stream
            .write_all(&(payload.len() as u64).to_be_bytes())
            .await?;
        stream.write_all(&payload).await?;
        stream.flush().await?;
        let mut length = [0u8; 8];
        stream.read_exact(&mut length).await?;
        let length = u64::from_be_bytes(length) as usize;
        let mut payload = vec![0u8; length];
        stream.read_exact(&mut payload).await?;
        Ok(serde_json::from_slice(&payload).expect("valid response"))
    }
}
