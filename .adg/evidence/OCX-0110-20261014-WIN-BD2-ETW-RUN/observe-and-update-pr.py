"""Update this task's draft description once, then inspect only the exact head CI."""
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "629b9c6f913b5e04badc9ce747f6bb8ba247fd89"
base = "bd2f6eb10d58115235e35d575507bb69f44666da"
api = "https://api.github.com/repos/gzers/opencodex-desktop/"
prior = {key: os.environ.get(key) for key in ("GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE")}


def save(name, value):
    path = root / name
    assert not path.exists(), "Keep existing evidence; inspect instead of overwriting."
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def relevant_pr(value):
    return {key: value[key] for key in
            ("number", "title", "body", "state", "draft", "merged", "auto_merge", "head", "base")}


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(
        ["git", "credential", "fill"], cwd=code, capture_output=True, text=True,
        input="protocol=https\nhost=github.com\n\n", timeout=20, check=True)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def request(path, patch=None):
        config = (f'url = "{api + path}"\nheader = "Authorization: Bearer {token}"\n'
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
            raise RuntimeError("GitHub transport unavailable; inspect actual PR before repeating any mutation.")
        return json.loads(response.stdout)

    before = relevant_pr(request("pulls/5"))
    assert before["head"]["sha"] == source and before["base"]["sha"] == base
    assert before["state"] == "open" and before["draft"] and not before["merged"] and not before["auto_merge"]
    save("pr5-before-update.json", before)
    patch = {"title": "test(windows): 修复ETW根进程校验与异常收尾",
             "body": (root / "pr5-body.md").read_text(encoding="utf-8")}
    save("pr5-mutation-prepared.json", {"source": source, "base": base, "patch": patch,
                                      "mergeOrDispatchRequested": False})
    returned = relevant_pr(request("pulls/5", patch))
    save("pr5-patch-response.json", returned)
    after = relevant_pr(request("pulls/5"))
    assert after["head"]["sha"] == source and after["base"]["sha"] == base
    assert after["title"] == patch["title"] and after["body"] == patch["body"]
    assert after["state"] == "open" and after["draft"] and not after["merged"] and not after["auto_merge"]
    save("pr5-after-readback.json", after)

    observation = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
                   "manuallyDispatched": False, "newOriginalLibraryRuns": 0, "runs": []}
    try:
        for run in request(f"actions/runs?head_sha={source}&per_page=30")["workflow_runs"]:
            assert run["head_sha"] == source
            observation["runs"].append({"run": run, "jobs": request(f"actions/runs/{run['id']}/jobs?per_page=100")})
        observation["status"] = "observed"
    except Exception as error:
        observation.update(status="ci-read-unavailable", errorType=type(error).__name__)
    save("imagefix-ci-initial.json", observation)
    print(json.dumps({"source": source, "draftReadbackVerified": True,
                      "runs": [{"id": row["run"]["id"], "event": row["run"]["event"],
                                "status": row["run"]["status"], "conclusion": row["run"]["conclusion"]}
                               for row in observation["runs"]]}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
