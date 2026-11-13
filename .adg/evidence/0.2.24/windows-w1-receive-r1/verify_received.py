"""Independently verify received Git bytes, source bindings and evidence scope.

Does not execute the received scripts, installer actions or native test binaries.
The large Windows archives/MSI/EXE remain on that host: only their recorded
digests and the archived descriptor/generated sources can be checked here.
"""
from pathlib import Path, PurePosixPath
import hashlib
import json
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
DOCS = next(p for p in HERE.parents if (p / ".adg/project.json").is_file())
CODE = Path(sys.argv[1]).resolve()
SOURCE = "b8f1bba91ab611765f23aad6935c21bb66490577"
BASE = "cbc6174de17ef658f34d361ee4a6c734bff8460c"
ROUNDS = DOCS / ".adg/evidence/0.2.24/windows-dev"


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path):
    return json.loads(path.read_bytes())


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root)


def relative(text):
    path = PurePosixPath(text)
    assert not path.is_absolute() and ".." not in path.parts and "\\" not in text
    return Path(*path.parts)


ingest = load(HERE / "ingest.json")
assert ingest["fileCount"] == len(ingest["files"]) == 82
for item in ingest["files"]:
    path = DOCS / relative(item["path"])
    raw = path.read_bytes()
    assert not path.is_symlink()
    assert len(raw) == item["bytes"] and sha(raw) == item["sha256"], item["path"]
    assert raw == git(DOCS, "show", ingest["sourceDocs"] + ":" + item["path"])
assert sum(v["bytes"] for v in ingest["files"]) == ingest["totalBytes"]

changed = git(CODE, "diff", "--name-only", "-z", BASE + "..." + SOURCE).split(b"\0")
changed = [p.decode() for p in changed if p]
assert len(changed) == 13
for path in changed:
    assert (CODE / relative(path)).read_bytes() == git(CODE, "show", SOURCE + ":" + path), path

