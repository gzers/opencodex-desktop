use super::*;
use crate::modules::model_config::{
    portable_v2::{export_portable_models_v2, ModelExportScopeV2},
    schema::{ModelRules, ModelSelection},
    schema_v2::{ChannelV2, ConnectionV2, LibraryV2, ModelConfigV2, PolicyV2, SourceKindV2},
    source_v2::source_from_provider_v2,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
const CHANNEL: &str = "11111111-1111-4111-8111-111111111111";
const OP: &str = "22222222-2222-4222-8222-222222222222";
const TIME: &str = "2026-11-06T10:00:00Z";
const SECRET: &str = "synthetic-wrapper-secret-value";

fn capture() -> CapturedObjects {
    let models = vec![ModelConfigV2 {
        raw_model_id: "fixture-model".into(),
        label: "Fixture".into(),
        observed: true,
        source: source_from_provider_v2(
            "fixture",
            "fixture-model",
            &json!({"adapter":"openai-chat"}),
            &digest(b"synthetic source"),
            SourceKindV2::ImportedFile,
        )
        .unwrap(),
        explicit_overrides: ModelRules::default(),
        binding: None,
    }];
    let channel = ChannelV2 {
        id: CHANNEL.into(),
        label: "Fixture".into(),
        connection: ConnectionV2 {
            provider_id: "fixture".into(),
            nodes: vec![],
            active_node_id: String::new(),
            credential_ref: None,
            enabled: true,
            allow_private_network: false,
            auth_mode: Some("key".into()),
            source_read_id: None,
            imported_config: None,
        },
        selection: ModelSelection::Whitelist(vec!["fixture-model".into()]),
        model_order: vec!["fixture-model".into()],
        models,
        template_refs: vec![],
        defaults: ModelRules::default(),
        family_overrides: BTreeMap::new(),
        naming: BTreeMap::new(),
        namespace: None,
    };
    let library = LibraryV2 {
        schema_version: 2,
        templates: vec![],
        submission_receipts: BTreeMap::new(),
    };
    let policy = PolicyV2 {
        schema_version: 2,
        channels: vec![channel],
        template_refs: vec![],
        default_channel_id: Some(CHANNEL.into()),
        pending_removals: vec![],
        recovery_id: None,
    };
    let scope = ModelExportScopeV2 {
        channel_ids: BTreeSet::from([CHANNEL.into()]),
        ..Default::default()
    };
    CapturedObjects {
        models: export_portable_models_v2(&library, &policy, &scope).unwrap(),
        manager: Default::default(),
        assets: Default::default(),
        channel_keys: SecretValues::new(),
        channel_authorizations: BTreeMap::new(),
        webdav_passwords: SecretValues::new(),
        content_versions: vec![],
    }
}
fn saved() -> SavedObjects {
    SavedObjects::capture(capture(), None).unwrap()
}
fn fail_code<T>(result: ModelResult<T>) -> String {
    match result {
        Ok(_) => panic!("expected failure"),
        Err(e) => e.code,
    }
}
fn sealed_document(document: &[u8], sections: Vec<String>, secret_objects: usize) -> Vec<u8> {
    portable_file::seal(
        portable_file::Manifest {
            id: OP.into(),
            created_at: TIME.into(),
            sections,
            secret_objects,
            document_digest: digest(document),
        },
        document,
        None,
    )
    .unwrap()
}
#[test]
fn file_objects_v2_plain_roundtrip_calls_real_model_codec_and_preserves_partitions() {
    let mut input = capture();
    input
        .channel_keys
        .insert(CHANNEL.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    input.content_versions.push(Version::unknown(
        "synthetic-scope",
        &digest(b"saved content"),
    ));
    let objects = SavedObjects::capture(input, None).unwrap();
    let before = semantic_digest(&objects).unwrap();
    let file = encode(&objects, OP, TIME, None, None).unwrap();
    assert!(!file.protected);
    assert_eq!(digest(file.bytes()), file.file_digest);
    let opened = decode(file.bytes(), None, &supported_schema(), None).unwrap();
    assert_eq!(semantic_digest(&opened.objects).unwrap(), before);
    assert_eq!(opened.objects.models().format_version, 6);
    assert_eq!(opened.objects.models().schema_version, 2);
    assert_eq!(
        opened.objects.partitions().channel_keys.get(CHANNEL),
        Some(SECRET)
    );
    assert_eq!(opened.objects.partitions().content_versions.len(), 1);
    assert_eq!(supported_schema().document.format, DOCUMENT_FORMAT);
    assert_eq!(supported_schema().document.version, 2);
}
#[test]
fn file_objects_v2_public_summary_does_not_include_secrets_or_source_or_mcp_values() {
    let mut input = capture();
    input
        .channel_keys
        .insert(CHANNEL.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    let o = SavedObjects::capture(input, None).unwrap();
    let summary = serde_json::to_string(&o.summary()).unwrap();
    assert!(!summary.contains(SECRET));
    for field in [
        "baseUrl",
        "source",
        "apiKey",
        "definition",
        "channelKeys",
        "username",
    ] {
        assert!(!summary.contains(&format!("\"{field}\"")));
    }
    assert_eq!(o.summary().channel_key_count, 1);
    assert_eq!(o.summary().channel_ids, vec![CHANNEL]);
}
#[test]
fn file_objects_v2_secret_aggregates_do_not_implement_debug_or_serialize() {
    // If any listed type accidentally gains Debug or Serialize, trait inference
    // becomes ambiguous and this test ceases to compile. No extra dependency.
    trait AmbiguousIfDebug<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfDebug<()> for T {}
    impl<T: ?Sized + std::fmt::Debug> AmbiguousIfDebug<u8> for T {}
    trait AmbiguousIfSerialize<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfSerialize<()> for T {}
    impl<T: ?Sized + serde::Serialize> AmbiguousIfSerialize<u8> for T {}
    let _ = <SavedObjects as AmbiguousIfDebug<_>>::marker;
    let _ = <SavedObjects as AmbiguousIfSerialize<_>>::marker;
    let _ = <CapturedObjects as AmbiguousIfDebug<_>>::marker;
    let _ = <CapturedObjects as AmbiguousIfSerialize<_>>::marker;
    let _ = <SecretValues as AmbiguousIfDebug<_>>::marker;
    let _ = <SecretValues as AmbiguousIfSerialize<_>>::marker;
    let _ = <EncodedFile as AmbiguousIfDebug<_>>::marker;
    let _ = <EncodedFile as AmbiguousIfSerialize<_>>::marker;
    let _ = <DecodedFile as AmbiguousIfDebug<_>>::marker;
    let _ = <DecodedFile as AmbiguousIfSerialize<_>>::marker;
    assert!(std::mem::needs_drop::<SecretValues>());
    assert!(std::mem::needs_drop::<WireObjects>());
}
#[test]
fn file_objects_v2_rejects_old_document_domains_and_old_model_versions() {
    let o = saved();
    let bytes = encode_document(&o, None).unwrap();
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["format"] = json!("OCXD_CONFIG_OBJECTS");
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "portable_objects_v2_version"
    );
    value["format"] = json!(DOCUMENT_FORMAT);
    value["version"] = json!(1);
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "portable_objects_v2_version"
    );
    value["version"] = json!(2);
    value["models"]["formatVersion"] = json!(5);
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "model_schema_unsupported"
    );
    value["models"]["formatVersion"] = json!(6);
    value["models"]["schemaVersion"] = json!(1);
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "model_schema_unsupported"
    );
}
#[test]
fn file_objects_v2_rejects_duplicate_nested_keys_and_unknown_preference_fields() {
    let o = saved();
    let bytes = encode_document(&o, None).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    let duplicate = text.replacen("\"version\":2", "\"version\":2,\"version\":2", 1);
    assert_eq!(
        fail_code(decode_document(duplicate.as_bytes(), None)),
        "portable_objects_v2_format"
    );
    let nested = text.replacen(
        "\"channelKeys\":{}",
        &format!(r#""channelKeys":{{"{CHANNEL}":"{SECRET}","{CHANNEL}":"other"}}"#),
        1,
    );
    assert_eq!(
        fail_code(decode_document(nested.as_bytes(), None)),
        "portable_objects_v2_format"
    );
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["manager"]["preferences"] = json!({"theme":"dark","network_proxy_host":"secret-host"});
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "portable_objects_v2_format"
    );
    value["manager"]["preferences"] = Value::Null;
    value["manager"]["futureField"] = json!(true);
    assert_eq!(
        fail_code(decode_document(&serde_json::to_vec(&value).unwrap(), None)),
        "portable_objects_v2_format"
    );
}
#[test]
fn file_objects_v2_exact_model_reference_and_source_validation_belongs_to_l1() {
    let mut input = capture();
    let evidence = input.models.payload.policy.as_ref().unwrap().channels[0].models[0]
        .source
        .fields["protocol"]
        .evidence
        .clone();
    input.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .source
        .fields
        .insert(
            "apiKey".into(),
            crate::modules::model_config::schema_v2::SourceFieldV2 {
                value: json!(SECRET),
                evidence,
            },
        );
    input.models.digest = crate::modules::model_config::schema_v2::digest(
        &crate::modules::model_config::schema_v2::canonical_bytes(&input.models.payload).unwrap(),
    );
    assert!(SavedObjects::capture(input, None).is_err());
    let mut input = capture();
    let channel = &mut input.models.payload.policy.as_mut().unwrap().channels[0];
    channel.models[0].binding = Some(
        crate::modules::model_config::schema_v2::ModelBindingV2::Manual {
            reference: crate::modules::model_config::schema_v2::RefV2 {
                template_id: OP.into(),
                revision_id: CHANNEL.into(),
                digest: digest(b"missing"),
            },
        },
    );
    input.models.digest = crate::modules::model_config::schema_v2::digest(
        &crate::modules::model_config::schema_v2::canonical_bytes(&input.models.payload).unwrap(),
    );
    assert_eq!(
        fail_code(SavedObjects::capture(input, None)),
        "model_dependency_missing"
    );
}
#[test]
fn file_objects_v2_semantic_digest_ignores_transport_time_but_not_secret_or_model_intent() {
    let mut input = capture();
    input
        .channel_keys
        .insert(CHANNEL.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    let o = SavedObjects::capture(input, None).unwrap();
    let first = semantic_digest(&o).unwrap();
    let mut doc: Value = serde_json::from_slice(&encode_document(&o, None).unwrap()).unwrap();
    doc["models"]["createdAt"] = json!("2026-11-06T11:00:00Z");
    let later = decode_document(&serde_json::to_vec(&doc).unwrap(), None).unwrap();
    assert_eq!(first, semantic_digest(&later).unwrap());
    doc["channelKeys"][CHANNEL] = json!("different-synthetic-secret");
    let changed = decode_document(&serde_json::to_vec(&doc).unwrap(), None).unwrap();
    assert_ne!(first, semantic_digest(&changed).unwrap());
}
#[test]
fn file_objects_v2_manifest_counts_sections_and_schema_are_binding() {
    let o = saved();
    let file = encode(&o, OP, TIME, None, None).unwrap();
    let mut wrong = supported_schema();
    wrong.document.format = "OCXD_CONFIG_OBJECTS".into();
    assert_eq!(
        fail_code(decode(file.bytes(), None, &wrong, None)),
        "portable_objects_v2_schema"
    );
    let doc = encode_document(&o, None).unwrap();
    let wrong = sealed_document(&doc, vec!["models".into()], 1);
    assert_eq!(
        fail_code(decode(&wrong, None, &supported_schema(), None)),
        "portable_manifest"
    );
    let duplicate = String::from_utf8(file.bytes().to_vec()).unwrap().replacen(
        "\"formatVersion\":3",
        "\"formatVersion\":3,\"formatVersion\":3",
        1,
    );
    assert_eq!(
        fail_code(decode(
            duplicate.as_bytes(),
            None,
            &supported_schema(),
            None
        )),
        "portable_objects_v2_format"
    );
}
#[test]
fn file_objects_v2_authorizations_require_real_host_direction_validation() {
    struct HostDirection {
        calls: std::cell::Cell<usize>,
        public: Value,
    }
    impl AuthorizationValidator for HostDirection {
        fn validate(
            &self,
            channel: &PortableChannelV2,
            authorization: &PortableAuthorization,
        ) -> ModelResult<()> {
            assert_eq!(channel.id, CHANNEL);
            self.calls.set(self.calls.get() + 1);
            // The host proves its captured public provider still belongs to the
            // original source binding before invoking the shared auth validator.
            if self.public["baseUrl"] != "https://fixture.example.test/v1" {
                return Err(codec_error("source_binding"));
            }
            crate::modules::model_config::provider_auth::validate_portable(
                &self.public,
                authorization,
            )
        }
    }
    fn auth() -> PortableAuthorization {
        serde_json::from_value(
            json!({"activeId":null,"fields":{"headers":{"X-Authorization":SECRET}}}),
        )
        .unwrap()
    }
    let mut input = capture();
    input.channel_authorizations.insert(CHANNEL.into(), auth());
    assert_eq!(
        fail_code(SavedObjects::capture(input, None)),
        "portable_authorization_context_required"
    );
    let direction = HostDirection {
        calls: std::cell::Cell::new(0),
        public: json!({"baseUrl":"https://fixture.example.test/v1","adapter":"openai"}),
    };
    let mut input = capture();
    input.channel_authorizations.insert(CHANNEL.into(), auth());
    let o = SavedObjects::capture(input, Some(&direction)).unwrap();
    let file = encode(&o, OP, TIME, None, Some(&direction)).unwrap();
    let result = decode(file.bytes(), None, &supported_schema(), Some(&direction)).unwrap();
    assert_eq!(result.objects.partitions().channel_authorizations.len(), 1);
    assert!(direction.calls.get() >= 3);
    assert_eq!(
        fail_code(decode(file.bytes(), None, &supported_schema(), None)),
        "portable_authorization_context_required"
    );
    let wrong = HostDirection {
        calls: std::cell::Cell::new(0),
        public: json!({"baseUrl":"https://wrong.example.test/v1","adapter":"openai"}),
    };
    assert_eq!(
        fail_code(decode(
            file.bytes(),
            None,
            &supported_schema(),
            Some(&wrong)
        )),
        "portable_authorization_binding"
    );
}
#[test]
fn file_objects_v2_rejects_unscoped_and_duplicate_secret_identities() {
    let mut input = capture();
    input
        .channel_keys
        .insert(OP.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    assert_eq!(
        fail_code(SavedObjects::capture(input, None)),
        "credential_scope"
    );
    let mut secrets = SecretValues::new();
    secrets
        .insert(CHANNEL.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    assert_eq!(
        fail_code(secrets.insert(CHANNEL.into(), Zeroizing::new("other".into()))),
        "credential_scope"
    );
    assert_eq!(secrets.get(CHANNEL), Some(SECRET));
    let mut input = capture();
    input
        .webdav_passwords
        .insert("missing-endpoint".into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    assert_eq!(
        fail_code(SavedObjects::capture(input, None)),
        "credential_scope"
    );
}
#[test]
fn file_objects_v2_preserves_real_assets_and_explicit_credentials_without_summary_leak() {
    let mut input = capture();
    let file = portable_assets::SkillFile {
        path: "SKILL.md".into(),
        digest: digest(b"# fixture"),
        data_b64: STANDARD.encode(b"# fixture"),
        executable: true,
        sensitive: false,
    };
    let files = vec![file];
    let skill = portable_assets::Skill {
        name: "fixture".into(),
        digest: digest(&serde_json::to_vec(&files).unwrap()),
        files,
        contains_secrets: false,
        warnings: vec![],
    };
    let mut secrets = portable_assets::Secrets::default();
    secrets.0.insert("/env/EXAMPLE_TOKEN".into(), SECRET.into());
    let mcp = portable_assets::Mcp {
        name: "fixture-mcp".into(),
        definition: json!({"command":"fixture","args":[]}),
        secret_fields: vec!["/env/EXAMPLE_TOKEN".into()],
        secrets,
        warnings: vec![],
    };
    input.assets.skills.push(skill);
    input.assets.mcps.push(mcp);
    let o = SavedObjects::capture(input, None).unwrap();
    let file = encode(&o, OP, TIME, None, None).unwrap();
    let decoded = decode(file.bytes(), None, &supported_schema(), None).unwrap();
    assert_eq!(decoded.objects.partitions().assets.skills.len(), 1);
    assert!(decoded.objects.partitions().assets.skills[0].files[0].executable);
    assert_eq!(
        decoded.objects.partitions().assets.mcps[0].secrets.0["/env/EXAMPLE_TOKEN"],
        SECRET
    );
    assert!(!serde_json::to_string(&decoded.objects.summary())
        .unwrap()
        .contains(SECRET));
}
#[test]
fn file_objects_v2_real_protection_roundtrip_wrong_password_and_ciphertext_tamper() {
    // Real portable_file Argon2/AES-GCM settings; never a reduced-cost substitute.
    let mut input = capture();
    input
        .channel_keys
        .insert(CHANNEL.into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    let o = SavedObjects::capture(input, None).unwrap();
    let file = encode(&o, OP, TIME, Some("synthetic passphrase"), None).unwrap();
    assert!(file.protected);
    assert!(!String::from_utf8_lossy(file.bytes()).contains(SECRET));
    let decoded = decode(
        file.bytes(),
        Some("synthetic passphrase"),
        &supported_schema(),
        None,
    )
    .unwrap();
    assert!(decoded.protected);
    assert_eq!(
        decoded.objects.partitions().channel_keys.get(CHANNEL),
        Some(SECRET)
    );
    assert_eq!(
        fail_code(decode(
            file.bytes(),
            Some("wrong synthetic passphrase"),
            &supported_schema(),
            None
        )),
        "portable_authentication"
    );
    assert_eq!(
        fail_code(decode(file.bytes(), None, &supported_schema(), None)),
        "passphrase_required"
    );
    let mut raw: Value = serde_json::from_slice(file.bytes()).unwrap();
    let ciphertext = raw["payload"]["ciphertext_b64"].as_str().unwrap();
    let mut body = STANDARD.decode(ciphertext).unwrap();
    body[0] ^= 1;
    raw["payload"]["ciphertext_b64"] = json!(STANDARD.encode(body));
    assert_eq!(
        fail_code(decode(
            &serde_json::to_vec(&raw).unwrap(),
            Some("synthetic passphrase"),
            &supported_schema(),
            None
        )),
        "portable_authentication"
    );
    assert_eq!(file.semantic_digest, semantic_digest(&o).unwrap());
}
#[test]
fn file_objects_v2_byte_budgets_apply_before_untrusted_decode_and_in_memory_capture() {
    assert_eq!(
        fail_code(decode_document(&vec![b' '; DOCUMENT_LIMIT + 1], None)),
        "portable_limit"
    );
    assert_eq!(
        fail_code(decode(
            &vec![b' '; FILE_LIMIT + 1],
            None,
            &supported_schema(),
            None
        )),
        "portable_limit"
    );
    let mut input = capture();
    let mut secrets = portable_assets::Secrets::default();
    secrets.0.insert("/env/EXAMPLE_TOKEN".into(), SECRET.into());
    input.assets.mcps.push(portable_assets::Mcp {
        name: "fixture".into(),
        definition: json!({"command":"fixture"}),
        secret_fields: vec!["/env/EXAMPLE_TOKEN".into()],
        secrets,
        warnings: vec!["x".repeat(DOCUMENT_LIMIT)],
    });
    assert_eq!(
        fail_code(SavedObjects::capture(input, None)),
        "portable_limit"
    );
}
#[test]
fn file_objects_v2_integrity_error_never_echoes_secret_material() {
    let mut input = capture();
    input
        .channel_keys
        .insert(CHANNEL.into(), Zeroizing::new(format!("{SECRET}\ninvalid")))
        .unwrap();
    let err = match SavedObjects::capture(input, None) {
        Err(e) => e,
        Ok(_) => panic!("expected invalid secret"),
    };
    assert!(!err.to_string().contains(SECRET));
    assert!(!format!("{err:?}").contains(SECRET));
    let mut value = json!({"nested":[SECRET,{"secret":SECRET}]});
    erase_value(&mut value);
    assert!(!value.to_string().contains(SECRET));
}
#[test]
fn file_objects_v2_host_owns_partitions_and_remote_self_sync_scope_is_separate() {
    let mut input = capture();
    input.manager.webdav.push(portable_manager::Endpoint {
        id: "fixture-endpoint".into(),
        url: "https://dav.example.test".into(),
        remote_path: "fixture".into(),
        username: "fixture-user".into(),
        conflict_policy: "ask".into(),
    });
    input
        .webdav_passwords
        .insert("fixture-endpoint".into(), Zeroizing::new(SECRET.into()))
        .unwrap();
    let file_objects = SavedObjects::capture(input, None).unwrap();
    assert_eq!(
        fail_code(file_objects.validate_remote_sync_scope()),
        "sync_self_scope"
    );
    let file = encode(&file_objects, OP, TIME, None, None).unwrap();
    let decoded = decode(file.bytes(), None, &supported_schema(), None).unwrap();
    let captured = decoded.objects.into_captured();
    assert_eq!(captured.manager.webdav.len(), 1);
    assert_eq!(
        captured.webdav_passwords.get("fixture-endpoint"),
        Some(SECRET)
    );
    let local = saved();
    assert!(local.validate_remote_sync_scope().is_ok());
}
#[test]
fn file_objects_v2_model_explicit_intent_remains_in_semantic_identity() {
    let original = saved();
    let baseline = semantic_digest(&original).unwrap();
    let mut parts = original.into_captured();
    parts.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .explicit_overrides
        .context_window = crate::modules::model_config::schema::Field::Custom(8192);
    parts.models.digest = crate::modules::model_config::schema_v2::digest(
        &crate::modules::model_config::schema_v2::canonical_bytes(&parts.models.payload).unwrap(),
    );
    let changed = SavedObjects::capture(parts, None).unwrap();
    assert_ne!(baseline, semantic_digest(&changed).unwrap());
    let mut parts = changed.into_captured();
    let previous = semantic_digest(&SavedObjects::capture(capture(), None).unwrap()).unwrap();
    parts.models.payload.policy.as_mut().unwrap().channels[0].models[0]
        .explicit_overrides
        .context_window = crate::modules::model_config::schema::Field::Inherit;
    parts.models.digest = crate::modules::model_config::schema_v2::digest(
        &crate::modules::model_config::schema_v2::canonical_bytes(&parts.models.payload).unwrap(),
    );
    parts
        .content_versions
        .push(Version::unknown("fixture", &digest(b"receipt-only-change")));
    let restored = SavedObjects::capture(parts, None).unwrap();
    assert_eq!(previous, semantic_digest(&restored).unwrap());
}
