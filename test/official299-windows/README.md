# Official 2.99 Windows proof (test only)

This directory is an isolated feasibility host. It does not register a production
Stop authority or replace the old Desktop lifetime contract. Desktop read-only
baseline: 0828a3955615df0a2a3e52fb96ab8249f2e04d1b. Handoff: docs baseline
f7c573349cf21c9c2984763a457fdc0926d21316, revision 1.

1. Run prototype-test.ps1 with a fresh output directory, as an ordinary user.
   Native cases include suspended creation/assignment, original exit code,
   surviving Job children, no-delete path guards, hard links, unreadable proofs,
   wrong identity and ordinary-user read-only program sealing.
2. Prepare the exact official npm archive, verify its 19,923,693 bytes and SHA256
   b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c.
   Install dependencies with lifecycle scripts disabled into a fresh fixture;
   pin platform Bun 1.4.4 using @oven/bun-windows-x64, retain package-lock.json
   and the read-only inventory.mjs byte inventories. Never run a Mac binary.
3. Invoke run.ps1 as an elevated provisioner, specifying the fixture input and
   archive roots and a fresh C:/ProgramData/ocxd299-proof-<10 hex> directory.
   The service and host run as one NEW, actual ordinary local account. Passwords
   and runtime management/HMAC secrets remain in memory/private test directories.
4. Preserve every failed attempt. A new attempt uses a new directory/account;
   a successful single-user proof is the stop point. Send separate test and docs
   branches for review; do not infer installer/tray/dual-user/release readiness.

The provisioner retains the real LogonUser token and LoadUserProfile hProfile;
it unloads that exact pair after the original host handle signals. A loaded,
special, mismatched, or unreadable Profile is retained and the account disabled.
No hive is forcibly unloaded and no daily session is logged off. The elevated
provisioner owns only fresh fixture ACLs, not existing repository/Profile ACLs.

Native.cs uses CreateProcessW with CREATE_SUSPENDED, a restricted inherited
stdio HANDLE_LIST and a fresh unnamed Job whose only principals are TokenUser
and SYSTEM. It observes TokenUser, elevation, session, image and FILETIME through
the original handle, assigns and checks membership before ResumeThread, and
rejects breakaway Job limits. Directory/file guards reject reparse points and
multi-link files and retain no-delete handles. The binding record is additionally
write-locked. Unknown identity or proof grants no stop authority. No full-machine
unreadable-process scan is used to manufacture ownership evidence.

The start stopwatch begins before native service creation. In 30 seconds it
requires official health identity/HMAC, readyz identity, exactly one official
completion line, matching listener PID and one host-generated startup JSONL
receipt. This receipt is explicitly a host record, not a new official producer.
The 5-second status action rechecks the original generation and official proof.
The 10-second stop action includes the API request, server restoration receipt,
original-handle exit/exit code, Job accounting plus complete PID list, listener
readback and synthetic client restoration. It calls POST /api/stop directly;
stopProxy/kill fallbacks are never counted as normal Stop. Forced termination
is a separate failure-cleanup fact limited to the retained owned Job.

Client originals contain synthetic config/credentials only. Complete client
trees are inventoried before/during/after. A locked running file is recorded as
unknown during observation; final unreadable or changed/additional client files
prevent full restoration acceptance. No live secret-bearing runtime-port.json
is copied to Git. package program files are sealed read-only for the runtime SID.

Run/fixture preparation is separate from action budgets and is not acceptance
of an old candidate. Evidence records exact attempted commit and raw file hashes;
later delivery-only commits do not rewrite the identity of prior attempts.

Revision 2 continues from docs 7b30abcdfe4b95808d260af13d1737323bbcd2aa.
Use run.ps1 -RestorationRevision2 to select host-r2.ps1. Recovery.cs captures
Agent original bytes/identities/permissions, forms a generated-marker operation
plan, then locks all existing target handles before reserving deleted paths with
CREATE_NEW. A mismatched/linked/unreadable postimage refuses the batch. Injected
write failures restore the sealed postimage through the same retained handles;
separate-process racing writes and absent-slot collisions are covered by
recovery-test.ps1. This is synchronous rollback evidence, not abrupt-crash or
durable-restart recovery evidence. Private preimage backup stays write/delete
locked and its digest is rechecked before compensation.

The after-ready Agent snapshot must still match at the pre-stop seal and after
the official process exits. A late change is unknown; it never silently enlarges
the write-set. Unrelated new user files are preserved and keep full-tree failure.
Compensation is explicitly attributed to the test host, separately from the
official /api/stop response. No broad prefix deletion or disable-injection trick
is used. The host holds every currently observed Job child's original handle,
captures actual IPv4/IPv6 listener rows, and checks those listeners and original
child exits within the same stop stopwatch. Ports are not inferred from +100.

lock-test.ps1 exercises unchanged official retainNativeMainOwner and
withNativeMainExclusiveClaim in real independent Windows Bun processes. A live
holder blocks the contender. A new process acquires after release without unlink,
with unchanged file ID and DACL. host-r2 performs the post-stop acquisitions as
the same new ordinary SID, within the original 10-second stop budget.

The native revision-2 run recorded at 709baa59d has Agent/Grok/lock/listener
proofs, but config.toml and the genuinely injected catalog differ after Stop;
five additional client files also remain. It is a failure regardless of the
still-unadopted exception for the two stable-lock files. Later delivery code
captures synthetic failed config/catalog postimage bytes for a possible future
run. That collection change has not been rerun as a service and does not repair
the missing postimage bytes from the recorded attempt. No further native service
retry is implied by this handoff.
