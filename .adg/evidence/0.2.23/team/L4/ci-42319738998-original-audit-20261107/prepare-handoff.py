import hashlib,json,pathlib,shutil,subprocess,zipfile,re
root=pathlib.Path(__file__).resolve().parent
raw=root/"receipt-text"
base=raw/"out/windows-backend/ordinary-rust"
child=json.loads((base/"ordinary/ordinary-suite.json").read_text(encoding="utf-8-sig"))
parent=json.loads((base/"rust-ordinary.json").read_text(encoding="utf-8-sig"))
frozen=json.loads((base/"frozen-artifacts.json").read_text(encoding="utf-8-sig"))
assert child["expectedCommit"]=="2d39eefa68a58a5b221bc26d40d92d0d3f1e953f"
assert child["result"]=="fail" and len(child["tests"])==5
assert len(frozen["artifacts"])==49
assert all(type(t["exitCode"]) is int and t["exitCode"]==0 and t["exitAuthority"]=="retained-CreateProcess-handle" for t in child["tests"])
assert child["allBinariesAttempted"] is False and child["docTestsExecuted"] is False
assert child["failureLine"]==334
assert not (base/"ordinary/test-6.json").exists()
assert child["tokenUser"]==child["tokenOwner"]==child["accountSid"]
for key in ["sourceBytesUnchanged","sourceOrArtifactAclRestored","desktopAclRestored","accountAndProfileRemoved"]:assert parent[key] is True
assert parent["remainingNewSidProcesses"]==[] and parent["aclRestoreErrors"]==[]
text=(base/"ordinary/test-6.stdout.log").read_text(encoding="utf-8-sig")
assert "test result: FAILED. 1593 passed; 96 failed; 5 ignored" in text
code=pathlib.Path(r"C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code")
def git(*args):return subprocess.check_output(["git",*args],cwd=code).decode().strip()
merge_tree=git("rev-parse",child["expectedCommit"]+"^{tree}")
baseline_tree=git("rev-parse","7730073e96999382697fe2ad8dd8f877352c75a3^{tree}")
assert merge_tree==baseline_tree=="8c419ad576ba6a0afc263ace82323056e17b91f0"
selected=[base/"rust-ordinary.json",base/"frozen-artifacts.json",base/"ordinary/ordinary-suite.json",*sorted((base/"ordinary").glob("test-*"))]
out=root/"handoff";out.mkdir(exist_ok=True)
with zipfile.ZipFile(root/"windows-startup-evidence-42319738998.zip") as z:
    for p in selected:
        rel=p.relative_to(raw);assert z.read(rel.as_posix())==p.read_bytes()
        dest=out/"original"/rel;dest.parent.mkdir(exist_ok=True,parents=True);shutil.copyfile(p,dest)
for name in ["windows-original-decoded.log","inspect-ci-evidence.py","prepare-handoff.py"]:shutil.copyfile(root/name,out/name)
result={"runId":42319738998,"jobId":128241109814,"jobConclusion":"failure","headCommit":"05d14a41dafaabcc7d4accce57d7762d6ef0213f","actualMergeCommit":child["expectedCommit"],"actualMergeTree":merge_tree,"treeEqualsPinned773":True,"artifactId":13553234235,"zipSize":23854943,"zipSha256":"fc4db5380ee6679a1ff0f771437852d84619441c6dfe662bb3971f6fd1ce1d45","verifiedAgainstOriginalZip":True,"completedExitReceipts":5,"coreStdout":{"passed":1593,"failed":96,"ignored":5,"exitAuthority":"missing"},"allBinariesAttempted":False,"docTestsExecuted":False,"cleanupVerifiedFromOriginalParent":True,"identityMatches":True,"failedTests":re.findall(r"^---- (.+) stdout ----",text,re.M),"nativeBinariesExtractedOrRunByAudit":False}
(out/"verification.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"verified":True,"selectedOriginalMembers":len(selected),"completedExitReceipts":5,"coreFailedStdout":96,"treeEquals773":True,"cleanupVerified":True}))
