//! Actual internal ocxd host before any native official Owner has been created.
//! This boundary cannot Start, Stop, restore, adopt a PID or produce a native
//! lifecycle proof. It enables isolated native host/peer continuity validation.
use super::*;
use crate::infrastructure::{
    hash, platform,
    windows_official_runtime024::{
        current_identity, PinnedDirectory, PrivatePipe, PrivateRecord, RetainedPeer,
    },
};
use std::{
    collections::BTreeSet,
    fs::OpenOptions,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EmptyBootstrap {
    pub schema: u32,
    pub scope: String,
    pub expected_sid: String,
    pub expected_session: u32,
    pub profile: PathBuf,
    pub installation: PathBuf,
    pub data: PathBuf,
    pub home: PathBuf,
    pub broker_sha256: String,
    pub manager_sha256: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Rendezvous {
    schema: String,
    instance_id: String,
    pipe_name: String,
    broker_pid: u32,
    broker_filetime: String,
    sid: String,
    data: PathBuf,
    home: PathBuf,
    installation: PathBuf,
    source_digest: String,
    generation: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Challenge {
    schema: u32,
    instance_id: String,
    manager_generation: String,
    challenge: String,
    broker_pid: u32,
    broker_filetime: String,
    scope_digest: String,
}
fn refused(reason: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason.into())
}
fn same(a: &Path, b: &Path) -> bool {
    dunce::simplified(a)
        .to_string_lossy()
        .replace('/', "\\")
        .eq_ignore_ascii_case(&dunce::simplified(b).to_string_lossy().replace('/', "\\"))
}
fn nonsecret_identity(p: &RetainedPeer) -> serde_json::Value {
    let i = p.identity();
    serde_json::json!({"pid":i.pid,"creationFiletime":i.creation_filetime.to_string(),"sid":i.token.sid,"session":i.token.session_id,"elevated":i.token.elevated,"elevationType":i.token.elevation_type,"image":i.executable,"volume":i.executable_id.volume,"fileId":i.executable_id.index.to_string()})
}
struct Admission {
    binding: String,
    _pins: Vec<PinnedDirectory>,
    _images: Vec<std::fs::File>,
    broker: RetainedPeer,
    broker_image: PathBuf,
    manager_image: PathBuf,
}
impl Admission {
    fn open(c: &EmptyBootstrap) -> io::Result<Self> {
        let token = current_identity()?;
        if c.schema != 1
            || c.scope != "isolated-empty-owner-continuity"
            || token.sid != c.expected_sid
            || token.session_id != c.expected_session
            || token.elevated
            || token.elevation_type == 2
        {
            return Err(refused(
                "exact ordinary SID/session and empty-host scope required",
            ));
        }
        if !same(
            &platform::home_dir().ok_or_else(|| refused("OS Profile missing"))?,
            &c.profile,
        ) {
            return Err(refused(
                "OS Profile differs; environment path is not token authority",
            ));
        }
        // This diagnostic mode is strictly restricted to a task-owned subtree.
        // Existing daily .codex/.grok/manager config is neither read nor written.
        let base = c
            .installation
            .parent()
            .ok_or_else(|| refused("task base missing"))?;
        if base.parent() != Some(c.profile.as_path())
            || !base
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("ocx024-h-"))
            || c.installation != base.join("I")
            || c.data != base.join("D")
            || c.home != base.join("H")
        {
            return Err(refused("explicit task-owned I/D/H only"));
        }
        let mut pins = vec![];
        for p in [&c.profile, base, &c.installation, &c.data, &c.home] {
            pins.push(PinnedDirectory::open(p)?);
        }
        let broker_image = c.installation.join("ocxd.exe");
        let manager_image = c.installation.join("opencodex-desktop.exe");
        let mut images = vec![];
        for (p, expected) in [
            (&broker_image, &c.broker_sha256),
            (&manager_image, &c.manager_sha256),
        ] {
            if expected.len() != 64 {
                return Err(refused("pinned image digest shape"));
            }
            images.push(
                OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .custom_flags(0x200000)
                    .open(p)?,
            );
            if hash::sha256_file(p).map_err(|e| refused(e.to_string()))? != *expected {
                return Err(refused("exact task image digest changed"));
            }
        }
        let broker = RetainedPeer::current()?;
        broker.verify_image(&broker_image)?;
        broker.no_outer_job()?;
        let identity = nonsecret_identity(&broker);
        let dirs:Vec<_>=pins.iter().map(|p|serde_json::json!({"path":p.path(),"volume":p.identity().volume,"id":p.identity().index.to_string()})).collect();
        let binding=hash::sha256_hex(&serde_json::to_vec(&serde_json::json!({"sid":token.sid,"session":token.session_id,"broker":identity,"directories":dirs,"brokerSha":c.broker_sha256,"managerSha":c.manager_sha256,"scope":c.scope})).map_err(io::Error::other)?);
        Ok(Self {
            binding,
            _pins: pins,
            _images: images,
            broker,
            broker_image,
            manager_image,
        })
    }
    fn recheck(&self) -> io::Result<()> {
        self.broker.no_outer_job()?;
        self.broker.verify_image(&self.broker_image)?;
        for p in &self._pins {
            p.recheck()?;
        }
        Ok(())
    }
}
/// Single-use inherited bootstrap framing: LE u32 JSON, bounded and EOF. The
/// native dispatcher requires pipe stdin, never a file, argv secret or env token.
pub fn read_bootstrap(mut input: impl std::io::Read) -> io::Result<EmptyBootstrap> {
    let mut n = [0u8; 4];
    input.read_exact(&mut n)?;
    let n = u32::from_le_bytes(n) as usize;
    if n == 0 || n > 65536 {
        return Err(refused("bootstrap length bound"));
    }
    let mut b = vec![0; n];
    input.read_exact(&mut b)?;
    let mut extra = [0u8; 1];
    if input.read(&mut extra)? != 0 {
        return Err(refused("trailing bootstrap bytes"));
    }
    serde_json::from_slice(&b).map_err(io::Error::other)
}
pub fn run_empty(c: EmptyBootstrap) -> io::Result<()> {
    let a = Admission::open(&c)?;
    let instance = uuid::Uuid::new_v4();
    let name = hash::sha256_hex(format!("{}:{instance}", a.binding).as_bytes());
    let pipe = PrivatePipe::create(&name)?;
    let rendezvous = Rendezvous {
        schema: "windows-owner-rendezvous-v1".into(),
        instance_id: instance.to_string(),
        pipe_name: pipe.name().into(),
        broker_pid: a.broker.identity().pid,
        broker_filetime: a.broker.identity().creation_filetime.to_string(),
        sid: c.expected_sid.clone(),
        data: c.data.clone(),
        home: c.home.clone(),
        installation: c.installation.clone(),
        source_digest: a.binding.clone(),
        generation: "0".into(),
    };
    let mut record = PrivateRecord::create(
        &c.data.join("rendezvous.json"),
        &serde_json::to_vec(&rendezvous).map_err(io::Error::other)?,
        65536,
    )?;
    let original_record = record.read()?;
    // The journal is nonsecret immutable diagnostic evidence. It is not the
    // client-originals phase journal and cannot grant restoration or authority.
    let mut events = platform::create_new_private_file(&c.data.join("host-events.jsonl"), 0o600)?;
    event(
        &mut events,
        "host-created",
        serde_json::json!({"identity":nonsecret_identity(&a.broker),"noOuterJob":true,"scope":c.scope,"officialOwnerCreated":false,"allAccepted":false}),
    )?;
    let mut generation = 0u64;
    let mut used = BTreeSet::new();
    loop {
        a.recheck()?;
        let peer = match pipe.accept(Instant::now() + Duration::from_secs(5)) {
            Ok(p) => p,
            Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e),
        };
        let window = Instant::now() + Duration::from_secs(5);
        let transaction = (|| -> io::Result<bool> {
            a.recheck()?;
            peer.verify_image(&a.manager_image)?;
            if peer.identity().token != a.broker.identity().token {
                return Err(refused("same actual ordinary SID/session/token required"));
            }
            generation = generation
                .checked_add(1)
                .ok_or_else(|| refused("generation overflow"))?;
            let challenge = rand::random::<[u8; 32]>()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let hello = Challenge {
                schema: 1,
                instance_id: instance.to_string(),
                manager_generation: generation.to_string(),
                challenge: challenge.clone(),
                broker_pid: a.broker.identity().pid,
                broker_filetime: a.broker.identity().creation_filetime.to_string(),
                scope_digest: a.binding.clone(),
            };
            pipe.write_frame(
                &serde_json::to_vec(&hello).map_err(io::Error::other)?,
                window,
            )?;
            let raw = pipe.read_frame(window)?;
            let request = decode(&raw)?;
            if request.instance_id != hello.instance_id
                || request.manager_generation != hello.manager_generation
                || request.challenge != challenge
            {
                return Err(refused(
                    "fresh connection challenge/instance/generation mismatch",
                ));
            }
            let sampled = Instant::now();
            let (now, freq) = qpc()?;
            let deadline = request.deadline_at(now, freq, sampled)?;
            let keys = [
                request.request_id.clone(),
                request.budget.action_id.clone(),
                challenge,
            ];
            if used.len() + 3 > 16384 || keys.iter().any(|k| used.contains(k)) {
                return Err(refused("replayed request/action/nonce"));
            }
            used.extend(keys);
            if pipe.peer()?.identity() != peer.identity() {
                return Err(refused("native connection peer generation changed"));
            }
            peer.verify_image(&a.manager_image)?;
            a.recheck()?;
            if record.read()? != original_record {
                return Err(refused("rendezvous original record changed"));
            }
            // No native Owner exists here. Every service action is Unknown and
            // may never become successful by using a JSON receipt or health.
            let shutdown = matches!(request.operation, Operation::ExplicitRecovery);
            let reply = serde_json::json!({"schema":1,"requestId":request.request_id,"instanceId":hello.instance_id,"managerGeneration":generation.to_string(),"scopeDigest":a.binding,"broker":nonsecret_identity(&a.broker),"result":"unknown","clientRecoveryRequired":true,"officialOwnerCreated":false,"lifecycleAccepted":false,"allAccepted":false,"emptyHostExitRequested":shutdown});
            event(
                &mut events,
                "authenticated-empty-action",
                serde_json::json!({"peer":nonsecret_identity(&peer),"managerGeneration":generation.to_string(),"operation":request.operation,"lifecycleAccepted":false,"emptyHostExitRequested":shutdown}),
            )?;
            pipe.write_frame(
                &serde_json::to_vec(&reply).map_err(io::Error::other)?,
                window.min(deadline.deadline),
            )?;
            {
                // A completed action never authorizes a second request on this
                // challenge. Keep the pipe connected while the client consumes
                // its reply, then explicitly refuse its old-connection replay.
                let late = pipe.read_frame(window.min(deadline.deadline));
                if late.is_ok() {
                    event(
                        &mut events,
                        "same-connection-replay-refused",
                        serde_json::json!({"managerGeneration":generation.to_string(),"lifecycleAccepted":false}),
                    )?;
                    let denial = serde_json::json!({"schema":1,"replayRefused":true,"lifecycleAccepted":false,"allAccepted":false});
                    let _ = pipe.write_frame(
                        &serde_json::to_vec(&denial).map_err(io::Error::other)?,
                        window.min(deadline.deadline),
                    );
                }
            }
            a.recheck()?;
            Ok(shutdown)
        })();
        if let Err(e) = &transaction {
            event(
                &mut events,
                "connection-refused",
                serde_json::json!({"error":e.to_string(),"raw":e.raw_os_error(),"lifecycleAccepted":false}),
            )?;
        }
        let _ = pipe.disconnect();
        if matches!(transaction, Ok(true)) {
            event(
                &mut events,
                "empty-host-explicit-exit",
                serde_json::json!({"forced":false,"officialOwnerCreated":false,"normalOfficialStop":false}),
            )?;
            return Ok(());
        }
    }
}
fn event(out: &mut std::fs::File, name: &str, facts: serde_json::Value) -> io::Result<()> {
    use std::io::Write;
    writeln!(out, "{}", serde_json::json!({"event":name,"facts":facts}))?;
    out.sync_all()
}
/// Synthetic manager helper consumes the exact public request codec and checks
/// the original server handle against the independently held creation identity.
/// It is a native component fixture, never a replacement for actual GUI proof.
pub fn manager_probe(
    data: &Path,
    original: &super::super::windows_official_runtime024::ProcessIdentity,
    operation: &str,
) -> io::Result<serde_json::Value> {
    let started = Instant::now();
    let (start, freq) = qpc()?;
    let deadline = started + Duration::from_secs(5);
    let mut locator = PrivateRecord::open(&data.join("rendezvous.json"), 65536)?;
    let r: Rendezvous = serde_json::from_slice(&locator.read()?).map_err(io::Error::other)?;
    if r.schema != "windows-owner-rendezvous-v1"
        || r.broker_pid != original.pid
        || r.broker_filetime != original.creation_filetime.to_string()
        || r.sid != original.token.sid
    {
        return Err(refused(
            "locator does not match original retained creation identity",
        ));
    }
    let pipe = PrivatePipe::connect_local(&r.pipe_name)?;
    let peer = pipe.peer()?;
    if peer.identity() != original {
        return Err(refused("native server differs from original broker"));
    }
    peer.no_outer_job()?;
    let hello: Challenge =
        serde_json::from_slice(&pipe.read_frame(deadline)?[4..]).map_err(io::Error::other)?;
    if hello.schema != 1
        || hello.instance_id != r.instance_id
        || hello.broker_pid != original.pid
        || hello.broker_filetime != original.creation_filetime.to_string()
        || hello.scope_digest != r.source_digest
    {
        return Err(refused("native-bound hello differs"));
    }
    let request = serde_json::json!({"schema":1,"requestId":uuid::Uuid::new_v4().to_string(),"instanceId":hello.instance_id,"managerGeneration":hello.manager_generation,"challenge":hello.challenge,"operation":operation,"budget":{"actionId":uuid::Uuid::new_v4().to_string(),"startedQpc":start.to_string(),"deadlineQpc":(start+5*freq).to_string(),"stopDeadlineQpc":(start+5*freq).to_string(),"qpcFrequency":freq.to_string()}});
    pipe.write_frame(
        &serde_json::to_vec(&request).map_err(io::Error::other)?,
        deadline,
    )?;
    let mut response: serde_json::Value =
        serde_json::from_slice(&pipe.read_frame(deadline)?[4..]).map_err(io::Error::other)?;
    if response["allAccepted"] != false
        || response["lifecycleAccepted"] != false
        || response["officialOwnerCreated"] != false
        || response["scopeDigest"] != r.source_digest
        || response["requestId"] != request["requestId"]
    {
        return Err(refused(
            "empty host response crossed service authority boundary",
        ));
    }
    {
        let raw = serde_json::to_vec(&request).map_err(io::Error::other)?;
        let replay = pipe
            .write_frame(&raw, deadline)
            .and_then(|_| pipe.read_frame(deadline));
        match replay {
            Err(_) => {
                response["oldConnectionReplayRefused"] = serde_json::json!(true);
            }
            Ok(bytes) => {
                let denial: serde_json::Value =
                    serde_json::from_slice(&bytes[4..]).map_err(io::Error::other)?;
                if denial["replayRefused"] != true || denial["lifecycleAccepted"] != false {
                    return Err(refused("old connection action replay not refused"));
                }
                response["oldConnectionReplayRefused"] = serde_json::json!(true);
            }
        }
    }
    Ok(response)
}
pub fn run_from_inherited_bootstrap() -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    if unsafe {
        windows_sys::Win32::Storage::FileSystem::GetFileType(std::io::stdin().as_raw_handle())
    } != 3
    {
        return Err(refused(
            "internal owner requires one private inherited pipe bootstrap",
        ));
    }
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = tx.send(read_bootstrap(std::io::stdin().lock()));
    });
    let config = rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
    run_empty(config)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(bytes: &[u8]) -> Vec<u8> {
        let mut framed = (bytes.len() as u32).to_le_bytes().to_vec();
        framed.extend_from_slice(bytes);
        framed
    }
    #[test]
    fn wl2_h_bootstrap_rejects_duplicate_fields_extra_bytes_and_oversized_input() {
        let config = serde_json::json!({"schema":1,"scope":"isolated-empty-owner-continuity","expectedSid":"S-1-5-21-synthetic","expectedSession":7,"profile":"C:/synthetic","installation":"C:/synthetic/ocx024-h-test/I","data":"C:/synthetic/ocx024-h-test/D","home":"C:/synthetic/ocx024-h-test/H","brokerSha256":"a".repeat(64),"managerSha256":"b".repeat(64)});
        let json = serde_json::to_string(&config).unwrap();
        assert!(read_bootstrap(std::io::Cursor::new(frame(json.as_bytes()))).is_ok());
        let duplicate = json.replacen("{", "{\"schema\":1,", 1);
        assert!(read_bootstrap(std::io::Cursor::new(frame(duplicate.as_bytes()))).is_err());
        let mut trailing = frame(json.as_bytes());
        trailing.push(b' ');
        assert!(read_bootstrap(std::io::Cursor::new(trailing)).is_err());
        let mut partial = frame(json.as_bytes());
        partial.pop();
        assert!(read_bootstrap(std::io::Cursor::new(partial)).is_err());
        assert!(read_bootstrap(std::io::Cursor::new(frame(&vec![b' '; 65537]))).is_err());
        assert!(read_bootstrap(std::io::Cursor::new(frame(b"{}"))).is_err());
        assert!(read_bootstrap(std::io::Cursor::new(frame(&[0xff]))).is_err());
    }
}
