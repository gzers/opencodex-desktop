from pathlib import Path
import json
b=Path(__file__).resolve().parent
load=lambda n:json.loads((b/n).read_text())
s=load("source-final.json");native=load("official-corrected/source.json")
assert sorted(s["files"],key=lambda r:r["path"])==sorted(native["files"],key=lambda r:r["path"])
assert s["actualExecutedLibraryBinarySha256"]==native["binarySha256"]
assert load("official/runner-summary.json")["exitCode"]==101
assert "initial-check" in load("direct-probe-result.json")["stdout"]
direct=load("direct-distinct-targets-result.json")
assert direct["helperSha256"]==s["currentHelperSha256"] and direct["bothExitCodes"]==[0,0]
assert direct["bothHoldersReady"] and direct["codexNormalReleaseLeavesGrokHeld"] and direct["grokNormalRelease"] and direct["bothConfigBytesUnchanged"]
assert "test result: ok. 19 passed; 0 failed; 5 ignored;" in (b/"holder-final-tests.log").read_text()
assert load("holder-final-tests.json")["exitCode"]==0 and load("strict-clippy-final.json")["exitCode"]==0
r=load("official-corrected/native-summary.json");assert r["accepted"] is False and r["result"]=="passed"
scenes=[row for row in r["rows"] if row.get("result")=="passed"]
assert {row["scene"] for row in scenes}=={"normal","eof-prepared","eof-moved","eof-reanchored","replacement-lock","new-user-root","wrong-command"}
assert all(row["elapsedMs"]<10000 and row["configBytesUnchanged"] and row["noServiceOrProductionRootPermission"] for row in scenes)
preserved=next(row for row in r["rows"] if row["scene"]=="replacement-preservation");assert preserved["bothUnknownAndOriginalRetained"] and preserved["sameTokenCannotReplaceIdentity"]
exitrow=next(row for row in r["rows"] if row["scene"]=="actual-parent-exit91");assert exitrow["actualWaitExitCode"]==91 and exitrow["holderGoneByESRCH"] and exitrow["lockReleasedAtPreparedDestination"] and exitrow["noCleanupSignal"]
assert (b/"official-corrected/preserved-new-user-data").read_bytes()==b"external user"
lock=load("official-corrected/preserved-original-lock.json");assert len(lock["token"])==16 and lock["pid"]>0
for label in ["official","official-corrected"]:
 runner=load(label+"/runner-summary.json");assert runner["exitCode"] is not None and not runner["ownedProcessesRemaining"]
 after=load(label+"/global-l-after.json");assert after["snapshot"]==load(label+"/global-l-before.json") and after["exclusiveReacquiredAndReleased"]
assert load("cleanup.json")["allNativeAndDirectProcessHandlesTerminal"]
print(json.dumps({"accepted":False,"verified":True,"routineFunctionsPassed":19,"routineProbeEntrypointsIncluded":2,"officialIndependentScenesPassed":7,"actualParentExit91Passed":True,"directCodexGrokSeparationPassed":True,"maxSceneMs":max(row["elapsedMs"] for row in scenes),"firstFailureRetained":True,"scope":"Official lock holder continuity and release with synthetic roots, no service lifecycle or complete root recovery acceptance"},indent=2))
