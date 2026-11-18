//! Native external-repair component tests with three synthetic originals.
use super::*;
use crate::infrastructure::hash::sha256_hex;
use crate::infrastructure::windows_client_originals024::tree::streams;
#[path = "external_repair_record.rs"]
mod repair_record;
use repair_record::{CompletionObservations, Plan, Progress};
use std::path::PathBuf;

const LEAVES: [&str; 3] = ["auth.json", "config.toml", "config.json"];
const GROUPS: [usize; 3] = [0, 0, 1];

pub(crate) struct WholeTree {
    root: Option<tempfile::TempDir>,
    clients: [PinnedDirectory; 2],
    custody: [PinnedDirectory; 2],
    retired: PinnedDirectory,
    records: PinnedDirectory,
    anchors: [SnapshotData; 2],
    claim_name: &'static str,
    stable: [FileImage; 2],
    originals: [FileImage; 3],
    working: [FileImage; 3],
    started: Instant,
    pub(crate) end: Instant,
}

pub(crate) struct DurableMoves {
    plan: Plan,
    records: Vec<(String, PrivateRecord, Vec<u8>)>,
    full_readback_sha256: String,
}

impl DurableMoves {
    fn raw(&self) -> impl Iterator<Item = &[u8]> {
        self.records.iter().map(|(_, _, bytes)| bytes.as_slice())
    }

    fn reread(&mut self) -> io::Result<()> {
        for (_, record, bytes) in &mut self.records {
            if record.read()? != *bytes {
                return Err(invalid("durable move record changed"));
            }
        }
        Ok(())
    }
}

pub(crate) struct Node<'a> {
    retire: FileStep<'a>,
    restore: FileStep<'a>,
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn observe_directory(dir: &PinnedDirectory, end: Instant) -> io::Result<SnapshotData> {
    check_time(end)?;
    dir.recheck()?;
    streams(dir.path(), true)?;
    let file = OpenOptions::new()
        .access_mode(0x20080)
        .share_mode(7)
        .custom_flags(0x200000 | 0x2000000)
        .open(dir.path())?;
    let stamp = native_stamp(&file, end).map_err(io::Error::other)?;
    let data = serde_json::to_value(&stamp)?;
    if data["fileId"] != dir.identity().index.to_string() || data["volume"] != dir.identity().volume
    {
        return Err(invalid("directory object differs from pinned directory"));
    }
    Ok(stamp)
}

fn observe_file(path: &Path, end: Instant) -> io::Result<FileImage> {
    check_time(end)?;
    streams(path, false)?;
    let image = read_image(path, end)?;
    check_time(end)?;
    Ok(image)
}

fn require_members(dir: &PinnedDirectory, names: &[&str], end: Instant) -> io::Result<()> {
    check_time(end)?;
    dir.recheck()?;
    let mut seen = Vec::with_capacity(names.len());
    for entry in fs::read_dir(dir.path())? {
        check_time(end)?;
        let name = entry?.file_name();
        if !names.iter().any(|expected| name == *expected) || seen.contains(&name) {
            return Err(invalid("whole tree has unexpected member"));
        }
        seen.push(name);
    }
    if seen.len() != names.len() {
        return Err(invalid("whole tree missing expected member"));
    }
    dir.recheck()?;
    check_time(end)
}

impl WholeTree {
    fn new() -> Self {
        Self::with_claim("claim.sqlite", |client, _| {
            fs::write(client.join("claim.sqlite"), b"claim.sqlite").unwrap();
        })
        .0
    }

