//! Data observations from complete native tree images and sealed snapshots.
//! No restore permit; complete source/lease/phase/exit proofs remain separate.
use super::*;
use crate::modules::runtime::client_artifact_policy::{Snapshot, Stage};
const NAMES: [&str; 3] = [
    "opencodex-catalog.json",
    "models_cache.json",
    "opencodex-catalog-audit.jsonl",
];
enum Row {
    Absent,
    File {
        bytes: Vec<u8>,
        generation: String,
        protection: String,
    },
}
fn row(r: &Row) -> Snapshot<'_> {
    match r {
        Row::Absent => Snapshot::Absent,
        Row::File {
            bytes,
            generation,
            protection,
        } => Snapshot::File {
            bytes,
            generation,
            protection,
        },
    }
}
pub(crate) struct ArtifactFrame {
    root: PathBuf,
    rows: [Row; 3],
}
impl ArtifactFrame {
    pub(super) fn from_entries(path: &Path, entries: &[Entry]) -> io::Result<Self> {
        if path.file_name().and_then(|p| p.to_str()) != Some(".codex") {
            return Err(invalid());
        }
        let mut rows = Vec::new();
        for name in NAMES {
            rows.push(match entries.iter().find(|e| e.relative == name) {
                None => Row::Absent,
                Some(entry) => {
                    if entry.directory || entry.bytes.len() > 2 * 1024 * 1024 {
                        return Err(invalid());
                    }
                    Row::File {
                        bytes: entry.bytes.clone(),
                        generation: format!("{}:{}", entry.id.volume, entry.id.index),
                        protection: entry.security.content_policy_protection()?,
                    }
                }
            });
        }
        Ok(Self {
            root: path.into(),
            rows: rows.try_into().map_err(|_| invalid())?,
        })
    }
    pub(crate) fn require_root(&self, root: &Path) -> io::Result<()> {
        if self.root != root {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    pub(crate) fn stage(&self) -> Stage<'_> {
        Stage {
            catalog: row(&self.rows[0]),
            cache: row(&self.rows[1]),
            audit: row(&self.rows[2]),
        }
    }
}
impl RootTree {
    pub(in crate::infrastructure::windows_client_originals024) fn original_artifact_frame(
        &mut self,
        end: Instant,
    ) -> io::Result<ArtifactFrame> {
        self.check_original(end)?;
        ArtifactFrame::from_entries(&self.original, &self.entries)
    }
    pub(in crate::infrastructure::windows_client_originals024) fn current_artifact_frame(
        &mut self,
        end: Instant,
    ) -> io::Result<ArtifactFrame> {
        let current = self.observe_working(end)?;
        current.artifact_frame()
    }
    /// No changed baseline is installed. This only vetoes every node outside
    /// the three known artifacts, including the root directory's other names.
    pub(in crate::infrastructure::windows_client_originals024) fn verify_non_artifact_working(
        &mut self,
        end: Instant,
    ) -> io::Result<()> {
        let result = (|| {
            self.check_original(end)?;
            let expected = self.working.as_ref().ok_or_else(invalid)?;
            let mut root = open(&self.original, true, false)?;
            let current = image(&self.original, &mut root, end)?;
            let omitted = |entry: &&Entry| NAMES.contains(&entry.relative.as_str());
            let old: Vec<_> = expected.iter().filter(|e| !omitted(e)).collect();
            let new: Vec<_> = current.iter().filter(|e| !omitted(e)).collect();
            if old.len() != new.len() {
                return Err(invalid());
            }
            for (a, b) in old.into_iter().zip(new) {
                if a.relative != b.relative
                    || a.directory != b.directory
                    || a.id != b.id
                    || a.security != b.security
                    || a.bytes != b.bytes
                {
                    return Err(invalid());
                }
                if a.relative.is_empty() {
                    let names = |v: &Vec<String>| {
                        v.iter()
                            .filter(|n| !NAMES.contains(&n.as_str()))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    if names(&a.children) != names(&b.children) {
                        return Err(invalid());
                    }
                } else if a.children != b.children {
                    return Err(invalid());
                }
            }
            // This confirms actual file kinds/complete8 data for each excluded
            // artifact; a directory with the same allowed leaf is not omitted.
            ArtifactFrame::from_entries(&self.original, &current)?;
            check(end)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
#[path = "artifacts_tests.rs"]
mod tests;
