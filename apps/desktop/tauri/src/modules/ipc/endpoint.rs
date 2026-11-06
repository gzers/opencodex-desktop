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

/// Windows named pipe 身份由真实 TokenUser SID 派生。显式 DACL 限制连接，
/// 服务端在执行业务前再次核对客户端 token；命名摘要不代替访问控制。
#[cfg(any(windows, test))]
const WINDOWS_PIPE_IDENTITY: &str = "OpenCodeXDesktop";

#[cfg(any(windows, test))]
fn windows_pipe_identity(
    account: &str,
    sandbox_enabled: bool,
    sandbox_root: Option<&Path>,
) -> Result<String, AppError> {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"windows-user-sid-v2\0");
    digest.update(account.as_bytes());
    let identity = if sandbox_enabled {
        let root = sandbox_root
            .filter(|root| root.is_absolute())
            .ok_or(AppError::NotConfigured)?
            .canonicalize()
            .map_err(|_| AppError::NotConfigured)?;
        digest.update(b"\0sandbox\0");
        digest.update(root.as_os_str().as_encoded_bytes());
        "OpenCodeXDesktop-Sandbox"
    } else {
        WINDOWS_PIPE_IDENTITY
    };
    let suffix = digest
        .finalize()
        .iter()
        .take(12)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!(r"\\.\pipe\{identity}-{suffix}"))
}

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

