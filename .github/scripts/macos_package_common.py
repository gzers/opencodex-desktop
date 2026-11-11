"""Shared immutable macOS artifact primitives; no installation or app launch."""
import ctypes
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shlex
import stat
import subprocess
import time

def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def xattrs(path):
    if hasattr(os, 'listxattr'):
        return {name: hashlib.sha256(os.getxattr(path, name, follow_symlinks=False)).hexdigest()
                for name in sorted(os.listxattr(path, follow_symlinks=False))}
    # CPython exposes os.listxattr only on some Unix platforms. Darwin's native
    # no-follow API keeps this script stdlib-only on the release runner.
    lib = ctypes.CDLL(None, use_errno=True)
    lib.listxattr.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
    lib.listxattr.restype = ctypes.c_ssize_t
    lib.getxattr.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_void_p,
                            ctypes.c_size_t, ctypes.c_uint32, ctypes.c_int]
    lib.getxattr.restype = ctypes.c_ssize_t
    raw = os.fsencode(path)
    size = lib.listxattr(raw, None, 0, 1)  # XATTR_NOFOLLOW
    if size < 0 or size > 1024 * 1024:
        raise OSError(ctypes.get_errno(), 'list app extended attributes')
    names = ctypes.create_string_buffer(size)
    if lib.listxattr(raw, names, size, 1) != size:
        raise OSError(ctypes.get_errno(), 'app extended attributes changed')
    result = {}
    for name in sorted(n for n in names.raw.split(b'\0') if n):
        length = lib.getxattr(raw, name, None, 0, 0, 1)
        if length < 0 or length > 16 * 1024 * 1024:
            raise OSError(ctypes.get_errno(), 'size app extended attribute')
        data = ctypes.create_string_buffer(length)
        if lib.getxattr(raw, name, data, length, 0, 1) != length:
            raise OSError(ctypes.get_errno(), 'read app extended attribute')
        result[os.fsdecode(name)] = hashlib.sha256(data.raw).hexdigest()
    return result


def tree_manifest(root):
    """Record signed bytes, modes, links and extended attributes; never follow links.

    Symlinks must remain within the sealed app. Directory mtimes are intentionally
    excluded: copying/mounting changes them without changing the sealed product.
    """
    root = root.resolve(strict=True)
    rows = []

    def visit(path):
        info = path.lstat()
        relative = path.relative_to(root).as_posix()
        row = {'path': relative, 'mode': stat.S_IMODE(info.st_mode)}
        if stat.S_ISLNK(info.st_mode):
            target = os.readlink(path)
            if Path(target).is_absolute() or not path.resolve().is_relative_to(root):
                raise ValueError(f'app symlink escapes sealed bundle: {relative}')
            row.update(kind='symlink', target=target)
        elif stat.S_ISDIR(info.st_mode):
            row['kind'] = 'directory'
        elif stat.S_ISREG(info.st_mode):
            row.update(kind='file', bytes=info.st_size, sha256=sha256(path))
        else:
            raise ValueError(f'unsupported app entry: {relative}')
        # Resource forks can affect signatures. Quarantine is preserved too;
        # this script never strips it to make signature verification pass.
        row['xattrs'] = xattrs(path)
        rows.append(row)
        if row['kind'] == 'directory':
            for child in sorted(path.iterdir(), key=lambda p: p.name):
                visit(child)

    visit(root)
    return rows


def tree_digest(rows):
    return hashlib.sha256(json.dumps(rows, ensure_ascii=False, sort_keys=True,
                                     separators=(',', ':')).encode()).hexdigest()


def version_info(app, expected):
    with (app / 'Contents/Info.plist').open('rb') as stream:
        info = plistlib.load(stream)
    if info.get('CFBundleShortVersionString') != expected:
        raise ValueError(f'CFBundleShortVersionString must equal {expected}')
    if not info.get('CFBundleIdentifier') or not info.get('CFBundleExecutable'):
        raise ValueError('app bundle identity/executable missing')
    executable = info['CFBundleExecutable']
    if Path(executable).name != executable or not (app / 'Contents/MacOS' / executable).is_file():
        raise ValueError('app executable is not a bundle-local file')
    return {key: info.get(key) for key in (
        'CFBundleShortVersionString', 'CFBundleVersion', 'CFBundleIdentifier',
        'CFBundleExecutable')}


class Commands:
    def __init__(self, report):
        self.report = report

    def run(self, args, timeout=300):
        args = [str(arg) for arg in args]
        record = {'argv': args, 'command': shlex.join(args), 'timeoutSeconds': timeout}
        self.report['commands'].append(record)
        start = time.monotonic()
        try:
            result = subprocess.run(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, timeout=timeout, check=False)
            record.update(returncode=result.returncode,
                          stdout=result.stdout.decode('utf-8', errors='replace'),
                          stderr=result.stderr.decode('utf-8', errors='replace'))
        except (OSError, subprocess.TimeoutExpired) as exc:
            record['error'] = str(exc)
            raise
        finally:
            record['elapsedSeconds'] = round(time.monotonic() - start, 3)
        if result.returncode:
            raise RuntimeError(f'command failed ({result.returncode}): {shlex.join(args)}')
        return result.stdout


def destination(value):
    # Reject both lexical and resolved .app ancestors. In particular, an output
    # symlink must never turn a fresh artifact request into an app modification.
    raw = Path(value).expanduser().absolute()
    resolved = raw.resolve(strict=False)
    if any(p.suffix.lower() == '.app' for p in (*raw.parents, raw, *resolved.parents, resolved)):
        raise ValueError('artifact/evidence destination must not be inside any .app')
    if os.path.lexists(raw) or os.path.lexists(resolved):
        raise FileExistsError(f'new output required; refusing existing destination: {raw}')
    if not raw.parent.is_dir():
        raise ValueError(f'destination parent must already exist: {raw.parent}')
    return resolved

