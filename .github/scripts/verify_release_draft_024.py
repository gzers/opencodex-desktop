#!/usr/bin/env python3
"""Bind v0.2.24 workflow reports to a draft and verify its uploaded bytes read-only."""

import argparse
import base64
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import urllib.parse
from zipfile import BadZipFile, ZipFile

from publish_release import PublishError, release_assets, validate_manifest
from verify_runtime_delivery_024 import load_json, require, sha256

REPO = 'gzers/opencodex-desktop'
TAG = 'v0.2.24'
WORKFLOW = '.github/workflows/release.yml'
WORKFLOW_ID = 373713240
CONFIG = 'apps/desktop/tauri/tauri.conf.json'
POLICY = '.github/runtime-platforms-024.json'
RECEIPT = 'release-receipt.json'
REPORT_LIMIT = 16 * 1024 * 1024
ASSET_LIMIT = 2 * 1024 * 1024 * 1024
GROUPS = {
    'release-macos-build-binding': (
        ('macos-build-binding.json', 'macos-updater-delivery.json'), ('macos-updater',)),
    'release-macos-installation-layout-evidence': (('package-evidence.json',), ('macos-dmg',)),
    'release-windows-installer-content': (('msi.json', 'nsis.json', 'summary.json'), ('msi', 'nsis')),
}
SUFFIXES = {'macos-updater': '.app.tar.gz', 'macos-dmg': '.dmg', 'msi': '.msi', 'nsis': '-setup.exe'}
PLATFORMS = {
    'darwin-aarch64': ('macos-updater',),
    'darwin-aarch64-app': ('macos-updater',),
    'windows-x86_64-msi': ('msi',),
    'windows-x86_64-nsis': ('nsis',),
    'windows-x86_64': ('msi', 'nsis'),
}


class GitHub:
    def __init__(self, *, actions_token=None):
        self.actions_token = actions_token

    def environment(self, path):
        if path.startswith('actions/') and self.actions_token:
            return {**os.environ, 'GH_TOKEN': self.actions_token}
        return None

    @staticmethod
    def command(path):
        return ['gh', 'api', '--hostname', 'github.com', f'repos/{REPO}/{path}']

    def json(self, path, *options):
        return load_json(subprocess.check_output(self.command(path) + list(options), timeout=60,
                                                 env=self.environment(path)))

    def download(self, path, destination):
        headers = ['-H', 'Accept: application/octet-stream'] if path.startswith('releases/assets/') else []
        with destination.open('xb') as file:
            subprocess.run(self.command(path) + headers, stdout=file, check=True, timeout=600,
                           env=self.environment(path))

    def create_draft(self, body):
        return load_json(subprocess.check_output(self.command('releases') + ['--input', '-'],
                                                 input=json.dumps(body).encode(), timeout=60))


def identity(source, draft, run, attempt, group):
    require(re.fullmatch(r'[0-9a-f]{40}', source), 'full source SHA required')
    require(all(type(value) is int and value > 0 for value in (draft, run, attempt)),
            'positive numeric draft, run and attempt required')
    return dict(schemaVersion=1, repository=REPO, workflow=WORKFLOW, tag=TAG,
                source=source, draftId=draft, runId=run, attempt=attempt, group=group,
                releaseAcceptance=False)


def artifact_name(group, run, attempt):
    return f'{group}-{run}-{attempt}'


def get_release(api, draft, *, allow_public=False):
    release = api.json(f'releases/{draft}')
    require(release['id'] == draft and release['tag_name'] == TAG,
            'release ID or tag differs from the assembly receipt')
    prerelease_draft = release['draft'] is True and release['prerelease'] is True
    public = release['draft'] is False and release['prerelease'] is False
    require(prerelease_draft or (allow_public and public),
            'expected the specified v0.2.24 prerelease draft or an explicitly allowed stable release')
    return release


def get_draft(api, draft):
    return get_release(api, draft)


def find_draft(api):
    pages = api.json('releases?per_page=100', '--paginate', '--slurp')
    matches = [row for page in pages for row in page if row['tag_name'] == TAG]
    require(len(matches) <= 1, 'multiple releases have the target tag')
    if matches:
        return get_draft(api, matches[0]['id'])
    return None


def prepare_draft(api, tag, notes):
    source = workflow_source(api, tag)
    release = find_draft(api)
    if release is None:
        release = api.create_draft(dict(tag_name=TAG, target_commitish=source,
                                        name=f'OpenCodeX Desktop {TAG}', body=notes,
                                        draft=True, prerelease=True))
    return get_draft(api, release['id'])['id']


