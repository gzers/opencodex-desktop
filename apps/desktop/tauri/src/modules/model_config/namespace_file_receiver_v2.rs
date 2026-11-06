//! Explicit LocalFile namespace authority over the same receive transaction.
//! This adapter never accepts a replacement Policy or caller-provided prefixes.
use super::*;
use crate::modules::model_config::file_transfer_v2::NamespaceReceiveV2;
use std::sync::Arc;

pub struct SystemNamespaceReceiveV2;

/// Construction is host authority, not an IPC DTO. The normal receiver's public
/// fields/literals remain unchanged, and every transaction method is shared.
pub struct NamespaceFileHostV2 {
    host: ReceiveFileHostV2,
    adopt_ids: Vec<String>,
}
impl ReceiveFileHostV2 {
    pub fn with_namespace_adoption(
        self,
        mut adopt_ids: Vec<String>,
    ) -> ModelResult<NamespaceFileHostV2> {
        // LocalFile adoption never grants the remote identity-mapping capability.
        if !adopt_ids.is_empty() && self.trusted_mapping_write.is_some() {
            return Err(model_error(fail(
                "model_receive_namespace_mapping_authority",
            )));
        }
        for id in &adopt_ids {
            check_id(id).map_err(model_error)?;
            if self
                .bindings
                .iter()
                .filter(|b| &b.portable_channel_id == id)
                .count()
                != 1
            {
                return Err(model_error(fail("model_receive_scope_invalid")));
            }
        }
        adopt_ids.sort();
        if adopt_ids.windows(2).any(|ids| ids[0] == ids[1]) {
            return Err(model_error(fail("model_receive_scope_invalid")));
        }
        Ok(NamespaceFileHostV2 {
            host: self,
            adopt_ids,
        })
    }
}
impl NamespaceReceiveV2 for SystemNamespaceReceiveV2 {
    fn plan(
        &self,
        library: &LibraryV2,
        policy: &PolicyV2,
        objects: &SavedObjects,
        scope: &SelectionV2,
        bindings: &[ReceiveBindingV2],
        adopt_ids: &[String],
    ) -> ModelResult<receive_models_v2::ReceiveModelsPlanV2> {
        local_file_scope(scope)?;
        receive_models_v2::plan_with_namespace_adoption(
            library,
            policy,
            objects.models(),
            scope,
            bindings,
            adopt_ids,
        )
        .map_err(model_error)
    }
    fn consumer(
        &self,
        d: &Path,
        h: &Path,
        bindings: Vec<ReceiveBindingV2>,
        adopt_ids: &[String],
    ) -> ModelResult<Arc<dyn ModelApplyV2>> {
        Ok(Arc::new(
            ReceiveFileHostV2 {
                root: d.to_owned(),
                home: h.to_owned(),
                bindings,
                trusted_mapping_write: None,
            }
            .with_namespace_adoption(adopt_ids.to_vec())?,
        ))
    }
}
impl ModelApplyV2 for NamespaceFileHostV2 {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        scope: &SelectionV2,
        expected: &str,
    ) -> ModelResult<ModelApplyPlanHandle> {
        local_file_scope(scope)?;
        self.host
            .prepare_inner(id, objects, scope, expected, &self.adopt_ids)
            .map_err(model_error)
    }
    fn protect(&self, id: &str, h: &ModelApplyPlanHandle, backup: bool) -> ModelResult<()> {
        self.host.protect_authorized(id, h, backup, &self.adopt_ids)
    }
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        h: &ModelApplyPlanHandle,
        backup: bool,
    ) -> ModelResult<ModelApplyReceipt> {
        self.host
            .apply_authorized(id, objects, h, backup, &self.adopt_ids)
    }
    fn status(&self, id: &str, h: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus> {
        self.host.status_authorized(id, h, &self.adopt_ids)
    }
}

fn local_file_scope(scope: &SelectionV2) -> ModelResult<()> {
    if scope.usage != crate::modules::model_config::objects_overlay_v2::UsageV2::LocalFile {
        return Err(model_error(fail(
            "model_receive_namespace_local_file_required",
        )));
    }
    Ok(())
}
