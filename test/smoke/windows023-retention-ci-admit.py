"""0.2.22 -> 0.2.23 admission, no execution. Each kind needs its own fresh job.

Mainline wiring: workflow_dispatch, windows-latest matrix kind=[msi,nsis],
contents:read/actions:read, checkout exact harness SHA. Pass every candidate pin
from a reviewed successful package run; never select latest. Then invoke
windows023-installer-retention.ps1 -AdmissionPath
<test/out/windows023-installer-retention/admission.json> -Execute. Always upload
the entire output directory, including admission/cleanup failures. No reruns.
Default scenario=adjacent. MSI 022 machine -> 023 user is explicitly blocked,
never reported as an upgrade pass. --scenario explicit-conversion verifies the
candidate's real MSI rejection, explicitly uninstalls 022 preserving seeds, then
installs 023. It is a test-only program switch, never automatic migration.
Use --scenario fresh023 in a separate job to
verify the new per-user MSI install/uninstall/reinstall, without removing 022.
"""
import argparse
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import zipfile

# Reuse only bounded HTTPS/download helpers, not historical 019 admission policy.
_spec = importlib.util.spec_from_file_location("windows023_download", Path(__file__).with_name("windows-etw-ci-admit.py"))
_http = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_http)
REPOSITORY = "gzers/opencodex-desktop"
API = f"https://api.github.com/repos/{REPOSITORY}/"
BASELINE_RELEASE = 435948372
BASELINE_TAG_OBJECT = "b666d85c368d2de6aef1fc84c131e54512c28fab"
BASELINE_COMMIT = "1b5c8402dc340540711efa2c2c2037bce673a479"
# Read from GitHub's published v0.2.22 API on 2026-11-07. No synthetic pins.
BASELINE = {
    "nsis": {"id": 715405138, "name": "OpenCodeX.Desktop_0.2.22_x64-setup.exe", "bytes": 5323941,
             "sha256": "0b824645ac1b1c3eacab58a3a7fcfededfc910104ac282cfd99c7855b4f97895"},
    "msi": {"id": 715405068, "name": "OpenCodeX.Desktop_0.2.22_x64_en-US.msi", "bytes": 7467008,
            "sha256": "204a9a8fa5d365de118cbf443ebae007fe062127b47281a5aa4e68e8e6ee81b3"},
}
OUTPUT = "test/out/windows023-installer-retention"
DOWNLOADS = "test/software/windows023-installer-retention"
require = _http.require


def verify_context(environ, platform):
    require(platform == "nt" and environ.get("GITHUB_ACTIONS") == "true"
            and environ.get("RUNNER_OS") == "Windows"
            and environ.get("RUNNER_ENVIRONMENT") == "github-hosted", "Fresh GitHub-hosted Windows runner only")
    require(environ.get("GITHUB_REPOSITORY") == REPOSITORY
            and environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch"
            and environ.get("GITHUB_RUN_ATTEMPT") == "1", "Explicit first dispatch required")
    require(re.fullmatch(r"[0-9a-f]{40}", environ.get("GITHUB_SHA", "")), "Exact harness SHA required")
    require(not any(environ.get(key) for key in ("OPENCODEX_SANDBOX", "OPENCODEX_SANDBOX_ROOT", "CODEX_HOME")),
            "Inherited product overrides refused")


def verify_baseline(release, asset, kind, tag_ref, tag_object):
    require(kind in BASELINE, "Unsupported installer kind")
    pin = BASELINE[kind]
    require(release.get("id") == BASELINE_RELEASE and release.get("tag_name") == "v0.2.22"
            and release.get("draft") is False and release.get("prerelease") is False
            and bool(release.get("published_at")), "Published v0.2.22 required")
    require(tag_ref.get("ref") == "refs/tags/v0.2.22"
            and tag_ref.get("object", {}).get("type") == "tag"
            and tag_ref["object"].get("sha") == BASELINE_TAG_OBJECT
            and tag_object.get("tag") == "v0.2.22" and tag_object.get("sha") == BASELINE_TAG_OBJECT
            and tag_object.get("object", {}).get("type") == "commit"
            and tag_object["object"].get("sha") == BASELINE_COMMIT, "Published tag source changed")
    expected = {"id": pin["id"], "name": pin["name"], "size": pin["bytes"],
                "digest": "sha256:" + pin["sha256"], "state": "uploaded",
                "browser_download_url": f"https://github.com/{REPOSITORY}/releases/download/v0.2.22/{pin['name']}"}
    attached = [row for row in release.get("assets", []) if row.get("id") == pin["id"]]
    require(len(attached) == 1, "Baseline attachment missing or ambiguous")
    for row in (asset, attached[0]):
        require(all(row.get(key) == value for key, value in expected.items()), "Official baseline identity/bytes changed")