#[cfg(unix)]
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
    use crate::infrastructure::windows_startup_root::user_scope;
    use std::os::windows::io::AsRawHandle;
    use tokio::net::windows::named_pipe::{NamedPipeServer, PipeMode, ServerOptions};

    fn fs_error(operation: &str, error: std::io::Error) -> AppError {
        AppError::FileSystem {
            operation: operation.into(),
            detail: error.to_string(),
        }
    }

    /// The process token SID is stable across sessions and same-account elevation;
    /// no environment account name or caller-supplied data root selects daily IPC.
    pub fn pipe_name() -> Result<String, AppError> {
        let sid =
            user_scope::current_user_sid().map_err(|e| fs_error("resolve Windows IPC SID", e))?;
        windows_pipe_identity(
            &sid,
            crate::modules::test_sandbox::enabled(),
            crate::modules::test_sandbox::sandbox_root().as_deref(),
        )
    }

    fn create_server(name: &str, first: bool) -> std::io::Result<NamedPipeServer> {
        let security = user_scope::private_pipe_security()?;
        let mut attributes = security.attributes();
        let mut options = ServerOptions::new();
        options
            .pipe_mode(PipeMode::Byte)
            .reject_remote_clients(true)
            .first_pipe_instance(first)
            .max_instances(2);
        // SAFETY: both the SECURITY_ATTRIBUTES and its descriptor are retained for
        // CreateNamedPipeW; Windows copies them and no inheritable handle escapes.
        unsafe {
            options.create_with_security_attributes_raw(
                name,
                (&mut attributes as *mut windows_sys::Win32::Security::SECURITY_ATTRIBUTES).cast(),
            )
        }
    }

    pub struct IpcEndpoint {
        pipe_name: String,
        server: Option<NamedPipeServer>,
        pub requests_served: Arc<AtomicU64>,
    }
    impl IpcEndpoint {
        pub fn bind(_cache_root: &Path) -> Result<Self, AppError> {
            Self::bind_name(pipe_name()?)
        }
        fn bind_name(pipe_name: String) -> Result<Self, AppError> {
            let server = create_server(&pipe_name, true)
                .map_err(|e| fs_error("bind Windows IPC named pipe", e))?;
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
            let result = async {
                server
                    .connect()
                    .await
                    .map_err(|e| fs_error("accept Windows IPC named pipe", e))?;
                // ImpersonateNamedPipeClient authenticates the last data read.
                // Bound parsing precedes identity, but business execution and any
                // response follow it. No await happens under the client token.
                let request = read_request(&mut server).await?;
                user_scope::verify_pipe_client(server.as_raw_handle())
                    .map_err(|e| fs_error("validate Windows IPC client SID", e))?;
                let response = service.execute_ipc(request).await;
                write_response(&mut server, &response).await
            }
            .await;
            // Keep our authenticated namespace handle alive until its replacement
            // exists. The old client may still be reading its response, so permit
            // exactly one replacement instance instead of disconnecting/discarding
            // unread bytes or dropping the namespace between requests.
            let replacement = create_server(&self.pipe_name, false)
                .map_err(|e| fs_error("rebind Windows IPC named pipe", e))?;
            drop(server);
            self.server = Some(replacement);
            result?;
            Ok(self.requests_served.fetch_add(1, Ordering::SeqCst) + 1)
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::modules::ipc::{service::IpcService, IpcCommand};
        use tokio::net::windows::named_pipe::ClientOptions;
        fn request() -> IpcRequest {
            IpcRequest {
                request_id: "req_00000000-0000-0000-0000-000000000001".into(),
                command: IpcCommand::Status,
                args: Default::default(),
                confirm: false,
                contract_version: 1,
                secret: None,
            }
        }
        #[tokio::test]
        async fn isolated_pipes_reject_duplicate_owners_and_keep_real_token_identity() {
            let a = tempfile::tempdir().unwrap();
            let b = tempfile::tempdir().unwrap();
            let sid = user_scope::current_user_sid().unwrap();
            let name = windows_pipe_identity(&sid, true, Some(a.path())).unwrap();
            let endpoint = IpcEndpoint::bind_name(name.clone()).unwrap();
            let other =
                IpcEndpoint::bind_name(windows_pipe_identity(&sid, true, Some(b.path())).unwrap())
                    .unwrap();
            assert_ne!(endpoint.endpoint_name(), other.endpoint_name());
            assert!(IpcEndpoint::bind_name(name.clone()).is_err());
            let client = ClientOptions::new().open(&name).unwrap();
            user_scope::verify_pipe_server(client.as_raw_handle()).unwrap();
            tokio::time::timeout(
                std::time::Duration::from_secs(2),
                endpoint.server.as_ref().unwrap().connect(),
            )
            .await
            .unwrap()
            .unwrap();
            drop(client);
        }
        #[tokio::test]
        async fn anonymous_peer_is_rejected_before_business_and_next_peer_still_works() {
            let root = tempfile::tempdir().unwrap();
            let sid = user_scope::current_user_sid().unwrap();
            let name = windows_pipe_identity(&sid, true, Some(root.path())).unwrap();
            let mut endpoint = IpcEndpoint::bind_name(name.clone()).unwrap();
            struct NeverExecute;
            #[async_trait::async_trait]
            impl crate::modules::ipc::service::IpcExecutor<serde_json::Value> for NeverExecute {
                async fn execute_ipc(&mut self, _: IpcRequest) -> IpcResponse<serde_json::Value> {
                    panic!("unauthenticated request reached business execution")
                }
            }
            let server = tokio::spawn(async move {
                let error = endpoint.accept_once(&mut NeverExecute).await.unwrap_err();
                assert!(
                    matches!(error, AppError::FileSystem { ref operation, .. } if operation == "validate Windows IPC client SID")
                );
                assert_eq!(endpoint.requests_served.load(Ordering::SeqCst), 0);
                {
                    let mut thread_token = std::ptr::null_mut();
                    let opened = unsafe {
                        windows_sys::Win32::System::Threading::OpenThreadToken(
                            windows_sys::Win32::System::Threading::GetCurrentThread(),
                            windows_sys::Win32::Security::TOKEN_QUERY,
                            1,
                            &mut thread_token,
                        )
                    };
                    if opened != 0 {
                        unsafe {
                            windows_sys::Win32::Foundation::CloseHandle(thread_token);
                        }
                        panic!("IPC validation left its worker impersonating the client");
                    }
                    assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(1008));
                }
                let mut service = IpcService::for_tests();
                endpoint.accept_once(&mut service).await.unwrap();
                assert_eq!(endpoint.requests_served.load(Ordering::SeqCst), 1);
            });
            let mut anonymous = ClientOptions::new()
                .security_qos_flags(0)
                .open(&name)
                .unwrap();
            let bytes = serde_json::to_vec(&request()).unwrap();
            anonymous
                .write_all(&(bytes.len() as u64).to_be_bytes())
                .await
                .unwrap();
            anonymous.write_all(&bytes).await.unwrap();
            let mut response = [0; 8];
            let rejected = tokio::time::timeout(
                std::time::Duration::from_secs(2),
                anonymous.read(&mut response),
            )
            .await
            .unwrap();
            assert!(
                matches!(rejected, Ok(0) | Err(_)),
                "unauthenticated client received response bytes"
            );
            drop(anonymous);
            let mut client = ClientOptions::new()
                .security_qos_flags(0x10000)
                .open(&name)
                .unwrap();
            client
                .write_all(&(bytes.len() as u64).to_be_bytes())
                .await
                .unwrap();
            client.write_all(&bytes).await.unwrap();
            let size = client.read_u64().await.unwrap();
            let mut bytes = vec![0; size as usize];
            client.read_exact(&mut bytes).await.unwrap();
            let response: IpcResponse<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
            assert!(response.ok);
            server.await.unwrap();
        }
        #[tokio::test]
        async fn authenticated_request_and_reopen_preserve_wire_contract() {
            let root = tempfile::tempdir().unwrap();
            let sid = user_scope::current_user_sid().unwrap();
            let name = windows_pipe_identity(&sid, true, Some(root.path())).unwrap();
            let mut endpoint = IpcEndpoint::bind_name(name.clone()).unwrap();
            let server = tokio::spawn(async move {
                let mut service = IpcService::for_tests();
                endpoint.accept_once(&mut service).await.unwrap();
                endpoint.accept_once(&mut service).await.unwrap();
                endpoint.requests_served.load(Ordering::SeqCst)
            });
            for _ in 0..2 {
                let mut client = loop {
                    match ClientOptions::new()
                        .security_qos_flags(0x0011_0000)
                        .open(&name)
                    {
                        Ok(client) => break client,
                        Err(e) if e.raw_os_error() == Some(231) => tokio::task::yield_now().await,
                        Err(e) => panic!("connect isolated pipe: {e}"),
                    }
                };
                let bytes = serde_json::to_vec(&request()).unwrap();
                client
                    .write_all(&(bytes.len() as u64).to_be_bytes())
                    .await
                    .unwrap();
                client.write_all(&bytes).await.unwrap();
                let size = client.read_u64().await.unwrap();
                let mut response = vec![0; size as usize];
                client.read_exact(&mut response).await.unwrap();
                let response: IpcResponse<serde_json::Value> =
                    serde_json::from_slice(&response).unwrap();
                assert!(response.ok);
                assert_eq!(response.request_id, request().request_id);
                drop(client);
            }
            assert_eq!(server.await.unwrap(), 2);
        }
    }
}

