#![cfg(unix)]
//! 真实链路：`ocx codex-shim` 开关经受控来源子进程读写（不触碰真实官方安装）。
//!
//! 用 shell 桩替代官方 `ocx`：桩会记录收到的参数、`HOME` 与 `PATH`，并按标记文件
//! 回报「已安装 / 未安装」。因此这里验证的是**真实 spawn 路径**（参数、冻结环境、
//! 退出码、输出解析、写后复读与目标状态校验），而不是内存 mock。

use opencodex_desktop_lib::infrastructure::codex_shim_source::OfficialCodexShimSource;
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::modules::codex_shim::{
    parse_state, CodexShimError, CodexShimSource, CodexShimState,
};
use opencodex_desktop_lib::modules::process::EnvironmentPolicy;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// 正常桩：install/uninstall 真的改标记文件，status 据此回报。
const STUB: &str = r#"#!/bin/sh
echo "$@|HOME=$HOME|PATH=$PATH" >> "@ARG_LOG@"
if [ "$1" = "codex-shim" ]; then
  case "$2" in
    status)
      if [ -f "@MARKER@" ]; then
        echo "Codex autostart shim: wrapper shim present at /x/codex; original backup present at /x/codex.opencodex-real."
      else
        echo "Codex autostart shim is not installed."
      fi
      ;;
    install) : > "@MARKER@"; echo "Codex autostart shim installed." ;;
    uninstall) /bin/rm -f "@MARKER@"; echo "Codex autostart shim removed." ;;
    *) echo "usage"; exit 1 ;;
  esac
  exit 0
fi
echo "unknown"; exit 1
"#;

// 复刻官方在「PATH 上找不到 codex」时的行为：打印告警，但退出码仍是 0，且什么都没做。
const NOT_FOUND_STUB: &str = r#"#!/bin/sh
if [ "$1" = "codex-shim" ]; then
  case "$2" in
    status) echo "Codex autostart shim is not installed." ;;
    install) echo "⚠️  Could not find a codex executable on PATH." ;;
    uninstall) echo "⚠️  Codex autostart shim is not installed." ;;
  esac
  exit 0
fi
exit 0
"#;

// 复刻 macOS 的 App 管理权限拒绝：官方 rename 失败，退出码 1，原因只在 stderr。
const EPERM_STUB: &str = r#"#!/bin/sh
if [ "$1" = "codex-shim" ]; then
  case "$2" in
    status) echo "Codex autostart shim is not installed." ; exit 0 ;;
    install)
      # 真机上学官方 Bun 的堆栈形态：首行只是源码上下文，原因在后面几行。
      echo "2192 |       wrapperWriteStarted: false," 1>&2
      echo "2197 |         renameSync(target.originalPath, target.backupPath);" 1>&2
      echo "EPERM: operation not permitted, rename '/Applications/ChatGPT.app/Contents/Resources/codex' -> 'x'" 1>&2
      exit 1 ;;
    uninstall) echo "Codex autostart shim is not installed." ; exit 0 ;;
  esac
fi
exit 0
"#;

fn write_stub(path: &Path, body: &str) {
    std::fs::write(path, body).expect("write stub");
    std::fs::set_permissions(path, Permissions::from_mode(0o755)).expect("chmod stub");
}

fn source_with(
    temp: &tempfile::TempDir,
    executable: &Path,
    home: &Path,
) -> OfficialCodexShimSource {
    let data_root = temp.path().join("OpenCodex Desktop");
    std::fs::create_dir_all(&data_root).expect("create data root");
    OfficialCodexShimSource::new(
        FixedRuntimeExecutable::resolved(executable.to_path_buf()),
        home.to_path_buf(),
        EnvironmentPolicy {
            home: Some(home.as_os_str().to_os_string()),
            opencodex_home: data_root.join("opencodex-home"),
            path: Some(PathBuf::from("/usr/bin:/bin").into_os_string()),
            ..Default::default()
        },
    )
}

/// 造出「隔离 HOME 里有 codex、另一个候选目录里没有」的局面。
fn prepare(temp: &tempfile::TempDir) -> (PathBuf, PathBuf, PathBuf) {
    let home = temp.path().join("home");
    std::fs::create_dir_all(home.join(".local/bin")).expect("create home bin");
    std::fs::write(home.join(".local/bin/codex"), b"#!/bin/sh\nexit 0\n").expect("fake codex");
    let marker = temp.path().join("installed.marker");
    let log = temp.path().join("args.log");
    (home, marker, log)
}

