//! 配置测试沙箱隔离验收（配置规划§10 / 迁移机制§11）。
//!
//! 受控的模拟「日常根」放置哨兵文件；在沙箱身份下完成偏好保存与配置迁移后，
//! 断言日常根的文件内容与 mtime 未改变，且错误环境（沙箱根缺失）不会回退到日常根。
//! 本测试单独成一个集成测试二进制，独占进程环境变量。

use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn snapshot(path: &Path) -> (Vec<u8>, SystemTime) {
    (
        std::fs::read(path).expect("read sentinel"),
        std::fs::metadata(path).expect("sentinel metadata").modified().expect("mtime"),
    )
}

fn write_daily_root(root: &Path) -> (PathBuf, PathBuf) {
    std::fs::create_dir_all(root.join("manager-state")).expect("daily manager-state");
    let prefs = root.join("manager-state/preferences.json");
    let meta = root.join("manager-state/data-root.json");
    std::fs::write(&prefs, br#"{"interface_scale":123,"app_update_channel":"beta-6h"}"#)
        .expect("seed preferences");
    std::fs::write(&meta, br#"{"structure_version":"1"}"#).expect("seed data-root");
    (prefs, meta)
}

#[test]
fn sandbox_writes_never_touch_daily_root_and_missing_root_fails_closed() {
    let daily = tempfile::tempdir().expect("daily root");
    let sandbox = tempfile::tempdir().expect("sandbox root");
    let (daily_prefs, daily_meta) = write_daily_root(daily.path());
    let before_prefs = snapshot(&daily_prefs);
    let before_meta = snapshot(&daily_meta);

    // 日常身份：沿用传入的默认根。
    std::env::remove_var("OPENCODEX_SANDBOX");
    std::env::remove_var("OPENCODEX_SANDBOX_ROOT");
    assert_eq!(
        opencodex_desktop_lib::modules::test_sandbox::resolve_data_root(daily.path()).unwrap(),
        daily.path()
    );

    // 沙箱启用但根缺失：拒绝回退到日常目录。
    std::env::set_var("OPENCODEX_SANDBOX", "1");
    assert!(opencodex_desktop_lib::modules::test_sandbox::resolve_data_root(daily.path()).is_err());

    // 沙箱根到位：所有写入都落在沙箱。
    std::env::set_var("OPENCODEX_SANDBOX_ROOT", sandbox.path());
    let resolved = opencodex_desktop_lib::modules::test_sandbox::resolve_data_root(daily.path())
        .expect("sandbox root");
    assert_eq!(resolved, sandbox.path());

    // 1) 偏好保存：写入沙箱 manager-state，而不是日常根。
    let store = opencodex_desktop_lib::modules::preferences::PreferencesStore::new(&resolved);
    let mut value = opencodex_desktop_lib::modules::preferences::Preferences::default();
    value.interface_scale = 175;
    store.save(&value).expect("save into sandbox");
    assert!(sandbox.path().join("manager-state/preferences.json").exists());

    // 2) 配置迁移：对沙箱内的 legacy 文件执行迁移提交。
    let sandbox_prefs = sandbox.path().join("manager-state/preferences.json");
    std::fs::write(&sandbox_prefs, br#"{"interface_scale":140,"app_update_channel":"manual"}"#)
        .expect("seed sandbox legacy");
    let legacy = std::fs::read(&sandbox_prefs).unwrap();
    let kind = opencodex_desktop_lib::modules::config_migration::DocumentKind::Preferences;
    let (report, migrated) =
        opencodex_desktop_lib::modules::config_migration::prepare(kind, &legacy).expect("prepare");
    opencodex_desktop_lib::modules::config_migration::commit(
        &resolved,
        &sandbox_prefs,
        &legacy,
        &report,
        &migrated.expect("migrated bytes"),
    )
    .expect("commit into sandbox");

    // 3) 日常根哨兵：内容与 mtime 均未改变。
    assert_eq!(snapshot(&daily_prefs), before_prefs, "daily preferences changed");
    assert_eq!(snapshot(&daily_meta), before_meta, "daily data-root changed");

    std::env::remove_var("OPENCODEX_SANDBOX_ROOT");
    std::env::remove_var("OPENCODEX_SANDBOX");
}

