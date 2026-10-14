"""Add the matching official compiler DLL needed by the standalone formatter."""
import hashlib
import io
import json
import subprocess
import tarfile
import tomllib
import urllib.request
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows-icon-format-20261014"
prior = json.loads((root / "official-rustfmt-result.json").read_text())
assert prior["manifestDate"] == "2026-10-01" and prior["exitCode"] == 3221225781
record = {"toolchainInstalled": False, "globalEnvironmentChanged": False,
          "priorExitCode": prior["exitCode"],
          "source": "https://static.rust-lang.org/dist/2026-10-01/channel-rust-stable.toml"}
try:
    with urllib.request.urlopen(record["source"], timeout=20) as response:
        manifest = tomllib.loads(response.read().decode())
    target = manifest["pkg"]["rustc"]["target"]["x86_64-pc-windows-msvc"]
    assert target["available"] and target["xz_url"].startswith("https://static.rust-lang.org/dist/2026-10-01/")
    with urllib.request.urlopen(target["xz_url"], timeout=60) as response:
        archive = response.read(200 * 1024 * 1024)
    assert hashlib.sha256(archive).hexdigest() == target["xz_hash"]
    record.update(archiveUrl=target["xz_url"], archiveSha256=target["xz_hash"], archiveBytes=len(archive))
    files = []
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:xz") as bundle:
        for item in bundle.getmembers():
            path = PurePosixPath(item.name)
            if not item.isfile() or path.suffix.lower() != ".dll" or path.parent.name != "bin":
                continue
            raw = bundle.extractfile(item).read()
            destination = runtime / path.name
            assert not destination.exists()
            destination.write_bytes(raw)
            files.append({"file": path.name, "sha256": hashlib.sha256(raw).hexdigest()})
    record["extractedDlls"] = files
    assert any(item["file"] == "rustc_driver-9cc664dca72b379e.dll" for item in files)
    app = code / "apps/desktop/tauri"
    paths = ["src/infrastructure/windows_tray_icon.rs", "src/infrastructure/mod.rs",
             "src/infrastructure/window_appearance.rs", "src/lib.rs",
             "tests/app_icon.rs", "tests/windows_tray_icon.rs"]
    args = [str(runtime / "rustfmt.exe"), "--edition", "2021", "--config", "skip_children=true"]
    formatted = subprocess.run(args + [str(app / path) for path in paths], cwd=app,
                               capture_output=True, timeout=30)
    (root / "rustfmt-with-driver.stdout.txt").write_bytes(formatted.stdout)
    (root / "rustfmt-with-driver.stderr.txt").write_bytes(formatted.stderr)
    record["exitCode"] = formatted.returncode
    assert formatted.returncode == 0
    checked = subprocess.run(args + ["--check"] + [str(app / path) for path in paths], cwd=app,
                             capture_output=True, timeout=30)
    record.update(checkExitCode=checked.returncode, status="official-formatter-passed")
    assert checked.returncode == 0
except Exception as error:
    record.update(status="formatter-unavailable", errorType=type(error).__name__, error=str(error))
finally:
    (root / "official-rustfmt-completed.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: record.get(key) for key in ["status", "exitCode", "checkExitCode", "errorType"]}))
