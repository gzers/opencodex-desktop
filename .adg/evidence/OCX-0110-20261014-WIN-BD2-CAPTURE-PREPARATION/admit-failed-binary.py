"""Admit the historical failed PR's actual test binary without running it."""
import gzip
import hashlib
import json
import os
import subprocess
import zipfile
import urllib.request
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/software/diagnostic-bd2-pr-failure-38537166560"
if runtime.exists():
    assert not any(runtime.iterdir()), "Existing payload needs a separate readback before mutation"
else:
    runtime.mkdir()
head = "bd2f6eb10d58115235e35d575507bb69f44666da"
checkout = "31e7748d9e82c79bef97fc1b73233f08bc1f1b3d"
run_id = 38537166560
prior = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}


def save(name, value):
    with (root / name).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def git(*args):
    return subprocess.run(["git", *args], cwd=code, capture_output=True, check=True).stdout


try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n", text=True, capture_output=True,
        cwd=code, check=True, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def request(path, output=None):
        config = f'url = "https://api.github.com/repos/gzers/opencodex-desktop/{path}"\n'
        config += 'header = "Accept: application/vnd.github+json"\n'
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "30", "--retry", "1",
                "--retry-all-errors", "--retry-delay", "2", "--fail", "--silent", "--show-error"]
        if output is not None:
            config += f'header = "Authorization: Bearer {token}"\n'
            args += ["--location", "--proto-redir", "=https", "--output", str(output)]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=65)
        if response.returncode != 0:
            if output is None:
                # One read-only alternate TLS stack; no certificate bypass.
                direct = urllib.request.Request(
                    f"https://api.github.com/repos/gzers/opencodex-desktop/{path}",
                    headers={"Accept": "application/vnd.github+json", "User-Agent": "ocx-bd2-readonly"})
                with urllib.request.urlopen(direct, timeout=25) as connection:
                    return json.load(connection)
            save("artifact-download-failure.json", {"curlExitCode": response.returncode,
                                                   "endpointPath": path, "error": response.stderr.decode(errors="replace")})
            raise RuntimeError("GitHub read failed; no executable run or guessed identity")
        return json.loads(response.stdout) if output is None else None

    run = request(f"actions/runs/{run_id}")
    assert run["head_sha"] == head and run["event"] == "pull_request"
    assert run["status"] == "completed" and run["conclusion"] == "failure"
    jobs = request(f"actions/runs/{run_id}/jobs?per_page=100")
    job = next(item for item in jobs["jobs"] if item["name"] == "backend-windows")
    assert job["conclusion"] == "failure"
    assert next(step for step in job["steps"] if step["name"] == "Windows backend regression")["conclusion"] == "failure"
    artifacts = request(f"actions/runs/{run_id}/artifacts?per_page=100")
    selected = [item for item in artifacts["artifacts"] if item["name"] == "windows-startup-evidence"]
    assert len(selected) == 1 and not selected[0]["expired"]
    artifact = selected[0]
    assert artifact["workflow_run"]["head_sha"] == head
    save("failed-pr-run.json", run)
    save("failed-pr-jobs.json", jobs)
    save("failed-pr-artifacts.json", artifacts)
    archive_path = runtime / f"artifact-{artifact['id']}.zip"
    request(f"actions/artifacts/{artifact['id']}/zip", archive_path)
    archive_sha = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    assert archive_sha == artifact["digest"].removeprefix("sha256:")
    assert archive_path.stat().st_size == artifact["size_in_bytes"]
    selected_files, entries = [], []
    with zipfile.ZipFile(archive_path) as archive:
        assert archive.testzip() is None
        for item in archive.infolist():
            path = PurePosixPath(item.filename.replace("\\", "/"))
            assert not path.is_absolute() and ".." not in path.parts
            entries.append({"path": item.filename, "bytes": item.file_size, "crc32": item.CRC})
            if item.is_dir() or not (
                path.name.startswith("opencodex_desktop_lib-") and path.suffix == ".exe"
                or path.name in ["environment.json", "test-executable-sha256.json",
                                 "test-executable-dependents.txt", "test-executable-imports.txt",
                                 "test-executable.manifest.xml"]
            ):
                continue
            raw = archive.read(item)
            destination = runtime / path.name
            with destination.open("xb") as stream:
                stream.write(raw)
            selected_files.append({"path": str(destination), "bytes": len(raw),
                                   "sha256": hashlib.sha256(raw).hexdigest()})
            if path.suffix != ".exe":
                (root / ("failed-runner-" + path.name)).write_bytes(raw)
    exe = next(item for item in selected_files if item["path"].endswith(".exe"))
    receipt = json.loads((runtime / "test-executable-sha256.json").read_text(encoding="utf-8-sig"))
    assert exe["sha256"] == receipt["Hash"].lower()
    actual_tree = git("rev-parse", checkout + "^{tree}").decode().strip()
    head_tree = git("rev-parse", head + "^{tree}").decode().strip()
    assert actual_tree == head_tree
    prior_logs = docs / (
        "docs/04-项目资料/03-项目协作资料/01-协作包/"
        "2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/"
        "20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据"
    )
    checkout_file = prior_logs / "control-pr-terminal-windows-checkout.log.gz"
    checkout_raw = gzip.decompress(checkout_file.read_bytes())
    assert checkout.encode() in checkout_raw and head.encode() in checkout_raw
    (root / "failed-pr-checkout-original.log.gz").write_bytes(checkout_file.read_bytes())
    previous_push = json.loads((docs / ".adg/evidence/OCX-0110-20261014-WIN-BD2-TRACE/test-artifact-admission.json")
                               .read_text(encoding="utf-8"))
    environment = json.loads((runtime / "environment.json").read_text(encoding="utf-8-sig"))
    save("failed-test-binary-admission.json", {
        "atUtc": datetime.now(timezone.utc).isoformat(), "associatedHead": head,
        "actualPrCheckout": checkout, "actualTree": actual_tree, "headTree": head_tree,
        "entireTrackedTreeEqualsHead": True, "runId": run_id, "runConclusion": "failure",
        "artifactId": artifact["id"], "artifactBytes": archive_path.stat().st_size,
        "artifactSha256": archive_sha, "apiDigestAndZipCrcPassed": True,
        "entries": entries, "selectedFiles": selected_files, "testExecutable": exe,
        "runnerTestExecutableReceiptMatches": True, "nodeVersion": environment["node"],
        "successfulPushExecutableSha256": previous_push["testExecutableSha256"],
        "failedAndSuccessfulTestExecutablesByteIdentical":
            exe["sha256"] == previous_push["testExecutableSha256"],
        "testExecutableRunCountThisStep": 0, "nodePrewarmed": False,
        "ciRerunOrNewCandidateDispatched": False, "historicalFailureClosed": False,
        "schedulingEtwOrFailureRawTraceObtained": False,
    })
    print(json.dumps({"failedTestExecutableAdmitted": True, "actualCheckout": checkout,
                      "wholeTreeMatchesBd2": True, "testExeSha256": exe["sha256"],
                      "matchesSuccessfulPushBinary": exe["sha256"] == previous_push["testExecutableSha256"],
                      "executableStarted": False}))
finally:
    for key, value in prior.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