def uploaded_asset(asset):
    require(type(asset.get('id')) is int and asset['id'] > 0 and asset['state'] == 'uploaded'
            and 0 < asset['size'] <= ASSET_LIMIT
            and re.fullmatch(r'sha256:[0-9a-f]{64}', asset.get('digest') or ''),
            f"missing uploaded asset identity: {asset['name']}")
    return {key: asset[key] for key in ('id', 'name', 'size', 'digest', 'state', 'updated_at',
                                        'browser_download_url')}


def file_identity(path):
    return dict(bytes=path.stat().st_size, sha256=sha256(path))


def json_report(raw):
    return load_json(raw.decode('utf-8-sig'))


def report_asset(role, reports):
    if role == 'macos-dmg':
        report = reports['package-evidence.json']
        return dict(bytes=report['artifactBytes'], sha256=report['artifactSha256'])
    filename = 'macos-updater-delivery.json' if role == 'macos-updater' else role + '.json'
    report = reports[filename]
    return dict(bytes=report['archiveBytes'], sha256=report['archiveSha256'])


def match_asset(assets, suffix, fingerprint):
    matches = [asset for asset in assets if asset['name'].endswith(suffix)
               and asset['size'] == fingerprint['bytes']
               and asset.get('digest') == 'sha256:' + fingerprint['sha256']]
    require(len(matches) == 1, f'expected one uploaded {suffix} matching local bytes')
    return uploaded_asset(matches[0])


def workflow_source(api, tag):
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    tag_ref = f'refs/tags/{TAG}'
    workflow_ref = f'{REPO}/{WORKFLOW}@{tag_ref}'
    require(tag == TAG and os.environ['GITHUB_REPOSITORY'] == REPO
            and os.environ['GITHUB_REF'] == tag_ref
            and os.environ['GITHUB_WORKFLOW_REF'] == workflow_ref
            and os.environ['GITHUB_WORKFLOW_SHA'] == source and os.environ['GITHUB_SHA'] == source,
            'release workflow and checkout must run from the exact tagged source')
    require(tagged_source(api)[1] == source, 'remote tag and workflow SHA differ')
    return source


def write_receipt(api, group, directory, draft, tag, signature=None):
    source = workflow_source(api, tag)
    receipt = identity(source, draft, int(os.environ['GITHUB_RUN_ID']),
                       int(os.environ['GITHUB_RUN_ATTEMPT']), group)
    filenames, roles = GROUPS[group]
    receipt['reports'] = {name: file_identity(directory / name) for name in filenames}
    reports = {name: json_report((directory / name).read_bytes()) for name in filenames}
    assets = list(release_assets(REPO, TAG, get_draft(api, draft)).values())
    receipt['assets'] = {}
    asset_names = {asset['name']: asset for asset in assets}
    for role in roles:
        payload = match_asset(assets, SUFFIXES[role], report_asset(role, reports))
        bound = {'payload': payload}
        if role != 'macos-dmg':
            local = signature if role == 'macos-updater' else one_signature(role)
            remote = asset_names.get(payload['name'] + '.sig')
            require(local is not None and remote is not None, f'missing detached signature: {role}')
            bound['signature'] = match_asset([remote], '.sig', file_identity(local))
        receipt['assets'][role] = bound
    with (directory / RECEIPT).open('x', encoding='utf-8') as file:
        json.dump(receipt, file, indent=2)
        file.write('\n')


def one_signature(role):
    directory = Path('apps/desktop/tauri/target/release/bundle') / role
    files = list(directory.glob('*' + SUFFIXES[role] + '.sig'))
    require(len(files) == 1, f'expected one local {role} signature')
    return files[0]


def tagged_source(api):
    ref = api.json(f'git/ref/tags/{TAG}')
    require(ref['ref'] == f'refs/tags/{TAG}', 'unexpected tag ref')
    target = ref['object']
    first = target
    for _ in range(8):
        require(re.fullmatch(r'[0-9a-f]{40}', target['sha']), 'invalid tag object SHA')
        if target['type'] == 'commit':
            return first, target['sha']
        require(target['type'] == 'tag', 'tag does not point to a commit')
        tag = api.json(f"git/tags/{target['sha']}")
        require(tag['sha'] == target['sha'], 'tag object identity mismatch')
        target = tag['object']
    raise ValueError('too many nested annotated tags')


