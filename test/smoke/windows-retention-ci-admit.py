"""Admit fixed 0.1.9 -> 431 installer checks on a fresh runner, never on the host."""
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import zipfile

spec = importlib.util.spec_from_file_location("etw_admit", Path(__file__).with_name("windows-etw-ci-admit.py"))
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
REPOSITORY = common.REPOSITORY
CANDIDATE_SOURCE = "43142bdbcc7a0b9f1325103febae3f44dce407ab"
CANDIDATE_RUN = 38709701621
CANDIDATE_ARTIFACT = 11960342769
CANDIDATE_ZIP_SHA = "5c91c10a2402d936a7bead216a11421a507c6f750c0651fa079aefd98fd97d58"
BASELINE_RELEASE = 407422226
BASELINE = {
    "msi": {"id": 623593980, "name": "OpenCodeX.Desktop_0.1.9_x64_en-US.msi", "bytes": 4030464,
            "sha256": "b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a"},
    "nsis": {"id": 623594038, "name": "OpenCodeX.Desktop_0.1.9_x64-setup.exe", "bytes": 2884185,
             "sha256": "e2538debc1f8acf5b5ed931f3e68d9b128263742c230fbc151513f2833a38006"},
}


def verify_candidate(run, artifact):
    common.require(run.get("id") == CANDIDATE_RUN and run.get("head_sha") == CANDIDATE_SOURCE,
                   "Wrong candidate source or run")
    common.require(run.get("status") == "completed" and run.get("conclusion") == "success"
                   and run.get("run_attempt") == 1 and run.get("event") == "workflow_dispatch",
                   "Candidate run not a successful first attempt")
    common.require(run.get("repository", {}).get("full_name") == REPOSITORY, "Foreign candidate")
    common.require(artifact.get("id") == CANDIDATE_ARTIFACT and not artifact.get("expired"),
                   "Candidate artifact missing or expired")
    common.require(artifact.get("size_in_bytes") == 7626479
                   and artifact.get("digest") == "sha256:" + CANDIDATE_ZIP_SHA,
                   "Candidate artifact bytes changed")
    common.require(artifact.get("workflow_run", {}).get("id") == CANDIDATE_RUN
                   and artifact["workflow_run"].get("head_sha") == CANDIDATE_SOURCE,
                   "Candidate artifact belongs to another source")


def verify_baseline(release, asset, kind):
    common.require(kind in BASELINE, "Unsupported installer kind")
    expected = BASELINE[kind]
    common.require(release.get("id") == BASELINE_RELEASE and release.get("tag_name") == "v0.1.9"
                   and release.get("draft") is False and release.get("prerelease") is False,
                   "Wrong official baseline release")
    common.require(any(row.get("id") == expected["id"] for row in release.get("assets", [])),
                   "Baseline asset not attached to fixed release")
    common.require(asset.get("id") == expected["id"] and asset.get("name") == expected["name"]
                   and asset.get("size") == expected["bytes"]
                   and asset.get("digest") == "sha256:" + expected["sha256"],
                   "Official baseline installer bytes changed")
    common.require(asset.get("browser_download_url") == (
        "https://github.com/" + REPOSITORY + "/releases/download/v0.1.9/" + expected["name"]),
        "Unexpected baseline download source")


