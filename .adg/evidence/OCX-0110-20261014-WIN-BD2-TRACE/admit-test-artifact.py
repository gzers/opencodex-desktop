"""Read exact-head CI and extract only its test binary/metadata; never install."""
import hashlib
import json
import os
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "bd2f6eb10d58115235e35d575507bb69f44666da"
run_id = 38537161445
runtime = code / "test/software/diagnostic-bd2f6eb1-38537161445"
if runtime.exists():
    assert not any(runtime.iterdir()), "Existing diagnostic payload requires readback before retry"
else:
    runtime.mkdir()
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    result = subprocess.run(["git", "credential", "fill"], cwd=code,
        input="protocol=https\nhost=github.com\n\n", capture_output=True, text=True,
        check=True, timeout=20)
    token = dict(line.split("=", 1) for line in result.stdout.splitlines() if "=" in line)["password"]

    def curl_config(path):
        return (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                f'header = "Authorization: Bearer {token}"\n'
                'header = "Accept: application/vnd.github+json"\n'
                'header = "X-GitHub-Api-Version: 2022-11-28"\n').encode()

    def api(path):
        response = subprocess.run([
            r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
            "--connect-timeout", "10", "--max-time", "25", "--retry", "1",
            "--retry-all-errors", "--retry-delay", "2", "--fail", "--silent", "--show-error"],
            input=(f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
                   'header = "Accept: application/vnd.github+json"\n').encode(),
            capture_output=True, timeout=60)
        if response.returncode != 0:
            failure = {"atUtc": datetime.now(timezone.utc).isoformat(), "path": path,
                       "curlExitCode": response.returncode,
                       "error": response.stderr.decode(errors="replace")}
            (root / "metadata-read-failure.json").write_text(json.dumps(failure, indent=2) + "\n", encoding="utf-8")
        assert response.returncode == 0, "Read-only GitHub API request failed"
        return json.loads(response.stdout)

    run = api(f"actions/runs/{run_id}")
    assert run["head_sha"] == source and run["event"] == "push"
    assert run["status"] == "completed" and run["conclusion"] == "success"
    jobs = api(f"actions/runs/{run_id}/jobs?per_page=100")
    backend = next(job for job in jobs["jobs"] if job["name"] == "backend-windows")
    assert backend["head_sha"] == source and backend["conclusion"] == "success"
    artifacts = api(f"actions/runs/{run_id}/artifacts?per_page=100")
    selected = [item for item in artifacts["artifacts"] if item["name"] == "windows-startup-evidence"]
    assert len(selected) == 1 and not selected[0]["expired"]
    artifact = selected[0]
    assert artifact["workflow_run"]["head_sha"] == source
    save("push-run.json", run)
    save("push-jobs.json", jobs)
    save("push-artifacts.json", artifacts)
    destination = runtime / f"artifact-{artifact['id']}.zip"
    response = subprocess.run([
        r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
        "--location", "--proto-redir", "=https", "--connect-timeout", "10", "--max-time", "90",
        "--fail", "--silent", "--show-error", "--output", str(destination)],
        input=curl_config(f"actions/artifacts/{artifact['id']}/zip"), capture_output=True, timeout=95)
    assert response.returncode == 0, "Download failed; retain partial output, never guess identity"
    digest = hashlib.sha256(destination.read_bytes()).hexdigest()
    assert digest == artifact["digest"].removeprefix("sha256:")
    assert destination.stat().st_size == artifact["size_in_bytes"]
    selected_files = []
    metadata = []
    with zipfile.ZipFile(destination) as archive:
        assert archive.testzip() is None
        for item in archive.infolist():
            name = PurePosixPath(item.filename.replace("\\", "/"))
            assert not name.is_absolute() and ".." not in name.parts and not any(":" in p for p in name.parts)
            metadata.append({"path": item.filename, "bytes": item.file_size, "crc32": item.CRC})
            keep = (name.name.startswith("opencodex_desktop_lib-") and name.suffix == ".exe"
                    or name.name in ["environment.json", "test-executable-sha256.json",
                                     "test-executable-dependents.txt", "test-executable-imports.txt",
                                     "test-executable-manifest.txt", "test-executable.manifest.xml"])
            if not keep or item.is_dir():
                continue
            path = runtime / name.name
            assert not path.exists()
            raw = archive.read(item)
            path.write_bytes(raw)
            selected_files.append({"archivePath": item.filename, "path": str(path),
                                   "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
            if name.suffix != ".exe":
                (root / ("runner-" + name.name)).write_bytes(raw)
    exe = [item for item in selected_files if item["path"].endswith(".exe")]
    assert len(exe) == 1
    record = json.loads((runtime / "test-executable-sha256.json").read_text(encoding="utf-8-sig"))
    if isinstance(record, list):
        assert len(record) == 1
        record = record[0]
    assert exe[0]["sha256"] == record["Hash"].lower()
    env = json.loads((runtime / "environment.json").read_text(encoding="utf-8-sig"))
    binary = Path(exe[0]["path"]).read_bytes()
    assert source.encode() in binary, "Compiled build identity must bind the exact head"
    save("test-artifact-admission.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": run_id,
        "independentPushHead": True, "runConclusion": "success",
        "artifactId": artifact["id"], "archivePath": str(destination),
        "archiveBytes": destination.stat().st_size, "sha256": digest,
        "apiDigestMatches": True, "zipCrcPassed": True, "entries": metadata,
        "selectedFiles": selected_files, "testExecutable": exe[0],
        "ciNodeVersion": env.get("node"), "compiledExactCommitPresent": True,
        "executableStarted": False, "productCandidateCreated": False,
        "installerExecuted": False, "authorizationForwardedAcrossHosts": False,
        "failedPrRun": 38537166560, "failedPrCleared": False,
    })
    print(json.dumps({"source": source, "testExeSha256": exe[0]["sha256"],
                      "testExeBytes": exe[0]["bytes"], "ciNodeVersion": env.get("node"),
                      "executableStarted": False}))
finally:
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
