"""Use the official generated experimental protocol for the human-requested goal edit."""
import json
import queue
import subprocess
import threading
import time
from pathlib import Path

root = Path(__file__).resolve().parent
output = root / "goal-user-edit-result.json"
assert not output.exists(), "Do not repeat or overwrite a goal edit"
thread_id = "01a12e0e-c38f-7df3-b42f-6e80f629545e"
objective = (root / "goal-objective.txt").read_text(encoding="utf-8").strip()
assert "0.1.10" in objective and "TASK-OPENCODEX-DESKTOP-182" in objective and "Mac" in objective
schema_path = Path(r"C:\Users\15119\.codex\tmp\ocx-goal-schema-path.json")
schema_root = Path(json.loads(schema_path.read_text(encoding="utf-8-sig"))["generatedDirectory"])
schema = json.loads((schema_root / "v2/ThreadGoalSetParams.json").read_text(encoding="utf-8"))
assert {"objective", "status", "origin", "threadId"} <= set(schema["properties"])
assert "user" in schema["definitions"]["ThreadGoalMutationOrigin"]["enum"]
assert "active" in schema["definitions"]["ThreadGoalStatus"]["enum"]
cli = Path(r"C:\Users\15119\AppData\Local\OpenAI\Codex\bin\9691020b546a15b2\codex.exe")
process = subprocess.Popen(
    [str(cli), "app-server", "--stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.PIPE, text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW,
)
responses = queue.Queue()
def receive():
    for line in process.stdout:
        try:
            responses.put(json.loads(line))
        except json.JSONDecodeError:
            pass
threading.Thread(target=receive, daemon=True).start()
def request(message, timeout=15):
    process.stdin.write(json.dumps(message, ensure_ascii=False) + "\n")
    process.stdin.flush()
    until = time.monotonic() + timeout
    while time.monotonic() < until:
        try:
            item = responses.get(timeout=min(1, max(0.01, until - time.monotonic())))
        except queue.Empty:
            if process.poll() is not None:
                raise RuntimeError("Official stdio server exited before response")
            continue
        if item.get("id") == message["id"]:
            if "error" in item:
                raise RuntimeError(json.dumps(item["error"], ensure_ascii=True))
            return item["result"]
    raise TimeoutError("Official stdio response timeout; re-read this same goal before any retry")
record = {
    "userAuthorization": "还有windows修改本对话的目标模式，继续开工",
    "threadId": thread_id, "protocol": "official CLI app-server stdio, generated experimental thread/goal/set",
    "goalMutationIssued": False, "newGoalCreated": False, "goalMarkedComplete": False,
    "tokenBudgetChanged": False, "nativeUiInputIssued": False, "daemonOrDesktopAppRestarted": False,
    "status": "preparing",
}
try:
    request({"id": 1, "method": "initialize", "params": {
        "clientInfo": {"name": "ocx_windows_user_goal_update", "version": "1.0.0"},
        "capabilities": {"experimentalApi": True},
    }})
    process.stdin.write(json.dumps({"method": "initialized"}) + "\n")
    process.stdin.flush()
    before = request({"id": 2, "method": "thread/goal/get", "params": {"threadId": thread_id}})["goal"]
    assert before is not None and before["threadId"] == thread_id and before["status"] == "paused"
    assert before["createdAt"] == 1791774103 and before["tokensUsed"] >= 18453728
    assert before["objective"].startswith("继续 OpenCodex Desktop 0.1.10")
    record["before"] = before
    record["status"] = "mutation-prepared"
    output.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    record["goalMutationIssued"] = True
    after = request({"id": 3, "method": "thread/goal/set", "params": {
        "threadId": thread_id, "objective": objective, "status": "active", "origin": "user",
    }})["goal"]
    record["setResponse"] = after
    readback = request({"id": 4, "method": "thread/goal/get", "params": {"threadId": thread_id}})["goal"]
    record["readback"] = readback
    assert readback["objective"] == objective and readback["status"] == "active"
    assert readback["createdAt"] == before["createdAt"]
    assert readback["tokensUsed"] >= before["tokensUsed"] and readback["timeUsedSeconds"] >= before["timeUsedSeconds"]
    assert readback.get("tokenBudget") == before.get("tokenBudget")
    record["goalObjectiveUpdated"] = True
    record["goalStatusResumed"] = True
    record["originalUsageAccountingPreserved"] = True
    record["status"] = "exact-user-requested-goal-edit-readback-verified"
except Exception as error:
    record["status"] = "error-requires-readback-before-retry"
    record["error"] = str(error)
    raise
finally:
    output.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if process.poll() is None:
        process.kill()
    process.wait(timeout=5)
print(json.dumps({
    "goalUpdated": record.get("goalObjectiveUpdated", False),
    "status": record.get("readback", {}).get("status"), "createdAtPreserved": True,
    "tokensUsed": record.get("readback", {}).get("tokensUsed"),
    "originalUsageAccountingPreserved": record.get("originalUsageAccountingPreserved", False),
    "nativeUiInputIssued": False,
}, ensure_ascii=True))