#[cfg(windows)]
pub use platform::pipe_name;

/// Authenticate a CLI connection before sending its first request. The shared
/// CLI entrypoint owns the call; server-side identity checks are independent.
#[cfg(windows)]
pub fn verify_windows_server(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    crate::infrastructure::windows_startup_root::user_scope::verify_pipe_server(
        file.as_raw_handle(),
    )
}
#[cfg(unix)]
pub use platform::socket_path;
pub use platform::IpcEndpoint;

#[cfg(test)]
mod windows_identity_tests {
    use super::*;

    #[test]
    fn daily_identity_is_sid_scoped_and_independent_of_roots() {
        let root = tempfile::tempdir().unwrap();
        let sid = "S-1-5-21-1-2-3-1001";
        assert_eq!(
            windows_pipe_identity(sid, false, None).unwrap(),
            windows_pipe_identity(sid, false, Some(root.path())).unwrap()
        );
        assert_ne!(
            windows_pipe_identity(sid, false, None).unwrap(),
            windows_pipe_identity("S-1-5-21-1-2-3-1002", false, None).unwrap()
        );
    }

    #[test]
    fn sandbox_pipe_is_root_scoped_and_invalid_root_never_selects_daily_pipe() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let daily = windows_pipe_identity("S-1-5-21-1-2-3-1001", false, None).unwrap();
        let isolated =
            windows_pipe_identity("S-1-5-21-1-2-3-1001", true, Some(first.path())).unwrap();
        assert_ne!(isolated, daily);
        assert!(isolated.starts_with(r"\\.\pipe\OpenCodeXDesktop-Sandbox-"));
        assert_ne!(
            isolated,
            windows_pipe_identity("S-1-5-21-1-2-3-1001", true, Some(second.path())).unwrap()
        );
        assert_eq!(
            isolated,
            windows_pipe_identity("S-1-5-21-1-2-3-1001", true, Some(&first.path().join(".")))
                .unwrap()
        );
        for invalid in [None, Some(Path::new("relative"))] {
            assert!(windows_pipe_identity("S-1-5-21-1-2-3-1001", true, invalid).is_err());
        }
        assert!(windows_pipe_identity(
            "S-1-5-21-1-2-3-1001",
            true,
            Some(&first.path().join("missing"))
        )
        .is_err());
    }
}

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
