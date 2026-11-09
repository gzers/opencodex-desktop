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
> Public test releases are available for macOS Apple Silicon and Windows x64. macOS packages use ad-hoc signing, without Developer ID signing or Apple notarization; Windows installers have no Authenticode signature. Updater-asset signatures are verified independently. See Releases for the versions actually published.

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
| OpenCodex | Starting with 0.2.23, Desktop includes complete Node.js 26.8.1, npm 11.19.0 and maintained runtime 2.97.0-desktop023.lifecycle.3, prepared on first launch; no separate Node/npm installation or runtime archive import is required |

### Download

> [!NOTE]
> The link below points to the currently public Latest release. Candidate branches and draft Releases do not automatically enter the stable channel. See Releases for historical versions and their limitations.

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><b>⬇ Get the latest version</b></a>
</p>

- **Latest (always the newest)**: <https://github.com/gzers/opencodex-desktop/releases/latest>
- All releases and history: <https://github.com/gzers/opencodex-desktop/releases>

Starting with 0.2.23, Desktop includes complete Node.js 26.8.1, npm 11.19.0 and the project-maintained OpenCodex runtime `2.97.0-desktop023.lifecycle.3` (based on upstream 2.97.0, including the official web panel assets). First launch prepares the environment under `runtime/` in the actual installation directory. These patches are a separate project revision, not a claim that upstream has released or incorporated them. Desktop updates preserve the existing runtime; runtime upgrades require separate explicit confirmation, and online installation still uses the official upstream package. See each release for its supplied components, verification information and limitations.

Artifacts are produced by GitHub Actions when a tag is pushed:

| Platform | File | Notes |
|---|---|---|
| macOS Apple Silicon | `*.dmg` / `*.app` | Ad-hoc signed; no Developer ID signing or Apple notarization |
| Windows x64 | `*_x64-setup.exe` (NSIS) / `*.msi` | Unsigned test build |

### macOS

Starting with 0.2.23, macOS uses a separate user-writable installation folder:

1. Open the version's DMG and copy the entire top-level **OpenCodeX Desktop folder** to your user `~/Applications`, keeping its contents together.
2. Open `~/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app`.
3. First launch prepares `runtime/` beside the app in that folder. The app and runtime are siblings, separate from the configuration HOME.

Because the test build is **ad-hoc signed, without Developer ID signing or Apple notarization**, macOS may block it. Handle the two symptoms differently:

**Symptom A — "cannot be opened because Apple cannot check it for malicious software"**
→ Right-click (or Control-click) the app → *Open* → click *Open* again in the dialog.

**Symptom B — "is damaged and can't be opened. You should move it to the Trash"** (the most common block)

Right-click → *Open* **does not fix** this. Clear the quarantine attribute in Terminal:

```bash
# Recommended: remove only the download quarantine flag
sudo xattr -dr com.apple.quarantine "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

If it is still blocked, try in order:

```bash
# 1) Clear all extended attributes on the app
sudo xattr -c "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"

# 2) If it still fails, ad-hoc re-sign locally
sudo codesign --force --deep --sign - "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"

# 3) Open again
open "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

> Debugging aids: `xattr -l "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"` to inspect attributes, and `spctl -a -vv "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"` to see Gatekeeper's verdict.
> If you ran the app directly from the mounted DMG, first copy the entire folder to `~/Applications`, then run the commands above.

### Windows

1. Download `*_x64-setup.exe` (or `.msi`) and run it.
2. If the blue **SmartScreen** dialog appears, click *More info* → *Run anyway*.
3. The default program directory is `%LOCALAPPDATA%/Programs/OpenCodeX Desktop` for the current user; runtime preparation happens inside it. An explicitly chosen custom installation directory also remains valid.

> Windows installers have no Authenticode signature. Release records state the normal/admin, installation and data-retention scenarios actually verified. Default uninstall preserves data; explicit NSIS cleanup removes only validated owned roots and preserves external HOME.

---

## Getting started

