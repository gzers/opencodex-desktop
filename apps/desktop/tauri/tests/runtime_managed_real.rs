//! 托管安装的真实链路（需要本机 node / npm 与可用网络）。
//!
//! 与 `runtime` 的单元测试（假 npm / 假探针）不同，这里把
//! `RuntimeInstaller` 完整跑起来：真实 `SystemNpmRunner`、真实 npm 子进程、
//! 真实 registry 下载、真实入口探针、真实文件系统，验证：
//!
//! 1. 联网安装 → 入口**真的跑起来**能读出官方版本 → 稳定入口可执行 → 一级卸载往返；
//! 2. `npm pack` 得到的真实官方 tarball 走离线导入：先被「需要安装脚本」拦下，
//!    二次确认后带脚本重跑成功 → 卸载往返；
//! 3. 系统保护目录被拒绝；
//! 4. 坏包被拒绝且**不改动现有运行来源**；
//! 5. 失败后不留下临时区残渣。
//!
//! 未设置 `OCX_TEST_RUNTIME_REAL=1` 时打印跳过原因并返回（视为未验证）。
//! 设置 `OCX_TEST_RUNTIME_HOME=<目录>` 可换一个全新 HOME（冷 npm 缓存），
//! 复现「首次联网安装」路径。
//!
//! 运行方式（会下载 npm 包，必须串行）：
//! `OCX_TEST_RUNTIME_REAL=1 cargo test --test runtime_managed_real -- --test-threads=1 --nocapture`

use std::path::{Path, PathBuf};

use opencodex_desktop_lib::modules::runtime::install::{
    InstallError, InstallRequest, InstallSource, NpmRunner, NullProgressSink, RealVersionProbe,
    RuntimeInstaller, SystemNpmRunner,
};
use opencodex_desktop_lib::modules::runtime::paths::validate_executable;
use opencodex_desktop_lib::modules::runtime::uninstall::{self, UninstallError};
use opencodex_desktop_lib::modules::runtime::{RuntimeHandle, RuntimeSourceKind, OFFICIAL_PACKAGE};
use opencodex_desktop_lib::types::discovery_paths::macos_default_paths;

fn enabled() -> bool {
    std::env::var("OCX_TEST_RUNTIME_REAL").ok().as_deref() == Some("1")
}

fn skip(reason: &str) {
    eprintln!("跳过：{reason}（视为未验证）");
}

struct Env {
    root: tempfile::TempDir,
    data_root: PathBuf,
    npm: PathBuf,
    node: PathBuf,
    home: PathBuf,
}

fn environment() -> Option<Env> {
    let paths = macos_default_paths();
    let node = validate_executable(&paths.node).ok()?;
    let npm = validate_executable(&paths.npm).ok()?;
    // 默认复用本机 HOME 的 npm 缓存；设置 `OCX_TEST_RUNTIME_HOME` 可用一个
    // 全新的 HOME（冷缓存）复现「首次下载」路径——真实 GUI 安装就是冷缓存。
    let home = std::env::var_os("OCX_TEST_RUNTIME_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))?;
    // `OCX_TEST_RUNTIME_DATA_ROOT` 指定时直接用给定数据根（用于「先装好，再用打包
    // 制品真机观察运行来源与状态采集是否跟随」）；否则用临时数据根。
    let root = tempfile::tempdir().ok()?;
    let data_root = match std::env::var_os("OCX_TEST_RUNTIME_DATA_ROOT") {
        Some(value) => PathBuf::from(value),
        None => root.path().join("data-root"),
    };
    Some(Env {
        root,
        data_root,
        npm,
        node,
        home,
    })
}

fn installer<'a>(
    env: &'a Env,
    npm_runner: &'a SystemNpmRunner,
    probe: &'a RealVersionProbe,
    sink: &'a NullProgressSink,
) -> RuntimeInstaller<'a> {
    RuntimeInstaller::new(
        &env.data_root,
        npm_runner,
        Some(env.home.clone()),
        sink,
        probe,
        Some(env.node.clone()),
    )
}

fn run_entry(entry: &Path) -> Option<String> {
    println!(
        "  入口文件：\n{}",
        std::fs::read_to_string(entry).unwrap_or_else(|_| "(unreadable)".to_string())
    );
    let output = std::process::Command::new(entry)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    println!("  入口退出码 = {:?}", output.status.code());
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let err = String::from_utf8_lossy(&output.stderr).into_owned();
    println!("  入口 stdout：{}", text.trim());
    if !err.trim().is_empty() {
        println!("  入口 stderr：{}", err.trim());
    }
    opencodex_desktop_lib::modules::runtime::install::extract_semver(&text)
}

