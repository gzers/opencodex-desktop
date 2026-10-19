//! One read-only saved-channel inventory covers both manager policy and original
//! official providers. Drafts, generated observations and source credentials are
//! never mistaken for portable committed configuration.
use super::{
    schema::*,
    store::ModelStore,
    transfer::{self, ExportScope, ModelPackage},
    upstream::UpstreamSnapshot,
};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};
pub fn catalog(root: &Path, snapshot: &UpstreamSnapshot) -> ModelResult<Vec<ChannelPolicy>> {
    let local = ModelStore::new(root).load()?;
    let identities = super::execution_directory::stable_ids(root, snapshot)?;
    let providers = snapshot.providers["providers"]
        .as_object()
        .ok_or_else(|| invalid("provider_fields", "官方渠道目录无效"))?;
    let rows = snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("provider_fields", "官方型号目录无效"))?;
    let mut result = vec![];
    for (provider, public) in providers {
        super::provider_fields::validate_public(public)?;
        let saved = local
            .policy
            .channels
            .iter()
            .find(|c| &c.connection.provider_id == provider);
        let id = saved
            .map(|c| c.id.clone())
            .unwrap_or_else(|| identities.0[provider].clone());
        let protocol = public["adapter"]
            .as_str()
            .ok_or_else(|| invalid("provider_fields", "原渠道缺少协议定义"))?;
        let url = public["baseUrl"]
            .as_str()
            .ok_or_else(|| invalid("provider_fields", "原渠道缺少服务地址"))?;
        if let Some(saved) = saved {
            if saved.connection.protocol != protocol
                || !saved.connection.nodes.iter().any(|n| n.url == url)
            {
                return Err(invalid(
                    "provider_source_drift",
                    "已提交渠道与官方连接不同，请先明确比较当前配置后再导出",
                ));
            }
        }
        let mut channel = saved.cloned().unwrap_or_else(|| ChannelPolicy {
            id: id.clone(),
            label: provider.clone(),
            template_refs: vec![],
            model_refs: Default::default(),
            connection: ChannelConnection {
                provider_id: provider.clone(),
                protocol: protocol.into(),
                nodes: vec![ServiceNode {
                    id: id.clone(),
                    name: "服务地址".into(),
                    url: url.into(),
                }],
                active_node_id: id.clone(),
                enabled: public["disabled"] != true,
                auth_mode: public["authMode"].as_str().map(str::to_owned),
                allow_private_network: public["allowPrivateNetwork"] == true,
                ..Default::default()
            },
            selection: ModelSelection::None,
            model_order: vec![],
            discovered: vec![],
            defaults: Default::default(),
            overrides: Default::default(),
            family_overrides: Default::default(),
        });
        if saved.is_none() {
            let mut seen = BTreeSet::new();
            for row in rows.iter().filter(|r| r["provider"] == provider.as_str()) {
                let model = row["id"]
                    .as_str()
                    .ok_or_else(|| invalid("provider_fields", "原型号缺少身份"))?;
                if seen.insert(model.to_string()) {
                    channel.discovered.push(DiscoveredModel {
                        id: model.into(),
                        label: public["modelDisplayNames"][model]
                            .as_str()
                            .unwrap_or(model)
                            .into(),
                        rules: Default::default(),
                    });
                }
            }
            if let Some(declared) = public["models"].as_array() {
                for model in declared {
                    let model = model
                        .as_str()
                        .ok_or_else(|| invalid("provider_fields", "原型号声明不是字符串"))?;
                    if seen.insert(model.into()) {
                        channel.discovered.push(DiscoveredModel {
                            id: model.into(),
                            label: public["modelDisplayNames"][model]
                                .as_str()
                                .unwrap_or(model)
                                .into(),
                            rules: Default::default(),
                        });
                    }
                }
            }
            channel.model_order = channel.discovered.iter().map(|m| m.id.clone()).collect();
            channel.selection = if let Some(values) = public["selectedModels"].as_array() {
                let values = values
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| invalid("provider_fields", "原选择含非型号值"))
                    })
                    .collect::<ModelResult<Vec<_>>>()?;
                if values.is_empty() {
                    ModelSelection::None
                } else {
                    ModelSelection::Whitelist(values)
                }
            } else if public["newModelPolicy"] == "on" {
                ModelSelection::All
            } else {
                let visible = rows
                    .iter()
                    .filter(|r| {
                        r["provider"] == provider.as_str()
                            && r["disabled"] != true
                            && r["initialSelectionPending"] != true
                    })
                    .filter_map(|r| r["id"].as_str().map(str::to_owned))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                if visible.is_empty() {
                    ModelSelection::None
                } else {
                    ModelSelection::Whitelist(visible)
                }
            };
        }
        result.push(channel);
    }
    Ok(result)
}
pub fn export(
    root: &Path,
    scope: ExportScope,
    snapshot: &UpstreamSnapshot,
) -> ModelResult<ModelPackage> {
    let channels = catalog(root, snapshot)?;
    let mut package = transfer::export(root, scope.clone())?;
    if let Some(policy) = package.payload.policy.as_mut() {
        policy.channels = scope
            .channel_ids
            .iter()
            .map(|id| {
                let channel = channels.iter().find(|c| &c.id == id).ok_or_else(|| {
                    invalid("provider_scope", "所选实际渠道已不存在，请重新读取范围")
                })?;
                let mut portable = transfer::portable(channel);
                if let Some(connection) = portable.connection.as_mut() {
                    connection.provider = Some(
                        snapshot.providers["providers"][&channel.connection.provider_id].clone(),
                    );
                }
                Ok(portable)
            })
            .collect::<ModelResult<_>>()?;
        if scope.include_global {
            policy.default_channel_id = channels
                .iter()
                .find(|c| snapshot.providers["defaultProvider"] == c.connection.provider_id)
                .map(|c| c.id.clone());
            policy.channel_order = Some(channels.iter().map(|c| c.id.clone()).collect());
        }
    }
    transfer::scope_models(&mut package, &scope.channel_ids, &scope.channel_models)?;
    package.format_version = if package
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
        .any(|c| c.connection.as_ref().is_some_and(|v| v.provider.is_some()))
    {
        3
    } else {
        package.format_version
    };
    package.digest = super::store::sha256(&serde_json::to_vec(&package.payload)?);
    transfer::validate_package(&package)?;
    Ok(package)
}
pub fn provider<'a>(package: &'a ModelPackage, id: &str) -> Option<&'a Value> {
    package
        .payload
        .policy
        .as_ref()?
        .channels
        .iter()
        .find(|c| c.id == id)?
        .connection
        .as_ref()?
        .provider
        .as_ref()
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn official_only_inventory_and_public_originals_enter_disabled_immutable_staging_and_old_readers_refuse(
    ) {
        let source = tempfile::tempdir().unwrap();
        let receiver = tempfile::tempdir().unwrap();
        let public = json!({"adapter":"openai-chat","baseUrl":"https://source.example.test/v1","authMode":"key","models":["model/a"],"selectedModels":["model/a"],"modelAliases":{"model/a":"friendly"},"apiKeyPoolStrategy":"fill-first","note":"source overlay"});
        let snapshot = UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: json!({"defaultProvider":"original","providers":{"original":public}}),
            models: json!([{"provider":"original","id":"model/a","namespaced":"original/model-a"}]),
            order: json!({"pickerOrder":["original/model-a"]}),
            catalog: json!({"models":[{"slug":"original/model-a"}]}),
        };
        let inventory = catalog(source.path(), &snapshot).unwrap();
        let id = inventory[0].id.clone();
        assert_eq!(catalog(source.path(), &snapshot).unwrap()[0].id, id);
        assert!(ModelStore::new(source.path())
            .load()
            .unwrap()
            .policy
            .channels
            .is_empty());
        let package = export(
            source.path(),
            ExportScope {
                channel_models: Default::default(),
                include_policy: true,
                channel_ids: vec![id.clone()],
                template_refs: vec![],
                include_global: false,
            },
            &snapshot,
        )
        .unwrap();
        assert_eq!(package.format_version, 3);
        assert_eq!(
            provider(&package, &id).unwrap()["modelAliases"],
            json!({"model/a":"friendly"})
        );
        let mut downgraded = package.clone();
        downgraded.format_version = 2;
        assert!(transfer::validate_package(&downgraded).is_err());
        let incoming = super::super::file_objects::receive(
            receiver.path(),
            &serde_json::to_vec(&package).unwrap(),
            None,
        )
        .unwrap();
        let adopted =
            super::super::file_objects::accept(receiver.path(), &incoming.id, false, None).unwrap();
        let channel = &adopted.workspace.as_ref().unwrap().policy.channels[0];
        assert!(!channel.connection.enabled);
        assert!(channel.connection.credential_ref.is_none());
        let reference = channel.connection.imported_config.as_ref().unwrap();
        let material =
            super::super::provider_import::load(receiver.path(), channel, reference).unwrap();
        assert_eq!(material.public, public);
        assert!(material.authorization.is_none());
        let mut changed = channel.clone();
        changed.connection.nodes[0].url = "https://later.example.test/v1".into();
        assert!(super::super::provider_import::load(receiver.path(), &changed, reference).is_err());
        let mut tampered = reference.clone();
        tampered.digest = "sha256:tampered".into();
        assert!(super::super::provider_import::load(receiver.path(), channel, &tampered).is_err());
        let mut later = adopted.workspace.unwrap().policy;
        later.channels[0].label = "later edit".into();
        let store = ModelStore::new(receiver.path());
        let current = store.load().unwrap();
        store
            .stage_policy(&current.policy_digest, &current.workspace_digest, later)
            .unwrap();
        let replay =
            super::super::file_objects::accept(receiver.path(), &incoming.id, false, None).unwrap();
        assert_eq!(
            replay.workspace.unwrap().policy.channels[0].label,
            "later edit"
        );
    }
}
