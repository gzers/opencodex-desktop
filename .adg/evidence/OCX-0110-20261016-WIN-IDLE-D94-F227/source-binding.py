"""Bind existing Git objects and working bytes without changing a checkout."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
DOCS = ROOT.parents[2]
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
DIAG = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")
assert not (ROOT / "manifest.json").exists()
assert not (ROOT / "source-binding.json").exists()


def git(repo, *args):
    return subprocess.run(["git", "-c", "core.longpaths=true", "-C", str(repo), *args],
                          check=True, capture_output=True).stdout


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


contract = read("contract.json")
assert git(DIAG, "rev-parse", "HEAD").decode().strip() == contract["collectorSource"]
assert git(CODE, "rev-parse", "HEAD").decode().strip() == contract["candidateSource"]
assert not git(DIAG, "status", "--porcelain") and not git(CODE, "status", "--porcelain")
method_bindings = {}
for path, expected in contract["methodWorkingFileSha256"].items():
    blob = git(DIAG, "show", contract["collectorSource"] + ":" + path)
    working = (DIAG / path).read_bytes()
    assert sha(working) == expected
    assert working.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n")
    method_bindings[path] = {
        "gitBlobSha256": sha(blob), "workingFileSha256": sha(working),
        "identicalExceptCrLf": True,
    }
shared = {}
for name in ("AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore"):
    records = {}
    blobs = []
    for label, repo in (("docs", DOCS), ("product", CODE), ("diagnostic", DIAG)):
        blob = git(repo, "show", "HEAD:" + name)
        working = (repo / name).read_bytes()
        assert working.replace(b"\r\n", b"\n") == blob.replace(b"\r\n", b"\n")
        blobs.append(blob)
        records[label] = {"gitBlobSha256": sha(blob), "workingFileSha256": sha(working)}
    assert len({sha(blob) for blob in blobs}) == 1
    shared[name] = {"gitObjectsIdentical": True, "workingContentMatchesGitExceptCrLf": True,
                    "workingBytesIdentical": len({v["workingFileSha256"] for v in records.values()}) == 1,
                    "records": records}
apps = git(DIAG, "rev-parse", "HEAD:apps").decode().strip()
assert apps == git(CODE, "rev-parse", "HEAD:apps").decode().strip()
workflow = git(DIAG, "show", "HEAD:.github/workflows/ci.yml").decode()
assert "  build:\n    if: github.ref == 'refs/heads/main'\n" in workflow
remote_output = git(DOCS, "-c", "http.sslBackend=schannel", "-c", "http.version=HTTP/1.1",
                    "ls-remote", "origin", "refs/heads/main", "refs/heads/docs/governance-main",
                    "refs/heads/feature/0.1.10-windows-query-etw",
                    "refs/heads/feature/0.1.10-windows-etw-query-threads").decode()
remote = dict((ref, value) for value, ref in (line.split() for line in remote_output.splitlines()))
assert remote["refs/heads/main"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert remote["refs/heads/docs/governance-main"] == "401e0847a168d79b18c401d1ccd2b2c70366ba35"
assert remote["refs/heads/feature/0.1.10-windows-query-etw"] == contract["candidateSource"]
assert remote["refs/heads/feature/0.1.10-windows-etw-query-threads"] == contract["collectorSource"]
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "remoteHeads": remote,
    "methods": method_bindings, "sharedFiles": shared,
    "sharedContentIdentical": True, "readmeWorkingByteDifferenceIsCrLfOnly": True,
    "appsTreeIdentical": True, "appsTree": apps, "bothCodeWorktreesClean": True,
    "expectedBuildSkipCheckedInWorkflow": True,
    "sourceChanged": False, "productStarted": False, "foregroundActivated": False,
}
(ROOT / "source-binding.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in summary.items() if k not in {"methods", "sharedFiles"}}))
