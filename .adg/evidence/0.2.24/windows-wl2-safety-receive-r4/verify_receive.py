#!/usr/bin/env python3
"""Read-only replay of pinned Windows evidence; not a native acceptance run."""
import hashlib
import json
import pathlib
import subprocess
from collections import Counter
CODE = "c47953ccfe4c66a976285eacffd691e3a3b0c01b"
DOCS = "fe1a54f6e"
BASE = "a6d9184b8"
PREFIX = ".adg/evidence/0.2.24/windows-dev/"
ROOT = pathlib.Path(__file__).resolve().parents[4]

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def verify():
    paths = git("diff", "--name-only", BASE, DOCS).decode().splitlines()
    assert len(paths) == 3601
    assert all(p.startswith(PREFIX) or p == "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md" for p in paths)
    cache = {}
    process = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    for p in paths:
        process.stdin.write((DOCS + ":" + p + "\n").encode())
        process.stdin.flush()
        h = process.stdout.readline().split()
        assert h[1] == b"blob"
        cache[p] = process.stdout.read(int(h[2]))
        assert process.stdout.read(1) == b"\n"
    process.stdin.close()
    assert process.wait() == 0
    manifests = []
    for path in paths:
        p = pathlib.PurePosixPath(path)
        if len(p.parts) != 6 or p.name not in ["manifest.json", "evidence-manifest.json"]:
            continue
        m = json.loads(cache[path])
        seen = set()
        total = 0
        for f in m["files"]:
            relative = pathlib.PurePosixPath(f["path"])
            assert not relative.is_absolute() and ".." not in relative.parts and str(relative) not in seen
            seen.add(str(relative))
            b = cache[str(p.parent / relative)]
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], (path, f["path"])
            total += len(b)
        if "payloadFiles" in m: assert len(seen) == m["payloadFiles"]
        if "payloadBytes" in m: assert total == m["payloadBytes"]
        manifests.append({"root": p.parent.name, "payloads": len(seen), "bytes": total})
    assert len(manifests) == 11
    latest = PREFIX + "wl2-current-reference-20261115-r1/"
    source = json.loads(cache[latest + "final-source-manifest.json"])
    source_rows = source["files"]
    for f in source_rows:
        b = git("show", CODE + ":" + f["path"])
        if hashlib.sha256(b).hexdigest() != f["sha256"]:
            b = b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
        assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], f["path"]
    execution = []
    for stage in ["library02", "product01"]:
        raw = json.loads(cache[latest + stage + "/source-manifest.json"])
        rows = raw if isinstance(raw, list) else raw["files"]
        rust = [f for f in rows if f["path"].endswith(".rs")]
        assert len(rust) == 39
        for f in rust:
            b = git("show", CODE + ":" + f["path"])
            if hashlib.sha256(b).hexdigest() != f["sha256"]:
                b = b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], f["path"]
        execution.append({"stage": stage, "matchedRustFiles": len(rust)})
    result = json.loads(cache[latest + "result.json"])
    for key in ["allAccepted", "productionExecuted", "actualGUI", "officialOwnerCreated", "OriginalSessionIssued", "ClosedEligibilityIssued", "fullSecurityProductionPositive"]: assert result[key] is False
    assert result["MSVCProduct"]["exitCode"] == 0 and result["MSVCProduct"]["warnings"] == 107
    assert result["tests"]["library02Final"] == 8
    h = json.loads(cache[PREFIX + "wl2-h-desktop-result-20261115-r1/result.json"])
    assert h["HEmptyHostComponentPassed"] and not h["actualGUI"] and not h["officialOwnerCreated"]
    native = h["newEmptyHost"]
    for k in ["originalCreationHandleRetained", "bootstrapNoJob", "suspendedBeforeResumeNoJob", "sameOriginalAcrossManagers", "explicitExitObserved"]: assert native[k]
    assert native["rawHostExitCode"] is None and native["managerGenerations"] == [1,2,3,4]
    assert h["ordinarySID"].endswith("-1003") and h["session"] == 7
    decision = json.loads(cache[PREFIX + "wl2-sd-contract-decision-20261115-r1/user-decision.json"])
    assert decision["productionStart"] == "refused under unchanged r3"
    request = json.loads(cache[latest + "environment-decision-request.json"])
    assert request["noCurrentAuthorizationToWeaken"] and not request["allAccepted"]
    existing = git("ls-tree", "-r", "--name-only", DOCS, PREFIX).decode().splitlines()
    shared = [p for p in existing if p.endswith("shared-minimum-request.json")]
    assert len(shared) == 2
    return {"receiveVerified": True, "allAccepted": False, "codeContribution": CODE,
        "docsContribution": git("rev-parse", DOCS).decode().strip(), "importedNetFiles": len(paths),
        "manifests": manifests, "totalPayloadFiles": sum(m["payloads"] for m in manifests),
        "totalPayloadBytes": sum(m["bytes"] for m in manifests), "latestSourceFilesVerified": len(source_rows),
        "latestExecutionInputs": execution, "latestResult": result, "HComponentResult": h,
        "latestDecisionRequest": request, "sharedMinimumRequestFiles": shared,
        "limits": ["Hash/source receipt and selected fact checks only; not a complete audit of 5573 changed Windows lines", "Contribution fetched and preserved on Windows branch; not merged wholesale to main by this safety-contract decision", "Partial diagnostic/CAS/file-move proofs are not complete original-tree recovery or ClosedEligibility", "Existing shared-minimum requests are historical; newest minimum request is environment-decision-request.json", "No native Windows or actual GUI execution on Mac"]}

if __name__ == "__main__":
    print(json.dumps(verify(), ensure_ascii=False, indent=2))
