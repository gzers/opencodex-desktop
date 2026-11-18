//! Synthetic proof of the external repair file steps. This module has no
//! entry point for an existing scene and cannot qualify a repair as complete.
use super::bounded::{native_stamp, SnapshotData};
use crate::backend::{self, PinnedDirectory, PrivateRecord, RetainedFileMove};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    time::{Duration, Instant},
};

const RECORD_LIMIT: usize = 128 * 1024;

#[derive(PartialEq)]
struct FileImage {
    security: SnapshotData,
    bytes: Vec<u8>,
}

fn read_image(path: &Path, end: Instant) -> io::Result<FileImage> {
    let mut file = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(1 | 4)
        .custom_flags(0x200000)
        .open(path)?;
    let security = native_stamp(&file, end).map_err(io::Error::other)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 2 * 1024 * 1024 || bytes.len() as u64 != file.metadata()?.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "file byte bound",
        ));
    }
    Ok(FileImage { security, bytes })
}

struct Scene {
    client: PinnedDirectory,
    custody: PinnedDirectory,
    retired: PinnedDirectory,
    records: PinnedDirectory,
    root: Option<tempfile::TempDir>,
    original: FileImage,
    working: FileImage,
    started: Instant,
    end: Instant,
}

struct FileStep<'a> {
    from: &'a PinnedDirectory,
    to: &'a PinnedDirectory,
    target_name: &'static str,
    expected: &'a FileImage,
    intent: &'static str,
    held: RetainedFileMove,
}

impl FileStep<'_> {
    fn verify_source(&mut self, end: Instant) -> io::Result<()> {
        self.from.recheck()?;
        self.to.recheck()?;
        self.held.verify(end)?;
        if &read_image(self.held.location(), end)? != self.expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "retained source changed since preparation",
            ));
        }
        Ok(())
    }
}

struct PreparedScene<'a> {
    scene: &'a Scene,
    retire: FileStep<'a>,
    restore: FileStep<'a>,
}

impl PreparedScene<'_> {
    fn run(mut self) -> io::Result<()> {
        self.retire.verify_source(self.scene.end)?;
        self.restore.verify_source(self.scene.end)?;
        self.scene
            .require_members(&["settings"], &["held"], &[], &[])?;
        self.scene.move_file(&mut self.retire, |_| Ok(()))?;
        self.scene.move_file(&mut self.restore, |_| Ok(()))?;
        self.retire.verify_source(self.scene.end)?;
        self.restore.verify_source(self.scene.end)?;
        self.scene.require_members(
            &["settings"],
            &[],
            &["work"],
            &["OriginalReturnIntent.json", "RetireIntent.json"],
        )
    }
}

