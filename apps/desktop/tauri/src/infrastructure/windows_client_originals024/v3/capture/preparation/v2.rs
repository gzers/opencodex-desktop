//! Actual original preparation membership projection. Output is DATA/Pending,
//! never a factory for native restoration or a disk-record reconstruction.
use super::*;
use crate::infrastructure::windows_client_originals024::before_resume_record_v2::{
    projection_digest, retained_object, Component, Object, Presence, ReferenceV2, Role,
    Source as SourceState, Stage,
};
fn opaque(
    session: uuid::Uuid,
    role: Role,
    ordinal: usize,
    label: &str,
    scope: &str,
) -> io::Result<Object> {
    retained_object(
        session,
        role,
        ordinal,
        Stage::HandleRetained,
        projection_digest(&(scope, label, ordinal))?,
        None,
        None,
        None,
    )
}
fn pin(
    session: uuid::Uuid,
    role: Role,
    ordinal: usize,
    pin: &PinnedDirectory,
    errors: &mut Vec<serde_json::Value>,
) -> io::Result<Object> {
    let path = projection_digest(&pin.path())?;
    let ids = match pin.owned_file_identities() {
        Ok(ids) => Some(projection_digest(
            &ids.iter()
                .map(|id| (id.volume, id.index.to_string()))
                .collect::<Vec<_>>(),
        )?),
        Err(e) => {
            errors.push(serde_json::json!({"role":role,"ordinal":ordinal,"raw":e.raw_os_error(),"kind":format!("{:?}",e.kind())}));
            None
        }
    };
    retained_object(
        session,
        role,
        ordinal,
        Stage::HandleRetained,
        path,
        ids,
        None,
        None,
    )
}
fn component(role: Role, objects: Vec<Object>) -> io::Result<Component> {
    let observed = !objects.is_empty();
    let prefix = if observed {
        Some(projection_digest(&objects)?)
    } else {
        None
    };
    Ok(Component {
        role,
        state: if observed {
            Presence::Retained
        } else {
            Presence::Unobserved
        },
        prefix_digest: prefix,
        objects,
    })
}
impl PartialCaptureAnchor {
    pub(in crate::infrastructure::windows_client_originals024) fn v2_reference_data(
        &mut self,
        end: Instant,
    ) -> io::Result<(ReferenceV2, serde_json::Value)> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        let mut errors = Vec::new();
        let mut source = Vec::new();
        let session = self.session_id;
        if self.source.is_some() {
            source.push(opaque(
                session,
                Role::Source,
                source.len(),
                "originalSource",
                &self.owner_scope,
            )?);
        }
        for (ordinal, (label, exists)) in [
            ("originalRoute", self.routes.is_some()),
            ("originalContentScope", self.content_start.is_some()),
            ("originalJournalContext", self.context.is_some()),
        ]
        .into_iter()
        .enumerate()
        {
            if exists {
                source.push(opaque(
                    session,
                    Role::Source,
                    ordinal + 1,
                    label,
                    &self.owner_scope,
                )?);
            }
        }
        for (ordinal, original) in self.pins.iter().enumerate() {
            source.push(pin(
                session,
                Role::Source,
                16 + ordinal,
                original,
                &mut errors,
            )?);
        }
        let mut custody = Vec::new();
        for (ordinal, original) in self
            .custody
            .iter()
            .enumerate()
            .filter_map(|(ordinal, pin)| pin.as_ref().map(|pin| (ordinal, pin)))
        {
            custody.push(pin(session, Role::Custody, ordinal, original, &mut errors)?);
        }
        let stages=self.directory_prefixes.iter().map(|p|{let (path,[attempted,parent,created,pending,complete,ready,failed])=p.retained_stage_data();
            Ok(serde_json::json!({"path":path,"attempted":attempted,"parent":parent,"created":created,"pendingOriginalHandle":pending,"completePin":complete,"ready":ready,"failed":failed,"nativeRead":super::directory_read::read(p,end)?}))}).collect::<io::Result<Vec<_>>>()?;
        for (i, data) in stages.iter().enumerate() {
            // Path is never persisted raw; this bounded native locator is only
            // hashed here. Create succeeded but open unknown remains explicit.
            let path = projection_digest(&(&data["path"], "original-directory-creation-prefix"))?;
            custody.push(retained_object(
                session,
                Role::Custody,
                16 + i,
                if data["pendingOriginalHandle"] == true || data["completePin"] == true {
                    Stage::HandleRetained
                } else if data["created"] == true {
                    Stage::NativeCreated
                } else {
                    Stage::IntentPublished
                },
                path,
                data["nativeRead"]["identityDigest"]
                    .as_str()
                    .map(str::to_string),
                None,
                data["nativeRead"]["contentDigest"]
                    .as_str()
                    .map(str::to_string),
            )?);
        }
        let mut tree_components = Vec::new();
        for (i, role) in [Role::Codex, Role::Grok].into_iter().enumerate() {
            let objects = if let Some(tree) = &self.trees[i] {
                let (objects, observed_errors) =
                    tree.v2_retained_object_data(session, role, end)?;
                errors.extend(observed_errors);
                objects
            } else {
                Vec::new()
            };
            tree_components.push(component(role, objects)?);
        }
        let journal_members = self.journal_prefix.retained_v2_membership_data();
        let mut journal = Vec::new();
        if self.journal.is_some() {
            journal.push(opaque(
                session,
                Role::Journal,
                0,
                "originalJournal",
                &self.owner_scope,
            )?);
        }
        let native_journal = if journal_members["attempted"] == true && self.journal.is_none() {
            let (actual, observation) = self.journal_prefix.v2_native_objects(session, end)?;
            journal.extend(actual);
            Some(observation)
        } else {
            None
        };
        let mut components = vec![
            component(Role::Source, source)?,
            component(Role::Custody, custody)?,
        ];
        components.extend(tree_components);
        components.push(component(Role::Journal, journal)?);
        let source_state = self
            .source
            .as_ref()
            .map(|s| {
                let (binding_digest, image_digest) = s.retained_v2_source_data();
                SourceState::Captured {
                    binding_digest,
                    image_digest,
                }
            })
            .unwrap_or(SourceState::Unobserved {});
        let intent_projection = serde_json::json!({"sessionId":session.to_string(),"ownerInstanceId":self.host_instance_id.to_string(),"scope":self.owner_scope,"intents":self.intents.iter().map(projection_digest).collect::<io::Result<Vec<_>>>()?});
        let stage_projection = serde_json::json!({"components":components,"directoryStages":stages.iter().map(projection_digest).collect::<io::Result<Vec<_>>>()?,"nativeDirectoryReads":stages.iter().map(|p|p["nativeRead"].clone()).collect::<Vec<_>>(),"journalMembership":journal_members,"nativeJournalRead":native_journal,"nativeReadErrors":errors,"failed":self.failed});
        let reference = ReferenceV2::Preparation {
            owner_instance_id: self.host_instance_id.to_string(),
            source: source_state,
            intent_prefix_digest: projection_digest(&intent_projection)?,
            native_stage_prefix_digest: projection_digest(&stage_projection)?,
            last_stage: if self.journal.is_some()
                || journal_members["completeOriginalJournal"] == true
            {
                Stage::CurrentPublished
            } else if journal_members["currentOriginalHandle"] == true
                || journal_members["manifestOriginalHandle"] == true
            {
                Stage::HandleRetained
            } else if journal_members["manifestRecord"] == true {
                Stage::JournalPublished
            } else if self.trees.iter().any(Option::is_some)
                || self.custody.iter().any(Option::is_some)
                || !self.pins.is_empty()
                || self.source.is_some()
                || self.context.is_some()
                || self.routes.is_some()
                || self.content_start.is_some()
                || journal_members["directory"] == true
                || journal_members["peer"] == true
                || journal_members["pins"]
                    .as_u64()
                    .is_some_and(|count| count > 0)
                || stages.iter().any(|stage| {
                    stage["pendingOriginalHandle"] == true || stage["completePin"] == true
                })
            {
                Stage::HandleRetained
            } else if stages.iter().any(|stage| stage["created"] == true) {
                Stage::NativeCreated
            } else if !self.intents.is_empty() || !stages.is_empty() {
                Stage::IntentPublished
            } else {
                Stage::Registered
            },
            components,
        };
        reference.validate()?;
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        // All source/scope and complete original-set proof still need native
        // recovery admission; this membership view deliberately stays Pending.
        Ok((
            reference,
            serde_json::json!({"sessionId":session.to_string(),"nativeReadErrors":errors,"objectMembership":stage_projection,"runtimeAuthority":false}),
        ))
    }
}
#[cfg(test)]
mod tests;
