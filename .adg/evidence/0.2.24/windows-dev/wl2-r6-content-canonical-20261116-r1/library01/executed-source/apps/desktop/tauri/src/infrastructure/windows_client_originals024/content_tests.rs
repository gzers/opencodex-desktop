use super::*;
use crate::infrastructure::windows_official_runtime024::{create_private_directory, RetainedPeer};
use std::{
    os::windows::{fs::OpenOptionsExt, process::CommandExt},
    process::{Command, Stdio},
    time::Duration,
};
#[test]
fn wl2_r6_production_content_audit_projection_unknown_sensitive_and_exact_native_time_refuse() {
    for (raw, expected) in [
        (r"C:\Users\Example\app\home", r"C:\Users\[USER]\app\home"),
        ("C:/Users/Example/app/home", "C:/Users/[USER]/app/home"),
        (r"D:\Custom\隔离\home", r"D:\Custom\隔离\home"),
        (
            r"C:\USERS\TokenAccount\app-home",
            r"C:\USERS\[USER]\app-home",
        ),
        (r"D:\OpenCodex Data\home", r"D:\OpenCodex Data\home"),
        (r"C:\Uſers\Example\home", r"C:\Uſers\Example\home"),
        (r"C:\Users\测试用户\home", r"C:\Users\[USER]\home"),
    ] {
        assert_eq!(audit_home_projection(Path::new(raw)).unwrap(), expected);
    }
    for raw in [
        r"C:\Custom\api_key_folder\home",
        r"C:\Custom\secret-folder\home",
        r"C:\Custom\sk-synthetic000\home",
        r"C:\Custom\ghp_synthetic000000\home",
        r"C:\Custom\Bearer synthetic000\home",
        r"C:\Custom\eyJsynthetic000.AAAAAAAA.AAAA\home",
        r"C:\Custom\x&token=synthetic\home",
        r"C:\Custom\quoted'fragment\home",
        r"\\server\share\home",
        r"C:\Custom\..\home",
    ] {
        assert!(
            audit_home_projection(Path::new(raw)).is_err(),
            "unknown official projection must not be guessed"
        );
    }
    assert!(
        audit_home_projection(Path::new(&format!(r"C:\Custom\{}\home", "x".repeat(260)))).is_err()
    );
    assert_eq!(
        created_at(116444736000000000).unwrap().timestamp_millis(),
        0
    );
    assert_eq!(
        created_at(116444736000019999).unwrap().timestamp_millis(),
        1
    );
    assert!(created_at(0).is_err());
    let peer = RetainedPeer::current().unwrap();
    let actual = created_at(peer.identity().creation_filetime).unwrap();
    assert!(actual <= Utc::now());
    println!("r6 production-content DATA projection/local syntax/timestamp boundaries validated; native current process FILETIME maps to UTC milliseconds; no actual Owner/NormalExit/production Stop context minted");
}
#[test]
fn wl2_r6_production_content_fixed_bun_original_audit_source_projection_and_pin_rechecks() {
    let mut f = super::tree::tests::Fixture::new();
    let end = Instant::now() + Duration::from_secs(10);
    let code =
        std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")).unwrap();
    let bundle = code.join(
        "test/windows024-native/target/installer-w1/attempt01/stage/platform-bundle/runtime-bundle",
    );
    let package = bundle.join("runtime/opencodex/node_modules/@bitkyc08/opencodex");
    let bun = bundle.join("runtime/bun/bun.exe");
    let mut binary = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&bun)
        .unwrap();
    let mut raw = Vec::new();
    binary.read_to_end(&mut raw).unwrap();
    assert_eq!(
        hash::sha256_hex(&raw),
        "cb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244"
    );
    drop(raw);
    let mut pins = Vec::new();
    for (name, sha) in AUDIT_SOURCES {
        pins.push(AuditSourcePin::capture(package.join(name), sha, end).unwrap());
    }
    let cases = vec![
        r"C:\Users\Example\app\home".to_string(),
        "C:/Users/Example/app/home".into(),
        r"D:\Custom\隔离\home".into(),
        r"C:\USERS\TokenAccount\app-home".into(),
        r"D:\OpenCodex Data\home".into(),
        r"D:\with[bracket]\home".into(),
        r"C:\Uſers\Example\home".into(),
        r"C:\Users\测试用户\home".into(),
        r"D:\Custom\api_key_folder\home".into(),
        r"D:\Custom\sk-synthetic000\home".into(),
        r"D:\Custom\ghp_synthetic000000\home".into(),
        format!(r"D:\custom\{}\home", "x".repeat(245)),
    ];
    let dir = create_private_directory(&f.path().join("fixed-audit-projection")).unwrap();
    let stdout_path = dir.path().join("stdout.log");
    let stderr_path = dir.path().join("stderr.log");
    let stdout = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&stdout_path)
        .unwrap();
    let stderr = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&stderr_path)
        .unwrap();
    let script="import {pathToFileURL} from 'node:url'; const {redactUserPath}=await import(pathToFileURL(process.argv.at(-2)).href); console.log(JSON.stringify(JSON.parse(process.argv.at(-1)).map(v=>redactUserPath(v).slice(0,256))));";
    let mut child = Command::new(&bun)
        .arg("--eval")
        .arg(script)
        .arg(package.join("src/lib/redact.ts"))
        .arg(serde_json::to_string(&cases).unwrap())
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().unwrap()))
        .stderr(Stdio::from(stderr.try_clone().unwrap()))
        .spawn()
        .unwrap();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < end,
            "fixed Bun projection exceeded original test cutoff"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = std::fs::read(&stdout_path).unwrap();
    let error = std::fs::read(&stderr_path).unwrap();
    assert_eq!(
        status.code(),
        Some(0),
        "raw fixed Bun stderr={}",
        String::from_utf8_lossy(&error)
    );
    assert!(output.len() < 64 * 1024);
    let values: Vec<String> = serde_json::from_slice(&output).unwrap();
    assert_eq!(values.len(), cases.len());
    let mut rows = Vec::new();
    for (path, official) in cases.iter().zip(values) {
        let local = audit_home_projection(Path::new(path));
        if let Ok(value) = &local {
            assert_eq!(
                value, &official,
                "supported input must equal exact original official output"
            );
        } else {
            assert_ne!(
                official,
                path.as_str(),
                "counterexample is deliberately uncertain or changed by official redactor"
            );
        }
        rows.push(serde_json::json!({"input":path,"fixedOfficialProjection":official,"localProjection":local.ok(),"unknownRefused":audit_home_projection(Path::new(path)).is_err()}));
    }
    for pin in &mut pins {
        pin.check(end).unwrap();
    }
    println!(
        "r6-production-content-fixed-audit={}",
        serde_json::json!({"rawExit":status.code(),"sources":AUDIT_SOURCES,"bunSha256":"cb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244","rows":rows,"serviceStarted":false,"networkOrModelRequests":false})
    );
    drop(pins);
    drop(binary);
    drop(stdout);
    drop(stderr);
    f.retain();
}
#[test]
fn wl2_r6_production_content_exact_source_pin_denies_write_replace_and_wrong_hash() {
    use crate::infrastructure::windows_official_runtime024::PrivateRecord;
    let mut f = super::tree::tests::Fixture::new();
    let end = Instant::now() + Duration::from_secs(10);
    let dir = create_private_directory(&f.path().join("synthetic-source-pin")).unwrap();
    let source = dir.path().join("source.ts");
    let replacement = dir.path().join("replacement.ts");
    let bytes = b"export const syntheticNoCredentials = true;\n";
    drop(PrivateRecord::create(&source, bytes, 1024).unwrap());
    drop(PrivateRecord::create(&replacement, bytes, 1024).unwrap());
    let digest = hash::sha256_hex(bytes);
    let mut pin = AuditSourcePin::capture(source.clone(), &digest, end).unwrap();
    pin.check(end).unwrap();
    let write = OpenOptions::new()
        .write(true)
        .share_mode(1 | 2 | 4)
        .open(&source)
        .unwrap_err();
    assert!(write.raw_os_error().is_some());
    let rename = std::fs::rename(&replacement, &source).unwrap_err();
    assert!(rename.raw_os_error().is_some());
    pin.check(end).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    assert_eq!(std::fs::read(&replacement).unwrap(), bytes);
    assert!(AuditSourcePin::capture(source.clone(), &"e".repeat(64), end).is_err());
    println!(
        "r6-production-content-source-pin={}",
        serde_json::json!({"syntheticOnly":true,"source":source,
        "expectedSha256":digest,"writeRawError":write.raw_os_error(),"replaceRawError":rename.raw_os_error(),
        "wrongHashRefused":true,"originalBytesUnchanged":true,"officialFilesModified":false,"nativeAuthority":false})
    );
    drop(pin);
    f.retain();
}
