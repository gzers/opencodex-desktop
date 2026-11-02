# PR #38 Windows MSI default-path regression

Date: 2026-11-02. Source: main commit `20af7cb5b1aa58da301263911d769270d5daaaeb` (merge of PR #38).

The Windows installer smoke now clears the product-owned HKCU `InstallDir` before both default MSI installation and explicit-path MSI installation. This prevents WiX AppSearch from redirecting the default check to a previous runner path. NSIS behavior is unchanged. An offline regression test asserts cleanup occurs before each MSI invocation.

Local validation on PR head `fc625c622c1b95a17b65378c5e37a0454dcd0208`: release-tools, Windows-release-acceptance, Windows ETW admission, and Windows retention CI-matching unittest groups passed (51 tests total); `git diff --check` passed.

PR #38 CI run `41486352670` completed successfully: release-tools, frontend, backend, Windows backend (including native credentials, managed upgrade/offline reinstall, release EXE smoke and ordinary-account smoke), and Windows smoke-evidence checks all passed. PR #38 was merged into main as `20af7cb5b1aa58da301263911d769270d5daaaeb`.

The unpublished `v0.2.20` draft/tag was recreated at that commit to produce a fresh candidate. macOS arm64 candidate asset digests matched the release API; the DMG checksum, updater archive minisign signature, app bundle signature, and packaged license files passed local verification. The bundle signature remains ad-hoc (`TeamIdentifier=not set`).

Release run `41494490495` produced source `20af7cb5b1aa58da301263911d769270d5daaaeb`, built EXE SHA-256 `70d057b7176eab5fa382dfc0496c274690b3f60211914a230d22b97b814de22f`, MSI SHA-256 `30cec712ef431426e17cba766c377849045ad82976186e6acf8fbba2a08d5c85`, and NSIS SHA-256 `0c3beccc6868ccbf2cf1972c13187488c3c2403ad801634b12a7088a8a450818`. Default MSI and NSIS installation/startup/uninstall checks passed. The custom MSI case failed: WiX AppSearch changed the requested temporary `INSTALLDIR` to `C:\Users\runneradmin\AppData\Local\OpenCodeX Desktop`, then installed and uninstalled successfully at that remembered location. The failing MSI log and `installer-verification.json` are retained under `/tmp/ocx020-rerel-win-release-evidence-2`.

PR #39 (`codex/0.2.20-fix-msi-registry-views`, head `cb7e5d9f`) extends the per-MSI cleanup to HKCU Registry32 and Registry64 and adds a regression assertion. The current candidate must not be treated as passing its four-cell matrix until a new Windows run proves custom MSI uses the requested destination. Release run `41494490495` is therefore a failure and is preserved as such. The draft has not been promoted to stable; a new asset build, complete MSI/NSIS matrix, macOS native acceptance, and adjacent-version OTA remain open.
