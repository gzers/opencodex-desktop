//! Buffer timings on the fixture thread and emit them after native work.
use serde::Serialize;
use std::{cell::RefCell, io, time::Instant};

use crate::{
    errors::{AppError, AppResult},
    modules::runtime::{program_image, program_selection::SelectedProgram},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    ordinal: usize,
    phase: &'static str,
    operation: &'static str,
    started_us: u64,
    elapsed_us: u64,
    remaining_us: u64,
    expected_objects: usize,
    expected_bytes: usize,
    completed: bool,
    raw_os_error: Option<i32>,
}

struct Trace {
    started: Instant,
    phase: &'static str,
    journal_stages: bool,
    rows: Vec<Row>,
    preparation_scans: Vec<PreparationScan>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparationScan {
    ordinal: u8,
    phase: &'static str,
    failed_action: String,
    source_digest: String,
    started_us: u64,
    finished_us: u64,
    deadline_us: u64,
    remaining_before_us: u64,
    remaining_after_us: u64,
    completed: bool,
    error_code: Option<String>,
}

thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

pub(super) struct Capture;

impl Capture {
    pub(super) fn begin() -> Self {
        Self::start(false)
    }

    pub(super) fn begin_with_journal_stages() -> Self {
        Self::start(true)
    }

    fn start(journal_stages: bool) -> Self {
        TRACE.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some(Trace {
                started: Instant::now(),
                phase: "setup",
                journal_stages,
                rows: Vec::with_capacity(256),
                preparation_scans: Vec::with_capacity(3),
            });
        });
        Self
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let trace = TRACE.with_borrow_mut(Option::take).unwrap();
        println!(
            "r7-recovery-timing={}",
            serde_json::to_string(&trace.rows).unwrap()
        );
        if !trace.preparation_scans.is_empty() {
            println!(
                "r7-preparation-image-scans={}",
                serde_json::to_string(&trace.preparation_scans).unwrap()
            );
        }
    }
}

pub(super) fn verify_preparation_image(
    ordinal: u8,
    failed_action: &str,
    selected: &SelectedProgram,
    end: Instant,
) -> AppResult<()> {
    let origin = TRACE.with_borrow(|slot| {
        slot.as_ref()
            .filter(|trace| trace.preparation_scans.len() < 3)
            .map(|trace| trace.started)
    });
    let Some(origin) = origin else {
        return program_image::verify_until(selected, end);
    };
    let start = Instant::now();
    let result = program_image::verify_until(selected, end);
    let finish = Instant::now();
    let micros =
        |later: Instant, earlier| later.saturating_duration_since(earlier).as_micros() as u64;
    let error_code = match &result {
        Err(AppError::RuntimeManaged { code, .. }) => Some(code.clone()),
        Err(_) => Some("other".into()),
        Ok(()) => None,
    };
    TRACE.with_borrow_mut(|slot| {
        slot.as_mut()
            .unwrap()
            .preparation_scans
            .push(PreparationScan {
                ordinal,
                phase: "preparation-full-image-verify",
                failed_action: failed_action.to_owned(),
                source_digest: selected.selection.image_sha256.clone(),
                started_us: micros(start, origin),
                finished_us: micros(finish, origin),
                deadline_us: micros(end, origin),
                remaining_before_us: micros(end, start),
                remaining_after_us: micros(end, finish),
                completed: result.is_ok(),
                error_code,
            });
    });
    result
}

pub(super) fn phase(name: &'static str) {
    TRACE.with_borrow_mut(|slot| {
        if let Some(trace) = slot {
            trace.phase = name;
        }
    });
}

pub(super) struct Readback {
    row: Row,
    start: Instant,
    end: Instant,
}

impl Readback {
    pub(super) fn finish<T>(mut self, result: &io::Result<T>) {
        let finish = Instant::now();
        self.row.elapsed_us = finish.duration_since(self.start).as_micros() as u64;
        self.row.remaining_us = self.end.saturating_duration_since(finish).as_micros() as u64;
        self.row.completed = result.is_ok();
        self.row.raw_os_error = result.as_ref().err().and_then(io::Error::raw_os_error);
        TRACE.with_borrow_mut(|slot| {
            let rows = &mut slot.as_mut().unwrap().rows;
            self.row.ordinal = rows.len() + 1;
            rows.push(self.row);
        });
    }
}

