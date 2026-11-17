//! Independent strict V2 DATA. No v1/v3 parsing fallback, record conversion,
//! native Owner/claim/restore constructors or inferred absence from IO errors.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
const SCHEMA: &str =
    crate::modules::runtime::windows_lifecycle024_contract::BEFORE_RESUME_RECOVERY_RECORD_V2;
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "explicit beforeResumeV2 reference unconfirmed",
    )
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok_and(|u| !u.is_nil() && u.to_string() == s)
}
fn number(s: &str) -> io::Result<u64> {
    s.parse::<u64>()
        .ok()
        .filter(|v| v.to_string() == s)
        .ok_or_else(invalid)
}
fn object_id(s: &str) -> bool {
    number(s).is_ok_and(|v| v > 0)
}
fn optional<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d)
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub(super) enum Role {
    Source,
    Custody,
    Codex,
    Grok,
    Journal,
}
const ROLES: [Role; 5] = [
    Role::Source,
    Role::Custody,
    Role::Codex,
    Role::Grok,
    Role::Journal,
];
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) enum Stage {
    Registered,
    IntentPublished,
    NativeCreated,
    HandleRetained,
    SecurityReadback,
    ContentReadback,
    JournalPublished,
    CurrentPublished,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) enum Presence {
    Unobserved,
    Retained,
    VerifiedAbsent,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Object {
    pub(super) object_id: String,
    pub(super) role: Role,
    pub(super) original_stage: Stage,
    pub(super) path_digest: String,
    #[serde(deserialize_with = "optional")]
    pub(super) identity_digest: Option<String>,
    #[serde(deserialize_with = "optional")]
    pub(super) security_digest: Option<String>,
    #[serde(deserialize_with = "optional")]
    pub(super) content_digest: Option<String>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Component {
    pub(super) role: Role,
    pub(super) state: Presence,
    #[serde(deserialize_with = "optional")]
    pub(super) prefix_digest: Option<String>,
    pub(super) objects: Vec<Object>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, tag = "state", rename_all = "camelCase")]
pub(super) enum Source {
    Unobserved {},
    #[serde(rename_all = "camelCase")]
    ObservedAbsent {
        parent_identity_digest: String,
        observation_digest: String,
    },
    #[serde(rename_all = "camelCase")]
    Captured {
        binding_digest: String,
        image_digest: String,
    },
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "camelCase")]
pub(super) enum ReferenceV2 {
    #[serde(rename_all = "camelCase")]
    Canonical {
        manifest_digest: String,
        binding_digest: String,
        storage_plan_digest: String,
        failed_prefix_digest: String,
    },
    #[serde(rename_all = "camelCase")]
    Preparation {
        owner_instance_id: String,
        source: Source,
        intent_prefix_digest: String,
        native_stage_prefix_digest: String,
        last_stage: Stage,
        components: Vec<Component>,
    },
}
impl Object {
    fn complete(&self) -> bool {
        self.identity_digest.is_some()
            && self.security_digest.is_some()
            && self.content_digest.is_some()
    }
}
pub(super) fn projection_digest<T: Serialize>(value: &T) -> io::Result<String> {
    serde_json::to_vec(value)
        .map(|bytes| crate::infrastructure::hash::sha256_hex(&bytes))
        .map_err(|_| invalid())
}
pub(super) fn retained_object(
    session: uuid::Uuid,
    role: Role,
    ordinal: usize,
    stage: Stage,
    path_digest: String,
    identity_digest: Option<String>,
    security_digest: Option<String>,
    content_digest: Option<String>,
) -> io::Result<Object> {
    Ok(Object {
        object_id: projection_digest(&(session.to_string(), role, ordinal))?,
        role,
        original_stage: stage,
        path_digest,
        identity_digest,
        security_digest,
        content_digest,
    })
}
impl ReferenceV2 {
    pub(super) fn validate(&self) -> io::Result<()> {
        if serde_json::to_vec(self).map_err(|_| invalid())?.len() > 65536 {
            return Err(invalid());
        }
        match self {
            Self::Canonical {
                manifest_digest,
                binding_digest,
                storage_plan_digest,
                failed_prefix_digest,
            } => {
                if ![
                    manifest_digest,
                    binding_digest,
                    storage_plan_digest,
                    failed_prefix_digest,
                ]
                .into_iter()
                .all(|h| digest(h))
                {
                    return Err(invalid());
                }
            }
            Self::Preparation {
                owner_instance_id,
                source,
                intent_prefix_digest,
                native_stage_prefix_digest,
                components,
                ..
            } => {
                if !uuid(owner_instance_id)
                    || !digest(intent_prefix_digest)
                    || !digest(native_stage_prefix_digest)
                    || components.len() != 5
                {
                    return Err(invalid());
                }
                match source {
                    Source::Unobserved {} => {}
                    Source::ObservedAbsent {
                        parent_identity_digest,
                        observation_digest,
                    } => {
                        if !digest(parent_identity_digest) || !digest(observation_digest) {
                            return Err(invalid());
                        }
                    }
                    Source::Captured {
                        binding_digest,
                        image_digest,
                    } => {
                        if !digest(binding_digest) || !digest(image_digest) {
                            return Err(invalid());
                        }
                    }
                }
                let mut objects = BTreeSet::new();
                let mut locators = BTreeSet::new();
                for (component, role) in components.iter().zip(ROLES) {
                    if component.role != role
                        || component.prefix_digest.as_ref().is_some_and(|v| !digest(v))
                    {
                        return Err(invalid());
                    }
                    match component.state {
                        Presence::Unobserved => {
                            if component.prefix_digest.is_some() || !component.objects.is_empty() {
                                return Err(invalid());
                            }
                        }
                        Presence::VerifiedAbsent => {
                            if component.prefix_digest.is_none() || !component.objects.is_empty() {
                                return Err(invalid());
                            }
                        }
                        Presence::Retained => {
                            if component.prefix_digest.is_none() || component.objects.is_empty() {
                                return Err(invalid());
                            }
                        }
                    }
                    for object in &component.objects {
                        if object.role != role
                            || !digest(&object.object_id)
                            || !objects.insert(object.object_id.as_str())
                            || !digest(&object.path_digest)
                            || !locators
                                .insert((format!("{:?}", role), object.path_digest.as_str()))
                            || [
                                &object.identity_digest,
                                &object.security_digest,
                                &object.content_digest,
                            ]
                            .into_iter()
                            .any(|v| v.as_ref().is_some_and(|v| !digest(v)))
                        {
                            return Err(invalid());
                        }
                    }
                }
                if objects.len() > 512 {
                    return Err(invalid());
                }
                if matches!(source, Source::Captured { .. })
                    && (components[0].state != Presence::Retained
                        || components[0].objects.is_empty())
                {
                    return Err(invalid());
                }
                if matches!(source, Source::ObservedAbsent { .. })
                    && components[0].state != Presence::VerifiedAbsent
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    pub(super) fn pending(&self) -> bool {
        match self {
            Self::Canonical { .. } => false,
            Self::Preparation {
                source, components, ..
            } => {
                !matches!(source, Source::Captured { .. })
                    || components.iter().any(|c| {
                        c.state == Presence::Unobserved || c.objects.iter().any(|o| !o.complete())
                    })
            }
        }
    }
    fn object(&self, id: &str) -> Option<&Object> {
        match self {
            Self::Canonical { .. } => None,
            Self::Preparation { components, .. } => components
                .iter()
                .flat_map(|c| &c.objects)
                .find(|o| o.object_id == id),
        }
    }
    fn client_objects(&self) -> usize {
        match self {
            Self::Canonical { .. } => 0,
            Self::Preparation { components, .. } => components
                .iter()
                .filter(|c| matches!(c.role, Role::Codex | Role::Grok))
                .map(|c| c.objects.len())
                .sum(),
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, tag = "kind")]
pub(super) enum EventV2 {
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
    PreparationRetainIntent {
        object_id: String,
        role: Role,
        original_stage: Stage,
        identity_digest: String,
        retained_membership_digest: String,
    },
    #[serde(rename_all = "camelCase")]
    PreparationRetainedVerified {
        object_id: String,
        role: Role,
        original_stage: Stage,
        identity_digest: String,
        retained_membership_digest: String,
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
        stage: Stage,
        #[serde(deserialize_with = "optional")]
        raw_error: Option<i32>,
    },
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RecordV2 {
    schema: String,
    session_id: String,
    failed_action_id: String,
    recovery_action_id: String,
    security_policy_digest: String,
    reference: ReferenceV2,
    sequence: u32,
    #[serde(deserialize_with = "optional")]
    previous_record_digest: Option<String>,
    started_qpc: String,
    deadline_qpc: String,
    qpc_frequency: String,
    event: EventV2,
}
impl RecordV2 {
    pub(super) fn decode(raw: &[u8]) -> io::Result<Self> {
        if raw.is_empty() || raw.len() > 65536 {
            return Err(invalid());
        }
        let record: Self = serde_json::from_slice(raw).map_err(|_| invalid())?;
        record.validate()?;
        Ok(record)
    }
    fn validate(&self) -> io::Result<()> {
        let start = number(&self.started_qpc)?;
        let end = number(&self.deadline_qpc)?;
        let frequency = number(&self.qpc_frequency)?;
        if self.schema != SCHEMA
            || ![
                &self.session_id,
                &self.failed_action_id,
                &self.recovery_action_id,
            ]
            .into_iter()
            .all(|s| uuid(s))
            || self.failed_action_id == self.recovery_action_id
            || !digest(&self.security_policy_digest)
            || self.sequence == 0
            || self.sequence > 4096
            || (self.sequence == 1) != self.previous_record_digest.is_none()
            || self
                .previous_record_digest
                .as_ref()
                .is_some_and(|s| !digest(s))
            || frequency == 0
            || end <= start
            || frequency
                .checked_mul(10)
                .is_none_or(|ten| end - start > ten)
        {
            return Err(invalid());
        }
        self.reference.validate()?;
        match &self.event {
            EventV2::AbortRecoveryIntent {} => {
                if self.sequence != 1 {
                    return Err(invalid());
                }
            }
            EventV2::LauncherQuiesced {
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
            EventV2::WorkRetireIntent {
                node_id,
                work_file_id,
                parent_id,
            }
            | EventV2::WorkRetiredVerified {
                node_id,
                work_file_id,
                parent_id,
            } => {
                if !digest(node_id)
                    || !object_id(work_file_id)
                    || !object_id(parent_id)
                    || !matches!(self.reference, ReferenceV2::Canonical { .. })
                {
                    return Err(invalid());
                }
            }
            EventV2::OriginalReturnIntent {
                node_id,
                original_file_id,
                parent_id,
            }
            | EventV2::OriginalReturnedVerified {
                node_id,
                original_file_id,
                parent_id,
            } => {
                if !digest(node_id)
                    || !object_id(original_file_id)
                    || !object_id(parent_id)
                    || !matches!(self.reference, ReferenceV2::Canonical { .. })
                {
                    return Err(invalid());
                }
            }
            EventV2::PreparationRetainIntent {
                object_id,
                role,
                original_stage,
                identity_digest,
                retained_membership_digest,
            }
            | EventV2::PreparationRetainedVerified {
                object_id,
                role,
                original_stage,
                identity_digest,
                retained_membership_digest,
            } => {
                let original = self.reference.object(object_id).ok_or_else(invalid)?;
                if original.role != *role
                    || original.original_stage != *original_stage
                    || original.identity_digest.as_ref() != Some(identity_digest)
                    || !digest(retained_membership_digest)
                {
                    return Err(invalid());
                }
            }
            EventV2::AbortRestoreVerified {
                original_nodes,
                coordination_objects,
                normal_stop,
                closed,
                public_start_open,
            } => {
                if *original_nodes > 1024
                    || *coordination_objects == 0
                    || *coordination_objects > 16
                    || *normal_stop
                    || *closed
                    || *public_start_open
                    || self.reference.pending()
                    || matches!(self.reference, ReferenceV2::Canonical { .. })
                        && *original_nodes == 0
                    || self.reference.client_objects() > *original_nodes as usize
                {
                    return Err(invalid());
                }
            }
            EventV2::AbortRecoveryPending { .. } => {}
        }
        Ok(())
    }
    fn same_origin(&self, other: &Self) -> bool {
        self.schema == other.schema
            && self.session_id == other.session_id
            && self.failed_action_id == other.failed_action_id
            && self.recovery_action_id == other.recovery_action_id
            && self.security_policy_digest == other.security_policy_digest
            && self.reference == other.reference
            && self.started_qpc == other.started_qpc
            && self.deadline_qpc == other.deadline_qpc
            && self.qpc_frequency == other.qpc_frequency
    }
}
mod chain;
pub(super) use chain::validate_chain_v2;
#[cfg(test)]
mod tests;
