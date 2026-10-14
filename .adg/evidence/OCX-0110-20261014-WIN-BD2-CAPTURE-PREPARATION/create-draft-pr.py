"""Create one draft for the prepared test-only collector; never merge or dispatch."""
import json
import os
import subprocess
import urllib.request
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
branch = "feature/0.1.10-windows-query-etw"
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
base = "feature/0.1.10-integration-candidate"
out = root / "draft-pr-created.json"
assert not out.exists()
prior = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n", text=True, capture_output=True,
        check=True, cwd=code, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]
    def request(path, body=None):
        payload = json.dumps(body, ensure_ascii=False).encode() if body is not None else None
        req = urllib.request.Request("https://api.github.com/repos/gzers/opencodex-desktop/" + path,
            data=payload, headers={"Authorization": "Bearer " + token,
                "Accept": "application/vnd.github+json", "Content-Type": "application/json",
                "User-Agent": "ocx-bd2-diagnostic-review"})
        with urllib.request.urlopen(req, timeout=25) as response:
            return json.load(response)
    before = request("pulls?state=open&head=gzers:" + branch)
    assert before == []
    actual_head = request("git/ref/heads/" + branch)
    actual_base = request("git/ref/heads/" + base)
    assert actual_head["object"]["sha"] == source
    assert actual_base["object"]["sha"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
    body = {"title": "test(windows): 自有进程实时ETW诊断准备",
            "head": branch, "base": base, "draft": True,
            "body": (root / "pr-body.md").read_text(encoding="utf-8")}
    prepared = root / "draft-pr-mutation-prepared.json"
    assert not prepared.exists(), "Read back any earlier mutation before repeating it"
    prepared.write_text(json.dumps({"body": body, "headSha": source,
                                   "status": "mutation-about-to-be-issued"},
                                  ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    created = request("pulls", body)
    assert created["draft"] and not created["merged"] and not created["auto_merge"]
    assert created["head"]["sha"] == source and created["base"]["ref"] == base
    out.write_text(json.dumps(created, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"draftPr": created["number"], "url": created["html_url"], "recordingStarted": False}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
