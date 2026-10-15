"""Read exact blobs for the installation-location binding fix."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
before = "a2e4ea1786723ceab0a1ed0d322503ac79cd64e0"
path = "apps/desktop/tauri/src/infrastructure/windows_startup_root.rs"
git = lambda *args: subprocess.check_output(["git", *args], cwd=code)
assert git("rev-parse", "HEAD").decode().strip() == source
assert not git("status", "--porcelain").strip()
assert git("diff", "--name-only", before, source).decode().splitlines() == [path]
data = git("show", source + ":" + path)
preserved = [
    "apps/desktop/tauri/src/infrastructure/official_version_source.rs",
    "apps/desktop/tauri/src/modules/about/remote.rs",
    "apps/desktop/tauri/src/infrastructure/windows_query_fixture.rs",
    "apps/desktop/tauri/src/infrastructure/windows_query_job.rs",
    "apps/desktop/tauri/src/modules/process/mod.rs",
    "apps/desktop/tauri/src/modules/ipc/audit.rs",
    "apps/desktop/tauri/src/lib.rs",
    ".github/workflows/ci.yml",
    "test/smoke/windows-owned-etw.cs",
]
for item in preserved:
    assert git("rev-parse", before + ":" + item) == git("rev-parse", source + ":" + item)
shared = {}
for item in ["AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore"]:
    blobs = {ref: git("rev-parse", ref + ":" + item).decode().strip()
             for ref in [source, "origin/main", "origin/docs/governance-main"]}
    assert len(set(blobs.values())) == 1
    shared[item] = blobs
record = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "before": before,
    "changedFile": {"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
                    "gitBlob": git("rev-parse", source + ":" + path).decode().strip()},
    "applicationTree": git("rev-parse", source + ":apps/desktop").decode().strip(),
    "unchangedFiles": preserved, "sharedFiles": shared,
    "contract": "IMP-20 section 4: installation location changes must not silently change an existing root",
    "nativeInstallRelocationOrRegistryProbeExecuted": False,
    "fullVersionGatePassed": False,
}
target = root / "source-identity.json"
assert not target.exists()
target.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": source, "changedProductFiles": 1, "unchangedFilesVerified": len(preserved),
                  "sharedFilesMatch": True}))
