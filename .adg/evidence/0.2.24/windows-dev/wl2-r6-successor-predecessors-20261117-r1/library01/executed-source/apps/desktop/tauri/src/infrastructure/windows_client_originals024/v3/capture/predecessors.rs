//! Native predecessor set derived ONLY from original completed slot/Owner
//! histories. Nonserde/nonClone, private fields; references are DATA only.
use super::super::predecessors::{self as data, Predecessor};
use super::super::read_window::HistoryReadWindow;
use super::*;
pub(in crate::infrastructure::windows_client_originals024) struct NativePredecessors {
    refs: Vec<Predecessor>,
    binding: Option<Vec<u8>>,
    window: ActionWindow,
}
impl NativePredecessors {
    pub(in crate::infrastructure::windows_client_originals024) fn empty(
        window: &ActionWindow,
    ) -> Self {
        Self {
            refs: Vec::new(),
            binding: None,
            window: *window,
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn collect_previous(
        histories: &mut [CaptureSlot],
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<Self> {
        window.remaining()?;
        let previous = owner.originals_previous_owners();
        if histories.len() != previous.len() || histories.len() > 512 {
            return Err(refused("原Owner.previous和closed slot集合不一致"));
        }
        let read = HistoryReadWindow::action(window).map_err(refused)?;
        let mut refs = Vec::new();
        let mut binding = None;
        for (slot, old) in histories.iter_mut().zip(previous) {
            if !old.originals_is_stopped() {
                return Err(refused("原历史Owner没有及时完整Stop"));
            }
            let (reference, raw) = slot.predecessor_for_read(old, &read)?;
            if binding.as_ref().is_some_and(|expected| *expected != raw) {
                return Err(refused("旧历史broker/来源绑定漂移"));
            }
            binding = Some(raw);
            refs.push(reference);
        }
        let out = Self {
            refs,
            binding,
            window: *window,
        };
        out.validate().map_err(refused)?;
        Ok(out)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn with_current(
        histories: &mut [CaptureSlot],
        current: &mut CaptureSlot,
        owner: &mut Owner,
        window: &ActionWindow,
    ) -> AppResult<Self> {
        let mut out = Self::collect_previous(histories, owner, window)?;
        if out.refs.len() >= 512 {
            return Err(refused("完整前驱集合预算耗尽"));
        }
        let read = HistoryReadWindow::action(window).map_err(refused)?;
        let (reference, binding) = current.predecessor_for_read(owner, &read)?;
        if out
            .binding
            .as_ref()
            .is_some_and(|expected| *expected != binding)
        {
            return Err(refused("当前closed来源不是原前驱broker"));
        }
        out.binding = Some(binding);
        out.refs.push(reference);
        out.validate().map_err(refused)?;
        Ok(out)
    }
    fn validate(&self) -> io::Result<()> {
        self.window
            .remaining()
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?;
        match &self.binding {
            Some(raw) => {
                let binding: super::super::super::binding::Binding =
                    serde_json::from_slice(raw).map_err(|_| invalid())?;
                data::validate_set(&self.refs, &binding)
            }
            None if self.refs.is_empty() => Ok(()),
            _ => Err(invalid()),
        }
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_source_window(
        &self,
        binding: &[u8],
        window: &ActionWindow,
    ) -> io::Result<()> {
        self.validate()?;
        if window.deadline() != self.window.deadline()
            || window.stop_deadline() != self.window.stop_deadline()
            || self.binding.as_ref().is_some_and(|raw| raw != binding)
        {
            return Err(invalid());
        }
        let binding: super::super::super::binding::Binding =
            serde_json::from_slice(binding).map_err(|_| invalid())?;
        data::validate_set(&self.refs, &binding)
    }
    pub(in crate::infrastructure::windows_client_originals024::v3) fn data(
        &self,
    ) -> io::Result<Vec<Predecessor>> {
        self.validate()?;
        serde_json::from_slice(&encoded(&self.refs)?).map_err(|_| invalid())
    }
    pub(in crate::infrastructure::windows_client_originals024) fn digest(
        &self,
    ) -> io::Result<String> {
        self.validate()?;
        data::set_digest(&self.refs)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_namespace(
        &self,
        windows: &PinnedDirectory,
        binding: &[u8],
        current: Option<&PinnedDirectory>,
        end: Instant,
    ) -> io::Result<()> {
        self.validate()?;
        if end != self.window.deadline() || self.binding.as_ref().is_some_and(|raw| raw != binding)
        {
            return Err(invalid());
        }
        let binding = serde_json::from_slice(binding).map_err(|_| invalid())?;
        data::check_namespace(windows, &self.refs, &binding, current, end)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn same(
        &self,
        other: &Self,
    ) -> io::Result<bool> {
        self.validate()?;
        other.validate()?;
        Ok(self.binding == other.binding
            && encoded(&self.refs)? == encoded(&other.refs)?
            && self.window.deadline() == other.window.deadline()
            && self.window.stop_deadline() == other.window.stop_deadline())
    }
}
impl CaptureSlot {
    fn predecessor_for_read(
        &mut self,
        owner: &mut Owner,
        read: &HistoryReadWindow,
    ) -> AppResult<(Predecessor, Vec<u8>)> {
        let result = (|| {
            self.completed_stop_read(owner, read)?;
            let store = self
                .retained
                .as_mut()
                .ok_or_else(|| refused("原完成Store缺席"))?;
            let reference = store
                .core
                .journal()
                .completed_predecessor(read)
                .map_err(refused)?;
            let binding = store.source.binding_raw.clone();
            self.completed_stop_read(owner, read)?;
            Ok((reference, binding))
        })();
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn check_active_namespace(
        &mut self,
        end: Instant,
    ) -> AppResult<()> {
        let store = self
            .retained
            .as_mut()
            .ok_or_else(|| refused("原active namespace Store缺席"))?;
        let result = store
            .core
            .journal()
            .check_capture_namespace(end)
            .map_err(refused);
        if result.is_err() {
            self.poison_completed_read();
        }
        result
    }
}
