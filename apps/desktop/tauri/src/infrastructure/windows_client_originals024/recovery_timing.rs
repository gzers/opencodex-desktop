//! Buffer timings on the fixture thread and emit them after native work.
use serde::Serialize;
use std::{cell::RefCell, io, time::Instant};

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
    rows: Vec<Row>,
}

thread_local! {
    static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) };
}

pub(super) struct Capture;

impl Capture {
    pub(super) fn begin() -> Self {
        TRACE.with_borrow_mut(|slot| {
            assert!(slot.is_none());
            *slot = Some(Trace {
                started: Instant::now(),
                phase: "setup",
                rows: Vec::with_capacity(256),
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
    }
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
    pub(super) fn finish(mut self, result: &io::Result<()>) {
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