/// 临时区 / 退役区残渣（`.new-*` / `.old-*`）必须为零。
fn leftover_temp_dirs(prefix: &Path) -> Vec<String> {
    let Some(parent) = prefix.parent() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".new-") || name.contains(".old-"))
        .collect()
}

#[test]
fn real_managed_runtime_round_trip() {
    if !enabled() {
        skip("未设置 OCX_TEST_RUNTIME_REAL=1");
        return;
    }
    let Some(env) = environment() else {
        skip("未发现可用的 node / npm 或 HOME");
        return;
    };
    println!("== 真实链路：node={:?} npm={:?}", env.node, env.npm);

    let npm_runner = SystemNpmRunner::new(Some(env.home.clone()));
    let probe = RealVersionProbe::default();
    let sink = NullProgressSink;
    let installer = installer(&env, &npm_runner, &probe, &sink);
    let prefix = uninstall::managed_prefix(&env.data_root);
    let entry = uninstall::managed_entry(&env.data_root);

    // ---- 1. 联网安装（真实 registry）----
    println!("-- 1. 联网安装 @{OFFICIAL_PACKAGE}@latest");
    let handle = RuntimeHandle::initialize(&env.data_root, Vec::new());
    assert_eq!(handle.kind(), RuntimeSourceKind::Unresolved);

    let registry = installer
        .install(&InstallRequest {
            prefix: prefix.clone(),
            source: InstallSource::Registry {
                version: "latest".to_string(),
                proxy: None,
            },
            // 官方包需要安装脚本准备 bun 运行时；这里显式允许，模拟用户二次确认。
            allow_scripts: true,
            npm_path: Some(env.npm.clone()),
            node_path: Some(env.node.clone()),
        })
        .expect("real registry install must succeed");
    println!(
        "  已安装 {} {}（scripts={} sha256={}…）",
        registry.package,
        registry.version,
        registry.scripts_enabled,
        &registry.tarball_sha256[..12]
    );
    assert_eq!(registry.package, OFFICIAL_PACKAGE);
    // 实测结论：官方包在 `--ignore-scripts` 下即可运行——它把平台运行时放在
    // 依赖（可选平台包）里，而不是靠安装脚本生成。因此安全的默认档可用，
    // 「需要安装脚本」只在入口真的跑不起来时才会触发。
    if registry.scripts_enabled {
        println!("  注：本机需要安装脚本才可运行");
    } else {
        println!("  注：安全默认档（--ignore-scripts）即可运行");
    }
    assert_eq!(registry.tarball_sha256.len(), 64);
    assert!(entry.is_file(), "稳定入口必须落地");
    assert!(prefix.join(".runtime-manifest.json").is_file());

    let printed = run_entry(&entry).expect("托管入口必须能读出官方版本");
    assert_eq!(printed, registry.version, "入口自报版本必须与安装记录一致");

    let resolution = handle.refresh();
    assert_eq!(
        resolution.kind,
        RuntimeSourceKind::Managed,
        "安装后来源应切到托管"
    );
    assert_eq!(resolution.path.as_deref(), Some(entry.as_path()));
    println!("  运行来源 = {}", resolution.kind.as_str());

    // 供真机观察时只装不卸：把「本应用装好的数据根」原地留给打包制品。
    if std::env::var_os("OCX_TEST_RUNTIME_DATA_ROOT").is_some() {
        println!("-- 已按 OCX_TEST_RUNTIME_DATA_ROOT 原地保留托管安装：{prefix:?}");
        println!("== 真实链路（保留模式）通过");
        return;
    }

    // ---- 2. 一级卸载往返 ----
    println!("-- 2. 一级卸载");
    let removed = uninstall::uninstall_managed(&env.data_root, &prefix, &entry).expect("uninstall");
    assert!(removed.removed_prefix && removed.removed_entry);
    assert!(!prefix.exists() && !entry.exists());
    assert!(
        removed.backup_directory.is_dir(),
        "卸载前必须留下可定位备份"
    );
    let after = handle.refresh();
    println!("  卸载后来源 = {}", after.kind.as_str());
    assert_ne!(after.kind, RuntimeSourceKind::Managed);
    assert!(leftover_temp_dirs(&prefix).is_empty(), "不得留下临时区残渣");

    // ---- 3. 离线导入（真实 tarball + `--offline`）----
    println!("-- 3. 离线导入（npm pack 得到的真实 tarball）");
    let work = env.root.path().join("downloads");
    std::fs::create_dir_all(&work).expect("mkdir downloads");
    let tarball = npm_runner
        .pack(
            &opencodex_desktop_lib::modules::runtime::install::NpmPackRequest {
                npm_path: env.npm.clone(),
                node_dir: env.node.parent().map(Path::to_path_buf),
                version: registry.version.clone(),
                proxy: None,
                work_dir: work,
            },
            &sink,
            &opencodex_desktop_lib::modules::runtime::install::CancelFlag::new(),
        )
        .expect("npm pack must produce a real official tarball");
    println!("  tarball = {:?}", tarball.file_name());

    let offline_request = |allow_scripts: bool| InstallRequest {
        prefix: prefix.clone(),
        source: InstallSource::Offline {
            archive: tarball.clone(),
        },
        allow_scripts,
        npm_path: Some(env.npm.clone()),
        node_path: Some(env.node.clone()),
    };

    // 3a. 未确认：官方包缺 bun 运行时会**真的跑不起来** → 必须被拦成「需要安装脚本」。
    let gated = installer.install(&offline_request(false));
    match gated {
        Err(InstallError::ScriptsRequired) => {
            println!("  未确认时按预期拦下：scripts_required");
            assert!(!prefix.exists(), "被拦下时不得留下半成品");
        }
        Err(other) => panic!("离线导入的未确认分支应拦成 scripts_required，实际 {other:?}"),
        Ok(_) => println!("  注：本机缓存已能让未确认安装直接可用"),
    }

    // 3b. 二次确认后带脚本重跑 → 必须成功，并且入口真的能跑。
    let offline = installer
        .install(&offline_request(true))
        .expect("offline import with consent must succeed");
    println!(
        "  离线导入成功：{} {}（source={}）",
        offline.package,
        offline.version,
        offline.source.as_str()
    );
    assert_eq!(offline.source.as_str(), "offline");
    assert_eq!(run_entry(&entry).as_deref(), Some(offline.version.as_str()));
    assert_eq!(handle.refresh().kind, RuntimeSourceKind::Managed);

    // ---- 4. 系统保护目录被拒绝 ----
    println!("-- 4. 系统保护目录 / 坏包拒绝");
    let protected = installer
        .install(&InstallRequest {
            prefix: PathBuf::from("/usr/lib/opencodex"),
            source: InstallSource::Offline {
                archive: tarball.clone(),
            },
            allow_scripts: true,
            npm_path: Some(env.npm.clone()),
            node_path: Some(env.node.clone()),
        })
        .expect_err("系统保护目录必须拒绝");
    println!("  系统目录拒绝 = {}", protected.code());
    assert_eq!(protected.code(), "system_protected");

    // ---- 5. 坏包被拒绝且不改动现有运行来源 ----
    let bad = env.root.path().join("opencodex-0.0.1.tgz");
    std::fs::write(&bad, b"definitely not a gzip stream").expect("write bad archive");
    let before = handle.current();
    let bad_error = installer
        .install(&InstallRequest {
            prefix: prefix.clone(),
            source: InstallSource::Offline { archive: bad },
            allow_scripts: true,
            npm_path: Some(env.npm.clone()),
            node_path: Some(env.node.clone()),
        })
        .expect_err("坏包必须拒绝");
    println!("  坏包拒绝 = {}", bad_error.code());
    let after_bad = handle.refresh();
    assert_eq!(after_bad.kind, before.kind, "坏包不得改动运行来源");
    assert_eq!(after_bad.path, before.path);
    assert!(entry.is_file(), "坏包不得动到已装好的托管入口");
    assert!(leftover_temp_dirs(&prefix).is_empty(), "不得留下临时区残渣");

    // ---- 6. 冲突：外部改写后拒绝删除 ----
    println!("-- 6. 外部改写后拒绝一级卸载");
    let package_json = prefix.join("node_modules/@bitkyc08/opencodex/package.json");
    let mut json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&package_json).expect("read")).expect("parse");
    json["version"] = serde_json::json!("9.9.9");
    std::fs::write(&package_json, serde_json::to_vec(&json).expect("ser")).expect("write");
    let conflict = uninstall::uninstall_managed(&env.data_root, &prefix, &entry)
        .expect_err("外部改写必须拒绝卸载");
    assert!(matches!(
        conflict,
        UninstallError::ExternallyModified { .. }
    ));
    assert!(prefix.is_dir(), "拒绝时不得删除前缀");
    println!("  冲突拒绝 = {}", conflict.code());

    // 收尾：把包体改回去再正常卸载，保证证据目录里不残留半成品。
    json["version"] = serde_json::json!(offline.version);
    std::fs::write(&package_json, serde_json::to_vec(&json).expect("ser")).expect("write");
    let final_removed =
        uninstall::uninstall_managed(&env.data_root, &prefix, &entry).expect("final uninstall");
    assert!(final_removed.removed_prefix && final_removed.removed_entry);
    println!("== 真实链路全部通过");
}
