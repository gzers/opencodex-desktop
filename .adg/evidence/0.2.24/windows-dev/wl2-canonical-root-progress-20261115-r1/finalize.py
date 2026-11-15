"""Close this evidence batch after the exact final product build, once."""
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
git = lambda *args: subprocess.check_output(["git", "-C", str(CODE), *args])


def write(name, value):
    with (ROOT / name).open("x", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


head = git("rev-parse", "HEAD").decode().strip()
tree = git("rev-parse", "HEAD^{tree}").decode().strip()
expected = {"before01": (101, 0, 2), "after01": (0, 15, 0),
            "retired-before01": (101, 0, 2), "after02": (101, None, None),
            "after03": (0, 23, 0), "product01": (0, None, None)}
runs, final_source, test_rust = {}, [], []
for name, (code, passed, failed) in expected.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == code and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        raw = (run / "executed-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
        if name == "product01" or name == "after03" and row["path"].endswith(".rs"):
            assert raw == (CODE / row["path"]).read_bytes(), row["path"]
            blob = git("show", f"{head}:{row['path']}")
            assert raw.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n"), row["path"]
            checked = dict(**row, workingBytesEqual=True, gitLineEndingEquivalent=True)
            (final_source if name == "product01" else test_rust).append(checked)
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        match = re.search(r"test result: .*?(\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert match and tuple(map(int, match.groups())) == (passed, failed, 0), name
    runs[name] = dict(rawExit=code, elapsedMs=result["elapsedMs"], passed=passed, failed=failed,
                       ignored=0 if passed is not None else None, accepted=False)
assert load(ROOT / "product01/source.json")["head"] == head
write("final-source.json", dict(head=head, tree=tree, files=final_source,
                                 finalTestRustInputsEqual=test_rust, preexistingUntrackedSchemaUntouched=True))
artifacts = []
for name in ["opencodex-desktop", "ocxd", "ocx"]:
    p = CODE / f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
    raw = p.read_bytes()
    artifacts.append(dict(name=name, path=str(p), bytes=len(raw), sha256=sha(raw),
                           profile="debug", features=["custom-protocol"], sourceHead=head, sourceTree=tree))
write("product01-artifacts.json", artifacts)
scenes = []
for name in ["before01", "after01", "retired-before01", "after03"]:
    for scene in load(ROOT / f"{name}-private-scenes.json")["scenes"]:
        assert sha(Path(scene["archive"]).read_bytes()) == scene["archiveSha256"]
        scenes.append(dict(source=scene["source"], archive=scene["archive"], archiveSha256=scene["archiveSha256"],
                           files=len(scene["files"]), bytes=sum(f["bytes"] for f in scene["files"]), removed=False))
previous = load(ROOT.parent / "wl2-production-originals-20261115-r1/final-status.json")
gui_output_files = [p.name for p in Path(r"C:/Users/Public/ocx-h024-actual-gui-20261115-r1/out01").iterdir()]
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "B3":
        row.update(result="component-pass", fact="canonical v2 requires both recorded working roots before Ready/PreparedRestore/OfficialTeardown and both restorations before Restored/Closed, including zero-step prefixes. Text remains PendingOnly.")
    elif row["id"] == "O1":
        row.update(result="partial-progress", fact="actual retired working root handle/private parent/full image remain retained and rechecked after original restoration; observed retired drift poisons. Official declared side-effect/Ready/PreStop snapshots remain incomplete.")
    elif row["id"] == "O2":
        row.update(result="partial/component-pass", fact="two baseline regressions reproduce premature Ready, two reproduce forgotten retired drift; 23 affected tree/v2 tests pass after fixes. Complete production interruption/Closed/successor matrix missing.")
    elif row["id"] == "H":
        row.update(result="prepared/awaiting-user-desktop" if not gui_output_files else "desktop-output-awaiting-verification", fact="immutable 0c618cdfd actual-GUI/empty-broker input entry prepared earlier. Observed output names recorded separately; no real official Owner or production GUI acceptance. Controller does not inject into marenben desktop.")
    elif row["id"] == "S/D":
        row.update(result="wiring-reused/production-unverified", fact="typed Stop wiring unchanged; no new real official family/source/Stop combination execution.")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=head, sourceTree=tree,
    contract="TEAM14/IMP140/windows-security-coverage-r4", codeCommits=["2b2e3a9c0", head],
    runs=runs, artifacts=artifacts, matrix=matrix, guiOutputFiles=gui_output_files,
    firstError=dict(run="before01", rawExit=101, passed=0, failed=2, rawWin32=None,
        detail="regressions reproduced unperformed-roots pure/native Ready acceptance; native phase2 written but only PendingOnly data, no production proof", preserved=True, fixed=True),
    otherFailures=[dict(run="retired-before01", rawExit=101, passed=0, failed=2, detail="retired working drift omitted from subsequent native readback; original restored, extra node retained", fixed=True),
        dict(run="after02", rawExit=101, executedTests=0, detail="E0308 old test expected PathBuf; adapted original work-ID assertion to retained root handle", staleExecutableNotAttributed=True, fixed=True)],
    cleanup=dict(newPrivateArchivedSceneCount=len(scenes), newPrivateArchivedFiles=sum(s["files"] for s in scenes),
        newPrivateArchivedBytes=sum(s["bytes"] for s in scenes), scenes=scenes, nativeMetadataCloned=False,
        originalsRetained=True, newAccounts=0, newProfiles=0, newPublicTaskRoots=0, GUIProcesses=0,
        officialProcesses=0, forcedCleanup=False, dailyPathConfigPolicyChanged=False,
        historicalRejectedDeletionRetried=False, overallCleanupComplete=False),
    limits=["Snapshot reads do not prove absence of all possible untrusted interval writes",
        "Complete synthetic stage chains remain locator data; no native ClosedEligibility is minted by parsing",
        "Readiness/client side-effect support and qualified successor native history/root-handle release remain unfinished",
        "GUI prepared source 0c remains immutable; current binaries are separate and not copied over those inputs",
        "Populated CAP/trust CREATE_NEW1314/5 prerequisites unchanged and not retried",
        "Debug artifacts are not release-budget or frozen-candidate acceptance"],
    allAccepted=False, productionExecuted=False, actualGUI=False, startOpen=False, W2=False, dualSID=False, published=False)
write("final-status.json", status)
print(json.dumps(dict(head=head, tree=tree, runs=runs, artifacts=artifacts,
                      scenes=len(scenes), sceneBytes=status["cleanup"]["newPrivateArchivedBytes"]), ensure_ascii=False))
