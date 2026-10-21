//! Incoming provider materials are immutable local staging objects. Accepting
//! them writes a disabled draft; the normal runtime save owns actual adoption.
use super::{provider_auth, schema::*, store::sha256};
use crate::{
    infrastructure::atomic_write::atomic_write, modules::backup::safety, state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Staged {
    pub id: String,
    pub channel_id: String,
    pub provider_id: String,
    pub public: Value,
    pub authorization: Option<provider_auth::Backup>,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/imported-provider-configs")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
pub fn stage(
    root: &Path,
    channel: &ChannelPolicy,
    public: &Value,
    auth: Option<&provider_auth::PortableAuthorization>,
) -> ModelResult<ImportedProviderRef> {
    super::provider_fields::validate_public(public)?;
    let id = uuid::Uuid::new_v4().to_string();
    let authorization = auth
        .map(|value| {
            provider_auth::stage_portable(root, &id, &channel.connection.provider_id, public, value)
        })
        .transpose()?;
    let value = Staged {
        id: id.clone(),
        channel_id: channel.id.clone(),
        provider_id: channel.connection.provider_id.clone(),
        public: public.clone(),
        authorization,
    };
    let bytes = serde_json::to_vec(&value)?;
    atomic_write(&path(root, &id)?, &bytes, 0o600)?;
    let reference = ImportedProviderRef {
        id,
        digest: sha256(&bytes),
    };
    load(root, channel, &reference)?;
    Ok(reference)
}
pub fn load(
    root: &Path,
    channel: &ChannelPolicy,
    reference: &ImportedProviderRef,
) -> ModelResult<Staged> {
    let bytes = safety::read_file(&path(root, &reference.id)?, 4 * 1024 * 1024)?;
    if sha256(&bytes) != reference.digest {
        return Err(invalid(
            "provider_import_digest",
            "原渠道接收材料已改变，未采用",
        ));
    }
    let value: Staged = serde_json::from_slice(&bytes)?;
    if value.id != reference.id
        || value.channel_id != channel.id
        || value.provider_id != channel.connection.provider_id
    {
        return Err(invalid(
            "provider_import_identity",
            "原渠道材料属于其他接收目标，未复用授权",
        ));
    }
    super::provider_fields::validate_public(&value.public)?;
    if value.public["adapter"] != channel.connection.protocol
        || !channel
            .connection
            .nodes
            .iter()
            .any(|node| value.public["baseUrl"] == node.url)
    {
        return Err(invalid(
            "provider_import_direction",
            "原渠道材料与当前协议或节点不同，请重新比较",
        ));
    }
    Ok(value)
}
pub fn authorization(
    root: &Path,
    context: &ProcessContext,
    value: &Staged,
) -> ModelResult<Option<provider_auth::Backup>> {
    value
        .authorization
        .as_ref()
        .map(|backup| {
            provider_auth::bind_portable(
                root,
                context,
                &value.id,
                &value.provider_id,
                &value.public,
                backup,
            )
        })
        .transpose()
}
