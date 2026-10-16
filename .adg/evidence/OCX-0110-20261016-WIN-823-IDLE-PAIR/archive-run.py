"""Copy one completed native pair by allowlist; never copy manager profiles."""
import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUN = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-823-pair-20261016T125734")
PREP = ROOT.parent / "OCX-0110-20261016-WIN-823-FRONT-PREP"
assert not (ROOT / "manifest.json").exists(), "Frozen archive; do not collect again"
FILES = [
    "human-window.json", "fixture-ready.json",
    "idle-baseline019-launch.json", "idle-candidate0110-launch.json",
    "baseline019-sky-premature-ready.original.json",
    "baseline019-sky-settled-ready.json", "candidate0110-sky-settled-ready.json",
    "baseline019-prerequisite-rejection.json",
    "baseline019-dimension-prerequisite-rejection.json",
    "observer-dpi-readback.json", "read-dimensions.ps1",
    "baseline019-exit-observer-ready.json", "candidate0110-exit-observer-ready.json",
    "baseline019-exit.json", "candidate0110-exit.json",
    "comparison.json", "midrun-fixture-and-preservation-readback.json",
    "final-preservation-readback.json", "foreground-release.json",
]
for version in ("baseline019", "candidate0110"):
    FILES.extend(f"{version}/{name}" for name in (
        "identity.json", "samples.original.jsonl", "progress.json", "terminal.json"))
copied = []
for relative in FILES:
    source, target = RUN / relative, ROOT / relative
    assert source.is_file() and not source.is_symlink()
    assert not target.exists(), f"Do not overwrite {relative}"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    data = source.read_bytes()
    assert target.read_bytes() == data
    copied.append({"path": relative, "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
shutil.copyfile(PREP / "contract.json", ROOT / "contract.original.json")
(ROOT / "runtime-copy-receipt.json").write_text(json.dumps({
    "sourceDirectory": str(RUN), "files": copied,
    "managerProfileOrPrivateSddlCopied": False,
    "frozenPreparationModified": False,
}, indent=2), encoding="utf-8")
print(json.dumps({"filesCopied": len(copied), "byteExact": True}))
