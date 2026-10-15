"""Bind this change and unchanged query deadlines to actual Git objects."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "bc14165ceb91cb084361e8dce23b10d381330e1a"
if "--final" in sys.argv:
    source = "c9f92ce7ed4002fa8fa6b1d5e8a62e8d4cea49a9"
if "--latest" in sys.argv:
    source = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
before = "3c51367f1be89abf66f69805d29b2a660a570a2c"

def git(*args):
    return subprocess.check_output(["git", *args], cwd=code)

assert git("rev-parse", "HEAD").decode().strip() == source and not git("status", "--porcelain").strip()
files = git("diff", "--name-only", before, source).decode().splitlines()
assert len(files) == (11 if "--latest" in sys.argv else 10)
assert all(path.startswith(("apps/", "test/")) for path in files)
rows = []
for path in files:
    data = git("show", source + ":" + path)
    rows.append({"path": path, "gitBlob": git("rev-parse", source + ":" + path).decode().strip(),
                 "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
unchanged = [
    "apps/desktop/tauri/src/infrastructure/official_version_source.rs",
    "apps/desktop/tauri/src/modules/about/remote.rs",
    "apps/desktop/tauri/src/modules/process/mod.rs",
    "apps/desktop/tauri/src/infrastructure/windows_query_job.rs",
    "apps/desktop/tauri/src/infrastructure/windows_query_fixture.rs",
]
for path in unchanged:
    assert git("rev-parse", before + ":" + path) == git("rev-parse", source + ":" + path)
shared = {}
for path in ["AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore"]:
    blobs = {ref: git("rev-parse", ref + ":" + path).decode().strip()
             for ref in [source, "origin/main", "origin/docs/governance-main"]}
    assert len(set(blobs.values())) == 1
    shared[path] = blobs
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "before": before,
    "applicationTree": git("rev-parse", source + ":apps/desktop").decode().strip(),
    "files": rows, "unchangedQueryFiles": unchanged, "sharedBlobs": shared,
    "scope": "Production recovery and diagnostic admission; no native acceptance claimed",
    "historicalFailureCleared": False, "fullVersionGatePassed": False,
}
target = root / ("final-source-identity.json" if "--final" in sys.argv else "source-identity.json")
if "--latest" in sys.argv:
    target = root / "a2e4ea17-source-identity.json"
assert not target.exists()
target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": source, "changedFiles": len(files), "sharedFilesMatch": True,
                  "unchangedQueryFiles": len(unchanged)}))
