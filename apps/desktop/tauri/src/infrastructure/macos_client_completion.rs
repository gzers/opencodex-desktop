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
    time(deadline)?;
    // All raw pins precede ANY SQLite acquisition; close all connections before
    // dropping any pin, preserving process-associated POSIX lock semantics.
    let mut pair = LockedPair {
        connections: Vec::new(),
        pins: Vec::new(),
    };
    for (name, identity) in LOCKS.iter().zip(expected) {
        let pin = CoordinationFilePin::open(&client.path.join(name))?;
        if pin.identity != *identity {
            return Err(denied("stable lock generation changed"));
        }
        pair.pins.push(pin);
    }
    for name in LOCKS {
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
    attempt: String,
    sha256: String,
}
impl ReadbackProof {
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn attempt(&self) -> &str {
        &self.attempt
    }
}
impl Originals {
    pub(super) fn complete_readback(
        &self,
        references: session::References,
        plan: session_index::RestorePlan,
        deadline: Instant,
    ) -> io::Result<ReadbackProof> {
        time(deadline)?;
        if references.attempt != self.attempt || references.original_sha256 != self.original_sha256
        {
            return Err(denied("readback references belong to another original"));
        }
        let original = self.frame(Phase::Original)?.0;
        let stopped = self.frame(Phase::Stopped)?.0;
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
                &serde_json::json!({"schemaVersion":1,"planSha256":plan.sha256,"decision":"complete"}),
            )?
        {
            return Err(denied("artifact outcome does not match independent plan"));
        }
        let codex = TreeSnapshot::read(&self.client)?;
        time(deadline)?;
        let grok = GrokSnapshot::read(&self.profile)?;
        time(deadline)?;
        if !original.tree.restored_matches(&codex, true) || !original.grok.restored_matches(&grok) {
            return Err(denied(
                "full client tree original bytes or protection differ",
            ));
        }
        let stable_locks = [
            stopped.tree.stable_identity(LOCKS[0])?,
            stopped.tree.stable_identity(LOCKS[1])?,
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
        if !original.tree.restored_matches(&after, true)
            || !codex.restored_matches(&after, true)
            || GrokSnapshot::read(&self.profile)? != grok
        {
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
        create(&self.directory, "completion-readback.json", &raw)?;
        let Image::File {
            bytes: readback,
            protection,
            ..
        } = capture_file(&self.directory, "completion-readback.json", MAX_FRAME)?
        else {
            return Err(denied("completion receipt missing"));
        };
        require_private(&protection, false)?;
        if readback != raw {
            return Err(denied("completion receipt changed"));
        }
        time(deadline)?;
        Ok(ReadbackProof {
            attempt: self.attempt.clone(),
            sha256: digest(&raw),
        })
    }
}

#[cfg(test)]
#[path = "macos_client_completion_tests.rs"]
mod tests;
