use super::declaration;

#[test]
fn config_byte_boundary_preserves_native_read_limit() {
    assert!(declaration(&vec![b' '; 2 * 1024 * 1024 + 1]).is_err());
    let mut bytes = br#"{"providers":{}}"#.to_vec();
    bytes.resize(2 * 1024 * 1024, b' ');
    assert_eq!(declaration(&bytes).unwrap(), Some(Vec::new()));
}

#[test]
fn independent_static_route_declaration_uses_official_slash_encoding_and_refuses_alias_collision() {
    let static_graph = serde_json::json!({"defaultModelAliases":false,"providers":{"Team_One":{"liveModels":false,"defaultAliases":false,"selectedModels":["gpt-6-sol","native/family-5.5"]}}});
    let bytes = serde_json::to_vec(&static_graph).unwrap();
    assert_eq!(
        declaration(&bytes).unwrap(),
        Some(vec![
            "Team_One/gpt-6-sol".into(),
            "Team_One/native-family-5.5".into()
        ])
    );
    let mut normalized = static_graph.clone();
    normalized["fastRows"] = true.into();
    assert_eq!(
        declaration(&serde_json::to_vec(&normalized).unwrap()).unwrap(),
        declaration(&bytes).unwrap()
    );
    normalized["fastRows"] = serde_json::json!("malformed");
    assert_eq!(
        declaration(&serde_json::to_vec(&normalized).unwrap()).unwrap(),
        None
    );
    let mut collision = static_graph.clone();
    collision["providers"]["Team_One"]["selectedModels"] =
        serde_json::json!(["native/family", "native-family"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&collision).unwrap()).unwrap(),
        None
    );
    let mut dynamic = static_graph.clone();
    dynamic["providers"]["Team_One"]["liveModels"] = true.into();
    dynamic["providers"]["Team_One"]["selectedModels"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&dynamic).unwrap()).unwrap(),
        None
    );
    let mut alias = static_graph;
    alias["customModels"] = serde_json::json!([{"id":"custom"}]);
    assert_eq!(
        declaration(&serde_json::to_vec(&alias).unwrap()).unwrap(),
        None
    );
    assert_eq!(declaration(br#"{"providers":null}"#).unwrap(), None);
    assert_eq!(declaration(br#"{}"#).unwrap(), None);
    assert_eq!(declaration(br#"{"providers":{}}"#).unwrap(), Some(vec![]));
    assert!(declaration(br#"{"providers":{},"providers":{}}"#).is_err());
}
#[test]
fn display_alias_flags_never_become_declared_route_ids_and_omitted_flags_are_valid() {
    let mut config = serde_json::json!({"providers":{"Team_One":{"liveModels":false,"selectedModels":["claude-opus-5-5","gpt-6-sol","vendor/gemini-3.5-pro"]}}});
    let expected = Some(vec![
        "Team_One/claude-opus-5-5".into(),
        "Team_One/gpt-6-sol".into(),
        "Team_One/vendor-gemini-3.5-pro".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for global in [false, true] {
        for local in [None, Some(false), Some(true)] {
            config["defaultModelAliases"] = global.into();
            if let Some(v) = local {
                config["providers"]["Team_One"]["defaultAliases"] = v.into();
            } else {
                config["providers"]["Team_One"]
                    .as_object_mut()
                    .unwrap()
                    .remove("defaultAliases");
            }
            config["providers"]["Team_One"]["modelAliases"] =
                serde_json::json!({"gpt-6-sol":"custom-sol"});
            config["fastRows"] = true.into();
            assert_eq!(
                declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
                expected
            );
        }
    }
    config["providers"]["Off"] = serde_json::json!({"disabled":true,"liveModels":true});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for (field, value) in [
        ("defaultAliases", serde_json::json!("bad")),
        ("modelAliases", serde_json::json!({"gpt-6-sol":3})),
        ("disabled", serde_json::json!("true")),
    ] {
        let mut bad = config.clone();
        bad["providers"]["Team_One"][field] = value;
        assert_eq!(
            declaration(&serde_json::to_vec(&bad).unwrap()).unwrap(),
            None
        );
    }
}
#[test]
fn explicit_nonempty_live_selection_is_independent_while_open_discovery_remains_unknown() {
    let mut config = serde_json::json!({"providers":{"Live":{"liveModels":true,"models":["stale-known-model"],"selectedModels":["claude-opus-5-5","vendor/gemini-3.5-pro"]}}});
    let expected = Some(vec![
        "Live/claude-opus-5-5".into(),
        "Live/vendor-gemini-3.5-pro".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Live"]
        .as_object_mut()
        .unwrap()
        .remove("liveModels");
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for models in [serde_json::json!([]), serde_json::json!(null)] {
        let mut c = config.clone();
        c["providers"]["Live"]["selectedModels"] = models;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
    config["providers"]["Live"]["liveModels"] = serde_json::json!("true");
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
}
#[test]
fn selection_defaults_static_roster_unions_models_and_retained_but_not_unselected_default() {
    let mut config = serde_json::json!({"providers":{"Static":{"liveModels":false,"models":["a","b"],"retainModels":["b","vendor/c"],"defaultModel":"not-in-explicit-roster"}}});
    let expected = Some(vec![
        "Static/a".into(),
        "Static/b".into(),
        "Static/vendor-c".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["selectedModels"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["selectedModels"] = serde_json::json!(["vendor/c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/vendor-c".into()])
    );
}
#[test]
fn selection_defaults_seed_only_static_empty_models_and_preserve_unknown_live_graph() {
    let mut config = serde_json::json!({"providers":{"Static":{"liveModels":false,"defaultModel":"seed","retainModels":["retained"]}}});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/retained".into(), "Static/seed".into()])
    );
    config["providers"]["Static"]["models"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/retained".into(), "Static/seed".into()])
    );
    config["providers"]["Static"]["liveModels"] = true.into();
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
}
#[test]
fn selection_defaults_disabled_equivalence_is_scoped_and_does_not_grant_colliding_ids() {
    let mut config = serde_json::json!({"disabledModels":["Static/vendor/c","native-only"],"providers":{"Static":{"liveModels":false,"models":["a","vendor/c"]},"Other":{"liveModels":false,"models":["vendor/c"]}}});
    let expected = Some(vec!["Other/vendor-c".into(), "Static/a".into()]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["disabledModels"] = serde_json::json!(["Static/vendor-c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["models"] = serde_json::json!(["vendor/c", "vendor-c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
    for bad in [
        serde_json::json!([3]),
        serde_json::json!(null),
        serde_json::json!({"Static/a":true}),
    ] {
        let mut c = config.clone();
        c["disabledModels"] = bad;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
}
#[test]
fn custom_declaration_overlays_same_model_and_adds_only_enabled_selected_custom_routes() {
    let mut config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["base"]}},"customModels":[{"provider":"P","modelId":"base","displayName":"friendly"},{"provider":"P","modelId":"vendor/custom","contextWindow":200000}]});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/base".into(), "P/vendor-custom".into()])
    );
    config["providers"]["P"]["selectedModels"] = serde_json::json!(["base"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/base".into()])
    );
    config["providers"]["P"]["selectedModels"] = serde_json::json!(["vendor/custom"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/vendor-custom".into()])
    );
    config["disabledModels"] = serde_json::json!(["P/vendor-custom"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec![])
    );
}
#[test]
fn custom_declaration_refuses_ambiguous_or_unbound_custom_routes_without_learning_from_catalog() {
    let config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["vendor/custom"]}},"customModels":[{"provider":"P","modelId":"vendor-custom"}]});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
    for custom in [
        serde_json::json!(null),
        serde_json::json!({}),
        serde_json::json!([{"provider":"Missing","modelId":"x"}]),
        serde_json::json!([{"provider":"P","modelId":""}]),
        serde_json::json!([{"provider":"P","modelId":3}]),
        serde_json::json!([{"provider":"P","modelId":"same"},{"provider":"P","modelId":"same"}]),
    ] {
        let mut c = config.clone();
        c["customModels"] = custom;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
}
#[test]
fn combo_declaration_uses_canonical_or_public_alias_without_relabeling_target_routes() {
    let mut config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["a","b"]}},"combos":{"mixed":{"strategy":"failover","targets":[{"provider":"P","model":"a"},{"provider":"P","model":"b"}]},"named":{"alias":"Public/mixed","targets":[{"provider":"P","model":"a"}]}}});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec![
            "P/a".into(),
            "P/b".into(),
            "Public/mixed".into(),
            "combo/mixed".into()
        ])
    );
    config["disabledModels"] = serde_json::json!(["Public/mixed"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/a".into(), "P/b".into(), "combo/mixed".into()])
    );
}
#[test]
fn combo_declaration_rejects_shadowing_missing_targets_nested_or_native_alias_authority() {
    let base = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["a"]}},"combos":{"mixed":{"targets":[{"provider":"P","model":"a"}]}}});
    for change in [
        serde_json::json!({"alias":"P/a"}),
        serde_json::json!({"alias":"combo/other"}),
        serde_json::json!({"alias":"a/b/c"}),
        serde_json::json!({"nativeAlias":true,"alias":"gpt-5.4"}),
        serde_json::json!({"targets":[]}),
        serde_json::json!({"targets":[{"provider":"Missing","model":"a"}]}),
        serde_json::json!({"targets":[{"provider":"P","model":"missing"}]}),
        serde_json::json!({"targets":[{"provider":"combo","model":"nested"}]}),
        serde_json::json!({"targets":[{"provider":"P","model":"a"},{"provider":"P","model":"a"}]}),
    ] {
        let mut c = base.clone();
        for (k, v) in change.as_object().unwrap() {
            c["combos"]["mixed"][k] = v.clone();
        }
        assert_eq!(
            declaration(&serde_json::to_vec(&c).unwrap()).unwrap(),
            None,
            "{change}"
        );
    }
}
