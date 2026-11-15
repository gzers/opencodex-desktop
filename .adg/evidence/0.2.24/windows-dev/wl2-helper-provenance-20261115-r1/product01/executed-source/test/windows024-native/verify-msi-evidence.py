"""Verify the pinned administrator MSI proof offline; never execute an installer."""

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

CODE = "1c6a0881c29ad014ed1644b9a57f8b2441e27df0"
DOCS = "b0e7c6d6a8757b301cbcd47c5ed7082ad86e30ae"
ROOT = ".adg/evidence/0.2.24/windows-dev/msi-local-admin-20261114-r1"
PRODUCT = "{67CF3D62-492D-4359-AAFE-8BE6FB17E647}"
PACKAGE_SHA = "9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282"
A = "attempts/out01/"
B = "attempts/out02/"


class Refused(ValueError):
    pass


def need(condition, message):
    if not condition:
        raise Refused(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(repo, *args):
    env = dict(os.environ, GIT_NO_LAZY_FETCH="1")
    return subprocess.check_output(["git", "-C", str(repo), *args], env=env)


def safe_path(value):
    need(isinstance(value, str) and value != "", "empty evidence path")
    path = PurePosixPath(value)
    need(not path.is_absolute() and str(path) == value, "non-canonical evidence path")
    need(not any(p in (".", "..") for p in path.parts), "relative path escape")
    need(not any(c in value for c in ("\\", ":", "\x00")), "ambiguous evidence path")
    return value


def load_snapshot(repo):
    paths = git(repo, "ls-tree", "-rz", "--name-only", DOCS, "--", ROOT).split(b"\0")
    files = {}
    for raw in filter(None, paths):
        path = raw.decode("utf-8")
        need(path.startswith(ROOT + "/"), "unexpected Git evidence root")
        relative = safe_path(path[len(ROOT) + 1:])
        need(relative not in files, "duplicate Git evidence path")
        files[relative] = git(repo, "show", DOCS + ":" + path)
    return files


def decode(data):
    return json.loads(data.decode("utf-8-sig"))


def verify_manifest(files):
    manifest = decode(files["manifest.json"])
    need(manifest["sourceHead"] == CODE, "unexpected evidence source commit")
    seen = set()
    total = 0
    for row in manifest["files"]:
        path = safe_path(row["path"])
        need(path not in seen and path != "manifest.json", "duplicate/self manifest entry")
        seen.add(path)
        data = files[path]
        need(len(data) == row["bytes"] and sha(data) == row["sha256"], "raw evidence mismatch: " + path)
        total += len(data)
    need(seen == set(files) - {"manifest.json"}, "unlisted or missing raw evidence")
    need(len(seen) == manifest["fileCount"] and total == manifest["totalBytes"], "manifest totals differ")
    return len(seen), total


def event_map(data):
    result = {}
    for line in data.decode("utf-8-sig").splitlines():
        row = json.loads(line)
        need(row["event"] not in result, "duplicate transaction event")
        result[row["event"]] = row["facts"]
    return result


def position(log, marker):
    need(marker in log, "missing real MSI log marker: " + marker)
    return log.index(marker)


def verify_logs(refused, normal, install):
    for text, code in ((install, 0), (refused, 1603), (normal, 0)):
        returns = re.findall(r"MainEngineThread is returning (\d+)", text)
        need(returns and int(returns[-1]) == code, "MSI final engine exit differs")
    schedule = "Doing action: OCX023RevokeTerminalCli"
    removal_schedule = "Doing action: RemoveFiles"
    execute = "Executing op: CustomActionSchedule(Action=OCX023RevokeTerminalCli,"
    removal_execute = "Executing op: ActionStart(Name=RemoveFiles,"
    failure = "CustomAction OCX023RevokeTerminalCli returned actual error code 1"
    start = position(refused, execute)
    end = position(refused, failure)
    need(position(refused, schedule) < position(refused, removal_schedule) < start < end, "cleanup scheduling/execution order differs")
    need(removal_execute not in refused[:end] and "Executing op: FileRemove(" not in refused[:end], "files removed before cleanup refusal")
    need("Return value 3" in refused[end:], "no actual transaction failure")
    need(position(normal, schedule) < position(normal, removal_schedule), "normal cleanup scheduling order differs")
    need(position(normal, execute) < position(normal, removal_execute), "normal cleanup execution order differs")
    need("Executing op: FileRemove(" in normal[position(normal, removal_execute):], "normal removal script absent")


def verify(files, code_repo):
    count, total = verify_manifest(files)
    doc = lambda name: decode(files[name])
    events_a = event_map(files[A + "native/native-events.jsonl"])
    events_b = event_map(files[B + "native/native-events.jsonl"])
    identity = events_a["hypothesis"]
    need(identity["administratorScenario"] is True and identity["elevated"] is True, "not an actual administrator scenario")
    need(identity["ordinaryUserAcceptance"] is False and identity["product"] == PRODUCT, "wrong scenario/product")
    sid = identity["sid"]
    for attempt in (A, B):
        child = doc(attempt + "native/child-result.json")
        controller = doc(attempt + "controller.json")
        need(child["sid"] == controller["sid"] == sid and child["elevated"] is True, "test token/SID changed")
        need(controller["administratorScenario"] is True and controller["ordinaryUserAcceptance"] is False, "controller scenario changed")
        package = doc(attempt + "package-facts.json")
        need(package["sha256"] == PACKAGE_SHA and package["bytes"] == 316784640, "package identity changed")
        need(package["productCode"] == PRODUCT and package["version"] == "0.2.24" and package["allUsersPresent"] is False, "package scope changed")
        need(doc(attempt + "daily-path-before.json") == doc(attempt + "daily-path-after.json"), "daily or machine PATH changed")
        need(doc(attempt + "daily-path-result.json")["dailyUserAndMachinePathUnchanged"] is True, "daily PATH readback failed")
    need(events_a["msi-only-install"]["exitCode"] == 0, "installation prerequisite failed")
    need(events_a["installed-context"] == {"perMachineQuery": 1605, "perUserUnmanagedQuery": 0}, "not per-user unmanaged MSI")
    need(events_a["msi-only-uninstall-refused"]["exitCode"] == 1603, "not actual failed uninstall")
    need(events_b["resumed-original-msi-refusal"]["sameSid"] == sid, "continuation used another SID")
    need(events_b["msi-only-uninstall-normal"]["exitCode"] == 0, "normal uninstall failed")
    need(doc(A + "native/child-result.json")["exitCode"] == 101, "original failed assertion was lost")
    need(doc(B + "native/child-result.json")["passed"] is True and doc(B + "native/child-result.json")["exitCode"] == 0, "continuation test did not pass")
    before = doc(A + "native/failed-before.private.json")
    after = doc(A + "native/failed-after.private.json")
    normal = doc(B + "native/normal-after.private.json")
    need(before == after and before["productState"] == 5, "failed uninstall did not retain exact observed state")
    need(before["pendingSha256"] == sha(files[A + "native/pending-before.private.json"]), "pending original bytes differ")
    for key in ("configSha256", "dataSentinelSha256", "homeSentinelSha256", "selection", "path"):
        need(before[key] == normal[key], "normal uninstall changed retained state: " + key)
    need(normal["productState"] == -1, "MSI registration retained")
    for key in ("terminalRecordAbsent", "pendingAbsent", "ownedProgramsAbsent"):
        need(normal[key] is True, "normal uninstall postcondition failed: " + key)
    need(doc(A + "native/msi-final-registration.json")["state"] == 5, "initial product retention lost")
    final = doc(B + "native/msi-final-registration.json")
    need(final == {"productCode": PRODUCT, "state": -1, "safeToRemoveTestIdentity": True}, "final product registration is unresolved")
    logs = [files[p].decode("utf-16") for p in (A + "native/msi-only-uninstall-refused.msi.log", B + "native/msi-only-uninstall-normal.msi.log", A + "native/msi-only-install.msi.log")]
    verify_logs(*logs)
    manifest = doc("source-manifest02.json")
    for row in manifest["files"]:
        data = files["executed02/" + PurePosixPath(safe_path(row["path"])).name]
        need(len(data) == row["bytes"] and sha(data) == row["sha256"], "actual execution source differs")
    for row in doc("final-source-manifest.json")["files"]:
        blob = git(code_repo, "show", CODE + ":" + safe_path(row["path"]))
        need(len(blob) == row["gitBytes"] and sha(blob) == row["gitSha256"], "final source Git blob differs")
    need(doc("build03.json")["exitCode"] == 0 and doc("build02.json")["exitCode"] == 101, "build success/failure history differs")
    cleanup = doc("cleanup05.json")
    need(cleanup["passed"] is False and cleanup["nativeDeleteError"] == 32, "Profile cleanup gap was lost")
    need(doc(B + "controller.json")["passed"] is False, "controller cleanup failure was hidden")
    need(doc("generated-cleanup-blocked.json")["commandExecuted"] is False, "deletion rejection state differs")
    summary = doc("result.json")
    need(summary["administratorTransactionsPassed"] is True and summary["ordinaryUserAcceptance"] is False and summary["allAccepted"] is False, "partial proof promoted to full acceptance")
    return {"proofVerified": True, "evidenceCommit": DOCS, "sourceCommit": CODE, "rawFileCount": count + 1, "payloadBytes": total, "administratorTransactionsPassed": True, "ordinaryUserAcceptance": False, "cleanupComplete": False, "allAccepted": False, "runtimeExecuted": False, "binaryClaims": "recorded hashes; optional --package/--test-executable needed to verify local bodies"}


def refresh(files, name, data):
    files[name] = data
    manifest = decode(files["manifest.json"])
    for row in manifest["files"]:
        if row["path"] == name:
            manifest["totalBytes"] += len(data) - row["bytes"]
            row.update(bytes=len(data), sha256=sha(data))
            break
    files["manifest.json"] = json.dumps(manifest).encode()


def self_test(original, repo):
    cases = []

    def changed_json(name, key, value):
        def change(files):
            row = decode(files[name])
            row[key] = value
            refresh(files, name, json.dumps(row).encode())
        return change

    def early_delete(files):
        name = A + "native/msi-only-uninstall-refused.msi.log"
        log = files[name].decode("utf-16")
        log = log.replace("CustomAction OCX023RevokeTerminalCli returned actual error code 1", "Executing op: FileRemove(FileName=ocx.exe)\r\nCustomAction OCX023RevokeTerminalCli returned actual error code 1", 1)
        refresh(files, name, log.encode("utf-16"))

    cases.append(("raw-byte-corruption", lambda f: f.__setitem__(A + "native/pending-before.private.json", b"corrupt")))
    cases.append(("delete-before-helper-refusal", early_delete))
    cases.append(("retention-drift", changed_json(A + "native/failed-after.private.json", "configSha256", "0" * 64)))
    cases.append(("pending-still-present", changed_json(B + "native/normal-after.private.json", "pendingAbsent", False)))
    cases.append(("lost-original-failure", changed_json(A + "native/child-result.json", "exitCode", 0)))
    cases.append(("different-token", changed_json(B + "native/child-result.json", "elevated", False)))
    cases.append(("cleanup-failure-hidden", changed_json(B + "controller.json", "passed", True)))
    cases.append(("false-full-acceptance", changed_json("result.json", "allAccepted", True)))
    cases.append(("manifest-path-escape", lambda f: f.__setitem__("manifest.json", json.dumps({"sourceHead": CODE, "files": [{"path": "../escape"}]}).encode())))
    rejected = []
    for name, mutate in cases:
        files = copy.copy(original)
        mutate(files)
        try:
            verify(files, repo)
        except (Refused, KeyError, ValueError):
            rejected.append(name)
        else:
            raise Refused("negative evidence accepted: " + name)
    return rejected


def verify_body(path, claim):
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            size += len(chunk)
            digest.update(chunk)
    need(size == claim["bytes"] and digest.hexdigest() == claim["sha256"], "local binary bytes differ: " + path.name)
    return {"bytes": size, "sha256": digest.hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docs-repo", type=Path, required=True)
    parser.add_argument("--code-repo", type=Path, required=True)
    parser.add_argument("--package", type=Path)
    parser.add_argument("--test-executable", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        files = load_snapshot(args.docs_repo)
        result = verify(files, args.code_repo)
        if args.self_test:
            result["negativeEvidenceRejected"] = self_test(files, args.code_repo)
        if args.package:
            result["packageBodyVerified"] = verify_body(args.package, decode(files[A + "package-facts.json"]))
        if args.test_executable:
            result["testBodyVerified"] = verify_body(args.test_executable, decode(files["test-artifact02.json"]))
    except (Refused, KeyError, ValueError, OSError, subprocess.CalledProcessError) as error:
        print(json.dumps({"proofVerified": False, "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        return 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