def verify_candidate(run, artifact, jobs, pins):
    require(re.fullmatch(r"[0-9a-f]{40}", pins["head"]) and re.fullmatch(r"[0-9a-f]{64}", pins["sha256"]),
            "Full candidate identities required")
    require(all(type(pins[key]) is int and pins[key] > 0 for key in ("run", "attempt", "artifact", "bytes")),
            "Positive candidate run/attempt/artifact/size required")
    require(run.get("id") == pins["run"] and run.get("head_sha") == pins["head"]
            and run.get("run_attempt") == pins["attempt"]
            and run.get("repository", {}).get("full_name") == REPOSITORY
            and run.get("head_repository", {}).get("full_name") == REPOSITORY
            and run.get("path") == ".github/workflows/windows-candidate.yml"
            and run.get("event") == "workflow_dispatch"
            and run.get("status") == "completed" and run.get("conclusion") == "success",
            "Candidate must be the exact successful package workflow run")
    require(len(jobs.get("jobs", [])) == jobs.get("total_count"), "Truncated package job evidence")
    package = [row for row in jobs["jobs"] if row.get("name") == "package"]
    require(len(package) == 1 and package[0].get("run_id") == pins["run"]
            and package[0].get("run_attempt") == pins["attempt"]
            and package[0].get("status") == "completed" and package[0].get("conclusion") == "success",
            "Exact-attempt successful package job required")
    require(artifact.get("id") == pins["artifact"] and artifact.get("expired") is False
            and artifact.get("name") == "opencodex-desktop-0.2.23-windows-x64-candidate"
            and artifact.get("size_in_bytes") == pins["bytes"]
            and artifact.get("digest") == "sha256:" + pins["sha256"]
            and artifact.get("workflow_run", {}).get("id") == pins["run"]
            and artifact["workflow_run"].get("head_sha") == pins["head"], "Candidate artifact identity/bytes changed")
    require(artifact.get("created_at", "") >= package[0].get("started_at", "~")
            and artifact.get("created_at", "~") <= package[0].get("completed_at", ""),
            "Artifact not created during admitted package attempt")


def checked_zip(archive):
    seen, total = set(), 0
    require(len(archive.infolist()) <= 20000, "Too many archive entries")
    for entry in archive.infolist():
        path = PurePosixPath(entry.filename)
        require(entry.filename and not path.is_absolute() and ".." not in path.parts
                and not any(c in entry.orig_filename for c in ("\\", ":", "\0"))
                and all(not part.endswith((".", " ")) for part in path.parts)
                and not stat.S_ISLNK(entry.external_attr >> 16), "Unsafe archive path/link")
        key = entry.filename.casefold().rstrip("/")
        require(key not in seen, "Duplicate archive destination")
        seen.add(key)
        total += entry.file_size
        require(total <= 2 * 1024**3, "Expanded archive too large")
    require(archive.testzip() is None, "Archive CRC failed")


def extract_candidate(archive, kind, head, downloads, output):
    checked_zip(archive)
    receipts = [name for name in archive.namelist() if PurePosixPath(name).name == "installer-verification.json"]
    require(len(receipts) == 1, "One original installer receipt required")
    require(archive.getinfo(receipts[0]).file_size <= 8 * 1024**2, "Installer receipt too large")
    raw = archive.read(receipts[0])
    (output / "candidate-receipt.original.json").write_bytes(raw)
    receipt = json.loads(raw.decode("utf-8-sig"))
    require(receipt.get("version") == "0.2.23" and receipt.get("sourceCommit") == head
            and receipt.get("result") == "pass", "Candidate installer receipt is not this version/head")
    rows = [row for row in receipt.get("installers", []) if row.get("kind") == kind]
    require(len(rows) == 1, "Candidate installer kind ambiguous")
    row = rows[0]
    name = f"OpenCodeX.Desktop_0.2.23_x64{'-setup.exe' if kind == 'nsis' else '_en-US.msi'}"
    require(row.get("name") == name and row.get("result") == "pass"
            and row.get("productVersion") == "0.2.23" and row.get("firstScreen") == "pass"
            and type(row.get("bytes")) is int and row["bytes"] > 0
            and re.fullmatch(r"[0-9a-f]{64}", row.get("sha256", ""))
            and re.fullmatch(r"[0-9a-f]{64}", row.get("installedExecutableSha256", ""))
            and row["installedExecutableSha256"] == row.get("expectedInstalledExecutableSha256"),
            "Candidate package/payload receipt incomplete")
    members = [name for name in archive.namelist() if PurePosixPath(name).name == row["name"]]
    require(len(members) == 1, "Candidate package ambiguous or missing")
    require(archive.getinfo(members[0]).file_size == row["bytes"], "Candidate package size mismatch")
    target = downloads / row["name"]
    _http.extract_one(archive, members[0], target, row["sha256"])
    return target, row


