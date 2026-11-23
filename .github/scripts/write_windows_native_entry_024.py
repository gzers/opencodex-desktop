"""Write the installer-side provenance record the Windows terminal entry needs.

The file records the exact ocx.exe / ocxd.exe bytes shipped by one installer
build, so the manager can later prove the installed entry still matches the
build that registered it. It is a build product, never a source file: the
release job runs this after compiling the Windows binaries and before the
bundler embeds them.

The bundler runs this as its beforeBundleCommand hook, whose working directory
is not stable enough to spell relative paths in the config, so every default is
derived from this file's own location instead.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

PACKAGE = "opencodex-desktop"
KIND = "installer-native-exe"
NAMES = ("ocx.exe", "ocxd.exe")

# .github/scripts/<this file> -> repository root -> apps/desktop/tauri
TAURI_DIRECTORY = Path(__file__).resolve().parents[2] / "apps" / "desktop" / "tauri"


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def version(tauri_config):
    value = json.loads(Path(tauri_config).read_bytes()).get("version")
    if not isinstance(value, str) or not re.fullmatch(r"\d+\.\d+\.\d+", value):
        raise SystemExit("tauri config must declare a plain semantic version")
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binaries", default=str(TAURI_DIRECTORY / "target" / "release"),
                        help="release directory holding ocx.exe and ocxd.exe")
    parser.add_argument("--config", default=str(TAURI_DIRECTORY / "tauri.conf.json"),
                        help="tauri.conf.json of the build")
    parser.add_argument("--output", default=str(TAURI_DIRECTORY / "resources" / "ocx-native-entry.json"),
                        help="path to resources/ocx-native-entry.json")
    args = parser.parse_args()

    binaries = Path(args.binaries).resolve()
    output = Path(args.output).resolve()
    files = {}
    for name in NAMES:
        path = binaries / name
        if not path.is_file():
            raise SystemExit("missing compiled native binary: " + name)
        if path.read_bytes()[:2] != b"MZ":
            raise SystemExit("native binary is not a PE image: " + name)
        files[name] = digest(path)
    payload = {
        "schemaVersion": 1,
        "package": PACKAGE,
        "version": version(args.config),
        "entryKind": KIND,
        "files": files,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps({"output": str(output), "version": payload["version"],
                      "files": files}))


if __name__ == "__main__":
    main()
