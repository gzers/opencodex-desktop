//! Test-only timing for the initial scans and the original recovery request.
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread::{self, ThreadId},
};

const PREFIX_CAPACITY: usize = 128;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::infrastructure) struct RequestBinding {
    pub request_id: String,
    pub transport_action_id: String,
    pub host_instance_id: String,
    pub broker_pid: u32,
    pub broker_filetime: String,
    pub started_qpc: u64,
    pub deadline_qpc: u64,
    pub qpc_frequency: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrefixRow {
    phase: &'static str,
    transition: &'static str,
    tick: Option<u64>,
    thread: String,
    detail: Option<String>,
    raw: Option<i32>,
}

pub(in crate::infrastructure) struct RequestTrace {
    binding: RequestBinding,
    rows: Box<[OnceLock<PrefixRow>]>,
    reserved: AtomicUsize,
}

impl RequestTrace {
    pub(in crate::infrastructure) fn new(binding: RequestBinding) -> Arc<Self> {
        Arc::new(Self {
            binding,
            rows: (0..PREFIX_CAPACITY).map(|_| OnceLock::new()).collect(),
            reserved: AtomicUsize::new(0),
        })
    }

    fn tick(&self) -> Option<u64> {
        let (tick, frequency) = crate::infrastructure::windows_owner_host024::qpc().ok()?;
        (frequency == self.binding.qpc_frequency && tick >= self.binding.started_qpc)
            .then_some(tick)
    }

    fn push(
        &self,
        phase: &'static str,
        transition: &'static str,
        detail: Option<String>,
        raw: Option<i32>,
    ) {
        let index = self.reserved.fetch_add(1, Ordering::Relaxed);
        if let Some(slot) = self.rows.get(index) {
            let _ = slot.set(PrefixRow {
                phase,
                transition,
                tick: self.tick(),
                thread: format!("{:?}", thread::current().id()),
                detail,
                raw,
            });
        }
    }

    // OnceLock::get never waits for a publisher. A gap ends the visible prefix.
    pub(in crate::infrastructure) fn snapshot(&self, reason: &'static str) -> serde_json::Value {
        let captured_qpc = self.tick();
        let reserved = self.reserved.load(Ordering::Acquire);
        let rows: Vec<_> = self
            .rows
            .iter()
            .take(reserved)
            .map_while(OnceLock::get)
            .collect();
        let unknown_qpc = rows.iter().filter(|row| row.tick.is_none()).count();
        let first_error = rows.iter().position(|row| row.transition == "error");
        serde_json::json!({"binding":self.binding,"reason":reason,"capturedQpc":captured_qpc,
            "rows":rows,"firstErrorIndex":first_error,"reserved":reserved,
            "overflow":reserved.saturating_sub(self.rows.len()),
            "unpublishedOrAfterGap":reserved.min(self.rows.len())-rows.len(),
            "unknownQpc":unknown_qpc,"runtimeAuthority":false})
    }
}

thread_local! {
    static REQUEST_TRACE: RefCell<Option<Arc<RequestTrace>>> = const { RefCell::new(None) };
}

pub(in crate::infrastructure) struct RequestTraceScope;

impl RequestTraceScope {
    pub(in crate::infrastructure) fn enter(trace: Option<Arc<RequestTrace>>) -> Self {
        REQUEST_TRACE.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = trace;
        });
        Self
    }
}

impl Drop for RequestTraceScope {
    fn drop(&mut self) {
        REQUEST_TRACE.with_borrow_mut(Option::take);
    }
}

pub(in crate::infrastructure) fn active_request() -> Option<Arc<RequestTrace>> {
    REQUEST_TRACE.with_borrow(Clone::clone)
}

pub(in crate::infrastructure) fn mark(
    phase: &'static str,
    transition: &'static str,
    detail: Option<String>,
    raw: Option<i32>,
) {
    REQUEST_TRACE.with_borrow(|trace| {
        if let Some(trace) = trace {
            trace.push(phase, transition, detail, raw);
        }
    });
}

pub(in crate::infrastructure) fn original_worker(
    thread: ThreadId,
    failed_action: uuid::Uuid,
    session: uuid::Uuid,
) {
    if active_request().is_some() {
        mark(
            "original-worker-binding",
            "return",
            Some(format!("{thread:?}/{failed_action}/{session}")),
            None,
        );
    }
}

pub(in crate::infrastructure) fn io_stage<T>(
    phase: &'static str,
    run: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    mark(phase, "enter", None, None);
    let result = run();
    match &result {
        Ok(_) => mark(phase, "return", None, None),
        Err(error) if active_request().is_some() => mark(
            phase,
            "error",
            Some(format!("{:?}", error.kind())),
            error.raw_os_error(),
        ),
        Err(_) => {}
    }
    result
}

pub(in crate::infrastructure) fn app_stage<T>(
    phase: &'static str,
    run: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    mark(phase, "enter", None, None);
    let result = run();
    match &result {
        Ok(_) => mark(phase, "return", None, None),
        Err(error) if active_request().is_some() => {
            let detail = match error {
                AppError::RuntimeManaged { code, .. } => code.chars().take(120).collect(),
                AppError::Timeout => "Timeout".into(),
                _ => "AppError".into(),
            };
            mark(phase, "error", Some(detail), None);
        }
        Err(_) => {}
    }
    result
}

