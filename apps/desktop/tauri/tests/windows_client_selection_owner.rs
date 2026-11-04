//! Two actual Windows owner cases. Paths come only from the runner's isolated
//! environment and the published resolver/launcher observations, never discovery
//! or a copied npm resolution algorithm. No provider, turn or tool is started.
#[cfg(windows)]
#[path = "../src/infrastructure/windows_query_job.rs"]
mod windows_query_job;

#[cfg(windows)]
mod actual {
    use super::windows_query_job::OwnedQueryJob;
    use opencodex_desktop_lib::{
        infrastructure::{platform, runtime_executable::FixedRuntimeExecutable},
        modules::model_config::cache_refresh::{
            client_npm_runtime::resolve_npm_native,
            client_selection_rename::{
                preflight_current_selection, ClientSelectionRename, ClientSelectionRenamePlan,
            },
        },
        state::ProcessContext,
    };
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::{
        collections::BTreeMap,
        io::Read,
        path::{Path, PathBuf},
        process::Stdio,
        time::Duration,
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
        process::{Child, ChildStdin, ChildStdout, Command},
    };

    const OLD: &str = "Krill-1/Org/RaW-模型/Version@2026";
    const NEW: &str = "Krill/Org/RaW-模型/Version@2026";
    const REVIEW_OLD: &str = "Krill-1/Review/Original-β/v2";
    const REVIEW_NEW: &str = "Krill/Review/Original-β/v2";
    const BASE: &str = "cli_auth_credentials_store = \"file\"\ncheck_for_update_on_startup = false\nweb_search = \"disabled\"\n[analytics]\nenabled = false\n[feedback]\nenabled = false\n";

    fn owned_env(name: &str, root: &Path) -> PathBuf {
        let path = dunce::canonicalize(std::env::var_os(name).expect(name)).expect(name);
        assert!(
            path.starts_with(root) && path != root,
            "{name} escaped the PS-owned root"
        );
        path
    }
    fn sha(path: &Path) -> String {
        let mut file = std::fs::File::open(path).unwrap();
        let mut hash = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let n = file.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        format!("{:x}", hash.finalize())
    }
    struct Fixture {
        home: PathBuf,
        codex: PathBuf,
        work: PathBuf,
        output: PathBuf,
        context: ProcessContext,
        native: PathBuf,
        raw: PathBuf,
        observed: Value,
    }
    impl Fixture {
        async fn new() -> Self {
            assert_eq!(
                std::env::var("OCX_WINDOWS_ACTUAL_OWNER").as_deref(),
                Ok("1")
            );
            let root = dunce::canonicalize(
                std::env::var_os("OCX_WINDOWS_OWNER_ROOT").expect("explicit PS root"),
            )
            .unwrap();
            let home = owned_env("HOME", &root);
            let codex = owned_env("CODEX_HOME", &root);
            let manager = owned_env("OPENCODEX_HOME", &root);
            let work = owned_env("OCX_WINDOWS_OWNER_WORK", &root);
            let observation = owned_env("OCX_WINDOWS_OWNER_BINDING", &root);
            let output = PathBuf::from(
                std::env::var_os("OCX_WINDOWS_OWNER_EVIDENCE")
                    .expect("explicit PS evidence output"),
            );
            assert!(output.is_absolute() && output.is_dir());
            let observed: Value =
                serde_json::from_slice(&std::fs::read(observation).unwrap()).unwrap();
            assert_eq!(observed["mode"], "default-npm-PATH-discovery");
            assert_eq!(observed["forcedRuntimeVariablesAbsent"], true);
            assert_eq!(
                observed["machineObservation"]["displayPath"],
                observed["codexRuntime"]["path"]
            );
            let runtime = &observed["machineObservation"]["runtime"];
            assert_eq!(runtime["source"], "path");
            assert_eq!(runtime["version"], "0.166.0");
            let raw = PathBuf::from(runtime["command"].as_str().unwrap());
            assert!(dunce::canonicalize(&raw).unwrap().starts_with(&root));
            assert_eq!(sha(&raw), observed["publicShim"]["sha256"]);
            // This is the registered production module, using the exact raw
            // official command. It is not a rewrite of the redacted settings.
            let native = resolve_npm_native(&raw, runtime["version"].as_str().unwrap()).unwrap();
            let trace = &observed["publicArtifacts"]["codex"];
            let expected =
                dunce::canonicalize(trace["actualLauncherSpawn"]["path"].as_str().unwrap())
                    .unwrap();
            assert_eq!(dunce::canonicalize(&native).unwrap(), expected);
            assert_eq!(sha(&native), trace["sha256"]);
            assert_eq!(trace["versionOutput"], "codex-cli 0.166.0");
            let context = ProcessContext {
                runtime: FixedRuntimeExecutable::unresolved(),
                working_directory: work.clone(),
                opencodex_home: manager,
            };
            let fixture = Self {
                home,
                codex,
                work,
                output,
                context,
                native,
                raw,
                observed,
            };
            fixture.seed(OLD, REVIEW_OLD);
            // Do not weaken the native owner to accept batch wrappers, even
            // though the dedicated production npm resolver verified the package.
            assert_eq!(
                fixture.plan(&fixture.raw).await.err().unwrap().code,
                "client_selection_executable"
            );
            fixture
        }
        fn file(&self) -> PathBuf {
            self.codex.join("config.toml")
        }
        fn bytes(&self) -> Vec<u8> {
            std::fs::read(self.file()).unwrap()
        }
        fn seed(&self, model: &str, review: &str) {
            std::fs::write(
                self.file(),
                format!(
                    "# isolated current selections\nmodel = {}\nreview_model = {}\n{BASE}",
                    json!(model),
                    json!(review)
                ),
            )
            .unwrap();
        }
        fn digest(&self) -> String {
            format!("sha256:{}", sha(&self.file()))
        }
        async fn plan(
            &self,
            binary: &Path,
        ) -> opencodex_desktop_lib::modules::model_config::schema::ModelResult<
            ClientSelectionRenamePlan,
        > {
            preflight_current_selection(
                &self.context,
                binary,
                &self.home,
                &self.codex,
                &self.work,
                None,
                &BTreeMap::from([("Krill-1".into(), "Krill".into())]),
            )
            .await
        }
        fn evidence(&self, name: &str, result: Value) {
            std::fs::write(
                self.output.join(format!("{name}.json")),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
        }
    }

