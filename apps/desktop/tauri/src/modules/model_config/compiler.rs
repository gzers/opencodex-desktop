//! Pure draft projection; catalog occupancy requires separate complete evidence.
use super::{matching, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProjection {
    pub channel_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub label: String,
    pub enabled: bool,
    pub selected: bool,
    pub discovered: bool,
    pub template_ref: Option<TemplateRef>,
    pub rules: ModelRules,
    pub source: BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftProjection {
    pub rows: Vec<ModelProjection>,
    pub known_active_models: usize,
    pub exact_catalog_count: Option<usize>,
    pub notices: Vec<String>,
}
pub fn validate_channel(channel: &ChannelPolicy) -> ModelResult<()> {
    if let Some(reference) = &channel.connection.imported_config {
        require_uuid(&reference.id)?;
        if !reference.digest.starts_with("sha256:") || reference.digest.len() != 71 {
            return Err(invalid("provider_import_digest", "原渠道接收引用摘要无效"));
        }
    }

    require_uuid(&channel.id)?;
    if channel.label.trim().is_empty() || channel.label.len() > 256 {
        return Err(invalid("channel", "渠道名称无效"));
    }
    let connection = &channel.connection;
    if connection
        .auth_mode
        .as_deref()
        .is_some_and(|mode| !["key", "local", "oauth", "forward"].contains(&mode))
    {
        return Err(invalid("auth_mode", "渠道认证方式无效"));
    }
    if connection.provider_id.is_empty()
        || connection.provider_id.len() > 128
        || !connection
            .provider_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(invalid(
            "provider",
            "运行渠道标识仅支持字母、数字、连字符和下划线",
        ));
    }
    ModelRules {
        protocol: Field::Custom(connection.protocol.clone()),
        ..Default::default()
    }
    .validate()?;
    if connection.nodes.is_empty() || connection.nodes.len() > 32 {
        return Err(invalid("nodes", "渠道须有1至32个服务节点"));
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut urls = BTreeSet::new();
    for node in &connection.nodes {
        require_uuid(&node.id)?;
        let url =
            reqwest::Url::parse(&node.url).map_err(|_| invalid("node_url", "服务节点地址无效"))?;
        if !["http", "https"].contains(&url.scheme())
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid(
                "node_url",
                "节点须为不含凭据、参数或片段的HTTP/HTTPS地址",
            ));
        }
        let path = url.path().trim_end_matches('/');
        if path.ends_with("/responses")
            || path.ends_with("/chat/completions")
            || path.ends_with("/messages")
        {
            return Err(invalid("node_url", "填写服务基地址，不填写生成接口"));
        }
        if node.name.trim().is_empty()
            || !ids.insert(&node.id)
            || !names.insert(node.name.trim().to_lowercase())
            || !urls.insert(url.as_str().trim_end_matches('/').to_string())
        {
            return Err(invalid("nodes", "节点名称/地址/身份为空或重复"));
        }
    }
    if !ids.contains(&connection.active_node_id) {
        return Err(invalid("active_node", "当前服务节点不存在"));
    }
    let unique = |values: &[String]| {
        !values.iter().any(|s| s.is_empty() || s.len() > 2048)
            && values.iter().collect::<BTreeSet<_>>().len() == values.len()
    };
    if !unique(&channel.model_order) {
        return Err(invalid("model_order", "模型顺序包含空或重复身份"));
    }
    if let ModelSelection::Whitelist(values) = &channel.selection {
        if values.is_empty() || !unique(values) {
            return Err(invalid(
                "selection",
                "白名单须为非空唯一模型ID；零项请明确选择全部隐藏",
            ));
        }
    }
    let mut models = BTreeSet::new();
    for model in &channel.discovered {
        if model.id.is_empty() || model.id.len() > 2048 || !models.insert(&model.id) {
            return Err(invalid("models", "发现清单包含空或重复模型"));
        }
        model.rules.validate()?;
        if model.label != model.id
            && !model.label.is_empty()
            && (model.label.trim() != model.label
                || model.label.chars().count() > 128
                || model.label.contains('/')
                || model.label.chars().any(char::is_control))
        {
            return Err(invalid(
                "model_label",
                "模型显示名称须为128字符以内、不含斜线或控制字符的文本；留空使用原模型ID",
            ));
        }
    }
    channel.defaults.validate()?;
    for rules in channel
        .overrides
        .values()
        .chain(channel.family_overrides.values())
    {
        rules.validate()?;
    }
    Ok(())
}
fn overlay_sources(target: &mut BTreeMap<String, String>, rules: &ModelRules, source: &str) {
    for (key, custom) in [
        ("protocol", !matches!(rules.protocol, Field::Inherit)),
        (
            "contextWindow",
            !matches!(rules.context_window, Field::Inherit),
        ),
        (
            "inputModalities",
            !matches!(rules.input_modalities, Field::Inherit),
        ),
        (
            "reasoningEfforts",
            !matches!(rules.reasoning_efforts, Field::Inherit),
        ),
        (
            "defaultEffort",
            !matches!(rules.default_effort, Field::Inherit),
        ),
        ("fast", !matches!(rules.fast, Field::Inherit)),
    ] {
        if custom {
            target.insert(key.into(), source.into());
        }
    }
}
pub fn project(library: &Library, policy: &Policy) -> ModelResult<DraftProjection> {
    let mut rows = vec![];
    let mut ids = BTreeSet::new();
    let mut providers = BTreeSet::new();
    let mut notices = vec![];
    for channel in &policy.channels {
        validate_channel(channel)?;
        if !ids.insert(&channel.id) || !providers.insert(&channel.connection.provider_id) {
            return Err(invalid("channel", "渠道身份或运行绑定重复"));
        }
        let mut order = channel.model_order.clone();
        for model in &channel.discovered {
            if !order.contains(&model.id) {
                order.push(model.id.clone());
            }
        }
        if let ModelSelection::Whitelist(selected) = &channel.selection {
            for id in selected {
                if !order.contains(id) {
                    order.push(id.clone());
                }
            }
        }
        for id in order {
            let discovered = channel.discovered.iter().find(|m| m.id == id);
            let mut rules = discovered.map(|m| m.rules.clone()).unwrap_or_default();
            rules.protocol = Field::Custom(channel.connection.protocol.clone());
            let mut source = BTreeMap::new();
            overlay_sources(&mut source, &rules, "上游声明");
            source.insert("protocol".into(), "渠道默认".into());
            let adopted = matching::resolve(library, policy, &channel.id, &id)?;
            if let Some((_, template)) = &adopted {
                rules = template.overlay(&rules);
                overlay_sources(&mut source, template, "已采用模版");
            }
            let protocol = rules.protocol.clone();
            rules = channel.defaults.overlay(&rules);
            overlay_sources(&mut source, &channel.defaults, "渠道默认");
            rules.protocol = protocol;
            if adopted
                .as_ref()
                .is_some_and(|(_, r)| !matches!(r.protocol, Field::Inherit))
            {
                source.insert("protocol".into(), "已采用模版".into());
            }
            if let Some((reference, _)) = &adopted {
                if let Some(overrides) = channel.family_overrides.get(&reference.template_id) {
                    rules = overrides.overlay(&rules);
                    overlay_sources(&mut source, overrides, "渠道族覆盖");
                }
            }
            if let Some(overrides) = channel.overrides.get(&id) {
                rules = overrides.overlay(&rules);
                overlay_sources(&mut source, overrides, "单模型覆盖");
            }
            rules.validate()?;
            let selected = match &channel.selection {
                ModelSelection::All => discovered.is_some(),
                ModelSelection::None => false,
                ModelSelection::Whitelist(values) => values.contains(&id),
            };
            if selected && discovered.is_none() {
                notices.push(format!(
                    "{}：模型{}未在当前清单发现，保留选择待核对",
                    channel.label, id
                ));
            }
            rows.push(ModelProjection {
                channel_id: channel.id.clone(),
                provider_id: channel.connection.provider_id.clone(),
                model_id: id.clone(),
                label: discovered.map(|m| m.label.clone()).unwrap_or(id),
                enabled: channel.connection.enabled,
                selected,
                discovered: discovered.is_some(),
                template_ref: adopted.map(|(r, _)| r),
                rules,
                source,
            });
        }
    }
    let known_active_models = rows
        .iter()
        .filter(|r| r.selected && r.enabled && r.discovered)
        .count();
    notices
        .push("当前为草稿基础模型投影；原生、别名、隐藏行及客户端分页占用须用真实目录核对".into());
    Ok(DraftProjection {
        rows,
        known_active_models,
        exact_catalog_count: None,
        notices,
    })
}
