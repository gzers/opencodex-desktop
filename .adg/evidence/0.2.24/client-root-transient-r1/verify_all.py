from pathlib import Path
import tempfile,shutil,subprocess,json,hashlib,sys,gzip
b=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if (b/"manifest.json").exists():
 for row in json.loads((b/"manifest.json").read_text())["files"]:
  p=b/row["path"];assert p.stat().st_size==row["bytes"] and sha(p)==row["sha256"]
s=json.loads((b/"source-final.json").read_text())
if (b/"payload-storage.json").exists():
 for row in json.loads((b/"payload-storage.json").read_text())["payloads"]:
  compressed=(b/row["storedPath"]).read_bytes();raw=gzip.decompress(compressed)
  assert len(compressed)==row["storedBytes"] and hashlib.sha256(compressed).hexdigest()==row["storedSha256"]
  assert len(raw)==row["originalBytes"] and hashlib.sha256(raw).hexdigest()==row["originalSha256"]
for label in ["official-delivery","official-delivery-late"]:
 n=json.loads((b/label/"source.json").read_text());assert sorted(s["files"],key=lambda r:r["path"])==sorted(n["files"],key=lambda r:r["path"]) and n["binarySha256"]==s["actualLibraryTestBinarySha256"]
assert "test result: ok. 22 passed; 0 failed;" in (b/"root-transient-delivery-tests.log").read_text()
assert json.loads((b/"strict-clippy-delivery.json").read_text())["exitCode"]==0
with tempfile.TemporaryDirectory(prefix="ocxd024-transient-offline-replay-") as d:
 scratch=Path(d)/"evidence"
 shutil.copytree(b,scratch,ignore=lambda directory,names:[n for n in names if n.startswith("offline-") or n=="manifest.json"])
 for script in ["verify_storage_delivery.py","verify_native_delivery.py"]:subprocess.run([sys.executable,str(scratch/script)],check=True,stdout=subprocess.DEVNULL)
 for name in ["verified-storage-delivery.json","verified-native-delivery.json"]:assert json.loads((scratch/name).read_text())==json.loads((b/name).read_text())
print(json.dumps({"accepted":False,"sourceBinaryAndRawBytesVerified":True,"affectedFunctionsPassed":22,"finalOfficialCompositionCases":2,"scope":"storage/native-lock integration with synthetic lifecycle permit; production Start/Stop not accepted"},indent=2))
