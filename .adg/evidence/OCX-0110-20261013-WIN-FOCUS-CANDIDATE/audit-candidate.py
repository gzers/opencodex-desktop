"""Verify and unpack exact candidate artifacts without invoking an installer or app."""
import hashlib
import json
import zipfile
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
receipt = read(root / "candidate-download-audit.json")
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert receipt["sourceCommit"] == source and receipt["runId"] == 38355803684
assert receipt["runConclusion"] == receipt["packageConclusion"] == "success"
assert len(receipt["artifactDownloads"]) == 2
raw_root = root / "artifact-raw"
assert not raw_root.exists() and not (root / "candidate-content-audit.json").exists()
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
software = code / "test/software/candidate-be7395a-38355803684"
assert not software.exists(), "Do not overwrite any candidate package or binary"
inventories = []
witnesses = []
packages = []
verification = None
verification_bytes = None
startup_records = {}
for item in receipt["artifactDownloads"]:
    archive = Path(item["zipPath"])
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == item["zipSha256"]
    with zipfile.ZipFile(archive) as z:
        infos = z.infolist()
        names = [info.filename for info in infos]
        assert len(names) == len(set(names))
        assert sum(info.file_size for info in infos) < 256 * 1024 * 1024
        for info in infos:
            relative = PurePosixPath(info.filename)
            assert not relative.is_absolute() and ".." not in relative.parts
            assert "\\" not in info.filename and ":" not in info.filename
            assert info.file_size < 64 * 1024 * 1024
        assert z.testzip() is None
        inventories.append({
            "artifactId": item["artifactId"], "name": item["name"], "zipSha256": item["zipSha256"],
            "crcVerified": True,
            "entries": [{"path": info.filename, "bytes": info.file_size, "crc32": f"{info.CRC:08x}"} for info in infos],
        })
        if item["name"] == "opencodex-desktop-0.1.10-windows-x64-candidate":
            receipts = [name for name in names if PurePosixPath(name).name == "installer-verification.json"]
            assert len(receipts) == 1
            verification_bytes = z.read(receipts[0])
            verification = json.loads(verification_bytes.decode("utf-8-sig"))
            assert verification["sourceCommit"] == source and verification["version"] == "0.1.10"
            assert verification["result"] == "pass" and not verification["codeSigned"] and not verification["releaseAccepted"]
            assert {entry["kind"] for entry in verification["installers"]} == {"nsis", "msi"}
            for entry in verification["installers"]:
                assert entry["result"] == "pass" and entry["firstScreen"] == "pass"
                assert entry["installExitCode"] in {0, 3010} and entry["uninstallExitCode"] in {0, 3010}
                assert entry["uninstallRemovedExecutable"] and entry["productVersion"] == "0.1.10"
                assert entry["installedExecutableSha256"] == entry["expectedInstalledExecutableSha256"]
                assert entry["bundleMarker"] == {"msi": "MSI", "nsis": "NSS"}[entry["kind"]]
                matching = [name for name in names if PurePosixPath(name).name == entry["name"]]
                assert len(matching) == 1
                value = z.read(matching[0])
                assert len(value) == entry["bytes"] and hashlib.sha256(value).hexdigest() == entry["sha256"]
                software.mkdir(parents=True, exist_ok=True)
                target = software / entry["name"]
                assert not target.exists()
                target.write_bytes(value)
                packages.append({
                    "kind": entry["kind"], "zipMember": matching[0], "path": str(target),
                    "bytes": len(value), "sha256": entry["sha256"],
                    "expectedInstalledExecutableSha256": entry["installedExecutableSha256"],
                })
            keep = receipts
        else:
            assert item["name"] == "windows-candidate-evidence"
            keep = [
                name for name in names
                if name in {"installer-verification.json", "environment.json", "Cargo.lock", "process-token/process-token-evidence.json"}
                or (
                    name.startswith(("msi-startup/", "nsis-startup/"))
                    and (
                        name.count("/") == 1 and PurePosixPath(name).name in {
                            "startup.json", "webview.json", "cdp-diagnostics.json", "startup-diagnostics.json",
                            "startup-stderr.log", "startup-stdout.log", "main-window.png"
                        }
                        or name.endswith("/logs/app.log")
                    )
                )
            ]
            for kind in ["msi", "nsis"]:
                prefix = kind + "-startup/"
                value = json.loads(z.read(prefix + "startup.json").decode("utf-8-sig"))
                assert value["result"] == "pass" and value["expectedCommit"] == source and value["sourceCommitMatched"]
                assert value["applicationTokenVerified"] and value["mainWindowPresent"] and value["responding"]
                assert value["homeRemoved"] and value["observedSeconds"] == 30
                assert value["mode"] == "any" and not value["applicationModeMatched"]
                webview = json.loads(z.read(prefix + "webview.json").decode("utf-8-sig"))
                assert webview["result"] == "pass" and webview["mainUiMounted"] and webview["sandboxDataRootInitialized"]
                assert webview["about"]["version"] == "0.1.10" and webview["about"]["platform"] == "Windows"
                startup_records[kind] = value
            assert "environment.json" in keep and "installer-verification.json" in keep
            environment = json.loads(z.read("environment.json").decode("utf-8-sig"))
            assert environment["source"] == source
        for name in keep:
            destination = raw_root / str(item["artifactId"]) / PurePosixPath(name)
            destination.parent.mkdir(parents=True, exist_ok=True)
            assert not destination.exists()
            value = z.read(name)
            destination.write_bytes(value)
            witnesses.append({
                "artifactId": item["artifactId"], "zipMember": name, "path": destination.relative_to(root).as_posix(),
                "bytes": len(value), "sha256": hashlib.sha256(value).hexdigest(),
            })
