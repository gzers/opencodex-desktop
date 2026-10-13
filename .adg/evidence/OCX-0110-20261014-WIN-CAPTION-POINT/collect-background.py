"""Read exact diagnostic source and CI, test contracts, query the retained EXE."""
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows-caption-point-5a36cda-20261014"
assert not runtime.exists(), "Keep earlier observations immutable"
runtime.mkdir()
source = "5a36cda04018fb367c2f77e3a2a1bdc22d991e03"
candidate = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image_sha = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
session_path = code / "test/runtime/windows0110/control/current-session.json"
daily = Path(r"D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe")
data = code / "test/runtime/windows0110/data/manager-state"


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout


def session():
    value = json.loads(session_path.read_text(encoding="utf-8-sig"))
    assert value["status"] == "running" and value["sourceCommit"] == candidate
    assert value["applicationPid"] == 10904 and value["executableSha256"] == image_sha
    assert value["applicationToken"]["creationTimeUtc"] == "2026-10-13T14:57:31.5757733Z"
    assert (datetime.now(timezone.utc) -
            datetime.fromisoformat(value["lastLiveAtUtc"].replace("Z", "+00:00"))).total_seconds() < 30
    return value


def protections():
    return {
        "dailyExecutableSha256": sha(daily),
        "protectedSandboxFiles": {
            name: sha(data / name) for name in
            ["preferences.json", "data-root.json", "backup-cleanup-policy.json", "update-schedule.json"]
        },
        "candidateExecutableSha256": sha(Path(session()["executable"])),
        "automaticCheck": json.loads((data / "preferences.json").read_text())["updates"]["app_update_auto_check"],
    }


save("session-before.json", session())
before = protections()
save("protected-before.json", before)
assert before["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert before["candidateExecutableSha256"] == image_sha and before["automaticCheck"] is False

paths = ["test/smoke/windows_caption_probe.py", "test/test_windows_caption_probe.py"]
provenance = []
for relative in paths:
    content = git("show", f"{source}:{relative}")
    destination = runtime / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(content)
    provenance.append({"path": relative, "gitBlob": git("rev-parse", f"{source}:{relative}").decode().strip(),
                       "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()})
save("source-provenance.json", {
    "diagnosticSource": source, "observedExecutableSource": candidate,
    "files": provenance, "productRuntimePathsChangedSinceCandidate":
        git("diff", "--name-only", candidate, source, "--", "apps").decode().splitlines(),
    "sourceCopiedOnlyToIgnoredRuntime": str(runtime), "codeCheckoutChanged": False,
    "productSourceAcceptanceInferred": False,
})

# Git's existing credential helper remains authoritative. Keep all credentials in
# memory, restore environment, and never include them in logs or evidence.
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(
        ["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
        cwd=code, capture_output=True, text=True, check=True, timeout=20,
    )
    fields = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)
    token = fields["password"]

    def api(path):
        request = urllib.request.Request("https://api.github.com/repos/gzers/opencodex-desktop/" + path,
            headers={"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
                     "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "ocx-windows-caption-readonly"})
        with urllib.request.urlopen(request, timeout=25) as response:
            return json.load(response)

    runs = api(f"actions/runs?head_sha={source}&per_page=30")
    save("github-runs-exact-diagnostic-sha.json", runs)
    for run in runs["workflow_runs"]:
        assert run["head_sha"] == source
        save(f"github-jobs-{run['id']}.json", api(f"actions/runs/{run['id']}/jobs?per_page=100"))
        save(f"github-artifacts-{run['id']}.json", api(f"actions/runs/{run['id']}/artifacts?per_page=100"))
finally:
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value

test = subprocess.run(
    [sys.executable, "-B", "-m", "unittest", "discover", "-s", str(runtime / "test"),
     "-p", "test_windows_caption_probe.py", "-v"],
    cwd=runtime, capture_output=True, timeout=30,
)
(root / "portable-contracts.stdout.txt").write_bytes(test.stdout)
(root / "portable-contracts.stderr.txt").write_bytes(test.stderr)
save("portable-contracts-result.json", {"source": source, "exitCode": test.returncode,
    "host": "Windows physical host, portable fake-backend contracts",
    "nativeCaptionAcceptance": False, "nativeInstanceStarted": False})
assert test.returncode == 0
spec = importlib.util.spec_from_file_location("caption_probe", runtime / paths[0])
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
native = probe.NativeWindows()
try:
    # Process creation has 100ns precision. Compare the complete original value.
    expected_creation = (int(datetime(2026, 10, 13, 14, 57, 31, tzinfo=timezone.utc).timestamp())
                         + 11644473600) * 10000000 + 5757733
    snapshot = probe.collect_snapshot(native, 10904, 6685560, image_sha, expected_creation)
finally:
    native.close()
save("native-caption-point.json", snapshot)
save("session-after.json", session())
after = protections()
save("protected-after.json", after)
assert before == after, "Protected sandbox or installation bytes changed"
save("background-result.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
    "candidateSource": candidate, "protectedBytesUnchanged": before == after,
    "originalApplicationPidAndCreationRetained": True,
    "inputIssued": False, "installerRun": False, "newNativeInstanceStarted": False,
    "completeNativeGatePassed": False, "fullPerformanceOrCyclePassed": False,
    "ciRuns": [{"id": run["id"], "headSha": run["head_sha"], "status": run["status"],
                "conclusion": run["conclusion"]} for run in runs["workflow_runs"]],
    "captionScope": snapshot["scope"],
})
print(json.dumps({"portableContractsExitCode": test.returncode, "nativeObservationSaved": True,
                  "protectedBytesUnchanged": True, "ciRuns": len(runs["workflow_runs"])}))
