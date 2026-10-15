"""Prepare one fresh candidate observation, preserving the completed baseline."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OLD = ROOT.parent / "OCX-0110-20261016-WIN-IDLE-8BE-019-SETTLED"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


assert not (ROOT / "contract.json").exists(), "Do not overwrite an existing attempt"
old_contract = read(OLD / "contract.json")
baseline = read(OLD / "baseline019-measurement.json")
assert baseline["observedCounterSpanSeconds"] >= 600
assert baseline["lostForegroundSamples"] == baseline["nativeInstanceViolations"] == 0
assert baseline["childDiscoveryReadFailures"] == 0
assert read(OLD / "baseline019-exit.json")["exitCode"] == 0
assert "baseline019" in read(OLD / "terminal.json")["completed"]
preparation = read(OLD / "preparation.original.json")
preferences = Path(preparation["sourceRoot"]) / "manager-state/preferences.json"
assert digest(preferences) == preparation["fixturePreferencesSha256"]

contract = dict(old_contract)
contract.update({
    "scope": "fresh candidate-only resampling; reuse the completed same-host baseline explicitly",
    "order": ["candidate0110"],
    "reusedBaselineEvidence": OLD.name,
    "baselineCompletedAtUtc": baseline["atUtc"],
    "pairedVersionsObservedWithoutInterveningInterruption": False,
    "candidateResampledAfterForegroundInterruption": True,
    "interruptedCandidateSamplesUsed": False,
    "readinessObserver": "Computer Use sky before collection, no UI observation during collection",
    "readinessObserverDiffersFromBaseline": True,
    "closeMethod": "Computer Use Alt+F4 after collection; no collector UI input or force kill",
})
(ROOT / "contract.json").write_text(
    json.dumps(contract, indent=2) + "\n", encoding="utf-8")
bindings = {}
for name in (
    "baseline019-measurement.json", "baseline019-samples.original.jsonl",
    "baseline019-exit.json", "idle-baseline019-launch.json",
    "idle-baseline019-settled-ready.json", "contract.json",
    "preparation.original.json", "terminal.json",
    "candidate0110-samples.original.jsonl", "interrupted-normal-stop.json",
):
    path = OLD / name
    bindings[name] = {"bytes": path.stat().st_size, "sha256": digest(path)}
(ROOT / "reused-evidence-bindings.json").write_text(
    json.dumps(bindings, indent=2) + "\n", encoding="utf-8")
(ROOT / "preferences-before.original.json").write_bytes(preferences.read_bytes())

method = (OLD / "measure.ps1").read_text(encoding="utf-8-sig")
method = method.replace(
    '[DllImport("user32.dll")] public static extern bool IsIconic(IntPtr w);',
    '[DllImport("user32.dll")] public static extern bool IsIconic(IntPtr w);\n'
    ' [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);\n'
    ' [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}\n'
    ' [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr w,out Rect r);')
start = method.index("        Start-Sleep -Seconds 4")
end = method.index("        $taskHandles=@{}", start)
method = method[:start] + r"""
        $taskReadyPath=Join-Path $PSScriptRoot 'sky-settled-ready.json'
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$taskVersion;phase='waiting-for-sky-readiness';pid=$taskProcess.Id} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'progress.json') -Encoding utf8
        $taskReadinessTimer=[Diagnostics.Stopwatch]::StartNew()
        while(!(Test-Path -LiteralPath $taskReadyPath)){
            if($taskProcess.HasExited){throw 'Product exited before readiness'}
            if($taskReadinessTimer.Elapsed.TotalSeconds -gt 240){throw 'External readiness timed out; do not collect'}
            Start-Sleep -Milliseconds 250
        }
        $taskReady=Get-Content -LiteralPath $taskReadyPath -Raw -Encoding utf8 | ConvertFrom-Json
        if($taskReady.accessibility.tree -notmatch '尚未接入 OpenCodex'){throw 'Observed scenario differs'}
        if(([datetimeoffset]$taskReady.observedAtUtc-[datetimeoffset]$taskToken.creationTimeUtc).TotalSeconds -lt 60){throw 'Fresh process did not settle for 60 real seconds'}
        $taskWindow=[IntPtr]$taskReady.window.id
        $taskOwner=[uint32]0
        [void][IdleNative]::GetWindowThreadProcessId($taskWindow,[ref]$taskOwner)
        if($taskOwner -ne $taskProcess.Id){throw 'Observed window does not belong to the launched process'}
        $taskClient=[IdleNative+Rect]::new()
        if(![IdleNative]::GetClientRect($taskWindow,[ref]$taskClient) -or
           $taskClient.Right-$taskClient.Left -ne 2360 -or $taskClient.Bottom-$taskClient.Top -ne 1520){throw 'Paired client canvas dimensions differ'}
        if([IdleNative]::GetForegroundWindow() -ne $taskWindow){throw 'Observed target is not foreground'}
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');hwnd=$taskWindow.ToInt64();ownerPid=$taskOwner;clientWidth=$taskClient.Right-$taskClient.Left;clientHeight=$taskClient.Bottom-$taskClient.Top;settledSeconds=([datetimeoffset]$taskReady.observedAtUtc-[datetimeoffset]$taskToken.creationTimeUtc).TotalSeconds;uiInputDuringCollection=$false} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'readiness-binding.json') -Encoding utf8
""" + method[end:]
start = method.index("        & $taskShell", method.index("'shortLivedChildrenFullyCovered'") if "'shortLivedChildrenFullyCovered'" in method else method.index("$taskController.Refresh()"))
end = method.index("        foreach($taskItem in $taskHandles.Values)", start)
method = method[:start] + """
        # Product is intentionally left alive for a bound Computer Use normal close.
""" + method[end:]
start = method.index("    if($taskProcess -and $taskToken")
end = method.index("    [ordered]@{atUtc=", start)
# Remove the entire old exception-input block, including its nested output.
end = method.index("    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=", end)
method = method[:start] + method[end:]
assert "native.ps1" not in method
assert "PostMessage" not in method
assert "AutomationElement" not in method
(ROOT / "measure.ps1").write_text(method, encoding="utf-8")
(ROOT / ".gitattributes").write_text("* -text\n", encoding="utf-8")
print(json.dumps({"prepared": True, "baselineReused": True,
                  "candidateRunsAuthorized": 1, "oldSamplesUsed": False}))
