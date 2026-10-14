"""Bind known query failures for diagnosis; never create a green CI/release admission."""
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
source = "3a91c175a6c42d1cec16ef3eeb1d799d9abe4aac"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
expected_failed = sorted([
    "infrastructure::official_version_source::windows_tests::version_nonzero_exit_reaps_owned_helper",
    "infrastructure::official_version_source::windows_tests::version_success_reaps_owned_helper_and_preserves_version_output",
    "infrastructure::official_version_source::windows_tests::version_timeout_cleans_owned_tree_and_preserves_unrelated_process",
])


def save(name, value):
    path = root / name
    assert not path.exists(), "Read existing evidence before repeating any mutation."
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout.decode().strip()


assert git("rev-parse", "HEAD") == source and not git("status", "--porcelain")
assert not git("diff", "--name-only", base, source, "--", "apps", ".github", "Cargo.lock",
               "AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore")
tree = git("rev-parse", source + "^{tree}")
prior_preparation = root.parent / "OCX-0110-20261014-WIN-BD2-ETW-IMAGEFIX-RUN"
prepare_path = prior_preparation / "volume-fix-prepare.stdout.original.txt"
prepare = json.loads(prepare_path.read_text(encoding="utf-8-sig"))
assert prepare["status"] == "passed"
assert sum(prepare[key] for key in ("pureScopeChecks", "pureLifecycleChecks", "pureNativeStopChecks",
                                  "controllerProcessImageChecks", "descendantHotScopeChecks", "utf8LineBudgetChecks")) == 78
assert not prepare["etwSessionStarted"] and not prepare["testOrNodeStarted"]
assert json.loads((prior_preparation / "volume-fix-boundary.json").read_text(encoding="utf-8-sig"))["source"] == source
rows = []
for run_id, event, conclusion in ((38572890687, "push", "failure"), (38572902717, "pull_request", "success")):
    terminal = read(f"ci-terminal-{run_id}.json")
    run = terminal["run"]
    assert run["head_sha"] == source and run["event"] == event
    assert run["status"] == "completed" and run["conclusion"] == conclusion
    jobs = {job["name"]: job for job in terminal["jobs"]["jobs"]}
    for name in ("frontend", "backend", "release-tools", "windows-smoke-evidence"):
        assert jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success"
    assert jobs["backend-windows"]["conclusion"] == conclusion
    assert jobs["build"]["conclusion"] == "skipped"
    summary = read(f"ci-log-summary-{run_id}.json")
    assert summary["zipCrcPassed"] and summary["safeEntries"]
    failed = sorted(re.search(r"test (\S+) \.\.\. FAILED", line).group(1)
                    for line in summary["failedTestLines"])
    assert failed == (expected_failed if conclusion == "failure" else [])
    assert summary["rustCounts"]["backend"] == {"suites": 35, "passed": 900, "failed": 0, "ignored": 6}
    assert summary["rustCounts"]["backend-windows"] == (
        {"suites": 1, "passed": 527, "failed": 3, "ignored": 4} if conclusion == "failure"
        else {"suites": 35, "passed": 627, "failed": 0, "ignored": 7})
    assert not summary["collectorRecordOrScopeTestsExecutedByCi"]
    actual = {}
    for job in ("backend", "backend-windows"):
        text = gzip.decompress((root / f"ci-{run_id}-{job}-checkout.log.gz").read_bytes()).decode("utf-8-sig")
        shas = re.findall(r"\b[0-9a-f]{40}\b", text[text.rfind("log -1 --format=%H"):])
        assert len(shas) == 1
        actual[job] = shas[0]
    assert actual["backend"] == actual["backend-windows"]
    if event == "push":
        assert actual["backend"] == source
    rows.append({"runId": run_id, "event": event, "head": source, "status": "completed",
                 "conclusion": conclusion, "actualCheckout": actual["backend"],
                 "originalLogsVerified": True, "failedTests": failed, "unexpectedFailedTests": [],
                 "failureClass": "unmodified-original-version-query-fixtures" if failed else None,
                 "rustCounts": summary["rustCounts"], "archiveBytes": summary["archiveBytes"],
                 "archiveSha256": summary["archiveSha256"],
                 "jobs": {name: value["conclusion"] for name, value in jobs.items()}})

text = gzip.decompress((root / "ci-38572890687-backend-windows-tests.log.gz").read_bytes()).decode("utf-8-sig")
snapshots = []
for match in re.finditer(r"query fixture version/([a-z]+): control-snapshot=(\{[^\r\n]+\})", text):
    snapshots.append({"mode": match.group(1), "snapshot": json.loads(match.group(2))})
assert len(snapshots) == 9
markers = {(match.group(1), match.group(2)): match.group(3)
           for match in re.finditer(
               r"query fixture version/([a-z]+): control-marker-at=([a-z-]+), evidence=([^\r\n]+)", text)}
