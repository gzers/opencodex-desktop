//! Real production save/snapshot calls. The only substituted boundary is the
//! process-local credential store; every configuration file lives in tempfile.
use super::*;
use crate::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::{agent, execution_directory, upstream::UpstreamSnapshot},
    state::ProcessContext,
};
use std::collections::BTreeMap;
const SERVICE: &str = "opencodex.provider-api-key.v1";
const OLD: &str = "Krill-1";
const NEW: &str = "Krill";
const GROUP: &str = "ocxd-30000000000040008000000000000001";
const UUID_CREDENTIAL: &str = "20000000-0000-4000-8000-000000000001";
fn slot(value: &str) -> String {
    sha256(value.as_bytes())[7..15].into()
}
fn put(account: &str, value: &str) {
    keyring::Entry::new(SERVICE, account)
        .unwrap()
        .set_password(value)
        .unwrap();
}
fn get(account: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, account)
        .unwrap()
        .get_password()
        .ok()
}
fn read_value(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
fn write_value(path: &Path, value: &Value) {
    atomic_write(path, &serde_json::to_vec(value).unwrap(), 0o600).unwrap();
}
struct Fixture {
    temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    id: String,
    raw: Value,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("manager");
        let home = temp.path().join("official");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let a = slot("fixture-a");
        let b = slot("fixture-b");
        for (account, value) in [
            (format!("{OLD}/{a}"), "fixture-a"),
            (format!("{OLD}/{b}"), "fixture-b"),
            (format!("{OLD}/header"), "fixture-header"),
        ] {
            put(&account, value);
        }
        let raw = json!({"providers":{
            OLD:{"name":OLD,"adapter":"openai-chat","authMode":"key","baseUrl":"https://fixture.invalid/v1","apiKey":format!("keychain:{OLD}/{b}"),
                "apiKeyPool":[{"id":a,"key":format!("keychain:{OLD}/{a}"),"label":"first","addedAt":100,"future":7},{"id":b,"key":format!("keychain:{OLD}/{b}"),"label":"second","addedAt":200}],
                "headers":{"X-Key":format!("keychain:{OLD}/header"),"X-Env":"$RENAME_TEST_UNUSED"},
                "models":["vendor/a:8b","vendor-a:8b","Mixed_Case/V2"],"selectedModels":["vendor/a:8b","vendor-a:8b","Mixed_Case/V2"],"futureField":{"literal":"Krill-1/leave-opaque","keep":42}},
            "Krill-10":{"adapter":"openai-chat","authMode":"key","baseUrl":"https://neighbor.invalid/v1","apiKey":"fixture-neighbor","models":["b1","b2"]},
            "openai":{"adapter":"openai-responses","authMode":"oauth","baseUrl":"https://official.invalid/v1","models":["native"]}},
            "defaultProvider":OLD,"modelPickerOrder":["native-head","Krill-1/vendor/a:8b","Krill-10/b1","opaque-middle","Krill-1/vendor-a:8b","Krill-1/opaque/row","Krill-10/b2"],
            "disabledModels":["Krill-1/Mixed_Case/V2","Krill-10/untouched","Krill-1/unknown/row"],"futureRoot":{"keep":true},
            "modelDiscovery":{"knownModels":{OLD:{"models":["vendor/a:8b"]}},"recentArrivals":{OLD:["vendor-a:8b"]}}});
        write_value(&home.join("config.json"), &raw);
        let snapshot = snapshot(&root, &home).unwrap();
        let mut policy = snapshot.policy;
        let c = policy
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == OLD)
            .unwrap();
        let id = c.id.clone();
        c.template_refs = vec![TemplateRef {
            template_id: snapshot.library.templates[0].id.clone(),
            revision_id: snapshot.library.templates[0].revisions[0].id.clone(),
        }];
        record_layout(&root, &policy);
        let mut layout = read_value(&root.join(METADATA));
        layout["channels"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|c| c["id"] == id)
            .unwrap()["connection"]["credentialRef"] = json!(UUID_CREDENTIAL);
        write_value(&root.join(METADATA), &layout);
        keyring::Entry::new(
            crate::infrastructure::keychain::keychain_service_name(),
            &format!("model_channel:{UUID_CREDENTIAL}"),
        )
        .unwrap()
        .set_password("fixture-uuid-owned")
        .unwrap();
        write_value(
            &root.join("manager-state/preferences.json"),
            &crate::modules::preferences::document_from_preferences(
                &crate::modules::preferences::Preferences::default(),
            ),
        );
        let mut prefs = read_value(&root.join("manager-state/preferences.json"));
        prefs["modelTemplateAgent"] = json!({"providerId":OLD,"modelId":"vendor/a:8b","reasoningEffort":null,"fast":null,"channelId":id,"connectionDigest":"old-digest"});
        prefs["futurePrefs"] = json!({"keep":7});
        write_value(&root.join("manager-state/preferences.json"), &prefs);
        write_value(
            &root.join("manager-state/model-execution-source-ids.json"),
            &json!({OLD:id}),
        );
        write_value(
            &root.join("manager-state/model-protocol-mappings.json"),
            &json!([{"id":"30000000-0000-4000-8000-000000000001","channelId":id,"protocol":"anthropic","providerId":GROUP}]),
        );
        Self {
            temp,
            root,
            home,
            id,
            raw,
        }
    }
    fn bytes(&self) -> Vec<Vec<u8>> {
        [
            self.home.join("config.json"),
            self.root.join(METADATA),
            self.root.join("manager-state/preferences.json"),
            self.root
                .join("manager-state/model-execution-source-ids.json"),
            self.root.join("manager-state/model-protocol-mappings.json"),
        ]
        .iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect()
    }
    fn request(&self, scoped: bool) -> SaveRequest {
        let s = snapshot(&self.root, &self.home).unwrap();
        let mut policy = s.policy;
        let c = policy
            .channels
            .iter_mut()
            .find(|c| c.id == self.id)
            .unwrap();
        c.label = NEW.into();
        c.connection.provider_id = NEW.into();
        SaveRequest {
            expected_file: s.policy_digest,
            policy,
            credentials: vec![],
            scope: scoped.then(|| SaveScope {
                channel_id: self.id.clone(),
            }),
        }
    }
    fn context(&self) -> ProcessContext {
        ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(self.temp.path().join("not-executed-ocx")),
            working_directory: self.temp.path().into(),
            opencodex_home: self.home.clone(),
        }
    }
    fn assert_saved(&self) {
        let raw = read_value(&self.home.join("config.json"));
        assert!(raw["providers"].get(OLD).is_none());
        assert_eq!(raw["defaultProvider"], NEW);
        assert_eq!(
            raw["providers"]["Krill-10"],
            self.raw["providers"]["Krill-10"]
        );
        assert_eq!(raw["futureRoot"], self.raw["futureRoot"]);
        assert_eq!(
            raw["providers"][NEW]["futureField"],
            self.raw["providers"][OLD]["futureField"]
        );
        assert_eq!(
            raw["modelPickerOrder"],
            json!([
                "native-head",
                "Krill/vendor/a:8b",
                "Krill-10/b1",
                "opaque-middle",
                "Krill/vendor-a:8b",
                "Krill/opaque/row",
                "Krill-10/b2"
            ])
        );
        assert_eq!(
            raw["disabledModels"],
            json!([
                "Krill/Mixed_Case/V2",
                "Krill-10/untouched",
                "Krill/unknown/row"
            ])
        );
        assert!(raw["modelDiscovery"]["knownModels"].get(OLD).is_none());
        assert_eq!(
            raw["modelDiscovery"]["recentArrivals"][NEW],
            json!(["vendor-a:8b"])
        );
        for (index, value) in ["fixture-a", "fixture-b"].iter().enumerate() {
            let id = slot(value);
            assert_eq!(raw["providers"][NEW]["apiKeyPool"][index]["id"], id);
            assert_eq!(
                raw["providers"][NEW]["apiKeyPool"][index]["key"],
                format!("keychain:{NEW}/{id}")
            );
            assert_eq!(get(&format!("{NEW}/{id}")).as_deref(), Some(*value));
            assert!(get(&format!("{OLD}/{id}")).is_none());
        }
        assert_eq!(raw["providers"][NEW]["apiKeyPool"][0]["future"], 7);
        assert_eq!(raw["providers"][NEW]["apiKeyPool"][0]["label"], "first");
        assert_eq!(raw["providers"][NEW]["apiKeyPool"][1]["addedAt"], 200);
        let view = credential(&self.home, NEW).unwrap();
        assert_eq!(view.value, "fixture-b");
        let s = snapshot(&self.root, &self.home).unwrap();
        let c = s.policy.channels.iter().find(|c| c.id == self.id).unwrap();
        assert_eq!(c.connection.provider_id, NEW);
        assert_eq!(c.label, NEW);
        assert_eq!(c.template_refs.len(), 1);
        let stored = read_value(&self.root.join(METADATA));
        assert_eq!(
            stored["channels"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == self.id)
                .unwrap()["connection"]["credentialRef"],
            UUID_CREDENTIAL
        );
        assert_eq!(
            crate::modules::model_config::discovery::credential(UUID_CREDENTIAL)
                .unwrap()
                .as_str(),
            "fixture-uuid-owned"
        );
        let agent = agent::settings(&self.root).unwrap().unwrap();
        assert_eq!(agent.provider_id, NEW);
        assert_eq!(agent.channel_id.as_deref(), Some(self.id.as_str()));
        let upstream = UpstreamSnapshot {
            version: "fixture".into(),
            providers: raw.clone(),
            models: json!([{"id":"vendor/a:8b","provider":NEW,"name":"Krill/vendor/a:8b"}]),
            order: json!({}),
            catalog: json!({}),
        };
        let bound =
            execution_directory::bind(&self.root, &self.context(), &upstream, agent).unwrap();
        assert!(bound.connection_digest.is_some());
        assert_eq!(
            read_value(
                &self
                    .root
                    .join("manager-state/model-execution-source-ids.json")
            )[NEW],
            self.id
        );
        assert!(!self
            .root
            .join("manager-state/provider-rename-pending.json")
            .exists());
    }
}
fn hook(f: impl FnMut(&str) -> ModelResult<()> + 'static) {
    channel_rename::TEST_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(f)));
}
fn clear() {
    channel_rename::TEST_HOOK.with(|h| *h.borrow_mut() = None);
}
fn pass(case: &str) {
    println!("RENAME_PRODUCTION_PASS {case}");
}

