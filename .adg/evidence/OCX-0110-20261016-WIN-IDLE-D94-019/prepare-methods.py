"""Prepare the bounded same-window pair without launching an application."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent / "OCX-0110-20261016-WIN-IDLE-8BE-RESAMPLE"
SETTLED = ROOT.parent / "OCX-0110-20261016-WIN-IDLE-8BE-019-SETTLED"
assert not (ROOT / "prepare.ps1").exists(), "Do not overwrite a prepared attempt"
prepare = (SETTLED / "prepare.ps1").read_text(encoding="utf-8-sig")
prepare = prepare.replace("OCX-0110-20261015-WIN-8BE-CANDIDATE",
                          "OCX-0110-20261016-WIN-D94-CANDIDATE")
prepare = prepare.replace("8be87e2d54a895d4fe02e31300100a1ffed90121",
                          "d94c215d869c0560717d79dc02881ce8858afe88")
prepare = prepare.replace("idle-pair-8be-settled-20261016", "idle-pair-d94-20261016")
prepare = prepare.replace("same-host fresh-process foreground idle subset after color-only buffer optimization",
                          "same-host fresh-process foreground idle pair after final-target mesh allocation fix")
(ROOT / "prepare.ps1").write_text(prepare, encoding="utf-8")
measure = (OLD / "measure.ps1").read_text(encoding="utf-8-sig")
measure = "param([Parameter(Mandatory=$true)][ValidateSet('baseline019','candidate0110')][string]$Version)\n" + measure
measure = measure.replace("OCX-0110-20261015-WIN-8BE-CANDIDATE",
                          "OCX-0110-20261016-WIN-D94-CANDIDATE")
measure = measure.replace("foreach($taskVersion in $taskContract.order)", "foreach($taskVersion in @($Version))")
measure = measure.replace("'sky-settled-ready.json'", "($taskVersion+'-sky-settled-ready.json')")
measure = measure.replace("'readiness-binding.json'", "($taskVersion+'-readiness-binding.json')")
measure = measure.replace("'terminal.json'", "($taskVersion+'-terminal.json')")
measure = measure.replace("'failure.original.txt'", "($Version+'-failure.original.txt')")
# Include the root handle in the same retention rule as discovered descendants.
measure = measure.replace("$taskHandles=@{}", "[void]$taskProcess.SafeHandle\n        $taskHandles=@{}")
(ROOT / "measure.ps1").write_text(measure, encoding="utf-8")
observe = (OLD / "observe-exit.ps1").read_text(encoding="utf-8-sig")
observe = "param([Parameter(Mandatory=$true)][ValidateSet('baseline019','candidate0110')][string]$Version)\n" + observe
observe = observe.replace("'idle-candidate0110-launch.json'", "('idle-'+$Version+'-launch.json')")
observe = observe.replace("'terminal.json'", "($Version+'-terminal.json')")
observe = observe.replace("'exit-observer-ready.json'", "($Version+'-exit-observer-ready.json')")
observe = observe.replace("'candidate0110-exit.json'", "($Version+'-exit.json')")
(ROOT / "observe-exit.ps1").write_text(observe, encoding="utf-8")
(ROOT / "roles.ps1").write_bytes((SETTLED / "roles.ps1").read_bytes())
(ROOT / ".gitattributes").write_text("* -text\n", encoding="utf-8")
(ROOT / "method-admission.json").write_text(json.dumps({
    "source": "d94c215d869c0560717d79dc02881ce8858afe88",
    "candidateExeSha256": "b2284ba66a20c7ef11b61fea328f7aa960cd6530011d6e7a09fc78e782c5b224",
    "humanAuthorization": "开始前台验证",
    "maximumRunsPerVersion": 1, "measurementSecondsPerVersion": 600,
    "bothFreshProcesses": True, "bothSameReadinessObserver": "Computer Use sky",
    "settleSecondsMinimum": 60, "closeMethod": "bound Computer Use Alt+F4 after measurement",
    "priorBaselineRetained": True, "priorSamplesUsedInNewComparison": False,
    "reasonForNewBaseline": "Same foreground window and observer for the changed candidate; old baseline was several hours earlier.",
    "oldCollectorSha256": hashlib.sha256((OLD / "measure.ps1").read_bytes()).hexdigest(),
    "newCollectorSha256": hashlib.sha256((ROOT / "measure.ps1").read_bytes()).hexdigest(),
    "noUiInputDuringCollection": True, "noBuildOrEtwDuringCollection": True,
    "budgets": {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10},
    "fullPerformanceGatePassed": False,
}, indent=2) + "\n", encoding="utf-8")
print("Prepared version-scoped collection and normal-close methods; no application started.")
