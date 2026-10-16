"""Append full tool CI and original build observations without rewriting prior evidence."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/windows-working-set/opencodex-desktop")
CI = CODE / "test/runtime/working-set-gpu-20261016T2145"
BASELINE = CODE / "test/runtime/baseline-build-provenance-20261016T2205"
assert not (ROOT / "original-copies.json").exists()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(name, value):
    with (ROOT / name).open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


copies = []


def copy(path, relative):
    data = path.read_bytes()
    text = data.decode("utf-8-sig")
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", text), path.name
    assert not re.search(r"\bgh[pousr]_[A-Za-z0-9]{20,}", text), path.name
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    copies.append({"path": relative, "runtimePath": str(path),
                   "bytes": len(data), "sha256": sha(data)})


terminal = []
for path in CI.glob("ci-*.original.json"):
    if re.fullmatch(r"ci-\d{8}T\d+Z.original.json", path.name):
        value = read(path)
        if value["run"]["status"] == "completed":
            assert value["run"]["head_sha"] == "3a2ff08a6b480d237e93502626861090aa14423d"
            terminal.append(path)
path, = terminal
copy(path, "original/ci/terminal.original.json")
copy(CI / "ci-terminal-admission.json", "original/ci/admission.json")
copy(CI / "collect-ci-terminal.py", "original/ci/collect-ci-terminal.py")
admission = read(CI / "ci-terminal-admission.json")
for row in admission["requiredJobLogs"]:
    copy(CI / row["path"], "original/ci/" + row["path"])
for path in sorted(BASELINE.iterdir()):
    assert path.is_file() and path.suffix in {".json", ".log", ".py", ".ps1"}
    copy(path, "original/baseline-build/" + path.name)
write("original-copies.json", copies)
bindings = []
for name in ["OCX-0110-20261016-WIN-WORKING-SET-POINT", "OCX-0110-20261016-WIN-B4-CANDIDATE",
             "OCX-0110-20261016-WIN-B4-IDLE-ROLES"]:
    directory = ROOT.parent / name
    bindings.append({"directory": directory.relative_to(PROJECT).as_posix(),
                     "manifestSha256": sha((directory / "manifest.json").read_bytes())})
write("prior-evidence-bindings.json", bindings)
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
print(json.dumps({"originalCopies": len(copies), "wholeToolCiSucceeded": admission["wholeToolCiAdmitted"],
                  "originalRssFailureCleared": False}))