def source_file(api, source, path):
    row = api.json(f'contents/{path}?ref={source}')
    require(row['type'] == 'file' and row['path'] == path and row['encoding'] == 'base64',
            f'unexpected source file: {path}')
    raw = base64.b64decode(''.join(row['content'].splitlines()), validate=True)
    header = f'blob {len(raw)}'.encode() + bytes([0])
    require(row['size'] == len(raw) and row['sha'] == hashlib.sha1(header + raw).hexdigest(),
            f'source blob identity mismatch: {path}')
    return load_json(raw)


def completed_attempt(api, source, run, attempt):
    latest = api.json(f'actions/runs/{run}')
    row = api.json(f'actions/runs/{run}/attempts/{attempt}')
    for metadata in (latest, row):
        require(metadata['id'] == run and metadata['run_attempt'] == attempt
                and metadata['workflow_id'] == WORKFLOW_ID and metadata['path'] == WORKFLOW
                and metadata['head_branch'] == TAG and metadata['head_sha'] == source
                and metadata['repository']['full_name'] == REPO
                and metadata['head_repository']['full_name'] == REPO
                and metadata['event'] in ('push', 'workflow_dispatch')
                and metadata['status'] == 'completed' and metadata['conclusion'] == 'success',
                'expected a completed successful release workflow attempt at the exact source')
    return row['run_started_at'], row['updated_at']


def report_archives(api, run, attempt, source, interval):
    pages = api.json(f'actions/runs/{run}/artifacts?per_page=100', '--paginate', '--slurp')
    artifacts = [item for page in pages for item in page['artifacts']]
    start, end = (datetime.fromisoformat(value.replace('Z', '+00:00')) for value in interval)
    selected = {}
    for group in GROUPS:
        name = artifact_name(group, run, attempt)
        matches = [item for item in artifacts if item['name'] == name]
        require(len(matches) == 1, f'expected one workflow artifact: {name}')
        item = matches[0]
        created = datetime.fromisoformat(item['created_at'].replace('Z', '+00:00'))
        require(type(item['id']) is int and item['id'] > 0 and item['expired'] is False
                and start <= created <= end and item['workflow_run']['id'] == run
                and item['workflow_run']['head_sha'] == source
                and 0 < item['size_in_bytes'] <= REPORT_LIMIT
                and re.fullmatch(r'sha256:[0-9a-f]{64}', item.get('digest') or ''),
                f'artifact is stale or incomplete: {name}')
        selected[group] = item
    return selected


def download_verified(api, endpoint, path, size, digest):
    api.download(endpoint, path)
    require(path.stat().st_size == size and 'sha256:' + sha256(path) == digest,
            f'downloaded bytes differ from GitHub metadata: {endpoint}')


def read_reports(api, artifact, directory, expected):
    identifier = artifact['id']
    archive = directory / f'{identifier}.zip'
    download_verified(api, f'actions/artifacts/{identifier}/zip', archive,
                      artifact['size_in_bytes'], artifact['digest'])
    group = expected['group']
    filenames, roles = GROUPS[group]
    with ZipFile(archive) as bundle:
        required = {*filenames, RECEIPT}
        diagnostics = {'msi.stderr.log', 'nsis.stderr.log'} if group == 'release-windows-installer-content' else set()
        members = bundle.infolist()
        entries = {entry.filename: entry for entry in members}
        require(len(entries) == len(members) and required <= entries.keys() <= required | diagnostics,
                f'unexpected or duplicate ZIP members: {group}')
        require(sum(entry.file_size for entry in members) <= REPORT_LIMIT,
                f'uncompressed ZIP is too large: {group}')
        reports = {}
        for name in (*filenames, RECEIPT):
            entry = entries[name]
            require(entry.file_size > 0, f'empty ZIP member: {name}')
            raw = bundle.read(entry)
            reports[name] = (json_report(raw), dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest()))
    receipt = reports.pop(RECEIPT)[0]
    require({key: receipt.get(key) for key in expected} == expected,
            f'wrong source/draft/run receipt: {group}')
    require(receipt['reports'] == {key: entry[1] for key, entry in reports.items()}
            and set(receipt['assets']) == set(roles), f'receipt evidence changed: {group}')
    return {key: entry[0] for key, entry in reports.items()}, receipt['assets']


