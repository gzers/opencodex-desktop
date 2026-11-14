use super::*;
use crate::modules::runtime::terminal_cli::tests::Fixture;
fn input(f: &Fixture) -> (SharedRuntimeExecutable, ProcessCommand) {
    let runtime: SharedRuntimeExecutable =
        crate::modules::runtime::RuntimeHandle::initialize_versioned(
            &f.c.desktop_root,
            &f.c.program_root,
        );
    let mut c = ProcessCommand::new(
        LifecycleAction::Start,
        runtime.executable().unwrap(),
        f.c.user_profile.clone(),
        f.c.opencodex_home.clone(),
    );
    c.environment.home = Some(f.c.user_profile.clone().into_os_string());
    (runtime, c)
}
#[test]
fn inherited_start_stop_restart_deadlines_never_reset() {
    let started = Instant::now();
    let parent = started + Duration::from_secs(3);
    for action in [
        LifecycleAction::Start,
        LifecycleAction::Stop,
        LifecycleAction::Restart,
    ] {
        let w = ActionWindow::begin(action, started, parent).unwrap();
        assert_eq!(w.deadline(), parent);
        assert_eq!(w.stop_deadline(), parent);
        assert!(w.remaining().unwrap() <= Duration::from_secs(3));
    }
    let w = ActionWindow::begin(
        LifecycleAction::Restart,
        started,
        started + Duration::from_secs(99),
    )
    .unwrap();
    assert_eq!(w.deadline(), started + Duration::from_secs(30));
    assert_eq!(w.stop_deadline(), started + Duration::from_secs(10));
    let w = ActionWindow::begin(
        LifecycleAction::Start,
        started,
        started + Duration::from_secs(99),
    )
    .unwrap();
    assert_eq!(w.deadline(), started + Duration::from_secs(30));
    assert!(ActionWindow::begin(LifecycleAction::Start, started, started).is_err());
}
#[test]
fn plan_uses_exact_node_mjs_profile_and_distinct_home_without_shell() {
    let f = Fixture::new();
    let (runtime, mut c) = input(&f);
    c.environment.https_proxy = Some("http://127.0.0.1:3128".into());
    c.environment.no_proxy = Some("localhost".into());
    let deadline = Instant::now() + Duration::from_secs(30);
    let p = LaunchPlan::prepare(&runtime, &c, deadline).unwrap();
    assert!(p.executable().ends_with("dependencies/node/bin/node"));
    assert!(p.arguments()[0].to_string_lossy().ends_with("bin/ocx.mjs"));
    assert_eq!(
        &p.arguments()[1..],
        &[OsString::from("start"), "--port".into(), "0".into()]
    );
    assert_eq!(p.version(), "2.99.0");
    assert_eq!(p.program_root(), f.c.program_root);
    assert_eq!(p.user_profile(), f.c.user_profile);
    assert_eq!(p.opencodex_home(), f.c.opencodex_home);
    assert_eq!(p.deadline(), deadline);
    assert_eq!(
        p.environment().get(std::ffi::OsStr::new("HOME")),
        Some(&f.c.user_profile.as_os_str().to_owned())
    );
    assert_eq!(
        p.environment().get(std::ffi::OsStr::new("OPENCODEX_HOME")),
        Some(&f.c.opencodex_home.as_os_str().to_owned())
    );
    assert!(p
        .environment()
        .contains_key(std::ffi::OsStr::new("HTTPS_PROXY")));
    assert!(!p
        .environment()
        .contains_key(std::ffi::OsStr::new("SECRET_KEY")));
    p.verify_full(&runtime).unwrap();
    c.requested_port = Some(18241);
    let p = LaunchPlan::prepare(&runtime, &c, deadline).unwrap();
    assert_eq!(p.arguments().last().unwrap(), "18241");
}
#[test]
fn plan_refuses_context_or_source_drift_and_expired_budget() {
    for case in ["home", "action", "entry", "source", "deadline"] {
        let f = Fixture::new();
        let (runtime, mut c) = input(&f);
        let mut deadline = Instant::now() + Duration::from_secs(30);
        match case {
            "home" => c.environment.home = Some("/another-profile".into()),
            "action" => c.action = LifecycleAction::Stop,
            "entry" => c.executable = f.c.program_root.join("ocx.exe"),
            "source" => {
                std::fs::write(f.c.program_root.join("runtime/current.json"), b"changed").unwrap()
            }
            "deadline" => deadline = Instant::now(),
            _ => unreachable!(),
        };
        assert!(
            LaunchPlan::prepare(&runtime, &c, deadline).is_err(),
            "{case}"
        );
    }
    let f = Fixture::new();
    let (runtime, c) = input(&f);
    let p = LaunchPlan::prepare(&runtime, &c, Instant::now() + Duration::from_secs(30)).unwrap();
    let (other, _) = input(&f);
    assert!(p.check_current(&other).is_err());
    std::fs::write(f.c.program_root.join("runtime/current.json"), b"changed").unwrap();
    assert!(p.check_current(&runtime).is_err());
}
#[test]
fn full_plan_rechecks_ordinary_files_not_only_selection() {
    let f = Fixture::new();
    let (runtime, c) = input(&f);
    let p = LaunchPlan::prepare(&runtime, &c, Instant::now() + Duration::from_secs(30)).unwrap();
    let extra = f.c.program_root.join("runtime/versions/2.99.0/foreign");
    std::fs::write(&extra, b"outside").unwrap();
    assert!(p.verify_full(&runtime).is_err());
    assert_eq!(std::fs::read(extra).unwrap(), b"outside");
}
