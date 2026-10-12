import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
control = read("control-observation.json")
overridden = read("environment-override-observation.json")
assert control["probeMode"] == "control" and overridden["probeMode"] == "environment-override"
assert control["processId"] != overridden["processId"]
for observation in [control, overridden]:
    assert observation["knownFolderApi"] == "SHGetKnownFolderPath(flags=0, token=NULL)"
    assert not observation["registryOrKnownFolderBindingChanged"]
    assert not observation["windowsSecuritySettingsChanged"] and not observation["baselineApplicationLaunched"]
    assert all(not row["rawUserPathIncluded"] for row in observation["rows"])
comparison = read("known-folder-comparison.json")
assert comparison["actualApiExperiment"] and not comparison["fullBaselineIsolationProved"]
assert not comparison["baselineApplicationLaunched"] and not comparison["settingsOrProfileChanged"]
c, o = [{row["folder"]: row for row in observation["rows"]} for observation in [control, overridden]]
for row in comparison["comparison"]:
    name = row["folder"]
    assert row["sameNativePathBeforeAndAfterOverride"] == (c[name]["nativePathSha256"] == o[name]["nativePathSha256"])
    assert row["overriddenEnvironmentInsideFixture"] and row["nativePathInsideFixture"] == o[name]["nativePathInsideFixture"]
profile = next(row for row in comparison["comparison"] if row["folder"] == "profile")
assert profile["sameNativePathBeforeAndAfterOverride"] and not profile["nativePathInsideFixture"]
assert not profile["overriddenEnvironmentMatchesNativePath"]
for name in ["roaming", "local"]:
    value = next(row for row in comparison["comparison"] if row["folder"] == name)
    assert not value["sameNativePathBeforeAndAfterOverride"]
    assert value["nativePathInsideFixture"] and value["overriddenEnvironmentMatchesNativePath"]
pe = read("official-baseline-pe-import-witness.json")
assert pe["baselineExeSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert pe["baselineCommit"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert pe["selectedImports"] == [{"dll": "shell32.dll", "function": "SHGetKnownFolderPath"}]
assert not pe["baselineExecutableRun"] and not pe["baselineModified"]
provenance = read("source-provenance.json")
assert not provenance["baselineNativeApplicationLaunched"] and not provenance["settingsChanged"]
assert len(provenance["primaryCrates"]) == 2
assert all(crate["matchesCurrentCommittedLock"] and not crate["baselineResolvedVersionIndependentlyProved"]
           for crate in provenance["primaryCrates"])
check = read("independent-post-probe-check.json")
assert check["desktopApplicationCount"] == 1 and check["originalDesktopPid"] == 38924
assert check["monitorLiveHandleVerified"] and check["cycleScheduleBytesRetained"]
assert check["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert check["official019ExeSha256"] == pe["baselineExeSha256"]
assert not check["pairedPerformanceAccepted"] and not check["baselineApplicationLaunched"]
for mode in ["control", "environment-override"]:
    assert (root / f"{mode}-stderr.log").read_text(encoding="utf-8-sig").strip() == ""
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".crate"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "native Profile/AppData boundary experiment, primary source witness and exact official baseline read-only PE imports",
    "profileEnvironmentOverrideIsolation": "not isolated in observed current account",
    "appDataEnvironmentOverrideIsolation": "inside fixture in observed current account",
    "baselineNativeAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "fullGateAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "baselineProfileIsolated": False, "files": len(entries)}))
