//! Real v6 capture/wrapper, authenticated owner recovery and native filesystem.
//! No network, actual keyring, global HOME or daily configuration access.
use super::super::{
    config,
    objects_overlay_v2::{ChannelSelectionV2, SelectionV2, UsageV2},
    saved_objects_capture_v2::{CaptureCredentialsV2, CaptureRequestV2},
    FileRecoveryKeysV2,
};
use super::*;
use serde_json::Value;
use std::sync::Arc;
use tempfile::TempDir;
const PASSWORD: &str = "synthetic-output-wrapper-password";
struct Keys;
impl FileRecoveryKeysV2 for Keys {
    fn key(&self, _d: &Path, _h: &Path, _create: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
        Ok(Zeroizing::new([19; 32]))
    }
}
struct Credentials;
impl CaptureCredentialsV2 for Credentials {
    fn channel_key(
        &self,
        _h: &Path,
        _s: &super::super::saved_objects_capture_v2::SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        panic!("output must never read unselected credentials");
    }
}
struct Fixture {
    t: TempDir,
    d: PathBuf,
    h: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let t = tempfile::Builder::new()
            .prefix("ocxd023-filetransfer-output-")
            .tempdir_in(std::env::temp_dir())
            .unwrap();
        let d = t.path().join("D");
        let h = t.path().join("H");
        platform::create_private_dir_all(&d.join("manager-state"), 0o700).unwrap();
        platform::create_private_dir_all(&h, 0o700).unwrap();
        atomic_write(&h.join("config.json"),br#"{"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://example.invalid/v1","authMode":"none","models":["vendor/custom"]}},"untouched":true}"#,0o600).unwrap();
        Self { t, d, h }
    }
    fn owner(&self) -> FileTransferOwnerV2 {
        FileTransferOwnerV2::new(self.d.clone(), self.h.clone(), Arc::new(Credentials))
            .unwrap()
            .with_recovery_keys(Arc::new(Keys))
    }
    fn plan(&self, o: &mut FileTransferOwnerV2) -> ExportPlanV2 {
        let id = config::with_saved_config(&self.d, &self.h, |_, p, _, _, _, _| {
            Ok(p.channels[0].id.clone())
        })
        .unwrap();
        o.prepare_export(
            CaptureRequestV2 {
                selection: SelectionV2 {
                    usage: UsageV2::LocalFile,
                    channels: vec![ChannelSelectionV2 {
                        portable_channel_id: id,
                        model_ids: None,
                        family_ids: None,
                        include_connection: true,
                        include_key: false,
                        include_authorization: false,
                    }],
                    ..Default::default()
                },
                mcp_clients: Default::default(),
                assets_reviewed: false,
            },
            Some(Zeroizing::new(PASSWORD.into())),
        )
        .unwrap()
    }
    fn custom(&self, name: &str) -> PathBuf {
        let p = self.t.path().join(name);
        std::fs::create_dir(&p).unwrap();
        p
    }
}
fn code<T>(r: ModelResult<T>) -> String {
    match r {
        Err(e) => e.code,
        Ok(_) => panic!("expected refusal"),
    }
}
#[test]
fn default_layout_manifest_and_real_receipt_preserve_original_v6_bytes() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let before = std::fs::read(f.h.join("config.json")).unwrap();
    assert!(!o.has_export_destination(&p.id).unwrap());
    assert!(o.export_output(&p.id).unwrap().is_none());
    let expected = o.output_original(&p.id).unwrap().1;
    let out = o.save_export_at(&p.id, None).unwrap();
    let a = o.read_output_allocation(&p.id).unwrap().unwrap();
    assert!(a.complete);
    assert_eq!(a.base, f.d.join("exports").canonicalize().unwrap());
    assert_eq!(Path::new(&out.path), calculated_path(&a));
    assert_eq!(out.filename, format!("{}.ocxdconf", stamp(&a)));
    assert_eq!(std::fs::read(&out.path).unwrap(), expected.as_slice());
    assert_eq!(
        out.receipt.file_digest,
        schema_v2::digest(&std::fs::read(&out.path).unwrap())
    );
    assert_eq!(out.receipt.length, expected.len());
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(Path::new(&out.directory).join(MANIFEST)).unwrap())
            .unwrap();
    assert_eq!(manifest["schema"]["envelope"]["version"], 3);
    assert_eq!(manifest["schema"]["document"]["version"], 2);
    assert_eq!(manifest["schema"]["model"]["formatVersion"], 6);
    assert_eq!(manifest["schema"]["model"]["schemaVersion"], 2);
    assert_eq!(manifest["filename"], out.filename);
    assert_eq!(manifest["digest"], p.digest);
    assert_eq!(manifest["counts"]["channels"], 1);
    assert_eq!(manifest["counts"]["channelKeys"], 0);
    let record = std::fs::read(allocation_path(&f.d, &p.id).unwrap()).unwrap();
    assert!(!String::from_utf8_lossy(&record).contains(PASSWORD));
    assert!(!String::from_utf8_lossy(&record).contains("document_b64"));
    assert_eq!(o.export_output(&p.id).unwrap(), Some(out.clone()));
    assert_eq!(
        o.export_directory(&p.id).unwrap(),
        PathBuf::from(&out.directory)
    );
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), before);
    assert!(!o.cancel_export(&p.id).unwrap());
}
#[test]
fn custom_existing_root_and_same_operation_restart_never_reselect_today_or_reencrypt() {
    let f = Fixture::new();
    let custom = f.custom("custom folder 中文");
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let out = o.save_export_at(&p.id, Some(&custom)).unwrap();
    let record = std::fs::read(allocation_path(&f.d, &p.id).unwrap()).unwrap();
    let bytes = std::fs::read(&out.path).unwrap();
    drop(o);
    std::fs::write(
        f.h.join("config.json"),
        b"later unrelated invalid user configuration",
    )
    .unwrap();
    let mut o = f.owner();
    assert!(o.has_export_destination(&p.id).unwrap());
    assert_eq!(o.save_export_at(&p.id, None).unwrap(), out);
    assert_eq!(o.save_export_at(&p.id, Some(&custom)).unwrap(), out);
    assert_eq!(std::fs::read(&out.path).unwrap(), bytes);
    assert_eq!(
        std::fs::read(allocation_path(&f.d, &p.id).unwrap()).unwrap(),
        record
    );
    assert!(!f.d.join("exports").exists());
    let other = f.custom("different");
    assert_eq!(
        code(o.save_export_at(&p.id, Some(&other))),
        "file_output_destination_frozen"
    );
    assert!(std::fs::read_dir(other).unwrap().next().is_none());
}
#[test]
fn invalid_custom_never_allocates_arbitrary_directory_or_marks_attempted() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let absent = f.t.path().join("not-created-by-native-picker");
    assert!(o.save_export_at(&p.id, Some(&absent)).is_err());
    assert!(!absent.exists());
    assert!(o
        .save_export_at(&p.id, Some(Path::new("relative")))
        .is_err());
    assert!(!o.has_export_destination(&p.id).unwrap());
    assert!(o.cancel_export(&p.id).unwrap());
}
#[test]
fn real_directory_failure_freezes_destination_and_restarts_original_clock_date_bytes() {
    let f = Fixture::new();
    let base = f.custom("custom");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let blocked = base.join(date);
    std::fs::write(&blocked, b"blocked date directory").unwrap();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    assert!(o.save_export_at(&p.id, Some(&base)).is_err());
    assert!(o.has_export_destination(&p.id).unwrap());
    assert!(o.export_output(&p.id).unwrap().is_none());
    assert_eq!(code(o.export_directory(&p.id)), "file_output_incomplete");
    let a = o.read_output_allocation(&p.id).unwrap().unwrap();
    let path = a.path.clone();
    let expected = o.output_original(&p.id).unwrap().1;
    assert!(!o.cancel_export(&p.id).unwrap());
    drop(o);
    std::fs::remove_file(blocked).unwrap();
    let mut o = f.owner();
    let out = o.save_export_at(&p.id, None).unwrap();
    assert_eq!(out.path, path.to_str().unwrap());
    assert_eq!(std::fs::read(out.path).unwrap(), expected.as_slice());
}
#[test]
fn completed_payload_or_manifest_changed_or_missing_never_recreated_or_overwritten() {
    for target in ["payload", "manifest"] {
        for remove in [false, true] {
            let f = Fixture::new();
            let mut o = f.owner();
            let p = f.plan(&mut o);
            let out = o.save_export_at(&p.id, None).unwrap();
            let path = if target == "payload" {
                PathBuf::from(&out.path)
            } else {
                Path::new(&out.directory).join(MANIFEST)
            };
            if remove {
                std::fs::remove_file(&path).unwrap();
            } else {
                std::fs::write(&path, b"user later changes").unwrap();
            }
            drop(o);
            let mut o = f.owner();
            assert!(o.has_export_destination(&p.id).unwrap());
            assert!(o.save_export_at(&p.id, None).is_err());
            assert!(o.export_output(&p.id).is_err());
            assert!(o.export_directory(&p.id).is_err());
            if remove {
                assert!(!path.exists());
            } else {
                assert_eq!(std::fs::read(path).unwrap(), b"user later changes");
            }
        }
    }
}
#[test]
fn partial_output_manifest_failure_resumes_without_new_package_or_bytes() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let out = o.save_export_at(&p.id, None).unwrap();
    // Persisted incomplete allocation models interruption after payload publish
    // but before manifest publication/owner finish/completion persistence.
    let mut a = o.read_output_allocation(&p.id).unwrap().unwrap();
    a.complete = false;
    a.receipt = None;
    o.persist_allocation(&a).unwrap();
    let manifest = Path::new(&out.directory).join(MANIFEST);
    std::fs::remove_file(&manifest).unwrap();
    let bytes = std::fs::read(&out.path).unwrap();
    std::fs::create_dir(&manifest).unwrap();
    assert!(o.save_export_at(&p.id, None).is_err());
    assert_eq!(std::fs::read(&out.path).unwrap(), bytes);
    std::fs::remove_dir(&manifest).unwrap();
    drop(o);
    let mut o = f.owner();
    assert_eq!(o.save_export_at(&p.id, None).unwrap(), out);
    assert_eq!(std::fs::read(out.path).unwrap(), bytes);
}
#[test]
fn exclusive_publish_and_readback_reject_conflicting_objects_without_truncation() {
    let f = Fixture::new();
    let base = f.custom("output");
    let path = base.join("sample.ocxdconf");
    std::fs::write(&path, b"existing unrelated output").unwrap();
    assert!(publish_exact(&base, &path, b"wanted output").is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"existing unrelated output");
    std::fs::remove_file(&path).unwrap();
    publish_exact(&base, &path, b"wanted output").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"wanted output");
    assert!(!temp_path(&path).unwrap().exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let m = std::fs::metadata(&path).unwrap();
        assert_eq!(m.nlink(), 1);
        assert_eq!(m.permissions().mode() & 0o777, 0o600);
    }
    let temp = temp_path(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&temp, b"partial crashed temp").unwrap();
    publish_exact(&base, &path, b"wanted output").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"wanted output");
}
#[test]
fn original_allocation_tampering_fails_even_with_recomputed_checksum() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    o.save_export_at(&p.id, None).unwrap();
    let record = allocation_path(&f.d, &p.id).unwrap();
    let raw = std::fs::read(&record).unwrap();
    let mut checked: CheckedAllocation = serde_json::from_slice(&raw).unwrap();
    let elsewhere = f.custom("elsewhere");
    checked.allocation.base = elsewhere;
    checked.allocation.path = calculated_path(&checked.allocation);
    checked.checksum = hash(&checked.allocation).unwrap();
    atomic_write(&record, &serde_json::to_vec(&checked).unwrap(), 0o600).unwrap();
    assert_eq!(
        code(o.has_export_destination(&p.id)),
        "file_output_record_invalid"
    );
    assert!(o.export_directory(&p.id).is_err());
    assert!(o.save_export_at(&p.id, None).is_err());
    atomic_write(&record, b"{broken", 0o600).unwrap();
    assert!(o.has_export_destination(&p.id).is_err());
}
#[test]
fn output_record_never_substitutes_for_original_sealed_owner_receipt() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let out = o.save_export_at(&p.id, None).unwrap();
    let recovery = super::super::recovery_path(&f.d, &p.id).unwrap();
    std::fs::write(&recovery, b"corrupt original sealed operation").unwrap();
    assert!(o.has_export_destination(&p.id).is_err());
    assert!(o.export_output(&p.id).is_err());
    assert!(o.export_directory(&p.id).is_err());
    let unchanged = std::fs::read(&out.path).unwrap();
    assert!(o.save_export_at(&p.id, None).is_err());
    assert_eq!(std::fs::read(out.path).unwrap(), unchanged);
}
#[cfg(unix)]
#[test]
fn symlinks_and_unrelated_package_entries_fail_closed() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let base = f.custom("chosen");
    let alternate = f.custom("alternate");
    let link = f.t.path().join("picked-link");
    std::os::unix::fs::symlink(&base, &link).unwrap();
    assert!(o.save_export_at(&p.id, Some(&link)).is_err());
    let out = o.save_export_at(&p.id, Some(&base)).unwrap();
    let path = PathBuf::from(&out.path);
    std::fs::remove_file(&path).unwrap();
    let user = alternate.join("user-file");
    std::fs::write(&user, b"user bytes").unwrap();
    std::os::unix::fs::symlink(&user, &path).unwrap();
    assert!(o.save_export_at(&p.id, None).is_err());
    assert_eq!(std::fs::read(user).unwrap(), b"user bytes");
    std::fs::remove_file(path).unwrap();
    std::fs::write(
        Path::new(&out.directory).join("unrelated.txt"),
        b"unrelated",
    )
    .unwrap();
    assert!(o.save_export_at(&p.id, None).is_err());
}

