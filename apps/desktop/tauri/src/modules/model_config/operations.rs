//! Process-local cancellation includes a bounded stop-before-start window.
//! A delayed IPC start cannot discard a cancellation and begin network work.
use super::schema::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Mutex,
};
use tokio::sync::watch;
#[derive(Default)]
struct State {
    active: BTreeMap<String, watch::Sender<bool>>,
    stopped: BTreeSet<String>,
    order: VecDeque<String>,
}
#[derive(Default)]
pub(super) struct Registry {
    state: Mutex<State>,
}
pub(super) struct Running {
    registry: &'static Registry,
    id: String,
}
impl Registry {
    pub fn stop(&self, id: &str) -> ModelResult<()> {
        require_uuid(id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("operation_state", "操作状态不可用"))?;
        if let Some(sender) = state.active.get(id) {
            sender.send_replace(true);
        }
        if state.stopped.insert(id.into()) {
            state.order.push_back(id.into());
        }
        while state.order.len() > 512 {
            if let Some(old) = state.order.pop_front() {
                state.stopped.remove(&old);
            }
        }
        Ok(())
    }
    pub fn start(&'static self, id: &str) -> ModelResult<(watch::Receiver<bool>, Running)> {
        require_uuid(id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("operation_state", "操作状态不可用"))?;
        if state.stopped.contains(id) {
            return Err(invalid(
                "operation_stopped",
                "该操作已停止等待，未开始网络请求",
            ));
        }
        if state.active.contains_key(id) {
            return Err(invalid("operation_duplicate", "同一操作已在执行"));
        }
        let (sender, cancel) = watch::channel(false);
        state.active.insert(id.into(), sender);
        Ok((
            cancel,
            Running {
                registry: self,
                id: id.into(),
            },
        ))
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Ok(mut state) = self.registry.state.lock() {
            state.active.remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn early_cancellation_is_consumed_before_any_network_work_can_start() {
        let registry = Box::leak(Box::new(Registry::default()));
        let id = uuid::Uuid::new_v4().to_string();
        registry.stop(&id).unwrap();
        assert!(matches!(registry.start(&id),Err(error) if error.code=="operation_stopped"));
        assert!(matches!(registry.start(&id),Err(error) if error.code=="operation_stopped"));
        assert!(registry.state.lock().unwrap().active.is_empty());
    }
    #[test]
    fn active_cancellation_remains_visible_and_duplicate_start_never_replaces_its_receiver() {
        let registry = Box::leak(Box::new(Registry::default()));
        let id = uuid::Uuid::new_v4().to_string();
        let (cancel, guard) = registry.start(&id).unwrap();
        assert!(matches!(registry.start(&id),Err(error) if error.code=="operation_duplicate"));
        registry.stop(&id).unwrap();
        assert!(*cancel.borrow());
        drop(guard);
        assert!(registry.state.lock().unwrap().active.is_empty());
    }
}
