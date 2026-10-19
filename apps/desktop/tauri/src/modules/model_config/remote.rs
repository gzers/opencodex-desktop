//! Shared conditional publication, disjoint from 0.1.x latest.txt containers.
//! Objects are immutable; a strong conditional pointer is the only commit point.
#[cfg(test)]
use super::transfer::{self, ExportScope, IncomingPlan, ModelPackage};
use super::{schema::*, store::sha256};
use crate::infrastructure::webdav_client::{
    WebDavConfig, WebDavError, WebDavMethod, WebDavTransport,
};
#[cfg(test)]
use crate::{infrastructure::atomic_write::atomic_write, modules::backup::safety};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
const POINTER: &str = "models-v1-latest.json";
const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Pointer {
    format: String,
    version: u32,
    operation_id: String,
    digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Baseline {
    pub pointer: Pointer,
    etag: String,
}
#[derive(Clone, Copy)]
pub(super) struct Domain {
    pointer: &'static str,
    prefix: &'static str,
    format: &'static str,
}
#[cfg(test)]
const MODELS: Domain = Domain {
    pointer: POINTER,
    prefix: "models-v1",
    format: "OCXD_MODELS_POINTER",
};
pub(super) const CONFIG: Domain = Domain {
    pointer: "config-v1-latest.json",
    prefix: "config-v1",
    format: "OCXD_CONFIG_POINTER",
};
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UploadPlan {
    pub id: String,
    pub phase: String,
    pub endpoint_binding: String,
    baseline: Option<Baseline>,
    pub package: ModelPackage,
}
fn transport_error(error: WebDavError) -> ModelError {
    invalid(
        "model_sync_transport",
        error.user_message(crate::infrastructure::webdav_client::WebDavOperation::Upload),
    )
}
fn object_path_for(domain: Domain, pointer: &Pointer) -> ModelResult<String> {
    require_uuid(&pointer.operation_id)?;
    let Some(hash) = pointer.digest.strip_prefix("sha256:") else {
        return Err(invalid("remote_digest", "远端清单摘要格式无效"));
    };
    if hash.len() != 64
        || !hash.bytes().all(|b| b.is_ascii_hexdigit())
        || pointer.format != domain.format
        || pointer.version != 1
    {
        return Err(invalid("remote_pointer", "远端配置域版本或摘要无效"));
    }
    Ok(format!("{}-{hash}.ocxd", domain.prefix))
}
#[cfg(test)]
fn plan_path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/model-uploads")
        .join(format!("{id}.json"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
#[cfg(test)]
fn persist(root: &Path, plan: &UploadPlan) -> ModelResult<()> {
    let bytes = serde_json::to_vec(plan)?;
    if bytes.len() > LIMIT {
        return Err(invalid("model_sync_limit", "模型同步包超过64MiB"));
    }
    atomic_write(&plan_path(root, &plan.id)?, &bytes, 0o600)?;
    Ok(())
}
#[cfg(test)]
pub fn read_plan(root: &Path, id: &str, binding: &str) -> ModelResult<UploadPlan> {
    let plan: UploadPlan =
        serde_json::from_slice(&safety::read_file(&plan_path(root, id)?, LIMIT as u64)?)?;
    if plan.id != id || plan.endpoint_binding != binding {
        return Err(invalid(
            "endpoint_changed",
            "活动端点或凭据已变化，原同步计划不能继续",
        ));
    }
    transfer::validate_package(&plan.package)?;
    Ok(plan)
}
#[cfg(test)]
async fn pointer<T: WebDavTransport>(
    transport: &T,
    config: &WebDavConfig,
) -> ModelResult<Option<Baseline>> {
    pointer_for(MODELS, transport, config).await
}
pub(super) async fn pointer_for<T: WebDavTransport>(
    domain: Domain,
    transport: &T,
    config: &WebDavConfig,
) -> ModelResult<Option<Baseline>> {
    match transport
        .request(config, WebDavMethod::Get, domain.pointer, None)
        .await
    {
        Err(WebDavError::NotFound) => Ok(None),
        Err(error) => Err(transport_error(error)),
        Ok(response) => {
            if response.body.len() > 4096 {
                return Err(invalid("remote_pointer", "远端配置指针超过上限"));
            }
            let pointer: Pointer = serde_json::from_slice(&response.body)
                .map_err(|_| invalid("remote_pointer", "远端配置指针无效"))?;
            object_path_for(domain, &pointer)?;
            let etag = response
                .etag
                .filter(|s| {
                    s.starts_with('"') && s.ends_with('"') && s.len() >= 2 && s.len() <= 1024
                })
                .ok_or_else(|| {
                    invalid("etag_required", "该WebDAV未提供强ETag，不能安全发布配置")
                })?;
            Ok(Some(Baseline { pointer, etag }))
        }
    }
}
#[cfg(test)]
async fn fetch<T: WebDavTransport>(
    transport: &T,
    config: &WebDavConfig,
    pointer: &Pointer,
) -> ModelResult<ModelPackage> {
    let bytes = fetch_for(MODELS, transport, config, pointer).await?;
    let package: ModelPackage = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("remote_package", "远端配置包无法解析"))?;
    transfer::validate_package(&package)?;
    Ok(package)
}
pub(super) async fn fetch_for<T: WebDavTransport>(
    domain: Domain,
    transport: &T,
    config: &WebDavConfig,
    pointer: &Pointer,
) -> ModelResult<Vec<u8>> {
    let response = transport
        .request(
            config,
            WebDavMethod::Get,
            &object_path_for(domain, pointer)?,
            None,
        )
        .await
        .map_err(transport_error)?;
    if response.body.len() > LIMIT || sha256(&response.body) != pointer.digest {
        return Err(invalid(
            "remote_digest",
            "远端配置包不完整或摘要不符，未采用",
        ));
    }
    Ok(response.body)
}
#[cfg(test)]
pub async fn prepare<T: WebDavTransport>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    binding: &str,
    scope: ExportScope,
) -> ModelResult<UploadPlan> {
    config.validate().map_err(transport_error)?;
    let local = transfer::export(root, scope)?;
    let baseline = pointer(transport, config).await?;
    let package = match &baseline {
        Some(base) => {
            transfer::merge_packages(fetch(transport, config, &base.pointer).await?, local)?
        }
        None => local,
    };
    let plan = UploadPlan {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "prepared".into(),
        endpoint_binding: binding.into(),
        baseline,
        package,
    };
    persist(root, &plan)?;
    Ok(plan)
}
/// A retry first resolves the original operation. It never creates a new ID or
/// writes against a refreshed remote baseline behind the user's confirmation.
#[cfg(test)]
pub async fn publish<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    id: &str,
    binding: &str,
    unchanged: F,
) -> ModelResult<UploadPlan> {
    let mut plan = read_plan(root, id, binding)?;
    let lock_path = plan_path(root, id)?;
    let _lock = crate::infrastructure::locking::TargetFileLock::lock(&lock_path)?;
    let baseline = plan.baseline.clone();
    let published = plan.phase == "published";
    let body = serde_json::to_vec(&plan.package)?;
    publish_blob(
        MODELS,
        transport,
        config,
        id,
        baseline.as_ref(),
        body,
        published,
        |phase| {
            plan.phase = phase.into();
            persist(root, &plan)
        },
        unchanged,
    )
    .await?;
    Ok(plan)
}
#[allow(clippy::too_many_arguments)]
pub(super) async fn publish_blob<
    T: WebDavTransport,
    F: Fn() -> ModelResult<()>,
    S: FnMut(&str) -> ModelResult<()>,
