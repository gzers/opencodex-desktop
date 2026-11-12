use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    process::{Command, Stdio},
    time::Duration,
};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    client: PinnedDirectory,
}
#[test]
#[ignore = "explicit retained owned Profile, read-only final receipt/tree evidence only"]
fn inspect_owned_completion_receipt_without_rewriting_it() {
    use base64::Engine;
    let root = PathBuf::from(
        std::env::var_os("OCXD024_COMPLETION_READBACK_ROOT").expect("owned profile root"),
    );
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
            && root.ancestors().any(|p| p.file_name().is_some_and(|n| n
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")))
    );
    let profile = root.join("profile");
    let data = profile.join(".codex-desktop/desktop");
    let client = profile.join(".codex");
    let index = rusqlite::Connection::open_with_flags(
        data.join("manager-state/client-session-index.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let raw: Vec<u8> = index
        .query_row("SELECT document FROM current_session", [], |r| r.get(0))
        .unwrap();
    index.close().unwrap();
    let head: session_index::Head = serde_json::from_slice(&raw).unwrap();
    let refs = head.references.unwrap();
    let originals = Originals::open_for_profile(
        &data,
        &client,
        &profile,
        &refs.attempt,
        &refs.original_sha256,
        &refs.phases,
    )
    .unwrap();
    let Image::File {
        bytes, protection, ..
    } = capture_file(&originals.directory, "completion-readback.json", MAX_FRAME).unwrap()
    else {
        panic!("receipt absent");
    };
    require_private(&protection, false).unwrap();
    let receipt: Receipt = serde_json::from_slice(&bytes).unwrap();
    assert!(receipt.references == refs && Some(&receipt.plan) == head.restore_plan.as_ref());
    let codex = TreeSnapshot::read(&originals.client).unwrap();
    let grok = GrokSnapshot::read(&originals.profile).unwrap();
    let codex_raw = encode(&codex).unwrap();
    let grok_raw = encode(&grok).unwrap();
    assert_eq!(digest(&codex_raw), receipt.codex_tree_sha256);
    assert_eq!(digest(&grok_raw), receipt.grok_tree_sha256);
    println!(
        "{}",
        serde_json::json!({"receiptSha256":digest(&bytes),"codexTreeBase64":base64::engine::general_purpose::STANDARD.encode(codex_raw),"grokTreeBase64":base64::engine::general_purpose::STANDARD.encode(grok_raw),"independentCurrentHashesMatchReceipt":true,"readOnly":true})
    );
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir_in("/private/tmp").unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("client");
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        fs::write(path.join("config.toml"), b"native='original'").unwrap();
        Self {
            _temp: temp,
            root,
            client: PinnedDirectory::open(&path).unwrap(),
        }
    }
    fn locks(&self) -> [Identity; 2] {
        std::array::from_fn(|i| {
            let path = self.client.path.join(LOCKS[i]);
            fs::write(&path, b"").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            let file = open_at(self.client.file(), OsStr::new(LOCKS[i]), false).unwrap();
            identity(&stat(&file).unwrap())
        })
    }
}
#[test]
fn full_readback_accepts_atomic_original_replacement_but_no_content_or_protection_change() {
    let f = Fixture::new();
    fs::create_dir(f.client.path.join("agents")).unwrap();
    fs::write(f.client.path.join("agents/AGENTS.md"), b"original agent").unwrap();
    let original = TreeSnapshot::read(&f.client).unwrap();
    let path = f.client.path.join("config.toml");
    let old = path.with_extension("old");
    fs::rename(&path, &old).unwrap();
    fs::write(&path, b"native='original'").unwrap();
    fs::remove_file(old).unwrap();
    assert!(original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
    fs::write(
        f.client.path.join("agents/AGENTS.md"),
        b"outside agent edit",
    )
    .unwrap();
    assert!(!original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
    fs::write(f.client.path.join("agents/AGENTS.md"), b"original agent").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(!original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
}
#[test]
fn only_exact_adopted_lock_names_are_exempt_no_stage_backup_or_sidecar() {
    let f = Fixture::new();
    let original = TreeSnapshot::read(&f.client).unwrap();
    f.locks();
    assert!(original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
    for name in [
        ".ocxd-restore-leftover",
        "config.toml.bak-opencodex",
        ".opencodex-native-main.owner.sqlite-journal",
        ".opencodex-native-main.claim.sqlite-journal",
        "nested-empty",
    ] {
        let path = f.client.path.join(name);
        fs::write(&path, b"unexpected").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(
            !original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true),
            "{name}"
        );
        fs::remove_file(path).unwrap();
    }
    fs::create_dir(f.client.path.join("empty")).unwrap();
    assert!(!original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
}
#[test]
fn xattrs_are_not_ignored_by_full_restoration_readback() {
    let f = Fixture::new();
    let original = TreeSnapshot::read(&f.client).unwrap();
    let file = File::open(f.client.path.join("config.toml")).unwrap();
    let key = CString::new("user.ocxd-readback").unwrap();
    let value = b"outside attribute";
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                key.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
                0,
            )
        },
        0
    );
    assert!(!original.restored_matches(&TreeSnapshot::read(&f.client).unwrap(), true));
}
#[test]
fn grok_root_absence_and_plain_tree_have_no_stable_lock_exception() {
    let f = Fixture::new();
    let before = GrokSnapshot::read(&PinnedDirectory::open(&f.root).unwrap()).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(f.root.join(".grok"))
        .unwrap();
    let after = GrokSnapshot::read(&PinnedDirectory::open(&f.root).unwrap()).unwrap();
    assert!(!before.restored_matches(&after));
    fs::write(f.root.join(".grok").join(LOCKS[1]), b"not a Grok exception").unwrap();
    assert!(!after
        .restored_matches(&GrokSnapshot::read(&PinnedDirectory::open(&f.root).unwrap()).unwrap()));
}
#[test]
fn real_stable_pair_requires_both_exclusive_locks_correct_generation_and_budget() {
    let f = Fixture::new();
    let ids = f.locks();
    let deadline = Instant::now() + Duration::from_secs(3);
    prove_released(&f.client, &ids, deadline).unwrap();
    assert!(prove_released(&f.client, &ids, Instant::now()).is_err());
    let mut changed = ids.clone();
    changed[1].inode += 1;
    assert!(prove_released(&f.client, &changed, deadline).is_err());
    let path = f.client.path.join(LOCKS[1]);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(prove_released(&f.client, &ids, deadline).is_err());
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o644
    );
}
#[test]
fn external_stable_owner_contention_refuses_then_release_succeeds() {
    const ROLE: &str = "OCXD_STABLE_PAIR_CHILD";
    const TEST:&str="infrastructure::macos_client_originals::completion::tests::external_stable_owner_contention_refuses_then_release_succeeds";
    if let Some(root) = std::env::var_os(ROLE) {
        let root = PathBuf::from(root);
        assert!(root.starts_with("/private/tmp"));
        let c = rusqlite::Connection::open(root.join("client").join(LOCKS[1])).unwrap();
        c.execute_batch("BEGIN EXCLUSIVE").unwrap();
        fs::write(root.join("held"), b"held").unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while !root.join("release").exists() {
            assert!(Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        c.execute_batch("ROLLBACK").unwrap();
        c.close().unwrap();
        return;
    }
    let f = Fixture::new();
    let ids = f.locks();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(ROLE, &f.root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    while !f.root.join("held").exists() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    let result = prove_released(&f.client, &ids, end);
    fs::write(f.root.join("release"), b"release").unwrap();
    assert!(child.wait().unwrap().success());
    assert!(result.is_err());
    prove_released(&f.client, &ids, end).unwrap();
}