def prepare(kind):
    common.require(os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true",
                   "Ephemeral Windows Actions runner only")
    common.require(os.environ.get("GITHUB_REPOSITORY") == REPOSITORY
                   and os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch"
                   and os.environ.get("GITHUB_RUN_ATTEMPT") == "1", "Explicit first runner dispatch required")
    common.require(kind in BASELINE, "Unsupported installer kind")
    import subprocess
    repo = Path(__file__).resolve().parents[2]
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo).decode().strip()
    common.require(source == os.environ.get("GITHUB_SHA"), "Harness checkout mismatch")
    common.require(not subprocess.check_output(["git", "status", "--porcelain"], cwd=repo).strip(),
                   "Harness checkout not clean")
    output = repo / "test/out/windows-installer-retention"
    common.require(not output.exists(), "Do not repeat the retained installation attempt")
    output.mkdir(parents=True)
    downloads = repo / "test/software/installer-retention"
    downloads.mkdir(parents=True)
    token = os.environ["GITHUB_TOKEN"]

    def get(path, name):
        target = output / name
        common.download(common.BASE + path, target, 4 * 1024 * 1024, token=token)
        return json.loads(target.read_text(encoding="utf-8-sig"))

    ci_id = os.environ.get("OPENCODEX_RETENTION_ADMISSION_RUN", "")
    common.require(ci_id.isdecimal() and int(ci_id) > 0, "Successful exact harness CI run required")
    ci = get("actions/runs/" + ci_id, "harness-ci.original.json")
    jobs = get("actions/runs/" + ci_id + "/jobs?per_page=100", "harness-ci-jobs.original.json")
    common.verify_ci(ci, jobs, source)
    candidate = get(f"actions/runs/{CANDIDATE_RUN}", "candidate-run.original.json")
    artifact = get(f"actions/artifacts/{CANDIDATE_ARTIFACT}", "candidate-artifact.original.json")
    verify_candidate(candidate, artifact)
    release = get(f"releases/{BASELINE_RELEASE}", "baseline-release.original.json")
    asset = get(f"releases/assets/{BASELINE[kind]['id']}", "baseline-asset.original.json")
    verify_baseline(release, asset, kind)
    baseline = downloads / BASELINE[kind]["name"]
    common.download(asset["browser_download_url"], baseline, 8 * 1024 * 1024,
                    expected=BASELINE[kind]["sha256"])
    common.require(baseline.stat().st_size == BASELINE[kind]["bytes"], "Baseline download size mismatch")
    archive_path = downloads / "candidate.original.zip"
    common.download(common.BASE + f"actions/artifacts/{CANDIDATE_ARTIFACT}/zip",
                    archive_path, 16 * 1024 * 1024, token=token, expected=CANDIDATE_ZIP_SHA)
    with zipfile.ZipFile(archive_path) as archive:
        common.checked_zip(archive)
        receipts = [name for name in archive.namelist() if name.endswith("/installer-verification.json")
                    or name == "installer-verification.json"]
        common.require(len(receipts) == 1, "Candidate receipt missing")
        receipt_bytes = archive.read(receipts[0])
        (output / "candidate-receipt.original.json").write_bytes(receipt_bytes)
        receipt = json.loads(receipt_bytes.decode("utf-8-sig"))
        common.require(receipt.get("sourceCommit") == CANDIDATE_SOURCE
                       and receipt.get("version") == "0.1.10" and receipt.get("result") == "pass",
                       "Candidate receipt source differs")
        matches = [row for row in receipt["installers"] if row["kind"] == kind]
        common.require(len(matches) == 1, "Candidate kind not unique")
        item = matches[0]
        members = [name for name in archive.namelist() if Path(name).name == item["name"]]
        common.require(len(members) == 1, "Candidate package not unique")
        target = downloads / item["name"]
        common.extract_one(archive, members[0], target, item["sha256"])
        common.require(target.stat().st_size == item["bytes"], "Candidate package size differs")
    record = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "harnessSource": source, "harnessCiRun": int(ci_id),
        "candidateSource": CANDIDATE_SOURCE, "candidateRun": CANDIDATE_RUN, "candidateArtifact": CANDIDATE_ARTIFACT,
        "kind": kind, "baselineVersion": "0.1.9", "candidateVersion": "0.1.10",
        "baselinePackage": str(baseline), "baselineSha256": BASELINE[kind]["sha256"],
        "candidatePackage": str(target), "candidateSha256": item["sha256"],
        "candidateInstalledExeSha256": item["installedExecutableSha256"],
        "baselineMsiExeSha256": "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608",
        "outputDirectory": str(output), "installerExecutedDuringAdmission": False,
        "applicationLaunched": False, "sameHarnessAndCandidateSourceClaimed": False,
        "hostMachineAcceptanceClaimed": False, "releaseRequested": False,
    }
    (output / "admission.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: record[key] for key in (
        "harnessSource", "candidateSource", "candidateRun", "kind", "installerExecutedDuringAdmission")}))


if __name__ == "__main__":
    try:
        common.require(len(sys.argv) == 2 and sys.argv[1] in BASELINE, "Expected msi or nsis")
        prepare(sys.argv[1])
    except Exception as error:
        # Never print authentication or signed artifact redirect URLs.
        print(json.dumps({"status": "admission-stopped", "type": type(error).__name__,
                          "httpStatus": getattr(error, "code", None),
                          "guard": str(error) if isinstance(error, ValueError) else None,
                          "installerExecutedDuringAdmission": False}))
        raise SystemExit(1) from None
