//! A fresh runtime observation on ONE authenticated loopback TCP connection.
//! No port discovery, reusable credentials, status cache or configuration writes.
use super::status_source::status_collect_timeout;
use crate::{
    infrastructure::runtime_executable::SharedRuntimeExecutable,
    modules::{backup::safety, status::CollectError},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::Sha256;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
const MAX_BODY: usize = 64 * 1024;
#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Record {
    pid: u32,
    port: u16,
    attestation_secret: String,
}
fn unavailable<T>(_: T) -> CollectError {
    CollectError::Unreachable
}
fn record(home: &Path) -> Result<(Record, Vec<u8>), CollectError> {
    let path = home.join("runtime-port.json");
    safety::check_path(home, &path, false).map_err(unavailable)?;
    let file = safety::open_file(&path, false).map_err(unavailable)?;
    safety::require_single_link(&file).map_err(unavailable)?;
    let metadata = file.metadata().map_err(unavailable)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(CollectError::Unreachable);
        }
    }
    #[cfg(windows)]
    super::windows_private_create::verify_runtime_record_handle(&file).map_err(unavailable)?;
    if metadata.len() > 4096 {
        return Err(CollectError::Parse);
    }
    let mut bytes = Vec::new();
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(unavailable)?;
    if bytes.len() > 4096 {
        return Err(CollectError::Parse);
    }
    let value: Record = serde_json::from_slice(&bytes).map_err(|_| CollectError::Parse)?;
    if value.pid == 0
        || value.port == 0
        || value.attestation_secret.len() != 43
        || URL_SAFE_NO_PAD
            .decode(&value.attestation_secret)
            .map_or(true, |v| v.len() != 32)
    {
        return Err(CollectError::Parse);
    }
    Ok((value, bytes))
}
struct Response {
    code: u16,
    proof: Option<String>,
    value: Value,
}
struct Session {
    io: BufReader<TcpStream>,
    port: u16,
}
impl Session {
    async fn get(&mut self, path: &str, challenge: Option<&str>) -> Result<Response, CollectError> {
        self.request("GET", path, challenge, None).await
    }
    async fn request(
        &mut self,
        method: &str,
        path: &str,
        challenge: Option<&str>,
        token: Option<&str>,
    ) -> Result<Response, CollectError> {
        let challenge = challenge
            .map(|c| format!("x-opencodex-attestation-challenge: {c}\r\n"))
            .unwrap_or_default();
        let auth = token
            .map(|token| format!("Authorization: Bearer {token}\r\n"))
            .unwrap_or_default();
        let body = if method == "POST" {
            "Content-Length: 0\r\n"
        } else {
            ""
        };
        let request=zeroize::Zeroizing::new(format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: keep-alive\r\nAccept: application/json\r\n{challenge}{auth}{body}\r\n",self.port));
        self.io
            .get_mut()
            .write_all(request.as_bytes())
            .await
            .map_err(unavailable)?;
        self.io.get_mut().flush().await.map_err(unavailable)?;
        // Bound even a header without a newline; never allocate by an untrusted size.
        let mut headers = Vec::new();
        loop {
            let byte = self.io.read_u8().await.map_err(unavailable)?;
            headers.push(byte);
            if headers.len() > 8192 {
                return Err(CollectError::Parse);
            }
            if headers.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        let text = std::str::from_utf8(&headers).map_err(|_| CollectError::Parse)?;
        let mut lines = text.split("\r\n");
        let status = lines.next().ok_or(CollectError::Parse)?;
        let mut status = status.split_whitespace();
        if status.next() != Some("HTTP/1.1") {
            return Err(CollectError::Parse);
        }
        let code = status
            .next()
            .ok_or(CollectError::Parse)?
            .parse::<u16>()
            .map_err(|_| CollectError::Parse)?;
        let mut length = None;
        let mut proof = None;
        for line in lines.filter(|s| !s.is_empty()) {
            let (key, value) = line.split_once(':').ok_or(CollectError::Parse)?;
            if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
                return Err(CollectError::Parse);
            }
            let value = value.trim();
            match key.to_ascii_lowercase().as_str() {
                "content-length" => {
                    if length.is_some() || !value.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(CollectError::Parse);
                    }
                    length = Some(value.parse::<usize>().map_err(|_| CollectError::Parse)?);
                }
                "x-opencodex-attestation-proof" => {
                    if proof.is_some() {
                        return Err(CollectError::Parse);
                    }
                    proof = Some(value.to_owned());
                }
                "transfer-encoding" | "content-encoding" => return Err(CollectError::Parse),
                "connection" if !value.eq_ignore_ascii_case("keep-alive") => {
                    return Err(CollectError::Unreachable)
                }
                _ => {}
            }
        }
        let length = length
            .filter(|n| *n > 0 && *n <= MAX_BODY)
            .ok_or(CollectError::Parse)?;
        let mut bytes = vec![0; length];
        self.io.read_exact(&mut bytes).await.map_err(unavailable)?;
        let value = serde_json::from_slice(&bytes).map_err(|_| CollectError::Parse)?;
        Ok(Response { code, proof, value })
    }
}