    struct Client {
        child: Child,
        input: ChildStdin,
        output: BufReader<ChildStdout>,
        stderr: tokio::task::JoinHandle<()>,
        job: OwnedQueryJob,
        next: u64,
    }
    impl Client {
        async fn start(f: &Fixture) -> Self {
            let mut command = Command::new(&f.native);
            command
                .args(["app-server", "--stdio"])
                .current_dir(&f.work)
                .env_clear()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            platform::apply_user_environment(command.as_std_mut(), Some(f.home.as_os_str()));
            command
                .env("HOME", &f.home)
                .env("CODEX_HOME", &f.codex)
                .env(
                    "CODEX_SQLITE_HOME",
                    f.context.opencodex_home.join("owner-probe-sqlite"),
                )
                .env("APPDATA", f.home.join("AppData/Roaming"))
                .env("LOCALAPPDATA", f.home.join("AppData/Local"))
                .env(
                    "PATH",
                    platform::controlled_path(std::iter::empty::<PathBuf>(), None),
                );
            std::fs::create_dir_all(f.context.opencodex_home.join("owner-probe-sqlite")).unwrap();
            let job = OwnedQueryJob::new(&mut command).unwrap();
            let mut child = command.spawn().unwrap();
            job.assign_and_resume(&child).unwrap();
            let input = child.stdin.take().unwrap();
            let output = BufReader::new(child.stdout.take().unwrap());
            let mut error = child.stderr.take().unwrap();
            let stderr = tokio::spawn(async move {
                let mut count = 0;
                let mut buffer = [0; 4096];
                loop {
                    let n = error.read(&mut buffer).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    count += n;
                    assert!(count <= 256 * 1024, "native probe stderr exceeded bound");
                }
            });
            let mut c = Self {
                child,
                input,
                output,
                stderr,
                job,
                next: 0,
            };
            c.call("initialize", json!({"clientInfo":{"name":"windows-isolated-owner-evidence","version":"0.0.0"},"capabilities":{"experimentalApi":true}})).await;
            c.input
                .write_all(b"{\"method\":\"initialized\"}\n")
                .await
                .unwrap();
            c.input.flush().await.unwrap();
            c
        }
        async fn call(&mut self, method: &str, params: Value) -> Value {
            assert!(
                ["initialize", "config/read", "thread/start"].contains(&method),
                "no turns/tools/approvals allowed"
            );
            self.next += 1;
            let id = self.next;
            let mut request =
                serde_json::to_vec(&json!({"id":id,"method":method,"params":params})).unwrap();
            request.push(b'\n');
            self.input.write_all(&request).await.unwrap();
            self.input.flush().await.unwrap();
            tokio::time::timeout(Duration::from_secs(20), async {
                let mut total = 0;
                loop {
                    let mut line = Vec::new();
                    (&mut self.output)
                        .take(2 * 1024 * 1024 + 1)
                        .read_until(b'\n', &mut line)
                        .await
                        .unwrap();
                    assert!(
                        !line.is_empty() && line.len() <= 2 * 1024 * 1024,
                        "native RPC output missing or exceeded line bound"
                    );
                    total += line.len();
                    assert!(total <= 8 * 1024 * 1024);
                    let row: Value = serde_json::from_slice(&line).unwrap();
                    if row["id"] == id {
                        assert!(row.get("error").is_none(), "{method}: native RPC error");
                        return row["result"].clone();
                    }
                    assert!(
                        row.get("id").is_none(),
                        "native client requested a tool/approval; refuse"
                    );
                }
            })
            .await
            .expect("bounded native config/default probe")
        }
        async fn close(mut self) {
            self.job.terminate_and_wait().await.unwrap();
            tokio::time::timeout(Duration::from_secs(2), self.child.wait())
                .await
                .unwrap()
                .unwrap();
            self.stderr.abort();
        }
    }
    impl Drop for Client {
        fn drop(&mut self) {
            let _ = self.job.terminate();
            let _ = self.child.start_kill();
            self.stderr.abort();
        }
    }

