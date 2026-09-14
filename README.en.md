# OpenCodeX-Desktop

English | [简体中文](README.md)

> An independent desktop manager for OpenCodex: discovery, process control, status observation, configuration migration, sync, and extension management — all in a GUI instead of a terminal.

**Status: requirements stage complete, no implementation yet.** This repository currently contains requirements, prototype, and governance documents only. The implementation contract (IMP) has not been created, and there is no runnable program or installable build.

---

## What this is

OpenCodex is currently started and observed mainly from the command line. **OpenCodeX-Desktop** is a desktop shell built with **Tauri v2 + Rust** (macOS first) that:

- discovers an existing local OpenCodex installation and shows its path, version, and executable validation;
- hosts the official `ocx start / stop / restart` subprocesses and surfaces status, port, process identity, and recent errors;
- provides logs, runtime diagnostics, and a notification center;
- performs encrypted configuration export/import and WebDAV sync;
- manages **Skills** and **MCP** extension configuration and distributes it across multiple clients;
- offers a tray icon, native menus, and an optional command-line control plane.

This project is **not** part of the official OpenCodex project and has **no affiliation, authorization, or endorsement** relationship with it or its rights holders. See [`LICENSE`](LICENSE) and "Attribution and license" below.

## Core capabilities

| Capability | Description |
|---|---|
| **Installation discovery** | Detects, displays, and validates an existing npm installation only; **does not manage npm** — no install, uninstall, or dependency resolution |
| **Data directory** | Choose or switch `OPENCODEX_HOME`; the manager's own artifacts live under a customisable data root, split by purpose. Switching is **reference-first, migration optional**, with a mandatory backup before migration and rollback on failure |
| **Process control** | Hosts the official start/stop subprocesses directly; does not install or manage launchd and does not take over `codex-shim` |
| **Status observation** | Status is split into three independent dimensions — **run / connection / operation** — that can hold at the same time; actions render per status with no unexplained disabled buttons |
| **Logs and diagnostics** | Read-only logs with masked secrets; shows a **read-only** summary of the official `ocx doctor`; states clearly when live logs are unavailable |
| **Official panel** | Embeds a source-level snapshot view of the official web panel; renders the official GUI without restyling or injecting controls |
| **Notification center** | Shares one data source with "Diagnostics → Notification history"; supports a detail dialog and per-item deletion |
| **Configuration migration** | Full export (including sensitive content) must go into a **passphrase-protected encrypted container**; import validates format/version/integrity, backs up automatically, requires explicit confirmation, and rolls back on failure |
| **WebDAV sync** | Content is **encrypted client-side** before upload; a recoverable backup is taken before overwriting; conflicts go to a pending state instead of being silently overwritten |
| **Extension management** | Skills discovery/import/update/uninstall/restore; MCP server import/add/edit/delete; multi-client targets |
| **Two upgrade tracks** | OpenCodex itself is guided through the official `ocx update`; the desktop app updates independently with signature verification, restart to apply, and rollback on failure |
| **Tray and menus** | Tray actions are limited to the manager's own domain and express run status through icon and colour; native macOS menus are provided |
| **CLI control plane (optional)** | The `ocxd` command, **off by default**; once enabled it acts as a client of the running instance and delegates over local IPC |

## Security boundaries: what this project deliberately does not do

- It does not rewrite the OpenCodex core and does not bypass the official API.
- It does not take over the npm package manager or replace the official `ocx update` transaction.
- It does not take over `codex-shim`, and does not install or manage launchd.
- It **never writes OpenCodex core runtime configuration** such as providers, routing, or model mappings — those changes always go through the official CLI.
- It never silently overwrites configuration owned by an external provider; it detects, explains, and guides the user through the official `restore` after confirmation.
- No credential may enter the repository, Markdown files, logs, command-line arguments, or unencrypted files.
- The WebDAV remote is always treated as **untrusted**; TLS certificate validation failures are rejected by default, with no "ignore certificate" option.

### Write boundary for extension management

| | Scope |
|---|---|
| **May write** | Skills directory sync; each client's MCP server nodes |
| **Must not write** | Providers, routing, model mappings, and OpenCodex core runtime configuration |

Every write is backed up first and applied with a temp file plus atomic replace, with rollback on failure; deletions are recoverable; `env` / `args` / `headers` are always masked as sensitive values; writes hold a **cross-process file lock** and detect external modification.

## Stack and platform

- **Desktop shell**: Tauri v2, Rust backend, web frontend.
- **Platform**: macOS first; cross-platform support is abstracted for later but is not part of the first acceptance scope.

## Documentation map

The requirements, prototype, and governance documents are **not published in this repository yet**; they are maintained on a separate documentation branch. This repository contains only the outward-facing description and the license.

## Project status

| Item | Status |
|---|---|
| Requirement scope and technology choice | Confirmed |
| Requirements analysis stage | Complete (baseline, scenario and acceptance matrix, risk register, security review, IMP input checklist) |
| Review gates | Gates A / B / C / D passed; Gate E artifacts in place |
| Implementation contract (IMP) | Not created |
| Implementation and build artifacts | **None** |

## Development

Implementation has not started, so there are no build steps, environment requirements, or run instructions. This section will be filled in once the IMP is created and the technical contracts are frozen.

## Attribution and license

- This project is an **independent desktop manager** with no affiliation, authorization, or endorsement relationship with the official OpenCodex project or its rights holders.
- The OpenCodex name, version, and links are used only as needed for compatibility descriptions; its source code, build artifacts, icons, UI assets, and trademarks are **not copied or redistributed**.
- The OpenCodex source code, name, trademarks, and other rights remain with their rights holders; when using the official software, refer to the `LICENSE`, `NOTICE`, and terms of service in the official repository.
- This project's own code and assets are released under the **MIT License** (see [`LICENSE`](LICENSE)). The two licenses are **independent** of each other.
- This repository **does not distribute official source assets** (such as official page snapshots, official UI screenshots, or official brand marks); those are kept locally only, consistent with the statement above.