1. **First launch**: wait for the bundled environment to be prepared in the installation directory. Settings shows actual installation, runtime, data-root and configuration paths. It does not search PATH, old runtimes or old data directories for a fallback installation.
2. **Start / stop / restart**: the Overview page hosts the official `ocx start / stop / restart` subprocess and surfaces port, process identity, and recent errors.
3. **Status**: split into three independent dimensions — **run / connection / operation** — that can hold at the same time; actions render per status, with no unexplained disabled buttons.
4. **Panel**: switch to *Panel* to view the official web panel (requires OpenCodex to be running).
5. **Diagnostics**: a read-only `ocx doctor` summary, logs (app log / call log, secrets masked), and notification history.
6. **Extensions**: manage **Skills** and **MCP** with distribution to six clients (Codex, Claude, Gemini, Grok, Opencode, Hermes); backup before write, atomic replace, per-client connect/disconnect.
7. **Model configuration**: define template matches using numeric versions, model families and name conditions. Reading existing channels automatically matches templates into a draft; review it and explicitly save to write runtime configuration. Overrides and protocols are per model, with no channel-wide protocol. Templates have immutable versions, with separate default and adoption choices.
8. **File import/export and sync**: select templates, channels, models, families or manager objects, preview differences, then confirm. The new v6 model package and ObjectsV2 object format have no old-format fallback. Credentials require a separate explicit selection. Encryption is optional; without it, the file contains plaintext credentials. The passphrase is not stored in the package. Accounts transfer only supported public information; sign in again on a new device.
9. **Data directory**: Settings shows actual paths. Explicit changes to the user aggregate root or `OPENCODEX_HOME` take effect after restart; old directories are not automatically scanned, adopted or migrated.
10. **Tray and menus**: status is expressed through tray icon and colour, plus native macOS menu entries.
11. **Optional CLI control plane**: enabling it registers `ocxd`, which delegates to the running instance over local IPC; **off by default**.


### Directories and upgrading to 0.2.23

Programs and the runtime use installation root `I`; user files use a separate aggregate root `U`:

| Directory | Default location and purpose |
|---|---|
| Installation root `I` | The actual `.app` parent on macOS, default `~/Applications/OpenCodeX Desktop`; the actual exe parent on Windows, default `%LOCALAPPDATA%/Programs/OpenCodeX Desktop`. Custom installations use their actual location |
| Runtime environment | `I/runtime/`, containing Node/npm, the maintained runtime and panel assets |
| User aggregate root `U` | `~/.codex-desktop` for the actual current user; independent on macOS, Windows and for each login user |
| Manager data root `D` | `U/desktop`, containing manager state, logs and backups |
| OpenCodex HOME `H` | `U/opencodex/home`; the configuration file is `H/config.json` |

Before upgrading, back up your existing OpenCodex `config.json` yourself. After the new directories initialize, put the backup at the **actual configuration-file path shown in Settings**. Read existing channels in Model configuration, review the automatic template-matching draft and explicitly save. The app does not automatically migrate old directories or fall back to old formats.

Upgrade every device using sync. After confirming the complete local source, follow the release notes to rebuild an empty sync directory. The sync tool does not delete a production remote directory on the user's behalf.

### Configuration files and settings

Version-frozen defaults and user choices are stored separately so upgrades never overwrite user preferences:

| File | Purpose | Location |
|---|---|---|
| `preferences.defaults.json` | Version-frozen default preferences (validated and embedded at build time) | Ships with the app; not edited by users |
| `runtime.defaults.json` | Version-frozen runtime policies (timeouts, retention, runtime entry, update endpoints) | Ships with the app; not edited by users |
| `network.defaults.json` | Version-frozen network behaviour policies (timeouts, retries, UA, TLS constraints) | Ships with the app; not edited by users |
| `manager-state/preferences.json` | User preferences (channel, scale, theme, effects changed in the UI) | Inside the data root, mode `0600` |
| `manager-location.json` | Current user’s selected aggregate-root location | Fixed `~/.codex-desktop/desktop/` directory |
| `manager-state/data-root.json` | Active data-root and instance HOME configuration | Inside active `D` |

