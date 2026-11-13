//! Content provenance for a backup first created during this independently
//! captured session. Native writer exclusion and reversible movement live in
//! the owning transaction; a filename alone never authorizes removing it.
use super::*;
use crate::infrastructure::macos_client_originals::tree::RecordedFile;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct GrokBackup {
    pub(super) stopped: Image,
    pub(super) root: Identity,
}
pub(super) struct History {
    original_root: Option<Identity>,
    original: Option<RecordedFile>,
    config: Option<RecordedFile>,
    ready: Option<RecordedFile>,
    pre: Option<Option<RecordedFile>>,
    stopped: Option<RecordedFile>,
}
impl History {
    pub(super) fn new(
        original: &GrokSnapshot,
        ready: &GrokSnapshot,
        pre: Option<&GrokSnapshot>,
        stopped: &GrokSnapshot,
    ) -> io::Result<Self> {
        fn tree(s: &GrokSnapshot) -> Option<&TreeSnapshot> {
            match s {
                GrokSnapshot::Directory { tree } => Some(tree),
                _ => None,
            }
        }
        let original_tree = tree(original);
        if let Some(a) = original_tree {
            for s in [Some(ready), pre, Some(stopped)].into_iter().flatten() {
                let b = tree(s).ok_or_else(|| denied("recorded Grok root vanished"))?;
                if a.root != b.root || a.root_protection != b.root_protection {
                    return Err(denied("recorded Grok root changed"));
                }
            }
        }
        let file = |s: &GrokSnapshot, path: &str| -> io::Result<Option<RecordedFile>> {
            match s {
                GrokSnapshot::Directory { tree } => tree.recorded_file(path),
                GrokSnapshot::Absent => Ok(None),
            }
        };
        Ok(Self {
            original_root: original_tree.map(|t| t.root.clone()),
            original: file(original, "config.toml.bak-opencodex")?,
            config: file(original, "config.toml")?,
            ready: file(ready, "config.toml.bak-opencodex")?,
            pre: pre
                .map(|s| file(s, "config.toml.bak-opencodex"))
                .transpose()?,
            stopped: file(stopped, "config.toml.bak-opencodex")?,
        })
    }
    fn eligible(&self) -> io::Result<Option<&RecordedFile>> {
        if self.original.is_some() || self.stopped.is_none() {
            return Ok(None);
        }
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| denied("backup lacks original config"))?;
        let ready = self
            .ready
            .as_ref()
            .ok_or_else(|| denied("backup lacks recorded Ready provenance"))?;
        if !config.same_content(ready)
            || self.pre.as_ref().is_some_and(|p| p.as_ref() != Some(ready))
            || self.stopped.as_ref() != Some(ready)
        {
            return Err(denied("first backup changed from original session copy"));
        }
        Ok(self.stopped.as_ref())
    }
    pub(super) fn capture(&self, profile: &PinnedDirectory) -> io::Result<Option<GrokBackup>> {
        let Some(recorded) = self.eligible()? else {
            if self.original.is_none() && self.stopped.is_none() {
                let path = profile.path().join(".grok");
                match std::fs::symlink_metadata(&path) {
                    Err(e)
                        if e.kind() == io::ErrorKind::NotFound && self.original_root.is_none() => {}
                    Ok(_) => {
                        let root = PinnedDirectory::open(&path)?;
                        if Some(root.identity()) != self.original_root.as_ref()
                            || capture_file(&root, "config.toml.bak-opencodex", MAX_CONTENT)?
                                != Image::Absent
                        {
                            return Err(denied("backup appeared outside independent history"));
                        }
                    }
                    _ => return Err(denied("Grok backup absence could not be verified")),
                }
                profile.recheck()?;
            }
            return Ok(None);
        };
        let root = PinnedDirectory::open(&profile.path().join(".grok"))?;
        if Some(root.identity()) != self.original_root.as_ref() {
            return Err(denied("Grok backup root changed"));
        }
        let value = GrokBackup {
            stopped: capture_file(&root, "config.toml.bak-opencodex", MAX_CONTENT)?,
            root: root.identity().clone(),
        };
        if !recorded.matches(&value.stopped) {
            return Err(denied(
                "current backup no longer matches independent history",
            ));
        }
        self.validate(Some(&value))?;
        Ok(Some(value))
    }
    pub(super) fn validate(&self, value: Option<&GrokBackup>) -> io::Result<()> {
        match (self.eligible()?, value) {
            (None, None) => Ok(()),
            (Some(recorded), Some(value))
                if Some(&value.root) == self.original_root.as_ref()
                    && recorded.matches(&value.stopped) =>
            {
                Ok(())
            }
            _ => Err(denied(
                "backup plan does not match original absence and known copy",
            )),
        }
    }
}