    pub(crate) fn with_claim<T>(
        claim_name: &'static str,
        seed: impl FnOnce(&Path, &Path) -> T,
    ) -> (Self, T) {
        let root = tempfile::Builder::new()
            .prefix("ocx024-r7-whole-tree-")
            .tempdir()
            .unwrap();
        let make = |name| backend::create_private_directory(&root.path().join(name)).unwrap();
        let clients = [make(".codex"), make(".grok")];
        let custody = [make("codex-custody"), make("grok-custody")];
        let retired = make("retired");
        let records = make("records");
        for (index, leaf) in LEAVES.iter().enumerate() {
            let group = GROUPS[index];
            fs::write(
                custody[group].path().join(leaf),
                format!("original {index}\n"),
            )
            .unwrap();
            fs::write(
                clients[group].path().join(leaf),
                format!("working {index}\n"),
            )
            .unwrap();
        }
        let seeded = seed(clients[0].path(), root.path());
        fs::write(clients[0].path().join("owner.sqlite"), b"owner.sqlite").unwrap();
        let started = Instant::now();
        let end = started + Duration::from_secs(10);
        let anchors = std::array::from_fn(|i| observe_directory(&clients[i], end).unwrap());
        let stable = [claim_name, "owner.sqlite"]
            .map(|leaf| observe_file(&clients[0].path().join(leaf), end).unwrap());
        let originals = std::array::from_fn(|i| {
            observe_file(&custody[GROUPS[i]].path().join(LEAVES[i]), end).unwrap()
        });
        let working = std::array::from_fn(|i| {
            observe_file(&clients[GROUPS[i]].path().join(LEAVES[i]), end).unwrap()
        });
        let scene = Self {
            root: Some(root),
            clients,
            custody,
            retired,
            records,
            anchors,
            claim_name,
            stable,
            originals,
            working,
            started,
            end,
        };
        (scene, seeded)
    }

    fn verify_anchors(&self) -> io::Result<()> {
        for (index, client) in self.clients.iter().enumerate() {
            if observe_directory(client, self.end)? != self.anchors[index] {
                return Err(invalid("stable client directory changed"));
            }
        }
        for (index, leaf) in [self.claim_name, "owner.sqlite"].into_iter().enumerate() {
            if observe_file(&self.clients[0].path().join(leaf), self.end)? != self.stable[index] {
                return Err(invalid("stable file changed"));
            }
        }
        Ok(())
    }

    fn verify_members(
        &self,
        retired: &[&str],
        records: &[&str],
        returned: usize,
    ) -> io::Result<()> {
        require_members(
            &self.clients[0],
            &[self.claim_name, "owner.sqlite", LEAVES[0], LEAVES[1]],
            self.end,
        )?;
        require_members(&self.clients[1], &[LEAVES[2]], self.end)?;
        for (group, dir) in self.custody.iter().enumerate() {
            let expected: Vec<_> = (returned..LEAVES.len())
                .filter(|&index| GROUPS[index] == group)
                .map(|index| LEAVES[index])
                .collect();
            require_members(dir, &expected, self.end)?;
        }
        require_members(&self.retired, retired, self.end)?;
        require_members(&self.records, records, self.end)
    }