#[test]
fn shim_status_set_and_read_back_through_controlled_subprocess() {
    let temp = tempfile::tempdir().expect("temp");
    let (home, marker, log) = prepare(&temp);
    let executable = temp.path().join("ocx");
    write_stub(
        &executable,
        &STUB
            .replace("@ARG_LOG@", &log.to_string_lossy())
            .replace("@MARKER@", &marker.to_string_lossy()),
    );
    let source = source_with(&temp, &executable, &home);

    let initial = source.status().expect("status");
    assert_eq!(
        parse_state(&String::from_utf8_lossy(&initial.stdout)),
        CodexShimState::NotInstalled
    );

    let after_enable = source.set_enabled(true).expect("install + re-read");
    assert_eq!(
        parse_state(&String::from_utf8_lossy(&after_enable.stdout)),
        CodexShimState::Installed
    );

    let after_disable = source.set_enabled(false).expect("uninstall + re-read");
    assert_eq!(
        parse_state(&String::from_utf8_lossy(&after_disable.stdout)),
        CodexShimState::NotInstalled
    );

    let calls = std::fs::read_to_string(&log).expect("args log");
    let lines: Vec<&str> = calls.lines().collect();
    assert!(lines[0].starts_with("codex-shim status|"), "{lines:?}");
    assert!(lines[1].starts_with("codex-shim install|"), "{lines:?}");
    assert!(
        lines[2].starts_with("codex-shim status|"),
        "写后必须复读：{lines:?}"
    );
    assert!(lines[3].starts_with("codex-shim uninstall|"), "{lines:?}");
    assert!(lines[4].starts_with("codex-shim status|"), "{lines:?}");
    // 冻结环境：子进程拿到的是隔离 HOME，而不是真实用户 HOME。
    assert!(
        lines[0].contains(&format!("HOME={}", home.display())),
        "{lines:?}"
    );
}

/// 回归：官方 `codex-shim` 只在 `PATH` 上找 `codex`；找不到时它**退出 0**却什么都不做。
/// 管理器必须把确实含 `codex` 的候选目录补进子进程 `PATH`，且不补不存在的目录。
#[test]
fn child_path_includes_only_candidate_dirs_that_really_hold_codex() {
    let temp = tempfile::tempdir().expect("temp");
    let (home, marker, log) = prepare(&temp);
    let executable = temp.path().join("ocx");
    write_stub(
        &executable,
        &STUB
            .replace("@ARG_LOG@", &log.to_string_lossy())
            .replace("@MARKER@", &marker.to_string_lossy()),
    );
    let source = source_with(&temp, &executable, &home);

    source.status().expect("status");
    let line = std::fs::read_to_string(&log).expect("args log");
    let path = line
        .lines()
        .next()
        .and_then(|l| l.split("PATH=").nth(1))
        .expect("logged PATH");

    let entries: Vec<&str> = path.split(':').collect();
    assert!(entries.contains(&"/usr/bin"), "{path}");
    assert!(
        entries.contains(&home.join(".local/bin").to_string_lossy().as_ref()),
        "隔离 HOME 里确实有 codex，必须补进 PATH：{path}"
    );
    // 候选目录里没有 `codex` 的一律不补（这台机器上 Homebrew / 系统前缀都没有 codex）。
    for candidate in ["/opt/homebrew/bin", "/usr/local/bin"] {
        if !Path::new(candidate).join("codex").is_file() {
            assert!(
                !entries.contains(&candidate),
                "不含 codex 的目录不该进 PATH：{path}"
            );
        }
    }
}

/// 回归（真机缺陷）：官方 CLI 退出 0 但状态没变（找不到 codex）时，不能当成功。
#[test]
fn silent_no_op_is_reported_as_rejected_with_official_first_line() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).expect("create home");
    let executable = temp.path().join("ocx");
    write_stub(&executable, NOT_FOUND_STUB);
    let source = source_with(&temp, &executable, &home);

    let error = source
        .set_enabled(true)
        .expect_err("退出码 0 但状态未变必须判失败");
    match error {
        CodexShimError::Rejected(detail) => {
            assert!(
                detail.contains("Could not find a codex executable"),
                "应带回官方首行：{detail}"
            );
        }
        other => panic!("expected Rejected, got {other:?}"),
    }

    // 目标状态已是「未安装」时，卸载同样属于幂等成功，不应报错。
    assert!(source.set_enabled(false).is_ok());
}

/// 回归（真机缺陷）：macOS 拒绝应用改动别的应用包时，官方只把原因写在 stderr 且退出非零；
/// 失败原因必须带回来（否则界面只能给一句没有信息量的「写入失败」）。
#[test]
fn stderr_reason_is_surfaced_when_official_write_is_denied() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).expect("create home");
    let executable = temp.path().join("ocx");
    write_stub(&executable, EPERM_STUB);
    let source = source_with(&temp, &executable, &home);

    let error = source.set_enabled(true).expect_err("被拒绝时必须失败");
    match error {
        CodexShimError::Rejected(detail) => {
            assert!(detail.contains("EPERM"), "应带回 stderr 原因：{detail}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[test]
fn unresolved_runtime_is_unreachable_and_never_spawns() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).expect("create home");
    let source = OfficialCodexShimSource::new(
        FixedRuntimeExecutable::unresolved(),
        home,
        EnvironmentPolicy {
            opencodex_home: temp.path().join("OpenCodex Desktop/opencodex-home"),
            ..Default::default()
        },
    );
    assert!(source.status().is_err());
    assert!(source.set_enabled(true).is_err());
}
