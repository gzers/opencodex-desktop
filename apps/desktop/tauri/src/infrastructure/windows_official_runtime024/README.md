# Windows official runtime backend (W0 / W-L1)

The product registers this module and the W-L1 Windows lifecycle adapter consumes
its original creation handles, private anonymous Job and listener observations.
The standalone test/windows024-native Cargo host imports this exact source with
the path attribute. It compiles real Windows APIs without Tauri or a copied module.
These are process-local Rust types; they are not frozen IPC or persistent formats.

Batch 1 supplies TokenUser/session/elevation observations, protected directory
creation, ancestor/file identity pins and immutable bounded private records.
Existing owner/DACL/link/type failures return errors without repairing objects.
Private creation names the current TokenUser as owner even when an elevated token
defaults to the Administrators owner. Only that SID and SYSTEM are allowed. File
handles pin the original generation and deny write/delete sharing as appropriate.
Path input is an explicit local drive path; UNC/device/ADS/dot/ambiguous names
are refused. No old-path migration or service discovery takes place.

Mac integration owns shared source admission, health HMAC/ready validation,
record encoding, the application state machine, recovery policy and total
30/5/10-second budgets. Windows primitives accept already-resolved program/HOME
inputs rather than scan the machine. CLI/PATH, installers, parent module
registration and shared manifests remain outside W0's write set.

Batch 2 supplies OwnedRuntime::create_suspended/resume with a restricted inherited
stdio handle list, explicit HOME/OPENCODEX_HOME, original process/thread handles,
TokenUser/session/elevation/FILETIME, pinned executable file ID, and a fresh private
anonymous no-breakaway Job. Assignment and membership checks happen before resume.
Wrong binding returns an error; PIDs are only opened from this retained Job's
bounded member list, with a membership/generation check on the new handle.

family_state retains observed descendant handles and checks active accounting
against a complete bounded PID list. Concurrent list changes return WouldBlock;
inaccessible/vanished observations remain errors. A root exit or HTTP success is
not complete family exit. wait_normal_exit only observes, honors the caller's
absolute deadline including late results, and never signals/kills a process.
force_owned_cleanup is explicitly separate and returns ForcedCleanup, never a
normal-stop success. Dropping OwnedRuntime closes handles and does not kill a
running runtime; integration must retain this owner or deliberately clean its Job.

tcp_listeners reads complete native IPv4/IPv6 owner-PID listener tables without
process-global ownership/command scans. listeners binds rows to the owned Job's
original retained generations and rechecks the member set. Final readback also
refuses success if another process has claimed a former listening endpoint; it
never stops that replacement. A snapshot is not a continuous listener monitor,
and unseen historical descendants are not assigned invented exit codes.

W-L1 associates a completion port before the suspended root is assigned to its
Job and starts the capture worker before resume. New-member handles retain their
FILETIME/SID/image file identity. Notifications are not guaranteed exit evidence:
any inaccessible member, generation mismatch or difference between total process
accounting and retained history keeps the result unknown. Only the exact console
image under the Win32-reported System32 directory allows Windows component-store
hardlinks; admitted runtime/package images retain their single-link requirement.

The backend consumes only existing product windows-sys features (0.61 API family)
and std. IP Helper/ACL SDK functions unavailable through the existing feature set
use local typed extern bindings. Root Cargo manifests, shared types, old Windows
modules and dependency features are not changed. The separate production adapter
consumes shared typed HMAC/ready/Stop and LaunchPlan/ActionWindow interfaces; this
backend never adds an official marker, persistence format or restoration journal.
Client-original recovery, supervisor absence and cross-manager owner continuity
remain distinct requirements; a native Job empty or forced test cleanup cannot
mint normal Stop, Restart eligibility or complete 30/5/10-second acceptance.
