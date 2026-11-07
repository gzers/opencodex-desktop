//! Read-only routing to an existing backend-owned result. No result payload,
//! endpoint, plan, file/key creation, recovery or write authority is issued here.
use super::{
    file_transfer_v2::FileTransferOwnerV2,
    schema::{invalid, ModelResult},
    sync_local_v2::restore,
    sync_service_v2::{StoredResultOwnerV2, SyncServiceV2},
    sync_v2::types::require_uuid,
};
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultOwnerKindV2 {
    File,
    Sync,
    Recovery,
    ScopeRestore,
}
/// Display/routing facts only. UI must still call the owner's existing read-only
/// receipt getter; this DTO cannot be deserialized into operation authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultOwnerV2 {
    pub operation_id: String,
    pub kind: ResultOwnerKindV2,
}

/// D/H and both owners come from Main's managed state, never UI-supplied paths.
/// None means every actual owner positively reported absence. An unreadable,
/// malformed or ambiguous store never becomes a different domain or NotFound.
/// No startup hook: unavailable result resources cannot block app construction.
pub(crate) fn resolve(
    desktop: &Path,
    home: &Path,
    operation_id: &str,
    file: &mut FileTransferOwnerV2,
    service: &SyncServiceV2,
) -> ModelResult<Option<ResultOwnerV2>> {
    require_uuid(operation_id).map_err(|_| invalid("result_operation_id", "原操作ID无效"))?;
    // Compare existing fixed contexts without resolving any user-selected path.
    // Alias-equivalent roots are allowed; an unrelated managed owner is not.
    let fixed = desktop.canonicalize()?;
    if file.desktop_root().canonicalize()? != fixed
        || service.desktop_root().canonicalize()? != fixed
    {
        return Err(invalid("result_owner_context", "原操作归属上下文不一致"));
    }
    let fixed_home = home.canonicalize()?;
    let service_home = service
        .upstream_home()
        .ok_or_else(|| invalid("result_owner_context", "原操作归属上下文不一致"))?;
    if file.upstream_home().canonicalize()? != fixed_home
        || service_home.canonicalize()? != fixed_home
    {
        return Err(invalid("result_owner_context", "原操作归属上下文不一致"));
    }
    let mut owner = None;
    if file.owns_record(operation_id)? {
        owner = Some(ResultOwnerKindV2::File);
    }
    if let Some(stored) = service.inspect_result_owner(operation_id)? {
        let kind = match stored {
            StoredResultOwnerV2::Sync => ResultOwnerKindV2::Sync,
            StoredResultOwnerV2::Recovery => ResultOwnerKindV2::Recovery,
        };
        if owner.replace(kind).is_some() {
            return Err(ambiguous());
        }
    }
    // The existing bounded scanner checks actual plan/source/progress schema
    // and receipt integrity. Even a corrupt unrelated restore is propagated;
    // it must not be silently skipped to pretend the queried ID is absent.
    for operation in restore::operations(desktop, home, None)? {
        if operation.restore_id == operation_id
            && owner.replace(ResultOwnerKindV2::ScopeRestore).is_some()
        {
            return Err(ambiguous());
        }
    }
    Ok(owner.map(|kind| ResultOwnerV2 {
        operation_id: operation_id.into(),
        kind,
    }))
}
fn ambiguous() -> super::schema::ModelError {
    invalid(
        "result_owner_ambiguous",
        "同一ID出现多个操作来源，请检查原记录",
    )
}

#[cfg(test)]
#[path = "result_owner_v2_tests.rs"]
mod tests;
