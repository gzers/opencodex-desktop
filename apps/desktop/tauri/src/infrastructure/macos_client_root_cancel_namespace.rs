//! Exact cancellation-plan names from independently indexed events only.
//! This is membership DATA, never a plan source, movement or restoration permit.
use super::*;
use std::{ffi::OsString, os::fd::AsRawFd};

pub(super) struct RegisteringSlot(PinnedDirectory);

impl RegisteringSlot {
    pub(super) fn create(parent: &PinnedDirectory, id: &str) -> io::Result<Self> {
        parent.private()?;
        let leaf = name(OsStr::new(id))?;
        if unsafe { libc::mkdirat(parent.file().as_raw_fd(), leaf.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let directory = parent.child_private(id, false)?;
        if !super::super::tree::names(directory.file())?.is_empty() {
            return Err(denied("new cancellation plan slot is not empty"));
        }
        Ok(Self(directory))
    }

    pub(super) fn directory(&self) -> &PinnedDirectory {
        &self.0
    }
}

#[derive(Clone)]
pub(super) struct Namespace<'a> {
    names: Vec<OsString>,
    registering: Option<&'a RegisteringSlot>,
}
impl<'a> Namespace<'a> {
    /// The SQLite history reader has already validated each canonical head
    /// and enforced its serialized size and total byte budget.
    pub(super) fn from_history(history: &[Head]) -> io::Result<Self> {
        if history.is_empty() || history.len() > 4096 {
            return Err(denied(
                "cancellation namespace history missing or over bound",
            ));
        }
        let mut plans = BTreeMap::<String, (String, u64, String)>::new();
        for (i, head) in history.iter().enumerate() {
            if head.revision != i as u64 + 1 {
                return Err(denied("cancellation namespace history sequence differs"));
            }
            if !matches!(
                head.state,
                State::RootCancellationPrepared | State::RootCancellationAbsent
            ) {
                continue;
            }
            let reference = head
                .root_retirement
                .as_ref()
                .ok_or_else(|| denied("indexed cancellation lacks plan reference"))?;
            let source_pos = usize::try_from(reference.source_revision)
                .ok()
                .and_then(|n| n.checked_sub(1))
                .ok_or_else(|| denied("cancellation source revision invalid"))?;
            let source = history
                .get(source_pos)
                .ok_or_else(|| denied("cancellation plan independent source missing"))?;
            if !matches!(source.state, State::Preparing | State::CaptureReserved)
                || source.ticket != head.ticket
                || source.references.is_some()
                || source.restore_plan.is_some()
                || source.root_retirement.is_some()
                || source.completion_sha256.is_some()
            {
                return Err(denied("cancellation plan independent source differs"));
            }
            let key = (
                head.ticket.clone(),
                reference.source_revision,
                reference.plan.sha256.clone(),
            );
            if head.state == State::RootCancellationPrepared {
                if source.revision.checked_add(1) != Some(head.revision)
                    || i.checked_sub(1).and_then(|n| history.get(n)) != Some(source)
                    || plans.insert(reference.plan.id.clone(), key).is_some()
                {
                    return Err(denied(
                        "cancellation plan is reused or not an adjacent registration",
                    ));
                }
            } else {
                let previous = i
                    .checked_sub(1)
                    .and_then(|n| history.get(n))
                    .ok_or_else(|| denied("cancellation completion predecessor missing"))?;
                if source.revision.checked_add(2) != Some(head.revision)
                    || previous.state != State::RootCancellationPrepared
                    || previous.ticket != head.ticket
                    || previous.root_retirement != head.root_retirement
                    || plans.get(&reference.plan.id) != Some(&key)
                {
                    return Err(denied(
                        "cancellation completion changes registered plan source",
                    ));
                }
            }
        }
        Ok(Self {
            names: plans.into_keys().map(OsString::from).collect(),
            registering: None,
        })
    }
    pub(super) fn during_registration(mut self, slot: Option<&'a RegisteringSlot>) -> Self {
        self.registering = slot;
        self
    }
    pub(super) fn require_at(&self, manager: &PinnedDirectory) -> io::Result<()> {
        if observation::entry(manager, "client-root-cancel-plans")?.is_none() {
            if self.names.is_empty() && self.registering.is_none() {
                return manager.recheck();
            }
            return Err(denied("indexed cancellation plan parent missing"));
        }
        self.require(&manager.child_private("client-root-cancel-plans", false)?)
    }
    pub(super) fn require(&self, parent: &PinnedDirectory) -> io::Result<()> {
        parent.private()?;
        let mut expected = self.names.clone();
        if let Some(slot) = self.registering.map(RegisteringSlot::directory) {
            let leaf = slot
                .path()
                .file_name()
                .ok_or_else(|| denied("new cancellation slot name missing"))?;
            if slot.path().parent() != Some(parent.path()) {
                return Err(denied("new cancellation slot parent differs"));
            }
            let position = expected
                .binary_search_by(|name| name.as_os_str().cmp(leaf))
                .err()
                .ok_or_else(|| denied("new cancellation slot already indexed"))?;
            PinnedDirectory::recheck_set(&[parent, slot])?;
            expected.insert(position, leaf.to_owned());
        }
        if super::super::tree::names(parent.file())? != expected {
            return Err(denied(
                "cancellation plan namespace differs from independent history",
            ));
        }
        if let Some(slot) = self.registering.map(RegisteringSlot::directory) {
            return PinnedDirectory::recheck_set(&[parent, slot]);
        }
        parent.recheck()
    }
    pub(super) fn contains(&self, id: &str) -> bool {
        self.names.binary_search(&OsString::from(id)).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::DirBuilderExt};

