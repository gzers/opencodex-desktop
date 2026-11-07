import datetime,hashlib,json,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent
code=pathlib.Path(r"C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code")
observed="52a84e233c4960e30077667d9990d12705cfb0fc"
pinned="7730073e96999382697fe2ad8dd8f877352c75a3"
def git(*args):return subprocess.check_output(["git",*args],cwd=code)
assert not git("status","--porcelain")
assert git("rev-parse","HEAD").decode().strip()=="05d14a41dafaabcc7d4accce57d7762d6ef0213f"
paths=["apps/desktop/tauri/src/commands/discovery.rs","apps/desktop/tauri/src/commands/discovery_v2_tests.rs"]
texts=[git("show",observed+":"+p) for p in paths]
source,tests=[b.decode() for b in texts]
assert tests.startswith("#[cfg(unix)]\nmod unix {") and "#[cfg(windows)]" not in tests
assert source.count("OwnedQueryJob::new")==1 and "job.assign_and_resume(&child)" in source and "job.terminate_and_wait().await" in source
assert "&[script.as_os_str().into(), \"--version\".into()]" in source and '.env_clear()' in source
assert "default_paths" not in source and 'Some(expected.identity.package_version.as_str())' in source
old=["apps/desktop/tauri/src/infrastructure/windows_private_create.rs","apps/desktop/tauri/src/modules/model_config/model_receive_file_v2.rs","apps/desktop/tauri/src/modules/model_config/model_transaction_v2.rs","apps/desktop/tauri/src/modules/data_root/uninstall.rs","apps/desktop/tauri/src/modules/runtime/archive.rs","test/smoke/windows023-rust-ordinary-helper.ps1"]
assert git("diff-tree","--no-commit-id","--name-only","-r",pinned,observed,"--",*old)==b""
result={"observedUtc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"observedIntegrationCommit":observed,"instructionCommit":"323b75df971658908cd6b23915bb880ce41296b0","stillPinnedBaseline":pinned,"adoptedNewBaseline":False,"nativeExecuted":False,"sourceChecks":{"fixedManagedProof":True,"nodeRunsManagedPackageScriptOnWindows":True,"windowsOwnedJobWiringPresent":True,"newDiscoveryTestsUnixOnly":True,"earlierReportedProductAndExecutorFilesUnchanged":True},"sourceFiles":[{"path":p,"sha256":hashlib.sha256(b).hexdigest()} for p,b in zip(paths,texts)],"activeWindowsJob":{"runId":42323530379,"jobId":128253406896,"lastObservedStage":"Windows backend regression","lastObservedStatus":"in_progress","acceptanceClaim":False}}
(root/"verification.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")
print(json.dumps({"readonlyVerified":True,"nativeAcceptance":False,"newTestsUnixOnly":True,"earlierFailureFilesUnchanged":True}))
