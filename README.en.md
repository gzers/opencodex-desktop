<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/logo.png" width="120" alt="OpenCodeX Desktop" />
</p>

<h1 align="center">OpenCodeX Desktop</h1>

<p align="center">
  Discovery, process control, status observation, model configuration, sync, and extension management for OpenCodex — in a desktop GUI instead of a terminal.
</p>

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><img src="https://img.shields.io/github/v/release/gzers/opencodex-desktop?include_prereleases&sort=semver&label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="MIT">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey.svg" alt="Platform">
  <img src="https://img.shields.io/badge/Rust-1.89%2B-orange.svg" alt="Rust 1.89+">
  <img src="https://img.shields.io/badge/Node-%E2%89%A524-339933.svg" alt="Node 24+">
</p>

<p align="center"><a href="README.md">简体中文</a> · English</p>

> [!IMPORTANT]
> This is an **independent, third-party desktop manager**. It is not part of the official OpenCodex project and has **no affiliation, authorization, or endorsement** relationship with it or its rights holders. It is a graphical host only — it **does not rewrite the OpenCodex core and does not bypass the official API**. See [Attribution and license](#attribution-and-license).
>
> Public test releases are available for macOS Apple Silicon and Windows x64. Installers have no formal OS code signature or Apple notarization; updater assets are verified separately. See Releases for the versions actually published.

---

## Screenshots

<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/overview-light.jpg" width="82%" alt="Overview (light)">
</p>

<table>
  <tr>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/overview-dark.jpg" alt="Overview (dark)"></td>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/panel-dark.jpg" alt="Panel (dark)"></td>
  </tr>
  <tr>
    <td align="center"><sub>Overview · dark</sub></td>
    <td align="center"><sub>Panel (embedded official web panel) · dark</sub></td>
  </tr>
</table>

> The app UI is currently **Chinese only**. Light/dark follows the system appearance, and there are three visual effects tiers (high / mid / low).

---

## Download and install

### Requirements

| Item | Requirement |
|---|---|
| OS | macOS (Apple Silicon, officially supported) / Windows x64 (test build) |
| OpenCodex | Node.js/npm and official OpenCodex; select an existing install or explicitly approve a private-prefix managed install or an official offline package. Third-party lifecycle scripts require separate permission |

### Download

> [!NOTE]
> The link below points to the currently public Latest release. Candidate branches and draft Releases do not automatically enter the stable channel. See Releases for historical versions and their limitations.

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><b>⬇ Get the latest version</b></a>
</p>

- **Latest (always the newest)**: <https://github.com/gzers/opencodex-desktop/releases/latest>
- All releases and history: <https://github.com/gzers/opencodex-desktop/releases>

Artifacts are produced by GitHub Actions when a tag is pushed:

| Platform | File | Notes |
|---|---|---|
| macOS Apple Silicon | `*.dmg` / `*.app` | Unsigned, not notarized |
| Windows x64 | `*_x64-setup.exe` (NSIS) / `*.msi` | Unsigned test build |

### macOS

1. Download the `.dmg`, open it, and drag **OpenCodeX Desktop** into *Applications*.
2. Launch it from *Applications*.

Because the build is **unsigned and not notarized**, macOS may block it. Handle the two symptoms differently:

**Symptom A — "cannot be opened because Apple cannot check it for malicious software"**
→ Right-click (or Control-click) the app → *Open* → click *Open* again in the dialog.

**Symptom B — "is damaged and can't be opened. You should move it to the Trash"** (the most common block)

Right-click → *Open* **does not fix** this. Clear the quarantine attribute in Terminal:

```bash
# Recommended: remove only the download quarantine flag
sudo xattr -dr com.apple.quarantine "/Applications/OpenCodeX Desktop.app"
```

If it is still blocked, try in order:

```bash
# 1) Clear all extended attributes on the app
sudo xattr -c "/Applications/OpenCodeX Desktop.app"

# 2) If it still fails, ad-hoc re-sign locally
sudo codesign --force --deep --sign - "/Applications/OpenCodeX Desktop.app"

# 3) Open again
open "/Applications/OpenCodeX Desktop.app"
```

> Debugging aids: `xattr -l "/Applications/OpenCodeX Desktop.app"` to inspect attributes, and `spctl -a -vv "/Applications/OpenCodeX Desktop.app"` to see Gatekeeper's verdict.
> If you ran the app directly from inside the mounted dmg instead of dragging it to *Applications*, move it first, then run the commands above.

### Windows

1. Download `*_x64-setup.exe` (or `.msi`) and run it.
2. If the blue **SmartScreen** dialog appears, click *More info* → *Run anyway*.

> Windows installers have no Authenticode signature. Release records state the normal/admin, installation and data-retention scenarios actually verified. Default uninstall preserves data; explicit NSIS cleanup removes only validated owned roots and preserves external HOME.

---

## Getting started

1. **First launch**: the Overview page shows discovery results and run status. If OpenCodex is not found, follow the on-screen guidance to install the official OpenCodex first.
2. **Start / stop / restart**: the Overview page hosts the official `ocx start / stop / restart` subprocess and surfaces port, process identity, and recent errors.
3. **Status**: split into three independent dimensions — **run / connection / operation** — that can hold at the same time; actions render per status, with no unexplained disabled buttons.
4. **Panel**: switch to *Panel* to view the official web panel (requires OpenCodex to be running).
5. **Diagnostics**: a read-only `ocx doctor` summary, logs (app log / call log, secrets masked), and notification history.
6. **Extensions**: manage **Skills** and **MCP** with distribution to six clients (Codex, Claude, Gemini, Grok, Opencode, Hermes); backup before write, atomic replace, per-client connect/disconnect.
7. **Model configuration**: keep channel models, nodes, per-model rules and ordering in a draft. Save template edits as new versions; default selection, adoption and runtime save are separate confirmations. The assistant uses a configured execution model, and external tasks copy their instructions and contract together.
8. **Migration and sync**: select templates, channels, models, families or manager objects, then compare before adopting. Credentials require encrypted content. Only supported account information transfers; sign in again on a new device. WebDAV backs up before overwrite and preserves conflicts for review.
9. **Data directory**: choose or switch the data root and `OPENCODEX_HOME` (reference-first, migration optional, rollback on failure).
10. **Tray and menus**: status is expressed through tray icon and colour, plus native macOS menu entries.
11. **Optional CLI control plane**: enabling it registers `ocxd`, which delegates to the running instance over local IPC; **off by default**.


### Configuration files and settings

Version-frozen defaults and user choices are stored separately so upgrades never overwrite user preferences:

| File | Purpose | Location |
|---|---|---|
| `preferences.defaults.json` | Version-frozen default preferences (validated and embedded at build time) | Ships with the app; not edited by users |
| `runtime.defaults.json` | Version-frozen runtime policies (timeouts, retention, discovery paths, update endpoints) | Ships with the app; not edited by users |
| `network.defaults.json` | Version-frozen network behaviour policies (timeouts, retries, UA, TLS constraints) | Ships with the app; not edited by users |
| `manager-state/preferences.json` | User preferences (channel, scale, theme, effects changed in the UI) | Inside the data root, mode `0600` |
| `manager-state/data-root.json` | Local data-root location and structure version | Inside the data root |
| `manager-state/config-migrations/` | Original backups and migration records for automatic format conversion | Inside the data root; only present after a conversion |

Precedence and effect: user preferences take priority over version defaults; missing new fields fall back to defaults, while invalid or corrupted preferences fail explicitly and keep the original file instead of being silently reset or overwritten. UI fields take effect immediately or are marked pending restart; export/sync transmit only the migratable preference projection — machine paths, credentials, and run state stay local.

Theme: the backend preference is the source of truth. On early startup a local cache renders the first frame, then the preference read-back takes over; the old cache never overrides a newer choice.

Update channel: the stable/beta channel is decoupled from "check automatically"; checks, installs, and background scheduling share one channel source, and switching the channel invalidates stale candidates. App self-update now points at this repository's real release manifest endpoints (stable reads `latest.json`, beta reads a fixed `beta` tag); installs happen only after signature verification, and a failure keeps the current version.

Network proxy: General settings offer no proxy / automatic (system proxy) / manual HTTP or SOCKS5 with an exception list; app self-update, official remote queries, and managed installs share it. This version has no proxy authentication and does not use the system keychain.

Automatic format conversion: the app version and the configuration schema version are managed separately. The preferences file is grouped by domain (`appearance` / `shell` / `backup` / `extensions` / `maintenance` / `sync` / `updates` / `network` / `cli`); reading an old flat format runs the published migration chain automatically and keeps a backup of the original; a version newer than the app supports is refused and the original is preserved — restore from the pre-migration backup if needed (this discards changes made after the migration).

Test isolation: development and automated tests run under a separate sandbox identity (own data root, credential service, and instance identity) and never write to the daily-use directory; if the sandbox root is missing, startup stops rather than falling back to daily configuration.

---

## Features

| Capability | Description |
|---|---|
| **Installation discovery** | Discovers existing installs and manages official packages in a private prefix after explicit confirmation, without writing the global npm prefix. Lifecycle scripts require separate permission |
| **Process control** | Hosts the official start/stop subprocesses directly; does not install or manage launchd and does not take over `codex-shim` |
| **Status observation** | Run / connection / operation dimensions that can hold simultaneously; actions render per status |
| **Logs and diagnostics** | Read-only logs with masked secrets; read-only `ocx doctor` summary; states clearly when live logs are unavailable |
| **Official panel** | Hosts the official web panel in a runtime child WebView — no restyling, no injection, and the official assets are not bundled |
| **Notification center** | Shares one data source with *Diagnostics → Notification history*; detail dialog and per-item deletion |
| **Model configuration** | Channels and immutable template versions, exact adoption, inherited fields, nodes and two-level ordering; compare, back up and read back before completing a runtime save |
| **Template assistant** | Local execution settings, public sources and iterative correction; accept fields into the buffer. Scoped external tasks validate baselines/locks and save candidates without automatic adoption or runtime application |
| **Configuration migration** | Full export into a **passphrase-protected encrypted container**; validate, back up automatically, confirm explicitly, roll back on failure |
| **WebDAV sync** | Stores configuration payloads and integrity metadata on a trusted HTTPS service; credentials remain in the OS credential store. Backs up before overwrite and requires conflict confirmation. File export containers are separately passphrase-encrypted |
| **Extension management** | Skills discovery/import/update/uninstall/restore; MCP import/add/edit/delete; multi-client targets |
| **Appearance** | Three visual effects tiers; overview background light field (WebGL grid gradient + grain shader, CSS aurora fallback) and glass materials |
| **Two upgrade tracks** | Manager and panel updates have separate confirmation. App updates use this repository's actual release manifests and verify updater signatures; panel updates use protected official-package installation |
| **Tray and menus** | Tray actions limited to the manager's own domain; native macOS menu entries |
| **CLI control plane (optional)** | `ocxd` is off by default; when enabled on macOS it delegates over local IPC. Windows product preferences still disable CLI access |

### What it deliberately does not do (security boundaries)

- It does not rewrite the OpenCodex core and does not bypass the official API.
- It does not take over npm or replace the official `ocx update` transaction.
- It does not take over `codex-shim`, and does not install or manage launchd.
- Model configuration submits provider, routing and model-mapping changes through the official CLI, with comparison, budget validation, backup and readback. The manager template library, draft and runtime configuration retain separate ownership.
- It never silently overwrites configuration owned by an external provider; it detects, explains, and guides the user through the official `restore`.
- No credential may enter the repository, Markdown, logs, command-line arguments, or unencrypted files.
- The WebDAV remote is always treated as **untrusted**; TLS validation failures are rejected by default, with **no "ignore certificate" option**.

---

## FAQ and troubleshooting

<details>
<summary><b>macOS says the app "is damaged and can't be opened"</b></summary>

An unsigned build gets the `com.apple.quarantine` attribute from Gatekeeper. Right-click → *Open* does not fix this; run:

```bash
sudo xattr -dr com.apple.quarantine "/Applications/OpenCodeX Desktop.app"
```

If that still fails, use `sudo xattr -c`, then `sudo codesign --force --deep --sign - "/Applications/OpenCodeX Desktop.app"`. See [macOS](#macos).
</details>

<details>
<summary><b>Windows shows SmartScreen / unknown publisher</b></summary>

The build is unsigned. Click *More info* → *Run anyway*; signing is planned for a later release.
</details>

<details>
<summary><b>Overview says "OpenCodex not found"</b></summary>

Follow Overview to provide Node.js/npm, then select an existing OpenCodex install or explicitly approve a private-prefix install/offline import. Third-party lifecycle scripts require separate permission.
</details>

<details>
<summary><b>Panel page is blank / fails to load</b></summary>

The panel hosts the **official web panel**, which needs OpenCodex running and its port reachable. Start OpenCodex on the Overview page first; the panel being unavailable while the agent is offline is expected.
</details>

<details>
<summary><b>Live logs unavailable</b></summary>

Live logs require a running agent. When it is offline the UI says so explicitly; local historical logs and "open log directory" still work.
</details>

<details>
<summary><b>Data directory is not writable / migration failed</b></summary>

Switching the data root or `OPENCODEX_HOME` checks writability and space, and forces a backup before migrating; failures roll back and keep the original directory. Make sure the target is a real directory (not a symlink), outside system-protected locations, and writable.
</details>

<details>
<summary><b>WebDAV sync reports TLS / certificate errors</b></summary>

By design there is **no "ignore certificate" option**. Install the self-signed certificate into the local trust store, or use a trusted endpoint.
</details>

<details>
<summary><b>Keychain / credential prompt</b></summary>

Sync and migration credentials are stored in the macOS Keychain (service name `OpenCodeX Desktop`); the first access prompts for authorization. This is expected.
</details>

<details>
<summary><b>Where are the logs and data?</b></summary>

The macOS default is `~/Library/Application Support/com.gzers.opencodex.desktop/`. New Windows installs use `data` under the actual installation directory; valid existing bindings are retained. Settings shows actual paths, with logs under `logs/`. Preference backups exclude full panel configuration, sessions and login state.
</details>

<details>
<summary><b>App auto-update does nothing</b></summary>

Use software-update settings to check manually and inspect the error and network configuration. Stable reads this repository's Latest manifest; no higher candidate means there is nothing to install. Automatic checks only check and notify, without automatic download or restart; installation requires signature verification.
</details>

---

## For developers

### Architecture

```text
┌─────────────────────────────────────────────┐
│ Frontend (Vue 3 + TypeScript)                 │
│ Overview / Panel / Models / Sync / Extensions / Diagnostics / Settings / Tray │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC commands
┌──────────────────▼──────────────────────────┐
│ Manager backend (Rust)                        │
│ Discovery · Process · Status · Logs · Migration · Sync │
│ Extensions · Update · Data root · CLI/IPC · Notifications │
└──────────────────┬──────────────────────────┘
                   │ subprocess / files / network
┌──────────────────▼──────────────────────────┐
│ External dependencies                         │
│ ocx CLI · official web panel · client configs │
│ WebDAV remote · app update channel            │
└─────────────────────────────────────────────┘
```

**Hard boundary: the backend exclusively owns external access.** The frontend never touches the filesystem or network directly; every external call goes through a backend module.

- **Stack**: Tauri v2 + Rust backend; Vue 3 + TypeScript + Vite + Pinia frontend; native CSS design tokens (no UI framework).
- **Frontend layout**: feature slices — `app/` (bootstrap and appearance), `features/<domain>/` (own IPC and business components), `components/` (`ui` / `layout` / `patterns`), `routes/` (composition only), `stores/`, `navigation/`, `lib/`, `contracts/`.
- **Backend modules**: `MOD-01` discovery … `MOD-13` tray/menu (see the module table in the docs).
- **Controlled writes**: user action → pre-checks → backup (blocks on failure) → temp write + atomic replace → re-check → success / rollback; writes hold a cross-process file lock and detect external modification.

Full architecture, domain model, and contracts: [`docs/02-项目核心/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/02-项目核心) (Chinese).

### Repository layout

```text
apps/desktop/ui/        # Frontend (Vue 3 + TypeScript + Vite)
apps/desktop/tauri/     # Backend (Rust + Tauri v2)
.github/workflows/      # CI (ci.yml) and release (release.yml)
test/                   # Local acceptance test scripts and sources
```

### Local development

```bash
# Requires Rust 1.89+ and Node >= 24
npm --prefix apps/desktop/ui ci
npm install --global @tauri-apps/cli@2.11.4

# Frontend dev server
npm --prefix apps/desktop/ui run dev

# Run the full desktop app (separate terminal)
cd apps/desktop/tauri && tauri dev
```

### Build and package

```bash
npm --prefix apps/desktop/ui ci
cd apps/desktop/tauri
tauri build --target aarch64-apple-darwin   # macOS Apple Silicon; output under target/<triple>/release/bundle/
```

On a Windows runner, `tauri build` produces NSIS / MSI (`bundle.targets` is `"all"`).

### Test gates

```bash
# Frontend
npm --prefix apps/desktop/ui run typecheck
npm --prefix apps/desktop/ui run test -- --run
npm --prefix apps/desktop/ui run build

# Backend (must run inside apps/desktop/tauri)
cd apps/desktop/tauri
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --features integration-test
```

### Releasing

1. Add a `## [X.Y.Z] - date` section to `CHANGELOG.md` (version **without** the `v`) — it is the source of truth for release notes.
2. Align the version in `apps/desktop/tauri/tauri.conf.json` and `Cargo.toml`.
3. Tag and push:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

Pushing a version tag triggers the Release workflow: it extracts the matching changelog, builds signed updater assets and creates a draft prerelease. Windows verifies the actual installers on an ephemeral runner. After checking the artifacts, run Publish Release with the version tag and choose stable or preserve-latest; the publisher verifies signatures, makes the release public and checks the anonymous manifest.

### Branch model

| Branch | Contents |
|---|---|
| `main` | **Code**: `apps/`, `.github/`, `test/`, and root config |
| `docs/governance-main` | **Docs and governance artifacts**: `docs/`, `.adg/` |

When documenting implementation status, only reference the implementation branch and real evidence — never copy source.

---

## Project status and known limitations

- Public test versions exist; Releases Latest identifies the actual stable delivery. Candidate branches and draft Releases do not count as public publication.
- Requirements, prototype, and implementation contracts are finalized and passed the review gates; all 13 capability domains are implemented, with multiple rounds of real-device acceptance on macOS.
- **Known limitations**: no formal OS code signing or Apple notarization; Intel macOS builds and Windows CLI are disabled. Account transfer includes only supported information. The current full-runtime catalog budget is 100 rows; exceeding it blocks writes and preserves drafts. Unsupported Responses/Anthropic advanced reasoning constraints on OpenCodex2.84.0 are rejected before writes. A runtime save does not prove that Codex loaded it; use the client-check entry. Historical unverified scenarios and resource exceptions keep their original release dispositions.

---

## Documentation

- Project fact entry: [`docs/README.md`](https://github.com/gzers/opencodex-desktop/blob/docs/governance-main/docs/README.md)
- Authoritative requirements (DMD): [`docs/01-需求管理/需求/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/01-需求管理)
- Project core (architecture / domain model / contracts): [`docs/02-项目核心/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/02-项目核心)
- Implementation (IMP / REL gates): [`docs/03-开发实施/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/03-开发实施)
- Changelog: [`CHANGELOG.md`](https://github.com/gzers/opencodex-desktop/blob/main/CHANGELOG.md)

> Governance and requirements documents are written in Chinese.

---

## Attribution and license

- This project is an **independent desktop manager** with no affiliation, authorization, or endorsement relationship with the official OpenCodex project or its rights holders.
- The OpenCodex name, version, links, and interface reference assets are used only as needed for compatibility descriptions and prototype fidelity; its **source code and build artifacts are not copied or redistributed**.
- The OpenCodex source code, name, trademarks, and other rights remain with their rights holders; when using the official software, refer to the `LICENSE`, `NOTICE`, and terms of service in the official repository.
- This project's own code and assets are released under the **MIT License** (see [`LICENSE`](LICENSE)). The two licenses are **independent** of each other.
- The `docs/` tree **includes a small amount of official source material** (official page snapshots, official UI screenshots, official brand marks) for prototype and compatibility reference only. That material is **not** covered by this project's MIT license, remains the property of its rights holders, and must not be redistributed apart from this project or used as a trademark.
