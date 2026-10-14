"""Download exact-head runner evidence and candidate archives without installing."""
import hashlib
import json
import os
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a"
run_id = 38499860139
output = root / "candidate-archive-verification.json"
assert not output.exists()
runtime = code / "test/software/candidate-bb3122d-38499860139"
runtime.mkdir(exist_ok=True)
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}


def save(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
                                cwd=code, text=True, capture_output=True, check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def config(path):
        url = "https://api.github.com/repos/gzers/opencodex-desktop/" + path
        return (f'url = "{url}"\nheader = "Authorization: Bearer {token}"\n'
                'header = "Accept: application/vnd.github+json"\n'
                'header = "X-GitHub-Api-Version: 2022-11-28"\n').encode()

    def api(path):
        response = subprocess.run([r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
            "--connect-timeout", "10", "--max-time", "30", "--retry", "1", "--retry-all-errors",
            "--retry-delay", "2", "--fail", "--silent", "--show-error"],
            input=config(path), capture_output=True, timeout=70)
        assert response.returncode == 0, "GitHub API read failed"
        return json.loads(response.stdout)

    run = api(f"actions/runs/{run_id}")
    assert run["head_sha"] == source and run["event"] == "workflow_dispatch"
    assert run["status"] == "completed" and run["conclusion"] == "success"
    artifacts = api(f"actions/runs/{run_id}/artifacts?per_page=100")
    assert artifacts["total_count"] == 2
    save("candidate-terminal.json", run)
    save("candidate-artifacts-api.json", artifacts)
    rows = []
    extracted = []
    for artifact in artifacts["artifacts"]:
        assert artifact["workflow_run"]["head_sha"] == source and not artifact["expired"]
        destination = runtime / f"artifact-{artifact['id']}.zip"
        expected = artifact["digest"].removeprefix("sha256:")
        if not destination.exists():
            # Curl strips Authorization on a cross-host redirect. Never enable
            # --location-trusted and never print redirect URLs or credentials.
            response = subprocess.run([r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--location", "--proto-redir", "=https", "--connect-timeout", "10", "--max-time", "90",
                "--fail", "--silent", "--show-error", "--output", str(destination)],
                input=config(f"actions/artifacts/{artifact['id']}/zip"), capture_output=True, timeout=95)
            assert response.returncode == 0, "Archive download failed; keep any existing bytes for readback"
        actual = hashlib.sha256(destination.read_bytes()).hexdigest()
        assert actual == expected and destination.stat().st_size == artifact["size_in_bytes"]
        extraction = runtime / f"artifact-{artifact['id']}"
        extraction.mkdir(exist_ok=False)
        metadata = []
        with zipfile.ZipFile(destination) as archive:
            assert archive.testzip() is None
            for item in archive.infolist():
                path = PurePosixPath(item.filename.replace("\\", "/"))
                assert not path.is_absolute() and ".." not in path.parts
                assert all(":" not in part for part in path.parts)
                metadata.append({"path": item.filename, "bytes": item.file_size, "crc32": item.CRC})
                keep = (item.filename.endswith(".msi") or item.filename.endswith("-setup.exe")
                        or item.filename.endswith("installer-verification.json")
                        or item.filename.endswith("startup.json")
                        or item.filename == "environment.json")
                if not keep or item.is_dir():
                    continue
                child = extraction.joinpath(*path.parts)
                assert child.resolve().is_relative_to(extraction.resolve())
                child.parent.mkdir(parents=True, exist_ok=True)
                with child.open("xb") as stream:
                    stream.write(archive.read(item))
                extracted.append(child)
        rows.append({"id": artifact["id"], "name": artifact["name"], "archivePath": str(destination),
                     "archiveBytes": artifact["size_in_bytes"], "sha256": actual,
                     "expectedSha256": expected, "sha256Matches": True, "zipCrcPassed": True,
                     "entries": metadata})
    receipts = [path for path in extracted if path.name == "installer-verification.json"]
    assert len(receipts) == 2 and receipts[0].read_bytes() == receipts[1].read_bytes()
    verification = json.loads(receipts[0].read_text(encoding="utf-8-sig"))
    (root / "installer-verification.json").write_bytes(receipts[0].read_bytes())
    packages = []
    for path in extracted:
        if path.suffix == ".msi" or path.name.endswith("-setup.exe"):
            kind = "msi" if path.suffix == ".msi" else "nsis"
            receipt = next(item for item in verification["installers"] if item["kind"] == kind)
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            assert actual == receipt["sha256"]
            packages.append({"kind": kind, "path": str(path), "bytes": path.stat().st_size,
                             "sha256": actual,
                             "expectedInstalledExecutableSha256": receipt["expectedInstalledExecutableSha256"],
                             "bundleMarkerOffset": receipt["bundleMarkerOffset"]})
    assert len(packages) == 2
    save("candidate-archive-verification.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": run_id,
        "archives": rows, "packages": packages,
        "builtExecutableSha256": verification["builtExecutableSha256"],
        "selectedExtractedPaths": [str(path) for path in extracted],
        "installerInvokedOnUserHost": False, "executableStartedOnUserHost": False,
        "authorizationForwardedAcrossHosts": False, "releaseAccepted": False,
    })
    print(json.dumps({"source": source, "archivesPassed": len(rows), "packagesPassed": len(packages),
                      "installedOnUserHost": False}))
finally:
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
