use std::path::{Path, PathBuf};

pub(crate) const CLIENT_HOME_DIRECTORY: &str = "codex-client";

/// Official OpenCodex 2.99 expects this private client directory before startup.
pub fn client_home(opencodex_home: &Path) -> PathBuf {
    opencodex_home.join(CLIENT_HOME_DIRECTORY)
}
