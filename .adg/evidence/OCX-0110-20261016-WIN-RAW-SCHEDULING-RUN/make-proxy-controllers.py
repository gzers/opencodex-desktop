"""Derive transports only; keep every admission and single-dispatch assertion."""
import ast
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
admit_original = (ROOT / "admit-ci.py").read_text(encoding="utf-8")
admit_from = 'ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py"'
admit_to = 'ROOT / "github-reader-via-proxy.py"'
assert admit_original.count(admit_from) == 1
admit = admit_original.replace(admit_from, admit_to)
dispatch_original = (ROOT / "dispatch-once.py").read_text(encoding="utf-8")
assert dispatch_original.count(admit_from) == 1
dispatch = dispatch_original.replace(admit_from, admit_to)
old_args = '"-4", "--http1.1", "--tlsv1.2", "--tls-max", "1.2",'
new_args = '"--proxy", "socks5h://[::1]:3060", "--noproxy", "", "--http1.1", "--tlsv1.2",'
assert dispatch.count(old_args) == 1
dispatch = dispatch.replace(old_args, new_args)
for name, text in [("admit-ci-via-proxy.py", admit), ("dispatch-once-via-proxy.py", dispatch)]:
    ast.parse(text, filename=name)
    with (ROOT / name).open("xb") as output:
        output.write(text.encode("utf-8"))
record = {
    "admissionOriginalSha256": hashlib.sha256(admit_original.encode("utf-8")).hexdigest(),
    "admissionDerivedSha256": hashlib.sha256(admit.encode("utf-8")).hexdigest(),
    "dispatchOriginalSha256": hashlib.sha256(dispatch_original.encode("utf-8")).hexdigest(),
    "dispatchDerivedSha256": hashlib.sha256(dispatch.encode("utf-8")).hexdigest(),
    "changes": ["GET transport module", "POST command-local IPv6 SOCKS route",
                "TLS minimum remains 1.2; allow newer negotiated versions"],
    "certificateVerificationEnabled": True, "admissionCriteriaChanged": False,
    "singleDispatchGuardsChanged": False, "defaultConcurrencyOrDeadlinesChanged": False,
    "libraryOrPinnedNodeExecutedByDerivation": False, "systemSettingsChanged": False,
}
with (ROOT / "proxy-controller-derivation.json").open("xb") as output:
    output.write((json.dumps(record, indent=2) + "\n").encode("utf-8"))
print(json.dumps(record))
