# Windows official runtime backend (W0)

This module is intentionally not registered by the product's parent mod.rs yet.
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
