use super::*;
use crate::infrastructure::windows_client_originals024::tree::tests::Fixture;
use std::time::Duration;
fn setup() -> (Fixture, PathBuf, RetainedRouteClose, Instant) {
    let fixture = Fixture::new();
    let home = fixture.path().join("route-home");
    drop(native::create_private_directory(&home).unwrap());
    let path = home.join("config.json");
    drop(
        native::PrivateRecord::create(
            &path,
            br#"{"providers":{"P":{"liveModels":false,"selectedModels":["synthetic/model"]}}}"#,
            4096,
        )
        .unwrap(),
    );
    let end = Instant::now() + Duration::from_secs(10);
    let pin = RoutePin::capture(&home, end).unwrap();
    (fixture, path, RetainedRouteClose::retain(pin, end), end)
}
#[test]
fn wl2_r6_route_close_native_all_original_main_security_and_two_ancestor_sets_close_before_independent_config_write(
) {
    let (mut fixture, path, mut candidate, end) = setup();
    let original = std::fs::read(&path).unwrap();
    assert_eq!(
        std::fs::write(&path, b"blocked before prepare")
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    candidate.prepare(end).unwrap();
    let trace = candidate.trace();
    let rows = trace["handles"]["rows"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        2 + 2 * path.parent().unwrap().ancestors().count(),
        "include primary and hidden BoundedAccess file plus BOTH actual pinned ancestor sets"
    );
    assert_eq!(rows[0]["row"]["key"], "route/security/file");
    assert_eq!(rows.last().unwrap()["row"]["key"], "route/config");
    assert!(rows.iter().all(|r| r["state"] == "Held"));
    let data = candidate.data().unwrap();
    assert_eq!(data["facts"]["configSha256"], hash::sha256_hex(&original));
    assert_eq!(
        std::fs::write(&path, b"still pinned after transfer")
            .unwrap_err()
            .raw_os_error(),
        Some(32)
    );
    candidate.close_owned(end).unwrap();
    let trace = candidate.trace();
    assert_eq!(trace["completedInOriginalWindow"], true);
    assert!(trace["handles"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["state"] == "ConfirmedClosed"));
    std::fs::write(&path, br#"{"providers":{},"syntheticSuccessor":true}"#).unwrap();
    let when = Instant::now();
    candidate
        .check_history(when + Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        candidate.data().unwrap(),
        data,
        "later independent config bytes are not adopted into old route facts"
    );
    assert_ne!(
        hash::sha256_hex(&std::fs::read(&path).unwrap()),
        data["facts"]["configSha256"]
    );
    println!(
        "r6-route-close-native={}",
        serde_json::json!({"case":"complete","data":data,"trace":trace,"newConfigWriteSucceeded":true,"productionStopQualified":false})
    );
    fixture.retain();
}
#[test]
fn wl2_r6_route_close_native_partial_protected_and_postclose_failure_keep_exact_owned_prefix_without_retry(
) {
    for fault in [81, 85, 82] {
        let (mut fixture, path, mut candidate, end) = setup();
        candidate.prepare(end).unwrap();
        let error = candidate.close_inner(fault, end).unwrap_err();
        let trace = candidate.trace();
        println!(
            "r6-route-close-attempt={}",
            serde_json::json!({"fault":fault,"error":error.to_string(),"trace":trace,"beforeAssertionsAndWriteProbe":true})
        );
        let rows = trace["handles"]["rows"].as_array().unwrap();
        let closed = rows
            .iter()
            .filter(|r| r["state"] == "ConfirmedClosed")
            .count();
        let unknown = rows.iter().filter(|r| r["state"] == "Unknown").count();
        assert_eq!(
            closed,
            match fault {
                81 => 1,
                85 => 0,
                _ => rows.len(),
            }
        );
        assert_eq!(unknown, usize::from(fault == 85));
        if fault == 81 || fault == 85 {
            assert_eq!(rows.last().unwrap()["row"]["key"], "route/config");
            assert_eq!(rows.last().unwrap()["state"], "Held");
        }
        assert_eq!(trace["completedInOriginalWindow"], false);
        assert_eq!(trace["poisoned"], true);
        assert_ne!(
            error.kind(),
            io::ErrorKind::TimedOut,
            "an upstream budget error cannot satisfy these specific close windows"
        );
        match fault {
            81 => assert!(error.to_string().contains("first confirmed native")),
            85 => assert!(rows[0]["rawError"].is_number()),
            _ => assert!(error
                .to_string()
                .contains("all actual route handles closed")),
        }
        if fault == 82 {
            std::fs::write(&path, b"all native closes happened but NOT completed proof").unwrap();
        } else {
            assert_eq!(
                std::fs::write(&path, b"hidden security observer/unknown remains")
                    .unwrap_err()
                    .raw_os_error(),
                Some(32)
            );
        }
        assert!(candidate.check_history(end).is_err());
        assert!(candidate.close_owned(end).is_err());
        assert_eq!(
            candidate.trace(),
            trace,
            "no retry/clear protection/recreate File after unknown/partial close"
        );
        println!(
            "r6-route-close-native={}",
            serde_json::json!({"fault":fault,"error":error.to_string(),"trace":trace,"productionStopQualified":false})
        );
        fixture.retain();
    }
}
#[test]
fn wl2_r6_route_close_native_wrong_deadline_or_data_refuses_before_close_and_retains_original_files(
) {
    for mode in 0..3 {
        let (mut fixture, path, mut candidate, end) = setup();
        if mode == 0 {
            assert!(candidate.prepare(end + Duration::from_nanos(1)).is_err());
            assert_eq!(candidate.trace()["originalPinRetained"], true);
        } else {
            candidate.prepare(end).unwrap();
            if mode == 1 {
                candidate.facts.as_mut().unwrap().config_sha256 = "e".repeat(64);
            } else {
                candidate.expected = Some(ExpectedHandleInventory::new(end));
            }
            assert!(candidate.close_owned(end).is_err());
            assert!(candidate.trace()["handles"]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["state"] == "Held"));
        }
        assert_eq!(
            std::fs::write(&path, b"must stay denied before any actual close")
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        let trace = candidate.trace();
        assert_eq!(trace["completedInOriginalWindow"], false);
        println!(
            "r6-route-close-native={}",
            serde_json::json!({"case":"preclose-refusal","mode":mode,"trace":trace,"productionStopQualified":false})
        );
        fixture.retain();
    }
}
