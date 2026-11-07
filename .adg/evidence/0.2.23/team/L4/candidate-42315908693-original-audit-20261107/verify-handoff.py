import hashlib,json,pathlib,re,shutil,subprocess,zipfile,datetime
ROOT=pathlib.Path(__file__).resolve().parent
CODE=pathlib.Path(r"C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code")
SOURCE="47eb7cf7420ba9127f4d7047e4d83231f826035f"
BASELINE="7730073e96999382697fe2ad8dd8f877352c75a3"
ZIP_SHA="e40a9767c614989972ef070f37ab334ecd474c8553b2809511a6adf9e2577073"
def read(p):return json.loads(p.read_text(encoding="utf-8-sig"))
def write(p,value):p.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
def git(*args):return subprocess.check_output(["git",*args],cwd=CODE)
archive=ROOT/"windows-candidate-evidence-42315908693.zip"
assert archive.stat().st_size==715119881
with archive.open("rb") as f:assert hashlib.file_digest(f,"sha256").hexdigest()==ZIP_SHA
raw=ROOT/"receipt-text"
parent=read(raw/"ordinary-rust/rust-ordinary.json")
child=read(raw/"ordinary-rust/ordinary/ordinary-suite.json")
frozen=read(raw/"ordinary-rust/frozen-artifacts.json")
core=read(raw/"ordinary-rust/ordinary/test-23.json")
launch=read(raw/"ordinary-rust/ordinary/test-49.stdout.log.launch.json")
assert child["expectedCommit"]==SOURCE
assert child["result"]=="fail" and child["exitCode"]==1
assert child["allBinariesAttempted"] is False and child["docTestsExecuted"] is False
assert len(child["tests"])==48
assert type(core["exitCode"]) is int and core["exitCode"]==101
assert core["exitAuthority"]=="retained-CreateProcess-handle"
assert not (raw/"ordinary-rust/ordinary/test-49.json").exists()
assert child["tokenUser"]==child["tokenOwner"]==child["accountSid"]
for key in ["sourceBytesUnchanged","sourceOrArtifactAclRestored","desktopAclRestored","accountAndProfileRemoved"]:assert parent[key] is True
assert parent["remainingNewSidProcesses"]==[] and parent["aclRestoreErrors"]==[]
summaries=[]
for p in sorted((raw/"ordinary-rust/ordinary").glob("test-*.stdout.log"),key=lambda p:int(p.name.split("-")[1].split(".")[0])):
    text=p.read_text(encoding="utf-8-sig")
    matches=re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
    assert len(matches)>=1,(p.name,matches)
    status,passed,failed,ignored=matches[-1]
    summaries.append({"path":p.relative_to(raw).as_posix(),"stdoutStatus":status,"passed":int(passed),"failed":int(failed),"ignored":int(ignored),"hasExitReceipt":p.with_name(p.name.replace(".stdout.log",".json")).exists()})
assert len(summaries)==49
assert [x for x in summaries if x["failed"]]==[summaries[22],summaries[48]]
assert (summaries[22]["passed"],summaries[22]["failed"],summaries[22]["ignored"])==(1590,97,5)
assert (summaries[48]["passed"],summaries[48]["failed"],summaries[48]["ignored"])==(14,3,4)
static=[]
paths=["src/infrastructure/windows_private_create.rs","src/modules/model_config/model_receive_file_v2.rs","src/modules/model_config/model_transaction_v2.rs","src/modules/data_root/uninstall.rs","src/modules/instance/mod.rs","src/modules/runtime/archive.rs","tests/windows_runtime.rs"]
for rel in paths:
    path="apps/desktop/tauri/"+rel
    old=git("show",SOURCE+":"+path);new=git("show",BASELINE+":"+path)
    static.append({"path":path,"sourceSha256":hashlib.sha256(old).hexdigest(),"baselineSha256":hashlib.sha256(new).hexdigest(),"unchangedAtPinned773":old==new})
receive=git("show",SOURCE+":apps/desktop/tauri/src/modules/model_config/model_receive_file_v2.rs").decode()
creation=git("show",SOURCE+":apps/desktop/tauri/src/infrastructure/windows_private_create.rs").decode()
assert "D:P(A;;FA;;;OW)" in receive and "a == b" in receive
assert "(A;{inherit};FA;;;{sid})(A;{inherit};FA;;;SY)" in creation
result={"schemaVersion":1,"observedUtc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runId":42315908693,"artifactId":13553127671,"sourceCommit":SOURCE,"pinnedBaseline":BASELINE,"originalZip":{"size_bytes":715119881,"sha256":ZIP_SHA,"verified":True},"nativeProgramsExtractedOrExecutedByThisAudit":False,"expectedStdoutGroups":49,"completedExitReceipts":48,"authoritativeLibraryExit":core["exitCode"],"lastBinaryExitAuthority":"unavailable","suiteFailure":{"code":child.get("failureCode"),"type":child.get("failureType"),"line":child.get("failureLine"),"lastLaunch":launch},"cleanupVerifiedFromOriginalParent":True,"ordinaryIdentityMatches":True,"allBinariesAttempted":False,"docTestsExecuted":False,"stdoutOnlyAggregate":{k:sum(x[k] for x in summaries) for k in ["passed","failed","ignored"]},"stdoutCountsAreNotFullSuiteAcceptance":True,"stdoutGroups":summaries,"sourceComparisons":static}
out=ROOT/"handoff";out.mkdir(exist_ok=True)
write(out/"verification.json",result)
selected=[raw/"ordinary-rust/rust-ordinary.json",raw/"ordinary-rust/frozen-artifacts.json",raw/"ordinary-rust/ordinary/ordinary-suite.json"]
selected.extend(sorted((raw/"ordinary-rust/ordinary").glob("test-*")))
assert all(p.is_file() and p.suffix in [".json",".log"] for p in selected)
with zipfile.ZipFile(archive) as z:
    for p in selected:
        rel=p.relative_to(raw)
        assert z.read(rel.as_posix())==p.read_bytes(),str(rel)
        dest=out/"original"/rel;dest.parent.mkdir(exist_ok=True,parents=True);shutil.copyfile(p,dest)
for name in ["download-verification.json","package-original-decoded.log","verify-handoff.py"]:shutil.copyfile(ROOT/name,out/name)
write(out/"original-member-manifest.json",[{"path":p.relative_to(raw).as_posix(),"size_bytes":p.stat().st_size,"sha256":hashlib.sha256(p.read_bytes()).hexdigest()} for p in selected])
print(json.dumps({"verified":True,"originalMembers":len(selected),"stdoutGroups":49,"exitReceipts":48,"libraryExit":101,"zipSha256":ZIP_SHA,"sourceComparisons":static},ensure_ascii=True))
