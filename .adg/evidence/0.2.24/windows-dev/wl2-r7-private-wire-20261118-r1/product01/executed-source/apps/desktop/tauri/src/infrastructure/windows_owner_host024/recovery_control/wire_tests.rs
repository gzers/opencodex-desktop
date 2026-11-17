use super::*;
use serde_json::{json, Value};
fn value(command: &str) -> Value {
    json!({"schema":SCHEMA,"requestId":"11111111-1111-4111-8111-111111111111","instanceId":"22222222-2222-4222-8222-222222222222","managerGeneration":"1","challenge":"a".repeat(64),"scopeDigest":"b".repeat(64),"command":command,"confirmation":if command=="begin-canonical-abort"{"explicit-before-first-resume-recovery"}else{"read-only"},"budget":{"transportActionId":"33333333-3333-4333-8333-333333333333","startedQpc":"1000","deadlineQpc":if command=="begin-canonical-abort"{"11000"}else{"6000"},"qpcFrequency":"1000"}})
}
fn frame(raw: &[u8]) -> Vec<u8> {
    let mut result = (raw.len() as u32).to_le_bytes().to_vec();
    result.extend_from_slice(raw);
    result
}
fn decode(value: &Value) -> io::Result<RequestV2> {
    RequestV2::decode(&frame(&serde_json::to_vec(value).unwrap()))
}
fn hello() -> HelloV2 {
    HelloV2 {
        schema: SCHEMA.into(),
        instance_id: value("read-worker-diagnostic")["instanceId"]
            .as_str()
            .unwrap()
            .into(),
        manager_generation: "1".into(),
        challenge: "a".repeat(64),
        scope_digest: "b".repeat(64),
    }
}
#[test]
fn wl2_r7_private_wire_v2_strict_every_field_duplicate_unknown_and_oldprotocol_refuse() {
    let good = value("begin-canonical-abort");
    decode(&good).unwrap();
    for key in good.as_object().unwrap().keys() {
        let mut bad = good.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(decode(&bad).is_err(), "missing {key}");
    }
    for key in good["budget"].as_object().unwrap().keys() {
        let mut bad = good.clone();
        bad["budget"].as_object_mut().unwrap().remove(key);
        assert!(decode(&bad).is_err(), "missingbudget {key}");
    }
    for (field, data) in [
        ("schema", json!(1)),
        ("schema", json!("windows-existing-failure-control-v1")),
        ("command", json!("explicit-recovery")),
        ("command", json!("start")),
        ("command", json!("force")),
        ("command", json!("restore-from-record")),
        ("pid", json!(21500)),
        ("path", json!("C:/arbitrary")),
        ("nativeProof", json!({})),
        ("requestId", json!("00000000-0000-0000-0000-000000000000")),
        ("managerGeneration", json!("01")),
        ("challenge", json!("A".repeat(64))),
    ] {
        let mut bad = good.clone();
        bad[field] = data;
        assert!(decode(&bad).is_err(), "{field}");
    }
    let raw = serde_json::to_string(&good).unwrap();
    for bad in [
        raw.replacen(
            '{',
            "{\"schema\":\"windows-before-resume-private-control-v2\",",
            1,
        ),
        raw.replace(
            "\"startedQpc\":\"1000\"",
            "\"startedQpc\":\"1000\",\"startedQpc\":\"1000\"",
        ),
        format!("{raw}{{}}"),
        format!("{raw}\0"),
    ] {
        assert!(RequestV2::decode(&frame(bad.as_bytes())).is_err());
    }
}
#[test]
fn wl2_r7_private_wire_v2_frame_and_confirmation_refuse_unknown_beforeadmission() {
    let good = frame(&serde_json::to_vec(&value("read-canonical-history")).unwrap());
    assert!(RequestV2::decode(&good[..good.len() - 1]).is_err());
    let mut extra = good.clone();
    extra.push(0);
    assert!(RequestV2::decode(&extra).is_err());
    assert!(RequestV2::decode(&frame(&vec![b' '; 65537])).is_err());
    assert!(RequestV2::decode(&frame(&[255])).is_err());
    assert!(RequestV2::decode(&frame(b"")).is_err());
    for command in [
        "begin-canonical-abort",
        "read-worker-diagnostic",
        "read-canonical-history",
    ] {
        let mut data = value(command);
        data["confirmation"] = json!(if command == "begin-canonical-abort" {
            "read-only"
        } else {
            "explicit-before-first-resume-recovery"
        });
        assert!(decode(&data).is_err());
    }
}
#[test]
fn wl2_r7_private_wire_v2_exact_original10_or5_qpc_window_floor_and_elapsed_notrenewed() {
    let now = Instant::now();
    for (command, seconds) in [
        ("begin-canonical-abort", 10),
        ("read-worker-diagnostic", 5),
        ("read-canonical-history", 5),
    ] {
        let receipt = decode(&value(command))
            .unwrap()
            .admit_at(&hello(), &mut BTreeSet::new(), 3000, 1000, now)
            .unwrap();
        assert_eq!(receipt.deadline.started, now - Duration::from_secs(2));
        assert_eq!(receipt.deadline.end, now + Duration::from_secs(seconds - 2));
        assert_eq!(receipt.request_id, value(command)["requestId"]);
        assert_eq!(
            receipt.transport_action_id,
            value(command)["budget"]["transportActionId"]
        );
        let mut longer = value(command);
        longer["budget"]["deadlineQpc"] = json!((1000 + (seconds + 1) * 1000).to_string());
        assert!(decode(&longer)
            .unwrap()
            .admit_at(&hello(), &mut BTreeSet::new(), 3000, 1000, now)
            .is_err());
    }
    let mut data = value("read-canonical-history");
    data["budget"]["startedQpc"] = json!("100");
    data["budget"]["deadlineQpc"] = json!("104");
    data["budget"]["qpcFrequency"] = json!("3");
    let result = decode(&data)
        .unwrap()
        .admit_at(&hello(), &mut BTreeSet::new(), 101, 3, now)
        .unwrap();
    assert_eq!(result.deadline.end, now + Duration::from_secs(1));
    assert_eq!(
        result.deadline.started,
        now - Duration::from_nanos(333333334)
    );
}
#[test]
fn wl2_r7_private_wire_v2_future_expiry_freq_overflow_and_clock_substitutions_refuse() {
    let now = Instant::now();
    for (counter, freq) in [
        (999, 1000),
        (6000, 1000),
        (3000, 999),
        (3000, 0),
        (3000, u64::MAX),
    ] {
        assert!(decode(&value("read-canonical-history"))
            .unwrap()
            .admit_at(&hello(), &mut BTreeSet::new(), counter, freq, now)
            .is_err());
    }
    for field in ["startedQpc", "deadlineQpc", "qpcFrequency"] {
        for text in ["+1", "01", "-1", "1e3", "18446744073709551616"] {
            let mut data = value("read-canonical-history");
            data["budget"][field] = json!(text);
            assert!(decode(&data).is_err());
        }
    }
    let mut data = value("begin-canonical-abort");
    data["budget"]["qpcFrequency"] = json!(i64::MAX.to_string());
    data["budget"]["startedQpc"] = json!("0");
    data["budget"]["deadlineQpc"] = json!("1");
    assert!(decode(&data)
        .unwrap()
        .admit_at(&hello(), &mut BTreeSet::new(), 0, i64::MAX as u64, now)
        .is_err());
}
#[test]
fn wl2_r7_private_wire_v2_fresh_challenge_scope_and_every_replay_key_refuse() {
    let now = Instant::now();
    let mut used = BTreeSet::new();
    let base = value("read-canonical-history");
    decode(&base)
        .unwrap()
        .admit_at(&hello(), &mut used, 3000, 1000, now)
        .unwrap();
    assert_eq!(used.len(), 3);
    for changed in ["requestId", "transportActionId", "challenge"] {
        let mut data = base.clone();
        let mut greeting = hello();
        data["requestId"] = json!(uuid::Uuid::new_v4().to_string());
        data["budget"]["transportActionId"] = json!(uuid::Uuid::new_v4().to_string());
        data["challenge"] = json!("c".repeat(64));
        greeting.challenge = "c".repeat(64);
        if changed == "transportActionId" {
            data["budget"][changed] = base["budget"][changed].clone();
        } else {
            data[changed] = base[changed].clone();
            if changed == "challenge" {
                greeting.challenge = "a".repeat(64);
            }
        }
        assert!(decode(&data)
            .unwrap()
            .admit_at(&greeting, &mut used, 3000, 1000, now)
            .is_err());
    }
    for field in [
        "schema",
        "instanceId",
        "managerGeneration",
        "challenge",
        "scopeDigest",
    ] {
        let mut h = serde_json::to_value(hello()).unwrap();
        h[field] = match field {
            "schema" => json!("old-v1"),
            "instanceId" => json!(uuid::Uuid::new_v4().to_string()),
            "managerGeneration" => json!("2"),
            _ => json!("c".repeat(64)),
        };
        let greeting: HelloV2 = serde_json::from_value(h).unwrap();
        assert!(decode(&base)
            .unwrap()
            .admit_at(&greeting, &mut BTreeSet::new(), 3000, 1000, now)
            .is_err());
    }
    let mut huge = (0..16382)
        .map(|i| format!("oldkey{i}"))
        .collect::<BTreeSet<_>>();
    assert!(decode(&base)
        .unwrap()
        .admit_at(&hello(), &mut huge, 3000, 1000, now)
        .is_err());
    assert_eq!(huge.len(), 16382);
}
#[test]
fn wl2_r7_private_wire_v2_locator_native_target_data_is_strict_local_and_notauthority() {
    let name = format!(
        "{s}{s}.{s}pipe{s}ocxd-owner024-S-1-5-21-synthetic-{}",
        "b".repeat(64),
        s = char::from(92)
    );
    let good = json!({"schema":SCHEMA,"instanceId":"22222222-2222-4222-8222-222222222222","pipeName":name,"brokerPid":123,"brokerFiletime":"42","scopeDigest":"b".repeat(64)});
    LocatorV2::decode(&serde_json::to_vec(&good).unwrap()).unwrap();
    for field in good.as_object().unwrap().keys() {
        let mut bad = good.clone();
        bad.as_object_mut().unwrap().remove(field);
        assert!(LocatorV2::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (field, data) in [
        ("schema", json!("windows-existing-failure-control-v1")),
        ("brokerPid", json!(0)),
        ("brokerFiletime", json!("042")),
        ("pipeName", json!(r"\\remote\pipe\owner")),
        ("pipeName", json!(format!("{name}\0"))),
        ("ownerHandle", json!(1)),
        ("canRestore", json!(true)),
    ] {
        let mut bad = good.clone();
        bad[field] = data;
        assert!(LocatorV2::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
