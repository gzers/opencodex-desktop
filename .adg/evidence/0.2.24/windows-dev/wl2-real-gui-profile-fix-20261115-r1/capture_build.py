"""One-shot terminal build capture; originals and native proof are not copied."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
sha = lambda raw: hashlib.sha256(raw).hexdigest()
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
git = lambda *args: subprocess.check_output(["git", "-C", str(CODE), *args])


def write(name, value):
    with (ROOT / name).open("x", encoding="utf-8") as out:
        json.dump(value, out, ensure_ascii=False, indent=2)


mode = sys.argv[1]
if mode == "library":
    result = load(ROOT / "library01/result.json")
    assert result["exitCode"] == 0
    text = (ROOT / "library01/stdout.log").read_text(encoding="utf-8-sig")
    assert "test result: ok. 7 passed; 0 failed; 4 ignored" in text
    path = CODE / "test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/b424977fd6e5186d/out/opencodex_desktop_lib-b424977fd6e5186d.exe"
    raw = path.read_bytes()
    write("library01-artifact.json", dict(path=str(path), bytes=len(raw), sha256=sha(raw),
        run="library01", accepted=False, sourceHead=load(ROOT / "library01/source.json")["head"], workingDiffCaptured=True))
    print(json.dumps(dict(rawExit=0, passed=7, ignored=4, artifactSha256=sha(raw))))
elif mode == "product":
    head = git("rev-parse", "HEAD").decode().strip()
    tree = git("rev-parse", "HEAD^{tree}").decode().strip()
    assert load(ROOT / "product01/result.json")["exitCode"] == 0
    assert load(ROOT / "product01/source.json")["head"] == head
    rows = {}
    for name in ["library01", "product01"]:
        rows[name] = []
        for row in load(ROOT / name / "source-manifest.json"):
            raw = (ROOT / name / "executed-source" / row["path"]).read_bytes()
            assert sha(raw) == row["sha256"] and len(raw) == row["bytes"]
            if name == "product01" or row["path"].endswith(".rs"):
                assert raw == (CODE / row["path"]).read_bytes(), row["path"]
                assert raw.replace(b"\r\n", b"\n") == git("show", f"{head}:{row['path']}").replace(b"\r\n", b"\n"), row["path"]
                rows[name].append(row)
    write("final-source.json", dict(head=head, tree=tree, inputs=rows, gitLineEndingEquivalence=True,
        preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
    artifacts = []
    for name in ["opencodex-desktop", "ocxd", "ocx"]:
        path = CODE / f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
        raw = path.read_bytes()
        artifacts.append(dict(name=name, path=str(path), bytes=len(raw), sha256=sha(raw), sourceHead=head,
            sourceTree=tree, profile="debug", features=["custom-protocol"]))
    write("product01-artifacts.json", artifacts)
    frontend = load(ROOT.parent / "wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json")
    for row in frontend:
        raw = (CODE / "apps/desktop/ui/dist" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    write("frontend-reuse.json", dict(files=len(frontend), bytes=sum(row["bytes"] for row in frontend), verified=True, rebuilt=False))
    print(json.dumps(dict(head=head, tree=tree, artifacts=artifacts, productInputs=len(rows["product01"]), testRustInputs=len(rows["library01"]))))
else:
    raise ValueError("expected library or product terminal capture")
