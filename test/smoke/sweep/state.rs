// Diagnostic-only read of the actual backend preference loader, in an explicit
// disposable account root. No endpoints are bound and no service is started.
use opencodex_desktop_lib::modules::preferences::{load_preferences, PreferencesStore};
use std::{env, path::PathBuf};
fn main() {
    if env::args().nth(1).as_deref() == Some("--runtime-observe") {
        observe_runtime();
        return;
    }
    let root = PathBuf::from(env::args_os().nth(1).expect("explicit owned fixture root"));
    assert!(root.is_absolute());
    #[cfg(windows)]
    match opencodex_desktop_lib::modules::ipc::endpoint::pipe_name() {
        Ok(name) => println!(
            "{{\"probe\":\"actual-backend-pipe-name\",\"name\":\"{}\",\"accepted\":false}}",
            name.replace('\\', "\\\\")
        ),
        Err(_) => println!(
            "{{\"probe\":\"actual-backend-pipe-name\",\"status\":\"fail\",\"accepted\":false}}"
        ),
    }
    for (name, result) in [
        (
            "raw-preferences",
            load_preferences(&root.join("manager-state/preferences.json")),
        ),
        (
            "actual-preferences-store",
            PreferencesStore::new(&root).load(),
        ),
    ] {
        match result {Ok(value)=>println!("{{\"probe\":\"{name}\",\"status\":\"pass\",\"cliEnabled\":{},\"schemaVersion\":{},\"accepted\":false}}",value.cli_enabled,value.schema_version),Err(error)=>println!("{{\"probe\":\"{name}\",\"status\":\"fail\",\"errorKind\":\"{error:?}\",\"accepted\":false}}")}
    }
}

// Read-only forensics using the real backend and current explicit I/H. This
// never admits a service or changes the production five-second budget.
fn observe_runtime() {
    use opencodex_desktop_lib::{
        infrastructure::status_source::OfficialStatusSource,
        modules::{
            process::EnvironmentPolicy,
            runtime::{install, MANAGED_ENTRY_RELATIVE},
            status::StatusSource,
        },
    };
    let args: Vec<_> = env::args_os().collect();
    assert_eq!(args.len(), 5);
    let installation = PathBuf::from(&args[2]);
    let desktop = PathBuf::from(&args[3]);
    let home = PathBuf::from(&args[4]);
    assert!(installation.is_absolute() && desktop.is_absolute() && home.is_absolute());
    let started = std::time::Instant::now();
    let identity = install::resolve_managed_installation(&installation);
    println!(
        "{}",
        serde_json::json!({"probe":"actual-activation-verification","elapsedMilliseconds":started.elapsed().as_millis(),"passed":identity.is_ok(),"error":identity.as_ref().err().map(|e|format!("{e:?}")),"accepted":false})
    );
    #[derive(Debug)]
    struct ExplicitInstallation(PathBuf);
    impl opencodex_desktop_lib::infrastructure::runtime_executable::RuntimeExecutableProvider
        for ExplicitInstallation
    {
        fn executable(&self) -> Option<PathBuf> {
            Some(self.0.join(MANAGED_ENTRY_RELATIVE))
        }
        fn managed_installation_root(&self) -> Option<PathBuf> {
            Some(self.0.clone())
        }
    }
    let runtime = std::sync::Arc::new(ExplicitInstallation(installation));
    let source = OfficialStatusSource::new(
        runtime,
        home.clone(),
        EnvironmentPolicy {
            opencodex_home: home,
            ..Default::default()
        },
    );
    let started = std::time::Instant::now();
    let tokio = tokio::runtime::Runtime::new().unwrap();
    let result = tokio.block_on(async { source.fetch() });
    let public = result.as_ref().ok().map(|v|serde_json::json!({"status":v["status"],"ready":v["ready"],"pid":v["pid"],"port":v["port"],"statusObservation":v["statusObservation"]}));
    println!(
        "{}",
        serde_json::json!({"probe":"actual-status-source","elapsedMilliseconds":started.elapsed().as_millis(),"budgetMilliseconds":5000,"payload":public,"error":result.as_ref().err(),"accepted":false})
    );
}