#[test]
fn production_provider_rename_transaction_in_memory_credential_process() {
    const CHILD: &str = "OCX_PRODUCTION_RENAME_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let status=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","modules::model_config::config_file::rename_tests::production_provider_rename_transaction_in_memory_credential_process","--nocapture","--test-threads=1"]).env(CHILD,"1").status().unwrap();
        assert!(status.success());
        return;
    }
    let memory = memory::Memory::default();
    memory.install();
    for scoped in [true, false] {
        memory.reset();
        let f = Fixture::new();
        let mut request = f.request(scoped);
        let original = memory.snapshot();
        let nodes = request
            .policy
            .channels
            .iter()
            .find(|c| c.id == f.id)
            .unwrap()
            .connection
            .nodes
            .clone();
        if scoped {
            let other = request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.connection.provider_id == "Krill-10")
                .unwrap();
            other.label = "ignored draft".into();
            other.connection.nodes.clear();
        }
        let result = save_with_context(&f.root, &f.context(), request).unwrap();
        assert!(result.changed);
        f.assert_saved();
        assert_eq!(
            result
                .snapshot
                .policy
                .channels
                .iter()
                .find(|c| c.id == f.id)
                .unwrap()
                .connection
                .nodes,
            nodes
        );
        assert_eq!(original.len(), memory.snapshot().len());
        pass(if scoped {
            "scoped-current-only"
        } else {
            "all-page"
        });
    }
    for stage in [
        "prepared",
        "credential:0",
        "credential:1",
        "credential:2",
        "before_file:0",
        "file:0",
        "file:1",
        "file:2",
        "file:3",
        "readback",
    ] {
        memory.reset();
        let f = Fixture::new();
        let request = f.request(true);
        let before = f.bytes();
        let keys = memory.snapshot();
        let point = stage.to_owned();
        hook(move |p| {
            if p == point {
                Err(invalid("fixture_failure", "injected"))
            } else {
                Ok(())
            }
        });
        let error = save_with_context(&f.root, &f.context(), request)
            .err()
            .unwrap();
        clear();
        assert_eq!(error.code, "fixture_failure", "{stage}");
        assert_eq!(f.bytes(), before, "{stage}");
        assert_eq!(memory.snapshot(), keys, "{stage}");
        pass(&format!("rollback-{stage}"));
    }
    for stage in [
        "credential:0",
        "file:0",
        "file:1",
        "file:3",
        "readback",
        "committed",
    ] {
        memory.reset();
        let f = Fixture::new();
        let request = f.request(true);
        let before = f.bytes();
        let keys = memory.snapshot();
        let point = stage.to_owned();
        hook(move |p| {
            if p == point {
                Err(invalid("rename_interrupted", "simulated process boundary"))
            } else {
                Ok(())
            }
        });
        let error = save_with_context(&f.root, &f.context(), request)
            .err()
            .unwrap();
        clear();
        assert_eq!(error.code, "rename_interrupted");
        assert!(f
            .root
            .join("manager-state/provider-rename-pending.json")
            .exists());
        snapshot(&f.root, &f.home).unwrap();
        if stage == "committed" {
            f.assert_saved();
        } else {
            assert_eq!(f.bytes(), before);
            assert_eq!(memory.snapshot(), keys);
        }
        pass(&format!("journal-recover-{stage}"));
    }
    for (name, offset) in [("source-read", 1), ("destination-put", 3)] {
        memory.reset();
        let f = Fixture::new();
        let request = f.request(true);
        let before = f.bytes();
        let keys = memory.snapshot();
        // Arm after transaction preparation; no platform credential is involved.
        let control = memory.clone();
        hook(move |point| {
            if point == "prepared" {
                control.fail_after(offset);
            }
            Ok(())
        });
        assert!(save_with_context(&f.root, &f.context(), request).is_err());
        clear();
        memory.clear_fault();
        snapshot(&f.root, &f.home).unwrap();
        assert_eq!(f.bytes(), before);
        assert_eq!(memory.snapshot(), keys);
        pass(&format!("credential-failure-{name}"));
    }
    for mode in [
        "occupied",
        "invalid",
        "reserved",
        "oauth",
        "forward",
        "derived",
        "foreign-reference",
        "occupied-account",
    ] {
        memory.reset();
        let f = Fixture::new();
        let mut request = f.request(true);
        let channel = request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == f.id)
            .unwrap();
        match mode {
            "occupied" => channel.connection.provider_id = "Krill-10".into(),
            "invalid" => channel.connection.provider_id = "bad/id".into(),
            "reserved" => channel.connection.provider_id = "openai".into(),
            "derived" => channel.connection.provider_id = GROUP.into(),
            "oauth" | "forward" => channel.connection.auth_mode = Some(mode.into()),
            "occupied-account" => put(
                &format!("{NEW}/{}", slot("fixture-a")),
                "fixture-target-occupied",
            ),
            _ => {
                let mut raw = f.raw.clone();
                raw["providers"][OLD]["apiKey"] = json!("keychain:Krill-10/foreign");
                write_value(&f.home.join("config.json"), &raw);
                request.expected_file = sha256(&std::fs::read(f.home.join("config.json")).unwrap());
            }
        }
        let before = f.bytes();
        let keys = memory.snapshot();
        assert!(save(&f.root, &f.home, request).is_err(), "{mode}");
        assert_eq!(f.bytes(), before);
        assert_eq!(memory.snapshot(), keys);
        pass(&format!("reject-{mode}"));
    }
    memory.reset();
    let f = Fixture::new();
    let request = f.request(true);
    let mut external = f.raw.clone();
    external["external"] = json!("newer");
    let path = f.home.join("config.json");
    let expected = external.clone();
    hook(move |stage| {
        if stage == "readback" {
            write_value(&path, &external);
            return Err(invalid("fixture_failure", "drift"));
        }
        Ok(())
    });
    let error = save(&f.root, &f.home, request).err().unwrap();
    clear();
    assert_eq!(error.code, "rename_recovery_required");
    assert_eq!(read_value(&f.home.join("config.json")), expected);
    assert!(get(&format!("{NEW}/{}", slot("fixture-a"))).is_some());
    pass("external-drift-not-overwritten");
    memory.reset();
    let f = Fixture::new();
    let request = f.request(true);
    let prefs = f.root.join("manager-state/preferences.json");
    let mut doc = read_value(&prefs);
    doc["futurePrefs"] = json!({"newer":true});
    let before_keys = memory.snapshot();
    let error = save_observed(&f.root, &f.home, request, move |_, _| {
        write_value(&prefs, &doc);
        Ok(())
    })
    .err()
    .unwrap();
    assert_eq!(error.code, "file_changed");
    assert_eq!(memory.snapshot(), before_keys);
    assert_eq!(
        read_value(&f.root.join("manager-state/preferences.json"))["futurePrefs"],
        json!({"newer":true})
    );
    pass("preview-preferences-drift");
    memory.reset();
    let f = Fixture::new();
    let request = f.request(true);
    let before = f.bytes();
    let keys = memory.snapshot();
    let error = save_observed(&f.root, &f.home, request, |_, _| {
        Err(invalid("cancelled", "cancelled"))
    })
    .err()
    .unwrap();
    assert_eq!(error.code, "cancelled");
    assert_eq!(f.bytes(), before);
    assert_eq!(memory.snapshot(), keys);
    pass("cancel-zero-business-write");
    memory.reset();
    let f = Fixture::new();
    let request = f.request(true);
    let control = memory.clone();
    hook(move |stage| {
        if stage == "cleanup" {
            control.fail_after(1);
        }
        Ok(())
    });
    let error = save_with_context(&f.root, &f.context(), request)
        .err()
        .unwrap();
    clear();
    memory.clear_fault();
    assert_eq!(error.code, "rename_cleanup_pending");
    assert!(read_value(&f.home.join("config.json"))["providers"]
        .get(NEW)
        .is_some());
    assert!(f
        .root
        .join("manager-state/provider-rename-pending.json")
        .exists());
    snapshot(&f.root, &f.home).unwrap();
    f.assert_saved();
    pass("committed-credential-cleanup-replay");
    // Renaming and ordinary rule/credential edits share the production mapper
    // and the same transaction, rather than a rename-only shadow save path.
    for action in ["keep", "replace", "clear"] {
        memory.reset();
        let f = Fixture::new();
        let mut request = f.request(true);
        let c = request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == f.id)
            .unwrap();
        c.overrides
            .entry("vendor/a:8b".into())
            .or_default()
            .context_window = Field::Custom(256000);
        if action != "keep" {
            request.credentials.push(CredentialEdit {
                channel_id: f.id.clone(),
                action: action.into(),
                value: (action == "replace").then(|| "fixture-replacement".into()),
            });
        }
        save_with_context(&f.root, &f.context(), request).unwrap();
        let raw = read_value(&f.home.join("config.json"));
        assert_eq!(
            raw["providers"][NEW]["modelContextWindows"]["vendor/a:8b"],
            256000
        );
        match action {
            "keep" => assert_eq!(credential(&f.home, NEW).unwrap().value, "fixture-b"),
            "replace" => {
                assert_eq!(
                    credential(&f.home, NEW).unwrap().value,
                    "fixture-replacement"
                );
                assert_eq!(
                    raw["providers"][NEW]["apiKeyPool"][1]["id"],
                    slot("fixture-replacement")
                );
            }
            _ => {
                assert!(raw["providers"][NEW].get("apiKey").is_none());
                assert!(raw["providers"][NEW].get("apiKeyPool").is_none());
            }
        }
        assert!(get(&format!("{OLD}/{}", slot("fixture-b"))).is_none());
        pass(&format!("compound-rule-and-key-{action}"));
    }
    memory.reset();
    let f = Fixture::new();
    save_with_context(&f.root, &f.context(), f.request(true)).unwrap();
    let source = snapshot(&f.root, &f.home).unwrap();
    let scope = crate::modules::model_config::transfer::ExportScope {
        channel_families: BTreeMap::new(),
        channel_models: BTreeMap::new(),
        include_policy: true,
        channel_ids: vec![f.id.clone()],
        template_refs: vec![],
        include_global: false,
    };
    let package = crate::modules::model_config::transfer::export_snapshot(&source, scope).unwrap();
    let wire = serde_json::to_value(&package).unwrap();
    assert!(wire["payload"]["policy"]["channels"][0]["connection"]
        .get("providerId")
        .is_none());
    let target = Fixture::new();
    let before = target.bytes();
    let local_keys = memory.snapshot();
    let incoming =
        crate::modules::model_config::transfer::receive_file(&target.root, package, &target.home)
            .unwrap();
    assert!(
        incoming.preview_error.is_none(),
        "{:?}",
        incoming.preview_error
    );
    let merged = incoming.merged_policy.as_ref().unwrap();
    assert_eq!(merged.channels.iter().filter(|c| c.id == f.id).count(), 1);
    assert_eq!(
        merged
            .channels
            .iter()
            .find(|c| c.id == f.id)
            .unwrap()
            .connection
            .provider_id,
        OLD
    );
    assert_eq!(target.bytes(), before);
    assert_eq!(memory.snapshot(), local_keys);
    pass("portable-import-stable-object-keeps-local-provider-and-keys");
    #[cfg(unix)]
    {
        memory.reset();
        let f = Fixture::new();
        let request = f.request(true);
        let before = f.bytes();
        let keys = memory.snapshot();
        let path = f.home.clone();
        hook(move |stage| {
            if stage == "before_file:0" {
                crate::infrastructure::platform::set_mode(&path, 0o500)?;
            }
            Ok(())
        });
        assert!(save(&f.root, &f.home, request).is_err());
        clear();
        crate::infrastructure::platform::set_mode(&f.home, 0o700).unwrap();
        assert_eq!(f.bytes(), before);
        assert_eq!(memory.snapshot(), keys);
        pass("actual-file-write-permission-failure");
    }
}

