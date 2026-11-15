"""One-shot delivery identity after final product compilation. No native proof."""
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
    with (ROOT / name).open("x", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


head = git("rev-parse", "HEAD").decode().strip()
tree = git("rev-parse", "HEAD^{tree}").decode().strip()
expected = {"library01": (101, 29, 3, 0), "library02": (0, 48, 0, 2),
            "library03": (0, 49, 0, 2), "product01": (0, None, None, None)}
runs, final, test = {}, [], []
for name, (exit_code, passed, failed, ignored) in expected.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == exit_code and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        raw = (run / "executed-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"], row["path"]
        if name == "product01" or name == "library03" and row["path"].endswith(".rs"):
            assert raw == (CODE / row["path"]).read_bytes(), row["path"]
            blob = git("show", f"{head}:{row['path']}")
            assert raw.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n"), row["path"]
            (final if name == "product01" else test).append(dict(**row, workingBytesEqual=True, gitLineEndingEquivalent=True))
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        m = re.search(r"test result: .*?(\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert m and tuple(map(int, m.groups())) == (passed, failed, ignored), name
    runs[name] = dict(rawExit=exit_code, elapsedMs=result["elapsedMs"], passed=passed, failed=failed, ignored=ignored, accepted=False)
assert load(ROOT / "product01/source.json")["head"] == head
write("final-source.json", dict(head=head, tree=tree, files=final, library03RustEqualFinal=test,
                                 preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts = []
for name in ["opencodex-desktop", "ocxd", "ocx"]:
    p = CODE / f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
    raw = p.read_bytes()
    artifacts.append(dict(name=name, path=str(p), bytes=len(raw), sha256=sha(raw), profile="debug",
                           features=["custom-protocol"], sourceHead=head, sourceTree=tree))
write("product01-artifacts.json", artifacts)
scenes = []
for name in ["library01", "library02", "library03"]:
    for scene in load(ROOT / f"{name}-private-scenes.json")["scenes"]:
        assert sha(Path(scene["archive"]).read_bytes()) == scene["archiveSha256"]
        scenes.append(dict(source=scene["source"], archive=scene["archive"], archiveSha256=scene["archiveSha256"],
                           files=len(scene["files"]), bytes=sum(row["bytes"] for row in scene["files"]), removed=False))
previous = load(ROOT.parent / "wl2-working-snapshots-20261115-r1/final-status.json")
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "B3":
        row.update(result="component-pass", fact="required canonical historyRecord binds OriginalHistory/session/context/policy/native archive refs at Closed; old missing fields refuse. Text Closed remains PendingOnly without actual native proofs/current.")
    elif row["id"] == "O1":
        row.update(result="partial-progress", fact="restored original bytes/absence/IDs/eight projections sealed in private same-volume historical data before canonical Closed; original DELETE handles released afterwards. Retained history independently rechecks archives/retired objects without following later live client roots. Full declared side effects/native coordination still incomplete.")
    elif row["id"] == "O2":
        row.update(result="partial/component-pass", fact="native partial-history data, partial current, premature release, archive drift and unknown top-level custody retain/refuse. Exact custody child membership and recheck poison added. Complete production interruption/successor/Closed matrix not executed.")
    elif row["id"] == "S/D":
        row.update(result="typed-history-wiring/component-pass", fact="NormalExit retained with original immutable Stop deadline and checked completion time in nonserde/nonClone ClosedExitHistory; later raw Job/role/exit/source readback uses new action window without extending old Stop. 2 real production tests ignored; no actual typed-history factory/official combined acceptance executed.")
    elif row["id"] == "P":
        row.update(result="refused", fact="history-set transfer/exact namespace registration/new successor capture and broker/GUI production dispatch still missing; public durable gate and authorize_successor refusal retained.")
gui = Path(r"C:/Users/Public/ocx-h024-actual-gui-20261115-r1/out01")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=head, sourceTree=tree,
    contract="TEAM14/IMP140/windows-security-coverage-r4", baselineMain="2ebdd3ae0423903e9e713108a09a165ac36ead37",
    baselineDocs="84ec482eef7afe87d85ea5c911daebc6b8189cfa", previousCode="57904c6bce37f4ba37338285409fa6f9551b40da",
    runs=runs, artifacts=artifacts, matrix=matrix, guiOutputFiles=[p.name for p in gui.iterdir()],
    firstError=dict(run="library01", rawExit=101, passed=29, failed=3, firstRawWin32=32,
        stage="seal_closed_originals: observe_restored_original reopened root while original DELETE-capable owner retained",
        downstream="partial-current fixture then read missing archive raw3; it had failed before injected current fault, not proof of that injection",
        fixed="read original tree through actual retained root handle; unchanged share flags; assert injected current error text", preserved=True),
    cleanup=dict(newPrivateArchivedSceneCount=len(scenes), newPrivateArchivedFiles=sum(s["files"] for s in scenes),
        newPrivateArchivedBytes=sum(s["bytes"] for s in scenes), scenes=scenes, nativeMetadataCloned=False,
        originalsRetained=True, newAccounts=0, newProfiles=0, newPublicTaskRoots=0, newGUIProcesses=0,
        officialProcesses=0, forcedCleanup=False, dailyPathConfigPolicyChanged=False,
        positiveSceneCleanupRejected=load(ROOT / "cleanup-positive-scenes-rejected.json"),
        historicalRejectedDeletionRetried=False, overallCleanupComplete=False),
    limits=["Native storage Closed fixtures never issue actual official ClosedEligibility or successor permission",
        "Historical archives are copies of old bytes plus original identity/security observations; archive fileID is not original fileID",
        "Live original/working tree readback remains mandatory through close; only later retained historical reads ignore new live client contents",
        "ClosedExitHistory factory/wiring compiled, but no new real official Owner/NormalExit/Stop production execution",
        "Shared pure route parser extraction requested in previous working-snapshots round; complete side-effect/native lease implementation still missing",
        "Known closed history-set transfer, new generation registration/capture/Owner dispatch and actual GUI/H-S-D acceptance remain incomplete",
        "Immutable 0c prepared GUI inputs are not overwritten by these new products; no desktop output verified",
        "Populated CAP/trust CREATE_NEW1314/5 conditions unchanged and not retried; historical cleanup/ordinary MSI gaps unchanged",
        "Debug build is not release-budget/frozen-candidate or publication acceptance"],
    allAccepted=False, productionExecuted=False, actualGUI=False, startOpen=False, W2=False, dualSID=False, published=False)
write("final-status.json", status)
print(json.dumps(dict(head=head, tree=tree, runs=runs, artifacts=artifacts,
                      scenes=len(scenes), files=status["cleanup"]["newPrivateArchivedFiles"],
                      bytes=status["cleanup"]["newPrivateArchivedBytes"]), ensure_ascii=False))
