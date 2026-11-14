#!/usr/bin/env python3
"""Verify immutable W-L2 Git blobs; no Windows execution or business reads."""
import hashlib
import json
import pathlib
import subprocess

CODE = "35e7336fcc64bcf3cab26ca8fa9688daba95a1f0"
DOCS = "70ebd2a67"
INTEGRATED = "301cb7e43"
PREFIX = ".adg/evidence/0.2.24/windows-dev/wl2-native-20261114-r1/"
ROOT = pathlib.Path(__file__).resolve().parents[4]

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def raw(p):
    return git("show", DOCS + ":" + PREFIX + p)

def obj(p):
    return json.loads(raw(p))

def equivalent(data, row):
    if hashlib.sha256(data).hexdigest() == row["sha256"]:
        assert len(data) == row["bytes"]
        return "exact"
    converted = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    assert hashlib.sha256(converted).hexdigest() == row["sha256"], row["path"]
    assert len(converted) == row["bytes"]
    return "git-LF / captured-CRLF"

def verify():
    m = obj("manifest.json")
    seen = set()
    total = 0
    for f in m["files"]:
        p = pathlib.PurePosixPath(f["path"])
        assert not p.is_absolute() and ".." not in p.parts and str(p) not in seen
        seen.add(str(p))
        b = raw(str(p))
        assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], p
        total += len(b)
    assert len(seen) == m["payloadFiles"] == 371 and total == m["payloadBytes"] == 2715343
    assert git("rev-parse", CODE + "^{tree}") == git("rev-parse", INTEGRATED + "^{tree}")
    source = obj("final-source-manifest.json")
    endings = []
    for f in source["files"]:
        assert git("rev-parse", CODE + ":" + f["path"]).decode().strip() == f["gitBlob"]
        kind = equivalent(git("show", CODE + ":" + f["path"]), f)
        if kind != "exact": endings.append({"path": f["path"], "conversion": kind})
    builds = []
    for stage in ["primitive-build02", "native-build02", "product-build03"]:
        rows = obj(stage + "/source-manifest.json")
        rust = [f for f in rows if f["path"].endswith(".rs")]
        assert len(rust) == 26
        for f in rust: equivalent(git("show", CODE + ":" + f["path"]), f)
        builds.append({"stage": stage, "rustFilesVerified": len(rust)})
    result = obj("result.json")
    assert not result["allAccepted"] and not result["productionExecuted"] and not result["fourNativeTraitsImplemented"]
    assert result["tests"] == {"libraryPassed": 6, "nativeCapturePassed": 1, "isolatedPrerequisiteFailed": 1, "isolatedTestRawExit": 101, "controllerExit": 1}
    actual = obj("public-raw/out01/native/wl2-independent-prerequisites.json")
    assert actual["elevated"] is False and actual["sid"].endswith("-1046")
    assert actual["security"]["rawWin32"] == 1314 and not actual["security"]["completeSecurityReadable"]
    assert not actual["ownerOuterJob"]["outerJobSafe"]
    assert actual["originalBefore"] == actual["originalAfter"] and not actual["originalBefore"]["completeSecurityProven"]
    assert not actual["productionExecuted"] and not result["cleanupComplete"]
    states = obj("status-matrix.json")["items"]
    assert len(states) == 36
    return {"receiveVerified": True, "allAccepted": False, "productionExecuted": False,
            "code": CODE, "docs": git("rev-parse", DOCS).decode().strip(), "payloadFiles": len(seen),
            "payloadBytes": total, "netTreeMatches": True, "executedRustInputs": builds,
            "sourceLineEndings": endings, "statusRows": states,
            "limits": ["Offline replay only, no new Windows execution", "O/H/S are cfg(test) prerequisites, not concrete production trait implementations", "Partial SD read and fixture query roles are not native restore/exit authority", "Current ordinary complete descriptor and outer-Job continuity unresolved; cleanup partial"]}

if __name__ == "__main__":
    print(json.dumps(verify(), ensure_ascii=False, indent=2))
