//! Read-only owner-issued proof. A serialized service context can reconstruct a
//! receiver, but only its original checked plan AND real applied receipt create
//! this capability. Nothing in this module grants another model write.
use super::*;

pub struct AppliedMappingReceiptProofV2 {
    operation_id: String,
    handle: ModelApplyPlanHandle,
    root_revision: String,
    selection_hash: String,
    bindings_hash: String,
    expected_target: String,
    before: Option<Vec<u8>>,
    after: Vec<u8>,
}
impl AppliedMappingReceiptProofV2 {
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }
    pub fn handle(&self) -> &ModelApplyPlanHandle {
        &self.handle
    }
    pub fn root_revision(&self) -> &str {
        &self.root_revision
    }
    pub fn selection_hash(&self) -> &str {
        &self.selection_hash
    }
    pub fn bindings_hash(&self) -> &str {
        &self.bindings_hash
    }
    pub fn expected_target(&self) -> &str {
        &self.expected_target
    }
    pub fn mapping_before(&self) -> Option<&[u8]> {
        self.before.as_deref()
    }
    pub fn mapping_after(&self) -> &[u8] {
        &self.after
    }
}
impl ReceiveFileHostV2 {
    /// Unlike status's fast Applied path, this reloads the original manifest and
    /// validates the exact receiver bindings and all returned mapping images.
    /// It deliberately does not compare today's config or recapture old Absent.
    pub fn applied_mapping_proof(
        &self,
        id: &str,
        handle: &ModelApplyPlanHandle,
    ) -> ModelResult<AppliedMappingReceiptProofV2> {
        (|| -> Result<_> {
            let frozen = self.load(id, handle)?;
            if !self.applied(id, handle)? {
                return Err(fail("model_receive_not_applied"));
            }
            let (index, entry) = frozen.entries.iter().enumerate()
                .find(|(_, e)| matches!(&e.target, TargetV2::Manager(path) if path == SYNC_IDENTITIES))
                .ok_or_else(|| fail("model_receive_mapping_proof_missing"))?;
            let before = entry.before.as_ref().map(|image| self.image(id, index, true, image).map(|b| b.to_vec())).transpose()?;
            let after = self.image(id, index, false, &entry.after)?.to_vec();
            mapping_document(&after)?;
            if let Some(bytes) = &before { mapping_document(bytes)?; }
            Ok(AppliedMappingReceiptProofV2 {
                operation_id: id.into(), handle: handle.clone(), root_revision: frozen.root_revision,
                selection_hash: frozen.selection_hash, bindings_hash: frozen.bindings_hash,
                expected_target: frozen.expected_target, before, after,
            })
        })().map_err(model_error)
    }
}
