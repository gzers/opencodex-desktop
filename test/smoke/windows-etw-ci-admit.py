"""Admit one ephemeral-runner diagnostic; never execute a downloaded binary here."""
import hashlib
import json
import os
import re
import stat
import subprocess
import urllib.parse
import urllib.request
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

REPOSITORY = "gzers/opencodex-desktop"
BASE = "https://api.github.com/repos/" + REPOSITORY + "/"
BD2 = "bd2f6eb10d58115235e35d575507bb69f44666da"
ORIGINAL_RUN = 38537166560
ARTIFACT_ID = 11881042603
ARTIFACT_SHA = "5f42b60d09e802b640b183cc1043f6a4aab067b4d1714082ad9fe8a4dcac5688"
TEST_NAME = "opencodex_desktop_lib-4ef9ce48b814b037.exe"
TEST_SHA = "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
NODE_SHA = "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
NODE_ZIP_SHA = "158f7685b44de51f6c0df1d153526cbcd3e1bc739a8dfc607721cef75de9e541"
NUGET_SHA = "664da746a9c1316767e451dec4c386c458aa5fde6c26461d2d57774fc4cc3d3f"
DECODERS = {
    "Microsoft.Diagnostics.Tracing.TraceEvent.dll": "401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb",
    "Microsoft.Diagnostics.FastSerialization.dll": "83e89baf0bbecca8cfa194cff13365f716849e22efc374b669206f26998a019f",
}
BD2_APPLICATION_TREE = "bf4365586d874d1a9268636e5227efad4621bad1"
REQUIRED_JOBS = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}


def require(value, message):
    if not value:
        raise ValueError(message)


def verify_ci(run, jobs, sha):
    require(run.get("head_sha") == sha and run.get("event") == "push", "CI is not the exact-head push")
    require(run.get("status") == "completed" and run.get("conclusion") == "success", "CI has not succeeded")
    require(run.get("path") == ".github/workflows/ci.yml", "Not the ordinary regression workflow")
    require(run.get("repository", {}).get("full_name") == REPOSITORY, "Foreign CI repository")
    require(len(jobs.get("jobs", [])) == jobs.get("total_count"), "Truncated job admission")
    for name in REQUIRED_JOBS:
        matches = [row for row in jobs["jobs"] if row["name"] == name]
        require(len(matches) == 1 and matches[0]["status"] == "completed"
                and matches[0]["conclusion"] == "success", "Required CI job not successful: " + name)


def verify_original(run, artifact):
    require(run.get("id") == ORIGINAL_RUN and run.get("head_sha") == BD2, "Wrong original run or head")
    require(run.get("event") == "pull_request" and run.get("status") == "completed"
            and run.get("conclusion") == "failure", "Original failure is not retained")
    require(run.get("repository", {}).get("full_name") == REPOSITORY, "Foreign original repository")
    require(artifact.get("id") == ARTIFACT_ID and not artifact.get("expired"), "Wrong or expired artifact")
    require(artifact.get("size_in_bytes") == 8810232
            and artifact.get("digest") == "sha256:" + ARTIFACT_SHA, "Original artifact bytes differ")
    require(artifact.get("workflow_run", {}).get("id") == ORIGINAL_RUN
            and artifact["workflow_run"].get("head_sha") == BD2, "Artifact belongs to another run")


def verify_diagnostic_path(value, application_tree):
    require(value in {"isolated-replay", "post-build-runner-temp"}, "Unknown diagnostic execution path")
    if value == "post-build-runner-temp":
        require(application_tree == BD2_APPLICATION_TREE, "Original regression application tree changed")


def verify_execution_context(event, first_regression):
    require(event in {"push", "pull_request"} if first_regression else event == "workflow_dispatch",
            "Unexpected diagnostic workflow event")


def checked_zip(archive):
    seen = set()
    total = 0
    require(len(archive.infolist()) <= 20000, "Too many archive entries")
    for entry in archive.infolist():
        path = PurePosixPath(entry.filename)
        require(entry.filename and "\0" not in entry.orig_filename and not path.is_absolute() and ".." not in path.parts
                and "\\" not in entry.orig_filename and ":" not in entry.orig_filename,
                "Unsafe archive path")
        require(not stat.S_ISLNK(entry.external_attr >> 16), "Archive link refused")
        key = entry.filename.casefold().rstrip("/")
        require(key not in seen, "Duplicate archive destination")
        seen.add(key)
        total += entry.file_size
        require(total <= 512 * 1024 * 1024, "Archive expanded size exceeded")
    require(archive.testzip() is None, "Archive CRC failed")


class SafeRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, newurl):
        target = urllib.parse.urlparse(newurl)
        require(target.scheme == "https" and target.hostname and not target.username and not target.password,
                "Insecure download redirect")
        redirected = super().redirect_request(request, fp, code, message, headers, newurl)
        if urllib.parse.urlparse(request.full_url).netloc != target.netloc:
            for name in list(redirected.headers):
                if name.lower() == "authorization":
                    del redirected.headers[name]
            for name in list(redirected.unredirected_hdrs):
                if name.lower() == "authorization":
                    del redirected.unredirected_hdrs[name]
        return redirected


def download(url, destination, cap, token=None, expected=None):
    require(not destination.exists(), "Download already exists; inspect before retry")
    request = urllib.request.Request(url, headers={"User-Agent": "OpenCodex-bounded-etw-admission"})
    if token:
        parsed = urllib.parse.urlparse(url)
        require(parsed.scheme == "https" and parsed.netloc == "api.github.com", "Credentials only go to GitHub API")
        request.add_header("Authorization", "Bearer " + token)
        request.add_header("Accept", "application/vnd.github+json")
    digest = hashlib.sha256()
    written = 0
    with urllib.request.build_opener(SafeRedirect()).open(request, timeout=45) as response:
        with destination.open("xb") as output:
            while chunk := response.read(128 * 1024):
                written += len(chunk)
                require(written <= cap, "Download size exceeded")
                digest.update(chunk)
                output.write(chunk)
    if expected:
        require(digest.hexdigest() == expected, "Download digest mismatch")
    return {"bytes": written, "sha256": digest.hexdigest()}


def extract_one(archive, member, destination, expected):
    require(not destination.exists(), "Extraction already exists")
    content = archive.read(member)
    require(hashlib.sha256(content).hexdigest() == expected, "Admitted payload digest mismatch")
    destination.write_bytes(content)


