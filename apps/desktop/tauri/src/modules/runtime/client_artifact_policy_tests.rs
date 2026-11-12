use super::*;
use serde_json::json;

struct Fixture {
    original: Vec<u8>,
    ready: Vec<u8>,
    stopped: Vec<u8>,
    cache: Vec<u8>,
    audit_before: Vec<u8>,
    audit_ready: Vec<u8>,
    audit_stopped: Vec<u8>,
    routes: Vec<String>,
}
fn wire(value: Value) -> Vec<u8> {
    serde_json::to_vec(&value).unwrap()
}
fn file<'a>(bytes: &'a [u8], generation: &'static str) -> Snapshot<'a> {
    Snapshot::File {
        bytes,
        generation,
        protection: "verified-owner-and-access",
    }
}
fn event(writer: &str, target: &str, intent: &str) -> Value {
    json!({"at":"2026-11-11T10:00:01Z","pid":11,"ppid":10,"command":writer,
        "opencodexHome":"admitted-audit-home","target":target,"outcome":"written",
        "intent":intent,"writer":writer,"configSource":"file","routedBefore":0,"routedAfter":1})
}
fn line(value: Value) -> Vec<u8> {
    let mut bytes = wire(value);
    bytes.push(b'\n');
    bytes
}
impl Fixture {
    fn new() -> Self {
        // Native slugs can themselves contain slashes. Route ownership is an
        // independently declared exact set, never a slash/substring heuristic.
        let native =
            json!({"slug":"native/family","display_name":"Native original","extra":"keep"});
        let normalized =
            json!({"slug":"native/family","display_name":"Native projected","extra":"keep"});
        let routed = json!({"slug":"channel/model","display_name":"Channel model"});
        let original = wire(json!({"models":[native],"source":"original"}));
        let ready = wire(json!({"models":[normalized.clone(),routed],"source":"original"}));
        let stopped = wire(json!({"models":[normalized],"source":"original"}));
        let parsed: Value = serde_json::from_slice(&ready).unwrap();
        let cache = wire(
            json!({"fetched_at":"2000-01-01T00:00:00Z","client_version":"0.0.0","models":parsed["models"]}),
        );
        let audit_before = b"untouched historical audit bytes\n".to_vec();
        let mut audit_ready = audit_before.clone();
        audit_ready.extend(line(event("startup-cache", "cache", "cache")));
        let mut audit_stopped = audit_ready.clone();
        audit_stopped.extend(line(event("codex-restore", "catalog", "restore")));
        Self {
            original,
            ready,
            stopped,
            cache,
            audit_before,
            audit_ready,
            audit_stopped,
            routes: vec!["channel/model".into()],
        }
    }
    fn observations(&self) -> Observations<'_> {
        let ready = Stage {
            catalog: file(&self.ready, "ready-catalog"),
            cache: file(&self.cache, "ready-cache"),
            audit: file(&self.audit_ready, "ready-audit"),
        };
        Observations {
            before: Stage {
                catalog: file(&self.original, "original-catalog"),
                cache: Snapshot::Absent,
                audit: file(&self.audit_before, "original-audit"),
            },
            ready,
            pre_stop: ready,
            stopped: Stage {
                catalog: file(&self.stopped, "stop-catalog"),
                cache: ready.cache,
                audit: file(&self.audit_stopped, "stop-audit"),
            },
            declared_routes: Some(&self.routes),
            retained_pids: &[10, 11],
            audit_home: "admitted-audit-home",
            audit_not_before: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:00Z")
                .unwrap()
                .into(),
            audit_not_after: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:02Z")
                .unwrap()
                .into(),
        }
    }
}

#[test]
fn official_native_subset_controlled_cache_and_own_append_are_content_eligible() {
    let f = Fixture::new();
    assert_eq!(
        assess(&f.observations()),
        Ok(Assessment([
            (Artifact::Catalog, Action::RestoreOriginal),
            (Artifact::ModelsCache, Action::RestoreOriginal),
            (Artifact::CatalogAudit, Action::RestoreOriginal)
        ]))
    );
}

