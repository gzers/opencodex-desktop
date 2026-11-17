//! Pending-data validation only. A syntactic tree step cannot recreate the
//! retained root, native exit, source admission or ClosedEligibility.
use super::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) enum TreeRoot {
    Codex,
    Grok,
}
impl TreeRoot {
    pub(super) fn from_index(index: usize) -> io::Result<Self> {
        match index {
            0 => Ok(Self::Codex),
            1 => Ok(Self::Grok),
            _ => Err(invalid()),
        }
    }
    pub(super) fn leaf(self) -> &'static str {
        match self {
            Self::Codex => ".codex",
            Self::Grok => ".grok",
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Codex => 0,
            Self::Grok => 1,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) enum TreeAction {
    MoveToCustody,
    CreateWorkingCopy,
    RestoreOriginal,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) enum TreeRecord {
    None,
    Intent { root: TreeRoot, action: TreeAction },
    Completed { root: TreeRoot, action: TreeAction },
}
#[derive(Default)]
pub(super) struct TreeProgress {
    states: [u8; 2],
    unfinished: Option<(TreeRoot, TreeAction)>,
}
impl TreeProgress {
    pub(super) fn observe(&mut self, phase: Phase, record: TreeRecord) -> io::Result<()> {
        match record {
            TreeRecord::None => {
                if self.unfinished.is_some()
                    || matches!(
                        phase,
                        Phase::Ready | Phase::PreparedRestore | Phase::OfficialTeardown
                    ) && self.states != [2, 2]
                    || matches!(phase, Phase::Restored | Phase::Closed) && self.states != [3, 3]
                {
                    return Err(invalid());
                }
            }
            TreeRecord::Intent { root, action } => {
                if self.unfinished.is_some() {
                    return Err(invalid());
                }
                let state = self.states[root.index()];
                let valid = match action {
                    TreeAction::MoveToCustody => phase == Phase::Captured && state == 0,
                    TreeAction::CreateWorkingCopy => phase == Phase::Captured && state == 1,
                    TreeAction::RestoreOriginal => {
                        matches!(phase, Phase::OfficialTeardown | Phase::Aborted)
                            && matches!(state, 1 | 2)
                    }
                };
                if !valid {
                    return Err(invalid());
                }
                self.unfinished = Some((root, action));
            }
            TreeRecord::Completed { root, action } => {
                if self.unfinished != Some((root, action)) {
                    return Err(invalid());
                }
                self.states[root.index()] = match action {
                    TreeAction::MoveToCustody => 1,
                    TreeAction::CreateWorkingCopy => 2,
                    TreeAction::RestoreOriginal => 3,
                };
                self.unfinished = None;
            }
        }
        Ok(())
    }
}
