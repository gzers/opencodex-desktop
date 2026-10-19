//! Build an exact public runtime restoration from real historical snapshots.
//! Later unrelated providers survive; an affected provider's unrecognized drift
//! invalidates the operation instead of being overwritten by an old full config.
use super::{recovery::Side, schema::*, upstream::UpstreamSnapshot};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Projection {
    pub providers: Value,
    pub order: Vec<String>,
    pub catalog: BTreeSet<String>,
    pub disabled: Vec<String>,
    pub affected: Vec<String>,
}
pub struct RecoveryScope<'a> {
    pub managed: &'a BTreeSet<String>,
    pub authorization_changes: &'a BTreeSet<String>,
    pub all: bool,
}
fn map(snapshot: &UpstreamSnapshot) -> ModelResult<&serde_json::Map<String, Value>> {
    snapshot.providers["providers"]
        .as_object()
        .ok_or_else(|| invalid("history_snapshot", "历史官方渠道结构无效"))
}
fn strings(value: &Value) -> ModelResult<Vec<String>> {
    value
        .as_array()
        .ok_or_else(|| invalid("history_snapshot", "历史顺序结构无效"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("history_snapshot", "历史顺序含非文本项"))
        })
        .collect()
}
fn catalog(snapshot: &UpstreamSnapshot) -> ModelResult<BTreeSet<String>> {
    let rows = snapshot.catalog["models"]
        .as_array()
        .ok_or_else(|| invalid("history_snapshot", "历史目录结构无效"))?;
    let ids = rows
        .iter()
        .map(|r| {
            r["slug"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("history_snapshot", "历史目录缺少原始身份"))
        })
        .collect::<ModelResult<BTreeSet<_>>>()?;
    if ids.len() != rows.len() {
        return Err(invalid("history_snapshot", "历史目录存在重复身份"));
    }
    Ok(ids)
}
fn namespaces(
    snapshot: &UpstreamSnapshot,
    providers: &BTreeSet<String>,
) -> ModelResult<BTreeSet<String>> {
    let map = map(snapshot)?;
    let mut values = BTreeSet::new();
    let mut owners = BTreeMap::new();
    for (id, row) in map {
        for namespace in [Some(id.as_str()), row["alias"].as_str()]
            .into_iter()
            .flatten()
        {
            if let Some(previous) = owners.insert(namespace, id) {
                if previous != id {
                    return Err(invalid(
                        "history_namespace",
                        "渠道命名空间存在碰撞，无法精确恢复目录",
                    ));
                }
            }
        }
    }
    for id in providers {
        values.insert(id.clone());
        if let Some(alias) = map.get(id).and_then(|r| r["alias"].as_str()) {
            values.insert(alias.into());
        }
    }
    Ok(values)
}
fn attributed(
    snapshot: &UpstreamSnapshot,
    providers: &BTreeSet<String>,
) -> ModelResult<BTreeSet<String>> {
    let namespaces = namespaces(snapshot, providers)?;
    let rows = snapshot
        .models
        .as_array()
        .ok_or_else(|| invalid("history_snapshot", "历史模型清单结构无效"))?;
    let explicit = rows
        .iter()
        .filter(|r| {
            r["provider"]
                .as_str()
                .is_some_and(|p| providers.contains(p))
        })
        .filter_map(|r| r["namespaced"].as_str())
        .collect::<BTreeSet<_>>();
    Ok(catalog(snapshot)?
        .into_iter()
        .filter(|slug| {
            explicit.contains(slug.as_str())
                || slug
                    .split_once('/')
                    .is_some_and(|(prefix, _)| namespaces.contains(prefix))
        })
        .collect())
}
fn affected_rows(
    before: &UpstreamSnapshot,
    after: &UpstreamSnapshot,
) -> ModelResult<BTreeSet<String>> {
    let a = before
        .models
        .as_array()
        .ok_or_else(|| invalid("history_snapshot", "历史清单无效"))?;
    let b = after
        .models
        .as_array()
        .ok_or_else(|| invalid("history_snapshot", "历史清单无效"))?;
    let mut providers = BTreeSet::new();
    for row in a.iter().chain(b) {
        let Some(provider) = row["provider"].as_str() else {
            continue;
        };
        let id = &row["id"];
        if a.iter()
            .find(|r| r["provider"] == provider && &r["id"] == id)
            != b.iter()
                .find(|r| r["provider"] == provider && &r["id"] == id)
        {
            providers.insert(provider.into());
        }
    }
    Ok(providers)
}
pub fn project(
    current: &UpstreamSnapshot,
    before: &UpstreamSnapshot,
    after: &UpstreamSnapshot,
    disabled_sets: [&[String]; 3],
    side: Side,
    scope: RecoveryScope<'_>,
) -> ModelResult<Projection> {
    let RecoveryScope {
        managed,
        authorization_changes,
        all,
    } = scope;
    let [current_disabled, before_disabled, after_disabled] = disabled_sets;
    let before_map = map(before)?;
    let after_map = map(after)?;
    let current_map = map(current)?;
    let row_changes = affected_rows(before, after)?;
    let mut affected = managed
        .iter()
        .filter(|id| {
            all || authorization_changes.contains(*id)
                || before_map.get(*id) != after_map.get(*id)
                || row_changes.contains(*id)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    if before.order["pickerOrder"] != after.order["pickerOrder"] {
        affected.extend(managed.clone());
    }
    let target = if side == Side::Before { before } else { after };
    let target_map = map(target)?;
    let mut providers = current.providers.clone();
    for id in &affected {
        let actual = current_map.get(id);
        if actual != before_map.get(id) && actual != after_map.get(id) {
            return Err(invalid(
                "history_runtime_drift",
                "本次恢复渠道存在后来修改，未用旧快照覆盖；请先明确处理当前配置",
            ));
        }
        match target_map.get(id) {
            Some(row) => {
                providers["providers"]
                    .as_object_mut()
                    .unwrap()
                    .insert(id.clone(), row.clone());
            }
            None => {
                providers["providers"].as_object_mut().unwrap().remove(id);
            }
        }
    }
    if before.providers["defaultProvider"] != after.providers["defaultProvider"] || all {
        if current.providers["defaultProvider"] != before.providers["defaultProvider"]
            && current.providers["defaultProvider"] != after.providers["defaultProvider"]
        {
            return Err(invalid(
                "history_default_drift",
                "默认渠道已被后来操作改变，未自动回退",
            ));
        }
        providers["defaultProvider"] = target.providers["defaultProvider"].clone();
    }
    let default = providers["defaultProvider"].as_str().unwrap_or("");
    if providers["providers"]
        .get(default)
        .is_none_or(|p| p["disabled"] == true)
    {
        return Err(invalid(
            "history_default",
            "恢复后默认渠道不存在或停用，未生成运行计划",
        ));
    }
    let old_slugs = attributed(current, &affected)?;
    let target_slugs = attributed(target, &affected)?;
    // New discoveries under an affected provider may change all/future selection.
    // Require a fresh explicit comparison rather than relying on an old count.
    let historical_rows = before
        .models
        .as_array()
        .into_iter()
        .flatten()
        .chain(after.models.as_array().into_iter().flatten())
        .filter(|r| r["provider"].as_str().is_some_and(|p| affected.contains(p)))
        .filter_map(|r| {
            Some((
                r["provider"].as_str()?.to_owned(),
                r["id"].as_str()?.to_owned(),
            ))
        })
        .collect::<BTreeSet<_>>();
    if current.models.as_array().into_iter().flatten().any(|r| {
        r["provider"].as_str().is_some_and(|p| affected.contains(p))
            && !historical_rows.contains(&(
                r["provider"].as_str().unwrap_or("").into(),
                r["id"].as_str().unwrap_or("").into(),
            ))
    }) {
        return Err(invalid(
            "history_discovery_drift",
            "恢复范围有历史之外的新型号，目录预算须重新明确比较",
        ));
    }
    let mut expected = catalog(current)?;
    expected.retain(|slug| !old_slugs.contains(slug));
    expected.extend(target_slugs.clone());
    if expected.len() > 100 {
        return Err(invalid(
            "catalog_budget",
            "恢复后的完整目录超过当前兼容预算100，草稿保留",
        ));
    }
    let desired = strings(&target.order["pickerOrder"])?;
    let mut order = vec![];
    let mut inserted = false;
    for slug in strings(&current.order["pickerOrder"])? {
        if old_slugs.contains(&slug) {
            if !inserted {
                order.extend(
                    desired
                        .iter()
                        .filter(|id| target_slugs.contains(*id))
                        .cloned(),
                );
                inserted = true;
            }
        } else if expected.contains(&slug) {
            order.push(slug);
        }
    }
    if !inserted {
        order.extend(desired.into_iter().filter(|id| target_slugs.contains(id)));
    }
    let mut seen = BTreeSet::new();
    order.retain(|id| seen.insert(id.clone()));
    let mut all_names = namespaces(current, &affected)?;
    all_names.extend(namespaces(target, &affected)?);
    let belongs = |slug: &String| {
        old_slugs.contains(slug)
            || target_slugs.contains(slug)
            || slug
                .split_once('/')
                .is_some_and(|(prefix, _)| all_names.contains(prefix))
    };
    let target_disabled = if side == Side::Before {
        before_disabled
    } else {
        after_disabled
    };
    let mut disabled = current_disabled
        .iter()
        .filter(|id| !belongs(id))
        .cloned()
        .collect::<Vec<_>>();
    disabled.extend(target_disabled.iter().filter(|id| belongs(id)).cloned());
    let mut seen = BTreeSet::new();
    disabled.retain(|id| seen.insert(id.clone()));
    Ok(Projection {
        providers,
        order,
        catalog: expected,
        disabled,
        affected: affected.into_iter().collect(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn snapshot(
        providers: Value,
        models: Value,
        slugs: Vec<&str>,
        order: Vec<&str>,
    ) -> UpstreamSnapshot {
        UpstreamSnapshot {
            version: "2.84.0".into(),
            providers,
            models,
            order: json!({"pickerOrder":order,"pickerOrderMode":null}),
            catalog: json!({"models":slugs.into_iter().map(|slug|json!({"slug":slug})).collect::<Vec<_>>()}),
        }
    }
    #[test]
    fn restores_all_original_public_fields_without_replacing_a_later_unrelated_provider() {
        let before = snapshot(
            json!({"defaultProvider":"keep","providers":{"removed":{"adapter":"anthropic","baseUrl":"https://old.example.test","models":["model/a"],"modelAliases":{"model/a":"alias"},"apiKeyTransport":"header","note":"original"},"keep":{"note":"keep"}}}),
            json!([{"provider":"removed","id":"model/a","namespaced":"removed/model-a"}]),
            vec!["native", "removed/model-a"],
            vec!["native", "removed/model-a"],
        );
        let after = snapshot(
            json!({"defaultProvider":"keep","providers":{"keep":{"note":"keep"}}}),
            json!([]),
            vec!["native"],
            vec!["native"],
        );
        let mut current = after.clone();
        current.providers["providers"]["later"] = json!({"note":"new unrelated"});
        current.catalog["models"]
            .as_array_mut()
            .unwrap()
            .push(json!({"slug":"later/model"}));
        current.order["pickerOrder"]
            .as_array_mut()
            .unwrap()
            .push(json!("later/model"));
        let result = project(
            &current,
            &before,
            &after,
            [&["later/hidden".into()], &["removed/hidden".into()], &[]],
            Side::Before,
            RecoveryScope {
                managed: &BTreeSet::from(["removed".into()]),
                authorization_changes: &BTreeSet::new(),
                all: false,
            },
        )
        .unwrap();
        assert_eq!(
            result.providers["providers"]["removed"],
            before.providers["providers"]["removed"]
        );
        assert_eq!(
            result.providers["providers"]["later"],
            current.providers["providers"]["later"]
        );
        assert_eq!(
            result.catalog,
            BTreeSet::from([
                "native".into(),
                "removed/model-a".into(),
                "later/model".into()
            ])
        );
        assert_eq!(result.disabled, vec!["later/hidden", "removed/hidden"]);
    }
    #[test]
    fn private_authorization_changes_remain_affected_when_public_configuration_is_identical() {
        let saved = snapshot(
            json!({"defaultProvider":"p","providers":{"p":{"adapter":"openai-chat"},"unrelated":{}}}),
            json!([{"provider":"p","id":"a","namespaced":"p/a"}]),
            vec!["p/a"],
            vec!["p/a"],
        );
        let result = project(
            &saved,
            &saved,
            &saved,
            [&[], &[], &[]],
            Side::Before,
            RecoveryScope {
                managed: &BTreeSet::from(["p".into(), "unrelated".into()]),
                authorization_changes: &BTreeSet::from(["p".into()]),
                all: false,
            },
        )
        .unwrap();
        assert_eq!(result.affected, vec!["p".to_string()]);
        assert_eq!(result.providers, saved.providers);
    }
    #[test]
    fn affected_external_edits_and_unknown_new_discoveries_require_explicit_recomparison() {
        let before = snapshot(
            json!({"defaultProvider":"p","providers":{"p":{"note":"before"}}}),
            json!([{"provider":"p","id":"a","namespaced":"p/a"}]),
            vec!["p/a"],
            vec!["p/a"],
        );
        let mut after = before.clone();
        after.providers["providers"]["p"]["note"] = json!("after");
        let mut current = after.clone();
        current.providers["providers"]["p"]["note"] = json!("later");
        assert_eq!(
            project(
                &current,
                &before,
                &after,
                [&[], &[], &[]],
                Side::Before,
                RecoveryScope {
                    managed: &BTreeSet::from(["p".into()]),
                    authorization_changes: &BTreeSet::new(),
                    all: false,
                }
            )
            .unwrap_err()
            .code,
            "history_runtime_drift"
        );
        current = after.clone();
        current
            .models
            .as_array_mut()
            .unwrap()
            .push(json!({"provider":"p","id":"new","namespaced":"p/new"}));
        assert_eq!(
            project(
                &current,
                &before,
                &after,
                [&[], &[], &[]],
                Side::Before,
                RecoveryScope {
                    managed: &BTreeSet::from(["p".into()]),
                    authorization_changes: &BTreeSet::new(),
                    all: false,
                }
            )
            .unwrap_err()
            .code,
            "history_discovery_drift"
        );
    }
}