use serde::Serialize;
use std::{cell::RefCell, io, sync::OnceLock, time::Instant};

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

// Record the fixture's initial Start across its entry and native worker.
static START_SCAN_ORIGIN: OnceLock<Instant> = OnceLock::new();
thread_local! {
    static START_SCANS: RefCell<Option<(&'static str, Instant, Vec<serde_json::Value>)>> = const { RefCell::new(None) };
}

pub(in crate::infrastructure) struct StartScans;

impl StartScans {
    pub(in crate::infrastructure) fn begin(phase: &'static str, end: Instant) -> Self {
        START_SCAN_ORIGIN.get_or_init(Instant::now);
        START_SCANS.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some((phase, end, Vec::new()));
        });
        Self
    }
}

impl Drop for StartScans {
    fn drop(&mut self) {
        let (phase, end, rows) = START_SCANS.with_borrow_mut(Option::take).unwrap();
        println!(
            "r7-initial-image-scans={}",
            serde_json::json!({"phase":phase,"deadlineNs":start_scan_ns(end),"scans":rows})
        );
    }
}

fn start_scan_ns(instant: Instant) -> u128 {
    instant
        .saturating_duration_since(*START_SCAN_ORIGIN.get().unwrap())
        .as_nanos()
}

pub(in crate::infrastructure) fn verify_initial_image<T>(
    operation: &'static str,
    verify: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    let end = START_SCANS.with_borrow(|slot| slot.as_ref().map(|(_, end, _)| *end));
    let Some(end) = end else { return verify() };
    let start = Instant::now();
    let result = verify();
    let finish = Instant::now();
    START_SCANS.with_borrow_mut(|slot| {
        let (_, _, rows) = slot.as_mut().unwrap();
        let ordinal = rows.len() + 1;
        rows.push(serde_json::json!({"ordinal":ordinal,"operation":operation,
            "startedNs":start_scan_ns(start),"finishedNs":start_scan_ns(finish),
            "elapsedUs":finish.duration_since(start).as_micros(),
            "remainingBeforeUs":end.saturating_duration_since(start).as_micros(),
            "remainingAfterUs":end.saturating_duration_since(finish).as_micros(),
            "completed":result.is_ok(),"firstError":result.as_ref().err().map(ToString::to_string)}));
    });
    result
}

pub(super) fn verify_preparation_image(
    ordinal: u8,
    failed_action: &str,
    selected: &SelectedProgram,
    end: Instant,
) -> AppResult<()> {
    let phase = match ordinal {
        1 => "source-fence-1-full-image",
        2 => "source-fence-2-full-image",
        3 => "source-fence-3-full-image",
        _ => "source-fence-unrecognized",
    };
    app_stage(phase, || {
        verify_preparation_image_observed(ordinal, failed_action, selected, end)
    })
}

fn verify_preparation_image_observed(
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
fn wl2_r7_timing_prefix_is_readable_during_inflight_worker_and_retains_raw_error() {
    let (start, frequency) = crate::infrastructure::windows_owner_host024::qpc().unwrap();
    let binding = RequestBinding {
        request_id: uuid::Uuid::new_v4().to_string(),
        transport_action_id: uuid::Uuid::new_v4().to_string(),
        host_instance_id: uuid::Uuid::new_v4().to_string(),
        broker_pid: std::process::id(),
        broker_filetime: "0".into(),
        started_qpc: start,
        deadline_qpc: start + frequency,
        qpc_frequency: frequency,
    };
    let trace = RequestTrace::new(binding.clone());
    let (entered, waiting) = std::sync::mpsc::channel();
    let (release, held) = std::sync::mpsc::channel();
    let worker_trace = trace.clone();
    let worker = thread::spawn(move || {
        let _scope = RequestTraceScope::enter(Some(worker_trace));
        io_stage("held-worker", || {
            entered.send(()).unwrap();
            held.recv().unwrap();
            Err::<(), _>(io::Error::from_raw_os_error(5))
        })
    });
    waiting
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap();
    let in_flight = trace.snapshot("held-worker-test");
    release.send(()).unwrap();
    assert_eq!(worker.join().unwrap().unwrap_err().raw_os_error(), Some(5));
    let returned = trace.snapshot("returned-worker-test");
    assert_eq!(in_flight["binding"], serde_json::to_value(binding).unwrap());
    assert_eq!(in_flight["rows"].as_array().unwrap().len(), 1);
    assert_eq!(in_flight["rows"][0]["transition"], "enter");
    assert!(in_flight["firstErrorIndex"].is_null());
    assert_eq!(returned["rows"][0], in_flight["rows"][0]);
    assert_eq!(returned["rows"][1]["raw"], 5);
    assert_eq!(returned["firstErrorIndex"], 1);
    assert_eq!(returned["unknownQpc"], 0);
    assert_eq!(returned["runtimeAuthority"], false);

    trace.reserved.fetch_add(1, Ordering::Relaxed); // Reserved but not published.
    for _ in 0..PREFIX_CAPACITY {
        trace.push("later", "return", None, None);
    }
    let overflow = trace.snapshot("gap-test");
    assert_eq!(overflow["rows"], returned["rows"]);
    assert_eq!(overflow["unpublishedOrAfterGap"], PREFIX_CAPACITY - 2);
    assert_eq!(overflow["overflow"], 3);
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