def prepare(args):
    verify_context(os.environ, os.name)
    require(args.scenario != "explicit-conversion" or args.kind == "msi", "Explicit conversion is MSI-only")
    repo = Path(__file__).resolve().parents[2]
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo).decode().strip()
    require(source == os.environ["GITHUB_SHA"], "Harness checkout mismatch")
    require(not subprocess.check_output(["git", "status", "--porcelain"], cwd=repo).strip(), "Clean harness required")
    output, downloads = repo / OUTPUT, repo / DOWNLOADS
    require(not output.exists() and not downloads.exists(), "Preserve previous attempt; new runner required")
    output.mkdir(parents=True)
    try:
        downloads.mkdir(parents=True)
        token = os.environ["GITHUB_TOKEN"]

        def get(path, name):
            target = output / (name + ".original.json")
            _http.download(API + path, target, 8 * 1024**2, token=token)
            return json.loads(target.read_text(encoding="utf-8-sig"))

        pins = {"run": args.candidate_run, "attempt": args.candidate_attempt, "head": args.candidate_head,
                "artifact": args.artifact_id, "bytes": args.artifact_bytes, "sha256": args.artifact_sha256}
        run = get(f"actions/runs/{pins['run']}", "candidate-run")
        artifact = get(f"actions/artifacts/{pins['artifact']}", "candidate-artifact")
        jobs = get(f"actions/runs/{pins['run']}/attempts/{pins['attempt']}/jobs?per_page=100", "candidate-jobs")
        verify_candidate(run, artifact, jobs, pins)
        release = get(f"releases/{BASELINE_RELEASE}", "baseline-release")
        asset = get(f"releases/assets/{BASELINE[args.kind]['id']}", "baseline-asset")
        tag_ref = get("git/ref/tags/v0.2.22", "baseline-tag-ref")
        tag_object = get(f"git/tags/{BASELINE_TAG_OBJECT}", "baseline-tag-object")
        verify_baseline(release, asset, args.kind, tag_ref, tag_object)
        baseline = downloads / BASELINE[args.kind]["name"]
        _http.download(asset["browser_download_url"], baseline, BASELINE[args.kind]["bytes"],
                       expected=BASELINE[args.kind]["sha256"])
        require(baseline.stat().st_size == BASELINE[args.kind]["bytes"], "Baseline size mismatch")
        archive_path = downloads / "candidate.original.zip"
        require(pins["bytes"] <= 1024**3, "Candidate archive too large")
        _http.download(API + f"actions/artifacts/{pins['artifact']}/zip", archive_path,
                       pins["bytes"], token=token, expected=pins["sha256"])
        require(archive_path.stat().st_size == pins["bytes"], "Candidate archive size mismatch")
        with zipfile.ZipFile(archive_path) as archive:
            candidate, row = extract_candidate(archive, args.kind, pins["head"], downloads, output)
        record = {
            "schemaVersion": 23, "scenario": args.scenario, "atUtc": datetime.now(timezone.utc).isoformat(), "harnessSource": source,
            "executionRun": os.environ["GITHUB_RUN_ID"], "executionJob": os.environ["GITHUB_JOB"],
            "candidate": pins, "kind": args.kind, "baselineRelease": BASELINE_RELEASE,
            "baselineVersion": "0.2.22", "candidateVersion": "0.2.23", "baselineSource": BASELINE_COMMIT,
            "baselinePackage": str(baseline), "baselineSha256": BASELINE[args.kind]["sha256"],
            "baselineBytes": BASELINE[args.kind]["bytes"], "candidatePackage": str(candidate),
            "candidateSha256": row["sha256"], "candidateBytes": row["bytes"],
            "candidateInstalledExeSha256": row["installedExecutableSha256"],
            "outputDirectory": str(output), "installerExecuted": False, "windowsAcceptancePassed": False,
        }
        (output / "admission.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(record))
    except Exception as error:
        failure = {"result": "admission-stopped", "errorType": type(error).__name__,
                   "guard": str(error) if isinstance(error, ValueError) else None,
                   "httpStatus": getattr(error, "code", None), "installerExecuted": False}
        (output / "admission-error.json").write_text(json.dumps(failure, indent=2) + "\n", encoding="utf-8")
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=BASELINE)
    parser.add_argument("--scenario", choices=("adjacent", "fresh023", "explicit-conversion"), default="adjacent")
    for key, value_type in (("candidate-run", int), ("candidate-attempt", int), ("candidate-head", str),
                            ("artifact-id", int), ("artifact-bytes", int), ("artifact-sha256", str)):
        parser.add_argument("--" + key, type=value_type, required=True)
    args = parser.parse_args()
    try:
        prepare(args)
    except Exception as error:
        # Never log credentials, signed redirect URLs, or raw urllib exceptions.
        print(json.dumps({"result": "admission-stopped", "errorType": type(error).__name__,
                          "guard": str(error) if isinstance(error, ValueError) else None,
                          "httpStatus": getattr(error, "code", None), "installerExecuted": False}))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
