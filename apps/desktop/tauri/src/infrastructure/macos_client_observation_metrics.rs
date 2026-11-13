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
thread_local! {static HISTORY_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static CHILD_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static SCOPE_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static DESCENDANT_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static ROOT_METADATA_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static XATTR_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
thread_local! {static LEASE_DIRECTORIES_REFERENCE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
pub(crate) fn lease_directories_reference_enabled() -> bool {
    LEASE_DIRECTORIES_REFERENCE.with(|v| v.get())
}
pub(crate) fn lease_directories_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            LEASE_DIRECTORIES_REFERENCE.with(|v| v.set(false));
        }
    }
    LEASE_DIRECTORIES_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn xattr_reference_enabled() -> bool {
    XATTR_REFERENCE.with(|v| v.get())
}
pub(crate) fn xattr_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            XATTR_REFERENCE.with(|v| v.set(false));
        }
    }
    XATTR_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn root_metadata_reference_enabled() -> bool {
    ROOT_METADATA_REFERENCE.with(|v| v.get())
}
pub(crate) fn root_metadata_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            ROOT_METADATA_REFERENCE.with(|v| v.set(false));
        }
    }
    ROOT_METADATA_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn descendant_reference_enabled() -> bool {
    DESCENDANT_REFERENCE.with(|v| v.get())
}
pub(crate) fn descendant_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            DESCENDANT_REFERENCE.with(|v| v.set(false));
        }
    }
    DESCENDANT_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn scope_reference_enabled() -> bool {
    SCOPE_REFERENCE.with(|v| v.get())
}
pub(crate) fn scope_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            SCOPE_REFERENCE.with(|v| v.set(false));
        }
    }
    SCOPE_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn child_reference_enabled() -> bool {
    CHILD_REFERENCE.with(|v| v.get())
}
pub(crate) fn child_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CHILD_REFERENCE.with(|v| v.set(false));
        }
    }
    CHILD_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
pub(crate) fn history_reference_enabled() -> bool {
    HISTORY_REFERENCE.with(|v| v.get())
}
pub(crate) fn history_reference<T>(action: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            HISTORY_REFERENCE.with(|v| v.set(false));
        }
    }
    HISTORY_REFERENCE.with(|v| {
        assert!(!v.get());
        v.set(true);
    });
    let _reset = Reset;
    action()
}
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
