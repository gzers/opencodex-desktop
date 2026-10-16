"""Read already-published ZIPs; prefetch grants no candidate/EXE admission."""
import hashlib
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
SOURCE = "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
RUN = 38893610415
DESTINATION = REPO / f"test/software/candidate-b4d487-{RUN}"
EXPECTED = [
    (12043624213, "opencodex-desktop-0.1.10-windows-x64-candidate",
     "installers.original.zip", "f424026dba42b32136d25e6a7b42e9593040b658dc813b6a72ba968844d612ab"),
    (12044305610, "windows-candidate-evidence", "runner-evidence.original.zip",
     "01336f9b13f3a71e814bb6496fc9cf7f2aa6dd883b6c136a06db2c1147fe7f75"),
]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


api = load("prefetch_reader", Path("D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/"
                                  "OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/github-read.py"))
safe = load("prefetch_transport", REPO / "test/smoke/windows-etw-ci-admit.py")
token = api.credential()
assert not DESTINATION.is_symlink() and not DESTINATION.is_junction()
DESTINATION.mkdir(parents=True, exist_ok=True)
assert DESTINATION.resolve().is_relative_to((REPO / "test/software").resolve())
rows = []
for artifact_id, name, basename, digest in EXPECTED:
    _, artifact = api.request(f"actions/artifacts/{artifact_id}", token)
    assert artifact["id"] == artifact_id and artifact["name"] == name
    assert artifact["workflow_run"]["id"] == RUN and artifact["workflow_run"]["head_sha"] == SOURCE
    assert artifact["digest"] == "sha256:" + digest and not artifact["expired"]
    target = DESTINATION / basename
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    metadata = ROOT / f"prefetch-artifact-{artifact_id}-{stamp}.original.json"
    metadata.write_bytes((json.dumps(artifact, indent=2) + "\n").encode())
    if not target.exists():
        attempt = target.with_name(target.name + ".prefetch-" + stamp)
        try:
            safe.download(artifact["archive_download_url"], attempt, 32 * 1024 * 1024,
                          token=token, expected=digest)
        except Exception as error:
            (ROOT / f"prefetch-failure-{artifact_id}-{stamp}.json").write_bytes((json.dumps({
                "exceptionType": type(error).__name__, "message": str(error),
                "artifactId": artifact_id, "partialPath": str(attempt),
                "candidateAdmitted": False, "workflowReexecuted": False,
            }, indent=2) + "\n").encode())
            raise
        attempt.rename(target)
    data = target.read_bytes()
    assert len(data) == artifact["size_in_bytes"] and hashlib.sha256(data).hexdigest() == digest
    rows.append({"artifactId": artifact_id, "path": str(target),
                 "bytes": len(data), "sha256": digest})
    print(json.dumps({"artifactId": artifact_id, "bytes": len(data), "zipDigestVerified": True}), flush=True)
result = ROOT / "prefetch-result.json"
assert not result.exists()
result.write_bytes((json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runId": RUN, "files": rows,
    "candidateAdmitted": False, "wholeRunSuccessRequiredForAdmission": True,
    "archivesExtracted": False, "hostInstallerExecuted": False,
    "hostProductStarted": False, "workflowReexecuted": False,
}, indent=2) + "\n").encode())
