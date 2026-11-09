#!/usr/bin/env python3
"""Publish existing release assets; stable promotion must be explicitly selected."""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


class PublishError(Exception):
    pass


def gh(*args):
    return subprocess.run(["gh", *args], check=True, text=True, capture_output=True).stdout


def version(tag):
    match = re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag)
    if not match:
        raise PublishError(f"Expected a stable vX.Y.Z tag, got {tag!r}")
    return tuple(map(int, match.groups()))


def latest(repo):
    try:
        return json.loads(gh("api", f"repos/{repo}/releases/latest"))["tag_name"]
    except subprocess.CalledProcessError as error:
        # Authentication/network errors must not be treated as an absent Latest.
        if "HTTP 404" in (error.stderr or ""):
            return None
        raise


def get_release(repo, tag):
    # gh release view resolves drafts too; the REST tag endpoint can return 404
    # for an unpublished release. Fetch its full asset metadata by numeric ID.
    view = json.loads(gh("release", "view", tag, "--repo", repo, "--json", "apiUrl,tagName"))
    api_url = view.get("apiUrl", "")
    prefix = f"https://api.github.com/repos/{repo}/releases/"
    if view.get("tagName") != tag or not isinstance(api_url, str) or not api_url.startswith(prefix) \
            or not api_url[len(prefix):].isdigit():
        raise PublishError("GitHub returned an unexpected release identity")
    return json.loads(gh("api", api_url))


def release_assets(repo, tag, release):
    """Map assets to their final URLs while accepting GitHub's draft-only URLs."""
    prefix = f"https://github.com/{repo}/releases/download/"
    assets = {}
    for asset in release["assets"]:
        name = asset["name"]
        if not isinstance(name, str) or not name or "/" in name or "\\" in name:
            raise PublishError("Invalid release asset name")
        expected = f"{prefix}{tag}/{name}"
        actual = urllib.parse.unquote(asset["browser_download_url"])
        draft_url = re.fullmatch(re.escape(prefix) + r"untagged-[A-Za-z0-9]+/" + re.escape(name), actual)
        if actual != expected and not (release.get("draft") is True and draft_url):
            raise PublishError(f"Unexpected uploaded artifact URL: {actual!r}")
        if expected in assets:
            raise PublishError(f"Duplicate release asset: {name}")
        assets[expected] = asset
    return assets


def validate_manifest(manifest, release, tag, repo):
    if not isinstance(manifest, dict):
        raise PublishError("latest.json must be a JSON object")
    if manifest.get("version") != tag[1:]:
        raise PublishError("latest.json version does not match the target tag")
    platforms = manifest.get("platforms", {})
    required = {"darwin-aarch64", "windows-x86_64", "windows-x86_64-msi", "windows-x86_64-nsis"}
    if not isinstance(platforms, dict) or not required.issubset(platforms):
        raise PublishError("latest.json is missing required macOS/Windows platforms")
    assets = release_assets(repo, tag, release)
    for platform, entry in platforms.items():
        if not isinstance(entry, dict):
            raise PublishError(f"Invalid platform entry: {platform}")
        url, signature = entry.get("url", ""), entry.get("signature")
        if not isinstance(url, str) or not url.startswith("https://"):
            raise PublishError(f"Invalid artifact URL: {platform}")
        asset = assets.get(urllib.parse.unquote(url))
        if not asset or asset.get("size", 0) <= 0 or asset.get("state") != "uploaded":
            uploaded = [{key: item.get(key) for key in ("name", "browser_download_url", "size", "state")}
                        for item in release["assets"]]
            raise PublishError(f"Artifact is not uploaded to the target release: {platform}; "
                               f"manifest URL={url!r}; release assets={uploaded!r}")
        if not isinstance(signature, str) or not signature.strip():
            raise PublishError(f"Missing updater signature: {platform}")


