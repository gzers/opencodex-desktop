"""One-shot source/artifact identity after final MSVC product compilation."""
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
expected = {"library01": (38, 0), "library02": (54, 2), "native01": (1, 0),
            "library03": (54, 2), "product01": (None, None)}
runs, final, rust = {}, [], []
for name, (passed, ignored) in expected.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == 0 and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        raw = (run / "executed-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
        if name == "product01" or name == "library03" and row["path"].endswith(".rs"):
            assert raw == (CODE / row["path"]).read_bytes(), row["path"]
            blob = git("show", f"{head}:{row['path']}")
            assert raw.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n"), row["path"]
            (final if name == "product01" else rust).append(dict(**row, workingBytesEqual=True, gitLineEndingEquivalent=True))
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        m = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert m and tuple(map(int, m.groups())) == (passed, 0, ignored), name
    runs[name] = dict(rawExit=0, elapsedMs=result["elapsedMs"], passed=passed,
                       failed=0 if passed is not None else None, ignored=ignored, accepted=False)
assert load(ROOT / "product01/source.json")["head"] == head
native_inputs = []
for row in load(ROOT / "native01/source-manifest.json"):
    if row["path"] in ["apps/desktop/tauri/src/infrastructure/windows_official_runtime024/process.rs",
                       "test/windows024-native/tests/process.rs"]:
        assert (ROOT / "native01/executed-source" / row["path"]).read_bytes() == (CODE / row["path"]).read_bytes()
        native_inputs.append(row)
assert len(native_inputs) == 2
write("final-source.json", dict(head=head, tree=tree, files=final, library03RustEqualFinal=rust,
                                 native01AffectedInputsEqualFinal=native_inputs,
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
frontend = load(ROOT.parent / "wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json")
for row in frontend:
    raw = (CODE / "apps/desktop/ui/dist" / row["path"]).read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
write("frontend-reuse.json", dict(files=len(frontend), bytes=sum(row["bytes"] for row in frontend),
                                   actualBytesEqualPriorImmutableDist=True, rebuilt=False))
previous = load(ROOT.parent / "wl2-closed-history-20261115-r1/final-status.json")
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "B3":
        row.update(result="component-pass", fact="mandatory exact ordered predecessor references bind old session/private directory/current IDs, canonical/current/manifest/context/policy digests and D volume. Older missing-field records refuse; text/serde only locates pending data.")
    elif row["id"] == "O1":
        row.update(result="native-storage/component-pass", fact="three synthetic native generations capture fresh user originals independently while old exact history remains unchanged; current original bytes restored each cycle. Full official side-effect assessment/live native coordination still incomplete.")
    elif row["id"] == "O2":
        row.update(result="partial/component-pass", fact="missing/sparse/reversed known histories, foreign namespace, source mismatch, native old archive drift and drift before next first move refuse with retained scenes. Complete actual official interruption/Closed matrix still unverified.")
    elif row["id"] == "H":
        row.update(result="prepared/awaiting-user-desktop", fact="old immutable0c actual product GUI/empty-broker entry not replaced; no new desktop result verified. Full production broker/actual GUI native Owner continuity missing.")
    elif row["id"] == "S/D":
        row.update(result="typed-multigeneration-wiring/component-pass", fact="retained completed old NormalExit and supervisor absence facts keep original deadlines; post-Resume old history checks fresh exact registration/raw old native family without falsely requiring new H runtime files absent. Actual factory/old Owner replacement not production-executed.")
    elif row["id"] == "P":
        row.update(result="public-refused/internal-wiring-progress", fact="one consumed ClosedEligibility now creates exact memory action/context/predecessor ticket; new Owner keeps all old original Owners/process/Job/source objects, rechecks complete native set around root changes and before Resume. Public durable gate still refuses; outward Stop/Restart success, full side effects/coordination and real production acceptance not finished.")
gui = Path(r"C:/Users/Public/ocx-h024-actual-gui-20261115-r1/out01")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=head, sourceTree=tree,
    contract="TEAM14/IMP140/windows-security-coverage-r4", baselineMain="2ebdd3ae0423903e9e713108a09a165ac36ead37",
    baselineDocs="84ec482eef7afe87d85ea5c911daebc6b8189cfa", previousCode="754814dc14903868e73223a1a8862c9b33b14e51",
    codeCommits=[git("rev-parse", "e9cbd80fc").decode().strip(), head], runs=runs, artifacts=artifacts,
    matrix=matrix, firstError=None, guiOutputFiles=[p.name for p in gui.iterdir()],
    cleanup=dict(newPrivateArchivedSceneCount=len(scenes), newPrivateArchivedFiles=sum(s["files"] for s in scenes),
        newPrivateArchivedBytes=sum(s["bytes"] for s in scenes), scenes=scenes, nativeMetadataCloned=False,
        originalsRetained=True, newAccounts=0, newProfiles=0, newPublicTaskRoots=0, newGUIProcesses=0,
        officialProcesses=0, forcedCleanup=False, dailyPathConfigPolicyChanged=False,
        deletionAttempted=False, historicalRejectedDeletionRetried=False, overallCleanupComplete=False),
    limits=["Three native generations use synthetic storage close only; no official ClosedEligibility, real service Stop or final-budget success is inferred",
        "Successor ticket, old Owner vector and ClosedAbsentHistory actual typed factory paths are compile/wiring only; no new official production execution",
        "Prospective pure ticket counterexample test mentioned in hypotheses04 was not added/executed; see review-boundaries.json",
        "Public durable gate remains; outward Stop and Restart still return recovery refusal, not complete production behavior",
        "Full declared official side effects/active native coordination pending, including prior shared pure route-declaration extraction request",
        "Existing0c actual GUI inputs remain immutable and task desktop output unverified; full official broker GUI route still missing",
        "CAP/trust populated prerequisites, ordinary MSI and all historic cleanup rejections unchanged; none retried",
        "ZIP copies byte contents only, not native NTFS identity/security; all forensic scenes remain",
        "Debug bins are not release-budget/frozen-candidate or publication acceptance"],
    allAccepted=False, productionExecuted=False, actualGUI=False, startOpen=False, W2=False, dualSID=False, published=False)
write("final-status.json", status)
print(json.dumps(dict(head=head, tree=tree, runs=runs, artifacts=artifacts, inputFiles=len(final), testRustFiles=len(rust),
                      scenes=len(scenes), files=status["cleanup"]["newPrivateArchivedFiles"],
                      bytes=status["cleanup"]["newPrivateArchivedBytes"]), ensure_ascii=False))