assert len(markers) == 9
assert all(markers[(mode, "query-returned")].startswith("Err(") for mode in ("nonzero", "success", "timeout"))
assert all(markers[(mode, "observer-returned")].startswith("Ok(") for mode in ("nonzero", "success", "timeout"))
assert all(row["snapshot"]["wait_status"] == 258 and row["snapshot"]["exit_code"] == 259
           and row["snapshot"]["process_times"]["exit_filetime_100ns"] == 0 for row in snapshots)
save("failed-control-snapshots.json", {
    "runId": 38572890687, "snapshots": snapshots,
    "markerObservations": [{"mode": mode, "point": point, "evidence": value}
                           for (mode, point), value in sorted(markers.items())],
    "nodeVersion": "24.21.0", "controlMarkersMissingAtQueryReturn": True,
    "controlMarkersPresentAtObserverReturn": True, "rawSchedulerOrFileIoEtwAvailable": False,
    "rootCauseClaimed": False, "consoleInspectionEncodingFailure": "An initial GBK stdout print failed; frozen UTF-8 log bytes were unchanged.",
})
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], cwd=code, text=True, capture_output=True,
                                input="protocol=https\nhost=github.com\n\n", check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def request(path, patch=None):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\nheader = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"]
        if patch is None:
            args += ["--retry", "1", "--retry-all-errors", "--retry-delay", "2"]
        else:
            save("pr5-patch-payload.json", patch)
            args += ["--request", "PATCH", "--data-binary", "@" + str(root / "pr5-patch-payload.json")]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=60)
        if response.returncode:
            raise RuntimeError("Read back actual PR before repeating any issued mutation.")
        return json.loads(response.stdout)

    for sha in sorted({source, *(row["actualCheckout"] for row in rows)}):
        commit = request(f"git/commits/{sha}")
        assert commit["sha"] == sha and commit["tree"]["sha"] == tree
        save(f"actual-commit-{sha}.json", commit)
    for row in rows:
        row["actualTree"] = tree
        row["actualTreeEqualsHead"] = True
    result = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "base": base, "runs": rows,
              "diagnosticOnly": True, "releaseAdmission": False, "allCiPassed": False,
              "standingUserAuthorization": "批准修正版后续都可以管理员后台 ETW 录制",
              "collectorLiveRecordTested": False, "collectorPreparationChecks": 78,
              "preparationEvidenceSha256": hashlib.sha256(prepare_path.read_bytes()).hexdigest(),
              "productFixtureWorkflowSharedFilesUnchanged": True, "originalBudgetsAndAssertionsUnchanged": True,
              "newLocalOriginalLibraryRuns": 0, "originalFailureClosed": False, "newCiFailureRetained": True,
              "fullVersionGatePromoted": False, "localEtwStillRequiredForActualFailure": True,
              "reason": "The collector is not executed by CI. Diagnose known unmodified fixture failures; no release or version gate is passed.",
              "greenOnlyHelperNotExecuted": True}
    save("ci-final-result.json", result)
    save("diagnostic-admission.json", result)
    before = request("pulls/5")
    old_body = (prior_preparation / "pr5-body.md").read_text(encoding="utf-8")
    assert before["head"]["sha"] == source and before["base"]["sha"] == base and before["body"] == old_body
    assert before["state"] == "open" and before["draft"] and not before["merged"] and not before["auto_merge"]
    save("pr5-before-ci-writeback.json", before)
    body = old_body.replace(
        "新head双CI和修正后真实采集均以实际新证据为准，旧绿不抵新head。",
        "3a91c175 的 push38572890687 completed/failure：Windows库527passed/三个version查询夹具失败/4ignored；"
        "PR38572902717 completed/success，Windows35组627passed/7ignored。两轮Mac各900passed/6ignored。"
        "push控制进程在约2秒query返回仍标记缺失，约5秒仍active、标记已出现；无调度/I/O根因证据。"
        "完整日志ZIP/SHA/CRC与实际检出树核验，原失败及本次失败均保留，PR绿不解除push红。"
        "通用CI未运行collector；仅在原夹具/预算/产品字节不变、78准备检查通过、失败精确限定并保存后作诊断准入，"
        "不是版本/发布准入。旧双绿辅助入口不执行，修正后实时采集仍待验证。")
    assert body != old_body
    (root / "pr5-final-body.md").write_text(body, encoding="utf-8")
    save("pr5-mutation-prepared.json", {"head": source, "action": "body-only; failure retained", "body": body})
    save("pr5-patch-response.json", request("pulls/5", {"body": body}))
    after = request("pulls/5")
    assert after["head"]["sha"] == source and after["base"]["sha"] == base
    assert after["title"] == before["title"] and after["body"] == body
    assert after["state"] == "open" and after["draft"] and not after["merged"] and not after["auto_merge"]
    save("pr5-after-ci-readback.json", after)
    print(json.dumps({"source": source, "actualTree": tree, "allCiPassed": False,
                      "diagnosticOnly": True, "failureRetained": True,
                      "actualCheckouts": [row["actualCheckout"] for row in rows], "draftReadbackVerified": True}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