>(
    domain: Domain,
    transport: &T,
    config: &WebDavConfig,
    id: &str,
    baseline: Option<&Baseline>,
    body: Vec<u8>,
    published: bool,
    mut stage: S,
    unchanged: F,
) -> ModelResult<()> {
    require_uuid(id)?;
    config.validate().map_err(transport_error)?;
    if body.len() > LIMIT {
        return Err(invalid("remote_limit", "配置载荷超过64MiB"));
    }
    let next = Pointer {
        format: domain.format.into(),
        version: 1,
        operation_id: id.into(),
        digest: sha256(&body),
    };
    let current = pointer_for(domain, transport, config).await?;
    if current.as_ref().is_some_and(|head| head.pointer == next) {
        if fetch_for(domain, transport, config, &next).await? != body {
            return Err(invalid("remote_readback", "已发布载荷不同，未宣称成功"));
        }
        unchanged()?;
        stage("published")?;
        return Ok(());
    }
    if published {
        return Err(invalid(
            "remote_advanced",
            "该操作曾发布，远端已有后续版本；未重复覆盖",
        ));
    }
    if serde_json::to_value(&current)? != serde_json::to_value(baseline)? {
        return Err(invalid(
            "remote_changed",
            "远端配置已变化，请重新比较；原计划未覆盖远端",
        ));
    }
    unchanged()?;
    let path = object_path_for(domain, &next)?;
    stage("uploading")?;
    let response = transport
        .put_conditional(config, &path, body.clone(), None)
        .await
        .map_err(transport_error)?;
    if response.status != 412 && ![200, 201, 204].contains(&response.status) {
        return Err(invalid("remote_write", "配置载荷上传未确认"));
    }
    let stored = transport
        .request(config, WebDavMethod::Get, &path, None)
        .await
        .map_err(transport_error)?;
    if stored.body != body {
        return Err(invalid("remote_readback", "配置载荷读回不一致，未发布清单"));
    }
    unchanged()?;
    stage("publishing")?;
    let response = transport
        .put_conditional(
            config,
            domain.pointer,
            serde_json::to_vec(&next)?,
            baseline.map(|head| head.etag.as_str()),
        )
        .await
        .map_err(transport_error)?;
    if response.status == 412 {
        return Err(invalid(
            "remote_changed",
            "发布时远端已被另一设备更新，未覆盖",
        ));
    }
    if ![200, 201, 204].contains(&response.status) {
        return Err(invalid("remote_write", "清单发布结果待核对"));
    }
    if !pointer_for(domain, transport, config)
        .await?
        .as_ref()
        .is_some_and(|head| head.pointer == next)
    {
        return Err(invalid(
            "remote_changed",
            "清单发布后已有后续变化，未宣称成功",
        ));
    }
    unchanged()?;
    stage("published")?;
    Ok(())
}
#[cfg(test)]
pub async fn download<T: WebDavTransport, F: Fn() -> ModelResult<()>>(
    root: &Path,
    transport: &T,
    config: &WebDavConfig,
    unchanged: F,
) -> ModelResult<IncomingPlan> {
    config.validate().map_err(transport_error)?;
    let base = pointer(transport, config).await?.ok_or_else(|| {
        invalid(
            "remote_empty",
            "该端点还没有模型配置；旧管理器配置域保持独立",
        )
    })?;
    let package = fetch(transport, config, &base.pointer).await?;
    if pointer(transport, config)
        .await?
        .as_ref()
        .map(|b| &b.pointer)
        != Some(&base.pointer)
    {
        return Err(invalid(
            "remote_changed",
            "下载期间远端模型清单改变，请重新下载",
        ));
    }
    unchanged()?;
    transfer::receive(root, package)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::infrastructure::webdav_client::WebDavResponse;
    use std::{collections::BTreeMap, sync::Mutex};
    #[derive(Default)]
    pub(crate) struct Memory {
        state: Mutex<State>,
    }
    impl Memory {
        pub fn lose_reply(&self) {
            self.state.lock().unwrap().drop_pointer_reply = true;
        }
        pub fn writes(&self) -> Vec<String> {
            self.state.lock().unwrap().writes.clone()
        }
    }
    #[derive(Default)]
    struct State {
        files: BTreeMap<String, (Vec<u8>, String)>,
        writes: Vec<String>,
        generation: usize,
        drop_pointer_reply: bool,
        race: Option<Vec<u8>>,
        corrupt_object: bool,
    }
    #[async_trait::async_trait]
    impl WebDavTransport for Memory {
        async fn request(
            &self,
            _: &WebDavConfig,
            method: WebDavMethod,
            path: &str,
            _: Option<Vec<u8>>,
        ) -> Result<WebDavResponse, WebDavError> {
            assert_eq!(
                method,
                WebDavMethod::Get,
                "this domain must never use unconditional writes"
            );
            let state = self.state.lock().unwrap();
            let (body, etag) = state.files.get(path).ok_or(WebDavError::NotFound)?;
            Ok(WebDavResponse {
                status: 200,
                body: body.clone(),
                etag: Some(etag.clone()),
            })
        }
        async fn put_conditional(
            &self,
            _: &WebDavConfig,
            path: &str,
            mut body: Vec<u8>,
            expected: Option<&str>,
        ) -> Result<WebDavResponse, WebDavError> {
            let mut state = self.state.lock().unwrap();
            let pointer = path.ends_with("-latest.json");
            if pointer {
                if let Some(other) = state.race.take() {
                    state.files.insert(path.into(), (other, "\"raced\"".into()));
                }
            }
            let existing = state.files.get(path);
            if existing.map(|(_, etag)| etag.as_str()) != expected {
                return Ok(WebDavResponse {
                    status: 412,
                    body: vec![],
                    etag: None,
                });
            }
            state.generation += 1;
            let etag = format!("\"{}\"", state.generation);
            state.writes.push(path.into());
            if state.corrupt_object && !pointer {
                body = b"truncated".to_vec();
            }
            state.files.insert(path.into(), (body, etag.clone()));
            if pointer && state.drop_pointer_reply {
                state.drop_pointer_reply = false;
                return Err(WebDavError::Network);
            }
            Ok(WebDavResponse {
                status: 201,
                body: vec![],
                etag: Some(etag),
            })
        }
    }
    fn config() -> WebDavConfig {
        WebDavConfig {
            base_url: "https://fixture.invalid".into(),
            remote_path: "sync".into(),
            username: "fixture".into(),
            password: "not-a-real-secret".into(),
        }
    }
    fn scope() -> ExportScope {
        ExportScope {
            channel_models: Default::default(),
            include_policy: false,
            channel_ids: vec![],
            template_refs: vec![],
            include_global: false,
        }
    }
    fn add(root: &Path, label: &str) -> super::super::schema::Commit {
        let store = super::super::ModelStore::new(root);
        let snapshot = store.load().unwrap();
        let mut content = TemplateContent::default();
        content.matching.include.push(Pattern {
            kind: MatchKind::Prefix,
            value: label.into(),
        });
        store
            .save(SaveTemplate {
                expected_library: snapshot.library_digest,
                template_id: None,
                base_revision: None,
                label: label.into(),
                content,
            })
            .unwrap()
    }
    #[tokio::test]
    async fn partial_upload_preserves_unrelated_remote_objects_and_download_is_review_only() {
        let remote = Memory::default();
        remote.state.lock().unwrap().files.insert(
            "latest.txt".into(),
            (b"old-domain".to_vec(), "\"old\"".into()),
        );
        let first = tempfile::tempdir().unwrap();
        let a = add(first.path(), "remote-only");
        let plan = prepare(first.path(), &remote, &config(), "binding", scope())
            .await
            .unwrap();
        publish(
            first.path(),
            &remote,
            &config(),
            &plan.id,
            "binding",
            || Ok(()),
        )
        .await
        .unwrap();
        let second = tempfile::tempdir().unwrap();
        let b = add(second.path(), "selected-local");
        let mut selected = scope();
        selected.template_refs.push(TemplateRef {
            template_id: b.template_id.clone(),
            revision_id: b.revision_id,
        });
        let plan = prepare(second.path(), &remote, &config(), "binding", selected)
            .await
            .unwrap();
        assert!(plan
            .package
            .payload
            .templates
            .iter()
            .any(|t| t.id == a.template_id));
        publish(
            second.path(),
            &remote,
            &config(),
            &plan.id,
            "binding",
            || Ok(()),
        )
        .await
        .unwrap();
        let third = tempfile::tempdir().unwrap();
        let before = super::super::ModelStore::new(third.path()).load().unwrap();
        let incoming = download(third.path(), &remote, &config(), || Ok(()))
            .await
            .unwrap();
        assert_eq!(
            before,
            super::super::ModelStore::new(third.path()).load().unwrap()
        );
        let accepted = transfer::accept(third.path(), &incoming.id).unwrap();
        assert!(accepted
            .library
            .templates
            .iter()
            .any(|t| t.id == a.template_id));
        assert!(accepted
            .library
            .templates
            .iter()
            .any(|t| t.id == b.template_id));
        assert_eq!(accepted.policy, before.policy);
        assert_eq!(
            remote.state.lock().unwrap().files["latest.txt"].0,
            b"old-domain"
        );
    }
    #[tokio::test]
    async fn lost_commit_response_resolves_same_operation_after_reload_without_a_second_publication(
    ) {
        let root = tempfile::tempdir().unwrap();
        let remote = Memory::default();
        let plan = prepare(root.path(), &remote, &config(), "binding", scope())
            .await
            .unwrap();
        remote.state.lock().unwrap().drop_pointer_reply = true;
        assert!(
            publish(root.path(), &remote, &config(), &plan.id, "binding", || Ok(
                ()
            ))
            .await
            .is_err()
        );
        assert_eq!(
            read_plan(root.path(), &plan.id, "binding").unwrap().phase,
            "publishing"
        );
        assert_eq!(
            publish(root.path(), &remote, &config(), &plan.id, "binding", || Ok(
                ()
            ))
            .await
            .unwrap()
            .phase,
            "published"
        );
        assert_eq!(
            remote
                .state
                .lock()
                .unwrap()
                .writes
                .iter()
                .filter(|p| p.as_str() == POINTER)
                .count(),
            1
        );
    }
    #[tokio::test]
    async fn concurrent_publish_and_weak_etags_never_silently_overwrite() {
        let root = tempfile::tempdir().unwrap();
        let remote = Memory::default();
        let a = prepare(root.path(), &remote, &config(), "binding", scope())
            .await
            .unwrap();
        let other = Pointer {
            format: "OCXD_MODELS_POINTER".into(),
            version: 1,
            operation_id: uuid::Uuid::new_v4().to_string(),
            digest: sha256(b"other"),
        };
        remote.state.lock().unwrap().race = Some(serde_json::to_vec(&other).unwrap());
        assert_eq!(
            publish(root.path(), &remote, &config(), &a.id, "binding", || Ok(()))
                .await
                .unwrap_err()
                .code,
            "remote_changed"
        );
        {
            let mut state = remote.state.lock().unwrap();
            assert_eq!(
                serde_json::from_slice::<Pointer>(&state.files[POINTER].0).unwrap(),
                other
            );
            state.files.get_mut(POINTER).unwrap().1 = "W/\"weak\"".into();
        }
        assert_eq!(
            prepare(root.path(), &remote, &config(), "binding", scope())
                .await
                .unwrap_err()
                .code,
            "etag_required"
        );
    }
    #[tokio::test]
    async fn incomplete_payload_and_endpoint_changes_cannot_publish_pointer() {
        let root = tempfile::tempdir().unwrap();
        let remote = Memory::default();
        let plan = prepare(root.path(), &remote, &config(), "binding", scope())
            .await
            .unwrap();
        assert!(
            publish(root.path(), &remote, &config(), &plan.id, "changed", || Ok(
                ()
            ))
            .await
            .is_err()
        );
        assert!(publish(
            root.path(),
            &remote,
            &config(),
            &plan.id,
            "binding",
            || Err(invalid("changed", "端点改变"))
        )
        .await
        .is_err());
        assert!(remote.state.lock().unwrap().writes.is_empty());
        remote.state.lock().unwrap().corrupt_object = true;
        assert_eq!(
            publish(root.path(), &remote, &config(), &plan.id, "binding", || Ok(
                ()
            ))
            .await
            .unwrap_err()
            .code,
            "remote_readback"
        );
        assert!(!remote.state.lock().unwrap().files.contains_key(POINTER));
    }
}
