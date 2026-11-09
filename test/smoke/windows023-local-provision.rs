//! Fixture-only access to the unchanged offline provisioner; no service launch.
use std::path::PathBuf;
fn main() {
    let fixture = PathBuf::from(std::env::args_os().nth(1).expect("explicit fresh fixture"));
    assert!(fixture.is_absolute());
    let installation = fixture.join("program");
    let desktop = fixture.join("data/desktop");
    let resources = installation.join("runtime-bundle");
    assert!(!installation.join("runtime").exists(), "fresh runtime only");
    assert!(
        desktop.join("manager-state/data-root.json").is_file(),
        "product must initialize its own data first"
    );
    std::env::set_var("OPENCODEX_SANDBOX", "1");
    std::env::set_var("OPENCODEX_SANDBOX_ROOT", &fixture);
    let bytes = std::fs::read(resources.join("runtime-bundle.manifest.json")).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let name = value["archiveName"].as_str().unwrap();
    assert!(!name.contains(['/', '\\', ':']) && !matches!(name, "." | ".."));
    assert_eq!(value["runtimeVersion"], "2.97.0-desktop023.lifecycle.3");
    assert_eq!(
        value["runtimeArchiveSha256"],
        "ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739"
    );
    let start = std::time::Instant::now();
    match opencodex_desktop_lib::modules::runtime::bundled::provision(
        &installation,
        &desktop,
        &resources.join(name),
        &bytes,
        env!("OCXD_BUNDLED_RUNTIME_DESCRIPTOR_SHA256"),
    ) {
        Ok(_) => println!(
            "{}",
            serde_json::json!({"provisioned":true,"source":opencodex_desktop_lib::build_commit(),"elapsedMilliseconds":start.elapsed().as_millis(),"serviceStarted":false,"accepted":false})
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
