"""Fetch a pinned Microsoft decoder and source documentation; never start ETW."""
import base64
import gzip
import hashlib
import json
import subprocess
import urllib.request
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path
from datetime import datetime, timezone

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/bd2-etw-preparation-20261014"
if runtime.exists():
    assert not any(runtime.iterdir()), "Do not replace prepared decoder files"
else:
    runtime.mkdir()


def read_https(url):
    request = urllib.request.Request(url, headers={"User-Agent": "ocx-bd2-prepare-only"})
    with urllib.request.urlopen(request, timeout=25) as response:
        raw = response.read()
        return gzip.decompress(raw) if response.headers.get("Content-Encoding") == "gzip" else raw


package_id, version = "microsoft.diagnostics.tracing.traceevent", "3.1.21"
registration_url = f"https://api.nuget.org/v3/registration5-semver1/{package_id}/{version}.json"
registration = json.loads(read_https(registration_url))
catalog_url = registration["catalogEntry"]
assert catalog_url.startswith("https://api.nuget.org/v3/catalog0/")
metadata_url = "https://www.nuget.org/api/v2/Packages(Id='Microsoft.Diagnostics.Tracing.TraceEvent',Version='3.1.21')"
metadata_raw = read_https(metadata_url)
metadata = ET.fromstring(metadata_raw)
properties = {element.tag.rsplit("}", 1)[-1]: element.text for element in metadata.iter()}
assert properties["Id"].lower() == package_id and properties["Version"] == version
assert properties["PackageHashAlgorithm"] == "SHA512"
package_url = f"https://api.nuget.org/v3-flatcontainer/{package_id}/{version}/{package_id}.{version}.nupkg"
package = read_https(package_url)
assert base64.b64encode(hashlib.sha512(package).digest()).decode() == properties["PackageHash"]
package_path = runtime / "traceevent-3.1.21.nupkg"
package_path.write_bytes(package)
files = []
with zipfile.ZipFile(package_path) as archive:
    assert archive.testzip() is None
    assert ".signature.p7s" in archive.namelist()
    for name in [
        "lib/netstandard2.0/Microsoft.Diagnostics.Tracing.TraceEvent.dll",
        "lib/netstandard2.0/Microsoft.Diagnostics.FastSerialization.dll",
        "lib/netstandard2.0/Microsoft.Diagnostics.Tracing.TraceEvent.xml",
    ]:
        raw = archive.read(name)
        destination = runtime / Path(name).name
        destination.write_bytes(raw)
        files.append({"file": destination.name, "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)})
sources = []
for name in ["TraceEventSession.cs", "Parsers/KernelTraceEventParser.cs"]:
    url = "https://raw.githubusercontent.com/microsoft/perfview/v3.1.21/src/TraceEvent/" + name
    raw = read_https(url)
    destination = runtime / Path(name).name
    destination.write_bytes(raw)
    sources.append({"url": url, "file": destination.name, "bytes": len(raw),
                    "sha256": hashlib.sha256(raw).hexdigest()})
(root / "traceevent-package-provenance.json").write_text(json.dumps({
    "atUtc": datetime.now(timezone.utc).isoformat(), "packageId": package_id, "version": version,
    "registrationUrl": registration_url, "catalogUrl": catalog_url, "metadataUrl": metadata_url,
    "packageUrl": package_url, "initialCatalogRequest404": True,
    "officialCatalogSha512Matches": True, "packageSha256": hashlib.sha256(package).hexdigest(),
    "packageBytes": len(package), "zipCrcPassed": True, "repositorySignaturePresent": True,
    "repositorySignatureCryptographicallyVerified": False, "decoderFiles": files,
    "sourceFiles": sources, "runtime": str(runtime),
    "installedGlobally": False, "packageScriptsExecuted": False,
    "etwSessionStarted": False, "testExeOrNodeStarted": False,
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"decoderVersion": version, "officialSha512Matches": True,
                  "filesExtracted": len(files), "etwSessionStarted": False}))