Precedence and effect: user preferences take priority over version defaults; missing new fields fall back to defaults, while invalid or corrupted preferences fail explicitly and keep the original file instead of being silently reset or overwritten. UI fields take effect immediately or are marked pending restart; export/sync transmit only the migratable preference projection. Machine paths and run state stay local; resource credentials require a separate explicit selection.

Theme: the backend preference is the source of truth. On early startup a local cache renders the first frame, then the preference read-back takes over; the old cache never overrides a newer choice.

Update channel: the stable/beta channel is decoupled from "check automatically"; checks, installs, and background scheduling share one channel source, and switching the channel invalidates stale candidates. App self-update now points at this repository's real release manifest endpoints (stable reads `latest.json`, beta reads a fixed `beta` tag); installs happen only after signature verification, and a failure keeps the current version.

Network proxy: General settings offer no proxy / automatic (system proxy) / manual HTTP or SOCKS5 with an exception list; app self-update, official remote queries, and managed installs share it. This version has no proxy authentication and does not use the system keychain.

Configuration formats: app versions and configuration schema versions are separate, with preferences grouped by domain. Version 0.2.23 uses new configuration and object formats; old paths and formats are not automatically converted. Preserve unsupported originals and follow the relevant release notes.

Test isolation: development and automated tests run under a separate sandbox identity (own data root, credential service, and instance identity) and never write to the daily-use directory; if the sandbox root is missing, startup stops rather than falling back to daily configuration.

---

## Features

| Capability | Description |
|---|---|
| **Runtime supply** | Starting with 0.2.23, Desktop includes the complete environment and prepares it under actual `I/runtime` on first launch. Separate upgrades require confirmation, without PATH scanning or old-directory fallback |
| **Process control** | Hosts the official start/stop subprocesses directly; does not install or manage launchd and does not take over `codex-shim` |
| **Status observation** | Run / connection / operation dimensions that can hold simultaneously; actions render per status |
| **Logs and diagnostics** | Read-only logs with masked secrets; read-only `ocx doctor` summary; states clearly when live logs are unavailable |
| **Official panel** | Hosts the official web panel in a runtime child WebView, without restyling or injection; panel assets are included with the runtime |
| **Notification center** | Shares one data source with *Diagnostics → Notification history*; detail dialog and per-item deletion |
| **Model configuration** | Numeric versions, family/name conditions and immutable templates; reading creates only a matching draft, with comparison, backup and readback on save. Protocols and overrides are per model |
| **Template assistant** | Local execution settings, public sources and iterative correction; accept fields into the buffer. Scoped external tasks validate baselines/locks and save candidates without automatic adoption or runtime application |
| **File import/export** | Selected v6 / ObjectsV2 objects, without old-format fallback; credentials require separate explicit selection. Encryption is optional; without it, files contain plaintext credentials. The passphrase is not included in the package. Adoption requires preview and confirmation |
| **WebDAV sync** | Uses a trusted HTTPS service, files, commits and graph relationships without relying on ETag; explicit upload/download, backup before local updates, conflict handling and verification of the original operation after failure. Remote storage, local application, local-content relation and remote-graph relation are reported separately; relations reflect the current observation |
| **Extension management** | Skills discovery/import/update/uninstall/restore; MCP import/add/edit/delete; multi-client targets |
| **Appearance** | Three visual effects tiers; overview background light field (WebGL grid gradient + grain shader, CSS aurora fallback) and glass materials |
| **Two upgrade tracks** | Desktop updates verify updater signatures and preserve the existing runtime. Runtime upgrades require separate confirmation; online installation still uses official upstream packages |
| **Tray and menus** | Tray actions limited to the manager's own domain; native macOS menu entries |
| **CLI control plane (optional)** | `ocxd` is off by default; when enabled on macOS it delegates over local IPC. Windows product preferences still disable CLI access |

### What it deliberately does not do (security boundaries)

