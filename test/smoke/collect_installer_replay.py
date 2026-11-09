"""Retain controlled installer receipts without traversing private sandbox trees."""
import hashlib
import json
from pathlib import Path
import shutil
import sys


def collect(root):
    root = Path(root).resolve()
    output = root/'artifact'
    output.mkdir(exist_ok=False)
    files, excluded = [], []

    def copy(path):
        if path.is_symlink():
            raise ValueError('Evidence symlink refused')
        if path.suffix not in {'.json', '.log', '.png', '.txt'}:
            return
        relative = path.relative_to(root)
        target = output/relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
        files.append(dict(path=relative.as_posix(), bytes=target.stat().st_size,
                          sha256=hashlib.sha256(target.read_bytes()).hexdigest()))

    def visit(directory):
        for path in directory.iterdir():
            # Do this lexical test before is_dir/stat/realpath. Removed ordinary
            # users' private sandbox nodes cannot be traversed by the parent.
            if path.name.startswith(('sandbox-', 'OpenCodeX Native ')):
                excluded.append(dict(path=path.relative_to(root).as_posix(),
                                     reason='private sandbox; controlled sibling receipts retained'))
                continue
            if path.is_symlink():
                raise ValueError('Evidence symlink refused')
            if path.is_dir():
                visit(path)
            elif path.is_file():
                copy(path)

    for name in ('evidence', 'fixture-contract'):
        source = root/name
        if source.exists():
            visit(source)
    for name in ('diagnostic.json', 'original-build-identities.json'):
        if (root/name).is_file():
            copy(root/name)
    (output/'evidence-manifest.json').write_text(json.dumps(dict(
        accepted=False, diagnosticOnly=True, originalsPreserved=True,
        files=files, excluded=excluded), ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    return files


if __name__ == '__main__':
    collect(sys.argv[1])