#[test]
fn original_bytes_with_new_file_identity_need_no_compensation() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    ctx.stopped = ctx.before;
    ctx.stopped.catalog = file(&f.original, "official-replaced-original-generation");
    assert_eq!(
        assess(&ctx),
        Ok(Assessment([
            (Artifact::Catalog, Action::AlreadyOriginal),
            (Artifact::ModelsCache, Action::AlreadyOriginal),
            (Artifact::CatalogAudit, Action::AlreadyOriginal)
        ]))
    );
}

#[test]
fn unknown_at_any_stage_or_artifact_never_becomes_absent() {
    let f = Fixture::new();
    for stage in 0..4 {
        for artifact in 0..3 {
            let mut ctx = f.observations();
            let s = match stage {
                0 => &mut ctx.before,
                1 => &mut ctx.ready,
                2 => &mut ctx.pre_stop,
                _ => &mut ctx.stopped,
            };
            match artifact {
                0 => s.catalog = Snapshot::Unknown,
                1 => s.cache = Snapshot::Unknown,
                _ => s.audit = Snapshot::Unknown,
            }
            assert_eq!(assess(&ctx), Err(Refusal::Unknown));
        }
    }
}

fn current_observations<'a>(ctx: Observations<'a>) -> CurrentRecoveryObservations<'a> {
    CurrentRecoveryObservations {
        before: ctx.before,
        ready: ctx.ready,
        current: ctx.stopped,
        declared_routes: ctx.declared_routes,
        retained_pids: ctx.retained_pids,
        audit_home: ctx.audit_home,
        audit_not_before: ctx.audit_not_before,
        audit_not_after: ctx.audit_not_after,
    }
}

#[test]
fn current_recovery_has_no_historical_pre_stop_and_does_not_relax_ordinary_stop() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    ctx.pre_stop.catalog = Snapshot::Unknown;
    assert_eq!(assess(&ctx), Err(Refusal::Unknown));
    assert!(assess_current_recovery(&current_observations(ctx)).is_ok());
    let mut ctx = f.observations();
    ctx.pre_stop.catalog = file(&f.stopped, "changed-before-stop");
    assert_eq!(assess(&ctx), Err(Refusal::ChangedSinceReady));
    assert!(assess_current_recovery(&current_observations(ctx)).is_ok());
}

#[test]
fn current_recovery_refuses_unknown_external_content_and_unproven_context() {
    let f = Fixture::new();
    for stage in 0..3 {
        for artifact in 0..3 {
            let mut ctx = current_observations(f.observations());
            let row = match stage {
                0 => &mut ctx.before,
                1 => &mut ctx.ready,
                _ => &mut ctx.current,
            };
            match artifact {
                0 => row.catalog = Snapshot::Unknown,
                1 => row.cache = Snapshot::Unknown,
                _ => row.audit = Snapshot::Unknown,
            };
            assert_eq!(assess_current_recovery(&ctx), Err(Refusal::Unknown));
        }
    }
    let user_model = wire(json!({"models":[{"slug":"user/added"}]}));
    for artifact in 0..3 {
        let mut ctx = current_observations(f.observations());
        match artifact {
            0 => ctx.current.catalog = file(&user_model, "external"),
            1 => ctx.current.cache = file(&user_model, "external"),
            _ => ctx.current.audit = file(b"external-event\n", "external"),
        }
        assert!(assess_current_recovery(&ctx).is_err());
    }
    let mut ctx = current_observations(f.observations());
    ctx.current.cache = Snapshot::File {
        bytes: &f.cache,
        generation: "external",
        protection: "new-owner",
    };
    assert_eq!(
        assess_current_recovery(&ctx),
        Err(Refusal::ProtectionChanged)
    );
    ctx = current_observations(f.observations());
    ctx.declared_routes = None;
    assert_eq!(assess_current_recovery(&ctx), Err(Refusal::Context));
    ctx = current_observations(f.observations());
    ctx.retained_pids = &[];
    assert_eq!(assess_current_recovery(&ctx), Err(Refusal::Context));
}

