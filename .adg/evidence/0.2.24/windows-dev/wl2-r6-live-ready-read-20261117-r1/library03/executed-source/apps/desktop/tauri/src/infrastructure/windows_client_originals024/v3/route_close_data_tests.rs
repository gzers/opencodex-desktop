use super::super::{
    closed::{tests::setup, ClosedOperation},
    segments::ChainEntry,
    tests::plan,
};
use super::*;
use serde_json::{json, Value};
pub(in crate::infrastructure::windows_client_originals024::v3) fn data(
    p: &StoragePlan,
) -> RouteCloseData {
    // DATA fixture only, no retained native close/completed Stop constructed.
    let home = Path::new(&p.roots[0].root).parent().unwrap().join("H");
    let depth = home.ancestors().count();
    let mut security = serde_json::to_value(&p.roots[0].nodes[0].security).unwrap();
    security["fileId"] = "999".into();
    let volume = p.roots[0].custody_volume;
    let mut rows = vec![HandleRow {
        key: "route/security/file".into(),
        volume,
        file_id: "999".into(),
        directory: false,
        historical_work: false,
    }];
    for role in ["security/parent", "home"] {
        for i in 0..depth {
            rows.push(HandleRow {
                key: format!("route/{role}/{i}"),
                volume,
                file_id: (800 + i).to_string(),
                directory: true,
                historical_work: false,
            });
        }
    }
    rows.push(HandleRow {
        key: "route/config".into(),
        volume,
        file_id: "999".into(),
        directory: false,
        historical_work: false,
    });
    let inventory_digest =
        hash::sha256_hex(&encoded(&serde_json::to_value(&rows).unwrap()).unwrap());
    RouteCloseData {
        home,
        config_security: serde_json::from_value(security).unwrap(),
        config_bytes: 123,
        config_sha256: "a".repeat(64),
        route_digest: "b".repeat(64),
        inventory_digest,
        handle_count: rows.len(),
        rows,
    }
}
pub(in crate::infrastructure::windows_client_originals024::v3) fn qualify(
    g: &mut Progress,
    p: &StoragePlan,
) {
    for op in [
        RouteCloseOperation::RouteHandlesCloseIntent,
        RouteCloseOperation::RouteHandlesClosedVerified,
    ] {
        let e = g.route_close_event(op, data(p)).unwrap();
        g.apply_entry(p, &ChainEntry::RouteClose(e)).unwrap();
    }
}
fn reject(p: &StoragePlan, g: &Progress, v: Value) {
    let before = encoded(g).unwrap();
    let mut trial: Progress = serde_json::from_slice(&before).unwrap();
    match serde_json::from_value::<ChainEntry>(v) {
        Ok(e) => assert!(trial.apply_entry(p, &e).is_err()),
        Err(_) => {}
    }
    assert_eq!(encoded(&trial).unwrap(), before);
}
#[test]
fn wl2_r6_route_chain_exact_parent_intent_result_and_both_original_ancestor_sets_required() {
    let (mut f, p, _, _) = plan();
    let (mut g, closed) = setup(&p);
    let e = g
        .closed_event(ClosedOperation::StopClosureIntent, closed)
        .unwrap();
    g.apply_entry(&p, &ChainEntry::Closed(e)).unwrap();
    reject(
        &p,
        &g,
        serde_json::to_value(
            g.route_close_event(RouteCloseOperation::RouteHandlesClosedVerified, data(&p))
                .unwrap(),
        )
        .unwrap(),
    );
    let e = g
        .route_close_event(RouteCloseOperation::RouteHandlesCloseIntent, data(&p))
        .unwrap();
    g.apply_entry(&p, &ChainEntry::RouteClose(e)).unwrap();
    let good = serde_json::to_value(
        g.route_close_event(RouteCloseOperation::RouteHandlesClosedVerified, data(&p))
            .unwrap(),
    )
    .unwrap();
    for pointer in [
        "/route/configSha256",
        "/route/routeDigest",
        "/route/inventoryDigest",
        "/closureDigest",
        "/deadlineScope",
        "/bindingDigest",
        "/claimKey",
    ] {
        let mut v = good.clone();
        *v.pointer_mut(pointer).unwrap() = json!("f".repeat(64));
        reject(&p, &g, v);
    }
    let e: ChainEntry = serde_json::from_value(good.clone()).unwrap();
    g.apply_entry(&p, &e).unwrap();
    assert!(g.route_close.as_ref().unwrap().verified);
    let data = data(&p);
    data.validate().unwrap();
    for mode in 0..7 {
        let mut v = serde_json::to_value(&data).unwrap();
        match mode {
            0 => {
                v["rows"].as_array_mut().unwrap().remove(0);
            }
            1 => {
                v["rows"][0]["key"] = "route/config".into();
            }
            2 => v["rows"][1]["fileId"] = "7".into(),
            3 => v["rows"][1]["key"] = "route/security/parent/00".into(),
            4 => v["rows"][0]["directory"] = true.into(),
            5 => v["configSecurity"]["fileId"] = "9".into(),
            _ => v["rows"][1]["historicalWork"] = true.into(),
        };
        v["handleCount"] = json!(v["rows"].as_array().unwrap().len());
        v["inventoryDigest"] = json!(hash::sha256_hex(&encoded(&v["rows"]).unwrap()));
        assert!(
            serde_json::from_value::<RouteCloseData>(v)
                .unwrap()
                .validate()
                .is_err(),
            "self-hashed malformed complete inventory {mode}"
        );
    }
    println!("r6 route-close canonical DATA requires original closure parent then exact intent/result; complete primary/security/BOTH ancestor identity/order/count/flags validated, self-hashed omissions/substitutions refuse; no Files or Stop permit from DATA");
    f.retain();
}
