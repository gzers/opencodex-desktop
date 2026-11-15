//! Pure independent route declaration from native config bytes. No I/O or native permission.
//! None means unresolved; Some(empty) is an explicitly empty route set.
use crate::{
    errors::{AppError, AppResult},
    modules::model_config::schema_v2::parse_strict_json_v2,
};

fn error<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_session_unconfirmed".into(),
        detail: "启动原件与当前服务的绑定未确认；原件和现场保留，未恢复客户端文件".into(),
    }
}
fn declared_id_list(value: Option<&serde_json::Value>) -> Option<Vec<&str>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let rows = value.as_array()?;
    if rows.len() > 4096 {
        return None;
    }
    rows.iter()
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
        })
        .collect()
}
fn declaration_slug_key(slug: &str) -> String {
    match slug.split_once('/') {
        Some((provider, id)) if !provider.is_empty() => {
            format!("{provider}/{}", id.replace('/', "-"))
        }
        _ => slug.to_string(),
    }
}
fn combo_identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().next().is_some_and(|v| v.is_ascii_alphanumeric())
        && id
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || b"._-".contains(&v))
}
fn add_combo_declarations(
    value: &serde_json::Value,
    providers: &serde_json::Map<String, serde_json::Value>,
    native_ids: &std::collections::BTreeSet<(String, String)>,
    routes: &mut std::collections::BTreeSet<String>,
) -> Option<()> {
    let Some(combos) = value.get("combos") else {
        return Some(());
    };
    let combos = combos.as_object()?;
    if combos.len() > 1024 || !combos.is_empty() && providers.contains_key("combo") {
        return None;
    }
    for (id, combo) in combos {
        if !combo_identifier(id)
            || providers.contains_key(id)
            || !combo.is_object()
            || combo
                .get("nativeAlias")
                .is_some_and(|v| !v.is_boolean() || v == &serde_json::Value::Bool(true))
        {
            return None;
        }
        let public = match combo.get("alias") {
            None | Some(serde_json::Value::Null) => format!("combo/{id}"),
            Some(serde_json::Value::String(alias)) => {
                let alias = alias.trim();
                if alias.is_empty() {
                    format!("combo/{id}")
                } else {
                    let segments: Vec<_> = alias.split('/').collect();
                    if segments.len() > 2
                        || !segments.iter().all(|s| combo_identifier(s))
                        || alias == "combo"
                        || alias.starts_with("combo/")
                        || segments.len() == 1
                            && ["gpt-", "o1-", "o3-", "o4-", "codex-"]
                                .iter()
                                .any(|p| alias.starts_with(p))
                    {
                        return None;
                    }
                    alias.to_string()
                }
            }
            _ => return None,
        };
        let targets = combo.get("targets")?.as_array()?;
        if targets.is_empty() || targets.len() > 256 {
            return None;
        }
        let mut seen = std::collections::BTreeSet::new();
        for target in targets {
            let provider = target.get("provider")?.as_str()?.trim();
            let model = target.get("model")?.as_str()?.trim();
            if !native_ids.contains(&(provider.into(), model.into()))
                || !seen.insert((provider, model))
            {
                return None;
            }
        }
        if !routes.insert(public) || routes.len() > 4096 {
            return None;
        }
    }
    Some(())
}
// Independent explicit selection declaration. An unbounded discovery graph
// stays unresolved; nonempty selectedModels is the official final allowlist.
// It is never learned from the generated catalog or a discovery response.
// The actual content policy subsequently requires EXACT ready route coverage.
pub fn declaration(bytes: &[u8]) -> AppResult<Option<Vec<String>>> {
    // Preserve the original native config-read bound at the shared byte API.
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(error(()));
    }
    let bytes = zeroize::Zeroizing::new(bytes.to_vec());
    let value = parse_strict_json_v2(&bytes).map_err(error)?;
    let Some(providers) = value
        .get("providers")
        .and_then(serde_json::Value::as_object)
    else {
        return Ok(None);
    };
    // Official default/model aliases label rows; they do not change a provider
    // row's catalogModelSlug. Omitted global/provider flags inherit false.
    // Never promote a friendly display alias into independent route authority.
    if value
        .get("defaultModelAliases")
        .is_some_and(|v| !v.is_boolean())
    {
        return Ok(None);
    }
    // Official schema defaults omitted fastRows to true. This boolean
    // normalization does not change independently selected base routes. The
    // content policy STILL refuses undeclared Fast/effort siblings in Ready.
    if value.get("fastRows").is_some_and(|v| !v.is_boolean()) {
        return Ok(None);
    }
    let Some(disabled) = declared_id_list(value.get("disabledModels")) else {
        return Ok(None);
    };
    let disabled: std::collections::BTreeSet<_> =
        disabled.into_iter().map(declaration_slug_key).collect();
    let mut routes = std::collections::BTreeSet::new();
    let mut native_ids = std::collections::BTreeSet::new();
    let mut custom_by_provider: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    if let Some(custom) = value.get("customModels") {
        let Some(custom) = custom.as_array().filter(|rows| rows.len() <= 4096) else {
            return Ok(None);
        };
        let mut seen = std::collections::BTreeSet::new();
        for row in custom {
            let Some(provider) = row.get("provider").and_then(serde_json::Value::as_str) else {
                return Ok(None);
            };
            let Some(id) = row
                .get("modelId")
                .and_then(serde_json::Value::as_str)
                .filter(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
            else {
                return Ok(None);
            };
            if !providers.contains_key(provider) || !seen.insert((provider, id)) {
                return Ok(None);
            }
            custom_by_provider.entry(provider).or_default().push(id);
        }
    }
    for (provider, entry) in providers {
        if !entry.is_object() || entry.get("disabled").is_some_and(|v| !v.is_boolean()) {
            return Ok(None);
        }
        if entry.get("disabled") == Some(&serde_json::Value::Bool(true)) {
            continue;
        }
        if provider.is_empty()
            || provider.len() > 256
            || !provider
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || entry.get("liveModels").is_some_and(|v| !v.is_boolean())
            || entry.get("defaultAliases").is_some_and(|v| !v.is_boolean())
            || entry.get("modelAliases").is_some_and(|v| {
                !v.as_object().is_some_and(|o| {
                    o.iter().all(|(id, alias)| {
                        !id.is_empty()
                            && id.len() <= 1024
                            && !id.chars().any(char::is_control)
                            && alias.as_str().is_some_and(|s| {
                                s.len() <= 1024 && !s.chars().any(char::is_control)
                            })
                    })
                })
            })
        {
            return Ok(None);
        }
        let Some(selected) = declared_id_list(entry.get("selectedModels")) else {
            return Ok(None);
        };
        let models = if !selected.is_empty() {
            selected
        } else {
            // Empty/omitted selection means ALL models, not an empty allowlist.
            // Only a static provider supplies a complete independent roster.
            if entry.get("liveModels") != Some(&serde_json::Value::Bool(false)) {
                return Ok(None);
            }
            let Some(configured) = declared_id_list(entry.get("models")) else {
                return Ok(None);
            };
            let Some(retained) = declared_id_list(entry.get("retainModels")) else {
                return Ok(None);
            };
            let mut ids: std::collections::BTreeSet<_> =
                configured.iter().copied().chain(retained).collect();
            if configured.is_empty() {
                if let Some(default) = entry.get("defaultModel") {
                    let Some(id) = default
                        .as_str()
                        .filter(|s| s.len() <= 1024 && !s.chars().any(char::is_control))
                    else {
                        return Ok(None);
                    };
                    if !id.is_empty() {
                        ids.insert(id);
                    }
                }
            }
            ids.into_iter().collect()
        };
        // Same native model may be overlaid with custom metadata; different
        // native IDs that encode to one slug are still ambiguous and refuse.
        // Check this even for unselected/disabled custom rows, before filtering.
        let mut encoded_custom = std::collections::BTreeMap::new();
        for id in models.iter().copied().chain(
            custom_by_provider
                .get(provider.as_str())
                .into_iter()
                .flatten()
                .copied(),
        ) {
            let encoded = id.replace('/', "-");
            if encoded_custom
                .insert(encoded, id)
                .is_some_and(|prior| prior != id)
            {
                return Ok(None);
            }
        }
        let mut models = models;
        if entry
            .get("selectedModels")
            .is_none_or(|v| v.as_array().is_some_and(|s| s.is_empty()))
        {
            if let Some(custom) = custom_by_provider.get(provider.as_str()) {
                for id in custom {
                    if !models.contains(id) {
                        models.push(id);
                    }
                }
            }
        }
        for model in models {
            native_ids.insert((provider.clone(), model.to_string()));
            // Original 2.99 providers/slug-codec::routedSlug maps inner slash to
            // hyphen, NOT a blind unencoded provider/model concatenation.
            if !routes.insert(format!("{provider}/{}", model.replace('/', "-")))
                || routes.len() > 4096
            {
                return Ok(None);
            }
        }
    }
    if add_combo_declarations(&value, providers, &native_ids, &mut routes).is_none() {
        return Ok(None);
    }
    Ok(Some(
        routes
            .into_iter()
            .filter(|slug| !disabled.contains(&declaration_slug_key(slug)))
            .collect(),
    ))
}

#[cfg(test)]
#[path = "client_route_declaration_tests.rs"]
mod tests;