def prepare(first_regression=False):
    require(os.name == "nt" and os.environ.get("GITHUB_ACTIONS") == "true", "Ephemeral Windows Actions runner only")
    require(os.environ.get("GITHUB_REPOSITORY") == REPOSITORY, "Unexpected repository")
    event = os.environ.get("GITHUB_EVENT_NAME")
    verify_execution_context(event, first_regression)
    require(os.environ.get("GITHUB_RUN_ATTEMPT") == "1", "No diagnostic rerun; review a new bounded request")
    sha = os.environ.get("GITHUB_SHA", "")
    require(re.fullmatch("[0-9a-f]{40}", sha), "Expected collector source missing")
    admission_run = os.environ.get("OPENCODEX_ETW_ADMISSION_RUN", "")
    if not first_regression:
        require(re.fullmatch("[1-9][0-9]{0,19}", admission_run), "Exact-head CI admission run required")
    token = os.environ["GITHUB_TOKEN"]
    repo = Path(__file__).resolve().parents[2]
    require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo).decode().strip() == sha,
            "Actual collector checkout mismatch")
    diagnostic_path = ("post-build-runner-temp" if first_regression else
                       os.environ.get("OPENCODEX_ETW_DIAGNOSTIC_PATH", "isolated-replay"))
    application_tree = subprocess.check_output(
        ["git", "rev-parse", "HEAD:apps/desktop"], cwd=repo).decode().strip()
    verify_diagnostic_path(diagnostic_path, application_tree)
    output = repo / "test/out/windows-query-etw"
    require(not output.exists(), "Diagnostic evidence already exists")
    output.mkdir(parents=True)
    downloads = repo / "test/software/ci-bd2-query-etw"
    require(not downloads.exists(), "Admitted software directory already exists")
    downloads.mkdir(parents=True)
    runtime = repo / "test/runtime"
    runtime.mkdir(parents=True, exist_ok=True)
    decoder = runtime / "ci-bd2-query-etw-decoder"
    decoder.mkdir()

    def get(path, name):
        target = output / name
        download(BASE + path, target, 4 * 1024 * 1024, token=token)
        return json.loads(target.read_text(encoding="utf-8-sig"))

    if not first_regression:
        ci = get("actions/runs/" + admission_run, "collector-ci.original.json")
        jobs = get("actions/runs/" + admission_run + "/jobs?per_page=100", "collector-ci-jobs.original.json")
        verify_ci(ci, jobs, sha)
    original = get("actions/runs/" + str(ORIGINAL_RUN), "original-failed-run.original.json")
    artifact = get("actions/artifacts/" + str(ARTIFACT_ID), "original-test-artifact.original.json")
    verify_original(original, artifact)
    archive_path = downloads / "original-test.zip"
    artifact_download = download(BASE + f"actions/artifacts/{ARTIFACT_ID}/zip", archive_path,
                                 16 * 1024 * 1024, token=token, expected=ARTIFACT_SHA)
    test_executable = downloads / TEST_NAME
    with zipfile.ZipFile(archive_path) as archive:
        checked_zip(archive)
        receipt = json.loads(archive.read("windows-backend/test-executable-sha256.json").decode("utf-8-sig"))
        require(receipt["Hash"].lower() == TEST_SHA, "Original runner test EXE receipt differs")
        (output / "original-test-executable-receipt.original.json").write_bytes(
            archive.read("windows-backend/test-executable-sha256.json"))
        extract_one(archive, "windows-backend/" + TEST_NAME, test_executable, TEST_SHA)

    node_zip = downloads / "node-v24.21.0-win-x64.zip"
    node_download = download("https://nodejs.org/dist/v24.21.0/" + node_zip.name, node_zip,
                             128 * 1024 * 1024, expected=NODE_ZIP_SHA)
    checksum_path = output / "node-official-SHASUMS256.original.txt"
    download("https://nodejs.org/dist/v24.21.0/SHASUMS256.txt", checksum_path, 1024 * 1024)
    checksum_lines = [line.split() for line in checksum_path.read_text().splitlines()]
    require([row[0] for row in checksum_lines if len(row) == 2 and row[1] == node_zip.name] == [NODE_ZIP_SHA],
            "Official Node checksum mismatch")
    node = downloads / "node.exe"
    with zipfile.ZipFile(node_zip) as archive:
        checked_zip(archive)
        extract_one(archive, "node-v24.21.0-win-x64/node.exe", node, NODE_SHA)
    nuget = downloads / "traceevent-3.1.21.nupkg"
    decoder_download = download(
        "https://api.nuget.org/v3-flatcontainer/microsoft.diagnostics.tracing.traceevent/"
        "3.1.21/microsoft.diagnostics.tracing.traceevent.3.1.21.nupkg",
        nuget, 8 * 1024 * 1024, expected=NUGET_SHA)
    with zipfile.ZipFile(nuget) as archive:
        checked_zip(archive)
        for name, expected in DECODERS.items():
            extract_one(archive, "lib/netstandard2.0/" + name, decoder / name, expected)
    record = {
        "atUtc": datetime.now(timezone.utc).isoformat(),
        "collectorSource": sha, "collectorCiRun": None if first_regression else int(admission_run),
        "firstOrdinaryCiRegression": first_regression,
        "currentCiIsVerificationNotPreviouslyAdmitted": first_regression,
        "diagnosticPath": diagnostic_path, "applicationTree": application_tree,
        "originalTestSource": BD2, "originalFailedRun": ORIGINAL_RUN, "originalArtifactId": ARTIFACT_ID,
        "pinnedHistoricalActualPrCheckout": "31e7748d9e82c79bef97fc1b73233f08bc1f1b3d",
        "testExecutable": str(test_executable), "testSha256": TEST_SHA,
        "node": str(node), "nodeVersion": "24.21.0", "nodeSha256": NODE_SHA,
        "decoder": str(decoder), "outputRoot": str(runtime / "ci-bd2-query-etw"),
        "downloads": {"testArtifact": artifact_download, "officialNode": node_download, "decoder": decoder_download},
        "apiDigestZipCrcAndPayloadHashesVerified": True,
        "crossHostAuthorizationForwarded": False, "testOrNodeExecutedDuringAdmission": False,
        "originalLibraryRunsAllowed": 1, "originalLibraryCommandArguments": [],
        "buildStageRunsTests": False, "additionalNodePrewarmRequested": False,
        "runnerEnvironmentModifiedOnlyInOriginalChild": diagnostic_path == "post-build-runner-temp",
        "exactHistoricalEnvironmentReproduced": False,
        "prewarm": False, "originalConcurrencyAndDeadlinesPreserved": True,
        "desktopStarted": False, "installerExecuted": False, "releaseAccepted": False,
        "historicalFailureCleared": False, "performanceAcceptanceClaimed": False,
    }
    (output / "admission.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: record[key] for key in (
        "collectorSource", "collectorCiRun", "originalFailedRun", "originalArtifactId",
        "testSha256", "nodeVersion", "nodeSha256", "testOrNodeExecutedDuringAdmission", "releaseAccepted",
    )}))


if __name__ == "__main__":
    try:
        import sys
        require(sys.argv[1:] in ([], ["--first-regression"]), "Unexpected admission arguments")
        prepare(first_regression=sys.argv[1:] == ["--first-regression"])
    except Exception as error:
        # urllib exceptions can include a signed redirect URL. Persist only
        # safe status/type information, never credentials or temporary SAS URLs.
        failure = {
            "status": "admission-stopped-before-execution",
            "errorType": type(error).__name__,
            "httpStatus": getattr(error, "code", None),
            "guard": str(error) if isinstance(error, ValueError) else None,
            "testOrNodeStarted": False,
        }
        target = Path(__file__).resolve().parents[2] / "test/out/windows-query-etw"
        if target.is_dir():
            (target / "admission-error.json").write_text(json.dumps(failure, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(failure))
        raise SystemExit(1) from None
