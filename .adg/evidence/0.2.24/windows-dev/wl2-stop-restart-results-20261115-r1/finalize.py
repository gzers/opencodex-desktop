"""Verify final source identity and retain failures without granting acceptance."""
from pathlib import Path
from datetime import datetime
import hashlib
import json
import re
import subprocess

ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda b: hashlib.sha256(b).hexdigest()
git = lambda *a: subprocess.check_output(["git", "-C", str(CODE), *a])


def write(name, value):
    with (ROOT / name).open("x", encoding="utf-8", newline="\n") as out:
        out.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


head = git("rev-parse", "HEAD").decode().strip()
tree = git("rev-parse", "HEAD^{tree}").decode().strip()
assert head == "03d8d4d69db40480fb200944896824c52572fbbd"
runs, final, rust = {}, [], []
for name, rc, passed, ignored in [
    ("library01", 101, None, None),
    ("library02", 0, 57, 2),
    ("product01", 0, None, None),
]:
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == rc and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        raw = (run / "executed-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
        if name == "product01" or name == "library02" and row["path"].endswith(".rs"):
            assert raw == (CODE / row["path"]).read_bytes(), row["path"]
            blob = git("show", f"{head}:{row['path']}")
            assert raw.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n"), row["path"]
            (final if name == "product01" else rust).append(dict(
                **row, workingBytesEqual=True, gitLineEndingEquivalent=True))
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        match = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert match and tuple(map(int, match.groups())) == (passed, 0, ignored)
    runs[name] = dict(rawExit=rc, elapsedMs=result["elapsedMs"], passed=passed,
                      failed=0 if passed is not None else None, ignored=ignored,
                      testsExecuted=0 if rc else passed, accepted=False)
assert load(ROOT / "product01/source.json")["head"] == head
assert "E0308" in (ROOT / "library01/stderr.log").read_text(encoding="utf-8-sig")
write("final-source.json", dict(head=head, tree=tree, files=final,
    library02RustEqualFinal=rust, preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts = []
for name in ["opencodex-desktop", "ocxd", "ocx"]:
    path = CODE / f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
    raw = path.read_bytes()
    artifacts.append(dict(name=name, path=str(path), bytes=len(raw), sha256=sha(raw),
        profile="debug", features=["custom-protocol"], sourceHead=head, sourceTree=tree))
write("product01-artifacts.json", artifacts)
scenes = []
for scene in load(ROOT / "library02-private-scenes.json")["scenes"]:
    assert sha(Path(scene["archive"]).read_bytes()) == scene["archiveSha256"]
    scenes.append(dict(source=scene["source"], archive=scene["archive"],
        archiveSha256=scene["archiveSha256"], files=len(scene["files"]),
        bytes=sum(row["bytes"] for row in scene["files"]), removed=False))
frontend = load(ROOT.parent / "wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json")
for row in frontend:
    raw = (CODE / "apps/desktop/ui/dist" / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
write("frontend-reuse.json", dict(files=len(frontend), bytes=sum(row["bytes"] for row in frontend),
    actualBytesEqualPriorImmutableDist=True, rebuilt=False))
previous = load(ROOT.parent / "wl2-successor-capture-20261115-r1/final-status.json")
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "S/D":
        row.update(result="native-close-readback-wiring/component-pass", fact="Current Closed tag now authorizes only fresh source/old-history/raw-family/listener/record/supervisor/tree rechecks. Actual production native factory and Stop/Restart combinations remain unexecuted.")
    elif row["id"] == "P":
        row.update(result="public-refused/stop-restart-wiring", fact="Stop returns Stopped only after typed Closed and full current native readback plus original-window completion; Repeat Stop/status recheck retained originals. Restart preserves original outer30/sub10 and exact launch input, no new ActionWindow. Public durable gate unchanged; live official side effects/coordination and production acceptance incomplete.")
    elif row["id"] == "H":
        row.update(result="actual-desktop-failed/retained", fact="User ran immutable0c actual product GUI observer under marenben ordinary SID1003/session7. Observer raw101: gui-normal.json absent; real GUI admission refused. Same-SID export complete. Separate GUI follow-up retains and diagnoses this failure; no full H acceptance.")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=head, sourceTree=tree,
    contract="TEAM14/IMP140/windows-security-coverage-r4", baselineMain="2ebdd3ae0423903e9e713108a09a165ac36ead37",
    baselineDocs="84ec482eef7afe87d85ea5c911daebc6b8189cfa", previousCode=previous["sourceHead"],
    codeCommits=[head], runs=runs, artifacts=artifacts, matrix=matrix,
    firstError=dict(run="library01", rawExit=101, testsExecuted=0, code="E0308",
        file="apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024_tests.rs",
        line=55, cause="ProcessCommand::new expects three PathBuf values; test supplied &str",
        fix="explicit .into() conversions; original assertions unchanged", newExecutableProduced=False),
    sharedMinimumRequests=["shared-status-deadline-request-r2.json",
        "../wl2-working-snapshots-20261115-r1/shared-minimum-request.json"],
    guiOutputFiles=[p.name for p in Path(r"C:/Users/Public/ocx-h024-actual-gui-20261115-r1/out01").iterdir()],
    cleanup=dict(newPrivateArchivedSceneCount=len(scenes), newPrivateArchivedFiles=sum(s["files"] for s in scenes),
        newPrivateArchivedBytes=sum(s["bytes"] for s in scenes), scenes=scenes, nativeMetadataCloned=False,
        originalsRetained=True, newAccounts=0, newProfiles=0, newPublicTaskRoots=0,
        newGUIProcesses=0, officialProcesses=0, forcedCleanup=False, dailyPathConfigPolicyChanged=False,
        deletionAttempted=False, historicalRejectedDeletionRetried=False, overallCleanupComplete=False),
    limits=["57 passes are component/storage/private-time tests, not real native StopReceipt factory or official production acceptance",
        "Explicit recovery_state_at consumes an original deadline but is not called by the shared StatusSource yet; old no-deadline interface remains conservative",
        "Full official side effects/live NativeLease coordination need the requested shared pure route declaration parser; no duplicate Windows parser or shared edits",
        "Restart private time fixture tests timely Stop at9 still historically valid at12; it does not simulate a real Stop or expired complete outer ActionWindow",
        "Actual0c GUI inputs remain immutable; real desktop run failed admission/raw101 and is retained for separate GUI follow-up; full official Owner/broker/H-S-D production proofs incomplete",
        "CAP/trust populated prerequisites and ordinary MSI/historic cleanup unchanged; rejected cleanup never retried",
        "ZIP copies preserve file bytes only, not NTFS identity/security; forensic originals retained",
        "Debug products are not release-budget or frozen-candidate acceptance"],
    allAccepted=False, productionExecuted=False, actualGUI=False, startOpen=False,
    W2=False, dualSID=False, published=False)
status["guiFailureEvidence"] = "../wl2-real-gui-run-20261115-r1/"
write("final-status.json", status)
print(json.dumps(dict(head=head, tree=tree, runs=runs, artifacts=artifacts,
    inputFiles=len(final), testRustFiles=len(rust), scenes=len(scenes),
    files=status["cleanup"]["newPrivateArchivedFiles"], bytes=status["cleanup"]["newPrivateArchivedBytes"])))
