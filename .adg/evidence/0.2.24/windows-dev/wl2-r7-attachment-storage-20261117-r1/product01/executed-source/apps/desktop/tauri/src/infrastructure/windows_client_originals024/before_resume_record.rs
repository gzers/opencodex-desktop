//! Strict r7 recovery attachment DATA. Never deserialized into native proofs.
//! The old v3 current/union remains unchanged; no Closed or successor factory.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io};
const SCHEMA: &str = "windows-before-resume-recovery-v1";
const MAX_RECORD: usize = 65536;
#[cfg(windows)]
mod storage;
#[cfg(windows)]
pub(super) use storage::AttachmentStorage;
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "beforeResume attachment unconfirmed",
    )
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s != "00000000-0000-0000-0000-000000000000"
        && s.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_digit() || (b'a'..=b'f').contains(&c)
            }
        })
}
fn number(s: &str) -> io::Result<u64> {
    s.parse::<u64>()
        .ok()
        .filter(|n| n.to_string() == s)
        .ok_or_else(invalid)
}
fn object_id(s: &str) -> bool {
    number(s).is_ok_and(|n| n > 0)
}
fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Record {
    schema: String,
    session_id: String,
    failed_action_id: String,
    manifest_digest: String,
    binding_digest: String,
    storage_plan_digest: String,
    security_policy_digest: String,
    failed_prefix_digest: String,
    recovery_action_id: String,
    sequence: u32,
    #[serde(deserialize_with = "required_option")]
    previous_record_digest: Option<String>,
    started_qpc: String,
    deadline_qpc: String,
    qpc_frequency: String,
    event: Event,
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Event {
    AbortRecoveryIntent {},
    #[serde(rename_all = "camelCase")]
    LauncherQuiesced {
        pid: u32,
        creation_filetime: String,
        forced: bool,
        raw_exit: u32,
        job_total: u32,
        job_active: u32,
    },
    #[serde(rename_all = "camelCase")]
    WorkRetireIntent {
        node_id: String,
        work_file_id: String,
        parent_id: String,
    },
    #[serde(rename_all = "camelCase")]
    WorkRetiredVerified {
        node_id: String,
        work_file_id: String,
        parent_id: String,
    },
    #[serde(rename_all = "camelCase")]
    OriginalReturnIntent {
        node_id: String,
        original_file_id: String,
        parent_id: String,
    },
    #[serde(rename_all = "camelCase")]
    OriginalReturnedVerified {
        node_id: String,
        original_file_id: String,
        parent_id: String,
    },
    #[serde(rename_all = "camelCase")]
    AbortRestoreVerified {
        original_nodes: u32,
        coordination_objects: u32,
        normal_stop: bool,
        closed: bool,
        public_start_open: bool,
    },
    #[serde(rename_all = "camelCase")]
    AbortRecoveryPending {
        stage: String,
        #[serde(deserialize_with = "required_option")]
        raw_error: Option<i32>,
    },
}
impl Record {
    pub(super) fn decode(raw: &[u8]) -> io::Result<Self> {
        if raw.is_empty() || raw.len() > MAX_RECORD {
            return Err(invalid());
        }
        let record: Self = serde_json::from_slice(raw).map_err(|_| invalid())?;
        record.validate()?;
        Ok(record)
    }
    fn validate(&self) -> io::Result<()> {
        let (start, end, frequency) = (
            number(&self.started_qpc)?,
            number(&self.deadline_qpc)?,
            number(&self.qpc_frequency)?,
        );
        if self.schema != SCHEMA
            || !uuid(&self.session_id)
            || !uuid(&self.failed_action_id)
            || !uuid(&self.recovery_action_id)
            || self.failed_action_id == self.recovery_action_id
            || self.sequence == 0
            || self.sequence > 4096
            || ![
                &self.manifest_digest,
                &self.binding_digest,
                &self.storage_plan_digest,
                &self.security_policy_digest,
                &self.failed_prefix_digest,
            ]
            .into_iter()
            .all(|s| digest(s))
            || (self.sequence == 1) != self.previous_record_digest.is_none()
            || self
                .previous_record_digest
                .as_deref()
                .is_some_and(|s| !digest(s))
            || frequency == 0
            || end <= start
            || frequency
                .checked_mul(10)
                .is_none_or(|ten| end - start > ten)
        {
            return Err(invalid());
        }
        match &self.event {
            Event::AbortRecoveryIntent {} => {
                if self.sequence != 1 {
                    return Err(invalid());
                }
            }
            Event::LauncherQuiesced {
                pid,
                creation_filetime,
                forced,
                raw_exit,
                job_total,
                job_active,
            } => {
                if *pid == 0
                    || !object_id(creation_filetime)
                    || !forced
                    || *raw_exit != 242
                    || *job_total != 1
                    || *job_active != 0
                {
                    return Err(invalid());
                }
            }
            Event::WorkRetireIntent {
                node_id,
                work_file_id,
                parent_id,
            }
            | Event::WorkRetiredVerified {
                node_id,
                work_file_id,
                parent_id,
            } => {
                if !digest(node_id) || !object_id(work_file_id) || !object_id(parent_id) {
                    return Err(invalid());
                }
            }
            Event::OriginalReturnIntent {
                node_id,
                original_file_id,
                parent_id,
            }
            | Event::OriginalReturnedVerified {
                node_id,
                original_file_id,
                parent_id,
            } => {
                if !digest(node_id) || !object_id(original_file_id) || !object_id(parent_id) {
                    return Err(invalid());
                }
            }
            Event::AbortRestoreVerified {
                original_nodes,
                coordination_objects,
                normal_stop,
                closed,
                public_start_open,
            } => {
                if *original_nodes == 0
                    || *original_nodes > 1024
                    || *coordination_objects == 0
                    || *coordination_objects > 16
                    || *normal_stop
                    || *closed
                    || *public_start_open
                {
                    return Err(invalid());
                }
            }
            Event::AbortRecoveryPending { stage, .. } => {
                if stage.is_empty()
                    || stage.len() > 128
                    || !stage
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    fn same_origin_action(&self, other: &Self) -> bool {
        self.session_id == other.session_id
            && self.failed_action_id == other.failed_action_id
            && self.manifest_digest == other.manifest_digest
            && self.binding_digest == other.binding_digest
            && self.storage_plan_digest == other.storage_plan_digest
            && self.security_policy_digest == other.security_policy_digest
            && self.failed_prefix_digest == other.failed_prefix_digest
            && self.recovery_action_id == other.recovery_action_id
            && self.started_qpc == other.started_qpc
            && self.deadline_qpc == other.deadline_qpc
            && self.qpc_frequency == other.qpc_frequency
    }
}
/// Validates only attachment byte linkage and legal DATA ordering. No normal
/// stop/Closed/restore permit can be obtained from this function's unit result.
pub(super) fn validate_chain(raw: &[Vec<u8>]) -> io::Result<()> {
    if raw.is_empty() || raw.len() > 4096 {
        return Err(invalid());
    }
    let mut previous: Option<Record> = None;
    let mut quiesced = false;
    let mut terminal = false;
    let mut nodes = BTreeMap::<String, (u8, String, String, Option<String>)>::new();
    for (i, bytes) in raw.iter().enumerate() {
        let r = Record::decode(bytes)?;
        if terminal || r.sequence as usize != i + 1 {
            return Err(invalid());
        }
        if let Some(p) = &previous {
            if !r.same_origin_action(p)
                || r.previous_record_digest.as_deref()
                    != Some(&crate::infrastructure::hash::sha256_hex(&raw[i - 1]))
            {
                return Err(invalid());
            }
        } else if !matches!(r.event, Event::AbortRecoveryIntent {}) {
            return Err(invalid());
        }
        match &r.event {
            Event::AbortRecoveryIntent {} => {}
            Event::LauncherQuiesced { .. } => {
                if quiesced {
                    return Err(invalid());
                }
                quiesced = true;
            }
            Event::WorkRetireIntent {
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
            Event::WorkRetiredVerified {
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
            Event::OriginalReturnIntent {
                node_id,
                original_file_id,
                parent_id,
            } => {
                if !quiesced {
                    return Err(invalid());
                }
                // A prefix may contain moved originals with no published work.
                // Native factory must prove which branch applies; DATA cannot.
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
            Event::OriginalReturnedVerified {
                node_id,
                original_file_id,
                parent_id,
            } => {
                let n = nodes.get_mut(node_id).ok_or_else(invalid)?;
                if n.0 != 3 || n.3.as_deref() != Some(original_file_id) || n.2 != *parent_id {
                    return Err(invalid());
                }
                n.0 = 4;
            }
            Event::AbortRestoreVerified { original_nodes, .. } => {
                if !quiesced
                    || nodes.values().any(|n| n.0 != 4)
                    || nodes.len() > *original_nodes as usize
                {
                    return Err(invalid());
                }
                terminal = true;
            }
            Event::AbortRecoveryPending { .. } => terminal = true,
        }
        previous = Some(r);
    }
    Ok(())
}
#[cfg(test)]
#[path = "before_resume_record/tests.rs"]
mod tests;
