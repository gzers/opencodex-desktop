use opencodex_desktop_lib::commands::app_status;

#[test]
fn app_status_reports_shell_ready() {
    let status = app_status().expect("app status should be available");
    assert_eq!(status.mode, opencodex_desktop_lib::types::AppMode::Shell);
    assert!(status.ready);
}
