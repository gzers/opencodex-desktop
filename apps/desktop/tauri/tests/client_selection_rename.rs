//! Tests link the registered production module through the application library.
use opencodex_desktop_lib::modules::model_config::cache_refresh::client_selection_rename::{
    preflight_current_selection, ClientSelectionRename, ClientSelectionRenamePlan,
};
use opencodex_desktop_lib::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::model_config::schema::ModelResult, state::ProcessContext,
};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

const OLD: &str = "Krill-1/Org/RaW-模型/Version@2026";
const NEW: &str = "Krill/Org/RaW-模型/Version@2026";
const REVIEW_OLD: &str = "Krill-1/Review/Original-β/v2";
const REVIEW_NEW: &str = "Krill/Review/Original-β/v2";
const BASE: &str = "cli_auth_credentials_store = \"file\"\nweb_search = \"disabled\"\ncheck_for_update_on_startup = false\n[analytics]\nenabled = false\n[feedback]\nenabled = false\n";

struct Fixture {
    _root: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    codex: PathBuf,
    work: PathBuf,
    context: ProcessContext,
}

impl Fixture {
    fn new(model: &str) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("selection-test-")
            .tempdir()
            .unwrap();
        let root = dunce::canonicalize(temp.path()).unwrap();
        let home = root.join("home");
        let codex = home.join("client-explicit");
        let work = root.join("work");
        let manager = root.join("manager");
        for dir in [&codex, &work, &manager] {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(
            codex.join("config.toml"),
            format!("# keep this comment\nmodel = {}\n{BASE}", json!(model)),
        )
        .unwrap();
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::unresolved(),
            working_directory: work.clone(),
            opencodex_home: manager,
        };
        Self {
            _root: temp,
            root,
            home,
            codex,
            work,
            context,
        }
    }

    async fn plan(
        &self,
        executable: &Path,
        profile: Option<&str>,
    ) -> ModelResult<ClientSelectionRenamePlan> {
        preflight_current_selection(
            &self.context,
            executable,
            &self.home,
            &self.codex,
            &self.work,
            profile,
            &BTreeMap::from([("Krill-1".into(), "Krill".into())]),
        )
        .await
    }

    fn bytes(&self) -> Vec<u8> {
        std::fs::read(self.codex.join("config.toml")).unwrap()
    }

    fn model(&self) -> String {
        let config: toml::Value =
            toml::from_str(std::str::from_utf8(&self.bytes()).unwrap()).unwrap();
        config["model"].as_str().unwrap().to_owned()
    }

    fn seed_current(&self, model: Option<&str>, review: Option<&str>) {
        let mut config = String::from("# keep this comment\n");
        for (key, value) in [("model", model), ("review_model", review)] {
            if let Some(value) = value {
                config.push_str(&format!("{key} = {}\n", json!(value)));
            }
        }
        config.push_str(BASE);
        std::fs::write(self.codex.join("config.toml"), config).unwrap();
    }

    fn review_model(&self) -> Option<String> {
        let config: toml::Value =
            toml::from_str(std::str::from_utf8(&self.bytes()).unwrap()).unwrap();
        config
            .get("review_model")
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    }

    #[cfg(unix)]
    fn fake(&self, behavior: &str, source: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(
            self.codex.join("fixture.json"),
            json!({"behavior": behavior, "source": source}).to_string(),
        )
        .unwrap();
        let executable = self.root.join("fake-owner");
        // Bypass macOS's xcrun Python shim, which injects SDK/build variables.
        #[cfg(target_os = "macos")]
        let script = FAKE_OWNER.replacen(
            "#!/usr/bin/python3",
            "#!/Library/Developer/CommandLineTools/usr/bin/python3",
            1,
        );
        #[cfg(not(target_os = "macos"))]
        let script = FAKE_OWNER;
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        executable
    }

    #[cfg(unix)]
    fn requests(&self) -> Vec<Value> {
        std::fs::read_to_string(self.codex.join("requests.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

#[cfg(unix)]
const FAKE_OWNER: &str = r##"#!/usr/bin/python3
import os,sys,json,hashlib,re,time,subprocess
from pathlib import Path
home=Path(os.environ['CODEX_HOME'])
allowed={'HOME','CODEX_HOME','CODEX_SQLITE_HOME','TMPDIR','TMP','TEMP','PATH','LANG','LC_CTYPE','__CF_USER_TEXT_ENCODING'}
assert not set(os.environ)-allowed, 'unexpected inherited environment'
assert Path(os.environ['HOME']).is_absolute()
assert all(Path(os.environ[k]).is_absolute() for k in ['TMPDIR','CODEX_HOME','CODEX_SQLITE_HOME'])
assert os.environ['PATH']=='/usr/bin:/bin:/usr/sbin:/sbin'
options=json.loads((home/'fixture.json').read_text())
behavior=options['behavior']
profile=sys.argv[sys.argv.index('--profile')+1] if '--profile' in sys.argv else None
file=home/((profile+'.config.toml') if profile else 'config.toml')
count_file=home/'write-count'
def count():return int(count_file.read_text()) if count_file.exists() else 0
def version():return 'sha256:'+hashlib.sha256(file.read_bytes()).hexdigest()
def scalar(key):
 match=re.search(r'^'+key+r'\s*=\s*(".*")',file.read_text(),re.M)
 return json.loads(match.group(1)) if match else None
def set_values(edits):
 text=file.read_text()
 for edit in edits:
  key=edit['keyPath'];value=edit['value']
  text=re.sub(r'^'+key+r'\s*=.*$',lambda _:key+' = '+json.dumps(value,ensure_ascii=False),text,count=1,flags=re.M)
 file.write_text(text)
def reply(ident,result=None,error=None):
 row={'id':ident}
 row['error' if error else 'result']=error if error else result
 print(json.dumps(row,ensure_ascii=False),flush=True)
if behavior in ['timeout','stdout_limit','stderr_limit']:
 child=subprocess.Popen(['/bin/sleep','60'])
 (home/'child-pid').write_text(str(child.pid))
 if behavior=='stdout_limit':sys.stdout.write('x'*(2*1024*1024+1));sys.stdout.flush()
 if behavior=='stderr_limit':sys.stderr.write('x'*(256*1024+1));sys.stderr.flush()
 time.sleep(60)
for line in sys.stdin:
 request=json.loads(line)
 with (home/'requests.jsonl').open('a') as log:log.write(json.dumps(request)+'\n')
 method=request['method'];ident=request.get('id');params=request.get('params',{})
 if method=='initialized':continue
 if method=='initialize':reply(ident,{'userAgent':'fixture'});continue
 if method=='config/read':
  if behavior=='readback_error' and count()==1:
   reply(ident,error={'code':-32000,'message':'DO-NOT-EXPOSE fixture-secret'});continue
  current={key:scalar(key) for key in ['model','review_model']}
  if behavior=='readback_mismatch' and count()==1:current['model']='Elsewhere/fixture'
  if behavior=='review_readback_mismatch' and count()==1:current['review_model']='Elsewhere/review'
  source={'type':options['source'],'file':str(file),'profile':profile}
  if options['source']=='foreign':source={'type':'user','file':str(home/'foreign.toml'),'profile':None}
  layer={'name':source,'version':version(),'config':dict(current),'disabledReason':None}
  if behavior=='disabled_layer':layer['disabledReason']='disabled'
  if behavior=='layer_mismatch':layer['version']='different'
  origins={key:{'name':dict(source),'version':version()} for key,value in current.items() if value is not None}
  layers=[layer]
  if current['review_model'] is not None:
   if behavior in ['review_project','review_system','review_profile']:
    review_source=dict(source)
    if behavior=='review_profile':review_source.update({'profile':'other','file':str(home/'other.config.toml')})
    else:review_source['type']='project' if behavior=='review_project' else 'system'
    origins['review_model']['name']=review_source
    layers.append({'name':review_source,'version':version(),'config':{'review_model':current['review_model']}})
   if behavior=='review_version_mismatch':origins['review_model']['version']='different'
   if behavior=='review_origin_missing':origins.pop('review_model',None)
  if behavior=='mixed_profile_owner':
   root=home/'config.toml'
   main=json.loads(re.search(r'^model\s*=\s*(".*")',root.read_text(),re.M).group(1))
   current['model']=main
   root_source={'type':'user','file':str(root),'profile':None}
   root_version='sha256:'+hashlib.sha256(root.read_bytes()).hexdigest()
   origins['model']={'name':root_source,'version':root_version}
   layers.append({'name':root_source,'version':root_version,'config':{'model':main}})
  if behavior=='all_origins_missing':origins={}
  selected_profile='active' if behavior=='unbound_profile' else None
  reply(ident,{'config':dict(current,profile=selected_profile),'origins':origins,'layers':layers});continue
 assert method in ['config/value/write','config/batchWrite'],method
 assert params['filePath']==str(file)
 edits=params['edits'] if method=='config/batchWrite' else [params]
 assert all(edit['keyPath'] in ['model','review_model'] and edit['mergeStrategy']=='replace' for edit in edits)
 assert params['expectedVersion']
 if behavior=='cas_drift' and count()==0:
  file.write_text(file.read_text()+'# external-before-cas\n')
 if behavior=='write_error' or params['expectedVersion']!=version():
  reply(ident,error={'code':-32000,'message':'DO-NOT-EXPOSE fixture-secret'});continue
 set_values(edits);writes=count()+1;count_file.write_text(str(writes))
 receipt={'filePath':str(file),'status':'ok','version':version(),'overriddenMetadata':None}
 if behavior in ['overridden','metadata_override'] and writes==1:
  if behavior=='overridden':receipt['status']='okOverridden'
  receipt['overriddenMetadata']={'effectiveValue':'Elsewhere/fixture','message':'DO-NOT-EXPOSE fixture-secret'}
 if behavior=='external_after_write' and writes==1:
  set_values([{'keyPath':'model','value':'External/new-choice'}]);file.write_text(file.read_text()+'# external-after-write\n')
 reply(ident,receipt)
"##;

#[tokio::test]
async fn empty_mapping_does_not_spawn_or_resolve_any_path() {
    let fixture = Fixture::new(OLD);
    let plan = preflight_current_selection(
        &fixture.context,
        Path::new("relative"),
        Path::new("relative"),
        Path::new("relative"),
        Path::new("relative"),
        None,
        &BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        plan.apply().await.unwrap(),
        ClientSelectionRename::Unchanged
    );
}

#[tokio::test]
async fn absolute_batch_wrapper_and_relative_identity_are_rejected_before_spawn() {
    let fixture = Fixture::new(OLD);
    let wrapper = fixture.root.join("codex.cmd");
    std::fs::write(&wrapper, "fixture-must-never-run").unwrap();
    let before = fixture.bytes();
    assert_eq!(
        fixture.plan(&wrapper, None).await.err().unwrap().code,
        "client_selection_executable"
    );
    assert_eq!(
        fixture
            .plan(Path::new("relative-codex"), None)
            .await
            .err()
            .unwrap()
            .code,
        "client_selection_paths"
    );
    assert_eq!(fixture.bytes(), before);
}

#[cfg(unix)]
#[tokio::test]
async fn preflight_is_read_only_and_apply_uses_latest_injection_version() {
    let fixture = Fixture::new(OLD);
    let owner = fixture.fake("normal", "user");
    let original = fixture.bytes();
    let plan = fixture.plan(&owner, None).await.unwrap();
    assert_eq!(fixture.bytes(), original);
    std::fs::write(
        fixture.codex.join("config.toml"),
        format!(
            "openai_base_url = \"http://fixture.invalid/v1\"\n{}",
            String::from_utf8(original).unwrap()
        ),
    )
    .unwrap();
    assert_eq!(plan.apply().await.unwrap(), ClientSelectionRename::Updated);
    assert_eq!(fixture.model(), NEW);
    let writes: Vec<_> = fixture
        .requests()
        .into_iter()
        .filter(|row| row["method"] == "config/value/write")
        .collect();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0]["params"]["value"], NEW);
    assert!(String::from_utf8(fixture.bytes())
        .unwrap()
        .contains("http://fixture.invalid/v1"));
}

#[cfg(unix)]
#[tokio::test]
async fn both_current_scalars_use_one_batch_and_the_latest_owner_version() {
    use sha2::{Digest, Sha256};
    let fixture = Fixture::new(OLD);
    fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
    let owner = fixture.fake("normal", "user");
    let before = fixture.bytes();
    let plan = fixture.plan(&owner, None).await.unwrap();
    assert_eq!(fixture.bytes(), before);
    let injected = format!(
        "future_model = {}\nopenai_base_url = \"http://fixture.invalid/v1\"\n{}",
        json!(OLD),
        String::from_utf8(before).unwrap()
    );
    let expected_version = format!("sha256:{:x}", Sha256::digest(injected.as_bytes()));
    std::fs::write(fixture.codex.join("config.toml"), &injected).unwrap();
    assert_eq!(plan.apply().await.unwrap(), ClientSelectionRename::Updated);
    assert_eq!(fixture.model(), NEW);
    assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_NEW));
    let writes: Vec<_> = fixture
        .requests()
        .into_iter()
        .filter(|row| row["method"] == "config/batchWrite")
        .collect();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0]["params"]["expectedVersion"], expected_version);
    assert_eq!(writes[0]["params"]["reloadUserConfig"], false);
    assert_eq!(
        writes[0]["params"]["edits"],
        json!([
            {"keyPath":"model","mergeStrategy":"replace","value":NEW},
            {"keyPath":"review_model","mergeStrategy":"replace","value":REVIEW_NEW}
        ])
    );
    assert!(!fixture
        .requests()
        .iter()
        .any(|row| row["method"] == "config/value/write"));
    assert!(String::from_utf8(fixture.bytes())
        .unwrap()
        .contains(&format!("future_model = {}", json!(OLD))));
}

