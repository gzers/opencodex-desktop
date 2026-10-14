"""Use a checksum-bound official rustfmt component without installing a toolchain."""
import hashlib
import io
import json
import os
import subprocess
import tarfile
import tomllib
import urllib.request
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows-icon-format-20261014"
runtime.mkdir(exist_ok=False)
record = {"toolchainInstalled": False, "globalEnvironmentChanged": False,
          "source": "https://static.rust-lang.org/dist/channel-rust-stable.toml"}
try:
    with urllib.request.urlopen(record["source"], timeout=20) as response:
        raw_manifest = response.read()
    manifest = tomllib.loads(raw_manifest.decode())
    target = manifest["pkg"]["rustfmt-preview"]["target"]["x86_64-pc-windows-msvc"]
    assert target["available"]
    url = target["xz_url"]
    assert url.startswith("https://static.rust-lang.org/dist/")
    with urllib.request.urlopen(url, timeout=30) as response:
        archive = response.read(32 * 1024 * 1024)
    assert hashlib.sha256(archive).hexdigest() == target["xz_hash"]
    record.update(manifestDate=manifest["date"], component=manifest["pkg"]["rustfmt-preview"]["version"],
                  archiveUrl=url, archiveSha256=target["xz_hash"], archiveBytes=len(archive))
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:xz") as bundle:
        matches = [item for item in bundle.getmembers() if item.name.endswith("/bin/rustfmt.exe")]
        assert len(matches) == 1 and matches[0].isfile()
        binary = bundle.extractfile(matches[0]).read()
    formatter = runtime / "rustfmt.exe"
    formatter.write_bytes(binary)
    record["binarySha256"] = hashlib.sha256(binary).hexdigest()
    files = [
        "src/infrastructure/windows_tray_icon.rs", "src/infrastructure/mod.rs",
        "src/infrastructure/window_appearance.rs", "src/lib.rs",
        "tests/app_icon.rs", "tests/windows_tray_icon.rs",
    ]
    app = code / "apps/desktop/tauri"
    result = subprocess.run([str(formatter), "--edition", "2021", "--config", "skip_children=true",
                             *[str(app / path) for path in files]],
                            cwd=app, capture_output=True, timeout=30)
    (root / "rustfmt.stdout.txt").write_bytes(result.stdout)
    (root / "rustfmt.stderr.txt").write_bytes(result.stderr)
    record["exitCode"] = result.returncode
    assert result.returncode == 0
    check = subprocess.run([str(formatter), "--edition", "2021", "--check", "--config", "skip_children=true",
                            *[str(app / path) for path in files]],
                           cwd=app, capture_output=True, timeout=30)
    record.update(checkExitCode=check.returncode, status="official-component-format-check-passed")
    assert check.returncode == 0
except Exception as error:
    record.update(status="formatter-unavailable-compile-check-still-required", errorType=type(error).__name__,
                  error=str(error))
finally:
    (root / "official-rustfmt-result.json").write_text(
        json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: record.get(key) for key in ["status", "component", "exitCode", "checkExitCode", "errorType"]}))
