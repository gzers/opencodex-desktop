//! Execution choices join the actual file's channel identities with the supported
//! upstream execution routes. Native rows retain an explicit binding reason.
use super::{
    agent::AgentSettings, compiler::ModelProjection, config_file, schema::*, store::sha256,
    upstream::UpstreamSnapshot,
};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
    state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
const NATIVE: &str = "783452bd-40da-4023-9b3a-f03b8bb3b4c1";
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub label: String,
    pub provider_id: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    #[serde(flatten)]
    pub model: ModelProjection,
    pub native_model_id: Option<String>,
    pub unavailable_reason: Option<String>,
    pub aliases: Vec<String>,
    pub connection_digest: Option<String>,
    /// 是否出现在主选择器的可见集合里。隐藏型号、停用渠道、待首次确认的型号
    /// 仍保留在目录中用于解释已保存设置，但不进入可选列表。
    pub picker_visible: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub channels: Vec<Channel>,
    pub rows: Vec<Row>,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Ids(pub BTreeMap<String, String>);
// Compatibility for portable_channels until its file-backed integration lands.
// The bound Agent directory must never read or create this retired registry.
pub(super) fn stable_ids(root: &Path, snapshot: &UpstreamSnapshot) -> ModelResult<Ids> {
    let path = root.join("manager-state/model-execution-source-ids.json");
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = TargetFileLock::lock(&path)?;
    let mut ids: Ids = if path.exists() {
        serde_json::from_slice(&safety::read_file(&path, 1024 * 1024)?)?
    } else {
        Ids::default()
    };
    if ids.0.len() > 10000 || ids.0.values().any(|id| require_uuid(id).is_err()) {
        return Err(invalid("execution_identity", "执行渠道身份登记无效"));
    }
    for provider in snapshot.providers["providers"]
        .as_object()
        .ok_or_else(|| invalid("execution_directory", "官方渠道目录无效"))?
        .keys()
    {
        ids.0
            .entry(provider.clone())
            .or_insert_with(|| uuid::Uuid::new_v4().to_string());
    }
    if ids.0.len() > 10000 {
        return Err(invalid("execution_identity", "执行来源身份登记超过上限"));
    }
    let bytes = serde_json::to_vec(&ids)?;
    atomic_write(&path, &bytes, 0o600)?;
    if safety::read_file(&path, 1024 * 1024)? != bytes {
        return Err(invalid("execution_identity", "执行身份登记未读回"));
    }
    Ok(ids)
}
fn rules(row: &Value) -> ModelResult<ModelRules> {
    let mut rules = ModelRules::default();
    if let Some(context) = row["contextWindow"].as_u64() {
        if context > 0 {
            rules.context_window = Field::Custom(context);
        }
    }
    for (name, target) in [
        ("inputModalities", &mut rules.input_modalities),
        ("reasoningEfforts", &mut rules.reasoning_efforts),
    ] {
        if let Some(values) = row[name].as_array() {
            let values = values
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| invalid("execution_directory", "实际型号能力包含非文本值"))
                })
                .collect::<ModelResult<Vec<_>>>()?;
            *target = Field::Custom(values);
        }
    }
    if let Some(default) = row["defaultReasoningEffort"].as_str() {
        rules.default_effort = Field::Custom(default.into());
    }
    if let Some(fast) = row["fastRowAvailable"].as_bool() {
        rules.fast = Field::Custom(fast);
    }
    rules.validate()?;
    Ok(rules)
}
pub fn build(
    root: &Path,
    context: &ProcessContext,
    snapshot: &UpstreamSnapshot,
) -> ModelResult<Directory> {
    let saved = config_file::snapshot(root, &context.opencodex_home)?;
    build_with_policy(snapshot, &saved.policy)
}
fn connection_matches(channel: &ChannelPolicy, raw: &Value) -> bool {
    let connection = &channel.connection;
    connection.protocol == raw["adapter"].as_str().unwrap_or("")
        && connection.nodes.iter().any(|node| {
            node.id == connection.active_node_id
                && Some(node.url.as_str()) == raw["baseUrl"].as_str()
        })
        && connection.auth_mode.as_deref().unwrap_or("key")
            == raw["authMode"].as_str().unwrap_or("key")
}
fn build_with_policy(snapshot: &UpstreamSnapshot, policy: &Policy) -> ModelResult<Directory> {
    let providers = snapshot.providers["providers"]
        .as_object()
        .ok_or_else(|| invalid("execution_directory", "官方渠道目录无效"))?;
    // 官方选择器可见集合：pickerOrder 即上游实际提供给用户选择的型号。
    // 为兼容缺少该字段的旧快照，只有字段存在时才把它当作筛选依据。
    let picker_order = snapshot.order["pickerOrder"].as_array().map(|values| {
        values
            .iter()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect::<BTreeSet<String>>()
    });
    let models = snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("execution_directory", "官方型号目录无效"))?;
    let mut channels = policy
        .channels
        .iter()
        .map(|saved| Channel {
            id: saved.id.clone(),
            label: saved.label.clone(),
            provider_id: saved.connection.provider_id.clone(),
        })
        .collect::<Vec<_>>();
    let mut rows = vec![];
    let mut seen = BTreeSet::new();
    for row in models {
        let id = row["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| invalid("execution_directory", "型号缺少完整原始ID"))?;
        let provider = row["provider"].as_str().unwrap_or("openai");
        let native = row["native"] == true;
        let saved = policy
            .channels
            .iter()
            .find(|channel| channel.connection.provider_id == provider);
        let channel = if native {
            NATIVE
        } else if let Some(saved) = saved {
            saved.id.as_str()
        } else {
            // A stale runtime row must not resurrect a provider deleted from the
            // file or invent a second identity for a newly created channel.
            continue;
        };
        if !seen.insert((channel.to_owned(), id.to_owned())) {
            return Err(invalid("execution_identity", "实际目录存在重复型号身份"));
        }
        let raw = providers.get(provider).unwrap_or(&Value::Null);
        let channel_disabled =
            raw["disabled"] == true || saved.is_some_and(|channel| !channel.connection.enabled);
        let namespaced = row["namespaced"].as_str().filter(|value| !value.is_empty());
        let hidden = row["disabled"] == true
            || row["initialSelectionPending"] == true
            || saved.is_some_and(|channel| match &channel.selection {
                ModelSelection::All => false,
                ModelSelection::None => true,
                ModelSelection::Whitelist(models) => !models.iter().any(|model| model == id),
            })
            || picker_order
                .as_ref()
                .is_some_and(|visible| !visible.contains(namespaced.unwrap_or(id)));
        let reason = if native {
            Some("原生目录尚无管理器执行绑定".into())
        } else if provider.to_ascii_lowercase().contains("a6api") {
            Some("此渠道不用于制模执行".into())
        } else if channel_disabled {
            Some("渠道停用".into())
        } else if saved.is_some_and(|channel| !connection_matches(channel, raw)) {
            Some("渠道连接已变化，请重新读取执行目录".into())
        } else if row["initialSelectionPending"] == true {
            Some("型号尚待首次选择确认".into())
        } else if hidden {
            Some("型号已隐藏，请重新选择".into())
        } else {
            None
        };
        let mut aliases = vec![];
        if let Some(alias) = raw["modelAliases"][id].as_str() {
            aliases.push(alias.into());
        }
        rows.push(Row {
            connection_digest: None,
            model: ModelProjection {
                aliases: vec![],
                channel_id: channel.into(),
                provider_id: provider.into(),
                model_id: id.into(),
                // UI 规范 §29.1：目录读回也统一 渠道名/原始模型ID，不读 modelDisplayNames
                // 或上游 displayName。原生沿原生身份，不补渠道前缀；同一模型 ID 属于两个
                // 渠道时靠渠道前缀天然区分。
                label: if native {
                    row["displayName"]
                        .as_str()
                        .filter(|value| !value.is_empty())
                        .unwrap_or(id)
                        .to_owned()
                } else {
                    let name = channels
                        .iter()
                        .find(|c| c.provider_id == provider)
                        .map(|c| c.label.clone())
                        .unwrap_or_else(|| provider.to_owned());
                    format!("{name}/{id}")
                },
                enabled: reason.is_none(),
                selected: false,
                discovered: true,
                template_ref: None,
                rules: rules(row)?,
                source: BTreeMap::from([("capabilities".into(), "当前已保存官方目录".into())]),
            },
            native_model_id: native.then(|| id.to_owned()),
            picker_visible: reason.is_none(),
            unavailable_reason: reason,
            aliases,
        });
    }
    if rows.iter().any(|r| r.native_model_id.is_some()) {
        channels.push(Channel {
            id: NATIVE.into(),
            label: "官方 / 原生".into(),
            provider_id: "native".into(),
        });
    }
    Ok(Directory { channels, rows })
}
pub fn bound_directory(
    root: &Path,
    context: &ProcessContext,
    snapshot: &UpstreamSnapshot,
) -> ModelResult<Directory> {
    let mut result = build(root, context, snapshot)?;
    for row in &mut result.rows {
        if row.native_model_id.is_none() {
            row.connection_digest = Some(connection_binding(
                context,
                snapshot,
                &row.model.provider_id,
            )?);
        }
    }
    Ok(result)
}
fn connection_binding(
    context: &ProcessContext,
    snapshot: &UpstreamSnapshot,
    provider: &str,
) -> ModelResult<String> {
    let row = &snapshot.providers["providers"][provider];
    Ok(sha256(&serde_json::to_vec(&(
        super::runtime::runtime_binding(context)?,
        provider,
        &row["adapter"],
        &row["baseUrl"],
        &row["authMode"],
        &row["apiKeyTransport"],
        &row["responsesPath"],
    ))?))
}
pub fn bind(
    root: &Path,
    context: &ProcessContext,
    snapshot: &UpstreamSnapshot,
    mut settings: AgentSettings,
) -> ModelResult<AgentSettings> {
    let directory = build(root, context, snapshot)?;
    let chosen = directory
        .rows
        .iter()
        .find(|r| {
            r.model.provider_id == settings.provider_id
                && r.model.model_id == settings.model_id
                && r.native_model_id.is_none()
        })
        .ok_or_else(|| invalid("agent_model", "执行型号已缺失，请重新选择"))?;
    if let Some(reason) = &chosen.unavailable_reason {
        return Err(invalid("agent_model", reason.clone()));
    }
    if settings
        .channel_id
        .as_ref()
        .is_some_and(|id| id != &chosen.model.channel_id)
    {
        return Err(invalid(
            "agent_binding",
            "执行型号的稳定渠道身份不同，请重新选择",
        ));
    }
    if let Some(effort) = &settings.reasoning_effort {
        if !matches!(&chosen.model.rules.reasoning_efforts,Field::Custom(values) if values.contains(effort))
        {
            return Err(invalid("agent_reasoning", "实际型号已不支持所选推理档位"));
        }
    }
    if settings.fast == Some(true) && !matches!(chosen.model.rules.fast, Field::Custom(true)) {
        return Err(invalid("agent_fast", "实际型号没有已核实的快速模式支持"));
    }
    settings.channel_id = Some(chosen.model.channel_id.clone());
    let digest = connection_binding(context, snapshot, &settings.provider_id)?;
    if settings
        .connection_digest
        .as_ref()
        .is_some_and(|old| old != &digest)
    {
        return Err(invalid(
            "agent_binding",
            "选择后渠道端点或认证方式已改变，请重新打开执行模型选择器",
        ));
    }
    settings.connection_digest = Some(digest);
    Ok(settings)
}
pub fn validate_binding(
    root: &Path,
    context: &ProcessContext,
    snapshot: &UpstreamSnapshot,
    settings: &AgentSettings,
) -> ModelResult<()> {
    if settings.channel_id.is_none() || settings.connection_digest.is_none() {
        return Err(invalid(
            "agent_binding",
            "旧执行设置尚未绑定稳定渠道与服务，请重新选择执行模型",
        ));
    }
    let actual = bind(root, context, snapshot, settings.clone())?;
    if actual.connection_digest != settings.connection_digest {
        return Err(invalid(
            "agent_binding",
            "执行渠道端点或认证方式已改变，请重新选择；未自动换模型",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn snapshot() -> UpstreamSnapshot {
        UpstreamSnapshot {
            version: "2.84.0".into(),
            providers: json!({"defaultProvider":"custom","providers":{"custom":{"adapter":"openai-chat","baseUrl":"https://fixture.example.test/v1","authMode":"key","selectedModels":["chosen"],"modelDisplayNames":{"hidden":"完整隐藏型号"},"modelAliases":{"hidden":"short"}},"off":{"disabled":true},"a6api":{"adapter":"openai-chat"}}}),
            models: json!([{"provider":"custom","id":"chosen","namespaced":"custom/chosen","reasoningEfforts":["none","high"],"defaultReasoningEffort":"high","fastRowAvailable":false},{"provider":"custom","id":"hidden","namespaced":"custom/hidden","disabled":true,"reasoningEfforts":["none","high"],"defaultReasoningEffort":"high","fastRowAvailable":false},{"provider":"openai","native":true,"id":"native-model","namespaced":"native-model"},{"provider":"off","id":"unavailable","namespaced":"off/unavailable"},{"provider":"a6api","id":"a","namespaced":"a6api/a"}]),
            // 隐藏型号（disabled: true）不在官方选择器可见集合里，执行目录须与之一致。
            order: json!({"pickerOrder":["custom/chosen","native-model","off/unavailable","a6api/a"]}),
            catalog: json!({"models":[]}),
        }
    }
    fn context(root: &Path) -> ProcessContext {
        ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                root.join("fixture-ocx"),
            ),
            working_directory: root.into(),
            opencodex_home: root.join("upstream"),
        }
    }
    fn write_config(context: &ProcessContext, snapshot: &UpstreamSnapshot) {
        std::fs::create_dir_all(&context.opencodex_home).unwrap();
        let mut config = snapshot.providers.clone();
        if let Some(order) = snapshot.order.get("pickerOrder") {
            config["modelPickerOrder"] = order.clone();
        }
        std::fs::write(
            context.opencodex_home.join("config.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
    }
    fn settings() -> AgentSettings {
        AgentSettings {
            provider_id: "custom".into(),
            model_id: "chosen".into(),
            reasoning_effort: Some("none".into()),
            fast: Some(false),
            channel_id: None,
            connection_digest: None,
        }
    }
    #[test]
    fn full_execution_directory_is_stable_includes_selector_hidden_models_and_explains_native_binding(
    ) {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = snapshot();
        write_config(&context, &snapshot);
        let saved = config_file::snapshot(root.path(), &context.opencodex_home).unwrap();
        let first = bound_directory(root.path(), &context, &snapshot).unwrap();
        let second = bound_directory(root.path(), &context, &snapshot).unwrap();
        assert_eq!(
            first.channels.iter().map(|c| &c.id).collect::<Vec<_>>(),
            second.channels.iter().map(|c| &c.id).collect::<Vec<_>>()
        );
        let hidden = first
            .rows
            .iter()
            .find(|r| r.model.model_id == "hidden")
            .unwrap();
        // 隐藏型号不再可选；§29.1：名称统一 渠道名/原始模型ID，不读旧 modelDisplayNames。
        assert!(!hidden.model.enabled);
        assert!(!hidden.picker_visible);
        assert_eq!(hidden.model.label, "custom/hidden");
        assert_eq!(hidden.aliases, vec!["short"]);
        assert_eq!(
            hidden.unavailable_reason.as_deref(),
            Some("型号已隐藏，请重新选择")
        );
        assert!(
            first
                .rows
                .iter()
                .find(|r| r.model.model_id == "chosen")
                .unwrap()
                .picker_visible
        );
        // 没有展示名时必须回落带渠道前缀的 namespaced，不能出现“有的带前缀、有的不带”。
        let chosen = first
            .rows
            .iter()
            .find(|r| r.model.model_id == "chosen")
            .unwrap();
        assert_eq!(chosen.model.label, "custom/chosen");
        assert_eq!(
            chosen.model.channel_id,
            saved
                .policy
                .channels
                .iter()
                .find(|c| c.connection.provider_id == "custom")
                .unwrap()
                .id
        );
        assert!(chosen.connection_digest.is_some());
        let native = first
            .rows
            .iter()
            .find(|r| r.native_model_id.is_some())
            .unwrap();
        assert_eq!(native.native_model_id.as_deref(), Some("native-model"));
        assert_eq!(
            native.unavailable_reason.as_deref(),
            Some("原生目录尚无管理器执行绑定")
        );
        assert_eq!(
            first
                .rows
                .iter()
                .find(|r| r.model.provider_id == "off")
                .unwrap()
                .unavailable_reason
                .as_deref(),
            Some("渠道停用")
        );
        assert!(
            !first
                .rows
                .iter()
                .find(|r| r.model.provider_id == "a6api")
                .unwrap()
                .model
                .enabled
        );
        assert!(!root.path().join(POLICY_PATH).exists());
        assert!(!root
            .path()
            .join("manager-state/model-execution-source-ids.json")
            .exists());
    }
    #[test]
    fn saved_execution_identity_rejects_endpoint_auth_or_root_changes_and_preserves_explicit_none_false(
    ) {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = snapshot();
        write_config(&context, &snapshot);
        let settings = bind(root.path(), &context, &snapshot, settings()).unwrap();
        require_uuid(settings.channel_id.as_deref().unwrap()).unwrap();
        validate_binding(root.path(), &context, &snapshot, &settings).unwrap();
        assert_eq!(settings.reasoning_effort.as_deref(), Some("none"));
        assert_eq!(settings.fast, Some(false));
        let mut changed = snapshot.clone();
        changed.providers["providers"]["custom"]["baseUrl"] =
            json!("https://another.example.test/v1");
        write_config(&context, &changed);
        assert_eq!(
            validate_binding(root.path(), &context, &changed, &settings)
                .unwrap_err()
                .code,
            "agent_binding"
        );
        let mut changed = snapshot.clone();
        changed.providers["providers"]["custom"]["authMode"] = json!("oauth");
        write_config(&context, &changed);
        assert!(validate_binding(root.path(), &context, &changed, &settings).is_err());
        let mut other = context;
        other.opencodex_home = root.path().join("other");
        write_config(&other, &snapshot);
        assert!(validate_binding(root.path(), &other, &snapshot, &settings).is_err());
        let mut legacy = settings.clone();
        legacy.connection_digest = None;
        assert!(validate_binding(root.path(), &other, &snapshot, &legacy).is_err());
        let mut unsupported = settings;
        unsupported.fast = Some(true);
        assert_eq!(
            bind(root.path(), &other, &snapshot, unsupported)
                .unwrap_err()
                .code,
            "agent_fast"
        );
    }

    #[test]
    fn file_layout_identity_wins_with_a_corrupt_legacy_draft_and_identity_registry() {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = snapshot();
        write_config(&context, &snapshot);
        let mut layout = config_file::snapshot(root.path(), &context.opencodex_home)
            .unwrap()
            .policy;
        let channel = layout
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == "custom")
            .unwrap();
        channel.id = uuid::Uuid::new_v4().to_string();
        channel.label = "文件布局渠道".into();
        let expected_id = channel.id.clone();
        std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
        std::fs::write(
            root.path().join("manager-state/model-channel-layout.json"),
            serde_json::to_vec(&layout).unwrap(),
        )
        .unwrap();
        let mut legacy = layout.clone();
        legacy.channels.iter_mut().for_each(|c| {
            c.id = uuid::Uuid::new_v4().to_string();
            c.label = "过期旧副本".into();
        });
        std::fs::write(
            root.path().join(POLICY_PATH),
            serde_json::to_vec(&legacy).unwrap(),
        )
        .unwrap();
        std::fs::write(root.path().join(DRAFT_PATH), b"{broken draft").unwrap();
        let registry = root
            .path()
            .join("manager-state/model-execution-source-ids.json");
        std::fs::write(&registry, b"{broken registry").unwrap();
        for _ in 0..2 {
            let actual = bound_directory(root.path(), &context, &snapshot).unwrap();
            let channel = actual
                .channels
                .iter()
                .find(|c| c.provider_id == "custom")
                .unwrap();
            assert_eq!(channel.id, expected_id);
            assert_eq!(channel.label, "文件布局渠道");
            let row = actual
                .rows
                .iter()
                .find(|r| r.model.model_id == "chosen")
                .unwrap();
            assert_eq!(row.model.channel_id, expected_id);
            assert_eq!(row.model.label, "文件布局渠道/chosen");
            let bound = bind(
                root.path(),
                &context,
                &snapshot,
                AgentSettings {
                    channel_id: Some(expected_id.clone()),
                    connection_digest: row.connection_digest.clone(),
                    ..settings()
                },
            )
            .unwrap();
            validate_binding(root.path(), &context, &snapshot, &bound).unwrap();
        }
        assert_eq!(std::fs::read(registry).unwrap(), b"{broken registry");
        assert_eq!(
            std::fs::read(root.path().join(DRAFT_PATH)).unwrap(),
            b"{broken draft"
        );
        std::fs::write(root.path().join(POLICY_PATH), b"{broken legacy policy").unwrap();
        let actual = bound_directory(root.path(), &context, &snapshot).unwrap();
        assert_eq!(
            actual
                .channels
                .iter()
                .find(|c| c.provider_id == "custom")
                .unwrap()
                .id,
            expected_id
        );
    }

    #[test]
    fn changed_file_connection_or_recreated_identity_cannot_reuse_the_old_runtime_binding() {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = snapshot();
        write_config(&context, &snapshot);
        let bound = bind(root.path(), &context, &snapshot, settings()).unwrap();
        let mut changed = snapshot.clone();
        changed.providers["providers"]["custom"]["baseUrl"] =
            json!("https://changed.example.test/v1");
        write_config(&context, &changed);
        let bytes = std::fs::read(context.opencodex_home.join("config.json")).unwrap();
        // Even an older upstream read cannot execute after the file changed.
        let directory = bound_directory(root.path(), &context, &snapshot).unwrap();
        let row = directory
            .rows
            .iter()
            .find(|r| r.model.model_id == "chosen")
            .unwrap();
        assert!(!row.model.enabled);
        assert!(row
            .unavailable_reason
            .as_deref()
            .unwrap()
            .contains("连接已变化"));
        assert!(validate_binding(root.path(), &context, &snapshot, &bound).is_err());
        assert_eq!(
            std::fs::read(context.opencodex_home.join("config.json")).unwrap(),
            bytes
        );
        write_config(&context, &snapshot);
        let mut layout = config_file::snapshot(root.path(), &context.opencodex_home)
            .unwrap()
            .policy;
        let recreated = layout
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == "custom")
            .unwrap();
        recreated.id = uuid::Uuid::new_v4().to_string();
        recreated.label = "重建渠道".into();
        std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
        std::fs::write(
            root.path().join("manager-state/model-channel-layout.json"),
            serde_json::to_vec(&layout).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_binding(root.path(), &context, &snapshot, &bound)
                .unwrap_err()
                .code,
            "agent_binding"
        );
    }

    #[test]
    fn exact_file_model_without_an_execution_route_can_be_saved_without_changing_preferences() {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = snapshot();
        write_config(&context, &snapshot);
        let original = bind(root.path(), &context, &snapshot, settings()).unwrap();
        let current = config_file::snapshot(root.path(), &context.opencodex_home).unwrap();
        let mut policy = current.policy;
        let channel = policy
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == "custom")
            .unwrap();
        let exact_id = "site/exact-unlisted-model";
        channel.discovered.push(DiscoveredModel {
            id: exact_id.into(),
            label: exact_id.into(),
            rules: ModelRules::default(),
        });
        channel.model_order.push(exact_id.into());
        channel.selection = ModelSelection::Whitelist(vec![exact_id.into()]);
        let result = config_file::save(
            root.path(),
            &context.opencodex_home,
            config_file::SaveRequest {
                expected_file: current.policy_digest,
                policy,
                credentials: vec![],
            },
        )
        .unwrap();
        assert!(result.changed);
        let saved_channel = result
            .snapshot
            .policy
            .channels
            .iter()
            .find(|c| c.connection.provider_id == "custom")
            .unwrap();
        assert!(saved_channel.discovered.iter().any(|m| m.id == exact_id));
        let bytes = std::fs::read(context.opencodex_home.join("config.json")).unwrap();
        let chosen = AgentSettings {
            model_id: exact_id.into(),
            ..settings()
        };
        let error = bind(root.path(), &context, &snapshot, chosen.clone()).unwrap_err();
        assert_eq!(error.code, "agent_model");
        // A newer saved selection also invalidates a previously selected model
        // while the upstream response still contains its old picker order.
        assert_eq!(
            validate_binding(root.path(), &context, &snapshot, &original)
                .unwrap_err()
                .code,
            "agent_model"
        );
        assert_eq!(chosen.model_id, exact_id);
        assert_eq!(
            std::fs::read(context.opencodex_home.join("config.json")).unwrap(),
            bytes
        );
        let mut removed = snapshot.clone();
        removed.providers["providers"]
            .as_object_mut()
            .unwrap()
            .remove("custom");
        write_config(&context, &removed);
        let directory = bound_directory(root.path(), &context, &snapshot).unwrap();
        assert!(!directory.channels.iter().any(|c| c.provider_id == "custom"));
        assert!(!directory
            .rows
            .iter()
            .any(|r| r.model.provider_id == "custom"));
    }

    /// WP-64 / §29.1：同一原始 ID 属于两个渠道时用 渠道名/原始ID 区分，不读旧展示名。
    #[test]
    fn catalog_labels_distinguish_same_raw_id_across_channels() {
        let root = tempfile::tempdir().unwrap();
        let context = context(root.path());
        let snapshot = UpstreamSnapshot {
            providers: json!({"providers":{
                "first":{"name":"FeiMao-CNM","adapter":"openai-chat","baseUrl":"https://first.example.test/v1","selectedModels":["deepseek-v4.1-flash"]},
                "second":{"name":"FeiMao-Claude","adapter":"openai-chat","baseUrl":"https://second.example.test/v1","selectedModels":["deepseek-v4.1-flash"]}
            }}),
            models: json!([
                {"provider":"first","id":"deepseek-v4.1-flash","displayName":"旧友好名称"},
                {"provider":"second","id":"deepseek-v4.1-flash","displayName":"旧友好名称"},
                {"provider":"openai","native":true,"id":"native-model","displayName":"原生型号"}
            ]),
            order: json!({}),
            ..snapshot()
        };
        write_config(&context, &snapshot);
        let directory = bound_directory(root.path(), &context, &snapshot).unwrap();
        let rows = directory
            .rows
            .iter()
            .filter(|r| r.native_model_id.is_none())
            .collect::<Vec<_>>();
        let labels = rows
            .iter()
            .map(|row| row.model.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                "FeiMao-CNM/deepseek-v4.1-flash",
                "FeiMao-Claude/deepseek-v4.1-flash"
            ]
        );
        assert_ne!(labels[0], labels[1]);
        assert_ne!(rows[0].model.channel_id, rows[1].model.channel_id);
        assert!(rows.iter().all(|row| row.model.enabled));
        // 旧友好名称不得作为任何一项的主名称。
        assert!(labels.iter().all(|label| !label.contains("旧友好名称")));
        // 原生渠道沿原生身份，不补渠道前缀。
        let native = directory
            .rows
            .iter()
            .find(|row| row.native_model_id.is_some())
            .unwrap();
        assert_eq!(native.model.label, "原生型号");
        assert!(native.connection_digest.is_none());
    }
}
