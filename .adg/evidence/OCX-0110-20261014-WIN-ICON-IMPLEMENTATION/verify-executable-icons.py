"""Parse PE icon resources as bytes; compare every ICO frame without loading code."""
import hashlib
import io
import json
import struct
from datetime import datetime, timezone
from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parent
audit = json.loads((root / "candidate-archive-verification.json").read_text(encoding="utf-8"))
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
folder = code / "test/software/candidate-bb3122d-38499860139/msi-readonly"
msi = json.loads((folder / "msi-readonly.json").read_text(encoding="utf-8-sig"))
assert msi["result"] == "pass" and not msi["installerExecuted"]
record = next(item for item in msi["extractedExecutables"] if item["name"] == "opencodex-desktop.exe")
raw = Path(record["path"]).read_bytes()
digest = hashlib.sha256(raw).hexdigest()
expected = next(item for item in audit["packages"] if item["kind"] == "msi")
assert digest == record["sha256"] == expected["expectedInstalledExecutableSha256"]
source = "bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a"
assert source.encode() in raw
marker = b"__TAURI_BUNDLE_TYPE_VAR_MSI"
offset = expected["bundleMarkerOffset"]
assert raw[offset:offset + len(marker)] == marker
normalized = bytearray(raw)
normalized[offset:offset + len(marker)] = b"__TAURI_BUNDLE_TYPE_VAR_UNK"
assert hashlib.sha256(normalized).hexdigest() == audit["builtExecutableSha256"]
assert raw[:2] == b"MZ"
pe_offset = struct.unpack_from("<I", raw, 0x3c)[0]
assert raw[pe_offset:pe_offset + 4] == b"PE\0\0"
coff = pe_offset + 4
section_count = struct.unpack_from("<H", raw, coff + 2)[0]
optional_size = struct.unpack_from("<H", raw, coff + 16)[0]
optional = coff + 20
magic = struct.unpack_from("<H", raw, optional)[0]
assert magic in [0x10b, 0x20b]
directories = optional + (112 if magic == 0x20b else 96)
resource_rva, resource_size = struct.unpack_from("<II", raw, directories + 16)
sections = []
for index in range(section_count):
    at = optional + optional_size + index * 40
    virtual_size, rva, stored_size, stored_offset = struct.unpack_from("<IIII", raw, at + 8)
    sections.append((rva, max(virtual_size, stored_size), stored_offset, stored_size))


def rva_offset(rva, size=1):
    for start, span, stored, stored_size in sections:
        if start <= rva and rva + size <= start + span:
            at = stored + rva - start
            assert rva - start + size <= stored_size and at + size <= len(raw)
            return at
    raise ValueError("RVA outside stored PE section")


resource_base = rva_offset(resource_rva, resource_size)


def entries(relative):
    assert 0 <= relative < resource_size
    at = resource_base + relative
    named, numbered = struct.unpack_from("<HH", raw, at + 12)
    assert named + numbered < 4096
    result = []
    for index in range(named + numbered):
        identity, target = struct.unpack_from("<II", raw, at + 16 + index * 8)
        result.append((identity, target & 0x7fffffff, bool(target & 0x80000000)))
    return result


def resources(kind):
    found = {}
    for type_id, type_dir, is_directory in entries(0):
        if type_id != kind:
            continue
        assert is_directory
        for icon_id, language_dir, is_directory in entries(type_dir):
            assert is_directory and not icon_id & 0x80000000
            for language, data_dir, is_directory in entries(language_dir):
                assert not is_directory
                rva, size, _, _ = struct.unpack_from("<IIII", raw, resource_base + data_dir)
                at = rva_offset(rva, size)
                found[(icon_id, language)] = raw[at:at + size]
    return found


icons = resources(3)
groups = resources(14)
assert groups and icons
source_ico = Image.open(code / "apps/desktop/tauri/icons/windows/icon.ico")
matches = []
for (group_id, language), group in groups.items():
    reserved, type_id, count = struct.unpack_from("<HHH", group)
    assert reserved == 0 and type_id == 1
    sizes = []
    frames = []
    for index in range(count):
        at = 6 + index * 14
        width, height, _, _ = struct.unpack_from("<BBBB", group, at)
        width, height = width or 256, height or 256
        bytes_count, icon_id = struct.unpack_from("<IH", group, at + 8)
        payload = icons.get((icon_id, language))
        if payload is None:
            payload = next(value for (item, _), value in icons.items() if item == icon_id)
        assert len(payload) == bytes_count
        assert payload[:8] == b"\x89PNG\r\n\x1a\n"
        actual = Image.open(io.BytesIO(payload)).convert("RGBA")
        expected_image = source_ico.ico.getimage((width, height)).convert("RGBA")
        assert actual.size == expected_image.size and actual.tobytes() == expected_image.tobytes()
        sizes.append(width)
        frames.append({"size": width, "iconResourceId": icon_id,
                       "payloadSha256": hashlib.sha256(payload).hexdigest(), "pixelsMatchSource": True})
    assert sorted(sizes) == [16, 24, 32, 48, 64, 128, 256]
    matches.append({"groupResourceId": group_id, "language": language, "frames": frames})
result = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source, "runId": audit["runId"],
    "extractedMsiExecutable": record["path"], "executableSha256": digest,
    "packageAndExactBundleHashMatches": True, "embeddedCommitMatches": True,
    "allIcoFramePixelsMatchApprovedSource": True, "groups": matches,
    "installerInvoked": False, "executableLoadedOrStarted": False,
    "newSourceNativeTrayAppearanceAccepted": False, "releaseAccepted": False,
}
(root / "executable-icon-identity.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"source": source, "icoFramesMatch": sum(len(group["frames"]) for group in matches),
                  "exeSha256": digest, "executableStarted": False}))
