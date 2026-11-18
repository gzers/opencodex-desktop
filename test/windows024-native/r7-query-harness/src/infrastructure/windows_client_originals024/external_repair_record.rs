//! Exact, bounded evidence for the synthetic external repair. This journal
//! neither opens a native object nor changes the failed session's history.
use super::{invalid, FileImage, FileStep};
use crate::infrastructure::hash::sha256_hex;
use serde::Serialize;
use std::{io, time::Duration};

const SCHEMA: &str = "windows-external-preresume-repair-v1";
const STEPS: usize = 6;
const MOVE_RECORDS: usize = 2 * STEPS;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Binding {
    failed_manifest_sha256: String,
    failed_current_sha256: String,
    confirmed_scope_sha256: String,
    quiescence_evidence_sha256: String,
    repair_action_id: String,
    deadline_ns_from_action_start: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Move {
    source: String,
    target: String,
    volume: u32,
    source_parent_id: String,
    target_parent_id: String,
    file_id: String,
    byte_count: usize,
    content_sha256: String,
    security_sha256: String,
}

impl Move {
    fn data(step: &FileStep<'_>) -> io::Result<Self> {
        let source = step.held.location();
        let target = step.to.path().join(step.target_name);
        Ok(Self {
            source: source
                .to_str()
                .ok_or_else(|| invalid("source path encoding"))?
                .into(),
            target: target
                .to_str()
                .ok_or_else(|| invalid("target path encoding"))?
                .into(),
            volume: step.held.identity().volume,
            source_parent_id: step.from.identity().index.to_string(),
            target_parent_id: step.to.identity().index.to_string(),
            file_id: step.held.identity().index.to_string(),
            byte_count: step.expected.bytes.len(),
            content_sha256: sha256_hex(&step.expected.bytes),
            security_sha256: security_digest(step.expected)?,
        })
    }
}

fn security_digest(image: &FileImage) -> io::Result<String> {
    image.security.validate()?;
    Ok(sha256_hex(&serde_json::to_vec(&image.security)?))
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
enum Event<'a> {
    RetireIntent {
        node: usize,
        object: &'a Move,
    },
    RetireIntentReadback {
        node: usize,
        object: &'a Move,
    },
    OriginalReturnIntent {
        node: usize,
        object: &'a Move,
    },
    OriginalReturnIntentReadback {
        node: usize,
        object: &'a Move,
    },
    ExternalOriginalsRestoredBeforeResume {
        full_readback_sha256: &'a str,
        lease_release_sha256: &'a str,
        no_failed_endpoints_sha256: &'a str,
        normal_stop: bool,
        closed: bool,
        all_accepted: bool,
        public_start_open: bool,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Record<'a> {
    schema: &'static str,
    binding: &'a Binding,
    sequence: usize,
    previous_record_sha256: Option<String>,
    event: Event<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Progress {
    RepairPending,
    CompleteData,
}

pub(super) struct Plan {
    binding: Binding,
    moves: Vec<Move>,
}

impl Plan {
    pub(super) fn synthetic<'a>(
        steps: impl IntoIterator<Item = &'a FileStep<'a>>,
        failed_manifest: &[u8],
        failed_current: &[u8],
        action_id: String,
        deadline: Duration,
    ) -> io::Result<Self> {
        let moves: Vec<_> = steps
            .into_iter()
            .map(Move::data)
            .collect::<io::Result<_>>()?;
        if moves.len() != STEPS || deadline.is_zero() || deadline > Duration::from_secs(10) {
            return Err(invalid("external repair plan bounds"));
        }
        let binding = Binding {
            failed_manifest_sha256: sha256_hex(failed_manifest),
            failed_current_sha256: sha256_hex(failed_current),
            confirmed_scope_sha256: sha256_hex(&serde_json::to_vec(&moves)?),
            quiescence_evidence_sha256: sha256_hex(
                b"synthetic scene: no service, worker or endpoint created",
            ),
            repair_action_id: action_id,
            deadline_ns_from_action_start: deadline.as_nanos() as u64,
        };
        Ok(Self { binding, moves })
    }

    pub(super) fn step_record(
        &self,
        ordinal: usize,
        previous: Option<&[u8]>,
    ) -> io::Result<Vec<u8>> {
        let object = self
            .moves
            .get(ordinal / 2)
            .ok_or_else(|| invalid("external move ordinal"))?;
        let node = ordinal / 4;
        let event = match ordinal % 4 {
            0 => Event::RetireIntent { node, object },
            1 => Event::RetireIntentReadback { node, object },
            2 => Event::OriginalReturnIntent { node, object },
            _ => Event::OriginalReturnIntentReadback { node, object },
        };
        self.encode(ordinal, previous, event)
    }

    pub(super) fn completion_record(
        &self,
        previous: &[u8],
        observed: &CompletionObservations,
    ) -> io::Result<Vec<u8>> {
        self.encode(
            MOVE_RECORDS,
            Some(previous),
            Event::ExternalOriginalsRestoredBeforeResume {
                full_readback_sha256: &observed.full_readback_sha256,
                lease_release_sha256: &observed.lease_release_sha256,
                no_failed_endpoints_sha256: &observed.no_failed_endpoints_sha256,
                normal_stop: false,
                closed: false,
                all_accepted: false,
                public_start_open: false,
            },
        )
    }

    fn encode(
        &self,
        ordinal: usize,
        previous: Option<&[u8]>,
        event: Event<'_>,
    ) -> io::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&Record {
            schema: SCHEMA,
            binding: &self.binding,
            sequence: ordinal + 1,
            previous_record_sha256: previous.map(sha256_hex),
            event,
        })?;
        if bytes.len() > super::super::RECORD_LIMIT {
            return Err(invalid("external repair record too large"));
        }
        Ok(bytes)
    }

    // Compare exact canonical bytes to independently supplied observations and
    // expected operations. No self-asserted disk field can become a proof.
    pub(super) fn verify<'a>(
        &self,
        records: impl IntoIterator<Item = &'a [u8]>,
        observed: Option<&CompletionObservations>,
    ) -> io::Result<Progress> {
        let mut previous = None;
        let mut progress = Progress::RepairPending;
        for (ordinal, raw) in records.into_iter().enumerate() {
            let expected = match ordinal {
                MOVE_RECORDS => {
                    let observed =
                        observed.ok_or_else(|| invalid("completion observations missing"))?;
                    progress = Progress::CompleteData;
                    self.completion_record(
                        previous.ok_or_else(|| invalid("external repair prefix missing"))?,
                        observed,
                    )?
                }
                0..MOVE_RECORDS => self.step_record(ordinal, previous)?,
                _ => return Err(invalid("external repair record follows completion")),
            };
            if raw != expected {
                return Err(invalid(
                    "external repair record differs from fixed plan or prefix",
                ));
            }
            previous = Some(raw);
        }
        Ok(progress)
    }
}

pub(super) struct CompletionObservations {
    full_readback_sha256: String,
    lease_release_sha256: String,
    no_failed_endpoints_sha256: String,
}

impl CompletionObservations {
    pub(super) fn synthetic(full_readback_sha256: String, lease_release_sha256: String) -> Self {
        Self {
            full_readback_sha256,
            lease_release_sha256,
            no_failed_endpoints_sha256: sha256_hex(b"synthetic scene: no endpoints were created"),
        }
    }
}