#[cfg(unix)]
#[tokio::test]
async fn review_only_update_does_not_create_or_modify_the_main_selection() {
    for model in [None, Some("Other/keep")] {
        let fixture = Fixture::new("unused");
        fixture.seed_current(model, Some(REVIEW_OLD));
        let owner = fixture.fake("normal", "user");
        assert_eq!(
            fixture
                .plan(&owner, None)
                .await
                .unwrap()
                .apply()
                .await
                .unwrap(),
            ClientSelectionRename::Updated
        );
        assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_NEW));
        let parsed: toml::Value =
            toml::from_str(std::str::from_utf8(&fixture.bytes()).unwrap()).unwrap();
        assert_eq!(parsed.get("model").and_then(toml::Value::as_str), model);
        let writes: Vec<_> = fixture
            .requests()
            .into_iter()
            .filter(|row| row["method"] == "config/value/write")
            .collect();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0]["params"]["keyPath"], "review_model");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn unset_or_missing_origin_fields_are_noops_and_never_invented() {
    for behavior in ["normal", "all_origins_missing"] {
        let fixture = Fixture::new("unused");
        if behavior == "normal" {
            fixture.seed_current(None, None);
        } else {
            fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        }
        let owner = fixture.fake(behavior, "user");
        let before = fixture.bytes();
        assert_eq!(
            fixture
                .plan(&owner, None)
                .await
                .unwrap()
                .apply()
                .await
                .unwrap(),
            ClientSelectionRename::Unchanged
        );
        assert_eq!(fixture.bytes(), before);
        assert!(!fixture.requests().iter().any(|row| row["method"]
            .as_str()
            .is_some_and(|method| method.contains("write") || method.contains("Write"))));
    }
    let fixture = Fixture::new(OLD);
    fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
    let owner = fixture.fake("review_origin_missing", "user");
    assert_eq!(
        fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap(),
        ClientSelectionRename::Updated
    );
    assert_eq!(fixture.model(), NEW);
    assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_OLD));
    let writes: Vec<_> = fixture
        .requests()
        .into_iter()
        .filter(|row| row["method"] == "config/value/write")
        .collect();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0]["params"]["keyPath"], "model");
}

