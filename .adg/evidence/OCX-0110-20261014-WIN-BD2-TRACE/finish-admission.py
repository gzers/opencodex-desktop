"""Verify retained exact-CI bytes and download the official matching Node only."""
import gzip
import hashlib
import json
import subprocess
import zipfile
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/software/diagnostic-bd2f6eb1-38537161445"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
source = "bd2f6eb10d58115235e35d575507bb69f44666da"
run = read(root / "push-run.json")
assert run["head_sha"] == source and run["event"] == "push" and run["conclusion"] == "success"
artifact = next(item for item in read(root / "push-artifacts.json")["artifacts"]
                if item["name"] == "windows-startup-evidence")
assert artifact["workflow_run"]["head_sha"] == source
archive_path = runtime / f"artifact-{artifact['id']}.zip"
assert hashlib.sha256(archive_path.read_bytes()).hexdigest() == artifact["digest"].removeprefix("sha256:")
with zipfile.ZipFile(archive_path) as archive:
    assert archive.testzip() is None
    binary = [item for item in archive.infolist()
              if Path(item.filename).name.startswith("opencodex_desktop_lib-") and item.filename.endswith(".exe")]
    assert len(binary) == 1
    raw = archive.read(binary[0])
exe = runtime / Path(binary[0].filename).name
assert exe.read_bytes() == raw
digest = hashlib.sha256(raw).hexdigest()
assert digest == read(runtime / "test-executable-sha256.json")["Hash"].lower()
prior_dir = docs / (
    "docs/04-项目资料/03-项目协作资料/01-协作包/"
    "2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/"
    "20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据"
)
checkout = prior_dir / "control-push-final-windows-checkout.log.gz"
checkout_raw = gzip.decompress(checkout.read_bytes())
assert source.encode() in checkout_raw and b"log -1 --format=%H" in checkout_raw
assert b"refs/remotes/origin/feature/0.1.10-integration-candidate" in checkout_raw
(root / "source-ci-checkout-original.log.gz").write_bytes(checkout.read_bytes())
admission = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": run["id"],
    "artifactId": artifact["id"], "archiveBytes": archive_path.stat().st_size,
    "archiveSha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
    "artifactApiSourceMatches": True, "zipCrcPassed": True,
    "testExecutable": str(exe), "testExecutableBytes": len(raw), "testExecutableSha256": digest,
    "runnerTestExecutableHashMatches": True, "independentHeadCheckoutLogMatches": True,
    "compiledCommitLiteralPresent": source.encode() in raw,
    "initialAdmissionStoppedOnAbsentCommitLiteral": True,
    "checkoutAssertionCorrected": "Actions checks out a remote ref then a local branch; refs/heads literal not required.",
    "identityBasis": "independent push checkout + exact-run artifact/source + ZIP digest/CRC + runner test EXE hash",
    "oldFailedPrRun": 38537166560, "oldFailureCleared": False,
    "desktopExecutableStarted": False, "testExecutableStartedAtAdmission": False,
}
(root / "test-artifact-admission.json").write_text(
    json.dumps(admission, indent=2) + "\n", encoding="utf-8")
env = read(runtime / "environment.json")
assert env["node"] == "v24.21.0"
node_root = code / "test/software/node-v24.21.0-official"
node_root.mkdir(exist_ok=False)
base = "https://nodejs.org/dist/v24.21.0/"


def download(name, destination):
    result = subprocess.run([
        r"C:\Windows\System32\curl.exe", "-4", "--http1.1", "--proto", "=https",
        "--connect-timeout", "10", "--max-time", "90", "--fail", "--silent", "--show-error",
        base + name, "--output", str(destination)], capture_output=True, timeout=95)
    assert result.returncode == 0, "Official Node download failed; no binary may run"


checksum_path = node_root / "SHASUMS256.txt"
download("SHASUMS256.txt", checksum_path)
package_name = "node-v24.21.0-win-x64.zip"
checksum_lines = checksum_path.read_text().splitlines()
expected = [line.split()[0] for line in checksum_lines if line.split()[-1] == package_name]
assert len(expected) == 1
package = node_root / package_name
download(package_name, package)
actual = hashlib.sha256(package.read_bytes()).hexdigest()
assert actual == expected[0]
with zipfile.ZipFile(package) as archive:
    assert archive.testzip() is None
    members = [item for item in archive.infolist() if item.filename == "node-v24.21.0-win-x64/node.exe"]
    assert len(members) == 1
    node_raw = archive.read(members[0])
(node_root / "node.exe").write_bytes(node_raw)
(root / "official-node-admission.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": base, "version": env["node"],
    "versionMatchesCi": True, "checksumSource": base + "SHASUMS256.txt",
    "checksumFileSha256": hashlib.sha256(checksum_path.read_bytes()).hexdigest(),
    "archiveSha256": actual, "officialChecksumMatches": True,
    "path": str(node_root / "node.exe"), "exeSha256": hashlib.sha256(node_raw).hexdigest(),
    "exeBytes": len(node_raw), "npmInstalled": False, "lifecycleScriptsExecuted": False,
    "nodeStartedOrPrewarmed": False, "globalNodeOrPathChanged": False,
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"testExeAdmitted": True, "compiledLiteralPresent": admission["compiledCommitLiteralPresent"],
                  "nodeVersion": env["node"], "officialNodeChecksumPassed": True, "nodePrewarmed": False}))