profile = load(ROUNDS / "w1-profile-20261113/result.json")
path_report = load(ROUNDS / "w1-path-20261113/result.json")
installer_dir = ROUNDS / "w1-installer-20261113"
installer = load(installer_dir / "result.json")
source_bindings = []
crlf_templates = {"apps/desktop/tauri/windows/" + n for n in ("installer-hooks.nsh", "nsis023.nsi", "wix023.wxs")}
for report in (profile, path_report, installer):
    assert report["released"] is False and report["dailyPathWritten"] is False if "dailyPathWritten" in report else report["released"] is False
    for item in report["sourceFiles"]:
        raw = (CODE / relative(item["path"])).read_bytes()
        exact = sha(raw) == item["sha256"]
        only_crlf = not exact and item["path"] in crlf_templates and sha(raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")) == item["sha256"]
        assert exact or only_crlf, item["path"]
        source_bindings.append({"path": item["path"], "gitSha256": sha(raw),
                                "executedSha256": item["sha256"], "exact": exact, "onlyCRLF": only_crlf})
assert sum(v["onlyCRLF"] for v in source_bindings) == 3
assert len(source_bindings) == 13
assert profile["exitCodes"] == [0, 0, 0] and profile["testsPassed"] == 7
assert path_report["attempt01"]["clippyExit"] == 101
assert path_report["attempt02"]["nativeTests"] == 8
assert all(path_report["attempt02"][k] == 0 for k in ("testExit", "clippyExit", "fmtExit"))
for name, count, log in (("w1-profile-20261113", 7, "attempt01/test.stdout.log"),
                         ("w1-path-20261113", 8, "attempt02/test.stdout.log")):
    text = (ROUNDS / name / log).read_text()
    assert re.search(r"test result: ok[.] " + str(count) + r" passed; 0 failed", text)
    assert not load(ROUNDS / name / "result.json")["W2Executed"]

raw_manifest = load(installer_dir / "raw-manifest.json")
assert len(raw_manifest) == 52
assert len({r["evidenceRelative"] for r in raw_manifest}) == 52
for item in raw_manifest:
    path = installer_dir / relative(item["evidenceRelative"])
    raw = path.read_bytes()
    assert not path.is_symlink() and len(raw) == item["bytes"] and sha(raw) == item["sha256"], str(path)

resource = installer_dir / "raw/attempt01/tauri/resources/runtime-bundle"
descriptor_bytes = (resource / "runtime-bundle.manifest.json").read_bytes()
descriptor = json.loads(descriptor_bytes)
assert sha(descriptor_bytes) == installer["descriptorSha256"]
assert (resource / "runtime-bundle.manifest.sha256").read_text().strip() == installer["descriptorSha256"]
policy = load(CODE / ".github/runtime-platforms-024.json")
assert descriptor["runtimeVersion"] == "2.99.0" and descriptor["platform"] == "win32-x64"
assert descriptor["runtimeArchiveSha256"] == policy["runtime"]["sha256"] == installer["officialSourceSha256"]
assert descriptor["archiveSha256"] == installer["archiveSha256"]
assert len({e["path"] for e in descriptor["entries"]}) == len(descriptor["entries"])
assert all(relative(e["path"]) for e in descriptor["entries"])

bundle = load(installer_dir / "raw/bundle01/result.json")
msi = load(installer_dir / "raw/msi-paths04/result.json")
nsis = load(installer_dir / "raw/nsis-paths02/result.json")
assert bundle["passed"] and not bundle["productExecutable"] and not bundle["installed"]
assert bundle["outputs"] == installer["bundles"] and bundle["commands"][0]["exitCode"] == 0
for report, modes in ((msi, ["default", "custom"]), (nsis, ["default", "custom", "remembered", "explicit-over-remembered"])):
    assert report["passed"] and not report["installed"]
    assert [c["mode"] for c in report["cells"]] == modes
    for cell in report["cells"]:
        assert not cell["directoryCreated"]
        assert cell["resolved"].rstrip("\\").casefold() == cell["expected"].rstrip("\\").casefold()
assert nsis["cleanupKeyMissing"] and nsis["compileExit"] == 0
for filename in ("runtime-bundle-0.2.24-win32-x64.tar.gz", "official-runtime.tgz", "runtime-bundle.manifest.json", "runtime-bundle.manifest.sha256"):
    assert sum(row[2].split("|")[-1] == filename for row in msi["files"]) == 1
assert not any(row[0] == "ALLUSERS" for row in msi["properties"])
assert all(not row[2] for row in msi["removeFiles"])
assert not load(installer_dir / "raw/msi-paths01/result.json")["passed"]
assert not load(installer_dir / "raw/msi-paths02/result.json")["passed"]
assert (installer_dir / "raw/msi-paths03/timeout-cleanup.json").is_file()
assert not load(installer_dir / "raw/nsis-paths01/result.json")["passed"]
cleanup = load(installer_dir / "cleanup-pending.json")
assert not cleanup["cleanupComplete"] and cleanup["retainedBuildBytes"] == 2108815151
assert cleanup["retainedBuildFiles"] == 11808
assert cleanup["automaticApprovalReview"]["commandExecuted"] is False
assert cleanup["automaticApprovalReview"]["result"] == "rejected"
print(json.dumps({"receivedBytesVerified": True, "ingestedFiles": 82, "installerRawFiles": 52,
                  "contributionSourceFiles": len(changed), "sourceBindings": source_bindings,
                  "nativeResultsReadFromWindowsLogs": {"profile": 7, "path": 8, "msiCosting": 2, "nsisPathFunctions": 4},
                  "windowsExecutedHere": False, "binaryArtifactsAvailableHere": False,
                  "wholeProductAccepted": False, "cleanupComplete": False,
                  "retainedWindowsBuildBytes": cleanup["retainedBuildBytes"]}, ensure_ascii=False, indent=2))
