#!/usr/bin/env python3
"""Pinned evidence replay; no Windows execution, writes or upstream requests."""
import hashlib
import json
import pathlib
import subprocess
CODE = "0549a660ee3f77ea243e798ec55b01f52545e492"
DOCS = "a7baa88e1"
BASE = "84ec482ee"
PREFIX = ".adg/evidence/0.2.24/windows-dev/"
ROOT = pathlib.Path(__file__).resolve().parents[4]

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def verify():
    paths = git("diff", "--name-only", BASE, DOCS).decode().splitlines()
    assert len(paths) == 5873
    assert all(p.startswith(PREFIX) or p == "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md" for p in paths)
    data = {}
    proc = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    for p in paths:
        proc.stdin.write((DOCS + ":" + p + "\n").encode()); proc.stdin.flush()
        head = proc.stdout.readline().split(); assert head[1] == b"blob"
        data[p] = proc.stdout.read(int(head[2])); assert proc.stdout.read(1) == b"\n"
    proc.stdin.close(); assert proc.wait() == 0
    def obj(relative): return json.loads(data[PREFIX + relative])
    manifests = []
    covered = set()
    for path in paths:
        p = pathlib.PurePosixPath(path)
        if len(p.parts) != 6 or p.name not in ["payload-manifest.json", "payload-inventory.json"]: continue
        total = 0; seen = set()
        for f in json.loads(data[path])["files"]:
            rel = pathlib.PurePosixPath(f["path"])
            assert not rel.is_absolute() and ".." not in rel.parts and str(rel) not in seen
            seen.add(str(rel)); full = str(p.parent / rel); b = data[full]
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], full
            total += len(b); covered.add(full)
        manifests.append({"round": p.parent.name, "payloads": len(seen), "bytes": total})
    assert len(manifests) == 14
    assert sum(m["payloads"] for m in manifests) == 5854
    assert sum(m["bytes"] for m in manifests) == 57943378
    def source_check(rows, sha):
        endings = []
        for f in rows:
            b = git("show", sha + ":" + f["path"])
            if hashlib.sha256(b).hexdigest() != f["sha256"]:
                b = b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"); endings.append(f["path"])
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], f["path"]
        return {"source": sha, "files": len(rows), "gitLFToExecutionCRLF": endings}
    source = obj("wl2-real-gui-profile-fix-20261115-r1/final-source.json")
    assert source["head"] == "6e312f02270f247d2f440ec369385e523a4a8043"
    checked = [source_check(rows, source["head"]) for rows in source["inputs"].values()]
    cleanup_source = obj("wl2-real-gui-success-cleanup-20261115-r1/final-source.json")
    assert cleanup_source["head"] == CODE and cleanup_source["productUnchanged"]
    checked.append(source_check(cleanup_source["files"], CODE))
    gui = obj("wl2-real-gui-fixed-run-20261115-r1/desktop-result.json")
    assert gui["actualGUIComponentPassed"] and gui["sameSidExportComplete"] and gui["exportedFiles"] == 32
    assert gui["result"]["elevated"] is False and gui["result"]["rawTestExit"] == 0
    assert gui["brokerRawExit"] is None
    for name in ["productionExecuted", "officialOwnerCreated", "allAccepted"]: assert gui[name] is False
    assert [m["generation"] for m in gui["modes"]] == [1,2,3,4]
    assert [m["managerRawExit"] for m in gui["modes"]] == [0,1,0,0]
    assert all(m["originalBrokerMatched"] and m["nativeWindowVerifiedByObserver"] for m in gui["modes"])
    cleanup = obj("wl2-real-gui-cleanup-receive-20261115-r1/receive-result.json")
    assert cleanup["inputBytesMatchPrepared"] and cleanup["successfulTaskSameSidCleanupPassed"]
    assert cleanup["removedProfileChild"]["files"] == 685 and cleanup["removedProfileChild"]["bytes"] == 119851895
    assert cleanup["archivedPublic"]["retained"] and cleanup["archivedPublic"]["bytes"] == 178669957
    for name in ["overallCleanupComplete", "productionExecuted", "officialOwnerCreated", "startOpen", "allAccepted", "privateArchivesIndependentlyReadByController"]: assert cleanup[name] is False
    assert cleanup["oldFailedBrokerExitUnconfirmed"] and not cleanup["historicalRejectedDeletionRetried"]
    request = obj("wl2-stop-restart-results-20261115-r1/shared-status-deadline-request-r2.json")
    routes = obj("wl2-working-snapshots-20261115-r1/shared-minimum-request.json")
    assert not request["allAccepted"] and not routes["allAccepted"]
    return {"receiveVerified": True, "allAccepted": False, "productionExecuted": False,
        "code": CODE, "docs": git("rev-parse", DOCS).decode().strip(), "netFiles": len(paths),
        "manifests": manifests, "payloadFiles": len(covered), "payloadBytes": sum(m["bytes"] for m in manifests),
        "outsidePayloadLists": [p for p in paths if p not in covered],
        "sourceChecks": checked, "actualGUIComponent": {k:v for k,v in gui.items() if k != "source"},
        "cleanup": cleanup, "sharedRequests": [routes, request],
        "limits": ["Source/hash/selected result receipt, not a full native code audit or new execution", "Latest 0549 only changes archive tests; product/source still 6e312", "Actual GUI plus empty broker is not official Owner/Job lifecycle acceptance", "NativeLease/full official side effects/CAP-trust populated positives and full production combinations remain incomplete", "Production gate remains refused; contribution not merged wholesale to main"]}

if __name__ == "__main__":
    print(json.dumps(verify(), ensure_ascii=False, indent=2))
