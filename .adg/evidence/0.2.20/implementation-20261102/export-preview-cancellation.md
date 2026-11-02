# Export preview cancellation lifecycle

Date: 2026-11-02. Product evidence was taken from an isolated local Tauri development build compiled from working tree based at main commit 70724914649a23f252563f8a3675fcef736122e4 with follow-up source derived from PR #37; the native cancel/restart observation preceded the final result-receipt guard and last UI unmount hook, which are covered by Rust/UI tests and pending current-head CI. The QA app bundle identifier was com.gzers.opencodex.desktop.discardqa; HOME, OpenCodex home, data root, npm config/cache, and command-only stub were under /tmp/ocx020-discard-native. The installed daily application remained running as a separate process. No real API, WebDAV, synchronization, credential, system Node/ocx, or user configuration was used.

## Native IPC observation

With real Rust IPC active in the Tauri app, Overview → Export preview prepared an immutable plan for the sandbox's 18 built-in model/template objects. The payload reported zero credentials and zero Skill/MCP assets. Before confirmation, the old implementation wrote:

- manager-state/model-file-exports/d7b91a12-4738-4e38-a8bc-eb145511afdc.json — 126,648 bytes.
- manager-state/model-file-exports/d7b91a12-4738-4e38-a8bc-eb145511afdc.ocxdconf — 168,998 bytes.

After clicking Cancel, both files were absent; the sandbox's user-facing exports/ directory remained empty. This validates the discard command through native IPC, not only a mocked UI test.

A second native run opened another export preview and was terminated by SIGKILL using only the test app's exact executable PID while the unattempted plan was present. On restarting that same isolated app, startup log recorded “discarded 1 abandoned unattempted export preview(s)” and the plan directory was empty. No export attempt or persistent transfer result existed for either fixture.

## Implementation boundaries and tests

The follow-up branch codex/0.2.20-discard-cancelled-export-plan (commit 0389bcc0) adds a per-plan attempt lock shared by protect, destination export, and discard. Cancellation removes only the plan JSON, frozen package, and protection-source link for plans with no attempt or durable export-result receipt. An attempt/result created before destination writing protects the same package and ID for retry. The UI tracks every base/protected plan ID and discards them on cancel/close/Escape and clean unmount. Startup enumerates at most 1,536 files / 512 plan IDs and removes abandoned unattempted snapshots; malformed, linked, or unreadable files fail closed. Cleanup failure leaves the affected file and surfaces a generic error message.

Validation: six focused Rust file_exports tests pass; 37 focused ModelTransfer UI tests pass; full UI suite passes (177 files / 1,092 tests); vue-tsc, production UI build, Clippy with warnings denied, and rustfmt pass. A prior full serial Rust library run passed 1,164 with four ignored before the final receipt-before-destination guard test; the new six-test focused suite covers that guard. PR #37 current head is 1cbe4a6921613ed2a66f322918f85058c6294c56. Its CI run 41472954091 is running; frontend/backend/release-tools smoke evidence has passed or is queued, and Windows clippy is in progress. The earlier head run 41471655619 remains separately recorded.

The current v0.2.20 tag and draft packages do not include this fix. If merged for 0.2.20, rebuild/sign and repeat macOS/Windows package, installer, and adjacent-release OTA acceptance before stable. This follow-up does not upgrade the existing release.
