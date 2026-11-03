# v0.2.20 release package and isolated macOS UI evidence

Date: 2026-11-02 (America/Los_Angeles)
Release source: `v0.2.20` → `ca434f7dd3490fbd900650fde626d524c7a9757c`.

## Exact draft release assets

Downloaded the eight assets from the draft `v0.2.20` release into `/tmp/ocx020-release-assets`. Every downloaded SHA-256 matched the GitHub Releases API digest. The macOS DMG checksum is valid (`hdiutil verify`). The `.app.tar.gz` passed `codesign --verify --deep --strict`; packaged `LICENSE` and `THIRD-PARTY-NOTICES.txt` are present. The bundle signature is ad-hoc (`TeamIdentifier=not set`), matching the repository's current signing policy.

| Asset | SHA-256 |
| --- | --- |
| `OpenCodeX.Desktop_0.2.20_aarch64.dmg` | `4186be0e880a7bfb987a0af0c5005ba47671b495763d1dfec23fd864c98b5ce9` |
| `OpenCodeX.Desktop_aarch64.app.tar.gz` | `4a0ef2773f2f653c9fd684de1af42d760c0d0fc7f001d1a20b99f5fbf2de68b8` |
| `OpenCodeX.Desktop_0.2.20_x64_en-US.msi` | `9054739868df77ce61812b70a6b42b663c3c1198a53078a3e9a296b9d851ff7f` |
| `OpenCodeX.Desktop_0.2.20_x64-setup.exe` | `6bcacd17bfd89612ff6e50be88c96c32edb8301f614490e6cd95ff626de7ec64` |
| `OpenCodeX.Desktop_0.2.20_x64_en-US.msi.sig` | `983fd4070c63f2c78ebafe861e78db3c75ca7b872005ef2df65bf484ac5bd339` |
| `OpenCodeX.Desktop_0.2.20_x64-setup.exe.sig` | `9d9cedfd0c8f5dd68f7a1beb91305be371d6a8be1e20401f5e8ef9fbdc105a39` |
| `OpenCodeX.Desktop_aarch64.app.tar.gz.sig` | `890937e627d502de00a7a3981dad04abd204b9af6d1dbcd9b6c14e6ad70ecf5c` |
| `latest.json` | `8efa7e5af80c22ccdbc2eb264ecf4d32b08e71b5f8c42f32325829b34fd61176` |

Using the updater public key from the tagged `tauri.conf.json`, minisign verification passed for the macOS updater archive, MSI, and NSIS package. These are updater signatures; they do not constitute Windows Authenticode signatures or Apple notarization.

## Native UI scope

The exact extracted release bundle passed signature and resource checks before UI testing. To attach macOS accessibility to a disposable instance while the daily app with the same bundle ID remained running, a copy under `/tmp/ocx020-native-ui-test` was given a test-only bundle ID and ad-hoc re-signed. The compiled executable's raw SHA changed with signing metadata; after accounting for the two `__LINKEDIT` virtual-size bytes changed by the different signature size, the other executable bytes matched. The original release package was left untouched.

The QA app first remained at the missing-Node prerequisite. For further read-only navigation, a temporary HOME was provisioned with Node/npm symlinks and an inert local ocx fixture that only returns a version and refuses all commands. The UI then reached the normal overview and Settings. CUA inspection observed the file export preview (default encryption off, protected fields after opt-in), import file chooser, WebDAV connection fields/cancel, Settings data-and-backup, empty backup cleanup preview, notification center/detail, model channel/new-channel, template editor/version management/match-use/Agent model picker, Skills/MCP/client views and Codex detail. No backup mutation, notification clear/delete, save, import, WebDAV test/connect/sync, or uninstall was executed. No user-facing export package was generated.

The browser route audit and limited native UI observations do not close remaining dialog states, saved-file/backup recovery, or OTA. The native screenshot/UI session was inspected live; no screenshot file was retained.

## Release gate still open

Release workflow `41429792658` is a failure and must remain recorded as such. The default MSI/NSIS checks passed, but custom MSI testing installed into the remembered default directory and then failed to find the executable at the requested custom path. PR #35 contains the disposable-runner registry isolation correction. A code review also found that the exact-package acceptance script's allowlist omitted `0.2.20`; commit `97c5aedd` adds it and a regression test. CI runs `41439071945` and `41439302028` are still in progress at the time of this evidence. Stable promotion and Windows OTA have not been performed.

## Export-preview cache behavior found during native QA (follow-up)

Opening the overview Export dialog called the production prepare path and wrote a frozen plan under manager-state/model-file-exports before the final Generate package action. The observed plan contained 18 model/template objects, 0 credentials, and no Skill/MCP assets. The plan metadata was 126,650 bytes (SHA-256 df8f95437469ade53521633cd322401ebe25c430720a9d47109e8291e68809a5); the frozen .ocxdconf payload was 168,998 bytes (SHA-256 202573dcb15967254e537cbffe4df6b823cc04051923597a7e9d133870d398c0). No package was written to the user-facing exports directory because the dialog was canceled. The released build left the internal frozen cache files behind after cancel.

The contract classifies this as a temporary frozen plan, distinct from a business-file write. Follow-up branch codex/0.2.20-discard-cancelled-export-plan adds a lock-protected discard that preserves any plan with an export-attempt record, tracks plaintext and protected-child plan IDs, and prunes abandoned unattempted plans at startup. Isolated Rust tests cover cancellation cleanup, protected child/source removal, attempt preservation, and startup pruning; UI tests cover cancellation wiring. These changes are not in the v0.2.20 tag/release yet and require review, CI, new package build/signature/install/OTA acceptance before they can be part of a release.