#[test]
fn content_identity_and_access_changes_after_ready_each_refuse_the_batch() {
    let f = Fixture::new();
    for kind in 0..3 {
        let mut ctx = f.observations();
        ctx.pre_stop.cache = match kind {
            0 => file(b"changed-by-user", "ready-cache"),
            1 => file(&f.cache, "replacement-generation"),
            _ => Snapshot::File {
                bytes: &f.cache,
                generation: "ready-cache",
                protection: "external-access",
            },
        };
        assert_eq!(assess(&ctx), Err(Refusal::ChangedSinceReady));
    }
}

#[test]
fn post_stop_external_access_is_not_repaired_or_adopted() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    ctx.stopped.cache = Snapshot::File {
        bytes: &f.cache,
        generation: "ready-cache",
        protection: "new-owner",
    };
    assert_eq!(assess(&ctx), Err(Refusal::ProtectionChanged));
}

#[test]
fn undeclared_ready_route_and_route_native_collision_refuse() {
    let mut f = Fixture::new();
    f.routes = vec!["native/family".into()];
    assert_eq!(assess(&f.observations()), Err(Refusal::Catalog));
    f.routes = vec!["another/model".into()];
    assert_eq!(assess(&f.observations()), Err(Refusal::Catalog));
    f.routes = vec!["channel/model".into(), "channel/model".into()];
    assert_eq!(assess(&f.observations()), Err(Refusal::Context));
    let mut ctx = f.observations();
    ctx.declared_routes = None;
    assert_eq!(assess(&ctx), Err(Refusal::Context));
}

#[test]
fn missing_original_native_rows_can_be_restored_but_new_rows_cannot_be_erased() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    let empty = wire(json!({"models":[]}));
    ctx.stopped.catalog = file(&empty, "official-empty");
    assert!(assess(&ctx).is_ok());
    for value in [
        json!({"models":[{"slug":"user/added"}]}),
        json!({"models":[{"slug":"native/family","display_name":"new-user-edit"}]}),
        json!({"models":[],"userSetting":true}),
    ] {
        let bytes = wire(value);
        let mut ctx = f.observations();
        ctx.stopped.catalog = file(&bytes, "external-edit");
        assert_eq!(assess(&ctx), Err(Refusal::Catalog));
    }
}

#[test]
fn cached_original_native_projection_is_eligible_but_changed_wrapper_is_not() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    let parsed: Value = serde_json::from_slice(&f.stopped).unwrap();
    let native_cache = json!({"fetched_at":"2000-01-01T00:00:00Z","client_version":"0.0.0","models":parsed["models"]});
    let bytes = wire(native_cache.clone());
    ctx.stopped.cache = file(&bytes, "official-native-cache");
    assert!(assess(&ctx).is_ok());
    for (key, value) in [
        ("fetched_at", json!("2026-11-11T10:00:01Z")),
        ("client_version", json!("1.2.3")),
        ("unexpected", json!(true)),
        ("models", json!([{"slug":"new-user-model"}])),
    ] {
        let mut candidate = native_cache.clone();
        candidate[key] = value;
        let bytes = wire(candidate);
        let mut ctx = f.observations();
        ctx.stopped.cache = file(&bytes, "other-cache");
        assert!(assess(&ctx).is_err());
    }
}

#[test]
fn duplicate_json_keys_or_model_ids_never_hide_a_conflict() {
    let f = Fixture::new();
    for bytes in [
        br#"{"models":[],"models":[]}"#.as_slice(),
        br#"{"models":[{"slug":"native/family"},{"slug":"native/family"}]}"#,
    ] {
        let mut ctx = f.observations();
        ctx.stopped.catalog = file(bytes, "malformed");
        assert!(assess(&ctx).is_err());
    }
}

#[test]
fn audit_is_only_a_preserved_prefix_plus_known_own_complete_events() {
    let f = Fixture::new();
    for (key, value) in [
        ("pid", json!(99)),
        ("ppid", json!(99)),
        ("opencodexHome", json!("another-home")),
        ("writer", json!("catalog-pull")),
        ("target", json!("unknown")),
        ("intent", json!("other")),
        ("configSource", json!("default")),
        ("at", json!("2026-11-11T09:00:00Z")),
        ("routedAfter", json!(1.5)),
        ("arbitrary", json!("not-allowed")),
    ] {
        let mut row = event("codex-restore", "catalog", "restore");
        row[key] = value;
        let mut actual = f.audit_ready.clone();
        actual.extend(line(row));
        let mut ctx = f.observations();
        ctx.stopped.audit = file(&actual, "bad-audit");
        assert!(assess(&ctx).is_err(), "key {key}");
    }
    for bytes in [b"not-the-original-prefix\n".as_slice(), b"unterminated"] {
        let mut ctx = f.observations();
        ctx.stopped.audit = file(bytes, "bad-audit");
        assert_eq!(assess(&ctx), Err(Refusal::Audit));
    }
}

