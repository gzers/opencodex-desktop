"""Verify the exact-source headless component geometry without promoting native gates."""
import hashlib
import json
from pathlib import Path
import struct
import sys

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
invocation, result = read("initial-invocation.json"), read("initial-result.json")
assert invocation["source"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert invocation["codeClean"] and invocation["headless"]
assert not invocation["nativeProductStarted"] and not invocation["currentAdministratorWindowOperated"]
assert not invocation["physicalDpiClaimed"] and not invocation["packageInstallationPerformed"]
assert result["exitCode"] == 0 and result["headless"] and not result["nativeProductStarted"]
assert invocation["nodeVersion"] == "v24.19.0"
assert invocation["browserVersion"] == "154.0.4258.62"
assert "PASS: 8 shared tree geometry cases" in (root / "initial-stdout.original.log").read_text(encoding="utf-8-sig")
assert not (root / "initial-stderr.original.log").read_text(encoding="utf-8-sig").strip()
geometry = read("initial-eight-cases/geometry.json")
assert geometry["result"] == "passed" and "not physical DPI" in geometry["scope"]
cases = geometry["cases"]
assert {(case["theme"], case["scale"]) for case in cases} == {
    (theme, scale) for theme in ("light", "dark") for scale in (1, 1.25, 1.5, 2)}
assert len(cases) == 8
all_rows, controls, purpose_deltas, action_heights = [], [], [], []
multiline = 0
for case in cases:
    assert case["result"] == "passed" and len(case["rows"]) == 5
    assert any(row["mixed"] for row in case["selections"])
    assert any(row["disabled"] for row in case["selections"])
    assert any(row["selected"] for row in case["selections"])
    image = (root / "initial-eight-cases" / case["screenshot"]).read_bytes()
    assert image[:8] == b"\x89PNG\r\n\x1a\n" and struct.unpack(">II", image[16:24]) == (1700, 1200)
    for row in case["rows"]:
        all_rows.append(row)
        controls.extend(row["controls"])
        assert all(abs(control["offsetFromFirstLine"]) <= 1 for control in row["controls"])
        delta = abs(row["purposeLine"] - row["firstLine"])
        assert delta <= 1
        purpose_deltas.append(delta)
        action_heights.extend(control["height"] / case["scale"] for control in row["controls"]
                              if "btn" in control["className"])
        multiline += row["labelHeight"] > row["lineHeight"] * case["scale"] * 1.5
assert len(all_rows) == 40 and len(controls) == 120 and multiline == 24
assert min(action_heights) >= 34
post = read("checkpoint.original.json")
assert post["source"] == invocation["source"] and post["codeWorktreeClean"]
assert post["nativeCount"] == 1 and post["nativeProcesses"][0]["ProcessId"] == 78512
assert post["administratorExitCode"] is None
assert not post["isolatedAcceptanceAccountExists"] and not post["windowsSandboxExecutablePresent"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert len(post["sourceFiles"]) == 9 and len(post["protectedSandboxHashes"]) == 4
method = next(row for row in post["sourceFiles"] if row["path"] == invocation["testScript"])
assert method["sha256"] == invocation["testScriptSha256"]
for key in ("productLaunched", "nativeUiInputInjected", "administratorWindowOperated", "installerExecuted",
            "newAccountOrProfileCreated", "forcedTermination", "macNativeHandoffPerformed",
            "physicalDpiOrNativeWebviewAccepted", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[key] is False
manifest_path, verification_path = root / "manifest.json", root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, verification_path}]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not verification_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path.name) == entries
report = {
    "archiveIntegrity": "passed", "source": post["source"], "manifestFiles": len(entries),
    "scope": "Windows headless Edge production Vue component geometry only",
    "browserCases": len(cases), "rows": len(all_rows), "controlsMeasured": len(controls),
    "multilineRowCases": multiline,
    "maximumControlCenterDeltaCssPixels": max(abs(control["offsetFromFirstLine"]) for control in controls),
    "maximumPurposeDeltaCssPixels": max(purpose_deltas),
    "minimumActionHeightBeforeCssZoom": min(action_heights),
    "allNativeTreeMatrixPassed": False, "physicalDpiAccepted": False,
    "newProductInstanceStarted": False, "productionSourceChanged": False,
    "originalAdministratorInstanceStillLive": True, "administratorExitCode": None,
    "historicalFailuresCleared": False, "fullVersionGatePassed": False,
    "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    verification_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(verification_path.name) == report
print(json.dumps(report))
