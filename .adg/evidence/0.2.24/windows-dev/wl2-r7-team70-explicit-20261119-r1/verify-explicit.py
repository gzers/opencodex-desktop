"""Verify archived range-r1 Explicit receipts before assigning the timing pin."""
import argparse
import hashlib
import json
import re
import stat
from pathlib import Path

NATIVE = "windows-before-resume-private-control-v2"
PREFIX = "original recovery retained/unknown; worker DATA="
ACTION = re.compile(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}")
SHA256 = re.compile(r"[0-9a-f]{64}")
PRIOR_SHA = "677d8e9f2d1ceec235837c69a1ddffd5d8023c89e5f464ae5f79843d24dd6977"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def decode(data):
    return json.loads(data, object_pairs_hook=unique_object)


def read(root, name):
    path = root / name
    info = path.lstat()
    require(stat.S_ISREG(info.st_mode) and not getattr(info, "st_file_attributes", 0) & 0x400,
            f"Receipt is not a plain file: {name}")
    require(info.st_size <= 1048576, f"Receipt exceeds 1 MiB: {name}")
    data = path.read_bytes()
    require(len(data) == info.st_size, f"Receipt size changed: {name}")
    return data


def verify(receipts, prior):
    original = read(prior, "DiagnosticBefore-result.json")
    require(hashlib.sha256(original).hexdigest() == PRIOR_SHA, "TEAM70 diagnostic pin differs")
    original_manager = decode(original)["manager"]["result"]["response"]
    original_host = decode(read(prior, "DiagnosticBefore-original-host.json"))
    original_intent = decode(read(prior, "DiagnosticBefore-intent.json"))
    raw = read(receipts, "Explicit-result.json")
    explicit = decode(raw)
    require(explicit["mode"] == "Explicit" and explicit["raw"] == 0 and explicit["passed"] is True
            and explicit.get("firstError") is None, "Explicit launcher failed or is incomplete")
    require(decode(read(receipts, "Explicit-original-host.json")) == original_host,
            "Explicit original Host differs")
    intent = decode(read(receipts, "Explicit-intent.json"))
    require(intent == dict(original_intent, mode="Explicit"), "Explicit invocation binding differs")
    created = decode(read(receipts, "Explicit-creation.json"))
    require(created["pid"] == explicit["pid"] and created["pid"] != original_host["pid"]
            and created["creationFiletime"] == explicit["creationFiletime"]
            and int(created["creationFiletime"]) > int(original_host["creationFiletime"])
            and created["image"] == original_host["image"]
            and "::isolated_r7_partial_capture_explicit" in created["arguments"],
            "Explicit manager process binding differs")
    stdout = read(receipts, "Explicit.stdout.log").decode("utf-8-sig")
    marker = "r7-partial-manager="
    lines = [line.split(marker, 1)[1] for line in stdout.splitlines() if marker in line]
    manager = explicit["manager"]
    require(len(lines) == 1 and decode(lines[0]) == manager, "Explicit raw manager reply differs")
    require("test result: ok. 1 passed; 0 failed;" in stdout and not read(receipts, "Explicit.stderr.log"),
            "Explicit native test or stderr differs")
    elapsed = manager["elapsedMs"]
    require(type(elapsed) is int and 0 <= elapsed <= 10000 and manager["publicStartOpen"] is False,
            "Explicit reply lacks the original 10-second bound")
    result = manager["result"]
    response = result["response"]
    require(result["operation"] == "begin-canonical-abort" and result["refused"] is True
            and response["schema"] == NATIVE and response["purpose"] == "refused"
            and response["scopeDigest"] == original_manager["scopeDigest"]
            and ACTION.fullmatch(response["requestId"]) and response["requestId"] != original_manager["requestId"],
            "Native canonical-abort refusal binding differs")
    require(response["facts"] == {"recoveryRequired": True, "runtimeAuthority": False},
            "Explicit refusal authority differs")
    error = response["error"]
    require(error["detail"].startswith(PREFIX), "Canonical-abort retained worker proof missing")
    snapshot = decode(error["detail"].removeprefix(PREFIX))
    require(all(row[key] is False for row in (explicit, response, snapshot)
                for key in ("normalStop", "closed"))
            and explicit["publicStartOpen"] is response["publicStartOpen"] is response["hostMayExit"] is False
            and response["allAccepted"] is snapshot["runtimeAuthority"] is False
            and snapshot["recoveryRequired"] is True, "Unexpected lifecycle authority")
    prior_proof = original_manager["facts"]["originalRangeProof"]
    recovery = explicit["recoveryActionId"]
    require(ACTION.fullmatch(recovery) and recovery == snapshot["actionId"]
            and recovery != prior_proof["failedActionId"]
            and snapshot["schema"] == "windows-original-recovery-worker-v1"
            and snapshot["workerThread"] == prior_proof["workerThread"]
            and snapshot["sessionId"] == prior_proof["sessionId"]
            and snapshot["scopeDigest"] == prior_proof["scopeDigest"]
            and snapshot["phase"] in ("FailedRetained", "ExpiredRetained"),
            "Retained recovery action or worker binding differs")
    require(snapshot["startWorker"] == original_manager["facts"]["snapshot"],
            "Explicit original Start failure or process binding differs")
    pending = snapshot["preparationPending"]
    require(pending["attachmentAttempted"] is pending["durable"] is True
            and pending["startRawError"] == prior_proof["startRawError"] == 31
            and pending["attachmentRawError"] is pending["attachmentErrorKind"] is None
            and pending["originalRanges"] == prior_proof["originalRanges"]
            and all(SHA256.fullmatch(pending[key]) for key in ("intentDigest", "pendingDigest"))
            and pending["intentDigest"] != pending["pendingDigest"],
            "Durable Intent/Pending or original ranges differ")
    return {"passed": True, "explicitSha256": hashlib.sha256(raw).hexdigest(),
            "recoveryActionId": recovery, "managerElapsedMs": elapsed, "managerBudgetMs": 10000,
            "nativeRefusalRaw": error["raw"], "nativeRefusalKind": error["kind"],
            "firstStartRaw": pending["startRawError"], "originalWorker": snapshot["workerThread"],
            "intentDigest": pending["intentDigest"], "pendingDigest": pending["pendingDigest"],
            "durabilitySource": "original native worker", "independentAttachmentReadback": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receipts", type=Path)
    parser.add_argument("team70_archive", type=Path)
    args = parser.parse_args()
    try:
        report = verify(args.receipts, args.team70_archive)
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print(json.dumps({"passed": False, "firstError": str(error)}, indent=2))
        return 1
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
