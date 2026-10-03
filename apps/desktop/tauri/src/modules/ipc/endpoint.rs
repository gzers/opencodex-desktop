//! MOD-12 真实 Unix socket 端点。
//!
//! 默认关闭；启用后只监听冻结的本机 Unix socket。目录保持 0700，
//! socket 保持 0600，连接前校验 `getpeereid`。本层只负责安全传输，
//! 不执行业务命令，也不搜索 PATH。

use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::errors::AppError;
use crate::modules::ipc::{IpcRequest, MAX_IPC_FRAME_BYTES};

pub const SOCKET_RELATIVE_PATH: &str = "ipc/opencodex.ipc";

pub fn socket_path(cache_root: &Path) -> PathBuf {
    cache_root.join(SOCKET_RELATIVE_PATH)
}

/// 只允许同一 UID 的 Unix 连接，逐帧读取并处理请求。
pub async fn handle_connection<S>(mut stream: UnixStream, service: &mut S) -> Result<(), AppError>
where
    S: super::service::IpcExecutor<serde_json::Value>,
{
    #[cfg(target_vendor = "apple")]
    {
        let raw_fd = stream.as_raw_fd();
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

    let mut length_bytes = [0u8; 8];
    stream
        .read_exact(&mut length_bytes)
        .await
        .map_err(|error| AppError::FileSystem {
            operation: "read IPC frame length".to_string(),
            detail: error.to_string(),
        })?;
    let length = u64::from_be_bytes(length_bytes) as usize;
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
    let request =
        serde_json::from_slice::<IpcRequest>(&payload).map_err(|error| AppError::FileSystem {
            operation: "parse IPC request".to_string(),
            detail: error.to_string(),
        })?;
    let response = service.execute_ipc(request).await;
    let bytes = serde_json::to_vec(&response).map_err(|error| AppError::FileSystem {
        operation: "serialize IPC response".to_string(),
        detail: error.to_string(),
    })?;
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
    })?;
    Ok(())
}

/// RAII Unix socket 端点；Drop 时先关闭监听器再移除 socket 文件。
pub struct IpcEndpoint {
    listener: UnixListener,
    socket_path: PathBuf,
    pub requests_served: Arc<AtomicU64>,
}

impl IpcEndpoint {
    /// 在缓存根目录创建私有 socket。已有可用端点会阻断第二次创建。
    pub fn bind(cache_root: &Path) -> Result<Self, AppError> {
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

        let probe = std::os::unix::net::UnixStream::connect(&path);
        if probe.is_ok() {
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

    /// 单线程接受循环；上层负责包装具体 service。
    pub async fn accept_once<S>(&self, service: &mut S) -> Result<u64, AppError>
    where
        S: super::service::IpcExecutor<serde_json::Value>,
    {
        let (stream, _address) =
            self.listener
                .accept()
                .await
                .map_err(|error| AppError::FileSystem {
                    operation: "accept IPC connection".to_string(),
                    detail: error.to_string(),
                })?;
        handle_connection(stream, service).await?;
        Ok(self.requests_served.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ipc::endpoint::IpcEndpoint;
    use crate::modules::ipc::service::IpcService;
    use crate::modules::ipc::{IpcCommand, IpcErrorCode, IpcResponse};
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn binds_private_socket_and_round_trips_frozen_response() {
        let root = tempfile::tempdir().expect("create temporary cache root");
        let endpoint = IpcEndpoint::bind(root.path()).expect("bind endpoint");
        let service = Arc::new(tokio::sync::Mutex::new(IpcService::for_tests()));
        let path = endpoint.local_path().to_path_buf();

        let served = endpoint.requests_served.clone();
        let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
            tokio::spawn(async move {
                {
                    let mut guard = service.lock().await;
                    endpoint
                        .accept_once(&mut *guard)
                        .await
                        .expect("handle request");
                    drop(guard);
                }
                drop(served);
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
        let response = send_request(path.clone(), fixture_request())
            .await
            .expect("response");
        assert!(response.ok);
        assert_eq!(
            response.request_id,
            "req_00000000-0000-0000-0000-000000000001"
        );
        let _ = server.await.expect("server task");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn rejects_bad_contract_and_too_large_frame() {
        let root = tempfile::tempdir().expect("create temporary cache root");
        let endpoint = IpcEndpoint::bind(root.path()).expect("bind endpoint");
        {
            let mut service = IpcService::for_tests();
            let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
                tokio::spawn(async move {
                    let _ = endpoint.accept_once(&mut service).await;
                    Ok::<(), std::io::Error>(())
                });
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let mut invalid = fixture_request();
            invalid.contract_version = 2;
            let response = send_request(root.path().join("ipc/opencodex.ipc"), invalid)
                .await
                .expect("response");
            assert_eq!(
                response.error.map(|error| error.code),
                Some(IpcErrorCode::ContractVersionMismatch)
            );
            let _ = server.await.expect("server task");
        }

        let _endpoint = IpcEndpoint::bind(root.path()).expect("rebind endpoint");
        let mut writer = tokio::net::UnixStream::connect(root.path().join("ipc/opencodex.ipc"))
            .await
            .expect("connect");
        writer
            .write_all(&(MAX_IPC_FRAME_BYTES as u64 + 1).to_be_bytes())
            .await
            .expect("write oversized length");
        let mut length = [0u8; 8];
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(120),
            writer.read_exact(&mut length),
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn prevents_second_live_endpoint() {
        let root = tempfile::tempdir().expect("create temporary cache root");
        // 第二次 bind 的探测连接会被真实监听器接受；原 endpoint 必须留在
        // 测试作用域内，否则其 Drop 会删除仍然可服务的 socket。
        let endpoint = Arc::new(IpcEndpoint::bind(root.path()).expect("bind endpoint"));
        let service = Arc::new(tokio::sync::Mutex::new(IpcService::for_tests()));
        let path = endpoint.local_path().to_path_buf();

        let server_endpoint = endpoint.clone();
        let served = endpoint.requests_served.clone();
        let server: tokio::task::JoinHandle<Result<(), std::io::Error>> =
            tokio::spawn(async move {
                {
                    let mut guard = service.lock().await;
                    // drain second bind's probe connection.
                    let _ = server_endpoint.accept_once(&mut *guard).await;
                    // serve the real request that follows the failed second bind.
                    let _ = server_endpoint.accept_once(&mut *guard).await;
                    drop(guard);
                }
                drop(served);
                Ok::<(), std::io::Error>(())
            });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let second_bind = IpcEndpoint::bind(root.path());
        assert!(second_bind.is_err());
        let response = send_request(path.clone(), fixture_request())
            .await
            .expect("response");
        assert!(response.ok);
        assert!(
            path.exists(),
            "failed second bind must not remove live socket"
        );
        let _ = server.await.expect("server task");
    }

    async fn send_request(
        path: std::path::PathBuf,
        request: IpcRequest,
    ) -> Result<IpcResponse<serde_json::Value>, std::io::Error> {
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
