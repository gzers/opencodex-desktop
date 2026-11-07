// Diagnostic-only read of the actual backend preference loader, in an explicit
// disposable account root. No endpoints are bound and no service is started.
use opencodex_desktop_lib::modules::preferences::{load_preferences, PreferencesStore};
use std::{env, path::PathBuf};
fn main() {
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
