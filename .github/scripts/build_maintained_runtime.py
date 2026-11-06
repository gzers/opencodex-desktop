"""Reproduce the adopted runtime bytes from the pinned npm archive and patches.

No installed package, npm configuration, or upstream repository is modified.
The reviewed candidate archive is retained byte-for-byte; release adoption and
maintenance are described in a separate delivery manifest.
"""
import argparse
import base64
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / ".github/maintained-runtime.json"
PATCHES = ROOT / "test/smoke/post021-windows-final/patches"
BUILDER = ROOT / "test/smoke/post021-windows-final/build-installable-runtime-candidate.py"


def download(url, limit):
    with urllib.request.urlopen(url, timeout=60) as response:
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError("Pinned runtime download exceeds its limit")
    return data


def build(output, registry_path=None, archive_path=None):
    policy = json.loads(POLICY.read_bytes())
    review_path = PATCHES / "runtime-candidate2-review.json"
    review = json.loads(review_path.read_bytes())
    metadata_url = "https://registry.npmjs.org/@bitkyc08%2fopencodex/" + policy["baseVersion"]
    archive_url = "https://registry.npmjs.org/@bitkyc08/opencodex/-/opencodex-" + policy["baseVersion"] + ".tgz"
    registry_bytes = Path(registry_path).read_bytes() if registry_path else download(metadata_url, 1024 * 1024)
    registry = json.loads(registry_bytes)
    archive = Path(archive_path).read_bytes() if archive_path else download(archive_url, 32 * 1024 * 1024)
    assert registry["name"] == "@bitkyc08/opencodex"
    assert registry["version"] == policy["baseVersion"] and registry["gitHead"] == policy["baseGitHead"]
    assert hashlib.sha256(archive).hexdigest() == policy["baseSha256"], "Pinned original archive changed"
    assert registry["dist"]["integrity"] == "sha512-" + base64.b64encode(hashlib.sha512(archive).digest()).decode()
    assert registry["dist"]["shasum"] == hashlib.sha1(archive).hexdigest()
    expected = review["expectedArchive"]
    assert all(expected[k] == policy[k] for k in ("version", "archive", "sha256", "bytes"))
    runtime_paths = [row["path"] for row in review["files"]]
    with tempfile.TemporaryDirectory(prefix="ocx-maintained-runtime-") as directory:
        temporary = Path(directory)
        source = temporary / "source"
        source.mkdir()
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
            for row in review["files"]:
                if row["beforeSha256"] is None:
                    continue
                member = package.getmember("package/" + row["path"])
                assert member.isfile() and member.size < 1024 * 1024
                data = package.extractfile(member).read()
                assert hashlib.sha256(data).hexdigest() == row["beforeSha256"]
                target = source / row["path"]
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
        for patch in review["patches"]:
            file = PATCHES / patch["file"]
            assert hashlib.sha256(file.read_bytes()).hexdigest() == patch["sha256"], "Reviewed patch changed"
            command = ["git", "apply", "--no-index", *["--include=" + p for p in runtime_paths]]
            subprocess.run([*command, "--check", str(file)], cwd=source, check=True)
            subprocess.run([*command, str(file)], cwd=source, check=True)
        metadata = temporary / "registry.json"
        metadata.write_bytes(registry_bytes)
        original = temporary / "original.tgz"
        original.write_bytes(archive)
        spec = importlib.util.spec_from_file_location("candidate_builder", BUILDER)
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)
        report = builder.build(metadata, original, source, review_path, output)
    assert all(report[k] == policy[k] for k in ("version", "archive", "sha256", "bytes")), "Rebuilt runtime differs from adopted bytes"
    output = Path(output)
    delivery = {**policy, "kind": "project-maintained-offline-runtime",
                "source": metadata_url, "patches": review["patches"],
                "runtimeFiles": report["runtimeFiles"], "changedFiles": report["changedFiles"],
                "originalCandidateIdentityRetained": True}
    (output / "maintained-runtime.json").write_text(json.dumps(delivery, ensure_ascii=False, indent=2) + "\n")
    (output / "maintained-runtime.sha256").write_text(policy["sha256"] + "  " + policy["archive"] + "\n")
    return delivery


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--registry", type=Path)
    parser.add_argument("--archive", type=Path)
    args = parser.parse_args()
    print(json.dumps(build(args.output, args.registry, args.archive), indent=2))