/// The write stays on the TCP connection that proved this HOME's key. Never
/// forward the reusable admin token to a newly connected port after attestation.
pub(crate) async fn stop_official(
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
    pid: u32,
    phase: impl Fn(&'static str),
) -> Result<(), CollectError> {
    phase("stop-runtime-activation");
    let root = runtime
        .managed_installation_root()
        .ok_or(CollectError::Unreachable)?;
    let identity = crate::modules::runtime::install::resolve_managed_installation(&root)
        .map_err(unavailable)?;
    if runtime.executable().as_ref() != Some(&identity.entry) {
        return Err(CollectError::Unreachable);
    }
    phase("stop-protected-record");
    let (expected, bytes) = record(&home)?;
    if expected.pid != pid {
        return Err(CollectError::Unreachable);
    }
    phase("stop-connect");
    let stream = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, expected.port))
        .await
        .map_err(unavailable)?;
    let mut session = Session {
        io: BufReader::new(stream),
        port: expected.port,
    };
    phase("stop-first-attestation");
    health(&mut session, &expected, &identity.package_version).await?;
    phase("stop-private-token");
    let path = home.join("admin-api-token");
    safety::check_path(&home, &path, false).map_err(unavailable)?;
    let token = zeroize::Zeroizing::new(safety::read_file(&path, 512).map_err(unavailable)?);
    let token = std::str::from_utf8(&token)
        .map_err(|_| CollectError::Parse)?
        .trim();
    if !token.starts_with("ocx_admin_")
        || token.len() != 53
        || !token[10..]
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    {
        return Err(CollectError::Parse);
    }
    phase("stop-read-capabilities");
    let capability = session
        .request("GET", "/api/stop", None, Some(token))
        .await?;
    if capability.code != 200
        || capability.value["schema"] != "ocx-stop-capabilities/1"
        || capability.value["connectedClientOwnership"] != true
        || capability.value["pid"].as_u64() != Some(pid.into())
    {
        return Err(CollectError::Unreachable);
    }
    phase("stop-second-attestation");
    health(&mut session, &expected, &identity.package_version).await?;
    phase("stop-revalidate-before-write");
    if record(&home)?.1 != bytes
        || crate::modules::runtime::install::resolve_managed_installation(&root)
            .map_err(unavailable)?
            != identity
    {
        return Err(CollectError::Unreachable);
    }
    phase("stop-post-client-restore");
    let stopped = session
        .request("POST", "/api/stop", None, Some(token))
        .await?;
    if stopped.code != 200
        || stopped.value["success"] != true
        || !matches!(
            stopped.value["sharedTeardown"].as_str(),
            Some("performed" | "not-owned")
        )
    {
        return Err(CollectError::Unreachable);
    }
    phase("stop-response-verified");
    Ok(())
}
fn identity(value: &Value, record: &Record, version: &str) -> bool {
    value["service"] == "opencodex"
        && value["pid"].as_u64() == Some(record.pid as u64)
        && value["port"].as_u64() == Some(record.port as u64)
        && value["version"] == version
}
async fn health(
    session: &mut Session,
    record: &Record,
    version: &str,
) -> Result<Value, CollectError> {
    let challenge = URL_SAFE_NO_PAD.encode(
        uuid::Uuid::new_v4()
            .as_bytes()
            .iter()
            .chain(uuid::Uuid::new_v4().as_bytes())
            .copied()
            .collect::<Vec<_>>(),
    );
    let response = session.get("/healthz", Some(&challenge)).await?;
    if response.code != 200
        || !identity(&response.value, record, version)
        || response.value["status"] != "ok"
    {
        return Err(CollectError::Unreachable);
    }
    let proof = response
        .proof
        .and_then(|p| URL_SAFE_NO_PAD.decode(p).ok())
        .ok_or(CollectError::Unreachable)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(record.attestation_secret.as_bytes())
        .map_err(unavailable)?;
    mac.update(
        format!(
            "opencodex-local-management-v1\n{challenge}\n{}\n{}",
            record.pid, record.port
        )
        .as_bytes(),
    );
    mac.verify_slice(&proof).map_err(unavailable)?;
    Ok(response.value)
}
async fn observe(
    home: &Path,
    expected: &Record,
    original: &[u8],
    version: &str,
) -> Result<Value, CollectError> {
    let stream = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, expected.port))
        .await
        .map_err(unavailable)?;
    let mut session = Session {
        io: BufReader::new(stream),
        port: expected.port,
    };
    let before = health(&mut session, expected, version).await?;
    let ready = session.get("/readyz", None).await?;
    if !identity(&ready.value, expected, version)
        || !matches!(
            (ready.code, ready.value["status"].as_str()),
            (200, Some("ready")) | (503, Some("pending" | "failed"))
        )
    {
        return Err(CollectError::Unreachable);
    }
    let after = health(&mut session, expected, version).await?;
    if before["uptime"]
        .as_f64()
        .zip(after["uptime"].as_f64())
        .is_none_or(|(a, b)| !a.is_finite() || !b.is_finite() || a < 0.0 || b < a)
    {
        return Err(CollectError::Unreachable);
    }
    let root = home.to_path_buf();
    let (_, readback) = tokio::task::spawn_blocking(move || record(&root))
        .await
        .map_err(unavailable)??;
    if readback != original {
        return Err(CollectError::Unreachable);
    }
    // Unknown startup/service/shim protection is omitted, never made reboot-safe.
    // This is a source projection of current official responses, not CLI diagnosis.
    let state = if ready.value["status"] == "ready" {
        "running"
    } else {
        "pending"
    };
    Ok(
        json!({"status":state,"ready":state=="running","officialReadiness":ready.value["status"],"port":expected.port,"pid":expected.pid.to_string(),"runtimeLabel":"bundled","versionDrift":version,"opencodexHome":home,"statusObservation":"attested-same-connection","startup":{}}),
    )
}
pub(crate) async fn fetch(
    runtime: SharedRuntimeExecutable,
    home: PathBuf,
) -> Result<Option<Value>, CollectError> {
    let Some(root) = runtime.managed_installation_root() else {
        return Ok(None);
    };
    tokio::time::timeout(status_collect_timeout(), async move {
        let final_root = root.clone();
        let final_runtime = runtime.clone();
        let prepared = tokio::task::spawn_blocking(move || {
            // Full activation validation on every observation; no mtime trust/cache.
            let identity = crate::modules::runtime::install::resolve_managed_installation(&root)
                .map_err(unavailable)?;
            if runtime.executable().as_ref() != Some(&identity.entry) {
                return Err(CollectError::Unreachable);
            }
            match std::fs::symlink_metadata(home.join("runtime-port.json")) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(_) => Err(CollectError::Unreachable),
                Ok(_) => {
                    let (r, b) = record(&home)?;
                    Ok(Some((home, r, b, identity.package_version)))
                }
            }
        })
        .await
        .map_err(unavailable)??;
        let Some((home, record, bytes, version)) = prepared else {
            return Ok(None);
        };
        let payload = observe(&home, &record, &bytes, &version).await?;
        let (verified, final_bytes) = tokio::task::spawn_blocking(move || {
            let identity =
                crate::modules::runtime::install::resolve_managed_installation(&final_root)
                    .map_err(unavailable)?;
            let (_, bytes) = self::record(&home)?;
            Ok::<_, CollectError>((identity, bytes))
        })
        .await
        .map_err(unavailable)??;
        if final_bytes != bytes
            || verified.package_version != version
            || final_runtime.executable().as_ref() != Some(&verified.entry)
        {
            return Err(CollectError::Unreachable);
        }
        Ok(Some(payload))
    })
    .await
    .map_err(|_| CollectError::Timeout)?
}
#[cfg(test)]
#[path = "attested_status_tests.rs"]
mod tests;
