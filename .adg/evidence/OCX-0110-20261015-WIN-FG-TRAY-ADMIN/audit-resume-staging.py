"""Read-only staging scope and original-byte audit for the second foreground receipt."""
import hashlib
import json
import subprocess
from pathlib import Path

workspace = Path(__file__).resolve().parents[3]
evidence = ".adg/evidence/OCX-0110-20261015-WIN-FG-TRAY-ADMIN/"
allowed = {
    "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md",
    "docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/"
    "05-运行维护/20261009-概览升级入口与备份通知策略/验证/"
    "20261015-Windows-e6托盘重开与双权限令牌/README.md",
}


def git(*args):
    return subprocess.check_output(["git", "-c", "core.longpaths=true", *args], cwd=workspace)


assert git("branch", "--show-current").decode().strip() == "docs/governance-main"
paths = [name for name in git("diff", "--cached", "--name-only", "-z").decode().split("\0") if name]
assert paths and all(name.startswith(evidence) or name in allowed for name in paths)
assert "administrator-exit-observer-result.json" not in {Path(name).name for name in paths}
for name in paths:
    blob = git("show", ":" + name)
    working = (workspace / name).read_bytes()
    if name.startswith(evidence):
        assert hashlib.sha256(blob).digest() == hashlib.sha256(working).digest(), name
    else:
        assert blob.replace(b"\r\n", b"\n") == working.replace(b"\r\n", b"\n"), name
raw_text = [name for name in paths if name.startswith(evidence) and name.endswith(".original.txt")]
whitespace = subprocess.run(
    ["git", "-c", "core.longpaths=true",
     "-c", "core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol",
     "diff", "--cached", "--check", "--", ".",
     *[":(exclude)" + name for name in raw_text]], cwd=workspace, capture_output=True,
)
assert whitespace.returncode == 0, whitespace.stdout.decode(errors="replace")
print(json.dumps({
    "stagedFiles": len(paths),
    "evidenceIndexMatchesExactWorkingBytes": True,
    "documentTextMatchesAfterStandardCrLfNormalization": True,
    "codeFilesStaged": False,
    "originalUserDeletionStaged": False,
    "futureAdministratorExitResultStaged": False,
    "rawOriginalTextWhitespacePreservedAndExactBytesVerified": True,
    "whitespaceCheck": "passed",
}))
