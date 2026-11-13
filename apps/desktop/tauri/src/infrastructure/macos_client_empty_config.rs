//! Content admission for one first-use Codex file. No native authority here.
//! Source witnesses are extracted from the already independently verified
//! phase trees. No missing historical image or new original is manufactured.
use super::*;
use crate::infrastructure::macos_client_originals::tree::RecordedFile;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EmptyConfig {
    /// A current native image bound to the recorded stopped/current tree. Its
    /// bytes are only ASCII whitespace. Never used as an earlier phase image.
    pub(super) stopped: Image,
}
pub(super) struct History {
    original: Option<RecordedFile>,
    ready: Option<RecordedFile>,
    pre_stop: Option<Option<RecordedFile>>,
    stopped: Option<RecordedFile>,
}
impl History {
    pub(super) fn new(
        original: &TreeSnapshot,
        ready: &TreeSnapshot,
        pre_stop: Option<&TreeSnapshot>,
        stopped: &TreeSnapshot,
    ) -> io::Result<Self> {
        for tree in [Some(ready), pre_stop, Some(stopped)].into_iter().flatten() {
            if tree.root != original.root || tree.root_protection != original.root_protection {
                return Err(denied("config source root changed"));
            }
        }
        Ok(Self {
            original: original.recorded_file("config.toml")?,
            ready: ready.recorded_file("config.toml")?,
            pre_stop: pre_stop
                .map(|tree| tree.recorded_file("config.toml"))
                .transpose()?,
            stopped: stopped.recorded_file("config.toml")?,
        })
    }
    fn eligible(&self) -> io::Result<Option<&RecordedFile>> {
        if self.original.is_some() || self.stopped.is_none() {
            return Ok(None);
        }
        let ready = self
            .ready
            .as_ref()
            .ok_or_else(|| denied("new config lacks recorded Ready source"))?;
        if self
            .pre_stop
            .as_ref()
            .is_some_and(|pre| pre.as_ref() != Some(ready))
        {
            return Err(denied("new config changed before Stop"));
        }
        let stopped = self.stopped.as_ref().unwrap();
        if ready.protection != stopped.protection {
            return Err(denied("new config protection changed"));
        }
        Ok(Some(stopped))
    }
    pub(super) fn capture(&self, client: &PinnedDirectory) -> io::Result<Option<EmptyConfig>> {
        let Some(_) = self.eligible()? else {
            if self.original.is_none()
                && self.stopped.is_none()
                && capture_file(client, "config.toml", 4096)? != Image::Absent
            {
                return Err(denied("config appeared outside recorded history"));
            }
            return Ok(None);
        };
        let value = EmptyConfig {
            stopped: capture_file(client, "config.toml", 4096)?,
        };
        self.validate(Some(&value))?;
        Ok(Some(value))
    }
    pub(super) fn validate(&self, value: Option<&EmptyConfig>) -> io::Result<()> {
        match (self.eligible()?, value) {
            (None, None) => Ok(()),
            (Some(recorded), Some(value)) => {
                let Image::File { bytes, .. } = &value.stopped else {
                    return Err(denied("new empty config image is missing"));
                };
                if bytes.len() > 4096
                    || !bytes
                        .iter()
                        .all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
                    || !recorded.matches(&value.stopped)
                {
                    return Err(denied("new config is not the exact recorded empty file"));
                }
                Ok(())
            }
            _ => Err(denied(
                "empty config decision does not match original history",
            )),
        }
    }
}
