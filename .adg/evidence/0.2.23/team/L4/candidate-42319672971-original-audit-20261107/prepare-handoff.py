import hashlib,json,pathlib,re,shutil,zipfile
root=pathlib.Path(__file__).resolve().parent
raw=root/"receipt-text";base=raw/"ordinary-rust"
def read(p):return json.loads(p.read_text(encoding="utf-8-sig"))
parent=read(base/"rust-ordinary.json");child=read(base/"ordinary/ordinary-suite.json");frozen=read(base/"frozen-artifacts.json")
runtime=read(base/"ordinary/test-2.json");launch=read(base/"ordinary/test-6.stdout.log.launch.json")
assert parent["expectedCommit"]==child["expectedCommit"]=="7730073e96999382697fe2ad8dd8f877352c75a3"
assert len(frozen["artifacts"])==49 and len(child["tests"])==5
assert child["result"]=="fail" and child["exitCode"]==1 and child["failureLine"]==334
assert child["allBinariesAttempted"] is False and child["docTestsExecuted"] is False
assert child["tokenUser"]==child["tokenOwner"]==child["accountSid"]
assert runtime["target"]=="windows_runtime" and type(runtime["exitCode"]) is int and runtime["exitCode"]==101
assert runtime["exitAuthority"]=="retained-CreateProcess-handle"
assert not (base/"ordinary/test-6.json").exists()
assert launch["stage"]=="exited" and launch["resumed"] is True
for k in ["sourceBytesUnchanged","sourceOrArtifactAclRestored","desktopAclRestored","accountAndProfileRemoved"]:assert parent[k] is True
assert parent["remainingNewSidProcesses"]==[] and parent["aclRestoreErrors"]==[]
core=(base/"ordinary/test-6.stdout.log").read_text(encoding="utf-8-sig")
runtime_text=(base/"ordinary/test-2.stdout.log").read_text(encoding="utf-8-sig")
assert "test result: FAILED. 1593 passed; 96 failed; 5 ignored" in core
assert "test result: FAILED. 14 passed; 3 failed; 4 ignored" in runtime_text
core_failures=re.findall(r"^---- (.+) stdout ----",core,re.M)
runtime_failures=re.findall(r"^---- (.+) stdout ----",runtime_text,re.M)
assert len(core_failures)==96 and len(runtime_failures)==3
ci=read(root.parent/"L4-ci-42319738998-20261107/handoff/verification.json")
assert sorted(ci["failedTests"])==sorted(core_failures)
selected=[base/"rust-ordinary.json",base/"frozen-artifacts.json",base/"ordinary/ordinary-suite.json",*sorted((base/"ordinary").glob("test-*"))]
out=root/"handoff";out.mkdir(exist_ok=True)
members=[]
with zipfile.ZipFile(root/"windows-candidate-evidence-42319672971.zip") as z:
    for p in selected:
        rel=p.relative_to(raw);b=p.read_bytes();assert z.read(rel.as_posix())==b
        dest=out/"original"/rel;dest.parent.mkdir(exist_ok=True,parents=True);shutil.copyfile(p,dest)
        members.append({"path":rel.as_posix(),"sha256":hashlib.sha256(b).hexdigest(),"size_bytes":len(b)})
for name in ["package-original-decoded.log","download-verification.json","inspect-original-receipts.py","prepare-handoff.py"]:shutil.copyfile(root/name,out/name)
result={"runId":42319672971,"jobId":128240894462,"conclusion":"failure","sourceCommit":child["expectedCommit"],"artifactId":13553623926,"zipSize":715014352,"zipSha256":"0e9fc18ef21e9204edb535fc5e2ab594b48bb4367a88f99a393d3d18545306f5","originalZipVerified":True,"expectedBinaries":49,"completedExitReceipts":5,"coreStdout":{"passed":1593,"failed":96,"ignored":5,"exitAuthority":"missing"},"windowsRuntime":{"passed":14,"failed":3,"ignored":4,"exitCode":101,"exitAuthority":"retained-CreateProcess-handle","failedTests":runtime_failures},"coreFailureSetEqualsSameTreeCI":True,"coreFailedTests":core_failures,"allBinariesAttempted":False,"docTestsExecuted":False,"lastLaunch":launch,"cleanupVerifiedFromOriginalParent":True,"identityMatches":True,"archiveProgramsExtractedOrExecuted":False,"originalMembers":members}
(out/"verification.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"verified":True,"originalMembers":len(selected),"expectedBinaries":49,"completedExitReceipts":5,"coreStdoutFailed":96,"runtimeExit":101,"runtimeFailed":3,"coreFailureSetEqualsCI":True,"cleanupVerified":True}))
