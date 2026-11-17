use std::path::{Path, Component};
pub(crate) fn clean_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path.to_str().is_some()
        && !path.to_string_lossy().chars().any(char::is_control)
        && !path
            .to_string_lossy()
            .split(['/', '\\'])
            .any(|s| matches!(s, "." | ".."))
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}
