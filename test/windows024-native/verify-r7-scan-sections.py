"""Verify mechanical source sections used by the low-memory MSVC harness.

No runtime scanner function is substituted. The harness replaces only the
AppError envelope and unrelated backup-document imports, never native calls.
"""
import argparse
import hashlib
import json
from pathlib import Path


def docs(text):
    return '\n'.join('//' + line[3:] if line.startswith('//!') else line
                     for line in text.splitlines())


def verify(repo, generated):
    relative = 'apps/desktop/tauri/src/'
    image_path = repo / (relative + 'modules/runtime/program_image.rs')
    image = image_path.read_text(encoding='utf-8')
    expected = docs(image.split('pub fn verify(selected:', 1)[0])
    assert (generated / 'program_image_section.rs').read_text(encoding='utf-8') == expected
    safety_path = repo / (relative + 'modules/backup/safety.rs')
    safety = safety_path.read_text(encoding='utf-8').split('pub fn read_file(', 1)[0]
    imports = 'use super::{BackupManifest, BackupRecord, MANIFEST_NAME, SCHEMA_VERSION};'
    assert safety.count(imports) == 1
    expected = docs(safety.replace(imports, '// Backup document types are outside this read-only section.'))
    assert (generated / 'safety_section.rs').read_text(encoding='utf-8') == expected
    layout_path = repo / (relative + 'modules/runtime/program_layout.rs')
    layout = layout_path.read_text(encoding='utf-8')
    start = layout.index('pub(crate) fn clean_absolute(')
    end = layout.index('\n}\n', start) + 3
    expected = 'use std::path::{Path, Component};\n' + layout[start:end]
    assert (generated / 'layout_section.rs').read_text(encoding='utf-8') == expected
    tests_path = repo / (relative + 'modules/runtime/program_image_windows_tests.rs')
    assert (generated / tests_path.name).read_bytes() == tests_path.read_bytes()
    windows_path = repo / (relative + 'modules/runtime/program_image_windows.rs')
    assert (generated / windows_path.name).read_bytes() == windows_path.read_bytes()
    return {
        'scannerFunctionsUnchanged': True,
        'pathAndNativeFunctionsUnchanged': True,
        'adaptations': ['CRLF decoding', 'outer documentation comments',
                        'unused backup document type import', 'test-only error envelope'],
        'doesNotCompile': ['product selection/verify wrapper', 'Tauri GUI', 'native Owner/lifecycle'],
        'files': [{'path': name, 'bytes': (generated / name).stat().st_size,
                   'sha256': hashlib.sha256((generated / name).read_bytes()).hexdigest()}
                  for name in ('program_image_section.rs', 'safety_section.rs',
                               'layout_section.rs', 'program_image_windows_tests.rs',
                               'program_image_windows.rs')],
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--generated', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = verify(args.repo, args.generated)
    with args.output.open('x', encoding='utf-8') as target:
        json.dump(result, target, ensure_ascii=False, indent=2)
        target.write('\n')
    print(json.dumps({'sameRuntimeFunctions': True, 'files': len(result['files'])}))
