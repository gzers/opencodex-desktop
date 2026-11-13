from pathlib import Path
import json,re,hashlib
b=Path(__file__).resolve().parent
if (b/"manifest.json").exists():
 for row in json.loads((b/"manifest.json").read_text())["files"]:
  p=b/row["path"];assert p.stat().st_size==row["bytes"] and hashlib.sha256(p.read_bytes()).hexdigest()==row["sha256"]
functions=set()
for name in ["root-stop-registered-tests","stop-binding-regression","root-movement-regression"]:
 assert json.loads((b/(name+".json")).read_text())["exitCode"]==0
 log=(b/(name+".log")).read_text();assert "test result: ok." in log and " passed; 0 failed;" in log
 functions.update(line.split(" ... ",1)[0][5:] for line in log.splitlines() if line.startswith("test ") and " ... " in line)
assert len(functions)==32
events={}
for line in (b/"root-stop-registered-tests.log").read_text().splitlines():
 for marker in ["ROOT_STOP_CURRENT_ABSENCE_EVIDENCE ","ROOT_STOP_BINDING_EVIDENCE ","ROOT_STOP_REGISTRATION_EVIDENCE "]:
  if marker in line:events[marker.strip()]=json.loads(line.split(marker,1)[1])
assert len(events)==3 and all(v["accepted"] is False for v in events.values())
v=events["ROOT_STOP_CURRENT_ABSENCE_EVIDENCE"];assert v["kernelServiceNormalExit"] and v["kernelCapturedTcpPorts"]==2 and v["secondaryPortRefused"] and v["unknownPortRefused"] and v["managerAndRecordReappearanceRefused"] and v["ownerChangeRefused"] and v["syntheticReferenceWrapperNotOfficialStop"]
v=events["ROOT_STOP_REGISTRATION_EVIDENCE"];assert v["headCopyWithoutActualRegistrationRefused"] and v["exactSourceEventRequired"] and v["currentRegistrationVerifiedAcrossMove"] and v["staleAfterCloseRefused"] and v["storageOnlyNoNormalStopPermission"]
v=events["ROOT_STOP_BINDING_EVIDENCE"];assert v["sourceAndFramesVerifiedBeforeAndAfterMove"] and v["logicalScopeUnchanged"] and v["wrongPlanScopeLiveAndNewRootRefused"] and v["syntheticFramesNoStopPermission"]
assert json.loads((b/"strict-clippy-corrected.json").read_text())["exitCode"]==0
assert json.loads((b/"root-stop-tests.json").read_text())["exitCode"]==101 and json.loads((b/"strict-clippy.json").read_text())["exitCode"]==101
print(json.dumps({"accepted":False,"verifiedRecordedAssertions":True,"uniqueAffectedFunctions":32,"scope":"kernel listener/exit reference predicates, independently bound source storage and current Index registration; no official complete missing-root lifecycle acceptance"},indent=2))
