//! Actual injected data-root/home/client files and a mock vault. No daily
//! profile, keyring, client writer, network or executable MCP is used.
use super::*;
use crate::modules::extensions::ProjectionFingerprint;
use serde_json::json;
use std::{cell::RefCell, fs};
const NAME: &str = "fixture";
const REF: &str = "11111111-1111-4111-8111-111111111111";
const OTHER_REF: &str = "22222222-2222-4222-8222-222222222222";
struct Vault {
    values: BTreeMap<String, String>,
    reads: RefCell<Vec<String>>,
}
impl Vault {
    fn new() -> Self {
        Self {
            values: BTreeMap::from([
                (REF.into(), "managed-synthetic-secret".into()),
                (OTHER_REF.into(), "unselected-secret".into()),
            ]),
            reads: RefCell::new(vec![]),
        }
    }
}
impl Credentials for Vault {
    fn read(&self, id: &str) -> ModelResult<String> {
        self.reads.borrow_mut().push(id.into());
        self.values
            .get(id)
            .cloned()
            .ok_or_else(|| invalid("extension_credential", "fixture missing"))
    }
    fn install(&self, _: &str, _: &str) -> ModelResult<()> {
        panic!("export must never write credentials")
    }
}
struct Fixture {
    root: tempfile::TempDir,
    home: tempfile::TempDir,
    vault: Vault,
}
impl Fixture {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
            home: tempfile::tempdir().unwrap(),
            vault: Vault::new(),
        }
    }
    fn managed(&self, definition: Value, secret: bool) {
        fs::create_dir_all(self.root.path().join("manager-state")).unwrap();
        let stored = StoredMcp {
            name: NAME.into(),
            definition,
            secret_fields: if secret {
                vec!["/env/TOKEN".into()]
            } else {
                vec![]
            },
            references: if secret {
                BTreeMap::from([("/env/TOKEN".into(), REF.into())])
            } else {
                BTreeMap::new()
            },
        };
        fs::write(
            self.root.path().join(MCP_STORE),
            serde_json::to_vec(&vec![stored]).unwrap(),
        )
        .unwrap();
    }
    fn client(&self, client: &str, definition: Value) {
        let (path, bytes) = match client {
            "codex" => (
                self.home.path().join(".codex/config.toml"),
                toml::to_string(&json!({"mcp_servers":{NAME:definition}}))
                    .unwrap()
                    .into_bytes(),
            ),
            "hermes" => (
                self.home.path().join(".hermes/config.yaml"),
                serde_yaml::to_string(&json!({"mcp_servers":{NAME:definition}}))
                    .unwrap()
                    .into_bytes(),
            ),
            _ => panic!("unknown fixture client"),
        };
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    fn export(
        &self,
        source: Option<&str>,
        include: bool,
        ids: Option<&[String]>,
    ) -> ModelResult<(Assets, Vec<McpConflict>)> {
        let sources = source
            .map(|s| BTreeMap::from([(NAME.into(), s.into())]))
            .unwrap_or_default();
        export_with_credential_scope_using(
            self.root.path(),
            self.home.path(),
            &[],
            &[NAME.into()],
            &sources,
            include,
            true,
            ids,
            &self.vault,
        )
    }
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(p: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for e in fs::read_dir(p).unwrap() {
                let e = e.unwrap();
                if e.path().is_dir() {
                    walk(&e.path(), out)
                } else {
                    out.insert(e.path(), fs::read(e.path()).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(self.root.path(), &mut out);
        walk(self.home.path(), &mut out);
        out
    }
}
#[test]
fn sources_v2_explicit_client_wins_over_stored_and_never_mutates_sources() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), false);
    f.client(
        "codex",
        json!({"command":"client","env":{"TOKEN":"client-secret"}}),
    );
    let before = f.snapshot();
    let (assets, conflicts) = f.export(Some("codex"), true, None).unwrap();
    assert!(conflicts.is_empty());
    assert_eq!(assets.mcps[0].definition["command"], "client");
    assert_eq!(assets.mcps[0].secrets.0["/env/TOKEN"], "client-secret");
    assert!(f.vault.reads.borrow().is_empty());
    assert_eq!(before, f.snapshot());
    let paths = saved_source_paths(
        f.root.path(),
        f.home.path(),
        &[],
        &[NAME.into()],
        &BTreeMap::from([(NAME.into(), "codex".into())]),
    )
    .unwrap();
    assert_eq!(paths, vec![f.home.path().join(".codex/config.toml")]);
    let details = detail(f.root.path(), f.home.path(), "mcps", NAME, Some("codex")).unwrap();
    assert_eq!(details.clients, vec!["codex"]);
    assert_ne!(details.source, "受管定义");
}
#[test]
fn sources_v2_explicit_managed_uses_only_its_vault_and_does_not_borrow_client_secret() {
    let f = Fixture::new();
    f.managed(json!({"command":"same"}), true);
    f.client(
        "codex",
        json!({"command":"same","env":{"TOKEN":"client-secret"}}),
    );
    let before = f.snapshot();
    let (assets, conflicts) = f.export(Some("managed"), true, None).unwrap();
    assert!(conflicts.is_empty());
    assert_eq!(
        assets.mcps[0].secrets.0["/env/TOKEN"],
        "managed-synthetic-secret"
    );
    assert_eq!(*f.vault.reads.borrow(), vec![REF]);
    assert_eq!(before, f.snapshot());
    let paths = saved_source_paths(
        f.root.path(),
        f.home.path(),
        &[],
        &[NAME.into()],
        &BTreeMap::from([(NAME.into(), "managed".into())]),
    )
    .unwrap();
    assert_eq!(paths, vec![f.root.path().join(MCP_STORE)]);
    let details = detail(f.root.path(), f.home.path(), "mcps", NAME, Some("managed")).unwrap();
    assert_eq!(details.source, "受管定义");
    assert_eq!(details.clients, vec!["managed"]);
}
#[test]
fn sources_v2_unselected_managed_secret_is_not_read_to_compare_public_candidates() {
    let f = Fixture::new();
    f.managed(json!({"command":"same"}), true);
    f.client(
        "codex",
        json!({"command":"same","env":{"TOKEN":"client-secret"}}),
    );
    let before = f.snapshot();
    for include in [false, true] {
        let (assets, conflicts) = f.export(None, include, None).unwrap();
        assert!(assets.mcps.is_empty());
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].clients.contains(&"managed".into()));
        assert!(conflicts[0].clients.contains(&"codex".into()));
        assert_eq!(conflicts[0].fields, vec!["credentials", "source"]);
    }
    assert!(f.vault.reads.borrow().is_empty());
    assert_eq!(before, f.snapshot());
    let public = serde_json::to_string(&f.export(None, false, None).unwrap().1).unwrap();
    assert!(!public.contains("client-secret"));
    assert!(!public.contains("managed-synthetic-secret"));
}
#[test]
fn sources_v2_same_public_without_any_secret_is_proven_equivalent_with_managed() {
    let f = Fixture::new();
    f.managed(json!({"command":"same","args":["--fixture"]}), false);
    f.client("codex", json!({"command":"same","args":["--fixture"]}));
    f.client("hermes", json!({"command":"same","args":["--fixture"]}));
    let before = f.snapshot();
    let (assets, conflicts) = f.export(None, true, None).unwrap();
    assert!(conflicts.is_empty());
    assert_eq!(assets.mcps.len(), 1);
    assert_eq!(assets.mcps[0].definition["command"], "same");
    assert!(assets.mcps[0].secrets.0.is_empty());
    assert!(f.vault.reads.borrow().is_empty());
    assert_eq!(before, f.snapshot());
}
#[test]
fn sources_v2_stored_and_client_public_difference_reports_actual_fields_and_all_sources() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), false);
    f.client("codex", json!({"command":"codex"}));
    f.client("hermes", json!({"command":"hermes"}));
    let (assets, conflicts) = f.export(None, false, None).unwrap();
    assert!(assets.mcps.is_empty());
    assert_eq!(conflicts[0].fields, vec!["command"]);
    assert_eq!(conflicts[0].clients, vec!["managed", "codex", "hermes"]);
    let details = detail(f.root.path(), f.home.path(), "mcps", NAME, None).unwrap();
    assert_eq!(details.clients, conflicts[0].clients);
    assert_eq!(details.conflicts, conflicts[0].fields);
    let paths = saved_source_paths(
        f.root.path(),
        f.home.path(),
        &[],
        &[NAME.into()],
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(paths.len(), 3);
    assert!(paths.contains(&f.root.path().join(MCP_STORE)));
}
#[test]
fn sources_v2_explicit_source_missing_or_unknown_never_falls_back() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), false);
    f.client("codex", json!({"command":"client"}));
    for choice in ["missing-client", "gemini"] {
        assert_eq!(
            f.export(Some(choice), false, None).err().unwrap().code,
            "mcp_source"
        );
    }
    fs::remove_file(f.home.path().join(".codex/config.toml")).unwrap();
    assert_eq!(
        f.export(Some("codex"), true, None).err().unwrap().code,
        "mcp_source"
    );
    f.client("codex", json!({"command":"client"}));
    fs::remove_file(f.root.path().join(MCP_STORE)).unwrap();
    assert_eq!(
        f.export(Some("managed"), true, None).err().unwrap().code,
        "mcp_source"
    );
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn sources_v2_secret_scope_false_or_not_selected_never_reads_managed_vault() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), true);
    let (assets, _) = f.export(Some("managed"), false, None).unwrap();
    assert!(assets.mcps[0].secrets.0.is_empty());
    assert_eq!(assets.mcps[0].secret_fields, vec!["/env/TOKEN"]);
    let (assets, _) = f.export(Some("managed"), true, Some(&[])).unwrap();
    assert!(assets.mcps[0].secrets.0.is_empty());
    assert!(f.vault.reads.borrow().is_empty());
    let (assets, _) = f
        .export(Some("managed"), true, Some(&[format!("mcp:{NAME}")]))
        .unwrap();
    assert_eq!(assets.mcps[0].secrets.0.len(), 1);
    assert_eq!(*f.vault.reads.borrow(), vec![REF]);
}
#[test]
fn sources_v2_scope_is_revalidated_before_reads_or_fallback() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), true);
    let err = export_with_credential_scope_using(
        f.root.path(),
        f.home.path(),
        &[],
        &[NAME.into()],
        &BTreeMap::from([("unselected".into(), "managed".into())]),
        true,
        true,
        None,
        &f.vault,
    )
    .err()
    .unwrap();
    assert_eq!(err.code, "mcp_source");
    let (withheld, _) = f
        .export(
            Some("managed"),
            true,
            Some(&["mcp:unselected".into(), format!("channel:{REF}")]),
        )
        .unwrap();
    assert!(withheld.mcps[0].secrets.0.is_empty());
    assert_eq!(
        f.export(
            Some("managed"),
            true,
            Some(&[format!("mcp:{NAME}"), format!("mcp:{NAME}")])
        )
        .err()
        .unwrap()
        .code,
        "credential_scope"
    );
    let err = export_with_credential_scope_using(
        f.root.path(),
        f.home.path(),
        &[],
        &[NAME.into(), NAME.into()],
        &BTreeMap::new(),
        true,
        true,
        None,
        &f.vault,
    )
    .err()
    .unwrap();
    assert_eq!(err.code, "asset_identity");
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn sources_v2_client_secret_difference_requires_selection_without_vault_or_value_leak() {
    let f = Fixture::new();
    f.client(
        "codex",
        json!({"command":"same","env":{"TOKEN":"codex-secret"}}),
    );
    f.client(
        "hermes",
        json!({"command":"same","env":{"TOKEN":"hermes-secret"}}),
    );
    let (assets, conflicts) = f.export(None, false, None).unwrap();
    assert!(assets.mcps.is_empty());
    assert_eq!(conflicts[0].fields, vec!["credentials", "source"]);
    f.client(
        "hermes",
        json!({"command":"same","env":{"TOKEN":"codex-secret"}}),
    );
    let (assets, conflicts) = f.export(None, true, None).unwrap();
    assert!(conflicts.is_empty());
    assert_eq!(assets.mcps[0].secrets.0["/env/TOKEN"], "codex-secret");
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn sources_v2_managed_missing_own_reference_does_not_borrow_identical_client_credentials() {
    let f = Fixture::new();
    f.managed(json!({"command":"same"}), true);
    f.client(
        "codex",
        json!({"command":"same","env":{"TOKEN":"client-secret"}}),
    );
    let mut stored = mcp_store(f.root.path()).unwrap();
    stored[0].references.clear();
    fs::write(
        f.root.path().join(MCP_STORE),
        serde_json::to_vec(&stored).unwrap(),
    )
    .unwrap();
    assert_eq!(
        f.export(Some("managed"), true, None).err().unwrap().code,
        "extension_credential"
    );
    assert!(f.vault.reads.borrow().is_empty());
    let (assets, _) = f.export(Some("codex"), true, None).unwrap();
    assert_eq!(assets.mcps[0].secrets.0["/env/TOKEN"], "client-secret");
}
#[test]
fn sources_v2_explicit_source_is_not_blocked_by_unselected_unreadable_definitions() {
    let f = Fixture::new();
    f.managed(json!({"command":"managed"}), false);
    f.client("codex", json!({"command":"client"}));
    fs::create_dir_all(f.home.path().join(".hermes")).unwrap();
    fs::write(f.home.path().join(".hermes/config.yaml"), b"{ invalid yaml").unwrap();
    let (assets, _) = f.export(Some("codex"), false, None).unwrap();
    assert_eq!(assets.mcps[0].definition["command"], "client");
    let (assets, _) = f.export(Some("managed"), false, None).unwrap();
    assert_eq!(assets.mcps[0].definition["command"], "managed");
    assert_eq!(f.export(None, false, None).err().unwrap().code, "mcp_read");
    fs::write(f.root.path().join(MCP_STORE), b"bad managed json").unwrap();
    assert!(f.export(Some("codex"), false, None).is_ok());
}
#[test]
fn sources_v2_actual_manager_source_config_is_checked_without_touching_client_files() {
    let f = Fixture::new();
    f.client("codex", json!({"command":"client"}));
    fs::create_dir_all(f.root.path().join("manager-state")).unwrap();
    let config = extensions::ExtensionConfig::default();
    let payload = ExtensionProjection {
        fingerprint: ProjectionFingerprint::create(config.fingerprint(), chrono::Utc::now()),
        config,
    };
    fs::write(
        f.root.path().join(projection::CONFIG_RELATIVE_PATH),
        serde_json::to_vec(&payload).unwrap(),
    )
    .unwrap();
    let before = f.snapshot();
    assert!(f.export(Some("codex"), false, None).is_ok());
    assert_eq!(before, f.snapshot());
    let mut payload = payload;
    payload.fingerprint.fingerprint = "wrong".into();
    fs::write(
        f.root.path().join(projection::CONFIG_RELATIVE_PATH),
        serde_json::to_vec(&payload).unwrap(),
    )
    .unwrap();
    assert_eq!(
        f.export(Some("codex"), false, None).err().unwrap().code,
        "asset_config"
    );
}