    fn append(history: &mut Vec<Head>) -> String {
        let ticket = uuid::Uuid::new_v4().to_string();
        let source = Head {
            revision: history.len() as u64 + 1,
            ticket,
            state: State::Preparing,
            references: None,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
            root_generation: None,
        };
        let mut pending = source.clone();
        pending.revision += 1;
        pending.state = State::RootCancellationPrepared;
        let id = uuid::Uuid::new_v4().to_string();
        pending.root_retirement = Some(RootRetirement {
            source_revision: source.revision,
            plan: RestorePlan {
                id: id.clone(),
                sha256: "a".repeat(64),
            },
        });
        let mut closed = pending.clone();
        closed.revision += 1;
        closed.state = State::RootCancellationAbsent;
        closed.completion_sha256 = Some("b".repeat(64));
        history.extend([source, pending, closed]);
        id
    }
    #[test]
    fn exact_multi_plan_membership_is_indexed_data_and_never_discovered_from_directories() {
        // Synthetic event DATA tests the finite namespace parser only; no root
        // lineage, Original, plan semantics or mutation authority is invented.
        let mut history = Vec::new();
        let first = append(&mut history);
        let second = append(&mut history);
        let namespace = Namespace::from_history(&history).unwrap();
        let temp_base = std::env::temp_dir().canonicalize().unwrap();
        let fixture = tempfile::Builder::new()
            .prefix("ocxd024-cancel-namespace-")
            .tempdir_in(temp_base)
            .unwrap();
        let root = fixture.path().to_path_buf();
        println!("CANCEL_NAMESPACE_FIXTURE_CREATED {}", root.display());
        let parent_path = root.join("plans");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent_path)
            .unwrap();
        let parent = PinnedDirectory::open(&parent_path).unwrap();
        assert!(namespace.require(&parent).is_err());
        for id in [&first, &second] {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(parent_path.join(id))
                .unwrap();
        }
        namespace.require(&parent).unwrap();
        assert!(namespace.contains(&first) && namespace.contains(&second));
        assert!(!namespace.contains(&uuid::Uuid::new_v4().to_string()));
        fs::write(
            parent_path.join("unindexed"),
            b"preserve unrelated namespace data",
        )
        .unwrap();
        assert!(namespace.require(&parent).is_err());
        assert_eq!(
            fs::read(parent_path.join("unindexed")).unwrap(),
            b"preserve unrelated namespace data"
        );
        drop(parent);
        fixture.close().unwrap();
        assert!(!root.exists());
        println!("CANCEL_NAMESPACE_FIXTURE_REMOVED {}", root.display());
    }
    #[test]
    fn reused_plan_and_changed_pending_source_or_completion_are_rejected() {
        let mut history = Vec::new();
        let first = append(&mut history);
        append(&mut history);
        let mut reused = history.clone();
        for head in &mut reused[4..] {
            head.root_retirement.as_mut().unwrap().plan.id = first.clone();
        }
        assert!(Namespace::from_history(&reused).is_err());
        let mut changed = history.clone();
        changed[5].root_retirement.as_mut().unwrap().plan.sha256 = "c".repeat(64);
        assert!(Namespace::from_history(&changed).is_err());
        let mut gap = history.clone();
        gap[3].revision += 1;
        assert!(Namespace::from_history(&gap).is_err());
        let mut cross = history.clone();
        cross[4].root_retirement.as_mut().unwrap().source_revision = 1;
        assert!(Namespace::from_history(&cross).is_err());
        let mut skipped = history.clone();
        skipped.remove(1);
        assert!(Namespace::from_history(&skipped).is_err());
        assert!(Namespace::from_history(&[]).is_err());
        let mut pending = history[..2].to_vec();
        Namespace::from_history(&pending).unwrap();
        pending[1].root_retirement.as_mut().unwrap().source_revision = 2;
        assert!(Namespace::from_history(&pending).is_err());
    }
}
