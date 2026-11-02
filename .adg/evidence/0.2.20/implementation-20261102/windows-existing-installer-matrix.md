# v0.2.20 existing Windows installer matrix

Date: 2026-11-02. Target release source `ca434f7dd3490fbd900650fde626d524c7a9757c`; original Windows build executable SHA-256 `1e06180a65a7cb4cdd4c00bdc48915f1e72c1a114fb7d3537af1febee85dbc7a`.

The matrix used the already uploaded v0.2.20 Release assets. Each job verified the GitHub asset digest, recovered the embedded original executable identity using the supplied build hash, installed the exact package, ran the isolated first-screen smoke, and uninstalled. Reports state `existingAssetsUnchanged=true`, `rebuilt=false`; these jobs did not rebuild or edit the release assets.

| Installer | Destination | Final result |
| --- | --- | --- |
| MSI | default | pass (initial job failed to find startup log once; the failed matrix job rerun passed) |
| MSI | custom | pass |
| NSIS | default | pass |
| NSIS | custom | pass |

Successful integrated matrix: [run 41448710359](https://github.com/gzers/opencodex-desktop/actions/runs/41448710359), head `d35d8ef7b2799e2376b8d79bd241a84ad61cb56c`. All four artifact reports identify the original assets: MSI `9054739868df77ce61812b70a6b42b663c3c1198a53078a3e9a296b9d851ff7f`; NSIS `6bcacd17bfd89612ff6e50be88c96c32edb8301f614490e6cd95ff626de7ec64`. Installed EXE identities matched in all four cells. The default uninstall retained its seeded data sentinel; the explicit isolated cleanup helper removed only its test roots.

Earlier failed runs remain separate evidence: 41446111057 had three passes and one NSIS custom startup-log failure. Diagnostic rerun 41447323857 exposed the inventory helper attempting to hash a live WebView lockfile. Run 41447830049 fixed that but its path separator filter still included WebView cache entries; one MSI default cell also reported no sandbox files. These were not overwritten. The final integrated run on 41448710359 passed all four cells; its failed MSI default attempt was rerun within the same workflow and passed.

Scope is limited to disposable-runner installation, exact executable identity, first-screen responsiveness, explicit test-root cleanup, uninstall, and default-uninstall retention. This does not cover human upgrade/DPI acceptance, Apple notarization, Windows Authenticode, OTA, or publication. Release remains a draft; OTA and stable promotion are not represented as complete.
