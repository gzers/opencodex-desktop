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

Revision 4: docs baseline 5ecf5b1a9f7f3046be17537916d9b511032ef6c2.
Use run.ps1 -RestorationRevision4. FiveRecovery captures only the exact five
registered Codex/Grok paths plus explicitly synthetic original config/credentials;
it retains differing and invalid raw bytes, length, SHA256, file ID, owner/DACL,
absent and unknown classifications. Each before/ready/pre-stop/official-stop/
compensated/finally phase is archived under a protected transaction directory;
unreached phases stay unreached. No runtime/admin/HMAC secret is an allowed target.

five-test.ps1 verifies auxiliary-original-absent and auxiliary-original-existing
cases, whole-batch refusal, same-prefix unknown files, external edits/audit append,
links/hardlinks/unreadability, tampered plans/originals, exclusive native handles,
CREATE_NEW collisions, second-operation rollback, and actual process exit 91 after
one complete write. A fresh process accepts only original or sealed postimage
contents and known file IDs; third contents are refused before other writes.
Durable intent is flushed before each operation; the independent protected plan
SHA binds before/after images and exact roots. This is a bounded crash test after
a full first write; arbitrary torn writes/power-loss recovery is not proven.

coordinator-r4.ts holds the unchanged official exclusive claim, N transaction and
K primitive for a maximum 2-second local test barrier, while the already-running
native host performs its synchronous handle batch. It does not call service or
spawn subprocesses under K. N is rolled back/closed and all locks released. This
is a test-only bridge across the Bun/native host processes, not production lock
integration. Its short test and exact namespace cleanup receipts are retained.

The exact five target paths are codex/config.toml, codex/opencodex-catalog.json,
codex/models_cache.json, codex/opencodex-catalog-audit.jsonl, and
grok/config.toml.bak-opencodex, relative to the freshly created fixture. A
producer-stage policy requires controlled ready/pre-stop consistency and narrow
official projections before authorizing any write. It is deliberately scoped to
synthetic fixtures and does not establish arbitrary daily-file authorship.

The single revision4 service run used df66f5bba5d20f387a143acc1e94fa6fa9aa9d90.
It stopped safely at the cache authorization gate: cache still held the controlled
routed model and did not satisfy the tested native-cache rule. No five-path batch
or current-run lock reacquisition followed. Full raw official-stop and finally
images are preserved; analyze-r4.ts and verify-r4.mjs review them offline. There
is no second service run or retroactive success label. Only the two specifically
adopted stable-lock paths may be excepted; all other differences still fail.

Revision 5 uses docs 1f35d520f74ac261f32c407f038e8f07a21c15fd and
run.ps1 -RestorationRevision5. cache-candidate.ts and its CLI/test inputs were
copied from the exact handoff Git blobs after manifest verification. The legacy
producer-r4 implementation remains unchanged: the original native-cache branch
and the other four path rules still run. The added cache branch can only restore
an originally absent cache to absence after verifying fixed official wrapping,
same raw bytes/file ID/owner/DACL through ready/pre-stop/official-stop, ready
catalog equality, independently frozen pre-start model declarations, and a
matching startup-cache audit event. It grants no artifact-retention exception.

host-r5 freezes scope before original service creation and records current native
context only after original normal exit, complete Job/listener observations and
Agent restoration. It copies the CURRENT integer elapsedMs from the original
Stop stopwatch, seals context/scope/stages with retained no-write/no-delete
sharing handles and hashes, invokes a create-only output CLI, and rechecks input
hashes before building the exact five-path plan. CLI, serialization, coordination,
batch writes, lock workers and final readback all consume the same 10 seconds.

The single recorded service run is 66afa61133d7654d3a2dfd172f49845bfb5272c3.
It passed the isolated proof under only the two previously adopted stable-lock
exceptions. Complete original-tree equality remains false because those two
files remain. That old assertion is retained separately. New code/verification
does not relabel any historical failure or constitute production integration,
daily-file provenance, dual-user acceptance, installer acceptance or release.
