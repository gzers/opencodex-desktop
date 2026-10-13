"""Single official status-only resume; do not replace the objective or reset usage."""
import json
import queue
import subprocess
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
out = root / "status-only-resume.json"
assert not out.exists(), "No duplicate goal mutation"
thread_id = "01a12e0e-c38f-7df3-b42f-6e80f629545e"
cli = Path(r"C:\Users\15119\AppData\Local\OpenAI\Codex\bin\9691020b546a15b2\codex.exe")
process = subprocess.Popen(
    [str(cli), "app-server", "--stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.PIPE, text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW,
)
messages = queue.Queue()


def receive():
    for line in process.stdout:
        try:
            messages.put(json.loads(line))
        except json.JSONDecodeError:
            pass


threading.Thread(target=receive, daemon=True).start()
request_id = 0


def request(method, params):
    global request_id
    request_id += 1
    process.stdin.write(json.dumps({"id": request_id, "method": method, "params": params}) + "\n")
    process.stdin.flush()
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            message = messages.get(timeout=min(1, max(0.01, deadline - time.monotonic())))
        except queue.Empty:
            if process.poll() is not None:
                raise RuntimeError("Owned stdio connection ended")
            continue
        if message.get("id") == request_id:
            return message
    raise TimeoutError("No response; read the goal before any new action")


record = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "userAuthorization": "那么你把Windows能做的继续修改本对话目标模式完成为止",
    "mutationParameters": {"threadId": thread_id, "status": "active", "origin": "user"},
    "objectiveReplacementIssued": False, "budgetChangeIssued": False,
    "writerLockOverrideIssued": False, "retryLoopStarted": False,
    "goalClearedOrMarkedComplete": False, "nativeInputIssued": False,
}
try:
    init = request("initialize", {
        "clientInfo": {"name": "ocx_windows_status_only_user_resume", "version": "1.0.0"},
        "capabilities": {"experimentalApi": True},
    })
    assert "error" not in init, init
    process.stdin.write(json.dumps({"method": "initialized"}) + "\n")
    process.stdin.flush()
    before = request("thread/goal/get", {"threadId": thread_id})
    assert "error" not in before, before
    record["before"] = before["result"]["goal"]
    assert record["before"]["createdAt"] == 1791774103
    assert record["before"]["status"] == "paused"
    record["setResponse"] = request("thread/goal/set", record["mutationParameters"])
    actual = request("thread/goal/get", {"threadId": thread_id})
    assert "error" not in actual, actual
    record["readback"] = actual["result"]["goal"]
    original, readback = record["before"], record["readback"]
    record["originalAccountingPreserved"] = (
        readback["createdAt"] == original["createdAt"]
        and readback["tokensUsed"] >= original["tokensUsed"]
        and readback["timeUsedSeconds"] >= original["timeUsedSeconds"]
        and readback.get("tokenBudget") == original.get("tokenBudget")
        and readback["objective"] == original["objective"]
    )
    record["resumeVerified"] = "error" not in record["setResponse"] and readback["status"] == "active"
    record["status"] = "active-verified" if record["resumeVerified"] else "not-resumed-readback-verified"
except Exception as error:
    record["status"] = "error-readback-required"
    record["error"] = str(error)
finally:
    if process.poll() is None:
        process.kill()
    process.wait(timeout=5)
    out.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: record.get(key) for key in
                  ("status", "resumeVerified", "originalAccountingPreserved")}))
