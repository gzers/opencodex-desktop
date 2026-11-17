use super::*;
#[test]
fn wl2_r7_quiesce_join_prefix_metadata_rejects_scope_deadline_claim_bytes_and_native_tag_substitutions(
) {
    let session = uuid::Uuid::new_v4();
    let end = Instant::now() + Duration::from_secs(10);
    let observed = serde_json::json!({"canonical":"synthetic DATA; not native original Owner permission","runtimeAuthority":false});
    // Private module test of DATA comparisons only. This literal is never
    // returned by a constructor or used against a real Owner/Job.
    let prefix = PreparedAbortPrefix {
        session,
        scope: "scope".into(),
        action: uuid::Uuid::new_v4().to_string(),
        end,
        epoch: "epoch".into(),
        claim: "claim".into(),
        digest: data_digest(&observed).unwrap(),
        observed,
    };
    prefix
        .check_read(Some(session), "epoch", "claim", end, &prefix.observed)
        .unwrap();
    assert!(prefix
        .check_read(None, "epoch", "claim", end, &prefix.observed)
        .is_err());
    assert!(prefix
        .check_read(
            Some(uuid::Uuid::new_v4()),
            "epoch",
            "claim",
            end,
            &prefix.observed
        )
        .is_err());
    assert!(prefix
        .check_read(Some(session), "other", "claim", end, &prefix.observed)
        .is_err());
    assert!(prefix
        .check_read(Some(session), "epoch", "other", end, &prefix.observed)
        .is_err());
    assert!(prefix
        .check_read(
            Some(session),
            "epoch",
            "claim",
            end + Duration::from_nanos(1),
            &prefix.observed
        )
        .is_err());
    assert!(prefix
        .check_read(
            Some(session),
            "epoch",
            "claim",
            end,
            &serde_json::json!({"canonical":"changed"})
        )
        .is_err());
    let proof = QuiescentOriginals {
        session,
        action: prefix.action.clone(),
        end,
        prefix_digest: prefix.digest.clone(),
    };
    proof.check_tag(&prefix).unwrap();
    for changed in ["session", "action", "deadline", "digest"] {
        let mut proof = QuiescentOriginals {
            session,
            action: prefix.action.clone(),
            end,
            prefix_digest: prefix.digest.clone(),
        };
        match changed {
            "session" => proof.session = uuid::Uuid::new_v4(),
            "action" => proof.action = uuid::Uuid::new_v4().to_string(),
            "deadline" => proof.end += Duration::from_nanos(1),
            _ => proof.prefix_digest = "f".repeat(64),
        }
        assert!(proof.check_tag(&prefix).is_err());
    }
    println!("r7 private prefix/tag DATA mismatches refused; no actual Owner/PreparedAbort/Quiescent native permission fabricated by these comparisons");
}
#[test]
fn wl2_r7_quiesce_join_prefix_metadata_expiry_stays_refused_without_renewal() {
    let end = Instant::now();
    let observed = serde_json::json!({"runtimeAuthority":false});
    let prefix = PreparedAbortPrefix {
        session: uuid::Uuid::new_v4(),
        scope: "scope".into(),
        action: uuid::Uuid::new_v4().to_string(),
        end,
        epoch: "epoch".into(),
        claim: "claim".into(),
        digest: data_digest(&observed).unwrap(),
        observed,
    };
    assert!(prefix
        .check_read(
            Some(prefix.session),
            "epoch",
            "claim",
            end,
            &prefix.observed
        )
        .is_err());
    assert_eq!(prefix.end, end);
}
