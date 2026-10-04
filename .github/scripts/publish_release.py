#!/usr/bin/env python3
"""Publish existing release assets; stable promotion must be explicitly selected."""

import argparse
import json
import re
import subprocess
import sys
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


def validate_manifest(manifest, release, tag):
    if not isinstance(manifest, dict):
        raise PublishError("latest.json must be a JSON object")
    if manifest.get("version") != tag[1:]:
        raise PublishError("latest.json version does not match the target tag")
    platforms = manifest.get("platforms", {})
    required = {"darwin-aarch64", "windows-x86_64", "windows-x86_64-msi", "windows-x86_64-nsis"}
    if not isinstance(platforms, dict) or not required.issubset(platforms):
        raise PublishError("latest.json is missing required macOS/Windows platforms")
    assets = {urllib.parse.unquote(a["browser_download_url"]): a for a in release["assets"]}
    for platform, entry in platforms.items():
        if not isinstance(entry, dict):
            raise PublishError(f"Invalid platform entry: {platform}")
        url, signature = entry.get("url", ""), entry.get("signature")
        if not isinstance(url, str) or not url.startswith("https://"):
            raise PublishError(f"Invalid artifact URL: {platform}")
        asset = assets.get(urllib.parse.unquote(url))
        if not asset or asset.get("size", 0) <= 0 or asset.get("state") != "uploaded":
            raise PublishError(f"Artifact is not uploaded to the target release: {platform}")
        if not isinstance(signature, str) or not signature.strip():
            raise PublishError(f"Missing updater signature: {platform}")
    # Metadata checks do not replace cryptographic verification by the updater.


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


def publish(repo, tag, mode):
    version(tag)
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise PublishError("Expected an owner/repository name")
    if mode not in ("preserve-latest", "stable"):
        raise PublishError("Unknown publication mode")
    release = json.loads(gh("api", f"repos/{repo}/releases/tags/{tag}"))
    if release.get("tag_name") != tag:
        raise PublishError("GitHub returned a different target release")
    manifest = json.loads(gh("release", "download", tag, "--repo", repo,
                             "--pattern", "latest.json", "--output", "-"))
    validate_manifest(manifest, release, tag)
    before = latest(repo)
    if mode == "preserve-latest" and before is None:
        raise PublishError("No existing Latest to preserve; explicitly select stable for the first release")
    if mode == "stable" and before and version(tag) < version(before):
        raise PublishError(f"Refusing stable downgrade: {before} -> {tag}")

    print(f"Target: {tag}; mode: {mode}; previous Latest: {before}", flush=True)
    expected_latest = tag if mode == "stable" else before
    gh("release", "edit", tag, "--repo", repo, "--draft=false", "--prerelease=false",
       f"--latest={'true' if expected_latest == tag else 'false'}")
    if mode == "preserve-latest" and before != tag:
        gh("release", "edit", before, "--repo", repo, "--latest=true")

    after = json.loads(gh("api", f"repos/{repo}/releases/tags/{tag}"))
    if after.get("tag_name") != tag or after.get("draft") is not False or after.get("prerelease") is not False:
        raise PublishError("Target release did not become public and non-prerelease")
    if latest(repo) != expected_latest:
        raise PublishError("Latest does not match the requested publication mode")
    base = f"https://github.com/{repo}/releases"
    verify_public_manifest(f"{base}/download/{tag}/latest.json", manifest)
    if mode == "stable" or before == tag:
        verify_public_manifest(f"{base}/latest/download/latest.json", manifest)
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
    except (PublishError, ValueError, KeyError, TypeError) as error:
        print(f"Release verification failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
