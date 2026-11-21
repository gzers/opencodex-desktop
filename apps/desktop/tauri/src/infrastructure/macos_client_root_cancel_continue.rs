//! Continue an indexed cancellation while holding the original native claim
//! and coordinators. The request digest binds the location and exact sidecars.
use super::*;
use crate::infrastructure::official_client_coordination::Context;

pub(super) fn create_or_confirm(
    directory: &PinnedDirectory,
    filename: &str,
    expected: &[u8],
) -> std::io::Result<()> {
    directory.private()?;
    if capture_file(directory, filename, MAX_FRAME)? == Image::Absent {
        create(directory, filename, expected)?;
    }
    let file = open_at(directory.file(), OsStr::new(filename), false)?;
    let observed = capture_file(directory, filename, MAX_FRAME)?;
    match &observed {
        Image::File {
            bytes,
            protection,
            identity: saved,
            ..
        } if bytes == expected && &identity(&stat(&file)?) == saved => {
            require_private(protection, false)?;
        }
        _ => return Err(denied("root-cancel exact sidecar differs")),
    }
    file.sync_all()?;
    directory.sync()?;
    if capture_file(directory, filename, MAX_FRAME)? != observed {
        return Err(denied("root-cancel sidecar changed during sync"));
    }
    Ok(())
}

pub(in crate::infrastructure::macos_client_originals) fn resume(
    index: &Index,
    head: &Head,
    confirmation: &str,
    service: &service::CurrentService<'_>,
    context: &Context,
    deadline: Instant,
) -> crate::errors::AppResult<(PreservedRoot, native::released::ReleasedCoordination)> {
    use super::super::root_cancellation::fail;

    let origin = index
        .root_registration
        .as_ref()
        .ok_or_else(|| fail("root origin missing"))?;
    let reference = head
        .root_retirement
        .as_ref()
        .ok_or_else(|| fail("plan reference missing"))?;
    let connection = index.connection().map_err(fail)?;
    let pending = indexed::inspect_plan(
        &index.data,
        &index.profile,
        &index.binding,
        head,
        origin,
        connection,
    )
    .map_err(fail)?;
    if pending.fingerprint != confirmation {
        return Err(fail("pending confirmation changed"));
    }
    let frozen = pending
        .plan
        .reacquisition
        .as_ref()
        .ok_or_else(|| fail("pending native coordination evidence missing"))?;
    service
        .require_action(index, head, deadline)
        .map_err(fail)?;
    native::with_native_reacquisition(
        context,
        service.selection(),
        frozen,
        pending.root.path(),
        deadline,
        |native| {
            let mut guard = native.config_write_guard()?;
            let inspect = |guard: &mut Guard<'_>| {
                index.check_storage()?;
                indexed::inspect_plan_guarded(
                    &index.data,
                    &index.profile,
                    &index.binding,
                    head,
                    origin,
                    connection,
                    native,
                    guard,
                )
            };
            let current = inspect(&mut guard).map_err(fail)?;
            if current.fingerprint != confirmation {
                return Err(fail("pending guarded confirmation changed"));
            }
            let profile = PinnedDirectory::open(&index.profile).map_err(fail)?;
            let target = current.directory.path().join("root");
            let intent =
                encode(&(reference.plan.sha256.as_str(), current.root.identity())).map_err(fail)?;
            service.require_guarded(native, &mut guard).map_err(fail)?;
            if current.root.path() == context.codex_home {
                guard.prepare_relocation(&target)?;
                create_or_confirm(&current.directory, "move-intent.json", &intent).map_err(fail)?;
                service.require_guarded(native, &mut guard).map_err(fail)?;
                let before_rename = inspect(&mut guard).map_err(fail)?;
                if before_rename.root.path() != current.root.path()
                    || before_rename.root.coordination_expectation()
                        != current.root.coordination_expectation()
                    || before_rename.directory.coordination_expectation()
                        != current.directory.coordination_expectation()
                    || before_rename.plan_image != current.plan_image
                {
                    return Err(fail("pending movement source changed"));
                }
                profile.recheck().map_err(fail)?;
                current.root.recheck().map_err(fail)?;
                current.directory.private().map_err(fail)?;
                check_time(deadline).map_err(fail)?;
                let from = name(OsStr::new(".codex")).map_err(fail)?;
                let to = name(OsStr::new("root")).map_err(fail)?;
                if unsafe {
                    libc::renameatx_np(
                        profile.file().as_raw_fd(),
                        from.as_ptr(),
                        current.directory.file().as_raw_fd(),
                        to.as_ptr(),
                        libc::RENAME_EXCL,
                    )
                } != 0
                {
                    return Err(fail(std::io::Error::last_os_error()));
                }
                profile.sync().map_err(fail)?;
                current.directory.sync().map_err(fail)?;
                native.reanchor_claim(&target)?;
                guard.reanchor(&target)?;
            } else {
                profile.sync().map_err(fail)?;
                current.directory.sync().map_err(fail)?;
                native.reanchor_claim(&target)?;
            }
            let archived = inspect(&mut guard).map_err(fail)?;
            if archived.root.path() != target
                || archived.root.identity() != current.root.identity()
                || archived.directory.identity() != current.directory.identity()
                || archived.plan_image != current.plan_image
            {
                return Err(fail("pending preserved root changed"));
            }
            service.require_guarded(native, &mut guard).map_err(fail)?;
            guard.finish()?;
            let released = archived
                .plan
                .root
                .without_config_guard(&archived.plan.writer)
                .map_err(fail)?;
            let prove_release = || -> std::io::Result<()> {
                check_time(deadline)?;
                index.check_storage()?;
                if TreeSnapshot::read(&archived.root)? != released
                    || observation::entry(&profile, ".codex")?.is_some()
                {
                    return Err(denied(
                        "pending writer release or logical absence unconfirmed",
                    ));
                }
                archived.directory.private()
            };
            prove_release().map_err(fail)?;
            native.check()?;
            let bytes = preservation_bytes(&reference.plan.sha256, archived.root.identity())
                .map_err(fail)?;
            create_or_confirm(&archived.directory, "preserved-readback.json", &bytes)
                .map_err(fail)?;
            prove_release().map_err(fail)?;
            Ok(PreservedRoot {
                id: reference.plan.id.clone(),
                plan_sha256: reference.plan.sha256.clone(),
                receipt_sha256: digest(&bytes),
                root_identity: archived.root.identity().clone(),
                admitted: true,
                continued: true,
                deadline,
            })
        },
    )
}
