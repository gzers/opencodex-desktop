"""Create this round's delivery manifest once; never modifies run evidence."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
from datetime import datetime

ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(name, value):
    with (ROOT / name).open("x", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def git(*args):
    return subprocess.check_output(["git", "-C", str(CODE), *args])


head = git("rev-parse", "HEAD").decode().strip()
tree = git("rev-parse", "HEAD^{tree}").decode().strip()
expected = {"library01": (0, 13, 0), "native01": (0, 1, 0),
            "library02": (0, 2, 0), "dispatch01": (0, 28, 2),
            "product01": (0, None, None)}
runs = {}
source_final = []
for name, (exit_code, passed, ignored) in expected.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == exit_code
    rows = load(run / "source-manifest.json")
    for row in rows:
        executed = (run / "executed-source" / row["path"]).read_bytes()
        assert len(executed) == row["bytes"] and sha(executed) == row["sha256"]
        if name == "product01":
            working = (CODE / row["path"]).read_bytes()
            blob = git("show", f"{head}:{row['path']}")
            assert executed == working, row["path"]
            assert executed.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n"), row["path"]
            source_final.append(dict(**row, gitBlob=git("rev-parse", f"{head}:{row['path']}").decode().strip(), workingBytesEqual=True, gitLineEndingEquivalent=True))
    if passed is not None:
        stdout = (run / "stdout.log").read_text(encoding="utf-8-sig")
        m = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", stdout)
        assert m and tuple(map(int, m.groups())) == (passed, 0, ignored), name
    stderr = (run / "stderr.log").read_text(encoding="utf-8-sig")
    runs[name] = dict(rawExit=exit_code, elapsedMs=result["elapsedMs"], passed=passed, ignored=ignored, warningLines=len(re.findall(r"^warning:", stderr, re.M)), productionAcceptance=False)
assert load(ROOT / "product01/source.json")["head"] == head
write("final-source.json", dict(head=head, tree=tree, files=source_final, untrackedSchemaUntouched=True))
artifacts = []
for name in ["opencodex-desktop", "ocxd", "ocx"]:
    path = CODE / f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
    artifacts.append(dict(name=name, path=str(path), bytes=path.stat().st_size, sha256=sha(path.read_bytes()), profile="debug", sourceHead=head, sourceTree=tree))
write("product01-artifacts.json", artifacts)
scenes = []
for name in ["library01", "dispatch01"]:
    for scene in load(ROOT / f"{name}-private-scenes.json")["scenes"]:
        archive = Path(scene["archive"])
        assert sha(archive.read_bytes()) == scene["archiveSha256"]
        scenes.append(dict(source=scene["source"], archive=str(archive), archiveSha256=scene["archiveSha256"], files=len(scene["files"]), bytes=sum(f["bytes"] for f in scene["files"]), removed=False, nativeMetadataCloned=False))
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=head, sourceTree=tree,
    baseline=dict(main="2ebdd3ae0423903e9e713108a09a165ac36ead37", docs="84ec482eef7afe87d85ea5c911daebc6b8189cfa", previousCode="7130ab83f98b7c0d9f908e9f3f756dc1c713f04c", previousDocs="a5b5d988427c1a04006d5b65741a03886c460e36"),
    contract="TEAM14/IMP140/windows-security-coverage-r4", runs=runs, firstError=None,
    missingArtifactHash=dict(run="library02", reason="test executable was overwritten by dispatch01 before separate hashing; no substituted hash"),
    matrix=[
        dict(id="B1", result="component-pass/reused", fact="fixed8+combined ordinary native checks unchanged; controller token is not marenben production acceptance"),
        dict(id="B2", result="partial/reused", fact="LABEL/conditional DACL/resource/filter positives reused; populated CAP/trust CREATE_NEW1314/5 unchanged and not retried"),
        dict(id="B3", result="component-pass", fact="strict v2/policy/current/progress tests passed; current source record schema/bytes/fileID/security/selection pin negative cases added"),
        dict(id="O1", result="partial-implementation", fact="actual ClientOriginals adapter/Store installed before native moves, opaque nonclone/nonserde tags consumed by Owner; current unchanged-working veto does not implement declared official side effects"),
        dict(id="O2", result="partial/component-pass", fact="prior native interruption/current poison tests reused and impacted v2 rerun; full production interruption matrix and qualified successor history transfer missing"),
        dict(id="H", result="environment-ready/implementation-pending", fact="user confirmed marenben logged in; query reports session7 Active. No new actual GUI execution; previous fixture manager/empty host is not actual GUI or official owner"),
        dict(id="S/D", result="wired/component-pass", fact="typed Ready/PreStop/NormalExit/Absent/StopAccepted connected to adapter; 28 pass/2 ignored is not real official combined Stop"),
        dict(id="P", result="refused", fact="public require_durable_capture still refuses; outward Stop recovery refusal and authorize_successor refusal retained; native history release/new capture and public production acceptance not complete"),
        dict(id="MSI/CLI", result="unaffected/reused", fact="no rerun; ordinary MSI acceptance and historic cleanup gaps unchanged")],
    cleanup=dict(newPrivateArchivedSceneCount=len(scenes), newPrivateArchivedFiles=sum(s["files"] for s in scenes), newPrivateArchivedBytes=sum(s["bytes"] for s in scenes), scenes=scenes, originalsRetained=True, nativeMetadataCloned=False, newAccounts=0, newProfiles=0, newPublicEntries=0, officialProcesses=0, forcedCleanup=False, dailyPathConfigPolicyChanged=False, historicalRejectedDeletionRetried=False, overallCleanupComplete=False),
    limits=["Creator guard is owned monotonic Resume state, not suspend-count proof against external untrusted APIs", "Ready/PreStop equality is an intentional veto while official side-effect validation is unfinished", "Store survives phase-tag-consuming errors; record decoding cannot issue native tags", "No real official Source/Owner/Closed/H/S-D production execution", "debug bins are not release-budget validation or frozen candidate"],
    allAccepted=False, productionExecuted=False, actualGUI=False, startOpen=False, W2=False, dualSID=False, published=False)
write("final-status.json", status)
print(json.dumps(dict(head=head, tree=tree, runs=runs, artifacts=artifacts, scenes=len(scenes)), ensure_ascii=False))