    pub(crate) fn prepare(&self) -> io::Result<Vec<Node<'_>>> {
        self.verify_anchors()?;
        self.verify_members(&[], &[], 0)?;
        let mut nodes = Vec::with_capacity(LEAVES.len());
        for (index, leaf) in LEAVES.into_iter().enumerate() {
            let group = GROUPS[index];
            streams(&self.clients[group].path().join(leaf), false)?;
            streams(&self.custody[group].path().join(leaf), false)?;
            let retire = FileStep::retain(
                (&self.clients[group], leaf),
                (&self.retired, leaf),
                &self.working[index],
                "RetireIntent",
                self.end,
            )?;
            let restore = FileStep::retain(
                (&self.custody[group], leaf),
                (&self.clients[group], leaf),
                &self.originals[index],
                "OriginalReturnIntent",
                self.end,
            )?;
            nodes.push(Node { retire, restore });
        }
        self.verify_anchors()?;
        self.verify_members(&[], &[], 0)?;
        Ok(nodes)
    }

    pub(crate) fn run(
        &self,
        mut nodes: Vec<Node<'_>>,
        mut checkpoint: impl FnMut() -> io::Result<()>,
        mut after_intent: impl FnMut(usize, &str, &Path) -> io::Result<()>,
    ) -> io::Result<DurableMoves> {
        self.verify_anchors()?;
        self.verify_members(&[], &[], 0)?;
        for node in &mut nodes {
            node.retire.verify_source(self.end)?;
            node.restore.verify_source(self.end)?;
        }
        checkpoint()?;
        let root = self.root.as_ref().unwrap().path();
        let action_id = sha256_hex(
            root.to_str()
                .ok_or_else(|| invalid("scene path encoding"))?
                .as_bytes(),
        );
        let manifest = serde_json::to_vec(&(
            &self.anchors,
            self.stable
                .each_ref()
                .map(|image| (&image.security, sha256_hex(&image.bytes))),
            LEAVES,
        ))?;
        let current = serde_json::to_vec(&(
            "synthetic interrupted before Resume",
            sha256_hex(&manifest),
            &action_id,
        ))?;
        let plan = Plan::synthetic(
            nodes.iter().flat_map(|node| [&node.retire, &node.restore]),
            &manifest,
            &current,
            action_id,
            self.end.duration_since(self.started),
        )?;
        let mut records: Vec<(String, PrivateRecord, Vec<u8>)> = Vec::with_capacity(12);
        for (index, node) in nodes.iter_mut().enumerate() {
            let names: Vec<_> = records.iter().map(|(name, _, _)| name.as_str()).collect();
            self.verify_members(&LEAVES[..index], &names, index)?;
            for step in [&mut node.retire, &mut node.restore] {
                step.verify_source(self.end)?;
                let source = step.held.location().to_owned();
                let target = step.to.path().join(step.target_name);
                require_absent(&target)?;
                let previous = records.last().map(|(_, _, bytes)| bytes.as_slice());
                let intent = plan.step_record(records.len(), previous)?;
                checkpoint()?;
                self.record(
                    &mut records,
                    format!("{index}-{}.json", step.intent),
                    intent,
                )?;
                after_intent(index, step.intent, &target)?;
                step.verify_source(self.end)?;
                checkpoint()?;
                step.held
                    .move_into(step.to.path(), step.target_name, self.end)?;
                checkpoint()?;
                require_absent(&source)?;
                step.verify_source(self.end)?;
                if observe_file(&target, self.end)? != *step.expected {
                    return Err(invalid("moved node readback differs"));
                }
                let previous = records.last().map(|(_, _, bytes)| bytes.as_slice());
                let readback = plan.step_record(records.len(), previous)?;
                self.record(
                    &mut records,
                    format!("{index}-{}Readback.json", step.intent),
                    readback,
                )?;
                checkpoint()?;
            }
        }
        let names: Vec<_> = records.iter().map(|(name, _, _)| name.as_str()).collect();
        self.verify_members(&LEAVES, &names, LEAVES.len())?;
        self.verify_anchors()?;
        for node in &mut nodes {
            node.retire.verify_source(self.end)?;
            node.restore.verify_source(self.end)?;
        }
        let full_readback_sha256 = sha256_hex(&serde_json::to_vec(&(
            manifest,
            records
                .iter()
                .map(|(_, _, bytes)| sha256_hex(bytes))
                .collect::<Vec<_>>(),
        ))?);
        let mut result = DurableMoves {
            plan,
            records,
            full_readback_sha256,
        };
        result.reread()?;
        if result.plan.verify(result.raw(), None)? != Progress::RepairPending {
            return Err(invalid("file moves alone cannot complete repair"));
        }
        checkpoint()?;
        check_time(self.end)?;
        Ok(result)
    }

    #[allow(dead_code)] // Product-linked test supplies the native lease observations.
    pub(crate) fn record_released(
        &self,
        mut moves: DurableMoves,
        lease_release_sha256: String,
    ) -> io::Result<()> {
        check_time(self.end)?;
        moves.reread()?;
        let names: Vec<_> = moves
            .records
            .iter()
            .map(|(name, _, _)| name.as_str())
            .collect();
        self.verify_members(&LEAVES, &names, LEAVES.len())?;
        self.verify_anchors()?;
        for index in 0..LEAVES.len() {
            let (client, _, retired) = self.node_paths(index);
            if observe_file(&client, self.end)? != self.originals[index] {
                return Err(invalid("returned original changed after lease release"));
            }
            if observe_file(&retired, self.end)? != self.working[index] {
                return Err(invalid("retired working file changed after lease release"));
            }
        }
        let observed = CompletionObservations::synthetic(
            moves.full_readback_sha256.clone(),
            lease_release_sha256,
        );
        let previous = moves
            .records
            .last()
            .ok_or_else(|| invalid("external repair prefix missing"))?;
        let bytes = moves.plan.completion_record(&previous.2, &observed)?;
        self.record(&mut moves.records, "completion.json".into(), bytes)?;
        moves.reread()?;
        let names: Vec<_> = moves
            .records
            .iter()
            .map(|(name, _, _)| name.as_str())
            .collect();
        self.verify_members(&LEAVES, &names, LEAVES.len())?;
        if moves.plan.verify(moves.raw(), Some(&observed))? != Progress::CompleteData {
            return Err(invalid("external repair completion differs"));
        }
        check_time(self.end)
    }

    fn record(
        &self,
        records: &mut Vec<(String, PrivateRecord, Vec<u8>)>,
        name: String,
        bytes: Vec<u8>,
    ) -> io::Result<()> {
        check_time(self.end)?;
        self.records.recheck()?;
        let handle = publish_record(&self.records.path().join(&name), &bytes)?;
        records.push((name, handle, bytes));
        check_time(self.end)
    }

    fn node_paths(&self, index: usize) -> (PathBuf, PathBuf, PathBuf) {
        let leaf = LEAVES[index];
        let group = GROUPS[index];
        (
            self.clients[group].path().join(leaf),
            self.custody[group].path().join(leaf),
            self.retired.path().join(leaf),
        )
    }

    pub(crate) fn finish(mut self) {
        let root = self.root.take().unwrap();
        let path = root.path().to_owned();
        drop(self);
        root.close().unwrap();
        assert!(!path.exists());
        println!("whole tree fixture removed: {}", path.display());
    }
}

