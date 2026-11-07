//! Read-only graph observation. A stable finite listing is an observation, never
//! a server CAS or a proof that no hidden/late writer exists.
use super::{
    store::{canonical, decode, digest, Seen, Store},
    types::*,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Duration,
};
use tokio::sync::Semaphore;

pub struct Network<'a> {
    pub transport: &'a dyn ConfigSyncTransport,
    pub connection: &'a ConnectionHandle,
    pub limits: &'a Budgets,
    pub slots: &'a Semaphore,
}
impl Network<'_> {
    pub async fn list(&self) -> SyncResult<BTreeSet<String>> {
        #[cfg(all(test, windows))]
        let _timing = super::TestPhaseTiming::start("network-list");
        let _permit = self
            .slots
            .acquire()
            .await
            .map_err(|_| failure(ErrorCode::SyncNetwork, Step::Observe))?;
        let response = tokio::time::timeout(
            Duration::from_millis(self.limits.request_ms),
            self.transport.list_children(
                self.connection,
                ListLimits {
                    max_entries: self.limits.listing_entries as u32,
                    max_xml_bytes: self.limits.listing_bytes as u64,
                    timeout_ms: self.limits.request_ms as u32,
                },
            ),
        )
        .await
        .map_err(|_| failure(ErrorCode::SyncNetwork, Step::Observe))?
        .map_err(|e| transport_failure(e, Step::Observe))?;
        if response.http_status != 207 {
            return Err(failure(ErrorCode::SyncListingIncomplete, Step::Observe));
        }
        if response.response_bytes > self.limits.listing_bytes
            || response.entries.len() + 1 > self.limits.listing_entries
        {
            return Err(failure(ErrorCode::SyncLimit, Step::Observe));
        }
        let mut names = BTreeSet::new();
        for e in response.entries {
            if e.is_collection
                || e.name.is_empty()
                || e.name.len() > 256
                || e.name.contains(['/', '\\', '?', '#', '%'])
                || !names.insert(e.name)
            {
                return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
            }
        }
        Ok(names)
    }
    pub async fn get(
        &self,
        path: &SyncRelativePath,
        max: usize,
        step: Step,
    ) -> SyncResult<Vec<u8>> {
        #[cfg(all(test, windows))]
        let _timing = super::TestPhaseTiming::start("network-get");
        let _permit = self
            .slots
            .acquire()
            .await
            .map_err(|_| failure(ErrorCode::SyncNetwork, step))?;
        let result = tokio::time::timeout(
            Duration::from_millis(self.limits.request_ms),
            self.transport.get_file(
                self.connection,
                path,
                ReadLimits {
                    max_bytes: max as u64,
                    timeout_ms: self.limits.request_ms as u32,
                },
            ),
        )
        .await;
        let response = result
            .map_err(|_| failure(ErrorCode::SyncNetwork, step))?
            .map_err(|e| transport_failure(e, step))?;
        if response.http_status != 200 {
            return Err(failure(ErrorCode::SyncListingIncomplete, step));
        }
        if response.body.len() > max {
            return Err(failure(ErrorCode::SyncLimit, step));
        }
        Ok(response.body)
    }
    pub async fn put(&self, path: &SyncRelativePath, bytes: &[u8], step: Step) -> SyncResult<()> {
        #[cfg(all(test, windows))]
        let _timing = super::TestPhaseTiming::start("network-put");
        let max = if step == Step::PayloadPut {
            self.limits.payload_bytes
        } else {
            self.limits.metadata_bytes
        };
        if bytes.len() > max {
            return Err(failure(ErrorCode::SyncLimit, step));
        }
        let _permit = self
            .slots
            .acquire()
            .await
            .map_err(|_| failure(ErrorCode::SyncNetwork, step))?;
        let response = tokio::time::timeout(
            Duration::from_millis(self.limits.request_ms),
            self.transport.put_file(
                self.connection,
                path,
                bytes,
                WriteLimits {
                    max_bytes: max as u64,
                    timeout_ms: self.limits.request_ms as u32,
                },
            ),
        )
        .await
        .map_err(|_| failure(ErrorCode::SyncNetwork, step))?
        .map_err(|e| transport_failure(e, step))?;
        if ![200, 201, 204].contains(&response.http_status) {
            return Err(failure(ErrorCode::SyncNetwork, step));
        }
        Ok(())
    }
    pub async fn get_retry(
        &self,
        path: &SyncRelativePath,
        max: usize,
        step: Step,
    ) -> SyncResult<Vec<u8>> {
        for n in 0..self.limits.get_attempts {
            match self.get(path, max, step).await {
                Err(e)
                    if e.code == ErrorCode::SyncListingIncomplete
                        && n + 1 < self.limits.get_attempts =>
                {
                    tokio::time::sleep(Duration::from_millis(if n == 0 { 100 } else { 300 })).await
                }
                r => return r,
            }
        }
        Err(failure(ErrorCode::SyncListingIncomplete, step))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileKey {
    Generation(String),
    Commit(String, String),
    Data(String, String),
}
pub fn file_key(name: &str) -> SyncResult<FileKey> {
    if let Some(g) = name
        .strip_prefix("config-v2-generation-")
        .and_then(|s| s.strip_suffix(".json"))
    {
        require_uuid(g)?;
        return Ok(FileKey::Generation(g.into()));
    }
    let (v, data) = if let Some(s) = name
        .strip_prefix("config-v2-commit-")
        .and_then(|s| s.strip_suffix(".json"))
    {
        (s, false)
    } else if let Some(s) = name
        .strip_prefix("config-v2-data-")
        .and_then(|s| s.strip_suffix(".ocxdconf"))
    {
        (s, true)
    } else {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
    };
    if !v.is_ascii() || v.len() != 73 || &v[36..37] != "-" {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
    }
    let (g, o) = (&v[..36], &v[37..]);
    require_uuid(g)?;
    require_uuid(o)?;
    Ok(if data {
        FileKey::Data(g.into(), o.into())
    } else {
        FileKey::Commit(g.into(), o.into())
    })
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct QualifiedRef {
    generation_id: String,
    operation_id: String,
    digest: String,
}
#[derive(Clone)]
pub struct Node {
    pub commit: Commit,
    pub bytes: Vec<u8>,
    pub reference: CommitRef,
}
pub struct Graph {
    pub observation: RemoteObservation,
    pub nodes: BTreeMap<(String, String), Node>,
    pub generation_bytes: BTreeMap<String, Vec<u8>>,
}
impl Graph {
    pub fn ordinary(&self, allow_conflict: bool) -> SyncResult<()> {
        let code = match self.observation.state {
            RemoteState::Empty | RemoteState::GenerationEmpty | RemoteState::Ready => return Ok(()),
            RemoteState::Conflict if allow_conflict => return Ok(()),
            RemoteState::Conflict => ErrorCode::SyncRemoteConflict,
            RemoteState::InitializationAmbiguous => ErrorCode::SyncGenerationAmbiguous,
            RemoteState::Incomplete => {
                return Err(self
                    .observation
                    .problems
                    .first()
                    .cloned()
                    .unwrap_or_else(|| failure(ErrorCode::SyncListingIncomplete, Step::Observe)))
            }
            RemoteState::Invalid => ErrorCode::SyncRemoteInvalid,
        };
        Err(failure(code, Step::Observe))
    }
    pub fn node(&self, g: &GenerationRef, h: &CommitRef) -> SyncResult<&Node> {
        if !self.observation.generations.contains(g) {
            return Err(failure(ErrorCode::SyncGenerationChanged, Step::Observe));
        }
        let n = self
            .nodes
            .get(&(g.generation_id.clone(), h.operation_id.clone()))
            .ok_or_else(|| failure(ErrorCode::SyncListingIncomplete, Step::Observe))?;
        if n.reference != *h {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
        }
        Ok(n)
    }
    pub fn remote_relation(&self, g: &GenerationRef, o: &str) -> RemoteRelation {
        if self.observation.generations.len() > 1 {
            return RemoteRelation::InitializationAmbiguous;
        }
        if self.observation.generation.as_ref() != Some(g) {
            return RemoteRelation::GenerationChanged;
        }
        if self.observation.state == RemoteState::Conflict {
            return RemoteRelation::Conflict;
        }
        if self.observation.state != RemoteState::Ready {
            return RemoteRelation::Unknown;
        }
        let h = &self.observation.heads[0];
        if h.operation_id == o {
            return RemoteRelation::ObservedEqual;
        }
        let mut todo = vec![h.operation_id.clone()];
        let mut seen = BTreeSet::new();
        while let Some(id) = todo.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if id == o {
                return RemoteRelation::Descendant;
            }
            if let Some(n) = self.nodes.get(&(g.generation_id.clone(), id)) {
                todo.extend(n.commit.parents.iter().map(|p| p.operation_id.clone()));
            }
        }
        RemoteRelation::Unknown
    }
}
struct Meter {
    requests: usize,
    bytes: usize,
}
impl Meter {
    fn request(&mut self, b: &Budgets) -> SyncResult<()> {
        self.requests += 1;
        if self.requests > b.requests {
            Err(failure(ErrorCode::SyncLimit, Step::Observe))
        } else {
            Ok(())
        }
    }
    fn bytes(&mut self, n: usize, b: &Budgets) -> SyncResult<()> {
        self.bytes = self.bytes.saturating_add(n);
        if self.bytes > b.observation_bytes {
            Err(failure(ErrorCode::SyncLimit, Step::Observe))
        } else {
            Ok(())
        }
    }
    async fn get(&mut self, net: &Network<'_>, p: &SyncRelativePath) -> SyncResult<Vec<u8>> {
        for n in 0..net.limits.get_attempts {
            self.request(net.limits)?;
            match net.get(p, net.limits.metadata_bytes, Step::Observe).await {
                Ok(bytes) => {
                    self.bytes(bytes.len(), net.limits)?;
                    return Ok(bytes);
                }
                Err(e)
                    if e.code == ErrorCode::SyncListingIncomplete
                        && n + 1 < net.limits.get_attempts =>
                {
                    tokio::time::sleep(Duration::from_millis(if n == 0 { 100 } else { 300 })).await
                }
                Err(e) => return Err(e),
            }
        }
        Err(failure(ErrorCode::SyncListingIncomplete, Step::Observe))
    }
}
pub fn validate_commit(c: &Commit, g: &str, id: &str, b: &Budgets) -> SyncResult<()> {
    require_uuid(g)?;
    require_uuid(id)?;
    require_digest(&c.payload_digest)?;
    if c.format != "OCXD_CONFIG_COMMIT"
        || c.format_version != 1
        || c.generation_id != g
        || c.operation_id != id
        || c.payload_bytes == 0
        || c.payload_bytes > b.payload_bytes as u64
        || c.parents.len() > b.heads
    {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
    }
    let mut prev = None;
    for p in &c.parents {
        require_uuid(&p.operation_id)?;
        require_digest(&p.digest)?;
        if prev.is_some_and(|v: &str| v >= p.operation_id.as_str()) || p.operation_id == id {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
        }
        prev = Some(&p.operation_id);
    }
    match c.intent {
        Intent::Update if c.parents.len() > 1 || c.resolution.is_some() => {
            return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode))
        }
        Intent::Resolve => {
            let r = c
                .resolution
                .as_ref()
                .ok_or_else(|| failure(ErrorCode::SyncRemoteInvalid, Step::Decode))?;
            require_digest(&r.scope_digest)?;
            require_digest(&r.preview_digest)?;
            if c.parents.len() < 2
                || (r.selection_mode == SelectionMode::Head
                    && r.selected_head
                        .as_ref()
                        .is_none_or(|h| !c.parents.contains(h)))
                || (r.selection_mode == SelectionMode::Merged && r.selected_head.is_some())
            {
                return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
            }
        }
        _ => (),
    }
    Ok(())
}
fn validate_branch(
    nodes: &BTreeMap<(String, String), Node>,
    g: &str,
    b: &Budgets,
) -> SyncResult<Vec<CommitRef>> {
    let branch: BTreeMap<_, _> = nodes
        .iter()
        .filter(|((gid, _), _)| gid == g)
        .map(|((_, id), n)| (id.clone(), n))
        .collect();
    if branch.len() > b.commits {
        return Err(failure(ErrorCode::SyncLimit, Step::Observe));
    }
    let mut children: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut indegrees = BTreeMap::new();
    let mut parents = BTreeSet::new();
    for (id, n) in &branch {
        indegrees.insert(id.clone(), n.commit.parents.len());
        for p in &n.commit.parents {
            let parent = branch
                .get(&p.operation_id)
                .ok_or_else(|| failure(ErrorCode::SyncListingIncomplete, Step::Observe))?;
            if parent.reference != *p {
                return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
            }
            parents.insert(p.operation_id.clone());
            children
                .entry(p.operation_id.clone())
                .or_default()
                .push(id.clone());
        }
    }
    let mut roots: VecDeque<_> = indegrees
        .iter()
        .filter(|(_, v)| **v == 0)
        .map(|(k, _)| k.clone())
        .collect();
    let mut count = 0;
    while let Some(id) = roots.pop_front() {
        count += 1;
        if let Some(cs) = children.get(&id) {
            for c in cs {
                let v = indegrees.get_mut(c).expect("known node");
                *v -= 1;
                if *v == 0 {
                    roots.push_back(c.clone());
                }
            }
        }
    }
    if count != branch.len() {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
    }
    let heads: Vec<_> = branch
        .iter()
        .filter(|(id, _)| !parents.contains(*id))
        .map(|(_, n)| n.reference.clone())
        .collect();
    if heads.len() > b.heads {
        return Err(failure(ErrorCode::SyncLimit, Step::Observe));
    }
    Ok(heads)
}
pub async fn observe(net: &Network<'_>, store: &Store) -> SyncResult<Graph> {
    #[cfg(all(test, windows))]
    let _timing = super::TestPhaseTiming::start("graph-observe-with-store-lock");
    net.limits.validate()?;
    require_digest(&net.connection.binding_id)?;
    let key = format!("seen-{}", &net.connection.binding_id[7..]);
    let _guard = store.lock(&key).await?;
    tokio::time::timeout(
        Duration::from_millis(net.limits.observation_ms),
        observe_inner(net, store),
    )
    .await
    .map_err(|_| failure(ErrorCode::SyncListingIncomplete, Step::Observe))?
}
async fn observe_inner(net: &Network<'_>, store: &Store) -> SyncResult<Graph> {
    let seen = store.seen(&net.connection.binding_id)?;
    let mut meter = Meter {
        requests: 0,
        bytes: 0,
    };
    let mut observed_names = BTreeSet::new();
    for _ in 0..net.limits.listing_rounds {
        meter.request(net.limits)?;
        let actual = net.list().await?;
        observed_names.extend(actual.clone());
        let mut names = observed_names.clone();
        let mut gids = BTreeSet::new();
        let mut problems = Vec::new();
        for name in &names {
            match file_key(name) {
                Ok(FileKey::Generation(g)) => {
                    gids.insert(g);
                }
                Ok(_) => (),
                Err(e) => problems.push(e),
            }
        }
        if gids.len() > net.limits.generations {
            return Err(failure(ErrorCode::SyncLimit, Step::Observe));
        }
        for name in &seen.names {
            if match file_key(name) {
                Ok(FileKey::Commit(g, _)) | Ok(FileKey::Data(g, _)) => gids.contains(&g),
                _ => false,
            } {
                names.insert(name.clone());
            }
        }
        if gids.is_empty() && !seen.generations.is_empty() {
            problems.push(failure(ErrorCode::SyncListingIncomplete, Step::Observe));
        }
        let mut generations = Vec::new();
        let mut generation_bytes = BTreeMap::new();
        let mut nodes = BTreeMap::new();
        let mut candidates = Vec::new();
        let mut allheads = Vec::new();
        for g in &gids {
            let branch = async {
                let bytes = meter.get(net, &SyncRelativePath::generation(g)?).await?;
                let marker: Generation = decode(&bytes)?;
                if marker.format != "OCXD_CONFIG_GENERATION"
                    || marker.format_version != 1
                    || marker.generation_id != *g
                {
                    return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe));
                }
                let gr = GenerationRef {
                    generation_id: g.clone(),
                    digest: digest(&bytes),
                };
                if seen
                    .generations
                    .iter()
                    .any(|old| old.generation_id == *g && old.digest != gr.digest)
                {
                    return Err(failure(ErrorCode::SyncGenerationChanged, Step::Observe));
                }
                generation_bytes.insert(g.clone(), bytes);
                generations.push(gr.clone());
                let mut todo: VecDeque<String> = names
                    .iter()
                    .filter_map(|name| match file_key(name) {
                        Ok(FileKey::Commit(gid, id)) if gid == *g => Some(id),
                        _ => None,
                    })
                    .collect();
                let mut loaded = BTreeSet::new();
                while let Some(id) = todo.pop_front() {
                    if !loaded.insert(id.clone()) {
                        continue;
                    }
                    if loaded.len() > net.limits.commits {
                        return Err(failure(ErrorCode::SyncLimit, Step::Observe));
                    }
                    let path = SyncRelativePath::commit(g, &id)?;
                    names.insert(path.as_str().into());
                    let bytes = meter.get(net, &path).await?;
                    if seen
                        .commits
                        .get(path.as_str())
                        .is_some_and(|old| *old != digest(&bytes))
                    {
                        return Err(failure(ErrorCode::SyncOperationCollision, Step::Observe));
                    }
                    let c: Commit = decode(&bytes)?;
                    validate_commit(&c, g, &id, net.limits)?;
                    todo.extend(c.parents.iter().map(|p| p.operation_id.clone()));
                    let reference = CommitRef {
                        operation_id: id.clone(),
                        digest: digest(&bytes),
                    };
                    nodes.insert(
                        (g.clone(), id),
                        Node {
                            commit: c,
                            bytes,
                            reference,
                        },
                    );
                }
                let hs = validate_branch(&nodes, g, net.limits)?;
                for h in hs {
                    allheads.push(QualifiedRef {
                        generation_id: g.clone(),
                        operation_id: h.operation_id.clone(),
                        digest: h.digest.clone(),
                    });
                    candidates.push(RecoveryCandidate {
                        generation: gr.clone(),
                        head: h,
                    });
                }
                Ok::<_, SyncFailure>(())
            }
            .await;
            if let Err(e) = branch {
                if e.code == ErrorCode::SyncLimit {
                    return Err(e);
                }
                problems.push(e);
            }
        }
        meter.request(net.limits)?;
        let after = net.list().await?;
        if after != actual {
            observed_names.extend(after);
            continue;
        }
        let ordinary_g = if gids.len() == 1 && generations.len() == 1 {
            generations.first().cloned()
        } else {
            None
        };
        let mut heads: Vec<_> = candidates
            .iter()
            .filter(|c| Some(&c.generation) == ordinary_g.as_ref())
            .map(|c| c.head.clone())
            .collect();
        heads.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
        let state = if gids.len() > 1 {
            RemoteState::InitializationAmbiguous
        } else if !problems.is_empty() {
            if problems.iter().any(|e| {
                matches!(
                    e.code,
                    ErrorCode::SyncRemoteInvalid
                        | ErrorCode::SyncGenerationChanged
                        | ErrorCode::SyncOperationCollision
                )
            }) {
                RemoteState::Invalid
            } else {
                RemoteState::Incomplete
            }
        } else if gids.is_empty() {
            if names.is_empty() {
                RemoteState::Empty
            } else {
                RemoteState::Invalid
            }
        } else if heads.is_empty() {
            RemoteState::GenerationEmpty
        } else if heads.len() == 1 {
            RemoteState::Ready
        } else {
            RemoteState::Conflict
        };
        if state == RemoteState::InitializationAmbiguous {
            problems.insert(
                0,
                failure(ErrorCode::SyncGenerationAmbiguous, Step::Observe),
            );
        }
        generations.sort_by(|a, b| a.generation_id.cmp(&b.generation_id));
        let mut qualified: Vec<_> = nodes
            .values()
            .map(|n| QualifiedRef {
                generation_id: n.commit.generation_id.clone(),
                operation_id: n.commit.operation_id.clone(),
                digest: n.reference.digest.clone(),
            })
            .collect();
        qualified.sort_by(|a, b| {
            (&a.generation_id, &a.operation_id).cmp(&(&b.generation_id, &b.operation_id))
        });
        allheads.sort_by(|a, b| {
            (&a.generation_id, &a.operation_id).cmp(&(&b.generation_id, &b.operation_id))
        });
        // Repeated malformed names must not produce an unbounded public error
        // array. Preserve each distinct stable reason/step; file names remain
        // fully bound in the view digest and the scan is still invalid.
        problems.sort_by_key(|p| (p.code, p.step));
        problems.dedup_by_key(|p| (p.code, p.step));
        for p in &mut problems {
            p.recovery_actions.retain(|a| {
                !matches!(
                    a.kind,
                    RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
                )
            });
        }
        let mut ps: Vec<_> = problems
            .iter()
            .map(|p| {
                (
                    serde_json::to_value(p.code).unwrap(),
                    serde_json::to_value(p.step).unwrap(),
                )
            })
            .collect();
        ps.sort_by_key(|p| (p.0.to_string(), p.1.to_string()));
        ps.dedup();
        let ps: Vec<_> = ps
            .into_iter()
            .map(|(code, step)| serde_json::json!({"code":code,"step":step}))
            .collect();
        let view_digest = digest(&canonical(
            &serde_json::json!({"hashDomain":"OCXD_CONFIG_VIEW_V1","bindingId":net.connection.binding_id,"state":state,"generations":generations,"commits":qualified,"heads":allheads,"fileNames":names,"problems":ps}),
        )?);
        let mut orphan_count = 0;
        let mut stale_count = 0;
        for name in &names {
            match file_key(name) {
                Ok(FileKey::Data(g, o)) => {
                    if !gids.contains(&g) {
                        stale_count += 1
                    } else if !nodes.contains_key(&(g, o)) {
                        orphan_count += 1
                    }
                }
                Ok(FileKey::Commit(g, _)) if !gids.contains(&g) => stale_count += 1,
                _ => (),
            }
        }
        let recovery_actions = if state == RemoteState::InitializationAmbiguous {
            vec![
                action(RecoveryKind::PrepareRecoveryDownload),
                action(RecoveryKind::PrepareRecoveryExport),
                action(RecoveryKind::SelectEmptyDirectory),
            ]
        } else if state == RemoteState::Conflict {
            vec![action(RecoveryKind::ChooseConflictSource)]
        } else {
            vec![]
        };
        // No partial/invalid scan is promoted to a new trustworthy seen baseline.
        if matches!(
            state,
            RemoteState::Empty
                | RemoteState::GenerationEmpty
                | RemoteState::Ready
                | RemoteState::Conflict
                | RemoteState::InitializationAmbiguous
        ) {
            store.save_seen(
                &net.connection.binding_id,
                &Seen {
                    generations: generations.clone(),
                    names: names.clone(),
                    commits: nodes
                        .values()
                        .map(|n| {
                            (
                                SyncRelativePath::commit(
                                    &n.commit.generation_id,
                                    &n.commit.operation_id,
                                )
                                .expect("validated UUID")
                                .as_str()
                                .to_string(),
                                n.reference.digest.clone(),
                            )
                        })
                        .collect(),
                },
            )?;
        }
        return Ok(Graph {
            observation: RemoteObservation {
                dto_version: 1,
                binding_id: net.connection.binding_id.clone(),
                state,
                generation: ordinary_g,
                generations,
                heads: if gids.len() > 1 {
                    vec![]
                } else {
                    std::mem::take(&mut heads)
                },
                orphan_count,
                stale_count,
                view_digest,
                observed_at: chrono::Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                problems,
                recovery_candidates: candidates,
                recovery_actions,
            },
            nodes,
            generation_bytes,
        });
    }
    Err(failure(ErrorCode::SyncListingIncomplete, Step::Observe))
}