def check_reports(reports, source, runtime):
    build = reports['macos-build-binding.json']
    updater = reports['macos-updater-delivery.json']
    dmg = reports['package-evidence.json']
    require(build['status'] == 'verified-build-binding' and build['source'] == source
            and sorted(row['name'] for row in build['executables']) == ['ocx', 'ocxd', 'opencodex-desktop']
            and all(re.fullmatch(r'[0-9a-f]{64}', row[field]) for row in build['executables']
                    for field in ('compiledSha256', 'sealedSha256', 'canonicalSignedSha256')),
            'macOS build binding is incomplete')
    require(updater['status'] == 'verified-signed-macos-updater' and updater['appVersion'] == TAG[1:]
            and all(updater[key] is True for key in ('updaterSignatureVerified', 'sealedAppSignatureVerified',
                                                     'allMembersMatchSealedApp', 'sourceUnchanged'))
            and dmg['status'] == 'verified-candidate' and dmg['inputKind'] == 'release-input'
            and dmg['expectedVersion'] == TAG[1:] and dmg['layout'] == 'single-app'
            and dmg['sourceUnchanged'] is True
            and all(dmg['cleanup'][key] is True for key in ('ownMountDetached', 'scratchRemoved', 'outputPublished'))
            and dmg['sourceAppTreeSha256'] == dmg['mountedAppTreeSha256'] == updater['sealedAppTreeSha256']
            and re.fullmatch(r'[0-9a-f]{64}', updater['sealedAppTreeSha256'])
            and dmg['resources'] == updater['officialRuntime'], 'macOS sealed app/DMG/updater binding differs')
    summary = reports['summary.json']
    require(sorted(row['kind'] for row in summary) == ['msi', 'nsis']
            and all(row['passed'] is True and row['releaseAcceptance'] is False for row in summary),
            'both Windows installer content checks must pass')
    for role in ('msi', 'nsis'):
        report = reports[role + '.json']
        require(report['status'] == 'verified-signed-windows-embedded-payload' and report['kind'] == role
                and report['updaterSignatureVerified'] is True and report['sourceUnchanged'] is True,
                f'incomplete Windows content report: {role}')
    for name, platform in (('macos-updater-delivery.json', 'darwin-arm64'),
                           ('msi.json', 'win32-x64'), ('nsis.json', 'win32-x64')):
        delivery = reports[name]['officialRuntime']
        require(delivery['status'] == 'verified-offline' and delivery['platform'] == platform
                and delivery['officialRuntimeVersion'] == runtime['version'] == '2.99.0'
                and delivery['officialRuntimeSha256'] == runtime['sha256']
                and delivery['releaseAcceptance'] is False, f'official runtime binding differs: {name}')
    require(all(report['releaseAcceptance'] is False for name, report in reports.items()
                if name != 'summary.json'), 'expected diagnostic reports')


def download_asset(api, asset, directory, limit=ASSET_LIMIT):
    require(asset['size'] <= limit, f"asset exceeds verification limit: {asset['name']}")
    path = directory / str(asset['id'])
    download_verified(api, f"releases/assets/{asset['id']}", path, asset['size'], asset['digest'])
    return path


def receipt_asset(receipt, release):
    if release['draft'] is True:
        return receipt
    # Only the validated draft URL can move to the canonical tagged URL.
    urls = release_assets(REPO, TAG, {'draft': True, 'assets': [receipt]})
    return {**receipt, 'browser_download_url': next(iter(urls))}


def asset_snapshot(release, *, canonical_urls=False):
    assets = {url: uploaded_asset(asset) for url, asset in release_assets(REPO, TAG, release).items()}
    if canonical_urls:
        for url, asset in assets.items():
            asset['browser_download_url'] = url
    return assets