def verify_artifacts(repo, tag, manifest, release):
    """Verify the actual uploaded bytes with the public key embedded in the tag."""
    print(f"Reading tagged updater public key for {tag}", flush=True)
    config_response = json.loads(gh("api", f"repos/{repo}/contents/apps/desktop/tauri/tauri.conf.json?ref={tag}"))
    config = json.loads(base64.b64decode(config_response["content"]))
    if config.get("version") != tag[1:]:
        raise PublishError("Tagged app version does not match the release")
    public_key = base64.b64decode(config["plugins"]["updater"]["pubkey"], validate=True)
    assets = release_assets(repo, tag, release)
    verified = set()
    with tempfile.TemporaryDirectory(prefix="opencodex-release-") as directory:
        root = Path(directory)
        key_file = root / "updater.pub"
        key_file.write_bytes(public_key)
        for platform, entry in manifest["platforms"].items():
            identity = (entry["url"], entry["signature"])
            if identity in verified:
                continue
            asset = assets[urllib.parse.unquote(entry["url"])]
            archive = root / "artifact"
            signature = root / "artifact.minisig"
            signature.write_bytes(base64.b64decode(entry["signature"], validate=True))
            subprocess.run(["gh", "release", "download", tag, "--repo", repo,
                            "--pattern", asset["name"], "--output", str(archive), "--clobber"],
                           check=True, text=True, capture_output=True)
            if archive.stat().st_size != asset["size"]:
                raise PublishError(f"Artifact size mismatch: {platform}")
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            if asset.get("digest") and asset["digest"] != f"sha256:{digest}":
                raise PublishError(f"Artifact digest mismatch: {platform}")
            subprocess.run(["minisign", "-V", "-m", str(archive), "-p", str(key_file),
                            "-x", str(signature)], check=True, text=True, capture_output=True)
            verified.add(identity)
            print(f"Cryptographically verified {platform}: {asset['name']}; "
                  f"size={asset['size']}; sha256={digest}", flush=True)
        # Starting with the adopted 0.2.22 delivery, the independent runtime is
        # part of the release contract, even though the desktop updater does
        # not automatically install it. Trust tagged policy, not asset metadata.
        if version(tag) >= (0, 2, 22):
            response = json.loads(gh("api", f"repos/{repo}/contents/.github/maintained-runtime.json?ref={tag}"))
            policy = json.loads(base64.b64decode(response["content"]))
            verify_maintained_runtime(repo, tag, release, policy, root, key_file)
            return policy
    return None


def verify_maintained_runtime(repo, tag, release, policy, root, key_file):
    name = policy["archive"]
    if not isinstance(name, str) or not re.fullmatch(r"opencodex-[A-Za-z0-9.+-]+[.]tgz", name):
        raise PublishError("Invalid maintained runtime archive identity")
    allowed_installation="desktop-integrated-runtime" if version(tag)>=(0,2,23) else "explicit-offline-import"
    if policy.get("maintainer") != repo or policy.get("installation") != allowed_installation:
        raise PublishError("Maintained runtime policy owner or installation mode mismatch")
    assets = release_assets(repo, tag, release)
    for asset_name in (name, name + ".sig", "maintained-runtime.json", "maintained-runtime.sha256"):
        asset = assets.get(f"https://github.com/{repo}/releases/download/{tag}/{asset_name}")
        if not asset or asset.get("state") != "uploaded" or asset.get("size", 0) <= 0:
            raise PublishError("Missing maintained runtime asset: " + asset_name)
        target = root / asset_name
        subprocess.run(["gh", "release", "download", tag, "--repo", repo, "--pattern", asset_name,
                        "--output", str(target), "--clobber"], check=True, text=True, capture_output=True)
        if target.stat().st_size != asset["size"]:
            raise PublishError("Maintained runtime download size mismatch: " + asset_name)
    archive = root / name
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if digest != policy["sha256"] or archive.stat().st_size != policy["bytes"]:
        raise PublishError("Maintained runtime differs from tagged adopted bytes")
    delivery = json.loads((root / "maintained-runtime.json").read_bytes())
    if any(delivery.get(k) != v for k, v in policy.items()):
        raise PublishError("Maintained runtime delivery manifest differs from tagged policy")
    if (root / "maintained-runtime.sha256").read_text().strip() != digest + "  " + name:
        raise PublishError("Maintained runtime checksum file mismatch")
    signature = root / "maintained-runtime.minisig"
    signature.write_bytes(base64.b64decode((root / (name + ".sig")).read_text().strip(), validate=True))
    subprocess.run(["minisign", "-V", "-m", str(archive), "-p", str(key_file), "-x", str(signature)],
                   check=True, text=True, capture_output=True)
    print(f"Cryptographically verified adopted runtime {policy['version']}: sha256={digest}", flush=True)


