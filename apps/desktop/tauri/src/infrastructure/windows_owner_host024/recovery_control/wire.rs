use super::super::{decimal, qpc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io,
    time::{Duration, Instant},
};
pub(super) const SCHEMA: &str = "windows-before-resume-private-control-v2";
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "strict private beforeResumeV2 DATA unconfirmed",
    )
}
fn uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|v| !v.is_nil() && v.to_string() == value)
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct LocatorV2 {
    pub(super) schema: String,
    pub(super) instance_id: String,
    pub(super) pipe_name: String,
    pub(super) broker_pid: u32,
    pub(super) broker_filetime: String,
    pub(super) scope_digest: String,
}
impl LocatorV2 {
    pub(super) fn decode(raw: &[u8]) -> io::Result<Self> {
        if raw.is_empty() || raw.len() > 65536 {
            return Err(invalid());
        }
        let data: Self = serde_json::from_slice(raw).map_err(|_| invalid())?;
        let prefix = format!("{s}{s}.{s}pipe{s}ocxd-owner024-", s = char::from(92));
        if data.schema != SCHEMA
            || !uuid(&data.instance_id)
            || data.broker_pid == 0
            || decimal(&data.broker_filetime)? == 0
            || !digest(&data.scope_digest)
            || !data.pipe_name.starts_with(&prefix)
            || data.pipe_name.len() > 256
            || data.pipe_name.chars().any(|c| c.is_control())
            || !data.pipe_name.ends_with(&format!("-{}", data.scope_digest))
        {
            return Err(invalid());
        }
        Ok(data)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct HelloV2 {
    pub(super) schema: String,
    pub(super) instance_id: String,
    pub(super) manager_generation: String,
    pub(super) challenge: String,
    pub(super) scope_digest: String,
}
impl HelloV2 {
    pub(super) fn validate(&self) -> io::Result<()> {
        if self.schema != SCHEMA
            || !uuid(&self.instance_id)
            || decimal(&self.manager_generation)? == 0
            || !digest(&self.challenge)
            || !digest(&self.scope_digest)
        {
            Err(invalid())
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum CommandV2 {
    BeginCanonicalAbort,
    ReadWorkerDiagnostic,
    ReadCanonicalHistory,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Confirmation {
    ExplicitBeforeFirstResumeRecovery,
    ReadOnly,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BudgetV2 {
    transport_action_id: String,
    started_qpc: String,
    deadline_qpc: String,
    qpc_frequency: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct RequestV2 {
    schema: String,
    request_id: String,
    instance_id: String,
    manager_generation: String,
    challenge: String,
    scope_digest: String,
    command: CommandV2,
    confirmation: Confirmation,
    budget: BudgetV2,
}
pub(super) struct ReadDeadline {
    pub(super) started: Instant,
    pub(super) end: Instant,
}
pub(super) struct AdmittedV2 {
    pub(super) request_id: String,
    pub(super) transport_action_id: String,
    pub(super) command: CommandV2,
    pub(super) deadline: ReadDeadline,
}
impl RequestV2 {
    pub(super) fn decode(frame: &[u8]) -> io::Result<Self> {
        if frame.len() < 4 {
            return Err(invalid());
        }
        let n = u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize;
        if n == 0 || n > 65536 || frame.len() != n + 4 {
            return Err(invalid());
        }
        let data: Self = serde_json::from_slice(&frame[4..]).map_err(|_| invalid())?;
        data.validate_shape()?;
        Ok(data)
    }
    fn validate_shape(&self) -> io::Result<()> {
        if self.schema != SCHEMA
            || ![
                &self.request_id,
                &self.instance_id,
                &self.budget.transport_action_id,
            ]
            .into_iter()
            .all(|s| uuid(s))
            || self.request_id == self.budget.transport_action_id
            || !digest(&self.challenge)
            || !digest(&self.scope_digest)
            || decimal(&self.manager_generation)? == 0
        {
            return Err(invalid());
        }
        for v in [
            &self.budget.started_qpc,
            &self.budget.deadline_qpc,
            &self.budget.qpc_frequency,
        ] {
            decimal(v)?;
        }
        if (self.command == CommandV2::BeginCanonicalAbort)
            != (self.confirmation == Confirmation::ExplicitBeforeFirstResumeRecovery)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(super) fn admit(
        self,
        hello: &HelloV2,
        used: &mut BTreeSet<String>,
    ) -> io::Result<AdmittedV2> {
        let sampled = Instant::now();
        let (counter, frequency) = qpc()?;
        self.admit_at(hello, used, counter, frequency, sampled)
    }
    fn admit_at(
        self,
        hello: &HelloV2,
        used: &mut BTreeSet<String>,
        counter: u64,
        frequency: u64,
        now: Instant,
    ) -> io::Result<AdmittedV2> {
        self.validate_shape()?;
        hello.validate()?;
        if self.instance_id != hello.instance_id
            || self.manager_generation != hello.manager_generation
            || self.challenge != hello.challenge
            || self.scope_digest != hello.scope_digest
        {
            return Err(invalid());
        }
        let start = decimal(&self.budget.started_qpc)?;
        let end = decimal(&self.budget.deadline_qpc)?;
        let freq = decimal(&self.budget.qpc_frequency)?;
        let seconds = if self.command == CommandV2::BeginCanonicalAbort {
            10
        } else {
            5
        };
        if frequency == 0
            || frequency > i64::MAX as u64
            || freq != frequency
            || start > counter
            || end <= counter
            || frequency
                .checked_mul(seconds)
                .and_then(|n| start.checked_add(n))
                .is_none_or(|max| end > max)
        {
            return Err(invalid());
        }
        // QPC is sampled before Instant: adding floored remaining ticks to the
        // earlier Instant is conservative. A challenge/transport never adds time.
        let nanos = (end - counter) as u128 * 1_000_000_000 / frequency as u128;
        let elapsed = ((counter - start) as u128 * 1_000_000_000).div_ceil(frequency as u128);
        let deadline = ReadDeadline {
            started: now
                .checked_sub(Duration::from_nanos(
                    u64::try_from(elapsed).map_err(|_| invalid())?,
                ))
                .ok_or_else(invalid)?,
            end: now
                .checked_add(Duration::from_nanos(
                    u64::try_from(nanos).map_err(|_| invalid())?,
                ))
                .ok_or_else(invalid)?,
        };
        if deadline.started > now || deadline.end <= now {
            return Err(invalid());
        }
        let keys = [
            self.request_id.clone(),
            self.budget.transport_action_id.clone(),
            self.challenge.clone(),
        ];
        if used
            .len()
            .checked_add(keys.len())
            .is_none_or(|len| len > 16384)
            || keys.iter().any(|key| used.contains(key))
        {
            return Err(invalid());
        }
        used.extend(keys);
        Ok(AdmittedV2 {
            request_id: self.request_id,
            transport_action_id: self.budget.transport_action_id,
            command: self.command,
            deadline,
        })
    }
}
#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