pub(super) fn readback(
    operation: &'static str,
    end: Instant,
    scope: impl FnOnce() -> (usize, usize),
) -> Option<Readback> {
    let (origin, phase) =
        TRACE.with_borrow(|slot| slot.as_ref().map(|trace| (trace.started, trace.phase)))?;
    let (expected_objects, expected_bytes) = scope();
    let start = Instant::now();
    Some(Readback {
        row: Row {
            ordinal: 0,
            phase,
            operation,
            started_us: start.duration_since(origin).as_micros() as u64,
            elapsed_us: 0,
            remaining_us: 0,
            expected_objects,
            expected_bytes,
            completed: false,
            raw_os_error: None,
        },
        start,
        end,
    })
}

pub(super) fn journal_stage<T>(
    operation: &'static str,
    end: Instant,
    scope: impl FnOnce() -> (usize, usize),
    run: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    let enabled = TRACE.with_borrow(|slot| slot.as_ref().is_some_and(|trace| trace.journal_stages));
    let timing = if enabled {
        readback(operation, end, scope)
    } else {
        None
    };
    let result = run();
    if let Some(timing) = timing {
        timing.finish(&result);
    }
    result
}

#[test]
fn wl2_r7_preparation_image_scan_observation_keeps_deadline_and_original_error() {
    use crate::{
        infrastructure::hash,
        modules::{
            data_root::layout::Platform,
            runtime::{
                program_layout::RuntimeVersionPaths,
                program_selection::{CurrentSelection, ImageManifest, IMAGE_MANIFEST},
                windows_lifecycle024_contract::BeforeResumeRecoveryWindow,
            },
        },
    };
    use std::{fs, time::Duration};

    let fixture = tempfile::tempdir().unwrap();
    let paths = RuntimeVersionPaths::new(fixture.path(), "2.99.0", Platform::Windows).unwrap();
    fs::create_dir(&paths.version_root).unwrap();
    let file = paths.version_root.join("payload");
    fs::write(&file, b"first bytes").unwrap();
    let manifest_path = paths.version_root.join(IMAGE_MANIFEST);
    let manifest = b"synthetic manifest bytes";
    fs::write(&manifest_path, manifest).unwrap();
    let selected = SelectedProgram {
        selection: CurrentSelection {
            schema_version: 1,
            version: "2.99.0".into(),
            platform: "win32-x64".into(),
            image_sha256: hash::sha256_hex(manifest),
        },
        image: ImageManifest {
            schema_version: 1,
            package: String::new(),
            version: "2.99.0".into(),
            platform: "win32-x64".into(),
            official_archive_sha256: String::new(),
            tree_sha256: program_image::content_digest(&paths.version_root).unwrap(),
            bin: String::new(),
            package_sha256: String::new(),
            bin_sha256: String::new(),
            node_sha256: String::new(),
            npm_sha256: String::new(),
            bun_sha256: String::new(),
            bun_launcher_sha256: String::new(),
        },
        stable_entry: fixture.path().join("unused-entry"),
        script: fixture.path().join("unused-script"),
        paths,
    };
    let started = Instant::now();
    let window =
        BeforeResumeRecoveryWindow::begin(started, started + Duration::from_secs(10)).unwrap();
    let end = window.deadline();
    let action = uuid::Uuid::new_v4().to_string();
    {
        let _capture = Capture::begin();
        verify_preparation_image(1, &action, &selected, end).unwrap();
        verify_preparation_image(2, &action, &selected, end).unwrap();
        fs::write(&file, b"other bytes").unwrap();
        let observed = verify_preparation_image(3, &action, &selected, end).unwrap_err();
        let unobserved = program_image::verify_until(&selected, end).unwrap_err();
        assert_eq!(observed.to_string(), unobserved.to_string());
        TRACE.with_borrow(|slot| {
            let rows = &slot.as_ref().unwrap().preparation_scans;
            assert_eq!(rows.len(), 3);
            for (row, ordinal) in rows.iter().zip(1..=3) {
                assert_eq!(row.ordinal, ordinal);
                assert_eq!(row.failed_action, action);
                assert_eq!(row.source_digest, selected.selection.image_sha256);
                assert_eq!(row.deadline_us, rows[0].deadline_us);
                assert!(row.started_us <= row.finished_us);
                assert!(row.remaining_before_us >= row.remaining_after_us);
                assert_eq!(row.completed, ordinal < 3);
            }
            assert!(rows
                .windows(2)
                .all(|pair| pair[0].finished_us <= pair[1].started_us
                    && pair[0].remaining_after_us >= pair[1].remaining_before_us));
            assert_eq!(
                rows[2].error_code.as_deref(),
                Some("runtime_image_unconfirmed")
            );
        });
    }
    let path = fixture.path().to_owned();
    fixture.close().unwrap();
    assert!(!path.exists());
    println!("r7-preparation-image-cleanup-absent={}", path.display());
}
