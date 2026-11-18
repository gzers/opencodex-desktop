//! Original failed initializer native reads. Output is pending DATA only.
//! Never completes initialization, clears poison, reopens a file for mutation,
//! infers absence from IO failure or grants native recovery from raw bytes.
use super::*;
use crate::infrastructure::windows_client_originals024::before_resume_record_v2::{
    projection_digest, retained_object, Object, Role, Stage,
};
#[cfg(test)]
thread_local! {static AFTER_ROWS:std::cell::Cell<Option<fn(&Path)>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]
pub(super) fn after_rows_for_test(hook: fn(&Path)) {
    AFTER_ROWS.with(|slot| {
        assert!(slot.get().is_none());
        slot.set(Some(hook));
    });
}
fn after_rows(path: &Path) {
    #[cfg(test)]
    AFTER_ROWS.with(|slot| {
        if let Some(hook) = slot.take() {
            hook(path);
        }
    });
    #[cfg(not(test))]
    let _ = path;
}
fn names(path: &Path) -> io::Result<Vec<String>> {
    let mut names = std::fs::read_dir(path)?
        .map(|row| row.and_then(|r| r.file_name().into_string().map_err(|_| invalid())))
        .collect::<io::Result<Vec<_>>>()?;
    names.sort();
    Ok(names)
}
fn raw(file: &mut File, max: usize) -> io::Result<Vec<u8>> {
    if file.metadata()?.len() > max as u64 {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.take(max as u64 + 1).read_to_end(&mut raw)?;
    if raw.len() > max || raw.len() as u64 != file.metadata()?.len() {
        return Err(invalid());
    }
    Ok(raw)
}
fn file_row(
    file: &mut File,
    path: &Path,
    expected: &[u8],
    baseline: Option<&SnapshotData>,
    max: usize,
    end: Instant,
) -> io::Result<serde_json::Value> {
    before(end)?;
    let slot = match path.file_name().and_then(|name| name.to_str()) {
        Some("manifest.json") => "manifest",
        Some("current.json") => "current",
        _ => return Err(invalid()),
    };
    native::verify_private_file(file)?;
    tree::streams(path, false)?;
    let original_id = identity(file)?;
    let observed = observe(path)?;
    if identity(&observed)? != original_id {
        return Err(invalid());
    }
    let sd = security(file, end)?;
    if security(&observed, end)? != sd || baseline.is_some_and(|old| *old != sd) {
        return Err(invalid());
    }
    let first = raw(file, max)?;
    // Read actual partial bytes; no missing result/short write is promoted to
    // the expected complete bytes. Unexpected bytes always retain Unknown.
    if !expected.starts_with(&first) {
        return Err(invalid());
    }
    let second = raw(file, max)?;
    tree::streams(path, false)?;
    if first != second
        || identity(file)? != original_id
        || identity(&observed)? != original_id
        || security(file, end)? != sd
        || security(&observed, end)? != sd
    {
        return Err(invalid());
    }
    before(end)?;
    Ok(
        serde_json::json!({"objectSlot":slot,"pathDigest":projection_digest(&path)?,"identityDigest":projection_digest(&(original_id.volume,original_id.index.to_string()))?,
        "observedSecurityDigest":projection_digest(&sd)?,"originalSecurityBaselineVerified":baseline.is_some(),
        "contentDigest":hash::sha256_hex(&first),"bytes":first.len(),"expectedBytes":expected.len(),
        "writeState":if first.is_empty(){"empty"}else if first.len()==expected.len(){"completeBytes"}else{"partialBytes"},"expectedPrefixMatched":true}),
    )
}
fn record_row(record: &mut Record, end: Instant) -> io::Result<serde_json::Value> {
    let expected = &record.raw;
    let path = &record.path;
    let baseline = &record.security;
    match &mut record.file {
        RecordFile::Original {
            file,
            parent,
            id,
            max,
        } => {
            parent.recheck()?;
            if identity(file)? != *id {
                return Err(invalid());
            }
            let result = file_row(file, path, expected, Some(baseline), *max, end)?;
            parent.recheck()?;
            if result["writeState"] != "completeBytes" {
                return Err(invalid());
            }
            Ok(result)
        }
        // Preparation initializes only original CREATE_NEW handles; a sealed
        // later-record backend is not adopted into a preparation permission.
        RecordFile::Sealed(_) => Err(invalid()),
    }
}
fn partial_rows(
    manifest: &mut Option<File>,
    record: &mut Option<Record>,
    current: &mut Option<File>,
    manifest_expected: Option<&[u8]>,
    current_expected: Option<&[u8]>,
    path: &Path,
    end: Instant,
) -> io::Result<Vec<serde_json::Value>> {
    let mut rows = Vec::new();
    if let Some(file) = manifest {
        rows.push(file_row(
            file,
            &path.join("manifest.json"),
            manifest_expected.ok_or_else(invalid)?,
            None,
            1024 * 1024,
            end,
        )?);
    }
    if let Some(record) = record {
        rows.push(record_row(record, end)?);
    }
    if let Some(file) = current {
        rows.push(file_row(
            file,
            &path.join("current.json"),
            current_expected.ok_or_else(invalid)?,
            None,
            4096,
            end,
        )?);
    }
    before(end)?;
    Ok(rows)
}
fn read_initial_journal(
    journal: &mut Journal,
    end: Instant,
) -> io::Result<(serde_json::Value, Vec<serde_json::Value>)> {
    before(end)?;
    journal.directory.recheck()?;
    journal.peer.verify()?;
    journal
        .manifest
        .binding
        .check_native(&journal.peer, &journal.pins)?;
    if !journal.events.is_empty()
        || journal.progress.sequence != 0
        || journal.pending_source_action.is_some()
        || journal.stop_completion.is_some()
        || journal.close_completion.is_some()
    {
        return Err(invalid());
    }
    let path = journal.directory.path();
    let expected = vec!["current.json".to_string(), "manifest.json".to_string()];
    if names(path)? != expected
        || directory_security(&journal.directory, end)? != journal.directory_security
    {
        return Err(invalid());
    }
    tree::streams(path, true)?;
    let manifest = record_row(&mut journal.manifest_record, end)?;
    if identity(&journal.current)? != journal.current_id {
        return Err(invalid());
    }
    let current = file_row(
        &mut journal.current,
        &path.join("current.json"),
        &journal.current_raw,
        Some(&journal.current_security),
        4096,
        end,
    )?;
    if current["writeState"] != "completeBytes" {
        return Err(invalid());
    }
    after_rows(path);
    let manifest_again = record_row(&mut journal.manifest_record, end)?;
    let current_again = file_row(
        &mut journal.current,
        &path.join("current.json"),
        &journal.current_raw,
        Some(&journal.current_security),
        4096,
        end,
    )?;
    if manifest != manifest_again || current != current_again {
        return Err(invalid());
    }
    if names(path)? != expected
        || directory_security(&journal.directory, end)? != journal.directory_security
    {
        return Err(invalid());
    }
    journal
        .manifest
        .binding
        .check_native(&journal.peer, &journal.pins)?;
    before(end)?;
    Ok((
        serde_json::json!({"members":expected,"directoryPathDigest":projection_digest(&("originalJournalPreparation", path))?,"directoryIdentityDigest":projection_digest(&(journal.directory.identity().volume,journal.directory.identity().index.to_string()))?,"observedDirectorySecurityDigest":projection_digest(&journal.directory_security)?,"fullInitialJournal":true}),
        vec![manifest, current],
    ))
}
impl JournalPreparation {
    fn native_prefix_read(&mut self, end: Instant) -> io::Result<serde_json::Value> {
        before(end)?;
        if !self.attempted {
            return Err(invalid());
        }
        if let Some(journal) = &mut self.completed {
            if self.directory.is_some()
                || self.pins.is_some()
                || self.peer.is_some()
                || self.manifest_record.is_some()
                || self.manifest_open.is_some()
                || self.current_open.is_some()
            {
                return Err(invalid());
            }
            let (directory, objects) = read_initial_journal(journal, end)?;
            return Ok(
                serde_json::json!({"directory":directory,"objects":objects,"membership":self.retained_v2_membership_data(),"pending":true,"runtimeAuthority":false}),
            );
        }
        let directory = self.directory.as_ref().ok_or_else(invalid)?;
        let path = directory.path().to_path_buf();
        directory.recheck()?;
        let pins = self.pins.as_ref().ok_or_else(invalid)?;
        for pin in pins {
            pin.recheck()?;
        }
        let peer = self.peer.as_ref().ok_or_else(invalid)?;
        peer.verify()?;
        let binding = self
            .binding
            .as_ref()
            .or_else(|| self.manifest.as_ref().map(|m| &m.binding))
            .ok_or_else(invalid)?;
        binding.check_native(peer, pins)?;
        if self.manifest_open.is_some() && self.manifest_record.is_some() {
            return Err(invalid());
        }
        if self.manifest_open.is_some() != self.manifest_raw.is_some() {
            return Err(invalid());
        }
        if self.current_open.is_some() && self.current_expected.is_none() {
            return Err(invalid());
        }
        let mut expected = Vec::new();
        if self.manifest_open.is_some() || self.manifest_record.is_some() {
            expected.push("manifest.json".to_string());
        }
        if self.current_open.is_some() {
            expected.push("current.json".to_string());
        }
        expected.sort();
        tree::streams(&path, true)?;
        let members = names(&path)?;
        if members != expected {
            return Err(invalid());
        }
        let directory_sd = directory_security(directory, end)?;
        let objects = partial_rows(
            &mut self.manifest_open,
            &mut self.manifest_record,
            &mut self.current_open,
            self.manifest_raw.as_deref(),
            self.current_expected.as_deref(),
            &path,
            end,
        )?;
        after_rows(&path);
        let again = partial_rows(
            &mut self.manifest_open,
            &mut self.manifest_record,
            &mut self.current_open,
            self.manifest_raw.as_deref(),
            self.current_expected.as_deref(),
            &path,
            end,
        )?;
        if objects != again {
            return Err(invalid());
        }
        tree::streams(&path, true)?;
        directory.recheck()?;
        for pin in pins {
            pin.recheck()?;
        }
        binding.check_native(peer, pins)?;
        if names(&path)? != members || directory_security(directory, end)? != directory_sd {
            return Err(invalid());
        }
        before(end)?;
        Ok(
            serde_json::json!({"directory":{"members":members,"directoryPathDigest":projection_digest(&("originalJournalPreparation", &path))?,"directoryIdentityDigest":projection_digest(&(directory.identity().volume,directory.identity().index.to_string()))?,"observedDirectorySecurityDigest":projection_digest(&directory_sd)?,"fullInitialJournal":false},"objects":objects,"membership":self.retained_v2_membership_data(),"pending":true,"runtimeAuthority":false}),
        )
    }
    pub(in crate::infrastructure::windows_client_originals024) fn read_v2_retained_native(
        &mut self,
        end: Instant,
    ) -> io::Result<serde_json::Value> {
        // Failure is monotonic for restoration. A successful read never clears
        // prior failed/poison or turns a pending native write into completion.
        let result = self.native_prefix_read(end);
        if result.is_err() {
            self.poison();
        }
        result
    }
    pub(in crate::infrastructure::windows_client_originals024) fn v2_native_objects(
        &mut self,
        session: uuid::Uuid,
        end: Instant,
    ) -> io::Result<(Vec<Object>, serde_json::Value)> {
        let data = self.read_v2_retained_native(end)?;
        let rows = data["objects"].as_array().ok_or_else(invalid)?;
        let mut objects = Vec::new();
        for row in rows {
            before(end)?;
            let ordinal = match row["objectSlot"].as_str() {
                Some("manifest") => 0,
                Some("current") => 1,
                _ => return Err(invalid()),
            };
            objects.push(retained_object(
                session,
                Role::Journal,
                ordinal,
                Stage::HandleRetained,
                row["pathDigest"].as_str().ok_or_else(invalid)?.to_string(),
                Some(
                    row["identityDigest"]
                        .as_str()
                        .ok_or_else(invalid)?
                        .to_string(),
                ),
                if row["originalSecurityBaselineVerified"] == true {
                    Some(
                        row["observedSecurityDigest"]
                            .as_str()
                            .ok_or_else(invalid)?
                            .to_string(),
                    )
                } else {
                    None
                },
                Some(
                    row["contentDigest"]
                        .as_str()
                        .ok_or_else(invalid)?
                        .to_string(),
                ),
            )?);
        }
        // Include the original directory/pins/peer/metadata aggregate as a
        // separate pending native member. Its locator follows the retained
        // directory path, while the observation carries changing membership.
        let directory = &data["directory"];
        objects.push(retained_object(
            session,
            Role::Journal,
            2,
            Stage::HandleRetained,
            directory["directoryPathDigest"]
                .as_str()
                .ok_or_else(invalid)?
                .into(),
            Some(
                directory["directoryIdentityDigest"]
                    .as_str()
                    .ok_or_else(invalid)?
                    .into(),
            ),
            None,
            None,
        )?);
        before(end)?;
        Ok((objects, data))
    }
}
