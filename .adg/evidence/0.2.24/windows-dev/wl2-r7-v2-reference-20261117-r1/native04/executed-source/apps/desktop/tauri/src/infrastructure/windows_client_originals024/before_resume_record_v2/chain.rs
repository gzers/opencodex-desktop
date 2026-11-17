//! Strict V2 byte ordering DATA only, no conversion through any old decoder.
use super::*;
pub(in crate::infrastructure::windows_client_originals024) fn validate_chain_v2(
    raw: &[Vec<u8>],
) -> io::Result<()> {
    if raw.is_empty() || raw.len() > 4096 {
        return Err(invalid());
    }
    let mut previous: Option<RecordV2> = None;
    let mut quiesced = false;
    let mut terminal = false;
    let mut nodes = BTreeMap::<String, (u8, String, String, Option<String>)>::new();
    let mut retain = BTreeMap::<String, (bool, Role, Stage, String, String)>::new();
    for (i, bytes) in raw.iter().enumerate() {
        let record = RecordV2::decode(bytes)?;
        if terminal || record.sequence as usize != i + 1 {
            return Err(invalid());
        }
        if let Some(p) = &previous {
            if !p.same_origin(&record)
                || record.previous_record_digest.as_deref()
                    != Some(&crate::infrastructure::hash::sha256_hex(&raw[i - 1]))
            {
                return Err(invalid());
            }
        } else if !matches!(record.event, EventV2::AbortRecoveryIntent {}) {
            return Err(invalid());
        }
        match &record.event {
            EventV2::AbortRecoveryIntent {} => {}
            EventV2::LauncherQuiesced { .. } => {
                if quiesced {
                    return Err(invalid());
                }
                quiesced = true;
            }
            EventV2::WorkRetireIntent {
                node_id,
                work_file_id,
                parent_id,
            } => {
                if !quiesced || nodes.contains_key(node_id) {
                    return Err(invalid());
                }
                nodes.insert(
                    node_id.clone(),
                    (1, work_file_id.clone(), parent_id.clone(), None),
                );
            }
            EventV2::WorkRetiredVerified {
                node_id,
                work_file_id,
                parent_id,
            } => {
                let n = nodes.get_mut(node_id).ok_or_else(invalid)?;
                if n.0 != 1 || n.1 != *work_file_id || n.2 != *parent_id {
                    return Err(invalid());
                }
                n.0 = 2;
            }
            EventV2::OriginalReturnIntent {
                node_id,
                original_file_id,
                parent_id,
            } => {
                if !quiesced {
                    return Err(invalid());
                }
                let n = nodes.entry(node_id.clone()).or_insert((
                    2,
                    String::new(),
                    parent_id.clone(),
                    None,
                ));
                if n.0 != 2 || n.2 != *parent_id || n.1 == *original_file_id {
                    return Err(invalid());
                }
                n.0 = 3;
                n.3 = Some(original_file_id.clone());
            }
            EventV2::OriginalReturnedVerified {
                node_id,
                original_file_id,
                parent_id,
            } => {
                let n = nodes.get_mut(node_id).ok_or_else(invalid)?;
                if n.0 != 3 || n.2 != *parent_id || n.3.as_deref() != Some(original_file_id) {
                    return Err(invalid());
                }
                n.0 = 4;
            }
            EventV2::PreparationRetainIntent {
                object_id,
                role,
                original_stage,
                identity_digest,
                retained_membership_digest,
            } => {
                if !quiesced || retain.contains_key(object_id) {
                    return Err(invalid());
                }
                retain.insert(
                    object_id.clone(),
                    (
                        false,
                        *role,
                        *original_stage,
                        identity_digest.clone(),
                        retained_membership_digest.clone(),
                    ),
                );
            }
            EventV2::PreparationRetainedVerified {
                object_id,
                role,
                original_stage,
                identity_digest,
                retained_membership_digest,
            } => {
                let n = retain.get_mut(object_id).ok_or_else(invalid)?;
                if n.0
                    || n.1 != *role
                    || n.2 != *original_stage
                    || n.3 != *identity_digest
                    || n.4 != *retained_membership_digest
                {
                    return Err(invalid());
                }
                n.0 = true;
            }
            EventV2::AbortRestoreVerified { original_nodes, .. } => {
                if !quiesced
                    || nodes.values().any(|v| v.0 != 4)
                    || nodes.len() > *original_nodes as usize
                    || retain.values().any(|v| !v.0)
                {
                    return Err(invalid());
                }
                if let ReferenceV2::Preparation { components, .. } = &record.reference {
                    let expected = components
                        .iter()
                        .flat_map(|c| &c.objects)
                        .map(|o| o.object_id.as_str())
                        .collect::<BTreeSet<_>>();
                    if expected != retain.keys().map(String::as_str).collect() {
                        return Err(invalid());
                    }
                }
                terminal = true;
            }
            EventV2::AbortRecoveryPending { .. } => terminal = true,
        }
        previous = Some(record);
    }
    Ok(())
}
