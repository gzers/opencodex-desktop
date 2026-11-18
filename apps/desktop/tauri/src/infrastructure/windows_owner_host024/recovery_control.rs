//! Independent r7 private protocol. Decoded requests/locator/clock are DATA,
//! never an original Owner, native Job, recovery proof or action admission.
//! Original authenticated same-image/ordinary/no-Job transport must bind them.
mod client;
mod server;
mod wire;
pub(super) fn command_budget(operation: &str) -> std::io::Result<u64> {
    client::command(operation).map(|(_, seconds)| seconds)
}
pub(super) fn request(
    c: &super::empty_host::EmptyBootstrap,
    operation: &str,
    started: std::time::Instant,
    end: std::time::Instant,
    counter: u64,
    frequency: u64,
) -> std::io::Result<(serde_json::Value, bool)> {
    let (command, _) = client::command(operation)?;
    client::request(c, command, started, end, counter, frequency)
}
#[cfg(debug_assertions)]
pub(super) fn run_empty_fixture(c: super::empty_host::EmptyBootstrap) -> std::io::Result<()> {
    let registration = install_before_capture(&c)?;
    // This exact test-only scope never creates/binds a RuntimeHandle/Owner or
    // calls official/source/capture. All V2 operations must remain Refused.
    loop {
        registration.check_alive()?;
        std::thread::park_timeout(std::time::Duration::from_millis(100));
    }
}
pub(super) use server::install_before_capture;