#[test]
fn audit_compaction_is_not_mistaken_for_an_owned_suffix() {
    let f = Fixture::new();
    let short = line(event("codex-restore", "catalog", "restore"));
    let mut ctx = f.observations();
    ctx.stopped.audit = file(&short, "compacted");
    assert_eq!(assess(&ctx), Err(Refusal::Audit));
}

#[test]
fn a_valid_new_audit_event_does_not_authorize_loss_of_the_ready_prefix() {
    let f = Fixture::new();
    let mut replacement = f.audit_before.clone();
    replacement.extend(line(event("codex-restore", "catalog", "restore")));
    let mut ctx = f.observations();
    ctx.stopped.audit = file(&replacement, "lost-ready-prefix");
    assert_eq!(assess(&ctx), Err(Refusal::Audit));
}

#[test]
fn incomplete_audit_records_cannot_gain_eligibility_from_null_index_defaults() {
    let f = Fixture::new();
    for key in [
        "at",
        "pid",
        "ppid",
        "command",
        "opencodexHome",
        "target",
        "outcome",
        "intent",
        "writer",
        "configSource",
        "routedBefore",
        "routedAfter",
    ] {
        let mut row = event("codex-restore", "catalog", "restore");
        row.as_object_mut().unwrap().remove(key);
        let mut actual = f.audit_ready.clone();
        actual.extend(line(row));
        let mut ctx = f.observations();
        ctx.stopped.audit = file(&actual, "incomplete-event");
        assert_eq!(assess(&ctx), Err(Refusal::Audit), "missing {key}");
    }
}

#[test]
fn audit_projection_limit_counts_official_utf16_units_not_utf8_path_bytes() {
    let mut f = Fixture::new();
    let home = "中".repeat(150);
    let mut row = event("startup-cache", "cache", "cache");
    row["opencodexHome"] = json!(home);
    f.audit_ready = f.audit_before.clone();
    f.audit_ready.extend(line(row));
    f.audit_stopped = f.audit_ready.clone();
    let mut ctx = f.observations();
    ctx.audit_home = &home;
    assert!(assess(&ctx).is_ok());
    let too_long = "😀".repeat(129);
    ctx.audit_home = &too_long;
    assert_eq!(assess(&ctx), Err(Refusal::Context));
}

