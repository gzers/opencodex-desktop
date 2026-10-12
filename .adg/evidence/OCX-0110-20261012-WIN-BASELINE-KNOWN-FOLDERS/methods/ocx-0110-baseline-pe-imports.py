"""Read only an exact official baseline's PE import table, never execute it."""
import hashlib
import json
import struct
from datetime import datetime, timezone
from pathlib import Path

baseline = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\software\baseline019-v2\opencodex-desktop.exe")
evidence = Path(r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-BASELINE-KNOWN-FOLDERS")
data = baseline.read_bytes()
digest = hashlib.sha256(data).hexdigest()
assert digest == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert data[:2] == b"MZ"
pe = struct.unpack_from("<I", data, 0x3C)[0]
assert data[pe:pe + 4] == b"PE\0\0"
machine, section_count = struct.unpack_from("<HH", data, pe + 4)
optional_size = struct.unpack_from("<H", data, pe + 20)[0]
optional = pe + 24
assert machine == 0x8664 and struct.unpack_from("<H", data, optional)[0] == 0x20B
import_rva, import_size = struct.unpack_from("<II", data, optional + 112 + 8)
assert import_rva and import_size
headers_size = struct.unpack_from("<I", data, optional + 60)[0]
sections = []
for index in range(section_count):
    offset = optional + optional_size + index * 40
    virtual_size, virtual_address, raw_size, raw_offset = struct.unpack_from("<IIII", data, offset + 8)
    sections.append((virtual_address, max(virtual_size, raw_size), raw_offset, raw_size))

def locate(rva):
    if rva < headers_size:
        assert rva < len(data)
        return rva
    for start, size, raw, raw_size in sections:
        if start <= rva < start + size:
            displacement = rva - start
            assert displacement < raw_size and raw + displacement < len(data)
            return raw + displacement
    raise AssertionError("PE RVA is outside file-backed sections")

def c_string(offset):
    end = data.find(b"\0", offset, offset + 4096)
    assert end != -1
    return data[offset:end].decode("ascii")

descriptors = locate(import_rva)
imports = []
for index in range(min(import_size // 20 + 1, 1000)):
    original_thunk, timestamp, forwarder, name_rva, first_thunk = struct.unpack_from("<IIIII", data, descriptors + index * 20)
    if not any((original_thunk, timestamp, forwarder, name_rva, first_thunk)):
        break
    dll = c_string(locate(name_rva))
    thunk = locate(original_thunk or first_thunk)
    for entry in range(65536):
        value = struct.unpack_from("<Q", data, thunk + entry * 8)[0]
        if value == 0:
            break
        if not value & (1 << 63):
            function = c_string(locate(value) + 2)
            if function == "SHGetKnownFolderPath":
                imports.append({"dll": dll, "function": function})
    else:
        raise AssertionError("Unbounded import thunk")
else:
    raise AssertionError("Unbounded import descriptors")
assert imports, "Official baseline did not have the expected Known Folder import"
result = {
    "collectedAtUtc": datetime.now(timezone.utc).isoformat(),
    "baselineCommit": "de6862d40389d527527b9db8ef52e186a58b625f",
    "baselineExeSha256": digest, "machine": "x86_64", "selectedImports": imports,
    "baselineExecutableRun": False, "baselineModified": False,
    "scope": "read-only exact official PE import provenance; does not prove all dependency versions or runtime call frequency",
}
(evidence / "official-baseline-pe-import-witness.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
