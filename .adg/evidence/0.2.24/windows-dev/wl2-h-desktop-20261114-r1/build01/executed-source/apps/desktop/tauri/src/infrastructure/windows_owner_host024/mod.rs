//! Wire and clock prerequisites for the original owner host. Decoding a frame
//! grants no ownership; native peer authentication and the four proofs remain
//! mandatory before any production operation can be dispatched.
use serde::{Deserialize, Serialize};
mod continuity;
#[cfg(test)]
mod desktop_tests;
pub mod empty_host;
use std::{
    io,
    time::{Duration, Instant},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Request {
    schema: u32,
    request_id: String,
    instance_id: String,
    manager_generation: String,
    challenge: String,
    operation: Operation,
    budget: WireBudget,
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Operation {
    Status,
    Start,
    Stop,
    Restart,
    Detach,
    ExplicitRecovery,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireBudget {
    action_id: String,
    started_qpc: String,
    deadline_qpc: String,
    stop_deadline_qpc: String,
    qpc_frequency: String,
}
#[derive(Debug)]
pub(crate) struct ValidatedDeadline {
    started: Instant,
    deadline: Instant,
    stop_deadline: Instant,
    action_id: uuid::Uuid,
}
fn invalid(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}
fn decimal(text: &str) -> io::Result<u64> {
    if text.is_empty()
        || text.len() > 20
        || !text.bytes().all(|b| b.is_ascii_digit())
        || text.len() > 1 && text.starts_with('0')
    {
        return Err(invalid("noncanonical QPC integer"));
    }
    text.parse().map_err(|_| invalid("QPC integer overflow"))
}
pub(crate) fn qpc() -> io::Result<(u64, u64)> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn QueryPerformanceCounter(value: *mut i64) -> i32;
        fn QueryPerformanceFrequency(value: *mut i64) -> i32;
    }
    let (mut counter, mut frequency) = (0i64, 0i64);
    if unsafe { QueryPerformanceCounter(&mut counter) } == 0
        || unsafe { QueryPerformanceFrequency(&mut frequency) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if counter < 0 || frequency <= 0 {
        return Err(invalid("invalid native QPC"));
    }
    Ok((counter as u64, frequency as u64))
}
pub(crate) fn outer_job_precondition() -> io::Result<()> {
    use windows_sys::Win32::System::{JobObjects::IsProcessInJob, Threading::GetCurrentProcess};
    let mut in_job = 0;
    if unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut in_job) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Querying only the innermost Job cannot prove all enclosing Jobs safe.
    // Do not add BREAKAWAY flags or silently escape the host's external policy.
    if in_job != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "inherited outer Job continuity not proven",
        ));
    }
    Ok(())
}
pub(crate) fn decode(frame: &[u8]) -> io::Result<Request> {
    if frame.len() < 4 {
        return Err(invalid("truncated length"));
    }
    let n = u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize;
    if n == 0 || n > 65536 || frame.len() != n + 4 {
        return Err(invalid("oversized, truncated or trailing frame"));
    }
    let text = std::str::from_utf8(&frame[4..]).map_err(|_| invalid("frame is not UTF-8"))?;
    if text.contains('\0') {
        return Err(invalid("NUL in frame"));
    }
    // Typed serde structures reject duplicate keys at every nesting level,
    // unknown fields/enums, non-string integer fields and trailing JSON data.
    let r: Request = serde_json::from_str(text).map_err(|_| invalid("strict request schema"))?;
    if r.schema != 1
        || uuid::Uuid::parse_str(&r.request_id).is_err()
        || uuid::Uuid::parse_str(&r.instance_id).is_err()
        || r.challenge.len() != 64
        || !r
            .challenge
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        || decimal(&r.manager_generation)? == 0
    {
        return Err(invalid("request identity shape"));
    }
    Ok(r)
}
impl Request {
    pub(crate) fn deadline_at(
        &self,
        now_qpc: u64,
        frequency: u64,
        instant: Instant,
    ) -> io::Result<ValidatedDeadline> {
        let start = decimal(&self.budget.started_qpc)?;
        let end = decimal(&self.budget.deadline_qpc)?;
        let stop = decimal(&self.budget.stop_deadline_qpc)?;
        let freq = decimal(&self.budget.qpc_frequency)?;
        let limit = match self.operation {
            Operation::Start | Operation::Restart => 30,
            Operation::Status | Operation::Detach => 5,
            Operation::Stop | Operation::ExplicitRecovery => 10,
        };
        let max = start
            .checked_add(
                frequency
                    .checked_mul(limit)
                    .ok_or_else(|| invalid("QPC frequency overflow"))?,
            )
            .ok_or_else(|| invalid("QPC deadline overflow"))?;
        let stop_max = start
            .checked_add(
                frequency
                    .checked_mul(10)
                    .ok_or_else(|| invalid("stop QPC overflow"))?,
            )
            .ok_or_else(|| invalid("stop deadline overflow"))?;
        if frequency == 0
            || frequency > i64::MAX as u64
            || freq != frequency
            || start > now_qpc
            || end <= now_qpc
            || end > max
            || stop > end
            || stop > stop_max
            || stop < start
        {
            return Err(invalid("expired or extended original action window"));
        }
        let remaining = |q: u64| -> io::Result<Duration> {
            // Floor to whole nanoseconds; transport/handshake never create a new
            // full budget. Past Stop subwindows remain expired for Restart.
            let ticks = q.saturating_sub(now_qpc) as u128;
            let nanos = ticks
                .checked_mul(1_000_000_000)
                .ok_or_else(|| invalid("QPC conversion overflow"))?
                / frequency as u128;
            Ok(Duration::from_nanos(
                u64::try_from(nanos).map_err(|_| invalid("QPC duration overflow"))?,
            ))
        };
        Ok(ValidatedDeadline {
            started: instant
                .checked_sub(Duration::from_nanos(
                    u64::try_from(
                        ((now_qpc - start) as u128 * 1_000_000_000).div_ceil(frequency as u128),
                    )
                    .map_err(|_| invalid("QPC original start overflow"))?,
                ))
                .ok_or_else(|| invalid("original Instant start overflow"))?,
            deadline: instant
                .checked_add(remaining(end)?)
                .ok_or_else(|| invalid("Instant overflow"))?,
            stop_deadline: instant
                .checked_add(remaining(stop)?)
                .ok_or_else(|| invalid("Stop Instant overflow"))?,
            action_id: uuid::Uuid::parse_str(&self.budget.action_id)
                .map_err(|_| invalid("action ID"))?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn json() -> serde_json::Value {
        serde_json::json!({"schema":1,"requestId":"00000000-0000-4000-8000-000000000001","instanceId":"00000000-0000-4000-8000-000000000002","managerGeneration":"1","challenge":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","operation":"restart","budget":{"actionId":"00000000-0000-4000-8000-000000000003","startedQpc":"1000","deadlineQpc":"31000","stopDeadlineQpc":"11000","qpcFrequency":"1000"}})
    }
    fn frame(s: &[u8]) -> Vec<u8> {
        let mut v = (s.len() as u32).to_le_bytes().to_vec();
        v.extend(s);
        v
    }
    #[test]
    fn wl2_strict_frames_reject_duplicate_truncated_and_unknown() {
        let s = serde_json::to_string(&json()).unwrap();
        assert!(decode(&frame(s.as_bytes())).is_ok());
        for bad in [
            s.replacen("{", "{\"schema\":1,", 1),
            s.replace(
                "\"startedQpc\":\"1000\"",
                "\"startedQpc\":\"1000\",\"startedQpc\":\"1000\"",
            ),
            s.replace("restart", "kill"),
            s.replace("\"1000\"", "1000"),
            s.replace("\"schema\":1", "\"schema\":2"),
            s.replace("\"schema\":1", "\"schema\":1,\"path\":\"C:/arbitrary\""),
            format!("{s}{{}}"),
            format!("{s}\0"),
        ] {
            assert!(
                decode(&frame(bad.as_bytes())).is_err(),
                "bad frame accepted: {bad}"
            );
        }
        let v = frame(s.as_bytes());
        assert!(decode(&v[..v.len() - 1]).is_err());
        let mut trailing = v.clone();
        trailing.extend([0, 0, 0, 0]);
        assert!(decode(&trailing).is_err());
        assert!(decode(&frame(&[0xff])).is_err());
        assert!(decode(&frame(&vec![b' '; 65537])).is_err());
    }
    #[test]
    fn wl2_qpc_preserves_original_restart_stop_subwindow() {
        let instant = Instant::now();
        let r = decode(&frame(serde_json::to_string(&json()).unwrap().as_bytes())).unwrap();
        let d = r.deadline_at(6000, 1000, instant).unwrap();
        assert_eq!(d.deadline, instant + Duration::from_secs(25));
        assert_eq!(d.stop_deadline, instant + Duration::from_secs(5));
        assert_eq!(
            d.action_id.to_string(),
            "00000000-0000-4000-8000-000000000003"
        );
        assert_eq!(
            r.deadline_at(12000, 1000, instant).unwrap().stop_deadline,
            instant
        );
        for (now, freq) in [
            (500, 1000),
            (31000, 1000),
            (6000, 999),
            (6000, 0),
            (6000, u64::MAX),
        ] {
            assert!(r.deadline_at(now, freq, instant).is_err());
        }
        for bad in ["+1000", "01", "-1", "18446744073709551616", "1.0", "1e3"] {
            assert!(decimal(bad).is_err());
        }
        let (a, f) = qpc().unwrap();
        let (b, g) = qpc().unwrap();
        assert!(b >= a && f == g && f > 0);
    }
}
