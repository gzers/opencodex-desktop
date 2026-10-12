import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
before, hidden = [read(name) for name in ["before-native.json", "hidden-native.json"]]
for value in [before, hidden]:
    assert value["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
    assert value["applicationPid"] == 29936 and value["hwnd"] == 852848
assert before["visible"] and not hidden["visible"]
assert not hidden["foreground"] and not hidden["minimized"]
assert hidden["backdropAttribute"] == 1
assert read("before-webview.json")["result"]["keepProxyOnClose"]
hidden_webview = read("hidden-webview.json")["result"]
assert hidden_webview["keepProxyOnClose"] and not hidden_webview["foreground"]
assert hidden_webview["appearance"]["reason"] == "inactive"
# Native hide does not necessarily change document.visibilityState; keep that
# observed distinction instead of replacing it with a minimization assumption.
assert hidden_webview["documentVisibility"] == "visible"
assert not read("close-action.json")["candidateStillVisibleImmediately"]
limitation = read("restore-instrumentation-limitation.json")
assert not limitation["trayInputAttempted"] and not limitation["windowShowApiInvoked"]
terminal = read("controlled-cleanup-terminal.json")
assert not terminal["originalApplicationStillPresent"] and not terminal["originalSupervisorStillPresent"]
assert terminal["applicationCount"] == 0 and not terminal["normalExitAcceptance"]
session = read("restarted-single-session.json")
assert session["status"] == "running" and session["applicationPid"] == 41328
assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
assert session["sourceCommitMatched"]
assert terminal["finalOwnedSession"]["finishedAtUtc"] < session["startedAtUtc"]
restarted = read("restarted-native.json")
assert restarted["applicationPid"] == 41328 and restarted["hwnd"] == 787306 and restarted["visible"]
webview = read("restarted-webview.json")["result"]
assert webview["preferences"] == {
    "keepProxyOnClose": True, "automaticCheck": False, "channel": "stable",
    "theme": "system", "themeNeedsImport": False,
}
assert webview["originalPin"]["pinned"]
assert webview["appearance"]["material"] == "mica"
states = {item["id"]: item["resolved"] for item in webview["notifications"]}
current = [resolved for name, resolved in states.items() if name.endswith("5a53223f-141f-4694-81e0-138255efddff")]
old = [resolved for name, resolved in states.items() if name.endswith("1695188a-60bd-44ad-829d-ca2ea77499fd")]
assert current == [True] and old == [False]
verification = read("restart-verification.json")
assert verification["retainedApplicationCount"] == 1 and verification["matchesLastRecordedPreferences"]
assert verification["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not verification["sameWindowRestorationAccepted"] and not verification["trayReopenAccepted"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "real caption close hides original process; sequential controlled restart and persistent state readback",
    "sameWindowRestoration": "not accepted; hidden-window Computer Use limitation",
    "trayReopenAccepted": False, "runningPanelPreservationAccepted": False,
    "normalExitAccepted": False, "fullNativeGateAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
