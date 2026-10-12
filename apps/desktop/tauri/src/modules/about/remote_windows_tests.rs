//! Actual Windows metadata queries share the owned-process fixture with version probes.
use super::*;
use crate::infrastructure::windows_query_fixture::{process_case, QueryOutcome};

fn metadata_process_case(mode: &str) {
    process_case(
        "metadata",
        mode,
        r#"{"version":"2.50.0","dist.integrity":"sha512-fixture"}"#,
        |executable, root, environment| match run_npm_view_with_timeout(
            executable,
            root,
            &root.join("cache/npm"),
            &environment,
            "test@latest",
            std::time::Duration::from_secs(2),
        ) {
            Ok(output) => {
                let value: serde_json::Value = serde_json::from_str(&output).unwrap();
                assert_eq!(parse_metadata(&value).unwrap().0, "2.50.0");
                QueryOutcome::Success
            }
            Err(AppError::Timeout) => QueryOutcome::Timeout,
            Err(_) => QueryOutcome::Failure,
        },
    );
}

#[test]
fn metadata_timeout_cleans_descendant_and_preserves_unrelated_process() {
    metadata_process_case("timeout");
}

#[test]
fn metadata_output_overflow_cleans_descendant() {
    metadata_process_case("overflow");
}

#[test]
fn metadata_nonzero_exit_cleans_descendant() {
    metadata_process_case("nonzero");
}

#[test]
fn metadata_success_also_reaps_its_owned_helper_tree() {
    metadata_process_case("success");
}
