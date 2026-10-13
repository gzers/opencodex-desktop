"""Check committed diagnostics against their parent without compiling or running product code."""
import json
import re
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
before = "d975b2351920bc027c931c8135b5605fde7a46c2"
after = "be7395acfce0ec1e36b83300b576db408f8c35bb"
def blob(commit, path):
    return subprocess.check_output(["git", "-C", str(code), "show", commit + ":" + path]).decode()
def identity(commit, path):
    return subprocess.check_output(["git", "-C", str(code), "rev-parse", commit + ":" + path]).decode().strip()

official_path = "apps/desktop/tauri/src/infrastructure/official_version_source.rs"
original = blob(before, official_path)
diagnostic = blob(after, official_path)
pattern = r'\n[ ]+#\[cfg\(all\(test, windows\)\)\]\n[ ]+eprintln!\(\n(?:[^\n]*\n)*?[ ]+\);'
release_source, count = re.subn(pattern, "", diagnostic)
assert count == 4 and release_source == original
assert diagnostic.count('official_version_timeout().saturating_sub(started.elapsed())') == 1
fixture_path = "apps/desktop/tauri/src/infrastructure/windows_query_fixture.rs"
fixture = blob(after, fixture_path)
module = blob(after, "apps/desktop/tauri/src/infrastructure/mod.rs")
assert re.search(r'#\[cfg\(all\(windows, test\)\)\]\s+pub\(crate\) mod windows_query_fixture;', module)
assert 'std::time::Duration::from_secs(5)' in fixture
assert 'std::time::Duration::from_secs(4)' in fixture
assert 'fixture descendant did not start: root={}' in fixture
workflow_path = ".github/workflows/ci.yml"
workflow = blob(after, workflow_path)
assert workflow.count("cargo test --locked --workspace --features integration-test") == 2
assert "RUST_TEST_THREADS =" not in workflow and "--test-threads" not in workflow
assert not re.search(r'^\s*(concurrency|continue-on-error)\s*:', workflow, flags=re.M)
record = {
    "result": "passed", "parentSourceCommit": before, "diagnosticSourceCommit": after,
    "officialVersionBlobs": {"before": identity(before, official_path), "after": identity(after, official_path)},
    "testOnlyTraceBlocks": count, "officialReleaseSourceEqualsParentAfterCfgRemoval": True,
    "fixtureModuleIsTestWindowsOnly": True, "fixtureObserverSeconds": 5,
    "originalCleanupBoundSeconds": 4, "productionTimeoutExpressionUnchanged": True,
    "ciTestCommandAndConcurrencyUnchanged": True,
    "scope": "committed source/config guard checks, not Rust compilation or CI acceptance",
    "localRustCompilationAccepted": False, "originalCiFailureCleared": False,
}
(root / "source-check-result.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(record))
