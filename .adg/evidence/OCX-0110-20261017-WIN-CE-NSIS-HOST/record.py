"""Freeze and verify one actual host NSIS retention run; never runs an installer."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/host-nsis-20261017T1220")
CANDIDATE = ROOT.parent / "OCX-0110-20261017-WIN-CE-CANDIDATE"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    original = ROOT / "original"
    result = read(original / "execution-result.json")
    payload = read(original / "payload-verification.json")
    final = read(original / "final-state-readback.json")
    release = read(original / "baseline-release.original.json")
    assert release["id"] == 407422226 and release["tag_name"] == "v0.1.9"
    assert not release["draft"] and not release["prerelease"]
    asset = next(a for a in release["assets"] if a["id"] == 623594038)
    assert asset["digest"] == "sha256:e2538debc1f8acf5b5ed931f3e68d9b128263742c230fbc151513f2833a38006"
    assert asset["size"] == 2884185
    assert result["source"] == "ce670ce76bb1c29f70989ee54838d5b31e97059c"
    assert result["candidateRunId"] == 38996189227 and result["result"] == "pass"
    assert payload["baselinePackageSha256"] == asset["digest"].split(":", 1)[1]
    assert payload["baselineActualNssExeSha256"] == result["baselineImage"]["sha256"]
    assert not payload["baselineMsiVariantComparisonAdmitted"]
    assert payload["candidatePackageSha256"] == "ee10b53f741ccdc5959c9360136e64d9c87fd3f73d0528446cdf7bb4a8c38ba3"
    expected = "059bc70d158e66863cab0aa0b828c4107806724e3b13e2b2cd337fc33ccf77a5"
    assert result["candidateImage"]["sha256"] == result["reinstalledImage"]["sha256"] == expected
    assert payload["candidateActualNssExeSha256"] == payload["candidateExpectedNssExeSha256"] == expected
    steps = [row for row in result["steps"] if "exitCode" in row]
    assert [s["name"] for s in steps] == [
        "install-official-019", "upgrade-to-ce", "default-uninstall-ce",
        "reinstall-ce", "final-default-uninstall-ce",
    ]
    for step in steps:
        assert step["exitCode"] == 0 and step["result"] == "pass"
        token = step["actualToken"]
        assert token["processId"] == step["processId"] and token["executable"] == step["executable"]
        assert not token["elevated"] and token["elevationType"] == 3
        assert step["arguments"].startswith("/S") and "/R " not in step["arguments"]
    seed_checks = [row for row in result["steps"] if "files" in row]
    assert len(seed_checks) == 5 and len(seed_checks[0]["files"]) == 6
    for row in seed_checks:
        assert row["allFixtureBytesUnchanged"] and row["files"] == seed_checks[0]["files"]
        for item in row["files"]:
            path = RUNTIME / item["relative"]
            assert digest(path) == item["sha256"] and path.stat().st_size == item["bytes"]
    assert result["beforeProtection"] == result["afterProtection"]
    assert not result["finalExecutablePresent"] and not result["finalUninstallEntryPresent"]
    assert result["nativeCount"] == final["nativeCount"] == final["installerCount"] == 0
    assert final["installedEntries"] == []
    assert not result["productLaunched"] and not result["uiInputInjected"] and not result["automaticRetry"]
    assert result["customSiblingIsNotAnApplicationBinding"] and not result["explicitPurgeTested"]
    assert not result["fullInstallerGatePassed"] and not result["releaseAccepted"]
    for row in read(ROOT / "copies.json"):
        assert digest(ROOT / row["path"]) == row["sha256"]
        assert (ROOT / row["path"]).read_bytes() == Path(row["runtimePath"]).read_bytes()
    binding = read(ROOT / "candidate-binding.json")
    assert digest(CANDIDATE / "manifest.json") == binding["manifestSha256"]
    assert digest(CANDIDATE / "verification.json") == binding["verificationSha256"]
    return {
        "status": "host-nsis-upgrade-default-uninstall-reinstall-seed-retention-passed",
        "source": result["source"], "candidateRunId": result["candidateRunId"],
        "actualInstallerTokenMode": "ordinary", "actualStepsWithExitZero": len(steps),
        "retainedFixtures": 6, "fixtureReadbackPhases": len(seed_checks),
        "originalPreferencesAndStartupPointerUnchanged": True,
        "nativeAndInstallerCountAtFinalReadback": 0,
        "applicationRootBindingTested": False, "explicitPurgeTested": False,
        "productUiTested": False, "fullInstallerGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        assert digest(ROOT / row["path"]) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists() and not (ROOT / "copies.json").exists()
names = ["run.ps1", "execution-result.json", "payload-verification.json",
         "final-state-readback.json", "baseline-release.original.json"]
copies = []
for name in names:
    target = ROOT / "original" / name
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as stream:
        stream.write((RUNTIME / name).read_bytes())
    copies.append({"path": "original/" + name, "runtimePath": str(RUNTIME / name),
                   "bytes": target.stat().st_size, "sha256": digest(target)})
write(ROOT / "copies.json", copies)
write(ROOT / "candidate-binding.json", {
    "path": CANDIDATE.relative_to(PROJECT).as_posix(),
    "manifestSha256": digest(CANDIDATE / "manifest.json"),
    "verificationSha256": digest(CANDIDATE / "verification.json"),
})
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [{
    "path": path.relative_to(ROOT).as_posix(), "bytes": path.stat().st_size, "sha256": digest(path),
} for path in sorted(ROOT.rglob("*")) if path.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