#[test]
fn completed_allocation_requires_original_authenticated_owner_receipt() {
    let f = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    let out = o.save_export_at(&p.id, None).unwrap();
    let (mut frozen, revision) = o.load_recovery(&p.id).unwrap();
    match &mut frozen {
        FrozenOperation::Export { receipt, .. } => *receipt = None,
        _ => panic!("export"),
    }
    o.save_recovery(&p.id, Some(&revision), &frozen).unwrap();
    let before = std::fs::read(&out.path).unwrap();
    assert_eq!(code(o.export_output(&p.id)), "file_output_receipt_missing");
    assert_eq!(
        code(o.save_export_at(&p.id, None)),
        "file_output_receipt_missing"
    );
    assert_eq!(std::fs::read(out.path).unwrap(), before);
}
#[test]
fn copied_allocation_cannot_open_another_data_roots_directory() {
    let f = Fixture::new();
    let other = Fixture::new();
    let mut o = f.owner();
    let p = f.plan(&mut o);
    o.save_export_at(&p.id, None).unwrap();
    let source = allocation_path(&f.d, &p.id).unwrap();
    let target = allocation_path(&other.d, &p.id).unwrap();
    atomic_write(&target, &std::fs::read(source).unwrap(), 0o600).unwrap();
    assert!(other.owner().has_export_destination(&p.id).is_err());
    assert!(other.owner().export_directory(&p.id).is_err());
}