mod memory {
    use super::*;
    use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi};
    use std::{
        any::Any,
        sync::{Arc, Mutex},
    };
    #[derive(Default)]
    struct State {
        values: BTreeMap<(String, String), Vec<u8>>,
        operations: usize,
        fail: Option<usize>,
    }
    #[derive(Clone, Default)]
    pub struct Memory(Arc<Mutex<State>>);
    struct Builder(Memory);
    struct Entry {
        memory: Memory,
        id: (String, String),
    }
    impl Memory {
        pub fn install(&self) {
            keyring::set_default_credential_builder(Box::new(Builder(self.clone())));
        }
        pub fn reset(&self) {
            *self.0.lock().unwrap() = State::default();
        }
        pub fn snapshot(&self) -> BTreeMap<(String, String), Vec<u8>> {
            self.0.lock().unwrap().values.clone()
        }
        pub fn fail_after(&self, n: usize) {
            let mut s = self.0.lock().unwrap();
            s.fail = Some(s.operations + n);
        }
        pub fn clear_fault(&self) {
            self.0.lock().unwrap().fail = None;
        }
    }
    fn step(s: &mut State) -> keyring::Result<()> {
        s.operations += 1;
        if s.fail == Some(s.operations) {
            s.fail = None;
            Err(keyring::Error::PlatformFailure(Box::new(
                std::io::Error::other("fixture credential failure"),
            )))
        } else {
            Ok(())
        }
    }
    impl CredentialBuilderApi for Builder {
        fn build(
            &self,
            _: Option<&str>,
            service: &str,
            user: &str,
        ) -> keyring::Result<Box<Credential>> {
            Ok(Box::new(Entry {
                memory: self.0.clone(),
                id: (service.into(), user.into()),
            }))
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    impl CredentialApi for Entry {
        fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
            let mut s = self.memory.0.lock().unwrap();
            step(&mut s)?;
            s.values.insert(self.id.clone(), value.into());
            Ok(())
        }
        fn get_secret(&self) -> keyring::Result<Vec<u8>> {
            let mut s = self.memory.0.lock().unwrap();
            step(&mut s)?;
            s.values
                .get(&self.id)
                .cloned()
                .ok_or(keyring::Error::NoEntry)
        }
        fn delete_credential(&self) -> keyring::Result<()> {
            let mut s = self.memory.0.lock().unwrap();
            step(&mut s)?;
            s.values
                .remove(&self.id)
                .map(|_| ())
                .ok_or(keyring::Error::NoEntry)
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

#[test]
fn process_exit_recovers_before_first_snapshot() {
    const ENV: &str = "OCX_RENAME_EXIT_MANIFEST";
    if let Some(marker) = std::env::var_os(ENV) {
        let memory = memory::Memory::default();
        memory.install();
        let f = Fixture::new();
        // No secret must survive this child: this case tests the actual disk
        // journal/process boundary, while the other suite tests credential IO.
        let mut raw = f.raw.clone();
        raw["providers"][OLD]["apiKey"] = json!("fixture-inline");
        raw["providers"][OLD]
            .as_object_mut()
            .unwrap()
            .remove("apiKeyPool");
        raw["providers"][OLD]
            .as_object_mut()
            .unwrap()
            .remove("headers");
        write_value(&f.home.join("config.json"), &raw);
        let request = f.request(true);
        write_value(
            Path::new(&marker),
            &json!({"root":f.root,"home":f.home,"config":raw,"layout":read_value(&f.root.join(METADATA)),"preferences":read_value(&f.root.join("manager-state/preferences.json"))}),
        );
        hook(|stage| {
            if stage == "file:1" {
                std::process::exit(86);
            }
            Ok(())
        });
        let _ = save(&f.root, &f.home, request);
        panic!("child must exit in production transaction");
    }
    let owner = tempfile::tempdir().unwrap();
    let marker = owner.path().join("manifest.json");
    let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","modules::model_config::config_file::rename_tests::process_exit_recovers_before_first_snapshot","--nocapture","--test-threads=1"]).env(ENV,&marker).status().unwrap();
    assert_eq!(result.code(), Some(86));
    let manifest = read_value(&marker);
    let root = PathBuf::from(manifest["root"].as_str().unwrap());
    let home = PathBuf::from(manifest["home"].as_str().unwrap());
    assert!(root.starts_with(std::env::temp_dir()));
    assert!(home.starts_with(std::env::temp_dir()));
    assert!(read_value(&home.join("config.json"))["providers"]
        .get(NEW)
        .is_some());
    let actual = snapshot(&root, &home).unwrap();
    assert!(actual
        .policy
        .channels
        .iter()
        .any(|c| c.connection.provider_id == OLD));
    assert_eq!(read_value(&home.join("config.json")), manifest["config"]);
    assert_eq!(read_value(&root.join(METADATA)), manifest["layout"]);
    assert_eq!(
        read_value(&root.join("manager-state/preferences.json")),
        manifest["preferences"]
    );
    assert!(!root
        .join("manager-state/provider-rename-pending.json")
        .exists());
    pass("actual-process-exit-recovery");
    std::fs::remove_dir_all(root.parent().unwrap()).unwrap();
}