def verify_public_manifest(url, expected):
    last_error = None
    for attempt in range(5):
        try:
            request = urllib.request.Request(url, headers={"User-Agent": "opencodex-release-verifier"})
            # No GH_TOKEN is sent: the updater must be able to read anonymously.
            with urllib.request.urlopen(request, timeout=20) as response:
                actual = json.load(response)
            if actual == expected:
                return
            last_error = "public manifest differs from the candidate manifest"
        except (urllib.error.URLError, OSError, ValueError) as error:
            last_error = str(error)
        if attempt < 4:
            time.sleep(3)
    raise PublishError(f"Public updater endpoint verification failed: {url}: {last_error}")


def verify_public_runtime(repo, tag, policy):
    url = f"https://github.com/{repo}/releases/download/{tag}/{policy['archive']}"
    last_error = None
    for attempt in range(5):
        try:
            request = urllib.request.Request(url, headers={"User-Agent": "opencodex-release-verifier"})
            with urllib.request.urlopen(request, timeout=60) as response:
                data = response.read(policy["bytes"] + 1)
            if len(data) == policy["bytes"] and hashlib.sha256(data).hexdigest() == policy["sha256"]:
                print("Verified anonymously downloadable maintained runtime", flush=True)
                return
            last_error = "public runtime differs from tagged adopted bytes"
        except (urllib.error.URLError, OSError) as error:
            last_error = str(error)
        if attempt < 4:
            time.sleep(3)
    raise PublishError(f"Public maintained runtime verification failed: {last_error}")


def publish(repo, tag, mode):
    version(tag)
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise PublishError("Expected an owner/repository name")
    if mode not in ("preserve-latest", "stable"):
        raise PublishError("Unknown publication mode")
    print(f"Reading release metadata for {tag}", flush=True)
    release = get_release(repo, tag)
    if release.get("tag_name") != tag:
        raise PublishError("GitHub returned a different target release")
    print(f"Downloading candidate manifest for {tag}", flush=True)
    manifest = json.loads(gh("release", "download", tag, "--repo", repo,
                             "--pattern", "latest.json", "--output", "-"))
    validate_manifest(manifest, release, tag, repo)
    before = latest(repo)
    if mode == "preserve-latest" and before is None:
        raise PublishError("No existing Latest to preserve; explicitly select stable for the first release")
    if mode == "stable" and before and version(tag) < version(before):
        raise PublishError(f"Refusing stable downgrade: {before} -> {tag}")
    runtime_policy = verify_artifacts(repo, tag, manifest, release)

    print(f"Target: {tag}; mode: {mode}; previous Latest: {before}", flush=True)
    expected_latest = tag if mode == "stable" else before
    gh("release", "edit", tag, "--repo", repo, "--draft=false", "--prerelease=false",
       f"--latest={'true' if expected_latest == tag else 'false'}")
    if mode == "preserve-latest" and before != tag:
        gh("release", "edit", before, "--repo", repo, "--latest=true")

    after = get_release(repo, tag)
    if after.get("tag_name") != tag or after.get("draft") is not False or after.get("prerelease") is not False:
        raise PublishError("Target release did not become public and non-prerelease")
    if latest(repo) != expected_latest:
        raise PublishError("Latest does not match the requested publication mode")
    base = f"https://github.com/{repo}/releases"
    verify_public_manifest(f"{base}/download/{tag}/latest.json", manifest)
    if mode == "stable" or before == tag:
        verify_public_manifest(f"{base}/latest/download/latest.json", manifest)
    if version(tag) >= (0, 2, 22):
        if not isinstance(runtime_policy, dict):
            raise PublishError("Adopted runtime policy verification is required")
        verify_public_runtime(repo, tag, runtime_policy)
    print(f"Verified public release {tag}; Latest={expected_latest}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag")
    parser.add_argument("--repo", required=True)
    parser.add_argument("--mode", choices=["preserve-latest", "stable"], default="preserve-latest")
    args = parser.parse_args()
    try:
        publish(args.repo, args.tag, args.mode)
    except subprocess.CalledProcessError as error:
        print(f"Release operation failed (exit {error.returncode}): {error.stderr}", file=sys.stderr)
        return error.returncode if error.returncode > 0 else 1
    except (PublishError, ValueError, KeyError, TypeError, OSError) as error:
        print(f"Release verification failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