impl Scene {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("ocx024-r7-external-steps-")
            .tempdir()
            .unwrap();
        let client = backend::create_private_directory(&root.path().join("client")).unwrap();
        let custody = backend::create_private_directory(&root.path().join("custody")).unwrap();
        let retired = backend::create_private_directory(&root.path().join("retired")).unwrap();
        let records =
            backend::create_private_directory(&root.path().join("external-records")).unwrap();
        fs::write(custody.path().join("held"), b"synthetic original\n").unwrap();
        let target = client.path().join("settings");
        fs::write(&target, b"synthetic working generation\n").unwrap();
        let started = Instant::now();
        let end = started + Duration::from_secs(10);
        Self {
            original: read_image(&custody.path().join("held"), end).unwrap(),
            working: read_image(&target, end).unwrap(),
            client,
            custody,
            retired,
            records,
            root: Some(root),
            started,
            end,
        }
    }

    fn check_time(&self) -> io::Result<()> {
        if Instant::now() >= self.end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }

    fn retain(&self, retire_work: bool) -> io::Result<FileStep<'_>> {
        self.check_time()?;
        let (from, source_name, to, target_name, expected, intent) = if retire_work {
            (
                &self.client,
                "settings",
                &self.retired,
                "work",
                &self.working,
                "RetireIntent",
            )
        } else {
            (
                &self.custody,
                "held",
                &self.client,
                "settings",
                &self.original,
                "OriginalReturnIntent",
            )
        };
        let source = from.path().join(source_name);
        from.recheck()?;
        to.recheck()?;
        let held = RetainedFileMove::open(&source, self.end)?;
        let expected_security = serde_json::to_value(&expected.security)?;
        if &read_image(&source, self.end)? != expected
            || held.identity().index.to_string() != expected_security["fileId"]
            || held.identity().volume != expected_security["volume"]
            || from.identity().volume != to.identity().volume
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "native source differs from fixed fixture image",
            ));
        }
        self.check_time()?;
        Ok(FileStep {
            from,
            to,
            target_name,
            expected,
            intent,
            held,
        })
    }

    fn require_members(
        &self,
        client: &[&str],
        custody: &[&str],
        retired: &[&str],
        records: &[&str],
    ) -> io::Result<()> {
        for (parent, expected) in [
            (&self.client, client),
            (&self.custody, custody),
            (&self.retired, retired),
            (&self.records, records),
        ] {
            self.check_time()?;
            parent.recheck()?;
            let mut seen = Vec::with_capacity(expected.len());
            for entry in fs::read_dir(parent.path())? {
                self.check_time()?;
                let name = entry?.file_name();
                if !expected.iter().any(|leaf| name == *leaf) || seen.contains(&name) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unexpected fixture directory member",
                    ));
                }
                seen.push(name);
            }
            if seen.len() != expected.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "missing fixture directory member",
                ));
            }
            parent.recheck()?;
        }
        self.check_time()
    }

    fn prepare(&self) -> io::Result<PreparedScene<'_>> {
        self.require_members(&["settings"], &["held"], &[], &[])?;
        let retire = self.retain(true)?;
        let restore = self.retain(false)?;
        self.require_members(&["settings"], &["held"], &[], &[])?;
        Ok(PreparedScene {
            scene: self,
            retire,
            restore,
        })
    }

    fn move_file(
        &self,
        step: &mut FileStep<'_>,
        after_intent: impl FnOnce(&Path) -> io::Result<()>,
    ) -> io::Result<()> {
        self.check_time()?;
        step.verify_source(self.end)?;
        let source = step.held.location().to_owned();
        let target = step.to.path().join(step.target_name);
        require_absent(&target)?;
        self.records.recheck()?;
        let payload = serde_json::to_vec(&serde_json::json!({
            "schema": "windows-external-preresume-repair-v1",
            "scope": "synthetic-file-step-only",
            "event": step.intent,
            "from": source,
            "to": target,
            "fromParentId": step.from.identity().index.to_string(),
            "toParentId": step.to.identity().index.to_string(),
            "object": step.expected.security,
            "bytesSha256": crate::infrastructure::hash::sha256_hex(&step.expected.bytes),
            "deadlineMs": self.end.duration_since(self.started).as_millis(),
            "normalStop": false, "closed": false, "publicStartOpen": false,
        }))?;
        let record_path = self.records.path().join(format!("{}.json", step.intent));
        self.check_time()?;
        let mut record = PrivateRecord::create(&record_path, &payload, RECORD_LIMIT)?;
        if record.read()? != payload {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "intent readback differs",
            ));
        }
        after_intent(&target)?;
        step.verify_source(self.end)?;
        self.check_time()?;
        step.held
            .move_into(step.to.path(), step.target_name, self.end)?;
        require_absent(&source)?;
        step.verify_source(self.end)?;
        if record.read()? != payload {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "intent readback differs after native move",
            ));
        }
        self.check_time()
    }

    fn step(
        &self,
        retire_work: bool,
        after_intent: impl FnOnce(&Path) -> io::Result<()>,
    ) -> io::Result<()> {
        self.move_file(&mut self.retain(retire_work)?, after_intent)
    }

    fn finish(mut self) {
        let root = self.root.take().unwrap();
        let path = root.path().to_owned();
        drop(self);
        root.close().unwrap();
        assert!(!path.exists());
        println!("synthetic fixture removed: {}", path.display());
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some(root) = self.root.take() {
                eprintln!(
                    "failed synthetic external repair fixture retained: {}",
                    root.keep().display()
                );
            }
        }
    }
}

fn require_absent(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.raw_os_error() == Some(2) => Ok(()),
        Err(error) => Err(error),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "destination occupied",
        )),
    }
}

#[test]
fn r7_external_steps_retire_then_return_bound_native_objects() {
    let scene = Scene::new();
    scene.step(true, |_| Ok(())).unwrap();
    scene.step(false, |_| Ok(())).unwrap();
    assert!(
        read_image(&scene.client.path().join("settings"), scene.end).unwrap() == scene.original
    );
    assert!(read_image(&scene.retired.path().join("work"), scene.end).unwrap() == scene.working);
    assert_eq!(fs::read_dir(scene.records.path()).unwrap().count(), 2);
    scene.finish();
}

