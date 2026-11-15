"""Snapshot this run's executed EXE and private retained scene bytes once."""
from pathlib import Path
import hashlib
import json
import re
import sys
import zipfile

ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
name = sys.argv[1]
assert name in ["library01", "library02", "library03"]
run = ROOT / name
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
exe = CODE / "test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/75e8f4b92f415339/out/opencodex_desktop_lib-75e8f4b92f415339.exe"
with (ROOT / f"{name}-artifact.json").open("x", encoding="utf-8") as out:
    json.dump(dict(path=str(exe), bytes=exe.stat().st_size, sha256=sha(exe.read_bytes()),
                   run=name, rawExit=load(run / "result.json")["exitCode"], accepted=False), out, indent=2)
text = "".join((run / f"{stream}.log").read_text(encoding="utf-8-sig") for stream in ["stdout", "stderr"])
scenes = []
parent = CODE / "test/windows024-native/target/r4-private-scenes"
for n, source in enumerate(dict.fromkeys(re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)", text))):
    root = Path(source)
    dest = parent / f"closed-history-{name}-{n}.zip"
    files = []
    with zipfile.ZipFile(dest, "x", compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(root.rglob("*")):
            if p.is_file():
                raw = p.read_bytes()
                rel = p.relative_to(root).as_posix()
                archive.writestr(rel, raw)
                files.append(dict(path=rel, bytes=len(raw), sha256=sha(raw)))
    with zipfile.ZipFile(dest) as archive:
        assert all(sha(archive.read(row["path"])) == row["sha256"] for row in files)
    scenes.append(dict(source=source, archive=str(dest), archiveSha256=sha(dest.read_bytes()),
                       files=files, verified=True, removed=False))
with (ROOT / f"{name}-private-scenes.json").open("x", encoding="utf-8") as out:
    json.dump(dict(scenes=scenes, deletionAttempted=False, nativeMetadataCloned=False), out, indent=2)
print(json.dumps(dict(run=name, scenes=len(scenes), files=sum(len(s["files"]) for s in scenes),
                      bytes=sum(f["bytes"] for s in scenes for f in s["files"]))))
