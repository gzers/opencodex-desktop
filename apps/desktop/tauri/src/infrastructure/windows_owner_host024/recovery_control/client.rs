//! Same ordinary exact-image private client; locator/reply are DATA only. The
//! caller supplies the already-started original10/5 window, including bootstrap.
use super::{
    server::LOCATOR,
    wire::{self, AckV2, CommandV2, HelloV2, LocatorV2, ReplyV2, RequestV2},
};
use crate::infrastructure::{platform, windows_official_runtime024 as native};
use std::{io, time::Instant};
pub(in crate::infrastructure::windows_owner_host024) fn command(
    text: &str,
) -> io::Result<(CommandV2, u64)> {
    match text {
        "begin-canonical-abort" => Ok((CommandV2::BeginCanonicalAbort, 10)),
        "read-worker-diagnostic" => Ok((CommandV2::ReadWorkerDiagnostic, 5)),
        "read-canonical-history" => Ok((CommandV2::ReadCanonicalHistory, 5)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "exact privateV2 operation required",
        )),
    }
}
fn denied(reason: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason.to_string())
}
pub(in crate::infrastructure::windows_owner_host024) fn request(
    c: &super::super::empty_host::EmptyBootstrap,
    command: CommandV2,
    started: Instant,
    end: Instant,
    start_qpc: u64,
    frequency: u64,
) -> io::Result<(serde_json::Value, bool)> {
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    let current = native::RetainedPeer::current()?;
    let token = &current.identity().token;
    if c.schema != 1
        || token.elevated
        || token.elevation_type != 1
        || token.sid != c.expected_sid
        || token.session_id != c.expected_session
        || platform::home_dir().as_deref() != Some(c.profile.as_path())
    {
        return Err(denied("same original ordinary desktop/bootstrap required"));
    }
    current.no_outer_job()?;
    let image = c.installation.join("ocxd.exe");
    current.verify_image(&image)?;
    let pins = [
        native::PinnedDirectory::open(&c.profile)?,
        native::PinnedDirectory::open_private(&c.data)?,
        native::PinnedDirectory::open_private(&c.home)?,
        native::PinnedDirectory::open(&c.installation)?,
    ];
    let locator_path = c.data.join(LOCATOR);
    let raw = native::PrivateRecord::locator_snapshot(&locator_path, 65536)?;
    let locator = LocatorV2::decode(&raw)?;
    let pipe = native::PrivatePipe::connect_local(&locator.pipe_name)?;
    let peer = pipe.peer()?;
    let recheck = |connected: bool| -> io::Result<()> {
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        current.no_outer_job()?;
        current.verify_image(&image)?;
        peer.no_outer_job()?;
        peer.verify_image(&image)?;
        if peer.identity().token != *token
            || peer.identity().pid != locator.broker_pid
            || peer.identity().creation_filetime.to_string() != locator.broker_filetime
            || connected && pipe.peer()?.identity() != peer.identity()
            || native::PrivateRecord::locator_snapshot(&locator_path, 65536)? != raw
        {
            return Err(denied("original native broker/locator binding changed"));
        }
        for pin in &pins {
            pin.recheck()?;
        }
        if Instant::now() >= end {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    };
    recheck(true)?;
    let hello = HelloV2::decode(&pipe.read_frame(end)?)?;
    if hello.schema != wire::SCHEMA
        || hello.instance_id != locator.instance_id
        || hello.scope_digest != locator.scope_digest
    {
        return Err(denied("original native hello scope/version changed"));
    }
    // Convert only the remaining duration after native reads; no fresh full
    // window is minted after bootstrap or handshake. QPC cutoff rounds down.
    let (counter, actual_frequency) = super::super::qpc()?;
    if actual_frequency != frequency || counter < start_qpc {
        return Err(denied("native QPC changed"));
    }
    let remaining = end.saturating_duration_since(Instant::now());
    let ticks = remaining
        .as_nanos()
        .checked_mul(frequency as u128)
        .ok_or_else(|| denied("QPC overflow"))?
        / 1_000_000_000;
    let cutoff = counter
        .checked_add(u64::try_from(ticks).map_err(|_| denied("QPC remaining overflow"))?)
        .ok_or_else(|| denied("QPC cutoff overflow"))?;
    let request = RequestV2::for_client(&hello, command, start_qpc, cutoff, frequency)?;
    let receipt = request.client_receipt(end, started);
    recheck(true)?;
    pipe.write_frame(
        &serde_json::to_vec(&request).map_err(io::Error::other)?,
        end,
    )?;
    let reply = ReplyV2::decode(&pipe.read_frame(end)?, &receipt, &locator.scope_digest)?;
    recheck(true)?;
    let refused = reply.is_refused();
    let data = reply.into_data()?;
    #[cfg(test)]
    if let Some(proof) = data["facts"]
        .get("originalRangeProof")
        .filter(|proof| !proof.is_null())
    {
        if command != CommandV2::ReadWorkerDiagnostic
            || proof["requestId"] != receipt.request_id
            || proof["transportActionId"] != receipt.transport_action_id
        {
            return Err(denied("original range proof differs from admitted request"));
        }
    }
    pipe.write_frame(
        &serde_json::to_vec(&AckV2::for_request(&receipt, &locator.scope_digest))
            .map_err(io::Error::other)?,
        end,
    )?;
    // The same native peer/locator/pins are still checked after terminal ACK.
    // The server may now disconnect: no new action depends on connection state.
    recheck(false)?;
    Ok((data, refused))
}
