
import hashlib, hmac, http.client, json, pathlib, re, sys, uuid

def require(condition):
    if not condition:
        raise ValueError('external task contract mismatch')

def main():
    args = json.load(sys.stdin)
    bundle = args['bundle']
    task, transport = bundle['task'], bundle['transport']
    task_id = task['taskId']
    require(str(uuid.UUID(task_id)) == task_id)
    bearer_header = transport['headers']['Authorization']
    require(bearer_header.startswith('Bearer '))
    bearer = bearer_header[7:]
    require(re.fullmatch('[0-9a-fA-F]{64}', bearer) is not None)
    descriptor_path = pathlib.Path(transport['discoveryFile'])
    body = pathlib.Path(args['submissionPath']).read_bytes()
    require(len(body) <= 4 * 1024 * 1024)
    submission = json.loads(body)
    require(submission['contractVersion'] == 2)
    submission_id = submission['submissionId']
    require(str(uuid.UUID(submission_id)) == submission_id)

    def request(method, suffix, data=None):
        descriptor_bytes = descriptor_path.read_bytes()
        descriptor = json.loads(descriptor_bytes)
        authority, pid = descriptor['authority'], descriptor['pid']
        require(descriptor['version'] == 1 and type(pid) is int and pid > 0)
        require(re.fullmatch(r'127\.0\.0\.1:[0-9]{1,5}', authority) is not None)
        port = int(authority.split(':')[1])
        require(0 < port < 65536)
        # Literal loopback only. HTTPConnection has no environment proxy or
        # automatic redirect/retry support. Keep the proven socket for bearer.
        conn = http.client.HTTPConnection('127.0.0.1', port, timeout=5)
        base = '/v1/tasks/' + task_id
        def response():
            reply = conn.getresponse()
            payload = reply.read(4 * 1024 * 1024 + 1)
            require(200 <= reply.status < 300 and len(payload) <= 4 * 1024 * 1024)
            return json.loads(payload)
        try:
            challenge = str(uuid.uuid4())
            conn.request('POST', base + '/proof', json.dumps({'challenge':challenge}).encode(), {'Content-Type':'application/json'})
            proof = response()
            require(proof['taskId'] == task_id and proof['challenge'] == challenge
                    and proof['authority'] == authority and proof['pid'] == pid)
            key = ('sha256:' + hashlib.sha256(bearer.encode()).hexdigest()).encode()
            message = f'ocxd-model-task-v1\n{task_id}\n{challenge}\n{authority}\n{pid}'.encode()
            require(hmac.compare_digest(proof['proof'], hmac.new(key, message, hashlib.sha256).hexdigest()))
            require(descriptor_path.read_bytes() == descriptor_bytes and conn.sock is not None)
            conn.request(method, base + suffix, data, {'Authorization':bearer_header,'Content-Type':'application/json'})
            return response()
        finally:
            conn.close()

    action = sys.argv[1]
    if action == 'submit':
        context = request('GET', '/context')
        contract = request('GET', '/contract')
        require(context == task and contract == task['contract'])
        require(contract['submission']['properties']['contractVersion']['type'] == 'integer')
        preflight = request('POST', '/preflight', body)
        require(preflight['dtoVersion'] == 2 and preflight['runId'] == task_id
                and preflight['expectedLibrary'] == task['expectedLibrary'])
        if task['scene'] == 'check_only':
            require(preflight['review'] is None and bool(preflight['report']['summary'].strip()))
        else:
            require(preflight['review']['valid'] is True)
        receipt = request('POST', '/submit', body)
        print('external-cli: proof/context/contract/preflight/submit verified', file=sys.stderr)
    elif action == 'receipt':
        # Recovery performs no POST /submit and uses the current descriptor,
        # not the stale exported baseUrl, so a moved port is recoverable.
        receipt = request('GET', '/receipts/' + submission_id)
        print('external-cli: proof/receipt recovered without submit', file=sys.stderr)
    else:
        raise ValueError('unsupported external CLI action')
    if receipt is not None:
        require(receipt['taskId'] == task_id and receipt['submissionId'] == submission_id)
        require(receipt['payloadDigest'] == 'sha256:' + hashlib.sha256(body).hexdigest())
        require(receipt['result']['phase'] in ('review', 'accepted'))
    json.dump(receipt, sys.stdout)

try:
    main()
except Exception as error:
    # No bearer, ticket or full request in diagnostics.
    print('external-cli failure: ' + type(error).__name__, file=sys.stderr)
    sys.exit(1)
