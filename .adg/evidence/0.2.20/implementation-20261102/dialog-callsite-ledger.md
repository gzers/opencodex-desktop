# 0.2.20 production dialog callsite verification ledger

Source inventory: `prototype-audit-20261102/application-dialog-inventory.json`, based on the v0.2.19 source `cb4c9c9b7`. It lists 37 component references across 21 files: 35 production references and two dev-gallery examples excluded from the production gate. A component reference is not a unique dialog state and does not establish runtime acceptance.

## Current route and native evidence

The first retained browser route audit report was 411 checks: 401 passed, 10 failed. It remains preserved as an earlier failed attempt. Re-running `apps/desktop/ui/qa/audit.browser.mjs` against the current source produced 428/428 passed, zero failed checks, and zero page errors at 2026-11-02T13:18Z. Both reports are retained; the newer result is the current full-route browser result.

The isolated native macOS session inspected overview, Settings general, file and WebDAV sync tabs, WebDAV connection dialog cancellation, model channel/template pages, a built-in template editor return path, Skills/MCP/client pages, add-MCP cancellation, and Codex client detail. The QA instance remained at the missing-Node prerequisite. It did not exercise save, import/export, notification cleanup, install/uninstall, WebDAV test/sync, or backup/restore. The session therefore does not fully accept any of the production dialog callsites listed in the static inventory.

Status for every production reference: **not fully verified**. Several feature-level behaviors have separate CI, component, route, or native evidence, but no evidence set yet maps all applicable states per callsite. The two component-gallery references are excluded because they are development-only.

## Closure conditions

- Map all 35 production references to the frozen-prototype interaction, or give a source-based not-applicable reason.
- For each applicable dialog, inspect its initial, busy, validation/error, confirm, cancel, close/Escape, and owner-return states as supported by the design. Verify route and scroll preservation for in-place flows.
- Exercise failure and rollback behavior with isolated fixtures without daily configuration, credentials, real WebDAV, or production accounts.
- Keep both browser reports. The earlier 411/401/10 failures are not erased by the new 428/428 route pass.
- Do not close prototype parity or all-entry coverage using route-shell screenshots, unit tests, or installer acceptance alone.