def verify_asset_bindings(api, release, reports, bindings, config, directory):
    assets = asset_snapshot(release, canonical_urls=release['draft'] is False)
    by_name = {asset['name']: asset for asset in assets.values()}
    require(len({asset['id'] for asset in assets.values()}) == len(assets), 'duplicate asset ID')
    key = directory / 'tagged.pub'
    key.write_bytes(base64.b64decode(config['plugins']['updater']['pubkey'], validate=True))
    manifest_path = download_asset(api, by_name['latest.json'], directory, REPORT_LIMIT)
    manifest = json_report(manifest_path.read_bytes())
    validate_manifest(manifest, release, TAG, REPO)
    require(set(manifest['platforms']) == set(PLATFORMS), 'updater platforms differ from the release matrix')
    signatures = {}
    for role, bound in bindings.items():
        payload = receipt_asset(bound['payload'], release)
        require(payload == match_asset(by_name.values(), SUFFIXES[role], report_asset(role, reports)),
                f'uploaded payload differs from receipt: {role}')
        binary = download_asset(api, payload, directory)
        if role == 'macos-dmg':
            require(set(bound) == {'payload'}, 'unexpected DMG signature binding')
            continue
        detached = by_name[payload['name'] + '.sig']
        require(set(bound) == {'payload', 'signature'} and detached == receipt_asset(bound['signature'], release),
                f'uploaded signature differs from receipt: {role}')
        signature_text = download_asset(api, detached, directory, REPORT_LIMIT).read_bytes()
        signatures[role] = signature_text
        signature = directory / f'{role}.minisig'
        signature.write_bytes(base64.b64decode(signature_text, validate=True))
        subprocess.run(['minisign', '-V', '-m', str(binary), '-p', str(key), '-x', str(signature)],
                       check=True, capture_output=True, timeout=120)
    for platform, row in manifest['platforms'].items():
        asset = assets[urllib.parse.unquote(row['url'])]
        require(any(asset == receipt_asset(bindings[role]['payload'], release) and row['signature'].encode() == signatures[role]
                    for role in PLATFORMS[platform]), f'wrong manifest asset or signature: {platform}')
    return {'manifest': by_name['latest.json'], 'platforms': sorted(manifest['platforms']),
            'boundAssets': bindings}, manifest


def verify_release(api, source, draft, run, attempt, *, allow_public=False):
    result = identity(source, draft, run, attempt, 'draft')
    tag = tagged_source(api)
    require(tag[1] == source, 'release tag does not resolve to specified source')
    release = get_release(api, draft, allow_public=allow_public)
    interval = completed_attempt(api, source, run, attempt)
    archives = report_archives(api, run, attempt, source, interval)
    config = source_file(api, source, CONFIG)
    policy = source_file(api, source, POLICY)
    require(config['version'] == policy['bundleVersion'] == TAG[1:]
            and policy['runtime']['distribution'] == 'official-npm', 'wrong release source policy')
    with tempfile.TemporaryDirectory(prefix='ocxd024-release-draft-') as temporary:
        root = Path(temporary)
        reports = {}
        bindings = {}
        for group, archive in archives.items():
            group_reports, group_bindings = read_reports(api, archive, root,
                                                         identity(source, draft, run, attempt, group))
            reports.update(group_reports)
            bindings.update(group_bindings)
        check_reports(reports, source, policy['runtime'])
        asset_result, manifest = verify_asset_bindings(api, release, reports, bindings, config, root)
        result.update(asset_result)
    require(tagged_source(api) == tag
            and get_release(api, draft, allow_public=allow_public) == release
            and completed_attempt(api, source, run, attempt) == interval
            and report_archives(api, run, attempt, source, interval) == archives,
            'release evidence changed during verification')
    result['reportArtifacts'] = {group: {key: archive[key] for key in ('id', 'name', 'digest', 'size_in_bytes')}
                                 for group, archive in archives.items()}
    state = 'draft' if release['draft'] else 'public'
    return dict(result, status=f'verified-{state}-assets'), manifest, release


def verify_draft(api, source, draft, run, attempt):
    result, _, _ = verify_release(api, source, draft, run, attempt)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    check = commands.add_parser('check-source')
    check.add_argument('--tag', required=True)
    prepare = commands.add_parser('prepare-draft')
    prepare.add_argument('--tag', required=True)
    receipt = commands.add_parser('receipt')
    receipt.add_argument('--group', required=True, choices=GROUPS)
    receipt.add_argument('--directory', required=True, type=Path)
    receipt.add_argument('--draft', required=True, type=int)
    receipt.add_argument('--tag', required=True)
    receipt.add_argument('--signature', type=Path)
    verify = commands.add_parser('verify')
    verify.add_argument('--source', required=True)
    for name in ('draft', 'run', 'attempt'):
        verify.add_argument('--' + name, required=True, type=int)
    args = parser.parse_args()
    api = GitHub()
    try:
        if args.command == 'check-source':
            workflow_source(api, args.tag)
        elif args.command == 'prepare-draft':
            print(prepare_draft(api, args.tag, os.environ['RELEASE_NOTES']))
        elif args.command == 'receipt':
            write_receipt(api, args.group, args.directory, args.draft, args.tag, args.signature)
        else:
            result = verify_draft(api, args.source, args.draft, args.run, args.attempt)
            print(json.dumps(result, indent=2))
    except (subprocess.SubprocessError, OSError, ValueError, PublishError, BadZipFile,
            KeyError, AttributeError, TypeError, IndexError) as error:
        print(f'Draft verification failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
