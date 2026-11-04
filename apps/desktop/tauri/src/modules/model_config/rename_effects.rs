//! Current-client rename intent, separate from historical sessions or aliases.
//! The local rename transaction writes it; explicit restart reconciles owners.
use super::super::{config_file_mapping::ProviderRename, schema::*, store::sha256};
use crate::{infrastructure::locking::TargetFileLock, modules::backup::safety};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub const PATH: &str = "manager-state/provider-client-renames.json";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    pub channel_id: String,
    pub from: String,
    pub to: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pending {
    pub version: u32,
    pub id: String,
    pub home_binding: String,
    pub bindings: Vec<Binding>,
}
fn valid_provider(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
fn home_binding(home: &Path) -> ModelResult<String> {
    if !home.is_absolute() {
        return Err(invalid("rename_client_home", "客户端改名未绑定配置目录"));
    }
    Ok(sha256(&serde_json::to_vec(home)?))
}
pub fn decode(home: &Path, bytes: Option<&[u8]>) -> ModelResult<Option<Pending>> {
    let Some(bytes) = bytes else { return Ok(None) };
    let pending: Pending = serde_json::from_slice(bytes)?;
    require_uuid(&pending.id)?;
    if pending.version != 1
        || pending.home_binding != home_binding(home)?
        || pending.bindings.len() > 1024
    {
        return Err(invalid(
            "rename_client_intent",
            "客户端改名记录的身份或范围不匹配",
        ));
    }
    let mut origins = std::collections::BTreeSet::new();
    for binding in &pending.bindings {
        require_uuid(&binding.channel_id)?;
        if !valid_provider(&binding.from)
            || !valid_provider(&binding.to)
            || binding.from == binding.to
            || !origins.insert(&binding.from)
        {
            return Err(invalid("rename_client_intent", "客户端改名目标无效或重复"));
        }
    }
    Ok(Some(pending))
}
pub(super) fn next(
    home: &Path,
    before: Option<&[u8]>,
    renames: &[ProviderRename],
) -> ModelResult<Vec<u8>> {
    let mut bindings = decode(home, before)?
        .map(|p| p.bindings)
        .unwrap_or_default();
    for rename in renames {
        for binding in &mut bindings {
            if binding.channel_id == rename.channel_id && binding.to == rename.from {
                binding.to = rename.to.clone();
            }
        }
        bindings.retain(|binding| binding.from != binding.to);
        if let Some(binding) = bindings.iter_mut().find(|b| b.from == rename.from) {
            if binding.channel_id != rename.channel_id {
                return Err(invalid(
                    "rename_client_intent",
                    "待更新的客户端名称已被其它渠道占用",
                ));
            }
            binding.to = rename.to.clone();
        } else {
            bindings.push(Binding {
                channel_id: rename.channel_id.clone(),
                from: rename.from.clone(),
                to: rename.to.clone(),
            });
        }
    }
    bindings.sort_by(|a, b| a.from.cmp(&b.from));
    let value = Pending {
        version: 1,
        id: uuid::Uuid::new_v4().to_string(),
        home_binding: home_binding(home)?,
        bindings,
    };
    let bytes = serde_json::to_vec(&value)?;
    decode(home, Some(&bytes))?;
    Ok(bytes)
}
pub fn load(root: &Path, home: &Path) -> ModelResult<Option<Pending>> {
    let path = root.join(PATH);
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = TargetFileLock::lock(&path)?;
    load_unlocked(root, home)
}
fn load_unlocked(root: &Path, home: &Path) -> ModelResult<Option<Pending>> {
    let path = root.join(PATH);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(None);
    }
    decode(home, Some(&safety::read_file(&path, 512 * 1024)?))
}
pub fn current_map(pending: &Pending, policy: &Policy) -> ModelResult<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for binding in &pending.bindings {
        let Some(channel) = policy.channels.iter().find(|c| c.id == binding.channel_id) else {
            continue;
        };
        if channel.connection.provider_id != binding.to {
            return Err(invalid(
                "rename_client_changed",
                "改名后渠道又有变化，请重新核对",
            ));
        }
        // A later explicitly-created provider may legitimately reuse a name.
        // It is a current route; never rewrite that selection to another object.
        if policy
            .channels
            .iter()
            .any(|c| c.connection.provider_id == binding.from)
        {
            continue;
        }
        result.insert(binding.from.clone(), binding.to.clone());
    }
    Ok(result)
}
pub fn acknowledge(root: &Path, home: &Path, id: &str) -> ModelResult<bool> {
    let path = root.join(PATH);
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = TargetFileLock::lock(&path)?;
    let Some(pending) = load_unlocked(root, home)? else {
        return Ok(true);
    };
    if pending.id != id {
        return Ok(false);
    }
    std::fs::remove_file(&path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rename(from: &str, to: &str) -> ProviderRename {
        ProviderRename {
            channel_id: "10000000-0000-4000-8000-000000000001".into(),
            from: from.into(),
            to: to.into(),
        }
    }
    #[test]
    fn pending_maps_compose_only_unacknowledged_current_renames_and_ack_is_cas() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("official");
        std::fs::create_dir_all(&home).unwrap();
        let a = next(&home, None, &[rename("A", "B")]).unwrap();
        let b = next(&home, Some(&a), &[rename("B", "C")]).unwrap();
        let old = decode(&home, Some(&a)).unwrap().unwrap();
        let current = decode(&home, Some(&b)).unwrap().unwrap();
        assert_eq!(
            current
                .bindings
                .iter()
                .map(|b| (&*b.from, &*b.to))
                .collect::<Vec<_>>(),
            vec![("A", "C"), ("B", "C")]
        );
        crate::infrastructure::atomic_write::atomic_write(&root.path().join(PATH), &b, 0o600)
            .unwrap();
        assert!(!acknowledge(root.path(), &home, &old.id).unwrap());
        assert!(load(root.path(), &home).unwrap().is_some());
        assert!(acknowledge(root.path(), &home, &current.id).unwrap());
        assert!(load(root.path(), &home).unwrap().is_none());
        assert!(decode(&root.path().join("other"), Some(&b)).is_err());
    }
}
