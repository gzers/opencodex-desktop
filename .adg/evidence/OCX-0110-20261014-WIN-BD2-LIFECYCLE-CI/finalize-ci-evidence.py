"""Validate new-head terminal logs and separately update the existing draft PR."""
import gzip
import hashlib
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
merge = "16c781ea3e4abee9e7a7bf89efe38f3a24b187a9"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout.decode().strip()
assert not (root / "pr5-mutation-prepared.json").exists()
assert git("diff", "--name-only", base, source, "--", "apps", ".github", "Cargo.lock",
           "AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore") == ""
rows = []
required = ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]
for run_id in [38561752521, 38561759502]:
    terminal = read(f"ci-terminal-{run_id}.json")
    run = terminal["run"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == "success"
    jobs = {job["name"]: job for job in terminal["jobs"]["jobs"]}
    assert all(jobs[name]["conclusion"] == "success" for name in required)
    assert jobs["build"]["conclusion"] == "skipped"
    logs = read(f"ci-log-summary-{run_id}.json")
    assert logs["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7},
    }
    actual = {}
    for job in ["backend", "backend-windows"]:
        raw = gzip.decompress((root / f"ci-{run_id}-{job}-checkout.log.gz").read_bytes())
        text = raw.decode("utf-8-sig")
        actual[job] = re.findall(r"\b[0-9a-f]{40}\b", text[text.rfind("log -1 --format=%H"):])[-1]
    assert actual["backend"] == actual["backend-windows"]
    assert actual["backend"] == (source if run["event"] == "push" else merge)
    assert git("rev-parse", actual["backend"] + "^{tree}") == git("rev-parse", source + "^{tree}")
    rows.append({"runId": run_id, "event": run["event"], "head": source,
                 "actualCheckout": actual["backend"], "actualTreeEqualsHead": True,
                 "tree": git("rev-parse", source + "^{tree}"),
                 "conclusion": run["conclusion"], "status": run["status"],
                 "jobs": {name: jobs[name]["conclusion"] for name in required + ["build"]},
                 "rustCounts": logs["rustCounts"], "logArchiveSha256": logs["archiveSha256"],
                 "logArchiveBytes": logs["archiveBytes"],
                 "liveCollectorOrPureCSharpTestsRunByCi": False})
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "base": base,
    "ci": rows, "originalProductAndWorkflowUnchanged": True,
    "localPureChecksCoveredByPriorLifecycleEvidence": 53,
    "originalTestOrNodeExecutedThisStep": False, "administratorRecordingApproved": False,
    "etwRecorded": False, "originalFailureReproduced": False, "originalFailureClosed": False,
    "nativeInstanceStarted": False, "newCandidateOrManualCiDispatched": False,
    "goalScopeNarrowed": False, "fullVersionGatePromoted": False, "releaseAccepted": False,
}
(root / "ci-final-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
body = (root.parent / "OCX-0110-20261014-WIN-BD2-COLLECTOR-LIFECYCLE/pr-body.md").read_text(encoding="utf-8")
body = body.replace(
    "旧5fa双CI是旧helper的通用回归成功，不能替代新abc的CI或录制执行；新CI正在核验。",
    "abc3a91 push CI38561752521、PR CI38561759502均completed/success，每轮Mac900/Windows627通过、忽略6/7，各35组，五必需job成功/main-only build skipped。PR实际检出16c781ea整tree与head相同，检出和制品身份仍分别记录。这些CI不执行C#采集器或53项本机纯准备检查，不能替代管理员录制和真实事件覆盖。")
assert "新CI正在核验" not in body
(root / "pr5-final-body.md").write_text(body, encoding="utf-8")
prior = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credentials = subprocess.run(["git", "credential", "fill"], cwd=code,
        input="protocol=https\nhost=github.com\n\n", text=True, capture_output=True, check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credentials.stdout.splitlines() if "=" in line)["password"]
    def api(patch=None):
        config = ('url = "https://api.github.com/repos/gzers/opencodex-desktop/pulls/5"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n'
                  'header = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"]
        if patch is not None:
            payload = root / "pr5-patch-body.json"
            payload.write_text(json.dumps(patch, ensure_ascii=False), encoding="utf-8")
            args += ["--request", "PATCH", "--data-binary", "@" + str(payload)]
        else:
            args += ["--retry", "1", "--retry-all-errors", "--retry-delay", "2"]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=60)
        assert response.returncode == 0, "Read back any issued mutation before retry"
        return json.loads(response.stdout)
    before = api()
    assert before["head"]["sha"] == source and before["draft"] and before["state"] == "open"
    assert not before["merged"] and not before["auto_merge"]
    (root / "pr5-before-ci-writeback.json").write_text(
        json.dumps(before, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (root / "pr5-mutation-prepared.json").write_text(
        json.dumps({"head": source, "body": body, "action": "body-only; no merge/retarget/dispatch"},
                   ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    after = api({"body": body})
    assert after["head"]["sha"] == source and after["base"]["sha"] == before["base"]["sha"]
    assert after["state"] == "open" and after["draft"] and not after["merged"] and not after["auto_merge"]
    assert after["body"] == body and after["title"] == before["title"]
    (root / "pr5-after-ci-writeback.json").write_text(
        json.dumps(after, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
print(json.dumps({"newShaCiSuccessCount": 2, "actualTreesMatched": True,
                  "pr5StillDraft": True, "adminRecordingExecuted": False}))