#[test]
fn r7_external_steps_collision_after_durable_intent_keeps_exact_prefix() {
    let scene = Scene::new();
    scene.step(true, |_| Ok(())).unwrap();
    let error = scene
        .step(false, |destination| {
            fs::write(destination, b"independent conflict\n")
        })
        .unwrap_err();
    println!(
        "collision raw={:?} kind={:?}",
        error.raw_os_error(),
        error.kind()
    );
    assert!(matches!(error.raw_os_error(), Some(80 | 183)));
    assert_eq!(
        fs::read(scene.client.path().join("settings")).unwrap(),
        b"independent conflict\n"
    );
    assert!(read_image(&scene.custody.path().join("held"), scene.end).unwrap() == scene.original);
    assert!(read_image(&scene.retired.path().join("work"), scene.end).unwrap() == scene.working);
    let original_intent = scene.records.path().join("OriginalReturnIntent.json");
    assert!(PrivateRecord::open(&original_intent, RECORD_LIMIT).is_ok());
    assert_eq!(fs::read_dir(scene.records.path()).unwrap().count(), 2);
    scene.finish();
}

#[test]
fn r7_external_steps_publish_failure_prevents_any_move() {
    let scene = Scene::new();
    let existing = scene.records.path().join("RetireIntent.json");
    let previous = PrivateRecord::create(&existing, b"existing bytes", RECORD_LIMIT).unwrap();
    let error = scene
        .step(true, |_| panic!("publication failed before this point"))
        .unwrap_err();
    println!(
        "publication raw={:?} kind={:?}",
        error.raw_os_error(),
        error.kind()
    );
    assert!(matches!(error.raw_os_error(), Some(80 | 183)));
    assert_eq!(fs::read(&existing).unwrap(), b"existing bytes");
    assert!(read_image(&scene.client.path().join("settings"), scene.end).unwrap() == scene.working);
    assert!(read_image(&scene.custody.path().join("held"), scene.end).unwrap() == scene.original);
    assert!(fs::read_dir(scene.retired.path()).unwrap().next().is_none());
    drop(previous);
    scene.finish();
}

#[test]
fn r7_external_steps_unreleased_source_handle_retains_original_after_retirement() {
    let scene = Scene::new();
    let source = scene.custody.path().join("held");
    let remote_pin = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(1)
        .open(&source)
        .unwrap();
    scene.step(true, |_| Ok(())).unwrap();
    let error = scene
        .step(false, |_| panic!("source access failed before intent"))
        .unwrap_err();
    println!(
        "retained source sharing error raw={:?}",
        error.raw_os_error()
    );
    assert_eq!(error.raw_os_error(), Some(32));
    require_absent(&scene.client.path().join("settings")).unwrap();
    assert!(read_image(&source, scene.end).unwrap() == scene.original);
    assert!(read_image(&scene.retired.path().join("work"), scene.end).unwrap() == scene.working);
    assert_eq!(fs::read_dir(scene.records.path()).unwrap().count(), 1);
    drop(remote_pin);
    scene.finish();
}

#[test]
fn r7_external_steps_prepared_pair_carries_both_native_handles_into_moves() {
    let scene = Scene::new();
    let prepared = scene.prepare().unwrap();
    for path in [
        scene.client.path().join("settings"),
        scene.custody.path().join("held"),
    ] {
        let error = OpenOptions::new()
            .write(true)
            .share_mode(1 | 2 | 4)
            .open(path)
            .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(32));
    }
    prepared.run().unwrap();
    assert!(
        read_image(&scene.client.path().join("settings"), scene.end).unwrap() == scene.original
    );
    assert!(read_image(&scene.retired.path().join("work"), scene.end).unwrap() == scene.working);
    scene.finish();
}

#[test]
fn r7_external_steps_preflight_denies_late_source_before_any_intent() {
    let scene = Scene::new();
    let source = scene.custody.path().join("held");
    let conflict = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(1)
        .open(&source)
        .unwrap();
    let error = scene.prepare().err().expect("blocked original must refuse");
    assert_eq!(error.raw_os_error(), Some(32));
    println!("preflight refused before first intent: raw32");
    scene
        .require_members(&["settings"], &["held"], &[], &[])
        .unwrap();
    assert!(read_image(&scene.client.path().join("settings"), scene.end).unwrap() == scene.working);
    assert!(read_image(&source, scene.end).unwrap() == scene.original);
    drop(conflict);
    scene.finish();
}

#[test]
fn r7_external_steps_preflight_rechecks_members_before_first_intent() {
    let scene = Scene::new();
    let prepared = scene.prepare().unwrap();
    let extra = scene.custody.path().join("extra");
    fs::write(&extra, b"unexpected member").unwrap();
    let error = prepared.run().unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "unexpected fixture directory member");
    scene
        .require_members(&["settings"], &["extra", "held"], &[], &[])
        .unwrap();
    assert!(read_image(&scene.client.path().join("settings"), scene.end).unwrap() == scene.working);
    assert!(read_image(&scene.custody.path().join("held"), scene.end).unwrap() == scene.original);
    assert_eq!(fs::read(extra).unwrap(), b"unexpected member");
    scene.finish();
}