- It does not rewrite the OpenCodex core and does not bypass the official API.
- It does not take over npm or replace the official `ocx update` transaction.
- It does not take over `codex-shim`, and does not install or manage launchd.
- Model configuration drafts come from actual saved sources. Explicit saves compare, validate and back up before writing, then read back. Templates, drafts and runtime configuration retain separate ownership; protocols are per model.
- It never silently overwrites configuration owned by an external provider; it detects, explains, and guides the user through the official `restore`.
- Keys stay out of the repository, Markdown, logs and command-line arguments; unselected credentials are not exported. Explicitly choosing unencrypted export produces a readable file containing plaintext credentials, as selected by the user.
- The WebDAV remote is always treated as **untrusted**; TLS validation failures are rejected by default, with **no "ignore certificate" option**.

---

## FAQ and troubleshooting

<details>
<summary><b>macOS says the app "is damaged and can't be opened"</b></summary>

An ad-hoc-signed build without Apple notarization may be quarantined by Gatekeeper. Right-click → *Open* does not fix the “damaged” symptom; run:

```bash
sudo xattr -dr com.apple.quarantine "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

If that still fails, use `sudo xattr -c`, then `sudo codesign --force --deep --sign - "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"`. See [macOS](#macos).
</details>

<details>
<summary><b>Windows shows SmartScreen / unknown publisher</b></summary>

The build is unsigned. Click *More info* → *Run anyway*; signing is planned for a later release.
</details>

<details>
<summary><b>Overview says "OpenCodex not found"</b></summary>

Starting with 0.2.23, the environment ships with Desktop. Check runtime preparation status and errors, and confirm the installation directory is writable. Use the actual runtime entry shown in Settings; old directories and global installations are not scanned as fallbacks.
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
<summary><b>Data directory is not writable / a switch has not taken effect</b></summary>

Check the actual aggregate root and `OPENCODEX_HOME` in Settings; explicit changes require a restart. The target must be writable and separate from the installation directory. The app does not move or scan old directories automatically; back up, place and read old configuration using the upgrade steps above.
</details>

<details>
<summary><b>WebDAV sync reports TLS / certificate errors</b></summary>

By design there is **no "ignore certificate" option**. Install the self-signed certificate into the local trust store, or use a trusted endpoint.
</details>

<details>
<summary><b>Keychain / credential prompt</b></summary>

WebDAV connection passwords and saved content-protection passphrases are stored under `D/manager-state/credentials/`: macOS uses private `0600` files and `0700` directories; Windows uses current-user DPAPI protection. Private model keys retain their bound source and native credential store. The local sealing key for recovering an original file operation is stored separately in the native keyring under `opencodex.file-transfer-recovery.v2`. System authorization prompts may come from access to model keys or this recovery key.
</details>

<details>
<summary><b>Where are the logs and data?</b></summary>

Both platforms default to the actual current user’s `~/.codex-desktop`: manager data is under `desktop/`, OpenCodex configuration is `opencodex/home/config.json`, and logs are under active `D/logs/`. Settings shows selected paths; each user is independent, while programs and runtime stay in the actual installation directory. Preference backups exclude full panel configuration, sessions and login state.
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
- **Known limitations**: macOS uses ad-hoc signing without Developer ID signing or Apple notarization; Windows has no Authenticode signature. Intel macOS builds and Windows CLI are disabled. Accounts transfer only supported public information and require login on a new device. A runtime save does not prove that Codex loaded it; use the client-check entry. Historical unverified scenarios and resource exceptions retain their original release dispositions.

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
- Compatibility descriptions and prototype references use the OpenCodex name, version, links and interface material. Starting with 0.2.23, installers also include the complete environment and maintained runtime; the upstream software and panel assets retain their own licenses, and maintained revisions are not upstream releases.
- The OpenCodex source code, name, trademarks, and other rights remain with their rights holders; when using the official software, refer to the `LICENSE`, `NOTICE`, and terms of service in the official repository.
- This project's own code and assets are released under the **MIT License** (see [`LICENSE`](LICENSE)). The two licenses are **independent** of each other.
- The `docs/` tree **includes a small amount of official source material** (official page snapshots, official UI screenshots, official brand marks) for prototype and compatibility reference only. That material is **not** covered by this project's MIT license, remains the property of its rights holders, and must not be redistributed apart from this project or used as a trademark.