#[cfg(unix)]
#[tokio::test]
async fn an_unsupported_review_origin_refuses_the_entire_plan_before_any_write() {
    for behavior in [
        "review_project",
        "review_system",
        "review_profile",
        "review_version_mismatch",
    ] {
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let owner = fixture.fake(behavior, "user");
        let before = fixture.bytes();
        assert_eq!(
            fixture.plan(&owner, None).await.err().unwrap().code,
            "client_selection_source",
            "{behavior}"
        );
        assert_eq!(fixture.bytes(), before);
        assert!(!fixture.requests().iter().any(
            |row| row["method"] == "config/value/write" || row["method"] == "config/batchWrite"
        ));
    }
}

#[cfg(unix)]
#[tokio::test]
async fn review_value_or_origin_drift_after_preflight_blocks_both_writes() {
    for source_drift in [false, true] {
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let owner = fixture.fake("normal", "user");
        let plan = fixture.plan(&owner, None).await.unwrap();
        if source_drift {
            fixture.fake("review_project", "user");
        } else {
            fixture.seed_current(Some(OLD), Some("External/new-review"));
        }
        let before = fixture.bytes();
        assert!(plan.apply().await.is_err());
        assert_eq!(fixture.bytes(), before);
        assert_eq!(fixture.model(), OLD);
        assert!(!fixture.requests().iter().any(
            |row| row["method"] == "config/value/write" || row["method"] == "config/batchWrite"
        ));
    }
}

