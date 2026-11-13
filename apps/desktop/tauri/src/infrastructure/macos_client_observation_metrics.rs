//! Opt-in test-thread measurement only. No behavior/deadline or source cache.
use std::{cell::RefCell, collections::BTreeMap, time::Instant};
#[derive(Default, serde::Serialize)]
pub(crate) struct Metric {
    calls: u64,
    inclusive_nanos: u128,
}
thread_local! {static ACTIVE:RefCell<Option<BTreeMap<&'static str,Metric>>>=const {RefCell::new(None)};}
thread_local! {static REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static PLAN_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
pub(crate) fn plan_reference_enabled() -> bool {
    PLAN_REFERENCE.with(|v| v.get())
}
pub(crate) fn plan_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PLAN_REFERENCE.with(|v| v.set(false));
        }
    }
    PLAN_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn reference_enabled() -> bool {
    REFERENCE.with(|v| v.get())
}
pub(crate) fn reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            REFERENCE.with(|v| v.set(false));
        }
    }
    REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) struct Span {
    label: &'static str,
    started: Option<Instant>,
}
pub(crate) fn span(label: &'static str) -> Span {
    Span {
        label,
        started: ACTIVE.with(|state| state.borrow().as_ref().map(|_| Instant::now())),
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            ACTIVE.with(|state| {
                if let Some(values) = state.borrow_mut().as_mut() {
                    let metric = values.entry(self.label).or_default();
                    metric.calls += 1;
                    metric.inclusive_nanos += started.elapsed().as_nanos();
                }
            });
        }
    }
}
pub(crate) fn measure<T>(action: impl FnOnce() -> T) -> (T, BTreeMap<&'static str, Metric>) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ACTIVE.with(|state| *state.borrow_mut() = None);
        }
    }
    ACTIVE.with(|state| {
        assert!(state.borrow().is_none());
        *state.borrow_mut() = Some(BTreeMap::new());
    });
    let reset = Reset;
    let result = action();
    let metrics = ACTIVE.with(|state| state.borrow_mut().take().unwrap());
    drop(reset);
    (result, metrics)
}
