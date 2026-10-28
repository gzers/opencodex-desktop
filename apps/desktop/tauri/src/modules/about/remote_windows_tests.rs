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
            &root.join("logs/runtime-query"),
            &environment,
            "test@latest",
            std::time::Duration::from_secs(2),
        ) {
            Ok((version, _)) => {
                assert_eq!(version, "2.50.0");
                QueryOutcome::Success
            }
            Err(AppError::RemoteQuery { reason, .. }) if reason == "npm_view_timeout" => {
                QueryOutcome::Timeout
            }
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

#[test]
fn metadata_stderr_overflow_cleans_descendant() {
    metadata_process_case("stderr-overflow");
}

#[test]
fn metadata_stderr_timeout_cleans_descendant() {
    metadata_process_case("stderr-timeout");
}

fn records(directory: &std::path::Path) -> Vec<serde_json::Value> {
    let logs = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(logs.len(), 1);
    let text = std::fs::read_to_string(&logs[0]).unwrap();
    assert!(!text.contains("fixture-pass"), "{text}");
    eprintln!("command log:\n{text}");
    text.lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn metadata_failure_retains_command_context_and_both_streams() {
    let node = PathBuf::from(std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit Node"));
    let root = tempfile::Builder::new()
        .prefix("npm view 中文 Spaces ")
        .tempdir()
        .unwrap();
    let npm = root.path().join("npm.cmd");
    std::fs::write(
        &npm,
        format!(
            "@echo off\r\n\"{}\" \"%~dp0failure.cjs\" %*\r\n",
            node.display()
        ),
    )
    .unwrap();
    std::fs::write(
        root.path().join("failure.cjs"),
        r#"
const fs = require('node:fs');
const path = require('node:path');
const environment = Object.fromEntries(['HOME','USERPROFILE','SystemRoot','COMSPEC',
  'PATH','OPENCODEX_HOME','npm_config_cache','npm_config_update_notifier','HTTPS_PROXY']
  .map(key => [key, process.env[key]]));
const invocation = {program:process.execPath, arguments:process.argv.slice(2),
  workingDirectory:process.cwd(), environment};
fs.writeFileSync(path.join(__dirname,'invocation.json'),JSON.stringify(invocation));
const stdout = 'query fixture stdout\n';
const stderr = 'npm error code EQUERYFIXTURE\nnpm error request https://fixture-user:fixture-pass@registry.invalid failed\n';
fs.writeFileSync(path.join(__dirname,'stdout.txt'),stdout);
fs.writeFileSync(path.join(__dirname,'stderr.txt'),stderr);
process.stdout.write(stdout);
process.stderr.write(stderr, () => process.exit(1));
"#,
    )
    .unwrap();
    let environment = EnvironmentPolicy {
        home: Some(root.path().join("User Home").into_os_string()),
        opencodex_home: root.path().join("Selected OpenCodex Home"),
        path: Some(node.parent().unwrap().as_os_str().to_owned()),
        https_proxy: Some("http://fixture-user:fixture-pass@127.0.0.1:9".into()),
        ..Default::default()
    };
    let error = query_remote_latest(
        &npm,
        root.path(),
        &root.path().join("cache/npm"),
        &root.path().join("logs/runtime-query"),
        &environment,
        "latest",
    )
    .unwrap_err();
    for file in ["invocation.json", "stdout.txt", "stderr.txt"] {
        eprintln!(
            "captured {file}: {}",
            std::fs::read_to_string(root.path().join(file)).unwrap()
        );
    }
    eprintln!("production query error: {error}");
    assert!(error.to_string().contains("EQUERYFIXTURE"));
    assert!(error.to_string().contains("退出码 1"));
    assert!(error.to_string().contains("日志："));
    assert!(!error.to_string().contains("fixture-pass"));
    let records = records(&root.path().join("logs/runtime-query"));
    assert_eq!(records[0]["program"], npm.to_str().unwrap());
    assert_eq!(
        records[0]["workingDirectory"],
        root.path().to_str().unwrap()
    );
    assert_eq!(
        records[0]["arguments"],
        serde_json::json!([
            "view",
            "@bitkyc08/opencodex@latest",
            "version",
            "dist.integrity",
            "--json"
        ])
    );
    assert_eq!(
        records[0]["environment"]["OPENCODEX_HOME"],
        environment.opencodex_home.to_str().unwrap()
    );
    assert_eq!(
        records[0]["environment"]["HOME"],
        environment.home.unwrap().to_str().unwrap()
    );
    assert_eq!(
        records[0]["environment"]["HTTPS_PROXY"],
        "http://***@127.0.0.1:9"
    );
    assert_eq!(records[1]["text"], "query fixture stdout\n");
    assert!(records[2]["text"]
        .as_str()
        .unwrap()
        .contains("EQUERYFIXTURE"));
    assert_eq!(records[3]["exitCode"], 1);
    assert_eq!(records[3]["reason"], "npm_view_exit");
    let payload = serde_json::to_value(error).unwrap();
    assert_eq!(payload["code"], 16);
}

type QueryEvidence = (
    Result<(String, Option<String>), AppError>,
    Vec<serde_json::Value>,
);

fn script_query(script: Option<&str>) -> QueryEvidence {
    script_query_paths(script, false)
}

fn script_query_paths(script: Option<&str>, verbatim: bool) -> QueryEvidence {
    let node = PathBuf::from(std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit Node"));
    let root = tempfile::Builder::new()
        .prefix("view 中文 Spaces ")
        .tempdir()
        .unwrap();
    let npm = root.path().join(if script.is_some() {
        "npm.cmd"
    } else {
        "missing.exe"
    });
    if let Some(script) = script {
        std::fs::write(root.path().join("query.cjs"), script).unwrap();
        std::fs::write(
            &npm,
            format!(
                "@echo off\r\n\"{}\" \"%~dp0query.cjs\" %*\r\n",
                node.display()
            ),
        )
        .unwrap();
    }
    let working = if verbatim {
        std::fs::canonicalize(root.path()).unwrap()
    } else {
        root.path().to_path_buf()
    };
    let environment = EnvironmentPolicy {
        home: Some(root.path().join("home").into_os_string()),
        opencodex_home: root.path().join("Selected Home"),
        path: Some(node.parent().unwrap().as_os_str().to_owned()),
        ..Default::default()
    };
    let result = run_npm_view_with_timeout(
        &working.join(npm.file_name().unwrap()),
        &working,
        &working.join("cache").join("npm"),
        &root.path().join("logs"),
        &environment,
        "test@latest",
        std::time::Duration::from_secs(2),
    );
    let records = records(&root.path().join("logs"));
    (result, records)
}

#[test]
fn metadata_windows_extended_paths_preserve_command_working_directory() {
    let (result, records) = script_query_paths(
        Some(
            r#"
const fs = require('node:fs');
if (fs.realpathSync(process.cwd()) !== fs.realpathSync(__dirname)) {
  process.stderr.write('wrong cwd: ' + process.cwd() + '\n', () => process.exit(1));
} else {
  process.stdout.write('{"version":"2.50.0"}');
}
"#,
        ),
        true,
    );
    assert_eq!(result.unwrap().0, "2.50.0");
    assert!(!records[2]["text"].as_str().unwrap().contains("CMD.EXE"));
}

#[test]
fn metadata_invalid_json_version_encoding_and_spawn_failures_keep_distinct_reasons() {
    for (script, reason) in [
        (
            Some("process.stdout.write('not json');"),
            "npm_view_metadata",
        ),
        (
            Some("process.stdout.write('{\"version\":\"latest\"}');"),
            "npm_view_metadata",
        ),
        (
            Some("process.stdout.write(Buffer.from([0xff]));"),
            "npm_view_encoding",
        ),
        (None, "npm_view_spawn"),
    ] {
        let (result, records) = script_query(script);
        assert!(
            matches!(result, Err(AppError::RemoteQuery { reason: actual, .. }) if actual == reason)
        );
        assert_eq!(records[3]["reason"], reason);
    }
}

#[test]
fn metadata_reads_both_pipes_concurrently_without_treating_stderr_as_failure() {
    let (result, records) = script_query(Some(
        "process.stderr.write('warning '.repeat(6000)); process.stdout.write('{\"version\":\"2.50.0\"}');"
    ));
    assert_eq!(result.unwrap().0, "2.50.0");
    assert_eq!(records[2]["capturedBytes"], 48000);
    assert_eq!(records[2]["complete"], true);
    assert_eq!(records[3]["exitCode"], 0);
    assert!(records[3]["reason"].is_null());
}

#[test]
fn metadata_non_utf8_stderr_keeps_following_diagnostic_lines() {
    let (result, records) = script_query(Some(
        "process.stderr.write(Buffer.from([0xff])); process.stderr.write(' native error\\nnpm error code EPERM\\n',()=>process.exit(1));"
    ));
    assert!(result.unwrap_err().to_string().contains("EPERM"));
    assert!(records[2]["text"]
        .as_str()
        .unwrap()
        .contains("native error\nnpm error code EPERM"));
}

#[test]
fn metadata_timeout_logs_partial_output_and_keeps_timeout_ipc_classification() {
    let (result, records) = script_query(Some(
        "process.stderr.write('before deadline\\n'); setInterval(()=>{},1000);",
    ));
    let payload = serde_json::to_value(result.unwrap_err()).unwrap();
    assert_eq!(payload["code"], 7);
    assert!(payload["message"]
        .as_str()
        .unwrap()
        .contains("before deadline"));
    assert_eq!(records[2]["text"], "before deadline\n");
    assert_eq!(records[3]["reason"], "npm_view_timeout");
}

#[test]
fn metadata_log_creation_failure_stops_before_executing_npm() {
    let root = tempfile::tempdir().unwrap();
    let npm = root.path().join("npm.cmd");
    let marker = root.path().join("unexpected-execution");
    std::fs::write(
        &npm,
        "@echo off\r\necho executed>\"%~dp0unexpected-execution\"\r\n",
    )
    .unwrap();
    let blocked = root.path().join("blocked-log-directory");
    std::fs::write(&blocked, "file, not a directory").unwrap();
    let environment = EnvironmentPolicy {
        home: Some(root.path().into()),
        opencodex_home: root.path().join("Selected Home"),
        ..Default::default()
    };
    let error = query_remote_latest(
        &npm,
        root.path(),
        &root.path().join("cache"),
        &blocked,
        &environment,
        "latest",
    )
    .unwrap_err();
    assert!(matches!(error, AppError::RemoteQuery { reason, .. } if reason == "npm_view_log"));
    assert!(!marker.exists());
}
