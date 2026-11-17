#!/usr/bin/env python3
"""Pinned read-only subset replay: current failure, cost and old authorization boundary."""
import hashlib
import json
import pathlib
import subprocess
CODE = "44bb9bdbabc78f1754b9df135f3d183205fcb9ec"
DOCS = "e34f3fe8a"
BASE = "578230717"
PREFIX = ".adg/evidence/0.2.24/windows-dev/"
ROUNDS = ["wl2-r6-fresh-existing-fixture-20261117-receive-r1", "wl2-r6-source-scan-cost-20261117-r1", "wl2-r6-authorized-failure-cleanup-20261117-receive-r1", "wl2-r6-failure-archive-r3-20261117-receive-r1"]
ROOT = pathlib.Path(__file__).resolve().parents[4]
def git(*args): return subprocess.check_output(["git", "-C", str(ROOT), *args])
def verify():
    all_paths = git("diff", "--name-only", BASE, DOCS).decode().splitlines()
    paths = [p for p in all_paths if any(p.startswith(PREFIX + n + "/") for n in ROUNDS) or p == "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md"]
    assert len(all_paths) == 79018 and len(paths) == 555
    proc = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    data = {}
    for p in paths:
        proc.stdin.write((DOCS + ":" + p + "\n").encode()); proc.stdin.flush()
        h = proc.stdout.readline().split(); assert h[1] == b"blob"
        data[p] = proc.stdout.read(int(h[2])); assert proc.stdout.read(1) == b"\n"
    proc.stdin.close(); assert proc.wait() == 0
    def obj(round_, name): return json.loads(data[PREFIX + round_ + "/" + name])
    manifests = []
    for n in ROUNDS:
        seen = set(); total = 0
        for f in obj(n, "payload-manifest.json")["files"]:
            p = pathlib.PurePosixPath(f["path"])
            assert not p.is_absolute() and ".." not in p.parts and str(p) not in seen
            seen.add(str(p)); b = data[PREFIX + n + "/" + str(p)]
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], str(p)
            total += len(b)
        manifests.append({"round": n, "payloads": len(seen), "bytes": total})
    request = obj(ROUNDS[0], "minimum-recovery-interface-request.json")
    f = request["facts"]
    assert f["brokerPid"] == 75744 and f["brokerFiletime"] == "134393656723346634"
    assert f["launcherPid"] == 21500 and f["launcherFiletime"] == "134393657318681068"
    assert not f["creatorResumed"] and not f["originalPhaseTagIssued"]
    assert f["originalsMoved"] == 3 and f["workFilesCreated"] == 3
    assert f["canonicalSequence"] == 9 and f["canonicalLease"] == "Released"
    result = obj(ROUNDS[0], "result.json")
    assert result["startElapsedMs"] == 30699 and result["originalBudgetMs"] == 30000
    assert not result["trace"]["creatorResumed"] and result["trace"]["family"]["active"] == 1
    suffix = "raw-public/out01/raw-export/D/official024/windows/7e3d4697-dd04-4d8a-b40b-f4468fab88c0/"
    manifest = obj(ROUNDS[0], suffix + "manifest.json")
    current = obj(ROUNDS[0], suffix + "current.json")
    assert current["lifecycle"] == "Preparing" and current["leaseState"] == "Released" and current["sequence"] == 9
    assert current["readyGeneration"] is None and current["closed"] is None
    mb = data[PREFIX + ROUNDS[0] + "/" + suffix + "manifest.json"]
    assert hashlib.sha256(mb).hexdigest() == current["manifestDigest"]
    originals = []
    for root in manifest["storagePlan"]["roots"]:
        for node in root["nodes"]:
            if node["kind"] == "CustodiedFile":
                originals.append({"root": root["root"], "custody": root["custody"], "custodyId": root["custodyId"], **{k:node[k] for k in ["nodeId", "relative", "fileId", "length", "contentDigest", "target", "parentId"]}})
    assert sorted((n["relative"], n["length"]) for n in originals) == [("auth.json", 3), ("config.json", 3), ("config.toml", 51)]
    cost = obj(ROUNDS[1], "cost-summary.json")
    assert cost["fullImageMs"] == [6079.433,6353.553,4805.45,8928.7]
    assert not cost["actualR2CausalAttributionProven"] and not cost["actualOrdinaryStageTimings"]
    perf = obj(ROUNDS[1], "shared-performance-request.json")
    assert not perf["productionChecksChanged"] and not perf["nativeSceneTouched"]
    return {"receiveVerified": True, "allAccepted": False, "code": CODE, "docs": git("rev-parse", DOCS).decode().strip(),
        "remoteNetFiles": len(all_paths), "selectedNetFiles": len(paths), "manifests": manifests,
        "payloadFiles": sum(m["payloads"] for m in manifests), "payloadBytes": sum(m["bytes"] for m in manifests),
        "recoveryRequest": request, "currentTip": current, "originalFiles": originals,
        "manifestSha256": current["manifestDigest"], "performance": perf, "costSummary": cost,
        "limits": ["Only four named rounds and handoff imported; other 50-round net history remains on exact remote docs Git commit and is not declared integrated/reviewed", "Manifest/file IDs here are locator data, not native restore or process-ownership authority", "No fresh Windows live state, archive or credential read performed on Mac", "Old 50580/54692 external cleanup consent is separate; does not cover 21500/75744", "Copy benchmark is not production causal timing or release acceptance", "No loaded-host recovery implementation/NativeProof/normal Stop/Closed minted by this decision"]}
if __name__ == "__main__": print(json.dumps(verify(), ensure_ascii=False, indent=2))
