"""Admit verified exact-source CI only; updating its draft is not recording approval."""
import gzip
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "3a91c175a6c42d1cec16ef3eeb1d799d9abe4aac"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
required = ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))


def save(name, value):
    path = root / name
    assert not path.exists(), "Inspect existing evidence instead of overwriting or repeating a mutation."
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout.decode().strip()


assert git("rev-parse", "HEAD") == source and git("status", "--porcelain") == ""
assert git("diff", "--name-only", base, source, "--", "apps", ".github", "Cargo.lock",
           "AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore") == ""
assert not (root / "pr5-mutation-prepared.json").exists()
tree = git("rev-parse", source + "^{tree}")
rows = []
for run_id, event in ((38572890687, "push"), (38572902717, "pull_request")):
    terminal = read(f"ci-terminal-{run_id}.json")
    run = terminal["run"]
    assert run["head_sha"] == source and run["event"] == event
    assert run["status"] == "completed" and run["conclusion"] == "success"
    jobs = {job["name"]: job for job in terminal["jobs"]["jobs"]}
    assert all(jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success"
               for name in required)
    assert jobs["build"]["conclusion"] == "skipped"
    summary = read(f"ci-log-summary-{run_id}.json")
    assert summary["zipCrcPassed"] and summary["safeEntries"] and not summary["failedTestLines"]
    assert summary["rustCounts"] == {
        "backend": {"suites": 35, "passed": 900, "failed": 0, "ignored": 6},
        "backend-windows": {"suites": 35, "passed": 627, "failed": 0, "ignored": 7}}
    actual = {}
    for job in ("backend", "backend-windows"):
        text = gzip.decompress((root / f"ci-{run_id}-{job}-checkout.log.gz").read_bytes()).decode("utf-8-sig")
        tail = text[text.rfind("log -1 --format=%H"):]
        candidates = re.findall(r"\b[0-9a-f]{40}\b", tail)
        assert len(candidates) == 1, "Actual checkout must be unambiguous in the original checkout step."
        actual[job] = candidates[0]
    assert actual["backend"] == actual["backend-windows"]
    if event == "push":
        assert actual["backend"] == source
    rows.append({"runId": run_id, "event": event, "head": source, "status": "completed",
                 "conclusion": "success", "actualCheckout": actual["backend"],
                 "jobs": {name: jobs[name]["conclusion"] for name in required + ["build"]},
                 "rustCounts": summary["rustCounts"], "archiveBytes": summary["archiveBytes"],
                 "archiveSha256": summary["archiveSha256"], "originalLogsVerified": True})

prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, capture_output=True,
                                text=True, input="protocol=https\nhost=github.com\n\n",
                                timeout=20, check=True)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def request(path, patch=None):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n'
                  'header = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"]
        if patch is None:
            args += ["--retry", "1", "--retry-all-errors", "--retry-delay", "2"]
        else:
            save("pr5-patch-payload.json", patch)
            args += ["--request", "PATCH", "--data-binary", "@" + str(root / "pr5-patch-payload.json")]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=60)
        if response.returncode:
            raise RuntimeError("Transport unavailable; read actual PR before repeating any issued mutation.")
        return json.loads(response.stdout)

    for sha in sorted({source, *(row["actualCheckout"] for row in rows)}):
        commit = request(f"git/commits/{sha}")
        assert commit["sha"] == sha and commit["tree"]["sha"] == tree
        save(f"actual-commit-{sha}.json", commit)
    for row in rows:
        row["actualTree"] = tree
        row["actualTreeEqualsHead"] = True
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "base": base,
              "runs": rows, "productAndWorkflowUnchanged": True, "collectorLiveRecordTested": False,
              "administratorRecordingApproved": True, "standingRecordingAuthority": True, "newOriginalLibraryRuns": 0,
              "originalFailureReproduced": False, "originalFailureClosed": False,
              "nativeInstanceStarted": False, "candidateOrCiManuallyDispatched": False,
              "fullVersionGatePromoted": False}
    save("ci-final-result.json", result)
    save("ci-admission.json", result)

    before = request("pulls/5")
    assert before["head"]["sha"] == source and before["base"]["sha"] == base
    assert before["state"] == "open" and before["draft"] and not before["merged"] and not before["auto_merge"]
    old_body = (root.parent / "OCX-0110-20261014-WIN-BD2-ETW-IMAGEFIX-RUN/pr5-body.md").read_text(encoding="utf-8")
    assert before["body"] == old_body
    save("pr5-before-ci-writeback.json", before)
    body = old_body.replace(
        "新head双CI和修正后真实采集均以实际新证据为准，旧绿不抵新head。",
        "3a91c175 的 push38572890687/PR38572902717 均 completed/success，五必需 job 成功、main-only build skipped；"
        "每轮 Mac900/Windows627 passed、忽略6/7，各35组。完整 ZIP 的 SHA/CRC、安全条目、原 checkout/测试日志已核验，"
        "实际PR合成检出与head整树一致，分别保留身份。通用CI不执行C#collector，修正后真实录制仍待单次有界执行。")
    assert body != old_body
    (root / "pr5-final-body.md").write_text(body, encoding="utf-8")
    save("pr5-mutation-prepared.json", {"head": source, "action": "body-only; no merge, retarget, or dispatch",
                                      "body": body})
    save("pr5-patch-response.json", request("pulls/5", {"body": body}))
    after = request("pulls/5")
    assert after["head"]["sha"] == source and after["base"]["sha"] == base and after["title"] == before["title"]
    assert after["body"] == body and after["state"] == "open" and after["draft"]
    assert not after["merged"] and not after["auto_merge"]
    save("pr5-after-ci-readback.json", after)
    print(json.dumps({"source": source, "verifiedCiSuccessCount": 2, "actualTree": tree,
                      "actualCheckouts": [row["actualCheckout"] for row in rows],
                      "draftReadbackVerified": True, "administratorRecordingApproved": True, "standingRecordingAuthority": True}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
