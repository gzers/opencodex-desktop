//! Removal is bounded by the last committed product policy and the full public
//! upstream baseline. Unrelated providers and native sources are never inferred deleted.
use super::{
    protocol_groups::{self, Mapping},
    schema::*,
    upstream::UpstreamSnapshot,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadView {
    #[serde(flatten)]
    pub snapshot: UpstreamSnapshot,
    pub read_id: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadReceipt {
    id: String,
    binding: String,
    snapshot: UpstreamSnapshot,
}
fn source_path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-source-reads")
        .join(format!("{id}.json"));
    crate::modules::backup::safety::check_path(root, &path, true)?;
    Ok(path)
}
pub fn remember(
    root: &Path,
    context: &crate::state::ProcessContext,
    snapshot: UpstreamSnapshot,
) -> ModelResult<ReadView> {
    let id = uuid::Uuid::new_v4().to_string();
    let record = ReadReceipt {
        id: id.clone(),
        binding: super::runtime::runtime_binding(context)?,
        snapshot: snapshot.clone(),
    };
    let path = source_path(root, &id)?;
    let bytes = serde_json::to_vec(&record)?;
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
    if crate::modules::backup::safety::read_file(&path, 256 * 1024 * 1024)? != bytes {
        return Err(invalid("source_readback", "渠道读取基准未读回"));
    }
    Ok(ReadView {
        snapshot,
        read_id: id,
    })
}
pub fn initial_removals(
    root: &Path,
    context: &crate::state::ProcessContext,
    current: &UpstreamSnapshot,
    local: &Policy,
    intended: &Policy,
) -> ModelResult<Vec<ChannelPolicy>> {
    let mut channels = vec![];
    let mut providers = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for request in &intended.pending_removals {
        require_uuid(&request.channel_id)?;
        if let Some(reference) = &request.credential_ref {
            require_uuid(reference)?;
        }
        if !providers.insert(&request.provider_id)
            || !ids.insert(&request.channel_id)
            || intended.channels.iter().any(|c| {
                c.connection.provider_id == request.provider_id || c.id == request.channel_id
            })
        {
            return Err(invalid("removal_scope", "删除范围重复或仍被草稿渠道使用"));
        }
        if local
            .channels
            .iter()
            .any(|c| c.id == request.channel_id && c.connection.provider_id == request.provider_id)
        {
            continue;
        }
        if local
            .channels
            .iter()
            .any(|c| c.id == request.channel_id || c.connection.provider_id == request.provider_id)
        {
            return Err(invalid(
                "removal_source",
                "删除来源与已有本机渠道身份冲突，请重新读取",
            ));
        }
        let record: ReadReceipt =
            serde_json::from_slice(&crate::modules::backup::safety::read_file(
                &source_path(root, &request.read_id)?,
                256 * 1024 * 1024,
            )?)?;
        if record.id != request.read_id
            || record.binding != super::runtime::runtime_binding(context)?
            || protected(&record.snapshot, &request.provider_id)
        {
            return Err(invalid(
                "removal_source",
                "删除的官方来源不匹配或属于受保护来源",
            ));
        }
        let original = &record.snapshot.providers["providers"][&request.provider_id];
        if !original.is_object()
            || current.providers["providers"].get(&request.provider_id) != Some(original)
        {
            return Err(invalid(
                "removal_source_drift",
                "首次读取后官方渠道发生变化，请重新读取再删除",
            ));
        }
        let protocol = original["adapter"]
            .as_str()
            .ok_or_else(|| invalid("removal_source", "原渠道协议无效"))?;
        let node = ServiceNode {
            id: uuid::Uuid::new_v4().to_string(),
            name: "删除前节点".into(),
            url: original["baseUrl"].as_str().unwrap_or_default().into(),
        };
        let rows = record
            .snapshot
            .models
            .as_array()
            .ok_or_else(|| invalid("removal_source", "原始清单无效"))?
            .iter()
            .filter(|r| r["provider"] == request.provider_id)
            .collect::<Vec<_>>();
        let discovered = rows
            .iter()
            .map(|r| DiscoveredModel {
                id: r["id"].as_str().unwrap_or_default().into(),
                label: original["modelDisplayNames"][r["id"].as_str().unwrap_or_default()]
                    .as_str()
                    .unwrap_or_else(|| r["id"].as_str().unwrap_or_default())
                    .into(),
                rules: ModelRules::default(),
            })
            .collect::<Vec<_>>();
        let selected = original["selectedModels"].as_array().map(|array| {
            array
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        });
        let selection = match selected {
            Some(ids) if !ids.is_empty() => ModelSelection::Whitelist(ids),
            Some(_) => ModelSelection::None,
            None => {
                let visible = rows
                    .iter()
                    .filter(|r| r["disabled"] != true && r["initialSelectionPending"] != true)
                    .filter_map(|r| r["id"].as_str().map(str::to_owned))
                    .collect::<Vec<_>>();
                if visible.len() == rows.len() && original["newModelPolicy"] == "on" {
                    ModelSelection::All
                } else if visible.is_empty() {
                    ModelSelection::None
                } else {
                    ModelSelection::Whitelist(visible)
                }
            }
        };
        let channel = ChannelPolicy {
            naming: Default::default(),
            id: request.channel_id.clone(),
            label: request.provider_id.clone(),
            template_refs: vec![],
            model_refs: Default::default(),
            connection: ChannelConnection {
                provider_id: request.provider_id.clone(),
                protocol: protocol.into(),
                active_node_id: node.id.clone(),
                nodes: vec![node],
                credential_ref: request.credential_ref.clone(),
                enabled: original["disabled"] != true,
                allow_private_network: original["allowPrivateNetwork"] == true,
                auth_mode: original["authMode"].as_str().map(str::to_owned),
                source_read_id: Some(request.read_id.clone()),
                imported_config: None,
            },
            selection,
            model_order: discovered.iter().map(|m| m.id.clone()).collect(),
            discovered,
            defaults: ModelRules::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        };
        super::compiler::validate_channel(&channel)?;
        channels.push(channel);
    }
    Ok(channels)
}
pub fn protected(snapshot: &UpstreamSnapshot, provider: &str) -> bool {
    ["openai", "chatgpt", "openai-multi", "combo"].contains(&provider)
        || snapshot.providers["providers"][provider]["authMode"]
            .as_str()
            .is_some_and(|mode| ["oauth", "forward"].contains(&mode))
        || snapshot.models.as_array().is_some_and(|rows| {
            rows.iter()
                .any(|r| r["provider"] == provider && r["native"] == true)
        })
}
pub fn removals(
    before: &UpstreamSnapshot,
    committed: &Policy,
    intended: &Policy,
    mappings: &[Mapping],
) -> ModelResult<BTreeSet<String>> {
    let mut removed = BTreeSet::new();
    for channel in &committed.channels {
        if intended.channels.iter().any(|c| c.id == channel.id) {
            if intended.channels.iter().any(|c| {
                c.id == channel.id && c.connection.provider_id != channel.connection.provider_id
            }) {
                return Err(invalid(
                    "provider_rename",
                    "已有渠道运行标识不能直接改名，请复制为新渠道",
                ));
            }
            continue;
        }
        let provider = &channel.connection.provider_id;
        if protected(before, provider) {
            return Err(invalid(
                "protected_provider",
                "原生、OAuth或转发来源不能从自建渠道列表删除",
            ));
        }
        if before.providers["providers"].get(provider).is_some() {
            removed.insert(provider.clone());
        }
        for mapping in mappings.iter().filter(|m| m.channel_id == channel.id) {
            if let Some(actual) = before.providers["providers"].get(&mapping.provider_id) {
                if actual["note"] != protocol_groups::mapping_marker(mapping) {
                    return Err(invalid(
                        "provider_drift",
                        "被删除渠道的协议分组归属改变，未移除",
                    ));
                }
                removed.insert(mapping.provider_id.clone());
            }
        }
    }
    if intended
        .channels
        .iter()
        .any(|c| removed.contains(&c.connection.provider_id))
    {
        return Err(invalid("provider_binding", "删除目标仍被其他渠道绑定"));
    }
    Ok(removed)
}
pub fn projection(
    before: &UpstreamSnapshot,
    removed: &BTreeSet<String>,
) -> ModelResult<UpstreamSnapshot> {
    let mut snapshot = before.clone();
    let rows = before
        .models
        .as_array()
        .ok_or_else(|| invalid("runtime_models", "官方模型清单无效"))?;
    let removed_slugs = rows
        .iter()
        .filter(|r| r["provider"].as_str().is_some_and(|p| removed.contains(p)))
        .filter_map(|r| r["namespaced"].as_str())
        .collect::<BTreeSet<_>>();
    let providers = snapshot.providers["providers"]
        .as_object_mut()
        .ok_or_else(|| invalid("provider", "官方渠道快照无效"))?;
    for provider in removed {
        providers.remove(provider);
    }
    snapshot
        .models
        .as_array_mut()
        .unwrap()
        .retain(|r| !r["provider"].as_str().is_some_and(|p| removed.contains(p)));
    let catalog = snapshot.catalog["models"]
        .as_array_mut()
        .ok_or_else(|| invalid("runtime_catalog", "目录结构无效"))?;
    catalog.retain(|row| {
        !row["slug"]
            .as_str()
            .is_some_and(|slug| removed_slugs.contains(slug))
    });
    for provider in removed {
        let alias = before.providers["providers"][provider]["alias"]
            .as_str()
            .unwrap_or(provider);
        if catalog.iter().any(|r| {
            r["slug"].as_str().is_some_and(|slug| {
                slug.starts_with(&format!("{provider}/")) || slug.starts_with(&format!("{alias}/"))
            })
        }) {
            return Err(invalid(
                "catalog_projection",
                "删除目标还有无法精确归属的目录占用，请先核对官方目录",
            ));
        }
    }
    if let Some(order) = snapshot.order["pickerOrder"].as_array_mut() {
        order.retain(|v| !v.as_str().is_some_and(|slug| removed_slugs.contains(slug)));
    }
    Ok(snapshot)
}
pub fn default_provider(policy: &Policy, providers: &mut Value) -> ModelResult<()> {
    if let Some(id) = &policy.default_channel_id {
        let channel = policy
            .channels
            .iter()
            .find(|c| &c.id == id)
            .ok_or_else(|| invalid("default_provider", "默认渠道不在当前策略内"))?;
        if !channel.connection.enabled || matches!(channel.selection, ModelSelection::None) {
            return Err(invalid(
                "default_provider",
                "默认渠道须启用且至少选择一个模型",
            ));
        }
        providers["defaultProvider"] = serde_json::json!(channel.connection.provider_id);
    }
    let default = providers["defaultProvider"].as_str().unwrap_or("");
    if providers["providers"].get(default).is_none() {
        return Err(invalid(
            "default_provider",
            "删除当前默认渠道前，请在同一草稿中选择替代默认渠道",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn first_read_deletion_rejects_changed_provider_or_a_different_runtime_root() {
        let root = tempfile::tempdir().unwrap();
        let (snapshot, _, mut intended) = setup();
        let context = crate::state::ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                root.path().join("fixture-ocx"),
            ),
            working_directory: root.path().into(),
            opencodex_home: root.path().join("upstream"),
        };
        let read = remember(root.path(), &context, snapshot.clone()).unwrap();
        intended.pending_removals = vec![SourceRemoval {
            read_id: read.read_id,
            channel_id: uuid::Uuid::new_v4().to_string(),
            provider_id: "remove".into(),
            credential_ref: None,
        }];
        let mut changed = snapshot.clone();
        changed.providers["providers"]["remove"]["note"] = json!("later edit");
        assert_eq!(
            initial_removals(
                root.path(),
                &context,
                &changed,
                &Policy::default(),
                &intended
            )
            .unwrap_err()
            .code,
            "removal_source_drift"
        );
        let mut different = context.clone();
        different.opencodex_home = root.path().join("other-root");
        assert_eq!(
            initial_removals(
                root.path(),
                &different,
                &snapshot,
                &Policy::default(),
                &intended
            )
            .unwrap_err()
            .code,
            "removal_source"
        );
    }
    fn setup() -> (UpstreamSnapshot, Policy, Policy) {
        let channel = |id: &str, provider: &str| ChannelPolicy {
            naming: Default::default(),
            id: id.into(),
            label: provider.into(),
            template_refs: vec![],
            model_refs: Default::default(),
            connection: ChannelConnection {
                provider_id: provider.into(),
                enabled: true,
                ..Default::default()
            },
            selection: ModelSelection::All,
            model_order: vec![],
            discovered: vec![],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        };
        let a = channel("a", "remove");
        let b = channel("b", "keep");
        let snapshot = UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: json!({"defaultProvider":"remove","providers":{"remove":{"authMode":"key"},"keep":{"note":"unrelated"},"third":{"note":"keep untouched"}}}),
            models: json!([{"provider":"remove","namespaced":"remove/m"},{"provider":"keep","namespaced":"keep/m"}]),
            catalog: json!({"models":[{"slug":"native"},{"slug":"remove/m"},{"slug":"keep/m"}]}),
            order: json!({"pickerOrder":["native","remove/m","keep/m"]}),
        };
        (
            snapshot,
            Policy {
                channels: vec![a, b.clone()],
                ..Default::default()
            },
            Policy {
                channels: vec![b],
                default_channel_id: Some("b".into()),
                ..Default::default()
            },
        )
    }
    #[test]
    fn removal_is_bounded_by_committed_product_ids_and_requires_a_replacement_default() {
        let (snapshot, committed, intended) = setup();
        let removed = removals(&snapshot, &committed, &intended, &[]).unwrap();
        assert_eq!(removed, BTreeSet::from(["remove".into()]));
        let mut projected = projection(&snapshot, &removed).unwrap();
        default_provider(&intended, &mut projected.providers).unwrap();
        assert_eq!(projected.providers["defaultProvider"], "keep");
        assert!(projected.providers["providers"].get("remove").is_none());
        assert_eq!(
            projected.providers["providers"]["third"],
            snapshot.providers["providers"]["third"]
        );
        assert_eq!(
            projected.catalog["models"],
            json!([{"slug":"native"},{"slug":"keep/m"}])
        );
        let mut intended = intended;
        intended.default_channel_id = None;
        let mut projected = projection(&snapshot, &removed).unwrap();
        assert_eq!(
            default_provider(&intended, &mut projected.providers)
                .unwrap_err()
                .code,
            "default_provider"
        );
    }
    #[test]
    fn native_auth_and_unattributed_catalog_are_refused_before_any_mutation() {
        let (mut snapshot, committed, intended) = setup();
        snapshot.providers["providers"]["remove"]["authMode"] = json!("oauth");
        assert_eq!(
            removals(&snapshot, &committed, &intended, &[])
                .unwrap_err()
                .code,
            "protected_provider"
        );
        snapshot.providers["providers"]["remove"]["authMode"] = json!("key");
        snapshot.catalog["models"]
            .as_array_mut()
            .unwrap()
            .push(json!({"slug":"remove/unmapped"}));
        assert_eq!(
            projection(&snapshot, &BTreeSet::from(["remove".into()]))
                .unwrap_err()
                .code,
            "catalog_projection"
        );
    }
}