#[cfg(unix)]
#[tokio::test]
async fn batch_verification_failure_compensates_both_original_scalars_in_one_cas() {
    for behavior in [
        "overridden",
        "metadata_override",
        "readback_error",
        "readback_mismatch",
        "review_readback_mismatch",
    ] {
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let owner = fixture.fake(behavior, "user");
        let error = fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap_err();
        assert_ne!(error.code, "client_selection_repair_conflict", "{behavior}");
        assert_eq!(fixture.model(), OLD);
        assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_OLD));
        let writes: Vec<_> = fixture
            .requests()
            .into_iter()
            .filter(|row| row["method"] == "config/batchWrite")
            .collect();
        assert_eq!(writes.len(), 2, "{behavior}");
        assert_ne!(
            writes[0]["params"]["expectedVersion"],
            writes[1]["params"]["expectedVersion"]
        );
        assert_eq!(
            writes[1]["params"]["edits"],
            json!([
                {"keyPath":"model","mergeStrategy":"replace","value":OLD},
                {"keyPath":"review_model","mergeStrategy":"replace","value":REVIEW_OLD}
            ])
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn batch_cas_failure_keeps_both_and_compensation_cannot_clobber_an_external_edit() {
    for behavior in ["write_error", "cas_drift", "external_after_write"] {
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let owner = fixture.fake(behavior, "user");
        let error = fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap_err();
        if behavior == "external_after_write" {
            assert_eq!(error.code, "client_selection_repair_conflict");
            assert_eq!(fixture.model(), "External/new-choice");
            assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_NEW));
        } else {
            assert_eq!(error.code, "client_selection_write");
            assert_eq!(fixture.model(), OLD);
            assert_eq!(fixture.review_model().as_deref(), Some(REVIEW_OLD));
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn noop_preserves_bytes_and_matches_only_the_complete_provider() {
    for model in [
        "Krill/model",
        "Krill-10/model",
        "prefix-Krill-1/model",
        "model-Krill-1",
        "Other/Krill-1/model",
    ] {
        let fixture = Fixture::new(model);
        let owner = fixture.fake("normal", "user");
        let before = fixture.bytes();
        assert_eq!(
            fixture
                .plan(&owner, None)
                .await
                .unwrap()
                .apply()
                .await
                .unwrap(),
            ClientSelectionRename::Unchanged
        );
        assert_eq!(fixture.bytes(), before);
        assert!(!fixture
            .requests()
            .iter()
            .any(|row| row["method"] == "config/value/write"));
    }
}

#[cfg(unix)]
#[tokio::test]
async fn unsupported_sources_and_disabled_or_mismatched_layers_fail_before_write() {
    for source in [
        "project",
        "system",
        "mdm",
        "enterpriseManaged",
        "sessionFlags",
        "legacyManagedConfigTomlFromFile",
        "foreign",
    ] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake("normal", source);
        let before = fixture.bytes();
        assert_eq!(
            fixture.plan(&owner, None).await.err().unwrap().code,
            "client_selection_source"
        );
        assert_eq!(fixture.bytes(), before);
        assert!(!fixture
            .requests()
            .iter()
            .any(|row| row["method"] == "config/value/write"));
    }
    for behavior in ["disabled_layer", "layer_mismatch", "unbound_profile"] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake(behavior, "user");
        assert_eq!(
            fixture.plan(&owner, None).await.err().unwrap().code,
            "client_selection_source"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn config_symlink_cannot_escape_the_explicit_custom_codex_home() {
    let fixture = Fixture::new(OLD);
    let owner = fixture.fake("normal", "user");
    let external = fixture.root.join("external-config.toml");
    let before = fixture.bytes();
    std::fs::rename(fixture.codex.join("config.toml"), &external).unwrap();
    std::os::unix::fs::symlink(&external, fixture.codex.join("config.toml")).unwrap();
    assert_eq!(
        fixture.plan(&owner, None).await.err().unwrap().code,
        "client_selection_source"
    );
    assert_eq!(std::fs::read(external).unwrap(), before);
    assert!(!fixture
        .requests()
        .iter()
        .any(|row| row["method"] == "config/value/write"));
}

#[cfg(unix)]
#[tokio::test]
async fn changed_model_or_origin_after_producer_is_not_overwritten() {
    for change_origin in [false, true] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake("normal", "user");
        let plan = fixture.plan(&owner, None).await.unwrap();
        if change_origin {
            fixture.fake("normal", "project");
        } else {
            std::fs::write(
                fixture.codex.join("config.toml"),
                "model = \"External/new-choice\"\n",
            )
            .unwrap();
        }
        let before = fixture.bytes();
        assert!(plan.apply().await.is_err());
        assert_eq!(fixture.bytes(), before);
        assert!(!fixture
            .requests()
            .iter()
            .any(|row| row["method"] == "config/value/write"));
    }
}

#[cfg(unix)]
#[tokio::test]
async fn native_write_error_and_cas_drift_keep_the_current_selection() {
    for behavior in ["write_error", "cas_drift"] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake(behavior, "user");
        let error = fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap_err();
        assert_eq!(error.code, "client_selection_write");
        assert!(!error.message.contains("fixture-secret"));
        assert_eq!(fixture.model(), OLD);
    }
}

#[cfg(unix)]
#[tokio::test]
async fn overridden_or_failed_readback_is_compensated_with_our_receipt_version() {
    for behavior in [
        "overridden",
        "metadata_override",
        "readback_error",
        "readback_mismatch",
    ] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake(behavior, "user");
        let error = fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap_err();
        assert!(!error.message.contains("fixture-secret"));
        assert_ne!(error.code, "client_selection_repair_conflict", "{behavior}");
        assert_eq!(fixture.model(), OLD, "{behavior}");
        let writes: Vec<_> = fixture
            .requests()
            .into_iter()
            .filter(|row| row["method"] == "config/value/write")
            .collect();
        assert_eq!(writes.len(), 2, "{behavior}");
        assert_ne!(
            writes[0]["params"]["expectedVersion"],
            writes[1]["params"]["expectedVersion"]
        );
        assert_eq!(writes[1]["params"]["value"], OLD);
    }
}

