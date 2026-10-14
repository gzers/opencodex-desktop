"""One description update plus read-only exact-head CI observation."""
import json
import os
import subprocess
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "abc3a9140fc6f1b5d2aa6e739299af260f0f5d94"
base_url = "https://api.github.com/repos/gzers/opencodex-desktop/"
prior = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n", text=True, capture_output=True,
        cwd=code, check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
    def request(path, patch=None):
        config = (f'url = "{base_url + path}"\nheader = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n'
                  'header = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "25", "--fail", "--silent", "--show-error"]
        if patch is None:
            args += ["--retry", "1", "--retry-all-errors", "--retry-delay", "2"]
        else:
            payload = root / "pr5-patch-payload.json"
            payload.write_text(json.dumps(patch, ensure_ascii=False), encoding="utf-8")
            args += ["--request", "PATCH", "--data-binary", "@" + str(payload)]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=60)
        if response.returncode:
            raise RuntimeError("GitHub transport failed; read back any issued mutation before retry")
        return json.loads(response.stdout)
    before = request("pulls/5")
    assert before["head"]["sha"] == source and before["state"] == "open" and before["draft"]
    assert not before["merged"] and not before["auto_merge"]
    (root / "pr5-before-description-update.json").write_text(
        json.dumps(before, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    patch = {"body": (root / "pr-body.md").read_text(encoding="utf-8"),
             "title": "test(windows): 修正实时ETW启动与自有会话收尾"}
    guard = root / "pr5-mutation-prepared.json"
    assert not guard.exists(), "Read actual PR before repeating an uncertain mutation"
    guard.write_text(json.dumps({"head": source, "patch": patch,
                                "action": "one title/body update, no merge or dispatch"},
                               ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    after = request("pulls/5", patch)
    assert after["head"]["sha"] == source and after["base"]["sha"] == before["base"]["sha"]
    assert after["draft"] and not after["merged"] and not after["auto_merge"]
    assert after["body"] == patch["body"] and after["title"] == patch["title"]
    (root / "pr5-after-description-update.json").write_text(
        json.dumps(after, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    observation = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
                   "manuallyDispatched": False, "recordingExecuted": False, "runs": []}
    try:
        for run in request(f"actions/runs?head_sha={source}&per_page=30")["workflow_runs"]:
            assert run["head_sha"] == source
            observation["runs"].append({"run": run, "jobs": request(f"actions/runs/{run['id']}/jobs?per_page=100")})
        observation["status"] = "observed"
    except Exception as error:
        observation.update(status="ci-read-unavailable", errorType=type(error).__name__)
    (root / "new-sha-ci-observation.json").write_text(
        json.dumps(observation, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"pr5DescriptionUpdated": True, "prStillDraft": True,
                      "ciStatus": observation["status"],
                      "runs": [{"id": item["run"]["id"], "status": item["run"]["status"],
                                "conclusion": item["run"]["conclusion"]} for item in observation["runs"]],
                      "recordingExecuted": False}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
