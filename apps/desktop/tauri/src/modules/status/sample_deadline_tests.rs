use super::*;
use std::cell::{Cell, RefCell};

struct Source {
    calls: RefCell<Vec<(&'static str, Instant)>>,
    fail_fetch: Cell<bool>,
    unknown_payload: Cell<bool>,
    fetch_delay: Cell<Duration>,
}
impl Source {
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            fail_fetch: Cell::new(false),
            unknown_payload: Cell::new(false),
            fetch_delay: Cell::new(Duration::ZERO),
        }
    }
    fn payload(&self, name: &'static str, end: Instant) -> Result<Value, CollectError> {
        self.calls.borrow_mut().push((name, end));
        std::thread::sleep(self.fetch_delay.get());
        if self.fail_fetch.get() {
            Err(CollectError::Unreachable)
        } else if self.unknown_payload.get() {
            Ok(serde_json::json!({"status":"unknown"}))
        } else {
            Ok(serde_json::json!({"status":"stopped","dataRoot":"/fixture/D"}))
        }
    }
}
impl StatusSource for Source {
    fn requires_fresh_sample(&self) -> bool {
        true
    }
    fn fetch(&self) -> Result<Value, CollectError> {
        panic!("unbounded legacy fetch must not run")
    }
    fn fetch_at(&self, end: Instant) -> Result<Value, CollectError> {
        self.payload("fetch", end)
    }
    fn fetch_light_at(&self, end: Instant) -> Result<Value, CollectError> {
        self.payload("light", end)
    }
}

#[test]
fn native_sample_uses_real_clock_and_one_deadline_even_inside_debounce() {
    let mut c = StatusCollector::new(Source::new());
    let before = Instant::now();
    let scheduler_now = before + Duration::from_secs(3600);
    c.refresh_at(scheduler_now).unwrap();
    let after = Instant::now();
    let first = c.source.calls.borrow().clone();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].0, "fetch");
    assert!(first[0].1 >= before + collect_timeout());
    assert!(first[0].1 <= after + collect_timeout());
    assert!(first[0].1 < scheduler_now);
    c.refresh_light_at(scheduler_now).unwrap();
    let calls = c.source.calls.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].0, "light");
}

#[test]
fn expired_sample_never_calls_io_or_uses_prior_recovery_permission() {
    let mut c = StatusCollector::new(Source::new());
    c.refresh().unwrap();
    let count = c.source.calls.borrow().len();
    let now = Instant::now();
    assert_eq!(
        c.refresh_with_deadline(now, false, now),
        Err(CollectError::Timeout)
    );
    assert_eq!(c.source.calls.borrow().len(), count);
}

#[test]
fn late_fetch_never_starts_a_fresh_recovery_window() {
    let mut c = StatusCollector::new(Source::new());
    c.source.fetch_delay.set(Duration::from_millis(20));
    let now = Instant::now();
    assert_eq!(
        c.refresh_with_deadline(now, true, now + Duration::from_millis(5)),
        Err(CollectError::Timeout)
    );
    let calls = c.source.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "light");
}

#[test]
fn failed_process_sample_cannot_combine_with_recovery_success() {
    let mut c = StatusCollector::new(Source::new());
    c.source.fail_fetch.set(true);
    assert_eq!(c.refresh(), Err(CollectError::Unreachable));
}

#[test]
fn unknown_payload_never_reuses_prior_stopped_as_new_start_permission() {
    let mut c = StatusCollector::new(Source::new());
    assert_eq!(c.refresh().unwrap().runtime, RuntimeState::Stopped);
    c.source.unknown_payload.set(true);
    assert_eq!(c.refresh(), Err(CollectError::Unmapped));
    let snapshot = c.snapshot();
    assert!(!snapshot.can_start && !snapshot.can_stop && !snapshot.can_restart);
}