impl Drop for WholeTree {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some(root) = self.root.take() {
                eprintln!(
                    "failed whole tree fixture retained: {}",
                    root.keep().display()
                );
            }
        }
    }
}

#[test]
fn r7_external_steps_whole_tree_restores_three_originals_with_durable_records() {
    let scene = WholeTree::new();
    let nodes = scene.prepare().unwrap();
    let moves = scene.run(nodes, || Ok(()), |_, _, _| Ok(())).unwrap();
    for index in 0..LEAVES.len() {
        let (client, custody, retired) = scene.node_paths(index);
        assert!(observe_file(&client, scene.end).unwrap() == scene.originals[index]);
        assert!(observe_file(&retired, scene.end).unwrap() == scene.working[index]);
        require_absent(&custody).unwrap();
    }
    assert_eq!(fs::read_dir(scene.records.path()).unwrap().count(), 12);
    assert_eq!(
        moves.plan.verify(moves.raw().take(7), None).unwrap(),
        Progress::RepairPending
    );
    let mut changed: Vec<_> = moves.raw().map(<[u8]>::to_vec).collect();
    let original = changed[0].clone();
    for field in [
        "failedManifestSha256",
        "failedCurrentSha256",
        "confirmedScopeSha256",
        "quiescenceEvidenceSha256",
        "repairActionId",
        "deadlineNsFromActionStart",
    ] {
        let field_value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        let value = serde_json::to_vec(&field_value["binding"][field]).unwrap();
        let at = original
            .windows(value.len())
            .position(|window| window == value)
            .unwrap();
        let offset = at + 1;
        changed[0][offset] = if original[offset] == b'8' { b'9' } else { b'8' };
        assert!(
            moves
                .plan
                .verify(changed.iter().map(Vec::as_slice), None)
                .is_err(),
            "{field}"
        );
        changed[0].clone_from(&original);
    }
    changed.swap(3, 4);
    assert!(moves
        .plan
        .verify(changed.iter().map(Vec::as_slice), None)
        .is_err());
    changed.swap(3, 4);
    changed[0].pop();
    changed[0].extend_from_slice(br#", "foreign":true}"#);
    assert!(moves
        .plan
        .verify(changed.iter().map(Vec::as_slice), None)
        .is_err());
    changed[0] = original;
    let fabricated = CompletionObservations::synthetic(
        moves.full_readback_sha256.clone(),
        "fabricated release".into(),
    );
    let false_completion = moves
        .plan
        .completion_record(changed.last().unwrap(), &fabricated)
        .unwrap();
    changed.push(false_completion);
    assert!(moves
        .plan
        .verify(changed.iter().map(Vec::as_slice), None)
        .is_err());
    changed.remove(6);
    assert!(moves
        .plan
        .verify(changed.iter().map(Vec::as_slice), Some(&fabricated))
        .is_err());
    drop(moves);
    println!("external repair record refused changed binding, unknown fields, reordered prefix and false completion");
    println!("three native originals returned; four anchors, exact children and twelve flushed intent/readback records verified in one 10 s action");
    scene.finish();
}

#[test]
fn r7_external_steps_whole_tree_late_custody_drift_refuses_before_any_record() {
    let scene = WholeTree::new();
    let nodes = scene.prepare().unwrap();
    let extra = scene.custody[1].path().join("unplanned");
    fs::write(&extra, b"foreign member").unwrap();
    let error = scene
        .run(
            nodes,
            || panic!("refuse before first checkpoint"),
            |_, _, _| panic!("refuse before first intent"),
        )
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "whole tree has unexpected member");
    assert!(fs::read_dir(scene.records.path()).unwrap().next().is_none());
    for index in 0..LEAVES.len() {
        let (client, custody, retired) = scene.node_paths(index);
        assert!(observe_file(&client, scene.end).unwrap() == scene.working[index]);
        assert!(observe_file(&custody, scene.end).unwrap() == scene.originals[index]);
        require_absent(&retired).unwrap();
    }
    assert_eq!(fs::read(extra).unwrap(), b"foreign member");
    println!("second custody drift refused before first intent; all six files unchanged");
    scene.finish();
}

