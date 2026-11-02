//! Device-local transfer choices; never part of a portable snapshot.
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::{backup::safety, model_config::schema::*},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
const FILE: &str = "manager-state/transfer-choices.json";
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Choice {
    pub selected_ids: Vec<String>,
    #[serde(default)]
    pub preferences: Option<bool>,
    #[serde(default)]
    pub extensions: Option<bool>,
    #[serde(default)]
    pub credential_ids: Vec<String>,
    #[serde(default)]
    pub versions: BTreeMap<String, String>,
    #[serde(default)]
    pub mcp_sources: BTreeMap<String, String>,
    #[serde(default)]
    pub include_global: bool,
    #[serde(default)]
    pub webdav_passwords: bool,
    #[serde(default)]
    pub channel_models: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub channel_families: BTreeMap<String, super::transfer::FamilyScope>,
}
impl Choice {
    fn validate(&self) -> ModelResult<()> {
        if self.selected_ids.len() > 30000
            || self.credential_ids.len() > 30000
            || self.versions.len() > 5000
            || self.mcp_sources.len() > 1000
            || self.channel_models.len() > 5000
            || self.channel_families.len() > 5000
        {
            return Err(invalid("transfer_choices", "本机范围超过上限"));
        }
        for (id, models) in &self.channel_models {
            require_uuid(id)?;
            if self.channel_families.contains_key(id)
                || !self
                    .selected_ids
                    .contains(&serde_json::to_string(&["channels", id.as_str()])?)
            {
                return Err(invalid("transfer_choices", "渠道子范围与已选对象不一致"));
            }
            if models.len() > 10000 || models.iter().any(|m| m.len() > 1024) {
                return Err(invalid("transfer_choices", "型号选择范围无效"));
            }
        }
        for (id, family) in &self.channel_families {
            require_uuid(id)?;
            if !self
                .selected_ids
                .contains(&serde_json::to_string(&["channels", id.as_str()])?)
            {
                return Err(invalid("transfer_choices", "模型族子范围与已选渠道不一致"));
            }
            if family.template_ids.is_empty() || family.template_ids.len() > 5000 {
                return Err(invalid("transfer_choices", "模型族范围无效"));
            }
            for id in &family.template_ids {
                require_uuid(id)?;
            }
        }
        for id in &self.selected_ids {
            let parts: Vec<String> = serde_json::from_str(id)
                .map_err(|_| invalid("transfer_choices", "本机范围身份无效"))?;
            if parts.len() != 2
                || ![
                    "manager",
                    "templates",
                    "channels",
                    "accounts",
                    "skills",
                    "mcps",
                ]
                .contains(&parts[0].as_str())
                || parts[1].len() > 512
            {
                return Err(invalid("transfer_choices", "本机范围身份无效"));
            }
        }
        if self.credential_ids.iter().any(|id| {
            id.len() > 600
                || !id.starts_with("channel:")
                    && !id.starts_with("skill:")
                    && !id.starts_with("mcp:")
        }) {
            return Err(invalid("transfer_choices", "凭据选择身份无效"));
        }
        Ok(())
    }
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Choices {
    file: Option<Choice>,
    remote: Option<Choice>,
    #[serde(default)]
    import_credentials: Option<ImportCredentials>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportCredentials {
    pub keys: bool,
    pub extensions: bool,
}
impl Default for ImportCredentials {
    fn default() -> Self {
        Self {
            keys: true,
            extensions: true,
        }
    }
}
pub fn import_credentials(
    root: &Path,
    value: Option<ImportCredentials>,
) -> ModelResult<ImportCredentials> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    let _g = TargetFileLock::lock(&p)?;
    let mut all = read(root)?;
    if let Some(value) = value {
        all.import_credentials = Some(value);
        atomic_write(&p, &serde_json::to_vec(&all)?, 0o600)?;
    }
    Ok(all.import_credentials.unwrap_or_default())
}
fn read(root: &Path) -> ModelResult<Choices> {
    let path = root.join(FILE);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(Choices::default());
    }
    Ok(serde_json::from_slice(&safety::read_file(
        &path,
        8 * 1024 * 1024,
    )?)?)
}
pub fn load(root: &Path, remote: bool) -> ModelResult<Option<Choice>> {
    let all = read(root)?;
    let c = if remote { all.remote } else { all.file };
    if let Some(c) = &c {
        c.validate()?
    }
    Ok(c)
}
pub fn save(root: &Path, remote: bool, choice: Choice) -> ModelResult<Choice> {
    choice.validate()?;
    let path = root.join(FILE);
    safety::check_path(root, &path, true)?;
    let _guard = TargetFileLock::lock(&path)?;
    let mut all = read(root)?;
    if remote {
        all.remote = Some(choice.clone())
    } else {
        all.file = Some(choice.clone())
    }
    atomic_write(&path, &serde_json::to_vec(&all)?, 0o600)?;
    Ok(choice)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_remote_and_cancelled_choice_remain_independent() {
        let t = tempfile::tempdir().unwrap();
        let c = Choice {
            selected_ids: vec![r#"["skills","fixture"]"#.into()],
            credential_ids: vec!["skill:fixture".into()],
            ..Default::default()
        };
        save(t.path(), false, c).unwrap();
        assert!(load(t.path(), true).unwrap().is_none());
        assert_eq!(
            load(t.path(), false).unwrap().unwrap().credential_ids,
            vec!["skill:fixture"]
        );
        assert!(!t.path().join("opencodex-home/config.json").exists());
    }
    #[test]
    fn unknown_and_secret_fields_are_rejected() {
        assert!(
            serde_json::from_str::<Choice>(r#"{"selectedIds":[],"passphrase":"secret"}"#).is_err()
        );
    }
}
