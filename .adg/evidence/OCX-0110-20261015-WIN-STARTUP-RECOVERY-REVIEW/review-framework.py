"""Verify the pinned framework source; keep provenance/line locations, not vendored source."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import tomllib
import urllib.request

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
lock = (code / "apps/desktop/tauri/Cargo.lock").read_bytes()
packages = [item for item in tomllib.loads(lock.decode())["package"] if item["name"] == "tauri"]
assert len(packages) == 1 and packages[0]["version"] == "2.12.1"
package = packages[0]
url = "https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.1/crates/tauri/src/app.rs"
with urllib.request.urlopen(urllib.request.Request(
        url, headers={"User-Agent": "OpenCodex-readonly-source-review"}), timeout=25) as response:
    raw = response.read(512 * 1024 + 1)
    assert response.status == 200 and len(raw) <= 512 * 1024
text = raw.decode("utf-8")
setup_body = text[text.index("fn setup<R: Runtime>"):text.index("fn on_event_loop_event<R: Runtime>")]
assert "(setup)(app).map_err(|e| crate::Error::Setup(e.into()))?;" in setup_body
ready = 'RuntimeRunEvent::Ready => {\n        if let Err(e) = setup(&mut self) {'
panic = 'panic!("Failed to setup app: {e}");'
assert ready in text and panic in text[text.index(ready):text.index(ready)+180]
report = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(),
    "provider": "Tauri official source tag matching the package lock",
    "package": package["name"], "version": package["version"], "lockedCrateChecksum": package["checksum"],
    "cargoLockWorkingFileSha256": hashlib.sha256(lock).hexdigest(),
    "sourceUrl": url, "httpStatus": 200, "sourceBytes": len(raw),
    "sourceSha256": hashlib.sha256(raw).hexdigest(),
    "setupFunctionLine": text[:text.index("fn setup<R: Runtime>")].count("\n") + 1,
    "readyCallsSetupLine": text[:text.index(ready)].count("\n") + 1,
    "readySetupErrorPanicsLine": text[:text.index(panic, text.index(ready))].count("\n") + 1,
    "setupHookErrorMapsToSetupError": True,
    "setupRunsAtReadyAndFailurePanics": True,
    "projectBuildExpectIsSetupFailureHandler": False,
    "previousUncommittedSourceReviewCorrection":
        "Initial project-only review conflated builder expect with setup error handling; corrected after pinned framework verification.",
    "fullThirdPartySourceCopied": False, "nativeApplicationExecuted": False,
}
target = root / "framework-source-review.json"
assert not target.exists()
target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"framework": "tauri", "version": "2.12.1",
                  "setupFailureAtReadyVerified": True, "nativeExecution": False}))
