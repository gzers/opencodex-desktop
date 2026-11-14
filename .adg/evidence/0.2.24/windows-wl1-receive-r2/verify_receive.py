#!/usr/bin/env python3
"""Offline Git-blob replay; no product, network, Windows host or secret reads."""
import hashlib
import json
import pathlib
import subprocess

DOCS = "13ea948b214ceffb3c3b06d8b1b6ae97b55c8ee5"
CODE = "3a9b71f7c20a404e136652e584c8f29de5efb226"
INTEGRATED = "6b3bd173f"
PREFIX = ".adg/evidence/0.2.24/windows-dev/wl1-production-20261114-r1/"
ROOT = pathlib.Path(__file__).resolve().parents[4]

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def raw(path):
    return git("show", DOCS + ":" + PREFIX + path)

def obj(path):
    return json.loads(raw(path))

def verify():
    manifest = obj("manifest.json")
    total = 0
    names = set()
    for row in manifest["files"]:
        p = pathlib.PurePosixPath(row["path"])
        assert not p.is_absolute() and ".." not in p.parts
        assert str(p) not in names
        names.add(str(p))
        data = raw(str(p))
        assert len(data) == row["bytes"], p
        assert hashlib.sha256(data).hexdigest() == row["sha256"], p
        total += len(data)
    assert len(names) == manifest["fileCount"] == 380
    assert total == manifest["totalBytes"] == 3046760
    assert git("rev-parse", CODE + "^{tree}") == git("rev-parse", INTEGRATED + "^{tree}")
    line_endings = []
    for filename, revision in [("source06.json", "360183c59"), ("source07.json", "72b9f74d6")]:
        for row in obj(filename)["files"]:
            data = git("show", revision + ":" + row["path"])
            if hashlib.sha256(data).hexdigest() != row["sha256"]:
                data = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
                line_endings.append([filename, row["path"], "git-LF / captured-CRLF"])
            assert len(data) == row["bytes"], row["path"]
            assert hashlib.sha256(data).hexdigest() == row["sha256"], row["path"]
    result = obj("result.json")
    facts = result["nativeAndTransportProof"]
    assert facts["startPassed"] and facts["startElapsedMs"] == 24965
    assert facts["statusPassed"] and facts["statusElapsedMs"] == [848, 855, 849]
    assert facts["pid"] == 64904 and facts["port"] == 56328
    assert facts["negativeBindingsRejected"] == 3
    assert not facts["stopPassed"] and facts["stopElapsedMs"] == 10005
    assert not facts["forcedCleanup"] and facts["jobCompleteExitAtFinalReadback"]
    assert facts["allOriginalMembersRetained"] == 334 and not facts["normalExitCodesAllZero"]
    readback = obj("release05-client-readback.json")
    config = next(r for r in readback["files"] if r["path"].endswith("config.toml"))
    assert config["bytesRetained"] and not config["fileIdRetained"] and not config["securityRetained"]
    assert not readback["completeRestoration"] and not readback["durableOriginalAuthority"]
    native = obj("release05-native-summary.json")
    # Keep raw roles unknown: independent source/command proof is not in r1.
    text = json.dumps(native)
    assert "Exited { code: 1060 }" in text and "Exited { code: 1 }" in text
    assert not result["completeStop"] and not result["ordinaryUserMsiAccepted"]
    assert not result["cleanupComplete"] and not result["allAccepted"]
    cleanup = result["cleanup"]
    assert cleanup["accountsRemoved"] == 5 and cleanup["exactOwnedJobsFinallyEmpty"] == 5
    assert cleanup["profileBytes"] == 3211038283 and cleanup["publicBytes"] == 319071336
    assert cleanup["publicDeletionAutoReviewRejected"] and not cleanup["rejectedCommandExecuted"] and cleanup["noBypass"]
    rows = obj("status-matrix.json")["rows"]
    assert len(rows) == 22
    return {"receiveVerified": True, "allAccepted": False, "manifestFiles": len(names), "bytes": total,
            "contributionTreeEqual": True, "sourceCaptureLineEndings": line_endings,
            "statusRows": rows, "sourceCode": CODE, "sourceDocs": DOCS,
            "limits": ["Offline evidence replay, not a new Windows execution", "Source06 original run; source07 action projection only", "Complete Stop, original recovery, continuity, ordinary MSI and cleanup unaccepted"]}

if __name__ == "__main__":
    print(json.dumps(verify(), ensure_ascii=False, indent=2))
