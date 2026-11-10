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
