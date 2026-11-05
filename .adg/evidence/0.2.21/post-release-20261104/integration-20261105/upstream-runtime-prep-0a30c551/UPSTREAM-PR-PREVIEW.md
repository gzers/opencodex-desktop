## Summary

- Replace per-process CIM GetOwner RPCs in Windows desktop/app-server discovery with one metadata snapshot and native-handle token SID, executable path and creation-time verification. Keep the 10s/8s limits, incomplete/unknown results, current-user/package boundaries, PID generation/ancestry protection and quit/relaunch ladder.
- Preserve the desktop base64 command-line column needed by the registered compatibility launch context. Embed query scripts in TypeScript so compiled distributions do not need adjacent PowerShell files.
- Update focused query/fixture coverage and the owning desktop contract. This is a local, unsubmitted draft candidate; it has not been integrated or released.

## Verification

- Bun 1.4.4: bun check --no-script --noEmit passed for 2,306 runtime files.
- Bun 1.4.4: bun test ./tests/clients/windows-process-identity.test.ts ./tests/clients/desktop-app-restart.test.ts ./tests/codex-integration/codex-app-server-processes.test.ts: 109 passed, 3 Windows-only checks not executed on macOS, 0 failed. The native policy fixtures were adapted; their actual Windows execution is still required.
- Compiled query-builder smoke: identical interpreted/compiled output with network and physical source-directory reads denied. This checks embedded query generation, not the complete standalone proxy or Windows API execution.
- Upstream scanText on the 10 changed files: zero findings. Full Git-index-dependent privacy/structure/layout gates and upstream CI remain pending.
- The 8 modified preimages match fixed dev commit 0a30c551304927c93c26a96c034bd39a166c7460; both new paths were absent. git apply --check and exact 10-file replay passed. The expected-layout fixture was taken from dev to retain unrelated entries. Lock and tsconfig match the released local snapshot, and package.json differs only by version. Local tests used those unchanged released surroundings; this is not a claim of a complete dev-tree validation.
- Prior failures remain preserved: old query-text expectations, the fake fixture's CreationDate-only dispatch, and the interrupted dev archive fetch. A separately verified older-runtime source candidate supplied the original Windows native comparison; that is not substituted for native validation of this port.

## Checklist

- [x] Scope stays focused and avoids unrelated cleanup.
- [x] Docs or release notes were updated when needed.
- [ ] Security-sensitive changes were reviewed for secrets, auth, and unsafe defaults.

Submission/ready-for-review is pending user authorization, a real branch based on current dev, required native validation, upstream CI and maintainer review. No fork, pull request, issue, label or notification has been created.