assert verification is not None and len(packages) == 2
evidence_verification = next(path for path in raw_root.rglob("installer-verification.json") if path.read_bytes() != b"" and path.read_bytes() == verification_bytes)
assert evidence_verification.is_file()
evidence_receipts = [path for path in raw_root.rglob("installer-verification.json")]
assert len(evidence_receipts) == 2 and all(path.read_bytes() == verification_bytes for path in evidence_receipts)
for entry in verification["installers"]:
    startup = startup_records[entry["kind"]]
    assert startup["sha256"] == entry["installedExecutableSha256"]
    assert startup["applicationToken"]["elevated"] is True and startup["applicationToken"]["elevationType"] == 1
    logs = [path for path in raw_root.rglob("app.log") if entry["kind"] + "-startup" in path.parts]
    assert len(logs) == 1 and "commit=" + source in logs[0].read_text(encoding="utf-8-sig")
record = {
    "sourceCommit": source, "runId": receipt["runId"], "result": "passed",
    "artifactZipDigestsAndCrcVerified": True, "installerReceiptCopiesByteIdentical": True,
    "packages": packages, "builtExecutableSha256": verification["builtExecutableSha256"],
    "selectedRawWitnesses": witnesses,
    "runnerInstallUninstallAndFirstScreenAccepted": True,
    "runnerInstallerKinds": ["msi", "nsis"],
    "runnerActualApplicationTokens": {
        kind: {
            "elevated": value["applicationToken"]["elevated"],
            "elevationType": value["applicationToken"]["elevationType"],
            "mode": value["mode"], "specificModeMatched": value["applicationModeMatched"],
        } for kind, value in startup_records.items()
    },
    "localPackagesAvailable": True, "localInstallerInvoked": False, "localApplicationStarted": False,
    "localFinalExeReadOnlyExtractionAccepted": False, "localOrdinaryAdminTokenPairAccepted": False,
    "physicalDpiMicaTitlebarAccepted": False, "realDataRetentionOrUpgradeAccepted": False,
    "nativeFocusFixAccepted": False, "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "originalDesktopOrCycleReplaced": False, "originalFailedCiFailureCleared": False,
    "originalFailedCiCauseIsolated": False, "mainIntegrated": False,
    "formalTaskReceipt": False, "codeSigned": False, "releasePublished": False, "releaseAccepted": False,
}
(root / "candidate-artifact-inventories.json").write_text(json.dumps(inventories, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
(root / "candidate-content-audit.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "artifactIntegrity": "passed", "rawWitnesses": len(witnesses),
    "packages": [{"kind": item["kind"], "sha256": item["sha256"]} for item in packages],
    "runnerFreshInstallSmoke": "passed", "localInstallerInvoked": False, "nativeFocusFixAccepted": False,
}))
