"""Bounded official-protocol retry of the human goal edit after this turn releases its writer."""
import json
import os
import queue
import subprocess
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

source_dir = Path(__file__).resolve().parent
runtime = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows-goal-user-edit-20261014")
assert not runtime.exists(), "Do not launch a duplicate deferred goal updater"
runtime.mkdir()
thread_id = "01a12e0e-c38f-7df3-b42f-6e80f629545e"
objective = (source_dir / "goal-objective.txt").read_text(encoding="utf-8").strip()
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
request_id = 0
def request(method, params, timeout=15):
    global request_id
    request_id += 1
    process.stdin.write(json.dumps({"id": request_id, "method": method, "params": params}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    until = time.monotonic() + timeout
    while time.monotonic() < until:
        try:
            item = responses.get(timeout=min(1, max(0.01, until - time.monotonic())))
        except queue.Empty:
            if process.poll() is not None:
                raise RuntimeError("Owned official stdio connection exited; no daemon restart permitted")
            continue
        if item.get("id") == request_id:
            return item
    raise TimeoutError("Official protocol observation timeout; stop without guessing mutation result")
record = {
    "userAuthorization": "还有windows修改本对话的目标模式，继续开工",
    "threadId": thread_id, "startedAtUtc": datetime.now(timezone.utc).isoformat(),
    "helperPid": os.getpid(), "maximumElapsedSeconds": 600, "retryIntervalSeconds": 45,
    "status": "waiting_for_current_writer", "attempts": [], "requestedObjective": objective,
    "goalMutationSucceeded": False, "newGoalCreated": False, "goalMarkedComplete": False,
    "tokenBudgetChanged": False, "writerLockOverridden": False, "threadOrDesktopRestarted": False,
    "nativeAppStarted": False, "foregroundInputIssued": False,
}
def save():
    pending = runtime / "status.pending.json"
    pending.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    pending.replace(runtime / "status.json")
save()
clock = time.monotonic()
try:
    init = request("initialize", {
        "clientInfo": {"name": "ocx_windows_user_goal_after_writer", "version": "1.0.0"},
        "capabilities": {"experimentalApi": True},
    })
    if "error" in init:
        raise RuntimeError(json.dumps(init["error"], ensure_ascii=True))
    process.stdin.write(json.dumps({"method": "initialized"}) + "\n")
    process.stdin.flush()
    first_goal = None
    while time.monotonic() - clock < 600:
        response = request("thread/goal/get", {"threadId": thread_id})
        if "error" in response:
            raise RuntimeError(json.dumps(response["error"], ensure_ascii=True))
        goal = response["result"]["goal"]
        assert goal and goal["threadId"] == thread_id and goal["createdAt"] == 1791774103
        if first_goal is None:
            first_goal = goal
            record["before"] = goal
        if goal["status"] == "active" and goal["objective"] == objective:
            record.update(status="user-goal-already-applied-verified", goalMutationSucceeded=True, readback=goal)
            break
        if goal["status"] != "paused" or goal["objective"] != first_goal["objective"]:
            record.update(status="goal-changed-by-another-user-action-stop-without-overwrite", readback=goal)
            break
        result = request("thread/goal/set", {
            "threadId": thread_id, "objective": objective, "status": "active", "origin": "user",
        })
        event = {"atUtc": datetime.now(timezone.utc).isoformat(), "result": "pending"}
        if "error" in result:
            event["error"] = result["error"]
            record["attempts"].append(event)
            if result["error"].get("code") == -32603 and "already has an active writer" in result["error"].get("message", ""):
                record["status"] = "waiting_for_current_writer"
                save()
                time.sleep(45)
                continue
            raise RuntimeError(json.dumps(result["error"], ensure_ascii=True))
        event["result"] = "goal-set-response"
        record["attempts"].append(event)
        readback_response = request("thread/goal/get", {"threadId": thread_id})
        if "error" in readback_response:
            raise RuntimeError("Goal was set but readback unavailable; do not retry mutation")
        actual = readback_response["result"]["goal"]
        record["readback"] = actual
        assert actual["objective"] == objective and actual["status"] == "active"
        assert actual["createdAt"] == first_goal["createdAt"]
        assert actual["tokensUsed"] >= first_goal["tokensUsed"]
        assert actual["timeUsedSeconds"] >= first_goal["timeUsedSeconds"]
        assert actual.get("tokenBudget") == first_goal.get("tokenBudget")
        record.update(status="user-requested-windows-goal-active-verified", goalMutationSucceeded=True,
                      originalUsageAccountingPreserved=True)
        break
    else:
        record["status"] = "bounded-writer-wait-ended-without-goal-change"
except Exception as error:
    record["status"] = "error-requires-goal-readback-before-retry"
    record["error"] = str(error)
    raise
finally:
    record["finishedAtUtc"] = datetime.now(timezone.utc).isoformat()
    record["elapsedSeconds"] = time.monotonic() - clock
    save()
    if process.poll() is None:
        process.kill()
    process.wait(timeout=5)
print(json.dumps({
    "status": record["status"], "goalMutationSucceeded": record["goalMutationSucceeded"],
    "writerLockOverridden": False, "nativeAppStarted": False,
    "originalUsageAccountingPreserved": record.get("originalUsageAccountingPreserved", False),
}, ensure_ascii=True))
