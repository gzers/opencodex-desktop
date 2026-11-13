from pathlib import Path
import json,re
b=Path(__file__).resolve().parent
load=lambda p:json.loads((b/p).read_text())
source=load("source-final.json");native_source=load("official/source.json")
assert sorted(source["files"],key=lambda r:r["path"])==sorted(native_source["files"],key=lambda r:r["path"])
assert source["actualLibraryTestBinarySha256"]==native_source["binarySha256"]==source["officialRoundBinarySha256"]
log=(b/"handoff-source-final-tests.log").read_text()
assert "test result: ok. 19 passed; 0 failed; 1 ignored;" in log
assert (b/"handoff-source-final-tests.json").exists() and load("handoff-source-final-tests.json")["exitCode"]==0
summary=load("official/native-summary.json");runner=load("official/runner-summary.json")
assert summary["accepted"] is False and summary["result"]=="passed" and summary["elapsedMs"]<summary["budgetMs"]==10000
assert summary["allNativeCoordinatorsExcludedAcrossMove"] and summary["originalRootAbsent"] and summary["archiveExists"]
assert runner["exitCode"]==0 and not runner["ownedProcessesRemaining"]
assert "test result: ok. 1 passed; 0 failed;" in (b/"official/native.stdout.log").read_text()
for state in ["blocked","released"]:
 row=load("official/official-writers-"+state+".stdout.log")
 assert row["expected"]==state and row["forgedRefused"]
 if state=="blocked":assert row["k"]=={"kind":"unavailable","reason":"busy","callbackRan":False} and row["l"]=={"kind":"busy","callbackRan":False} and row["before"]==row["after"]
 else:assert row["k"]["kind"]=="completed" and row["l"]=={"kind":"completed","callbackRan":True} and row["revokedRefused"]
after=load("official/global-l-after.json");assert after["sameIdentityModeBytes"] and after["exclusiveReacquiredAndReleased"] and after["snapshot"]==load("official/global-l-before.json")
assert load("strict-clippy.json")["exitCode"]==0 and load("cleanup.json")["allProcessHandlesTerminal"]
print(json.dumps({"accepted":False,"verified":True,"ordinaryFunctionsPassed":19,"explicitOfficialRoundPassed":1,"routineProbeEntryPointsIncluded":2,"nativeCoordinatorBudgetMs":summary["elapsedMs"],"scope":"Source/binary correspondence, recorded cross-process assertions and actual official coordinator/writer evidence only; no service Stop or whole-root permission"},indent=2))
