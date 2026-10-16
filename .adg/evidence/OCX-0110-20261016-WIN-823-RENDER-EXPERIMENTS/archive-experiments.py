"""Archive bounded rejected render experiments, deduplicating original PNGs."""
import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
RUNTIME = CODE / "test/runtime"
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
assert not (ROOT / "manifest.json").exists()
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=CODE, text=True).strip() == SOURCE
assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=CODE).strip()
RUNS = {
    "crop-height": "mesh-clipping-20261016T1345",
    "control": "mesh-clipping-control-20261016T1347",
    "crop-transform": "mesh-clipping-transform-20261016T1348",
    "premultiplied": "mesh-alpha-20261016T1354",
}
images, originals = [], []


def copy(source, relative):
    target = ROOT / relative
    assert source.is_file() and not target.exists()
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    assert source.read_bytes() == target.read_bytes()
    originals.append({"path": relative, "sourcePath": str(source),
                      "sha256": hashlib.sha256(target.read_bytes()).hexdigest()})


for name, directory in RUNS.items():
    run = RUNTIME / directory
    copy(run / "pixels.original.json", f"{name}/pixels.original.json")
    report = "compositor-control.json" if name == "control" else "compositor-comparison.json"
    copy(run / report, f"{name}/{report}")
    for path in sorted(run.glob("*.png")):
        if path.name.endswith("-difference.png"):
            continue
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        blob = f"png/{digest}.png"
        if not (ROOT / blob).exists():
            copy(path, blob)
        images.append({"run": name, "name": path.name, "blob": blob, "sha256": digest,
                       "bytes": len(data)})

copy(RUNTIME / "mesh-clipping-20261016T1344/setup-failure.json", "setup/setup-failure.json")
copy(RUNTIME / "mesh-clipping-20261016T1344/method-initial.original.mjs", "methods/initial-origin.original.mjs")
copy(RUNTIME / RUNS["crop-height"] / "method-loopback.original.mjs", "methods/crop-height.original.mjs")
copy(RUNTIME / RUNS["premultiplied"] / "method-premultiplied.original.mjs", "methods/premultiplied.original.mjs")
copy(RUNTIME / RUNS["premultiplied"] / "restoration.json", "production-restoration.original.json")
copy(RUNTIME / RUNS["crop-transform"] / "rejected-candidate/restoration.json",
     "crop-restoration.original.json")

source_path = "apps/desktop/ui/src/features/runtime/motion/meshGlow.ts"
baseline = subprocess.check_output(["git", "show", f"{SOURCE}:{source_path}"], cwd=CODE).decode()
prototype_paths = {
    "crop-height": RUNTIME / RUNS["crop-height"] / "meshGlow-height-initial.original.ts",
    "crop-transform": RUNTIME / RUNS["crop-transform"] / "rejected-candidate/meshGlow.ts",
    "premultiplied": RUNTIME / RUNS["premultiplied"] / "rejected-premultiplied-source.original.ts",
}
prototypes = []
for name, path in prototype_paths.items():
    data = path.read_bytes()
    observation = json.loads((ROOT / name / "pixels.original.json").read_text())
    assert hashlib.sha256(data).hexdigest() == observation["candidateSourceSha256"]
    normalized = data.decode().replace("\r\n", "\n")
    patch = "".join(difflib.unified_diff(baseline.splitlines(keepends=True),
        normalized.splitlines(keepends=True), fromfile=f"{SOURCE}/{source_path}",
        tofile=f"rejected-{name}/{source_path}"))
    (ROOT / name / "rejected-source.patch").write_bytes(patch.encode())
    prototypes.append({"name": name, "rawSourceSha256": hashlib.sha256(data).hexdigest(),
                       "normalizedSourceSha256": hashlib.sha256(normalized.encode()).hexdigest(),
                       "sourceAdopted": False})

(ROOT / "archive-provenance.json").write_bytes((json.dumps({
    "productionSource": SOURCE, "productionWorktreeClean": True,
    "originalCopies": originals, "images": images, "prototypes": prototypes,
    "transformHarnessExactOriginalSeparatelySaved": False,
    "transformHarnessObservation": "Raw PNGs and results retained; the original crop harness plus the later premultiply harness are separately preserved.",
    "hostProductStarted": False, "formalMemoryMeasurementRepeated": False,
    "historicalFailureCleared": False, "releaseAccepted": False,
}, indent=2) + "\n").encode())
print(json.dumps({"runs": len(RUNS), "imageReferences": len(images),
                  "uniquePngFiles": len({x["blob"] for x in images}),
                  "productionSourceUnchanged": True}))
