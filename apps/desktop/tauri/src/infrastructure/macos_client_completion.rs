//! Full readback and the exact TWO adopted stable-lock release facts.
//! This receipt is a completion prerequisite, not a generic process-family
//! certificate or permission to discard the owning session/index.
use super::*;
use std::time::Instant;
const LOCKS: [&str; 2] = [
    ".opencodex-native-main.claim.sqlite",
    ".opencodex-native-main.owner.sqlite",
];
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("original Stop readback budget expired"))
    } else {
        Ok(())
    }
}
struct LockedPair {
    connections: Vec<rusqlite::Connection>,
    pins: Vec<CoordinationFilePin>,
}
impl LockedPair {
    fn release(&mut self) -> io::Result<()> {
        let mut failed = false;
        while let Some(c) = self.connections.pop() {
            if c.execute_batch("ROLLBACK").is_err() {
                failed = true;
            }
            if let Err((c, _)) = c.close() {
                drop(c);
                failed = true;
            }
        }
        if failed {
            Err(denied("stable lock release unconfirmed"))
        } else {
            Ok(())
        }
    }
}
impl Drop for LockedPair {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
fn prove_released(
    client: &PinnedDirectory,
    expected: &[Identity; 2],
    deadline: Instant,
) -> io::Result<()> {
    prove_optional_released(client, &expected.clone().map(Some), deadline)
}
pub(super) fn prove_optional_released(
    client: &PinnedDirectory,
    expected: &[Option<Identity>; 2],
    deadline: Instant,
) -> io::Result<()> {
    time(deadline)?;
    // All raw pins precede ANY SQLite acquisition; close all connections before
    // dropping any pin, preserving process-associated POSIX lock semantics.
    let mut pair = LockedPair {
        connections: Vec::new(),
        pins: Vec::new(),
    };
    for (name, identity) in LOCKS.iter().zip(expected) {
        let Some(identity) = identity else {
            if session_index::observation::entry(client, name)?.is_some() {
                return Err(denied("uncaptured stable lock appeared"));
            }
            continue;
        };
        let pin = CoordinationFilePin::open(&client.path.join(name))?;
        if pin.identity != *identity {
            return Err(denied("stable lock generation changed"));
        }
        pair.pins.push(pin);
    }
    for (name, identity) in LOCKS.into_iter().zip(expected) {
        if identity.is_none() {
            continue;
        }
        time(deadline)?;
        let c = rusqlite::Connection::open_with_flags(
            client.path.join(name),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|_| denied("stable SQLite lock unavailable"))?;
        c.busy_timeout(std::time::Duration::ZERO)
            .map_err(|_| denied("stable lock budget unavailable"))?;
        c.execute_batch("BEGIN EXCLUSIVE")
            .map_err(|_| denied("stable lock still held or unknown"))?;
        pair.connections.push(c);
        for pin in &pair.pins {
            pin.recheck()?;
        }
    }
    for c in &pair.connections {
        let mut moved = 1i32;
        let result = unsafe {
            rusqlite::ffi::sqlite3_file_control(
                c.handle(),
                c"main".as_ptr(),
                rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
                (&mut moved as *mut i32).cast(),
            )
        };
        if c.is_autocommit() || result != rusqlite::ffi::SQLITE_OK || moved != 0 {
            return Err(denied("stable native lock identity unknown"));
        }
    }
    time(deadline)?;
    pair.release()?;
    for pin in &pair.pins {
        pin.recheck()?;
    }
    client.recheck()?;
    for (name, identity) in LOCKS.iter().zip(expected) {
        if identity.is_none() && session_index::observation::entry(client, name)?.is_some() {
            return Err(denied("stable lock appeared during release proof"));
        }
    }
    time(deadline)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    references: session::References,
    plan: session_index::RestorePlan,
    outcome_sha256: String,
    codex_tree_sha256: String,
    grok_tree_sha256: String,
    stable_locks: [Identity; 2],
}
/// Private native issuance only, nonserializable. Index closure must consume
/// its own independently bound reference, not an arbitrary `completed` boolean.
pub struct ReadbackProof {
    decision: restore::Decision,
    attempt: String,
    sha256: String,
    references: session::References,
    plan: session_index::RestorePlan,
    data: Identity,
    client: Identity,
    deadline: Instant,
}
impl ReadbackProof {
    pub(super) fn decision(&self) -> restore::Decision {
        self.decision
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn attempt(&self) -> &str {
        &self.attempt
    }
    pub(super) fn require_index(
        &self,
        head: &session_index::Head,
        data: &Identity,
        client: &Identity,
    ) -> io::Result<()> {
        time(self.deadline)?;
        if !matches!(
            head.state,
            session_index::State::RestorePrepared | session_index::State::RecoveryPrepared
        ) || head.references.as_ref() != Some(&self.references)
            || head.restore_plan.as_ref() != Some(&self.plan)
            || self.data != *data
            || self.client != *client
        {
            return Err(denied(
                "completion proof belongs to another session/plan/root",
            ));
        }
        Ok(())
    }
}
/// Read only the immutable receipt selected by an independent index reference.
/// Current user edits AFTER closure are not a reason to replay old restoration.
pub(super) fn verify_registered(
    data: &Path,
    client: &Path,
    profile: &Path,
    head: &session_index::Head,
) -> io::Result<()> {
    let decision = match head.state {
        session_index::State::Restored => restore::Decision::Complete,
        session_index::State::RestoreRolledBack => restore::Decision::Rollback,
        _ => return Err(denied("registered readback is not a terminal state")),
    };
    let refs = head
        .references
        .as_ref()
        .ok_or_else(|| denied("completion original references missing"))?;
    let plan = head
        .restore_plan
        .as_ref()
        .ok_or_else(|| denied("completion plan missing"))?;
    let expected = head
        .completion_sha256
        .as_ref()
        .ok_or_else(|| denied("completion digest missing"))?;
    let originals = Originals::open_for_profile(
        data,
        client,
        profile,
        &refs.attempt,
        &refs.original_sha256,
        &refs.phases,
    )?;
    session::verify_closed_reference_files(&originals, refs)
        .map_err(|_| denied("closed session scope/live reference changed"))?;
    let pending = restore::Pending::open(&originals, &plan.id, &plan.sha256)?;
    pending.require_indexed_references(refs)?;
    let receipt_name = pending.readback_name(decision);
    let Image::File {
        bytes, protection, ..
    } = capture_file(&originals.directory, &receipt_name, MAX_FRAME)?
    else {
        return Err(denied("registered completion receipt absent"));
    };
    require_private(&protection, false)?;
    if digest(&bytes) != *expected {
        return Err(denied("registered completion receipt changed"));
    }
    let receipt: Receipt = serde_json::from_value(
        parse_strict_json_v2(&bytes).map_err(|_| denied("invalid completion JSON"))?,
    )
    .map_err(|_| denied("invalid completion fields"))?;
    if receipt.schema_version != 1 || receipt.references != *refs || receipt.plan != *plan {
        return Err(denied("completion reference binding differs"));
    }
    let Image::File {
        bytes: outcome,
        protection,
        ..
    } = capture_file(
        &originals.directory,
        &format!("restore-{}-outcome.json", plan.id),
        MAX_FRAME,
    )?
    else {
        return Err(denied("completion outcome missing"));
    };
    require_private(&protection, false)?;
    if digest(&outcome) != receipt.outcome_sha256
        || outcome
            != encode(
                &serde_json::json!({"schemaVersion":1,"planSha256":plan.sha256,"decision":decision}),
            )?
    {
        return Err(denied("completion outcome not original plan"));
    }
    Ok(())
}
impl Originals {
    pub(super) fn rearm_fingerprint(
        &self,
        head: &session_index::Head,
        leased: bool,
    ) -> io::Result<Option<String>> {
        if head.state != session_index::State::RestoreRolledBack {
            return Ok(None);
        }
        verify_registered(&self.data.path, &self.client.path, &self.profile.path, head)?;
        Ok(self
            .recovery_fingerprint(head, leased, restore::Decision::Rollback)?
            .map(|fp| digest(format!("rearm:{fp}").as_bytes())))
    }
    pub(super) fn recovery_fingerprint(
        &self,
        head: &session_index::Head,
        leased: bool,
        decision: restore::Decision,
    ) -> io::Result<Option<String>> {
        let (complete, rollback) = self.recovery_fingerprints(head, leased)?;
        Ok(if decision == restore::Decision::Complete {
            complete
        } else {
            rollback
        })
    }
    pub(super) fn recovery_fingerprints(
        &self,
        head: &session_index::Head,
        leased: bool,
    ) -> io::Result<(Option<String>, Option<String>)> {
        if !matches!(
            head.state,
            session_index::State::RestorePrepared | session_index::State::RecoveryPrepared
        ) && head.state != session_index::State::RestoreRolledBack
        {
            return Ok((None, None));
        }
        let plan = head
            .restore_plan
            .as_ref()
            .ok_or_else(|| denied("recovery plan absent"))?;
        let pending = restore::Pending::open(self, &plan.id, &plan.sha256)?;
        let review = pending.review_recovery()?;
        if review.terminal == Some(restore::Decision::Complete) {
            return Ok((None, None));
        }
        if capture_file(&self.directory, "completion-readback.json", MAX_FRAME)? != Image::Absent {
            return Err(denied("partial plan has unexpected completion evidence"));
        }
        let original = self.frame(Phase::Original)?.0;
        let stopped_tree = pending.stopped_tree()?;
        let codex = TreeSnapshot::read(&self.client)?;
        let grok = GrokSnapshot::read(&self.profile)?;
        let remainder_matches = pending.remainder_matches(&original.tree, &codex, leased);
        if !remainder_matches || !pending.grok_remainder_matches(&original.grok, &grok) {
            return Ok((None, None));
        }
        for name in LOCKS {
            let id = codex.stable_identity(name)?;
            if stopped_tree.stable_identity(name)? != id
                || !original.tree.original_stable_identity_matches(name, &id)
            {
                return Ok((None, None));
            }
        }
        // Full tree changes caused by coordinator acquisition are deliberately
        // not confirmation input. The remainder and stable identities above
        // are revalidated, while exact artifact generations remain bound here.
        let fingerprint = |decision| -> io::Result<String> {
            Ok(digest(&encode(&(
                head,
                &review.fingerprint,
                decision,
                self.data.identity(),
                self.client.identity(),
                self.profile.identity(),
            ))?))
        };
        let complete = if review.terminal.is_none() {
            Some(fingerprint(restore::Decision::Complete)?)
        } else {
            None
        };
        let rollback = Some(fingerprint(restore::Decision::Rollback)?);
        Ok((complete, rollback))
    }
    pub(super) fn finalization_fingerprint(
        &self,
        head: &session_index::Head,
    ) -> io::Result<Option<String>> {
        if !matches!(
            head.state,
            session_index::State::RestorePrepared | session_index::State::RecoveryPrepared
        ) {
            return Ok(None);
        }
        let refs = head
            .references
            .as_ref()
            .ok_or_else(|| denied("finalization original references absent"))?;
        let plan = head
            .restore_plan
            .as_ref()
            .ok_or_else(|| denied("finalization original plan absent"))?;
        let pending = restore::Pending::open(self, &plan.id, &plan.sha256)?;
        let review = pending.review_recovery()?;
        if review.terminal != Some(restore::Decision::Complete) {
            return Ok(None);
        }
        let original = self.frame(Phase::Original)?.0;
        let stopped_tree = pending.stopped_tree()?;
        let codex = TreeSnapshot::read(&self.client)?;
        let grok = GrokSnapshot::read(&self.profile)?;
        if !original.tree.restored_matches(&codex, true) || !original.grok.restored_matches(&grok) {
            return Ok(None);
        }
        let stable = [
            stopped_tree.stable_identity(LOCKS[0])?,
            stopped_tree.stable_identity(LOCKS[1])?,
        ];
        for (name, id) in LOCKS.iter().zip(&stable) {
            if codex.stable_identity(name)? != *id
                || !original.tree.original_stable_identity_matches(name, id)
            {
                return Ok(None);
            }
        }
        let outcome = capture_file(
            &self.directory,
            &format!("restore-{}-outcome.json", plan.id),
            MAX_FRAME,
        )?;
        let Image::File {
            bytes: outcome,
            protection,
            ..
        } = outcome
        else {
            return Ok(None);
        };
        require_private(&protection, false)?;
        let codex_sha = digest(&encode(&codex)?);
        let grok_sha = digest(&encode(&grok)?);
        let receipt_name = pending.readback_name(restore::Decision::Complete);
        let receipt_hash = match capture_file(&self.directory, &receipt_name, MAX_FRAME)? {
            Image::Absent => None,
            Image::File {
                bytes, protection, ..
            } => {
                require_private(&protection, false)?;
                let receipt: Receipt = serde_json::from_value(
                    parse_strict_json_v2(&bytes)
                        .map_err(|_| denied("invalid pending completion JSON"))?,
                )
                .map_err(|_| denied("invalid pending completion fields"))?;
                if receipt.schema_version != 1
                    || receipt.references != *refs
                    || receipt.plan != *plan
                    || receipt.outcome_sha256 != digest(&outcome)
                    || receipt.codex_tree_sha256 != codex_sha
                    || receipt.grok_tree_sha256 != grok_sha
                    || receipt.stable_locks != stable
                {
                    return Err(denied(
                        "pending completion differs; not overwritten or adopted",
                    ));
                }
                Some(digest(&bytes))
            }
        };
        Ok(Some(digest(&encode(&(
            head,
            self.data.identity(),
            self.client.identity(),
            self.profile.identity(),
            codex_sha,
            grok_sha,
            digest(&outcome),
            receipt_hash,
            review.fingerprint,
        ))?)))
    }
    pub(super) fn finalize_readback(
        &self,
        refs: session::References,
        plan: session_index::RestorePlan,
        deadline: Instant,
    ) -> io::Result<ReadbackProof> {
        self.complete_readback_inner(refs, plan, deadline, true, restore::Decision::Complete)
    }
    pub(super) fn complete_readback(
        &self,
        references: session::References,
        plan: session_index::RestorePlan,
        deadline: Instant,
    ) -> io::Result<ReadbackProof> {
        self.complete_readback_inner(
            references,
            plan,
            deadline,
            false,
            restore::Decision::Complete,
        )
    }
    pub(super) fn rollback_readback(
        &self,
        references: session::References,
        plan: session_index::RestorePlan,
        deadline: Instant,
    ) -> io::Result<ReadbackProof> {
        self.complete_readback_inner(
            references,
            plan,
            deadline,
            true,
            restore::Decision::Rollback,
        )
    }
    fn complete_readback_inner(
        &self,
        references: session::References,
        plan: session_index::RestorePlan,
        deadline: Instant,
        reuse_exact_receipt: bool,
        decision: restore::Decision,
    ) -> io::Result<ReadbackProof> {
        if self.original_root_absence.is_some() {
            return Err(denied(
                "original root absence requires whole-root restoration proof",
            ));
        }
        time(deadline)?;
        if references.attempt != self.attempt || references.original_sha256 != self.original_sha256
        {
            return Err(denied("readback references belong to another original"));
        }
        let original = self.frame(Phase::Original)?.0;
        let pending = restore::Pending::open(self, &plan.id, &plan.sha256)?;
        pending.require_indexed_references(&references)?;
        let stopped_tree = pending.stopped_tree()?;
        let receipt_name = pending.readback_name(decision);
        let outcome_name = format!("restore-{}-outcome.json", plan.id);
        let outcome = capture_file(&self.directory, &outcome_name, MAX_FRAME)?;
        let Image::File {
            bytes, protection, ..
        } = outcome
        else {
            return Err(denied("completed artifact outcome absent"));
        };
        require_private(&protection, false)?;
        if bytes
            != encode(
                &serde_json::json!({"schemaVersion":1,"planSha256":plan.sha256,"decision":decision}),
            )?
        {
            return Err(denied("artifact outcome does not match independent plan"));
        }
        let codex = TreeSnapshot::read(&self.client)?;
        time(deadline)?;
        let grok = GrokSnapshot::read(&self.profile)?;
        time(deadline)?;
        let tree_matches = match decision {
            restore::Decision::Complete => original.tree.restored_matches(&codex, true),
            restore::Decision::Rollback => {
                let review = pending.review_recovery()?;
                review.terminal == Some(restore::Decision::Rollback)
                    && pending.remainder_matches(&original.tree, &codex, false)
            }
        };
        if !tree_matches
            || !(if decision == restore::Decision::Rollback {
                pending.grok_remainder_matches(&original.grok, &grok)
            } else {
                original.grok.restored_matches(&grok)
            })
        {
            return Err(denied(
                "full client tree original bytes or protection differ",
            ));
        }
        let stable_locks = [
            stopped_tree.stable_identity(LOCKS[0])?,
            stopped_tree.stable_identity(LOCKS[1])?,
        ];
        for (name, expected) in LOCKS.iter().zip(&stable_locks) {
            if codex.stable_identity(name)? != *expected
                || !original
                    .tree
                    .original_stable_identity_matches(name, expected)
            {
                return Err(denied(
                    "stable locks not bound to original/Stopped generations",
                ));
            }
        }
        // This runs only after the original native lease has normally released.
        // Do not read/open/close a SQLite file while this pair owns it.
        prove_released(&self.client, &stable_locks, deadline)?;
        let after = TreeSnapshot::read(&self.client)?;
        time(deadline)?;
        if !codex.restored_matches(&after, true) || GrokSnapshot::read(&self.profile)? != grok {
            return Err(denied("client tree changed during final lock proof"));
        }
        for (name, expected) in LOCKS.iter().zip(&stable_locks) {
            if after.stable_identity(name)? != *expected {
                return Err(denied("stable lock changed after release"));
            }
        }
        let receipt = Receipt {
            schema_version: 1,
            references,
            plan,
            outcome_sha256: digest(&bytes),
            codex_tree_sha256: digest(&encode(&after)?),
            grok_tree_sha256: digest(&encode(&grok)?),
            stable_locks,
        };
        let raw = encode(&receipt)?;
        time(deadline)?;
        if reuse_exact_receipt {
            match capture_file(&self.directory, &receipt_name, MAX_FRAME)? {
                Image::Absent => create(&self.directory, &receipt_name, &raw)?,
                Image::File {
                    bytes, protection, ..
                } => {
                    require_private(&protection, false)?;
                    if bytes != raw {
                        return Err(denied("previous completion receipt changed; not replaced"));
                    }
                }
            }
        } else {
            create(&self.directory, &receipt_name, &raw)?;
        }
        let Image::File {
            bytes: readback,
            protection,
            ..
        } = capture_file(&self.directory, &receipt_name, MAX_FRAME)?
        else {
            return Err(denied("completion receipt missing"));
        };
        require_private(&protection, false)?;
        if readback != raw {
            return Err(denied("completion receipt changed"));
        }
        time(deadline)?;
        Ok(ReadbackProof {
            decision,
            attempt: self.attempt.clone(),
            sha256: digest(&raw),
            references: receipt.references,
            plan: receipt.plan,
            data: self.data.identity().clone(),
            client: self.client.identity().clone(),
            deadline,
        })
    }
}

#[cfg(test)]
#[path = "macos_client_completion_tests.rs"]
mod tests;