#[test]
#[ignore = "explicit pinned data-only projection from retained Windows native evidence; not native acceptance"]
fn retained_official_windows_artifact_bytes_replay_without_service_or_writes() {
    use base64::Engine;
    use serde::Deserialize;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Captured {
        state: String,
        bytes_base64: Option<String>,
        generation: Option<String>,
        protection: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Capture {
        catalog: Captured,
        cache: Captured,
        audit: Captured,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Replay {
        before: Capture,
        ready: Capture,
        pre_stop: Capture,
        stopped: Capture,
        declared_routes: Vec<String>,
        retained_pids: Vec<u32>,
        audit_home: String,
        not_before: chrono::DateTime<chrono::Utc>,
        not_after: chrono::DateTime<chrono::Utc>,
    }
    let path = std::path::PathBuf::from(
        std::env::var_os("OCXD_ARTIFACT_POLICY_REPLAY").expect("explicit replay path"),
    );
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.len() <= 16 * 1024 * 1024);
    assert_eq!(
        crate::infrastructure::hash::sha256_hex(&bytes),
        std::env::var("OCXD_ARTIFACT_POLICY_REPLAY_SHA256")
            .expect("independently pinned replay digest")
    );
    let value: Replay = serde_json::from_value(parse_strict_json_v2(&bytes).unwrap()).unwrap();
    let captures = [&value.before, &value.ready, &value.pre_stop, &value.stopped];
    let decoded: Vec<_> = captures
        .iter()
        .flat_map(|s| [&s.catalog, &s.cache, &s.audit])
        .map(|s| {
            s.bytes_base64
                .as_ref()
                .map(|b| base64::engine::general_purpose::STANDARD.decode(b).unwrap())
        })
        .collect();
    fn stage<'a>(s: &'a Capture, decoded: &'a [Option<Vec<u8>>], offset: usize) -> Stage<'a> {
        fn row<'a>(s: &'a Captured, bytes: Option<&'a [u8]>) -> Snapshot<'a> {
            match s.state.as_str() {
                "absent" => {
                    assert!(bytes.is_none() && s.generation.is_none() && s.protection.is_none());
                    Snapshot::Absent
                }
                "file" => Snapshot::File {
                    bytes: bytes.unwrap(),
                    generation: s.generation.as_deref().unwrap(),
                    protection: s.protection.as_deref().unwrap(),
                },
                _ => Snapshot::Unknown,
            }
        }
        Stage {
            catalog: row(&s.catalog, decoded[offset].as_deref()),
            cache: row(&s.cache, decoded[offset + 1].as_deref()),
            audit: row(&s.audit, decoded[offset + 2].as_deref()),
        }
    }
    let ctx = Observations {
        before: stage(&value.before, &decoded, 0),
        ready: stage(&value.ready, &decoded, 3),
        pre_stop: stage(&value.pre_stop, &decoded, 6),
        stopped: stage(&value.stopped, &decoded, 9),
        declared_routes: Some(&value.declared_routes),
        retained_pids: &value.retained_pids,
        audit_home: &value.audit_home,
        audit_not_before: value.not_before,
        audit_not_after: value.not_after,
    };
    assert_eq!(
        assess(&ctx),
        Ok(Assessment([
            (Artifact::Catalog, Action::RestoreOriginal),
            (Artifact::ModelsCache, Action::RestoreOriginal),
            (Artifact::CatalogAudit, Action::RestoreOriginal)
        ]))
    );
    println!("Retained official Windows three-artifact content replay passed; no native execution or write authorization.");
}

#[test]
fn a_complete_empty_route_set_is_distinct_from_unknown_discovery() {
    let mut f = Fixture::new();
    f.original = wire(json!({"models":[]}));
    f.ready = f.original.clone();
    f.stopped = f.original.clone();
    f.routes.clear();
    f.cache =
        wire(json!({"fetched_at":"2000-01-01T00:00:00Z","client_version":"0.0.0","models":[]}));
    assert!(assess(&f.observations()).is_ok());
    let mut ctx = f.observations();
    ctx.declared_routes = None;
    assert_eq!(assess(&ctx), Err(Refusal::Context));
}

#[test]
fn absence_and_original_cache_bytes_are_preserved_without_format_reinterpretation() {
    let f = Fixture::new();
    let mut ctx = f.observations();
    let opaque = b"original private cache content";
    ctx.before.cache = file(opaque, "original-cache");
    ctx.stopped.cache = file(opaque, "restored-cache");
    assert!(assess(&ctx).is_ok());
    ctx.stopped.cache = Snapshot::Absent;
    assert_eq!(assess(&ctx), Err(Refusal::Cache));
}

#[test]
fn oversize_or_unproven_snapshot_and_unsafe_context_refuse_without_outputting_content() {
    let f = Fixture::new();
    let huge = vec![b'x'; MAX_BYTES + 1];
    let mut ctx = f.observations();
    ctx.before.cache = file(&huge, "old");
    assert_eq!(assess(&ctx), Err(Refusal::Snapshot));
    let mut ctx = f.observations();
    ctx.retained_pids = &[10, 11, 11];
    assert_eq!(assess(&ctx), Err(Refusal::Context));
    let mut ctx = f.observations();
    ctx.audit_not_after = ctx.audit_not_before - chrono::Duration::seconds(1);
    assert_eq!(assess(&ctx), Err(Refusal::Context));
    let debug = format!("{:?}", file(b"must-not-be-printed", "identity"));
    assert!(!debug.contains("must-not-be-printed"));
}
