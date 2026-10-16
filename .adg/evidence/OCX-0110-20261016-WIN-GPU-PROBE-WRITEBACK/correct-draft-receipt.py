"""Remove an authored EOF blank line and rebind the uncommitted receipt."""
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object

receipt_path = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-GPU-PROBE-20261016.json"
original = ROOT / "receipt-draft-revision1.original.json"
assert not original.exists(), "Do not overwrite the original draft"
raw = receipt_path.read_bytes()
receipt = json.loads(raw)
validate_object(receipt)
assert receipt["revision"] == 1 and receipt["result"] == "succeeded"
impact = receipt["fact_impacts"][0]
document = PROJECT / impact["path"]
before = document.read_bytes()
assert hashlib.sha256(before).hexdigest() == impact["after_digest"]
assert before.endswith(b"\n\n") and b"\r\n" not in before
original.write_bytes(raw)
after = before.rstrip(b"\n") + b"\n"
document.write_bytes(after)
revised = dict(receipt, revision=2,
    fact_impacts=[dict(impact, after_digest=hashlib.sha256(after).hexdigest())])
revised = seal_object(revised)
validate_object(revised)
assert revised["checks"] == receipt["checks"] and revised["result"] == receipt["result"]
receipt_path.write_bytes((json.dumps(revised, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
record = {
    "scope": "uncommitted authored IMP EOF whitespace and receipt binding",
    "reason": "Staged diff rejected the new blank line at EOF",
    "oldDocumentSha256": hashlib.sha256(before).hexdigest(),
    "persistedDocumentSha256": hashlib.sha256(after).hexdigest(),
    "originalReceiptContentDigest": receipt["content_digest"],
    "correctedReceiptContentDigest": revised["content_digest"],
    "recordedChecksOrResultChanged": False, "rawRuntimeOrFrozenEvidenceChanged": False,
    "taskChanged": False,
}
(ROOT / "correction.json").write_bytes((json.dumps(record, indent=2) + "\n").encode("utf-8"))
print(json.dumps(record))
