"""Prepare only the pinned decoder. Never fetch/run the original test or Node."""
import hashlib
import importlib.util
import json
import os
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
output = (ROOT / "test/runtime/controlled-memory-decoder").resolve()
assert os.name == "nt" and not output.exists()
spec = importlib.util.spec_from_file_location("decoder_transport", Path(__file__).with_name("windows-etw-ci-admit.py"))
safe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(safe)
output.mkdir(parents=True)
package = output / "traceevent-3.1.21.original.nupkg"
safe.download("https://api.nuget.org/v3-flatcontainer/microsoft.diagnostics.tracing.traceevent/3.1.21/microsoft.diagnostics.tracing.traceevent.3.1.21.nupkg",
              package, 16 * 1024 * 1024, expected=safe.NUGET_SHA)
with zipfile.ZipFile(package) as archive:
    safe.checked_zip(archive)
    for name, digest in safe.DECODERS.items():
        entry, = [m for m in archive.infolist() if m.filename == "lib/netstandard2.0/" + name]
        data = archive.read(entry)
        assert hashlib.sha256(data).hexdigest() == digest
        with (output / name).open("xb") as handle:
            handle.write(data)
print(json.dumps({"decoder": str(output), "version": "3.1.21",
                  "originalTestFetched": False, "originalTestExecuted": False, "officialQueryNodeExecuted": False}))
