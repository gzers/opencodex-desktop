#!/usr/bin/env python3
"""Probe real Codex app-server behavior with a temporary home and local Responses stub.

Usage: python3 test/codex-collaboration/probe_native.py --codex /absolute/path/to/codex
No dependencies, real accounts, production config writes, or paid model calls.
The stub verifies outgoing parameters, not model quality or OpenCodex routing.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import queue
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class LocalResponses(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        if self.path != '/v1/responses':
            self.send_error(404)
            return
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.server.requests.append({
            'model': body.get('model'),
            'reasoning': body.get('reasoning'),
            'service_tier': body.get('service_tier'),
        })
        number = len(self.server.requests)
        message = {
            'id': f'msg_{number}', 'type': 'message', 'role': 'assistant',
            'status': 'completed',
            'content': [{'type': 'output_text', 'text': 'Local audit response.', 'annotations': []}],
        }
        response = {
            'id': f'resp_{number}', 'object': 'response', 'status': 'completed',
            'output': [message],
            'usage': {'input_tokens': 1, 'output_tokens': 1, 'total_tokens': 2},
        }
        events = [
            {'type': 'response.created', 'response': {**response, 'status': 'in_progress', 'output': []}},
            {'type': 'response.output_item.added', 'output_index': 0,
             'item': {**message, 'status': 'in_progress', 'content': []}},
            {'type': 'response.output_text.delta', 'item_id': message['id'],
             'output_index': 0, 'content_index': 0, 'delta': 'Local audit response.'},
            {'type': 'response.output_item.done', 'output_index': 0, 'item': message},
            {'type': 'response.completed', 'response': response},
        ]
        payload = ''.join(f'event: {e["type"]}\ndata: {json.dumps(e)}\n\n' for e in events).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


class RpcError(RuntimeError):
    def __init__(self, method, error):
        super().__init__(f'{method}: {error}')
        self.error = error


class AppServer:
    def __init__(self, binary, root, port, reuse_home=False):
        self.binary, self.root, self.port = binary, root, port
        self.home = root / 'home'
        self.codex_home = self.home / '.codex'
        self.workspace = root / 'workspace'
        self.codex_home.mkdir(parents=True, exist_ok=True)
        self.workspace.mkdir(exist_ok=True)
        if not reuse_home:
            (self.codex_home / 'config.toml').write_text(f'''
model = "gpt-5.4"
model_provider = "audit_local"
model_reasoning_effort = "low"
approval_policy = "never"
sandbox_mode = "read-only"
web_search = "disabled"
check_for_update_on_startup = false
[model_providers.audit_local]
name = "Local audit stub"
base_url = "http://127.0.0.1:{port}/v1"
wire_api = "responses"
requires_openai_auth = false
request_max_retries = 0
stream_max_retries = 0
supports_websockets = false
[analytics]
enabled = false
[feedback]
enabled = false
''')
        # Deliberately do not inherit account tokens, provider env vars, or proxy config.
        env = {
            'PATH': '/usr/bin:/bin:/usr/sbin:/sbin',
            'HOME': str(self.home), 'CODEX_HOME': str(self.codex_home),
            'TMPDIR': str(root), 'RUST_LOG': 'error',
            'HTTP_PROXY': f'http://127.0.0.1:{port}',
            'HTTPS_PROXY': f'http://127.0.0.1:{port}',
            'NO_PROXY': 'localhost,127.0.0.1',
        }
        self.stderr = (root / ('second.stderr' if reuse_home else 'app-server.stderr')).open('w+')
        self.process = subprocess.Popen(
            [binary, 'app-server', '--listen', 'stdio://'],
            cwd=self.workspace, env=env, text=True,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr,
        )
        self.messages = queue.Queue()
        self.notifications = []
        self.next_id = 0
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        for line in self.process.stdout:
            try:
                self.messages.put(json.loads(line))
            except json.JSONDecodeError:
                self.messages.put({'invalid_json': line[:300]})
        self.messages.put({'eof': True})

    def send(self, value):
        self.process.stdin.write(json.dumps(value) + '\n')
        self.process.stdin.flush()

    def receive(self, deadline):
        try:
            message = self.messages.get(timeout=max(0.01, deadline - time.monotonic()))
        except queue.Empty as error:
            raise TimeoutError('app-server response timed out') from error
        if 'eof' in message or 'invalid_json' in message:
            self.stderr.seek(0)
            raise RuntimeError(f'app-server stopped or invalid output: {message}; {self.stderr.read()[-2000:]}')
        # This probe never authorizes model tools or UI actions.
        if 'method' in message and 'id' in message:
            self.send({'id': message['id'], 'error': {'code': -32601, 'message': 'Not supported by audit probe'}})
        return message

    def call(self, method, params):
        self.next_id += 1
        request_id = self.next_id
        self.send({'id': request_id, 'method': method, 'params': params})
        deadline = time.monotonic() + 25
        while time.monotonic() < deadline:
            message = self.receive(deadline)
            if message.get('id') == request_id and 'method' not in message:
                if 'error' in message:
                    raise RpcError(method, message['error'])
                return message['result']
            self.notifications.append(message)
        raise TimeoutError(method)

    def turn(self, thread_id, **overrides):
        result = self.call('turn/start', {
            'threadId': thread_id,
            'input': [{'type': 'text', 'text': 'Reply with a short plain-text acknowledgement.'}],
            **overrides,
        })
        turn_id = result['turn']['id']
        deadline = time.monotonic() + 25
        while time.monotonic() < deadline:
            for index, message in enumerate(self.notifications):
                params = message.get('params', {})
                if (message.get('method') == 'turn/completed'
                        and params.get('threadId') == thread_id
                        and params.get('turn', {}).get('id') == turn_id):
                    self.notifications.pop(index)
                    assert params['turn']['status'] == 'completed', params['turn']
                    return
            self.notifications.append(self.receive(deadline))
        raise TimeoutError('turn/completed')

    def close(self):
        self.process.terminate()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=5)
        self.reader.join(timeout=2)
        self.process.stdin.close()
        self.process.stdout.close()
        self.stderr.close()


def probe(server, http, report):
    def record(name, detail):
        report['checks'].append({'name': name, 'status': 'passed', 'observed': detail})

    server.call('initialize', {
        'clientInfo': {'name': 'collaboration_audit', 'version': '0.1'},
        'capabilities': {'experimentalApi': True},
    })
    server.send({'method': 'initialized', 'params': {}})
    projects = []
    for name in ['Audit A', 'Audit B']:
        project = server.call('project/create', {
            'idempotencyKey': name, 'name': name,
            'roots': [{'path': str(server.workspace)}],
        })['project']
        projects.append(project)
    first = server.call('project/list', {'limit': 1})
    assert len(first['data']) == 1 and first.get('nextCursor'), first
    second = server.call('project/list', {'limit': 1, 'cursor': first['nextCursor']})
    assert {p['id'] for p in first['data'] + second['data']} == {p['id'] for p in projects}
    project_id = projects[0]['id']
    renamed = server.call('project/update', {'projectId': project_id, 'name': 'Renamed A'})['project']
    assert renamed['id'] == project_id and renamed['name'] == 'Renamed A'
    record('projects_and_pagination', 'Two projects enumerated; rename preserves native project ID.')

    initial = server.call('thread/start', {'cwd': str(server.workspace), 'projectId': project_id})
    thread_id = initial['thread']['id']
    assert initial['model'] == 'gpt-5.4' and initial['thread']['projectId'] == project_id
    server.turn(thread_id)
    assert http.requests[-1]['model'] == 'gpt-5.4', http.requests[-1]
    server.call('thread/name/set', {'threadId': thread_id, 'name': 'Audit title'})
    metadata = server.call('thread/read', {'threadId': thread_id})['thread']
    assert metadata['name'] == 'Audit title' and metadata['projectId'] == project_id, metadata
    scoped = server.call('thread/list', {'projectId': project_id, 'modelProviders': [], 'sourceKinds': []})
    unassigned = server.call('thread/list', {'projectId': None, 'modelProviders': [], 'sourceKinds': []})
    assert thread_id in {t['id'] for t in scoped['data']}, scoped
    assert thread_id not in {t['id'] for t in unassigned['data']}, unassigned
    record('thread_project_and_title', 'Persistent thread is project-filterable; name/set is readable after one local turn.')

    server.call('thread/settings/update', {'threadId': thread_id, 'model': 'gpt-5.4-mini', 'effort': 'high'})
    metadata = server.call('thread/read', {'threadId': thread_id})['thread']
    assert metadata['model'] == 'gpt-5.4-mini' and metadata['reasoningEffort'] == 'high', metadata
    server.turn(thread_id)
    assert http.requests[-1]['model'] == 'gpt-5.4-mini', http.requests[-1]
    assert (http.requests[-1]['reasoning'] or {}).get('effort') == 'high', http.requests[-1]
    record('settings_hot_update', 'Same running app-server/thread sends mini + high on the next turn.')

    server.turn(thread_id, model='gpt-5.4', effort='low')
    server.turn(thread_id)
    assert all(r['model'] == 'gpt-5.4' and (r['reasoning'] or {}).get('effort') == 'low'
               for r in http.requests[-2:]), http.requests[-2:]
    record('turn_override_is_sticky', 'turn/start model + effort persists when the following turn omits both fields.')

    server.call('config/value/write', {'keyPath': 'model', 'value': 'gpt-5.4-mini', 'mergeStrategy': 'replace'})
    readback = server.call('config/read', {'cwd': str(server.workspace), 'includeLayers': True})
    assert readback['config']['model'] == 'gpt-5.4-mini', readback
    server.turn(thread_id)
    assert http.requests[-1]['model'] == 'gpt-5.4', http.requests[-1]
    new_thread = server.call('thread/start', {'cwd': str(server.workspace)})
    assert new_thread['model'] == 'gpt-5.4-mini', new_thread['model']
    record('config_write_vs_loaded_thread', 'Updated file reads back; new thread uses new default; loaded thread retains its model.')

    server.call('thread/settings/update', {'threadId': thread_id, 'model': None, 'effort': None})
    server.turn(thread_id)
    assert http.requests[-1]['model'] == 'gpt-5.4', http.requests[-1]
    server.call('thread/settings/update', {'threadId': thread_id, 'model': readback['config']['model'], 'effort': 'low'})
    server.turn(thread_id)
    assert http.requests[-1]['model'] == 'gpt-5.4-mini', http.requests[-1]
    record('restore_inheritance_explicitly', 'Null model does not restore the new global default; explicit resolved model does.')

    second_server = AppServer(server.binary, server.root, server.port, reuse_home=True)
    try:
        second_server.call('initialize', {
            'clientInfo': {'name': 'collaboration_audit_second', 'version': '0.1'},
            'capabilities': {'experimentalApi': True},
        })
        second_server.send({'method': 'initialized', 'params': {}})
        found = second_server.call('thread/read', {'threadId': thread_id})['thread']
        assert found['id'] == thread_id
        try:
            second_server.call('thread/settings/update', {'threadId': thread_id, 'model': 'gpt-5.4'})
        except RpcError as error:
            assert error.error['code'] == -32600, error
            assert any(text in error.error['message'].lower() for text in ['not loaded', 'not found']), error
            detail = error.error['message'].replace(thread_id, '<threadId>')
        else:
            raise AssertionError('A second app-server unexpectedly accepted settings for a thread loaded in the first server')
        server.turn(thread_id)
        assert http.requests[-1]['model'] == 'gpt-5.4-mini', http.requests[-1]
        record('second_server_is_not_live_attachment', {
            'metadata_readable': True, 'settings_update_error': detail,
            'original_server_model_unchanged': True,
        })
    finally:
        second_server.close()

    server.call('thread/archive', {'threadId': thread_id})
    normal = server.call('thread/list', {'projectId': project_id, 'archived': False, 'modelProviders': [], 'sourceKinds': []})
    archived = server.call('thread/list', {'projectId': project_id, 'archived': True, 'modelProviders': [], 'sourceKinds': []})
    assert thread_id not in {t['id'] for t in normal['data']}, normal
    assert thread_id in {t['id'] for t in archived['data']}, archived
    record('archived_is_not_deleted', 'Archived thread retains its project ID and is returned by the separate archived query.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--codex', required=True, help='Absolute path to the real Codex binary, not an OpenCodex shim')
    parser.add_argument('--output', type=Path, help='Optional JSON evidence path; otherwise stdout only')
    args = parser.parse_args()
    binary = str(Path(args.codex).resolve(strict=True))
    report = {
        'tested_at_utc': datetime.now(timezone.utc).isoformat(),
        'probe_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'codex_version': subprocess.check_output([binary, '--version'], text=True).strip(),
        'experimental_api': True, 'isolated_home': True, 'provider': 'loopback Responses stub',
        'checks': [], 'limitations': [
            'Fresh app-server only; does not attach to an existing desktop session.',
            'No OpenCodex sidecar, real search, child-agent spawn, or title generation inference tested.',
        ],
    }
    with tempfile.TemporaryDirectory(prefix='codex-collaboration-probe-') as temporary:
        http = ThreadingHTTPServer(('127.0.0.1', 0), LocalResponses)
        http.requests = []
        http_thread = threading.Thread(target=http.serve_forever, daemon=True)
        http_thread.start()
        server = None
        try:
            server = AppServer(binary, Path(temporary), http.server_port)
            probe(server, http, report)
            report['status'] = 'passed'
        except Exception as error:
            report['status'] = 'failed'
            report['error'] = str(error)
        finally:
            report['local_requests'] = http.requests
            if server is not None:
                server.close()
            http.shutdown()
            http.server_close()
            http_thread.join(timeout=2)
    output = json.dumps(report, ensure_ascii=False, indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(output)
    print(output, end='')
    return 0 if report['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