#[test]
fn r7_external_steps_whole_tree_second_return_collision_keeps_exact_native_prefix() {
    let scene = WholeTree::new();
    let nodes = scene.prepare().unwrap();
    let error = scene
        .run(
            nodes,
            || Ok(()),
            |index, intent, target| {
                if index == 1 && intent == "OriginalReturnIntent" {
                    fs::write(target, b"foreign destination")?;
                }
                Ok(())
            },
        )
        .err()
        .unwrap();
    assert!(matches!(error.raw_os_error(), Some(80 | 183)));
    require_members(
        &scene.records,
        &[
            "0-RetireIntent.json",
            "0-RetireIntentReadback.json",
            "0-OriginalReturnIntent.json",
            "0-OriginalReturnIntentReadback.json",
            "1-RetireIntent.json",
            "1-RetireIntentReadback.json",
            "1-OriginalReturnIntent.json",
        ],
        scene.end,
    )
    .unwrap();
    for index in 0..LEAVES.len() {
        let (client, custody, retired) = scene.node_paths(index);
        if index == 0 {
            assert!(observe_file(&client, scene.end).unwrap() == scene.originals[index]);
            require_absent(&custody).unwrap();
        } else {
            assert!(observe_file(&custody, scene.end).unwrap() == scene.originals[index]);
        }
        if index < 2 {
            assert!(observe_file(&retired, scene.end).unwrap() == scene.working[index]);
        } else {
            require_absent(&retired).unwrap();
            assert!(observe_file(&client, scene.end).unwrap() == scene.working[index]);
        }
    }
    assert_eq!(
        fs::read(scene.node_paths(1).0).unwrap(),
        b"foreign destination"
    );
    scene.verify_anchors().unwrap();
    println!(
        "second return collision raw{}; exact seven-record prefix; last node unchanged",
        error.raw_os_error().unwrap()
    );
    scene.finish();
}