#[cfg(unix)]
#[tokio::test]
async fn compensation_never_adopts_a_newer_external_version() {
    let fixture = Fixture::new(OLD);
    let owner = fixture.fake("external_after_write", "user");
    assert_eq!(
        fixture
            .plan(&owner, None)
            .await
            .unwrap()
            .apply()
            .await
            .unwrap_err()
            .code,
        "client_selection_repair_conflict"
    );
    assert_eq!(fixture.model(), "External/new-choice");
    assert!(String::from_utf8(fixture.bytes())
        .unwrap()
        .contains("external-after-write"));
}

#[cfg(unix)]
#[tokio::test]
async fn profile_origin_is_updated_only_when_explicitly_bound_and_verified() {
    let fixture = Fixture::new("root/keep");
    let owner = fixture.fake("normal", "user");
    std::fs::write(
        fixture.codex.join("active.config.toml"),
        format!(
            "model = {}\nreview_model = {}\n",
            json!(OLD),
            json!(REVIEW_OLD)
        ),
    )
    .unwrap();
    let root_before = fixture.bytes();
    assert_eq!(
        fixture
            .plan(&owner, Some("active"))
            .await
            .unwrap()
            .apply()
            .await
            .unwrap(),
        ClientSelectionRename::Updated
    );
    assert_eq!(fixture.bytes(), root_before);
    assert!(
        std::fs::read_to_string(fixture.codex.join("active.config.toml"))
            .unwrap()
            .contains(NEW)
    );
    let profile: toml::Value =
        toml::from_str(&std::fs::read_to_string(fixture.codex.join("active.config.toml")).unwrap())
            .unwrap();
    assert_eq!(profile["review_model"].as_str(), Some(REVIEW_NEW));
    let writes: Vec<_> = fixture
        .requests()
        .into_iter()
        .filter(|row| row["method"] == "config/batchWrite")
        .collect();
    assert_eq!(writes.len(), 1);
    assert_eq!(
        writes[0]["params"]["filePath"],
        json!(fixture.codex.join("active.config.toml"))
    );
    assert!(fixture.plan(&owner, Some("../active")).await.is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn separately_valid_root_and_profile_owners_cannot_be_split_into_partial_writes() {
    let fixture = Fixture::new(OLD);
    let profile = fixture.codex.join("active.config.toml");
    std::fs::write(&profile, format!("review_model = {}\n", json!(REVIEW_OLD))).unwrap();
    let owner = fixture.fake("mixed_profile_owner", "user");
    let before = fixture.bytes();
    let profile_before = std::fs::read(&profile).unwrap();
    assert_eq!(
        fixture
            .plan(&owner, Some("active"))
            .await
            .err()
            .unwrap()
            .code,
        "client_selection_source"
    );
    assert_eq!(fixture.bytes(), before);
    assert_eq!(std::fs::read(profile).unwrap(), profile_before);
    assert!(!fixture
        .requests()
        .iter()
        .any(|row| row["method"] == "config/value/write" || row["method"] == "config/batchWrite"));
}

#[cfg(unix)]
#[tokio::test]
async fn transport_limits_and_timeout_kill_the_owned_descendant() {
    for behavior in ["stdout_limit", "stderr_limit", "timeout"] {
        let fixture = Fixture::new(OLD);
        let owner = fixture.fake(behavior, "user");
        let started = std::time::Instant::now();
        let error = fixture.plan(&owner, None).await.err().unwrap();
        assert!(started.elapsed() < Duration::from_secs(24));
        assert!(matches!(
            error.code.as_str(),
            "client_selection_output_limit" | "client_selection_timeout"
        ));
        let pid: libc::pid_t = std::fs::read_to_string(fixture.codex.join("child-pid"))
            .unwrap()
            .parse()
            .unwrap();
        for _ in 0..100 {
            if unsafe { libc::kill(pid, 0) } != 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_ne!(
            unsafe { libc::kill(pid, 0) },
            0,
            "descendant survived {behavior}"
        );
        assert_eq!(fixture.model(), OLD);
    }
}

#[cfg(target_os = "macos")]
mod real {
    use super::*;
    use std::process::Stdio;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        process::{Child, ChildStdin, ChildStdout, Command},
    };

    fn binary() -> PathBuf {
        assert_eq!(
            std::env::var("OCX_CLIENT_SELECTION_ISOLATED").as_deref(),
            Ok("1")
        );
        let denied = std::net::TcpListener::bind("127.0.0.1:0").unwrap_err();
        assert_eq!(
            denied.kind(),
            std::io::ErrorKind::PermissionDenied,
            "real owner tests require OS network denial"
        );
        let path = PathBuf::from(
            std::env::var_os("OCX_CLIENT_SELECTION_CODEX").expect("explicit Codex binary required"),
        );
        assert!(path.is_absolute() && path.is_file());
        path
    }

    fn evidence(name: &str, value: Value) {
        if let Some(root) = std::env::var_os("OCX_CLIENT_SELECTION_EVIDENCE") {
            std::fs::write(
                PathBuf::from(root).join(format!("{name}.json")),
                serde_json::to_vec_pretty(&value).unwrap(),
            )
            .unwrap();
        }
    }

    struct Client {
        child: Child,
        input: ChildStdin,
        output: BufReader<ChildStdout>,
        next: u64,
        pid: libc::pid_t,
    }

    impl Client {
        async fn start(fixture: &Fixture, binary: &Path) -> Self {
            let sqlite = fixture.root.join("probe-sqlite");
            std::fs::create_dir_all(&sqlite).unwrap();
            let mut command = Command::new(binary);
            command
                .args(["app-server", "--stdio"])
                .current_dir(&fixture.work)
                .env_clear()
                .env("HOME", &fixture.home)
                .env("CODEX_HOME", &fixture.codex)
                .env("CODEX_SQLITE_HOME", &sqlite)
                .env("TMPDIR", &fixture.root)
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .process_group(0);
            let mut child = command.spawn().unwrap();
            let pid = child.id().unwrap() as libc::pid_t;
            let input = child.stdin.take().unwrap();
            let output = BufReader::new(child.stdout.take().unwrap());
            let mut client = Self {
                child,
                input,
                output,
                next: 0,
                pid,
            };
            client.call("initialize", json!({"clientInfo":{"name":"isolated-default-verification","version":"0.0.0"},"capabilities":{"experimentalApi":true}})).await;
            client
                .input
                .write_all(b"{\"method\":\"initialized\"}\n")
                .await
                .unwrap();
            client
        }

        async fn raw(&mut self, method: &str, params: Value) -> Value {
            self.next += 1;
            let id = self.next;
            let line = format!("{}\n", json!({"id":id,"method":method,"params":params}));
            self.input.write_all(line.as_bytes()).await.unwrap();
            self.input.flush().await.unwrap();
            tokio::time::timeout(Duration::from_secs(12), async {
                loop {
                    let mut line = String::new();
                    assert!(self.output.read_line(&mut line).await.unwrap() > 0);
                    let row: Value = serde_json::from_str(&line).unwrap();
                    if row["id"] == id {
                        return row;
                    }
                }
            })
            .await
            .unwrap()
        }

        async fn call(&mut self, method: &str, params: Value) -> Value {
            let row = self.raw(method, params).await;
            assert!(row.get("error").is_none(), "{method} failed");
            row["result"].clone()
        }

        async fn close(mut self) {
            unsafe {
                libc::kill(-self.pid, libc::SIGKILL);
            }
            tokio::time::timeout(Duration::from_secs(2), self.child.wait())
                .await
                .unwrap()
                .unwrap();
        }
    }

    impl Drop for Client {
        fn drop(&mut self) {
            unsafe {
                libc::kill(-self.pid, libc::SIGKILL);
            }
            let _ = self.child.start_kill();
        }
    }

    #[tokio::test]
    #[ignore = "requires explicit official 0.166 binary and OS-isolated tmp HOME/network"]
    async fn real_owner_root_and_fresh_default_thread_follow_the_rename() {
        let binary = binary();
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let history = fixture.codex.join("sessions/sentinel.jsonl");
        std::fs::create_dir_all(history.parent().unwrap()).unwrap();
        std::fs::write(&history, "fixture historical selector Krill-1/keep\n").unwrap();
        let history_before = std::fs::read(&history).unwrap();
        let original = fixture.bytes();
        let plan = fixture.plan(&binary, None).await.unwrap();
        assert_eq!(fixture.bytes(), original, "preflight mutated config");
        // Represents injection changing unrelated client fields between stages.
        std::fs::write(
            fixture.codex.join("config.toml"),
            format!(
                "openai_base_url = \"http://127.0.0.1:9/v1\"\n{}",
                String::from_utf8(original).unwrap()
            ),
        )
        .unwrap();
        assert_eq!(plan.apply().await.unwrap(), ClientSelectionRename::Updated);
        assert_eq!(fixture.model(), NEW);
        let parsed: toml::Value =
            toml::from_str(std::str::from_utf8(&fixture.bytes()).unwrap()).unwrap();
        assert_eq!(parsed["review_model"].as_str(), Some(REVIEW_NEW));
        assert_eq!(std::fs::read(&history).unwrap(), history_before);
        let mut client = Client::start(&fixture, &binary).await;
        let read = client
            .call(
                "config/read",
                json!({"cwd":fixture.work,"includeLayers":true}),
            )
            .await;
        assert_eq!(read["config"]["model"], NEW);
        assert_eq!(read["config"]["review_model"], REVIEW_NEW);
        let request = json!({"cwd":fixture.work,"ephemeral":true,"approvalPolicy":"never","sandbox":"read-only"});
        assert!(request.get("model").is_none() && request.get("config").is_none());
        let thread = client.call("thread/start", request.clone()).await;
        assert_eq!(thread["model"], NEW);
        evidence(
            "real-root-default",
            json!({"passed":true,"model":read["config"]["model"],"reviewModel":read["config"]["review_model"],"origin":read["origins"]["model"],"reviewOrigin":read["origins"]["review_model"],"threadStartRequest":request,"threadStartModel":thread["model"],"suffixPreserved":true,"historyUnchanged":true,"injectionVersionChangeAccepted":true}),
        );
        client.close().await;
        let saved = fixture.bytes();
        assert_eq!(
            fixture
                .plan(&binary, None)
                .await
                .unwrap()
                .apply()
                .await
                .unwrap(),
            ClientSelectionRename::Unchanged
        );
        assert_eq!(fixture.bytes(), saved);
    }

    #[tokio::test]
    #[ignore = "requires explicit official 0.166 binary and OS-isolated tmp HOME/network"]
    async fn real_owner_preserves_single_model_and_supports_review_only_or_unset() {
        let binary = binary();
        let mut checks = Vec::new();
        for (model, review, expected) in [
            (Some(OLD), None, ClientSelectionRename::Updated),
            (None, Some(REVIEW_OLD), ClientSelectionRename::Updated),
            (
                Some("Other/keep"),
                Some(REVIEW_OLD),
                ClientSelectionRename::Updated,
            ),
            (None, None, ClientSelectionRename::Unchanged),
        ] {
            let fixture = Fixture::new("unused");
            fixture.seed_current(model, review);
            let before = fixture.bytes();
            assert_eq!(
                fixture
                    .plan(&binary, None)
                    .await
                    .unwrap()
                    .apply()
                    .await
                    .unwrap(),
                expected
            );
            if expected == ClientSelectionRename::Unchanged {
                assert_eq!(fixture.bytes(), before);
            }
            let mut client = Client::start(&fixture, &binary).await;
            let read = client
                .call(
                    "config/read",
                    json!({"cwd":fixture.work,"includeLayers":true}),
                )
                .await;
            let expected_model = model.map(|value| if value == OLD { NEW } else { value });
            let expected_review = review.map(|_| REVIEW_NEW);
            assert_eq!(read["config"]["model"], json!(expected_model));
            assert_eq!(read["config"]["review_model"], json!(expected_review));
            checks.push(json!({"model":read["config"]["model"],"reviewModel":read["config"]["review_model"],"passed":true}));
            client.close().await;
        }
        evidence(
            "real-single-review-unset",
            json!({"passed":true,"checks":checks}),
        );
    }

    #[tokio::test]
    #[ignore = "requires explicit official 0.166 binary and OS-isolated tmp HOME/network"]
    async fn real_native_batch_rejects_invalid_second_edit_and_stale_cas_atomically() {
        let binary = binary();
        let fixture = Fixture::new(OLD);
        fixture.seed_current(Some(OLD), Some(REVIEW_OLD));
        let mut client = Client::start(&fixture, &binary).await;
        let before = client
            .call(
                "config/read",
                json!({"cwd":fixture.work,"includeLayers":true}),
            )
            .await;
        let original = fixture.bytes();
        let version = before["origins"]["model"]["version"].clone();
        assert_eq!(before["origins"]["review_model"]["version"], version);
        let invalid = client.raw("config/batchWrite", json!({"filePath":fixture.codex.join("config.toml"),"expectedVersion":version,"reloadUserConfig":false,"edits":[
            {"keyPath":"model","mergeStrategy":"replace","value":NEW},
            {"keyPath":"","mergeStrategy":"replace","value":REVIEW_NEW}
        ]})).await;
        assert!(invalid.get("error").is_some());
        assert_eq!(fixture.bytes(), original);
        fixture.seed_current(Some("External/main"), Some("External/review"));
        let external = fixture.bytes();
        let stale = client.raw("config/batchWrite", json!({"filePath":fixture.codex.join("config.toml"),"expectedVersion":version,"reloadUserConfig":false,"edits":[
            {"keyPath":"model","mergeStrategy":"replace","value":NEW},
            {"keyPath":"review_model","mergeStrategy":"replace","value":REVIEW_NEW}
        ]})).await;
        assert!(stale.get("error").is_some());
        assert_eq!(fixture.bytes(), external);
        evidence(
            "real-batch-atomic-refusal",
            json!({"passed":true,"invalidSecondEditCode":invalid["error"]["code"],"staleCasCode":stale["error"]["code"],"bothScalarsAndBytesPreserved":true}),
        );
        client.close().await;
    }

    #[tokio::test]
    #[ignore = "requires explicit official 0.166 binary and OS-isolated tmp HOME/network"]
    async fn real_owner_refuses_changed_selection_and_unsupported_profile() {
        let binary = binary();
        let fixture = Fixture::new(OLD);
        let plan = fixture.plan(&binary, None).await.unwrap();
        std::fs::write(
            fixture.codex.join("config.toml"),
            format!("model = \"External/new-choice\"\n{BASE}"),
        )
        .unwrap();
        let changed = fixture.bytes();
        assert_eq!(
            plan.apply().await.unwrap_err().code,
            "client_selection_changed"
        );
        assert_eq!(fixture.bytes(), changed);
        std::fs::write(
            fixture.codex.join("active.config.toml"),
            format!(
                "model = {}\nreview_model = {}\n",
                json!(OLD),
                json!(REVIEW_OLD)
            ),
        )
        .unwrap();
        let profile_before = std::fs::read(fixture.codex.join("active.config.toml")).unwrap();
        assert!(fixture.plan(&binary, Some("active")).await.is_err());
        assert_eq!(fixture.bytes(), changed);
        assert_eq!(
            std::fs::read(fixture.codex.join("active.config.toml")).unwrap(),
            profile_before
        );
        evidence(
            "real-selection-profile-refusal",
            json!({"passed":true,"externalChoicePreserved":true,"unsupportedProfileRefusedBeforeWrite":true,"rootAndProfileUnchanged":true}),
        );
    }

    #[tokio::test]
    #[ignore = "requires explicit official 0.166 binary and OS-isolated tmp HOME/network"]
    async fn real_native_owner_cas_rejects_external_edit() {
        let binary = binary();
        let fixture = Fixture::new(OLD);
        let mut client = Client::start(&fixture, &binary).await;
        let before = client
            .call(
                "config/read",
                json!({"cwd":fixture.work,"includeLayers":true}),
            )
            .await;
        std::fs::write(
            fixture.codex.join("config.toml"),
            format!("model = \"External/new-choice\"\n{BASE}"),
        )
        .unwrap();
        let external = fixture.bytes();
        let row = client.raw("config/value/write", json!({"filePath":fixture.codex.join("config.toml"),"keyPath":"model","value":NEW,"mergeStrategy":"replace","expectedVersion":before["origins"]["model"]["version"]})).await;
        assert!(row.get("error").is_some());
        assert_eq!(fixture.bytes(), external);
        evidence(
            "real-native-cas",
            json!({"passed":true,"nativeErrorCode":row["error"]["code"],"externalEditPreserved":true}),
        );
        client.close().await;
    }
}
