//! Closed inventory locator grammar only. No parsed row/native permission.
use super::super::handle_release::HandleRow;
use super::*;
fn ordinal(value: &str, padded: bool) -> io::Result<usize> {
    let n = value.parse::<usize>().map_err(|_| invalid())?;
    if n >= 4096
        || value
            != if padded {
                format!("{n:04}")
            } else {
                n.to_string()
            }
    {
        return Err(invalid());
    }
    Ok(n)
}
pub(super) fn validate(row: &HandleRow, plan: &StoragePlan, g: &Progress) -> io::Result<()> {
    let parts = row.key.split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        ["tree", root, role, index] => {
            let root = ordinal(root, false)?;
            let index = ordinal(index, true)?;
            let tree = plan.roots.get(root).ok_or_else(invalid)?;
            match *role {
                "original" | "history-source" => {
                    let node = tree.nodes.get(index).ok_or_else(invalid)?;
                    if row.volume != node.volume
                        || row.file_id != node.file_id
                        || row.directory != (node.kind == Kind::DirectoryAnchor)
                        || row.historical_work
                    {
                        return Err(invalid());
                    }
                }
                "initial-work" => {
                    let node = tree.nodes.get(index).ok_or_else(invalid)?;
                    if node.kind != Kind::CustodiedFile
                        || row.directory
                        || !row.historical_work
                        || row.volume != node.volume
                        || g.nodes.get(&node.node_id).and_then(|s| s.1.as_ref())
                            != Some(&row.file_id)
                    {
                        return Err(invalid());
                    }
                }
                "stopped-anchor" => g.validate_stopped_handle_row(root, index, row)?,
                _ => return Err(invalid()),
            }
        }
        ["tree", root, "old-work", snapshot, index] => {
            let root = ordinal(root, false)?;
            let _ = plan.roots.get(root).ok_or_else(invalid)?;
            let _ = ordinal(snapshot, true)?;
            let _ = ordinal(index, true)?;
            if row.directory
                || !row.historical_work
                || row.volume != plan.roots[root].custody_volume
            {
                return Err(invalid());
            }
        }
        ["lease", rest @ ..] => {
            let tail = match rest {
                ["final", tail @ ..] => tail,
                ["old", index, tail @ ..] => {
                    let _ = ordinal(index, true)?;
                    tail
                }
                _ => return Err(invalid()),
            };
            if row.historical_work {
                return Err(invalid());
            }
            match tail {
                ["coord", coord, "file"] => {
                    if ordinal(coord, false)? > 3 || row.directory {
                        return Err(invalid());
                    }
                }
                ["coord", "0", "claim-parent"] => {
                    if !row.directory {
                        return Err(invalid());
                    }
                }
                ["coord", coord, role, index] => {
                    let coord = ordinal(coord, false)?;
                    let _ = ordinal(index, false)?;
                    if !row.directory
                        || !((coord == 0 && *role == "ancestor")
                            || ((1..=3).contains(&coord) && *role == "parent"))
                    {
                        return Err(invalid());
                    }
                }
                ["bound", name, index] => {
                    let _ = ordinal(index, false)?;
                    if !row.directory || !["profile", "home", "client"].contains(name) {
                        return Err(invalid());
                    }
                }
                _ => return Err(invalid()),
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}
#[cfg(test)]
#[path = "handle_row_tests.rs"]
mod tests;
