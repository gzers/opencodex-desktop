"""Verify reused foreground entries and refusal results, without running them."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
DIAGNOSTIC = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")


def read(p):
    return json.loads(p.read_text(encoding="utf-8-sig"))


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def verify():
    contract = read(ROOT / "contract.json")
    assert contract["candidateSource"] == "ce670ce76bb1c29f70989ee54838d5b31e97059c"
    assert contract["candidateExeSha256"] == "4fe09bce265b51860da17f2cd1466a50430513e5ed29050f85caf5d913652bb5"
    assert contract["baselineSource"] == "de6862d40389d527527b9db8ef52e186a58b625f"
    assert sha(Path(contract["candidateExecutable"])) == contract["candidateExeSha256"]
    assert sha(Path(contract["baselineExecutable"])) == contract["baselineExeSha256"]
    assert contract["minimumCompletedCounterSpanSeconds"] == 600 and contract["settleSecondsPerVersion"] == 60
    assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
    assert contract["minimumCounterStartRemainingSeconds"] == 750
    assert not contract["priorSamplesUsedInNewComparison"] and not contract["dpiAcceptanceRequired"]
    assert contract["formalPairAllowedOnlyBecauseProductChanged"]
    assert sha(Path(contract["candidateManifestPath"])) == contract["candidateManifestSha256"]
    assert sha(Path(contract["candidateVerificationPath"])) == contract["candidateVerificationSha256"]
    candidate = read(Path(contract["candidateVerificationPath"]))
    assert candidate["status"] == "verified-exact-ce-candidate"
    assert candidate["source"] == contract["candidateSource"]
    assert sha(ROOT / "retained-process-times.ps1") == contract["kernelTimesMethodSha256"]
    assert sha(ROOT / "preparation-readback.json") == contract["preparationReadbackSha256"]
    for path, expected in contract["methodWorkingFileSha256"].items():
        assert sha(DIAGNOSTIC / path) == expected
    for row in read(ROOT / "method-origins.json"):
        assert sha(ROOT / row["path"]) == row["preparedSha256"]
        assert sha(Path(row["origin"])) == row["originSha256"]
    for row in read(ROOT / "syntax-checks.json"):
        assert row["syntaxPassed"] and sha(ROOT / row["path"]) == row["sha256"]
    guards = read(ROOT / "guards/guard-checks.json")
    assert len(guards["cases"]) == 4 and guards["beforeAfterProtectionEqual"]
    assert not guards["humanReceiptCreated"] and not guards["productStarted"]
    assert not guards["configurationMoved"] and not guards["registryOrAclChanged"]
    for case in guards["cases"]:
        assert case["missingHumanReceiptRefused"] and case["exitCode"] != 0
        assert sha(ROOT / "guards" / case["log"]) == case["logSha256"]
    assert read(ROOT / "guards/before-protection.json") == read(ROOT / "guards/after-protection.json")
    readback = read(ROOT / "preparation-readback.json")
    assert readback["nativeCount"] == 0 and not readback["productStarted"] and not readback["humanWindowGranted"]
    assert readback["counterMethodsUnchanged"] and not readback["originalProfilesMoved"]
    assert readback["fixtureReadback"][0]["preferencesSha256"] == "2d612148a55152e05047d1f13c4587a3b75f2740b1a51cc2c0f64f30a767495e"
    assert readback["fixtureReadback"][1]["preferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert readback["startupPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert readback["taskFileSha256"] == sha(PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json")
    assert all(row["runtime"]["source"] == "unresolved" and not row["runtime"]["path"]
               for row in readback["fixtureReadback"])
    assert not read(ROOT / "human-window.template.json")["explicitHumanStart"]
    assert not (ROOT / "human-window.json").exists()
    if (ROOT / "manifest.json").exists():
        for row in read(ROOT / "manifest.json")["files"]:
            path = ROOT / row["path"]
            assert path.stat().st_size == row["bytes"] and sha(path) == row["sha256"]
    return {"status": "prepared-same-method-019-ce-rss-pair", "source": contract["candidateSource"],
            "estimatedForegroundMinutes": 27, "settleSecondsEach": 60, "counterSpanSecondsEach": 600,
            "rssBudgetMiB": 10, "cpuBudgetPercentagePoints": 0.5, "originalCollectorBytesUnchanged": True,
            "missingHumanEntriesRefused": 4, "humanWindowGranted": False, "productStarted": False,
            "profileMoved": False, "rssBenefitMeasured": False, "fullPerformanceGatePassed": False}


if __name__ == "__main__":
    print(json.dumps(verify()))
