import hashlib
import json
import sys
import zipfile
from pathlib import Path, PurePosixPath

label = sys.argv[1]
assert all(char in "abcdefghijklmnopqrstuvwxyz0123456789-" for char in label)
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
evidence = Path(r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-PROCESS-TREE")
receipt = json.loads((evidence / (label + "-download-verification.json")).read_text(encoding="utf-8-sig"))
zip_path = Path(receipt["zip"])
assert hashlib.sha256(zip_path.read_bytes()).hexdigest() == receipt["sha256"]
output = code / ("test/software/query-job-source/" + label)
output.mkdir(exist_ok=True)
with zipfile.ZipFile(zip_path) as archive:
    assert sum(item.file_size for item in archive.infolist()) < 150_000_000
    for entry in archive.infolist():
        relative = PurePosixPath(entry.filename)
        assert not relative.is_absolute() and ".." not in relative.parts and ":" not in entry.filename
        target = output.joinpath(*relative.parts)
        assert target.resolve().is_relative_to(output.resolve())
        if entry.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(archive.read(entry))
executables = list((output / "windows-backend").glob("opencodex_desktop_lib-*.exe"))
assert len(executables) == 1
exe = executables[0]
digest = hashlib.sha256(exe.read_bytes()).hexdigest()
ci = json.loads((output / "windows-backend/test-executable-sha256.json").read_text(encoding="utf-8-sig"))
assert digest.upper() == ci["Hash"]
result = {"label": label, "sourceCommit": receipt["sourceCommit"], "artifactId": receipt["artifactId"],
          "executable": str(exe), "sha256": digest, "ciTestExecutableReceiptMatched": True,
          "applicationExecutableLaunched": False}
(evidence / (label + "-test-executable-verification.json")).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
