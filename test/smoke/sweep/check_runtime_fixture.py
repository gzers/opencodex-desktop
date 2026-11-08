"""Check generated real-user fixtures against the adopted runtime's real schema.

Explicit isolated runtime/Bun inputs only; no service or business request. The
caller keeps stdout as evidence; the owned H/driver are removed on success/fail.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--bun', type=Path, required=True)
    args = parser.parse_args()
    runtime, bun = args.runtime.resolve(), args.bun.resolve()
    helper = Path(__file__).resolve().parents[1] / 'windows023-two-user-native-helper.ps1'
    source = helper.read_text(encoding="utf-8")
    function = source[source.index('function fixtureConfig(port)'):source.index('function publicRecord(record)')]
    schema = runtime / 'src/config/schema/config-schema.ts'
    assert schema.is_file() and bun.is_file()
    with tempfile.TemporaryDirectory(prefix='ocxd023-fixture-schema-') as temp:
        root = Path(temp)
        script = root / 'check.ts'
        script.write_text("import assert from 'node:assert/strict';import {configSchema} from " + json.dumps(schema.as_uri()) + ';\n' + function + """
for(const port of [0,54321]){
 const config=fixtureConfig(port), parsed=configSchema.parse(config);
 assert.equal(parsed.port,port);
 assert.equal(Object.hasOwn(config,'port'),port!==0);
 assert.equal(parsed.defaultProvider,'sweep-fixture');
 assert.equal(Object.keys(parsed.providers).length,1);
 assert.equal(parsed.providers['sweep-fixture'].liveModels,false);
 assert.equal(parsed.providers['sweep-fixture'].baseUrl,'http://127.0.0.1:1/v1');
}
console.log('actual-adopted-schema-two-fixtures-passed');
""", encoding="utf-8")
        env = dict(HOME=temp, USERPROFILE=temp, OPENCODEX_HOME=str(root/'ocx'),
                   CODEX_HOME=str(root/'codex'), PATH='', CI='1', OCX_NO_DESKTOP_HANDOFF='1')
        for key in ('SystemRoot', 'WINDIR', 'COMSPEC', 'TEMP', 'TMP'):
            if key in os.environ:
                env[key] = os.environ[key]
        result = subprocess.run([str(bun), '--no-env-file', str(script)], cwd=runtime,
                                env=env, text=True, encoding="utf-8", capture_output=True, timeout=20)
        print(result.stdout, end='')
        print(result.stderr, end='')
    print(json.dumps(dict(temporaryRootsRemoved=not root.exists(), accepted=False)))
    raise SystemExit(result.returncode)


if __name__ == '__main__':
    main()
