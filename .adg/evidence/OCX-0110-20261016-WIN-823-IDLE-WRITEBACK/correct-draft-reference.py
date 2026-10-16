"""Bind an uncommitted result receipt to the actual Git document bytes.

Preserve the first draft. Do not change recorded checks or any native evidence.
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object

assert not (ROOT / "manifest.json").exists()
receipt_path = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-823-IDLE-20261016.json"
before_bytes = receipt_path.read_bytes()
before = json.loads(before_bytes)
assert before["revision"] == 1 and before["result"] == "failed"
original = ROOT / "receipt-draft-revision1.original.json"
assert not original.exists()
original.write_bytes(before_bytes)
impact = before["fact_impacts"][0]
document = PROJECT / impact["path"]
old_doc = document.read_bytes()
assert hashlib.sha256(old_doc).hexdigest() == impact["after_digest"]
normalized = old_doc.replace(bytes([13, 10]), bytes([10]))
staged = subprocess.check_output(
    ["git", "-c", "core.longpaths=true", "show", ":" + impact["path"]],
    cwd=PROJECT)
assert normalized == staged, "Only the already staged EOL normalization is allowed"
document.write_bytes(normalized)
after = dict(before)
after["revision"] = 2
after["fact_impacts"] = [dict(impact, after_digest=hashlib.sha256(staged).hexdigest())]
after = seal_object(after)
validate_object(after)
assert after["checks"] == before["checks"] and after["result"] == before["result"]
receipt_path.write_bytes((json.dumps(after, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
record = {
    "scope": "uncommitted writeback receipt and authored IMP EOL only",
    "reason": "Windows working document CRLF differed from the Git LF blob; bind the fact digest to the persisted document",
    "originalReceiptContentDigest": before["content_digest"],
    "correctedReceiptContentDigest": after["content_digest"],
    "oldDocumentSha256": hashlib.sha256(old_doc).hexdigest(),
    "persistedDocumentSha256": hashlib.sha256(staged).hexdigest(),
    "documentTextChanged": False, "recordedChecksChanged": False,
    "recordedResult": "failed", "rawRuntimeOrFrozenEvidenceChanged": False,
    "taskChanged": False,
}
(ROOT / "correction.json").write_bytes((json.dumps(record, indent=2) + "\n").encode("utf-8"))
print(json.dumps(record))
