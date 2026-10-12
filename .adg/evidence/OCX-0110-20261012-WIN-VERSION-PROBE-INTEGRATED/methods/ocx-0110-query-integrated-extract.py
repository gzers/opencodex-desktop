"""Extract only verified task artifacts; never install or launch their contents."""
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path, PurePosixPath

evidence = Path(sys.argv[1] if len(sys.argv) > 1 else
                r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-INTEGRATED")
source = sys.argv[2] if len(sys.argv) > 2 else "4852820b89f7f9df7d54bc623e94f3f55d884023"
assert re.fullmatch(r"[0-9a-f]{40}", source)
assert evidence.resolve().is_relative_to(Path(r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence").resolve())
downloads = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\software\downloads")
values = json.loads((evidence / "download-verification.json").read_text(encoding="utf-8-sig"))
for value in values:
    assert value["sourceCommit"] == source
    archive_path = Path(value["zipPath"])
    assert archive_path.resolve().is_relative_to(downloads.resolve())
    assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == value["sha256"]
    destination = Path(value["extracted"])
    assert destination.resolve().is_relative_to(downloads.resolve())
    destination.mkdir(exist_ok=True)
    with zipfile.ZipFile(archive_path) as archive:
        assert sum(entry.file_size for entry in archive.infolist()) < 300_000_000
        for entry in archive.infolist():
            relative = PurePosixPath(entry.filename)
            assert not relative.is_absolute() and ".." not in relative.parts and ":" not in entry.filename
            target = destination.joinpath(*relative.parts)
            assert target.resolve().is_relative_to(destination.resolve())
            if entry.is_dir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            data = archive.read(entry)
            if target.exists():
                assert target.read_bytes() == data, "Refuse overwriting changed extracted file"
            else:
                target.write_bytes(data)
    print(json.dumps({"artifactId": value["id"], "extracted": str(destination), "installerExecuted": False}))
