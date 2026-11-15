//! Ordered locator data only; native retained history must be revalidated by
//! capture/history. Neither serde nor these digests recreate closed authority.
use super::*;
pub(super) const MAX_HISTORY: usize = 512;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ClosedReference {
    pub(super) session_id: String,
    pub(super) directory: Identity,
    pub(super) current: Identity,
    pub(super) closed_sequence: u32,
    pub(super) phase_digest: String,
    pub(super) manifest_digest: String,
    pub(super) current_digest: String,
    pub(super) binding_digest: String,
    pub(super) security_policy_digest: String,
}
pub(super) fn check_predecessors(
    refs: &[ClosedReference],
    new_session: &str,
    binding: &Binding,
    policy: &str,
) -> io::Result<()> {
    if refs.len() > MAX_HISTORY {
        return Err(invalid());
    }
    let binding_digest = hash::sha256_hex(&serde_json::to_vec(binding).map_err(|_| invalid())?);
    let mut sessions = BTreeSet::new();
    let mut objects = BTreeSet::new();
    let (data_volume, data_id) = binding.data_identity();
    objects.insert((data_volume, data_id));
    for item in refs {
        if !session(&item.session_id)
            || item.session_id == new_session
            || !sessions.insert(&item.session_id)
            || !item.directory.valid()
            || !item.current.valid()
            || item.directory.volume != item.current.volume
            || item.directory.volume != data_volume
            || !objects.insert((item.directory.volume, &item.directory.file_id))
            || !objects.insert((item.current.volume, &item.current.file_id))
            || !(2..=512).contains(&item.closed_sequence)
            || item.binding_digest != binding_digest
            || item.security_policy_digest != policy
            || [
                &item.phase_digest,
                &item.manifest_digest,
                &item.current_digest,
            ]
            .iter()
            .any(|s| !digest(s))
        {
            return Err(invalid());
        }
    }
    Ok(())
}