    #[tokio::test]
    #[ignore = "requires exact public npm packages and PS-owned temporary Windows identity"]
    async fn windows_actual_dual_batch_and_fresh_default() {
        let f = Fixture::new().await;
        let before = f.bytes();
        let plan = f.plan(&f.native).await.unwrap();
        assert!(plan.has_changes());
        assert_eq!(f.bytes(), before, "preflight must be read-only");
        // An unrelated raw-byte-only change must not be confused with Codex's
        // opaque semantic CAS token. The production adapter owns one dual batch.
        std::fs::write(
            f.file(),
            [
                b"# post-preflight unrelated observation\n".as_slice(),
                before.as_slice(),
            ]
            .concat(),
        )
        .unwrap();
        let digest = f.digest();
        let mut prior = Client::start(&f).await;
        let prior_read = prior
            .call("config/read", json!({"cwd":f.work,"includeLayers":true}))
            .await;
        assert_ne!(prior_read["origins"]["model"]["version"], digest);
        prior.close().await;
        assert_eq!(
            plan.apply_with_file_snapshot(digest.clone()).await.unwrap(),
            ClientSelectionRename::Updated
        );
        let mut fresh = Client::start(&f).await;
        let read = fresh
            .call("config/read", json!({"cwd":f.work,"includeLayers":true}))
            .await;
        assert_eq!(read["config"]["model"], NEW);
        assert_eq!(read["config"]["review_model"], REVIEW_NEW);
        assert_eq!(
            read["origins"]["model"]["version"],
            read["origins"]["review_model"]["version"]
        );
        let request =
            json!({"cwd":f.work,"ephemeral":true,"approvalPolicy":"never","sandbox":"read-only"});
        assert!(request.get("model").is_none() && request.get("config").is_none());
        let thread = fresh.call("thread/start", request.clone()).await;
        assert_eq!(thread["model"], NEW);
        fresh.close().await;
        f.evidence("windows-actual-owner-dual",json!({"result":"pass","rawCommand":f.raw,"actualNative":f.native,"nativeSha256":sha(&f.native),"observedNativeSha256":f.observed["publicArtifacts"]["codex"]["sha256"],"defaultSettings":f.observed["codexRuntime"],"productionResolver":true,"productionPreflightApply":true,"dualBatchCas":true,"model":read["config"]["model"],"reviewModel":read["config"]["review_model"],"origins":read["origins"],"threadStartRequest":request,"freshDefaultModel":thread["model"],"rawFileDigest":digest,"beforeNativeVersion":prior_read["origins"]["model"]["version"],"cmdRefusedByNativeOwner":true,"noTurnOrTools":true,"ownedProbeCleanupConfirmed":true}));
    }
    #[tokio::test]
    #[ignore = "requires exact public npm packages and PS-owned temporary Windows identity"]
    async fn windows_actual_external_choice_refusal() {
        let f = Fixture::new().await;
        let plan = f.plan(&f.native).await.unwrap();
        f.seed("External/main-choice", "External/review-choice");
        let external = f.bytes();
        let error = plan.apply_with_file_snapshot(f.digest()).await.unwrap_err();
        assert_eq!(error.code, "client_selection_changed");
        assert_eq!(f.bytes(), external);
        let mut fresh = Client::start(&f).await;
        let read = fresh
            .call("config/read", json!({"cwd":f.work,"includeLayers":true}))
            .await;
        assert_eq!(read["config"]["model"], "External/main-choice");
        assert_eq!(read["config"]["review_model"], "External/review-choice");
        fresh.close().await;
        f.evidence("windows-actual-owner-external-refusal",json!({"result":"pass","productionResolver":true,"productionPreflightApply":true,"code":error.code,"externalBytesPreserved":true,"freshModel":read["config"]["model"],"freshReviewModel":read["config"]["review_model"],"cmdRefusedByNativeOwner":true,"ownedProbeCleanupConfirmed":true,"noTurnOrTools":true}));
    }
}
