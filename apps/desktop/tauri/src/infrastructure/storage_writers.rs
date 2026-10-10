//! Process-local admission for immutable startup storage bindings.
//! A successful binding save latches admission closed until process restart.
//! No waiting, cancellation, filesystem work or lock inversion under the mutex.
use crate::errors::{AppError, AppResult};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Default)]
struct State {
    active: usize,
    frozen: bool,
}
#[derive(Default)]
pub(crate) struct WriterGate(Mutex<State>);
impl WriterGate {
    pub(crate) fn admit(self: &Arc<Self>) -> AppResult<Admission> {
        let mut state = self.0.lock().map_err(|_| closed())?;
        if state.frozen {
            return Err(closed());
        }
        state.active += 1;
        Ok(Admission(self.clone()))
    }
    pub(crate) fn freeze(self: &Arc<Self>) -> AppResult<Option<BindingSave>> {
        let mut state = self.0.lock().map_err(|_| closed())?;
        if state.frozen || state.active != 0 {
            return Ok(None);
        }
        state.frozen = true;
        Ok(Some(BindingSave {
            gate: self.clone(),
            committed: false,
        }))
    }
    pub(crate) fn frozen(&self) -> bool {
        self.0.lock().map_or(true, |state| state.frozen)
    }
}
pub(crate) fn global() -> Arc<WriterGate> {
    static GATE: OnceLock<Arc<WriterGate>> = OnceLock::new();
    GATE.get_or_init(|| Arc::new(WriterGate::default())).clone()
}
fn closed() -> AppError {
    AppError::FileSystem {
        operation: "storage binding".into(),
        detail: "数据目录绑定正在切换或已保存；请重启管理器后再执行写入。".into(),
    }
}
pub(crate) struct Admission(Arc<WriterGate>);
impl Drop for Admission {
    fn drop(&mut self) {
        if let Ok(mut state) = self.0 .0.lock() {
            state.active -= 1;
        }
    }
}
pub(crate) struct BindingSave {
    gate: Arc<WriterGate>,
    committed: bool,
}
impl BindingSave {
    pub(crate) fn permits(&self, gate: &Arc<WriterGate>) -> bool {
        Arc::ptr_eq(&self.gate, gate) && !self.committed
    }
    /// Only latch once the binding is persisted and differs from startup services.
    pub(crate) fn commit(mut self) {
        self.committed = true;
    }
}
impl Drop for BindingSave {
    fn drop(&mut self) {
        if !self.committed {
            if let Ok(mut state) = self.gate.0.lock() {
                state.frozen = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn busy_save_refuses_without_stopping_or_releasing_existing_writer() {
        let gate = Arc::new(WriterGate::default());
        let worker = gate.admit().unwrap();
        assert!(gate.freeze().unwrap().is_none());
        assert!(!gate.frozen());
        drop(worker);
        let binding = gate.freeze().unwrap().unwrap();
        assert!(gate.admit().is_err());
        drop(binding); // Validation failure or panic restores admission.
        assert!(gate.admit().is_ok());
    }
    #[test]
    fn persisted_binding_stays_closed_until_a_new_process_gate() {
        let gate = Arc::new(WriterGate::default());
        gate.freeze().unwrap().unwrap().commit();
        assert!(gate.frozen());
        assert!(gate.admit().is_err());
        assert!(gate.freeze().unwrap().is_none());
        assert!(Arc::new(WriterGate::default()).admit().is_ok());
    }
    #[test]
    fn racing_admission_and_binding_save_never_overlap() {
        for _ in 0..100 {
            let gate = Arc::new(WriterGate::default());
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let worker_gate = gate.clone();
            let worker_barrier = barrier.clone();
            let worker = std::thread::spawn(move || {
                worker_barrier.wait();
                worker_gate.admit()
            });
            barrier.wait();
            let save = gate.freeze().unwrap();
            let admission = worker.join().unwrap();
            assert!(!(save.is_some() && admission.is_ok()));
        }
    }
}
