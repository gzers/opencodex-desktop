//! Adopt the saved provider roster through the official 2.84 editor owner.
//! The caller owns the explicit restart and durable pending-rename transaction.
//! This module only reads config.json; credentials and unknown disk fields never
//! enter the request. A failed/uncertain adoption must leave pending work intact.
use crate::{
    modules::{
        backup::safety,
        model_config::{management::Management, provider_fields, schema::*},
    },
    state::ProcessContext,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use zeroize::Zeroizing;

const MAX_CONFIG: u64 = 16 * 1024 * 1024;
// safeConfigDTO in official src/server/auth-cors.ts adds precisely these fields
// to providerEditorConfigDTO. The latter owns baseUrl/cost/note/account-mode
// projections; recreating those from raw disk would produce a stale baseline.
const PUBLIC_DERIVED: &[&str] = &[
    "hasApiKey",
    "hasHeaders",
    "initialModelSelection",
    "xaiResponsesOptInState",
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedConfig {
    default_provider: String,
    providers: BTreeMap<String, SavedProvider>,
}

// Ignore provider values other than default eligibility, rather than loading
// apiKey/headers/unknown fields into a secret-bearing serde_json::Value.
#[derive(Deserialize)]
struct SavedProvider {
    #[serde(default)]
    disabled: bool,
}

struct SavedDisk {
    config: SavedConfig,
    digest: [u8; 32],
}

fn file_error() -> ModelError {
    invalid(
        "saved_runtime_file",
        "无法核对已保存的渠道配置；运行态同步未完成，请稍后重试",
    )
}

fn config_bytes(context: &ProcessContext) -> ModelResult<Zeroizing<Vec<u8>>> {
    let path = context.opencodex_home.join("config.json");
    safety::check_path(&context.opencodex_home, &path, false).map_err(|_| file_error())?;
    safety::read_file(&path, MAX_CONFIG)
        .map(Zeroizing::new)
        .map_err(|_| file_error())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name.as_bytes()[name.len() - 1].is_ascii_alphanumeric()
        && !["__proto__", "constructor", "prototype"].contains(&name)
}

impl SavedDisk {
    fn read(context: &ProcessContext) -> ModelResult<Self> {
        let bytes = config_bytes(context)?;
        let config: SavedConfig = serde_json::from_slice(&bytes).map_err(|_| file_error())?;
        if config.providers.is_empty()
            || config.providers.keys().any(|name| !valid_name(name))
            || config
                .providers
                .get(&config.default_provider)
                .is_none_or(|row| row.disabled)
        {
            return Err(file_error());
        }
        Ok(Self {
            config,
            digest: Sha256::digest(bytes.as_slice()).into(),
        })
    }

    fn unchanged(&self, context: &ProcessContext) -> ModelResult<()> {
        let bytes = config_bytes(context)?;
        let digest: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
        if digest != self.digest {
            return Err(invalid(
                "saved_runtime_drift",
                "同步期间配置文件已变化；未确认运行态生效，请保留待处理改名并重试",
            ));
        }
        Ok(())
    }

    fn validate_renames(&self, renames: &BTreeMap<String, String>) -> ModelResult<()> {
        for (from, to) in renames {
            if !valid_name(from)
                || !valid_name(to)
                || from == to
                || self.config.providers.contains_key(from)
                || !self.config.providers.contains_key(to)
            {
                return Err(invalid(
                    "saved_runtime_rename",
                    "待处理改名与当前保存的渠道不一致；未执行运行态同步，请重新核对后重试",
                ));
            }
        }
        Ok(())
    }
}

fn dto_error() -> ModelError {
    invalid(
        "saved_runtime_dto",
        "官方公开渠道 DTO 含未支持或不可编辑字段；未提交运行态同步，请更新适配后重试",
    )
}

fn editor_row(value: &Value) -> ModelResult<Value> {
    let mut row = value.as_object().cloned().ok_or_else(dto_error)?;
    for field in PUBLIC_DERIVED {
        row.remove(*field);
    }
    // Reuse the frozen 2.84 editor policy. Secret, runtime-only and new fields
    // fail closed; they are never silently sent back as editor write authority.
    let row = Value::Object(row);
    provider_fields::validate_public(&row).map_err(|_| dto_error())?;
    if row["adapter"].as_str().is_none_or(str::is_empty)
        || row["baseUrl"].as_str().is_none_or(str::is_empty)
    {
        return Err(dto_error());
    }
    Ok(row)
}

fn saved_editor(public: &Value, saved: &SavedDisk) -> ModelResult<Value> {
    let rows = public
        .get("providers")
        .and_then(Value::as_object)
        .ok_or_else(dto_error)?;
    let mut providers = Map::new();
    for name in saved.config.providers.keys() {
        let row = rows.get(name).ok_or_else(|| {
            invalid(
                "saved_runtime_baseline",
                "官方运行态缺少已保存的渠道，无法组成完整基线；请保留待处理改名并重试",
            )
        })?;
        providers.insert(name.clone(), editor_row(row)?);
    }
    Ok(json!({"defaultProvider":saved.config.default_provider,"providers":providers}))
}

fn owner_error(error: ModelError) -> ModelError {
    // Management deliberately suppresses upstream bodies. Only its fixed HTTP
    // status message is inspected; no server-controlled error is returned.
    let (code, message) = match error.code.as_str() {
        "management_rejected"
            if ["HTTP404", "HTTP405", "HTTP501"]
                .iter()
                .any(|status| error.message.contains(status)) =>
        {
            (
                "saved_runtime_unsupported",
                "当前官方运行实例不支持所需渠道同步操作；未完成同步，请保留待处理改名并重试",
            )
        }
        "management_rejected" if error.message.contains("HTTP409") => (
            "saved_runtime_baseline",
            "官方拒绝了已变化的渠道基线或同步前置条件；请保留待处理改名并重试",
        ),
        "stale_runtime" => (
            "stale_runtime",
            "官方运行实例已变化；未确认运行态同步，请稍后重试",
        ),
        _ => (
            "saved_runtime_owner",
            "官方渠道同步未完整成功，运行态结果未确认；请保留待处理改名并重试",
        ),
    };
    invalid(code, message)
}

fn require_catalog_commit(response: &Value) -> ModelResult<()> {
    // Official CatalogDisposition's committed variant, matching
    // codex/catalog-refresh-status.ts::normalizeCatalogDisposition. Mutation
    // success alone also accompanies skipped/failed catalog production.
    let catalog = &response["catalogRefresh"];
    let notices_valid = catalog["notices"].as_array().is_some_and(|notices| {
        notices.len() <= 3
            && notices.iter().all(|notice| {
                matches!(
                    notice.as_str(),
                    Some("provider-auth" | "provider-network" | "fallback")
                )
            })
    });
    if catalog["status"].as_str() != Some("committed")
        || !catalog["changed"].is_boolean()
        || !catalog["degraded"].is_boolean()
        || !notices_valid
    {
        return Err(invalid(
            "saved_runtime_catalog",
            "官方未确认模型目录已提交；未进入重启，请保留待处理改名并重试",
        ));
    }
    Ok(())
}

async fn adopt(
    management: &Management,
    context: &ProcessContext,
    saved: &SavedDisk,
) -> ModelResult<()> {
    // Every saved row needs a fresh official editor projection: a previously
    // added OR modified sibling may otherwise leave the full baseline stale.
    // Canonical rows pass through the same official transport validation.
    for provider in saved.config.providers.keys() {
        saved.unchanged(context)?;
        management
            .reload_provider(provider)
            .await
            .map_err(owner_error)?;
        saved.unchanged(context)?;
    }
    let public = management.public_config().await.map_err(owner_error)?;
    saved.unchanged(context)?;
    let editor = saved_editor(&public, saved)?;
    let response = management
        .adopt_saved_provider_editor(editor.clone())
        .await
        .map_err(owner_error)?;
    saved.unchanged(context)?;
    if response.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(invalid(
            "saved_runtime_response",
            "官方未确认渠道同步成功；请保留待处理改名并重试",
        ));
    }
    require_catalog_commit(&response)?;
    let live = management.public_config().await.map_err(owner_error)?;
    let rows = live
        .get("providers")
        .and_then(Value::as_object)
        .ok_or_else(dto_error)?;
    if rows.keys().collect::<BTreeSet<_>>()
        != saved.config.providers.keys().collect::<BTreeSet<_>>()
        || live.get("defaultProvider").and_then(Value::as_str)
            != Some(saved.config.default_provider.as_str())
        || saved_editor(&live, saved)? != editor
    {
        return Err(invalid(
            "saved_runtime_readback",
            "官方运行态渠道集合、默认项或配置回读不一致；请保留待处理改名并重试",
        ));
    }
    Ok(())
}

/// Called only during preparation of the already confirmed restart action.
/// Success confirms a committed catalog, the full live roster/default and
/// identical config bytes, including saves made without a pending rename.
/// This does not restart either process, write files, retry mutations, or clear
/// pending renames. The caller must clear pending only after its whole action.
pub async fn reconcile(
    management: &Management,
    context: &ProcessContext,
    renames: &BTreeMap<String, String>,
) -> ModelResult<()> {
    management.verify_binding().map_err(owner_error)?;
    let saved = SavedDisk::read(context)?;
    saved.validate_renames(renames)?;
    let result = adopt(management, context, &saved).await;
    // Check even a rejected/timed-out owner: it may already have had effects.
    saved.unchanged(context)?;
    result
}
