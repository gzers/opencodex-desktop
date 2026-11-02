# 0.2.20 production dialog callsite verification ledger

Source inventory: `prototype-audit-20261102/application-dialog-inventory.json`, based on v0.2.19 source `cb4c9c9b7`. It lists 37 component references in 21 files: 35 production references and two dev-gallery examples excluded from the production gate. A component reference is not a unique dialog state and does not establish runtime acceptance.

## Current evidence and limits

The retained browser route audit is **411 checks: 401 pass, 10 fail**, not 428/428. Failures include two Settings migration/sync structural checks, the action blur budget, four mid/low animation checks, the quality-card interaction, a disabled uninstall action, and notification cleanup. Keep these failures in the source report until individually resolved or shown to be stale with a new run. Route-state checks do not substitute for production dialog callsite coverage.

The isolated native macOS session inspected overview, Settings general, the file and WebDAV sync tabs, WebDAV connection dialog cancellation, model channel/template pages, a built-in template editor return path, Skills/MCP/client pages, add-MCP cancellation, and Codex client detail. The QA instance remained at the missing-Node prerequisite. It did not exercise dialog save, import/export, notification cleanup, install/uninstall, WebDAV test/sync, or backup/restore. Therefore no production dialog callsite is fully accepted by that session.

The referenced inventory is the authoritative row list for S20-20; status for every production entry is currently **not fully verified**. Evidence at the feature level exists for synchronization and model/UI behavior, but each callsite still requires a concrete applicable state and evidence mapping. The two component-gallery entries are excluded because they are development-only.

## Closure conditions

- Map all 35 production references to a frozen-prototype interaction, or give a source-based not-applicable reason.
- For each applicable dialog, inspect its initial, busy, validation/error, confirm, cancel, close/Escape, and owner-return states as supported by the design. Verify route and scroll preservation for in-place flows.
- Exercise failure/rollback behavior in isolated fixtures without daily configuration, credentials, real WebDAV, or production accounts.
- Re-run failed browser audit checks and retain original failure reports. Do not rewrite the 411/401 result as a pass.
- Do not close prototype parity or all-entry coverage based on route-shell screenshots, unit tests, or installer acceptance.
