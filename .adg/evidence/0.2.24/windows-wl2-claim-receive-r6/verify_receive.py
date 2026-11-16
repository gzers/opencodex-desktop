#!/usr/bin/env python3
"""Pinned r5 Windows evidence readback; no native execution or product writes."""
import hashlib
import json
import pathlib
import subprocess
CODE = "3e19075846dfea81ba8392bb60a3d3031afbf116"
DOCS = "13e5c3223"
BASE = "676cec18a"
PREFIX = ".adg/evidence/0.2.24/windows-dev/"
ROOT = pathlib.Path(__file__).resolve().parents[4]

def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])

def verify():
    paths = git("diff", "--name-only", BASE, DOCS).decode().splitlines()
    assert len(paths) == 2659
    assert all(p.startswith(PREFIX) or p == "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md" for p in paths)
    raw = {}
    proc = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    for p in paths:
        proc.stdin.write((DOCS + ":" + p + "\n").encode()); proc.stdin.flush()
        h = proc.stdout.readline().split(); assert h[1] == b"blob"
        raw[p] = proc.stdout.read(int(h[2])); assert proc.stdout.read(1) == b"\n"
    proc.stdin.close(); assert proc.wait() == 0
    def obj(path): return json.loads(raw[PREFIX + path])
    manifests = []
    for p in paths:
        full = pathlib.PurePosixPath(p)
        if len(full.parts) != 6 or full.name != "payload-manifest.json": continue
        seen = set(); total = 0
        for f in json.loads(raw[p])["files"]:
            rel = pathlib.PurePosixPath(f["path"])
            assert not rel.is_absolute() and ".." not in rel.parts and str(rel) not in seen
            seen.add(str(rel)); b = raw[str(full.parent / rel)]
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], str(full.parent / rel)
            total += len(b)
        manifests.append({"round": full.parent.name, "payloads": len(seen), "bytes": total})
    assert len(manifests) == 4
    assert sum(m["payloads"] for m in manifests) == 2654
    assert sum(m["bytes"] for m in manifests) == 31356233
    source_checks = []
    for m in manifests:
        source = obj(m["round"] + "/final-source.json")
        rows = source.get("productInputs", source.get("product02Inputs"))
        assert rows is not None
        endings = []
        for f in rows:
            b = git("show", source["head"] + ":" + f["path"])
            if hashlib.sha256(b).hexdigest() != f["sha256"]:
                b = b.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"); endings.append(f["path"])
            assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"], f["path"]
        assert git("rev-parse", source["head"] + "^{tree}").decode().strip() == source["tree"]
        source_checks.append({"round": m["round"], "head": source["head"], "productInputFiles": len(rows), "gitLFToCapturedCRLF": endings})
    latest = "wl2-r5-claim-relocation-20261116-r1/"
    request = obj(latest + "claim-handoff-decision-request.json")
    facts = request["facts"]
    assert facts["baselineRootMove"] and facts["metadataOnlyRootMove"]
    assert not facts["readOnlyClaimFileShareDeleteRootMove"] and not facts["readOnlyClaimFilePlusMetadataRootMove"] and not facts["nativeClaimLeaseRootMove"]
    assert facts["failedNativeMoveRawWin32"] == 5
    assert request["currentBehavior"]["publicStartRefused"] and not request["currentBehavior"]["automaticClaimHandoffImplemented"]
    source = obj(latest + "claim-only-writer-source.json")
    assert source["officialTgzSha256"] == "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    for f in source["files"]:
        b = raw[PREFIX + latest + "official-source/" + f["path"]]
        assert len(b) == f["bytes"] and hashlib.sha256(b).hexdigest() == f["sha256"]
    assert not source["dynamicCredentialWriteExecuted"]
    result = obj(latest + "final-status.json")
    assert result["sourceHead"] == CODE
    assert result["runs"]["library08"]["passed"] == 10 and result["runs"]["library08"]["ignored"] == 2
    assert result["runs"]["product01"]["rawExit"] == 0
    assert result["runs"]["library05"]["rawExit"] == 101 and result["runs"]["library06"]["rawExit"] == 101
    # Every round preserves its own implementation/acceptance boundary.
    return {"receiveVerified": True, "allAccepted": False, "productionExecuted": False,
        "code": CODE, "docs": git("rev-parse", DOCS).decode().strip(), "netFiles": len(paths),
        "manifests": manifests, "payloadFiles": sum(m["payloads"] for m in manifests),
        "payloadBytes": sum(m["bytes"] for m in manifests), "sourceChecks": source_checks,
        "claimDecisionRequest": request, "claimOnlyWriterSource": source,
        "latestRuns": result["runs"], "latestMatrix": result["matrix"],
        "limits": ["Offline evidence/source review only; no Windows execution on Mac", "Open-claim root move is a retained failure, not a passed diagnostic", "N/L/K do not establish exclusion of the source-proven claim-only writer", "Contribution stays on Windows branch; no wholesale implementation merge", "No real original-package production/Closed sample positive acceptance; historical cleanup and B2/ordinary MSI remain unfinished"]}

if __name__ == "__main__":
    print(json.dumps(verify(), ensure_ascii=False, indent=2))
