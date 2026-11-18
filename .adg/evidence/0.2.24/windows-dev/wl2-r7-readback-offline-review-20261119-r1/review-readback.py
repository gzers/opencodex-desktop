"""Review archived r7 receipts after the independent timed readback has run."""

import argparse
import hashlib
import json
import re
import stat
import sys
from pathlib import Path

SID = "S-1-5-21-587408064-856152790-2583328096-1003"
PREFIX = "original recovery retained/unknown; worker DATA="
RECORD_NAMES = tuple(f"DiagnosticAfter-v2-record-{i:06}.json" for i in (1, 2))
SHA256 = re.compile(r"[0-9a-f]{64}")
ACTION = re.compile(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def json_object(pairs):
    obj = {}
    for key, value in pairs:
        require(key not in obj, f"Duplicate JSON field {key!r}.")
        obj[key] = value
    return obj


def decode(data):
    return json.loads(data, object_pairs_hook=json_object)


def read(root, name, limit=1048576):
    path = root / name
    info = path.lstat()
    require(stat.S_ISREG(info.st_mode) and not getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT,
            f"{name} must be a plain file.")
    with path.open("rb") as stream:
        data = stream.read(limit + 1)
    require(len(data) <= limit and len(data) == info.st_size, f"{name} size is invalid or changed.")
    return decode(data), hashlib.sha256(data).hexdigest(), len(data)


def qpc(value):
    require(isinstance(value, str) and re.fullmatch(r"[0-9]{1,19}", value), "Invalid QPC counter.")
    ticks = int(value)
    require(ticks <= 2**63 - 1, "Invalid QPC counter.")
    return ticks


def review(root, explicit_sha, cmd_sha, launcher_sha):
    require(all(SHA256.fullmatch(value) for value in (explicit_sha, cmd_sha, launcher_sha)),
            "Expected SHA-256 arguments must be lowercase hex.")
    info = root.lstat()
    require(stat.S_ISDIR(info.st_mode) and not getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT,
            "Evidence root must be a plain archived directory.")
    explicit, actual_explicit_sha, _ = read(root, "Explicit-result.json")
    require(actual_explicit_sha == explicit_sha and explicit["mode"] == "Explicit"
            and explicit["passed"] is True and explicit.get("firstError") is None, "Explicit receipt differs.")
    action = explicit["recoveryActionId"]
    require(ACTION.fullmatch(action), "Invalid recovery action ID.")
    explicit_detail = explicit["manager"]["result"]["response"]["error"]["detail"]
    require(explicit_detail.startswith(PREFIX), "Explicit receipt has no original refusal snapshot.")
    explicit_snapshot = decode(explicit_detail[len(PREFIX):])
    require(explicit_snapshot["actionId"] == action, "Explicit snapshot action differs.")

    expected_binding = {"sid": SID, "session": 7, "launcherSha256": cmd_sha,
                        "launcherScriptSha256": launcher_sha, "explicitResultSha256": explicit_sha,
                        "recoveryActionId": action}
    attempt, _, _ = read(root, "DiagnosticAfter-independent-timing-attempt.json")
    timing, timing_sha, _ = read(root, "DiagnosticAfter-independent-timing.json")
    require(attempt == timing["binding"] == expected_binding, "Attempt or timing binding differs.")
    frequency = qpc(timing["qpcFrequency"])
    elapsed = qpc(timing["finishedQpc"]) - qpc(timing["startedQpc"])
    require(frequency > 0 and 0 <= elapsed < 5 * frequency, "Original five-second readback bound unproven.")
    require(timing["originalBudgetMs"] == 5000 and timing["strictlyWithinBudget"] is True
            and timing["elapsedUpperBoundMs"] == (1000 * elapsed + frequency - 1) // frequency,
            "Derived timing fields differ from original ticks.")
    require(timing["accepted"] is True and type(timing["launcherRaw"]) is int and timing["launcherRaw"] == 0
            and all(timing[key] is None for key in ("firstError", "launcherFirstError", "launcherError")),
            "Timing receipt records failure or unknown.")

    after, after_sha, _ = read(root, "DiagnosticAfter-result.json")
    require(after_sha == timing["resultSha256"] and after["mode"] == "DiagnosticAfter"
            and after["passed"] is True and after.get("firstError") is None, "DiagnosticAfter receipt differs.")
    snapshot = after["manager"]["result"]["response"]["facts"]["snapshot"]
    require(snapshot["actionId"] == action, "DiagnosticAfter snapshot action differs.")
    pending = snapshot["preparationPending"]
    require(pending == explicit_snapshot["preparationPending"] and pending["durable"] is True
            and pending["attachmentAttempted"] is True and pending["startRawError"] == 31
            and pending["attachmentRawError"] is None and pending["attachmentErrorKind"] is None,
            "The two snapshots do not match the same durable Pending.")
    require([entry["name"] for entry in after["attachment"]] == list(RECORD_NAMES),
            "Readback attachment names or order differ.")
    exports = []
    previous = None
    for index, (entry, digest_key, event) in enumerate(zip(
            after["attachment"], ("intentDigest", "pendingDigest"), ("AbortRecoveryIntent", "AbortRecoveryPending")), 1):
        record, digest, length = read(root, entry["name"], 65536)
        require(digest == entry["sha256"] == pending[digest_key] and length == entry["bytes"],
                f"Independent attachment {index} bytes differ.")
        require(record["recoveryActionId"] == action and record["event"]["kind"] == event
                and record["sequence"] == index and record["previousRecordDigest"] == previous,
                f"Independent attachment {index} action or chain differs.")
        exports.append({"name": entry["name"], "bytes": length, "sha256": digest})
        previous = digest
    return {"reviewPassed": True, "recoveryActionId": action, "explicitSha256": actual_explicit_sha,
            "diagnosticSha256": after_sha, "timingSha256": timing_sha, "elapsedQpc": elapsed,
            "frequency": frequency, "exports": exports}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--explicit-sha256", required=True)
    parser.add_argument("--cmd-sha256", required=True)
    parser.add_argument("--launcher-sha256", required=True)
    args = parser.parse_args()
    try:
        result = review(args.archive, args.explicit_sha256, args.cmd_sha256, args.launcher_sha256)
    except (KeyError, OSError, TypeError, ValueError) as error:
        print(json.dumps({"reviewPassed": False, "firstError": str(error)}))
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
