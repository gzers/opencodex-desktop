"""Validate this handoff only; no application imports, configuration, or network."""
from pathlib import Path
from datetime import datetime
import copy
import hashlib
import json
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text())


def sha(body):
    return "sha256:" + hashlib.sha256(body).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def wire(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


schema = read("dto.schema.json")
examples = read("examples.json")
policy = read("error-policy.json")
budgets = read("budgets.json")
Draft202012Validator.check_schema(schema)
validators = {
    name: Draft202012Validator(
        {"$schema": schema["$schema"], "$defs": schema["$defs"], "$ref": "#/$defs/" + name},
        format_checker=FormatChecker(),
    )
    for name in schema["$defs"]
}
records = {record["name"]: record["value"] for record in examples["records"]}
for record in examples["records"]:
    validators[record["type"]].validate(record["value"])

vectors = {vector["name"]: vector for vector in examples["exactByteVectors"]}
for vector in vectors.values():
    assert sha(vector["utf8"].encode()) == vector["sha256"]

for vector in examples["hashMaterials"]:
    assert canonical(vector["material"]).decode() == vector["canonicalUtf8"]
    assert sha(canonical(vector["material"])) == vector["digest"]
    field = "viewDigest" if vector["kind"] == "view" else "planDigest"
    assert records[vector["name"]][field] == vector["digest"]
    if vector["kind"] == "plan":
        assert vector["material"] == {k: v for k, v in records[vector["name"]].items() if k != field}

execution = examples["executionHashVector"]
assert canonical(execution["material"]).decode() == execution["canonicalUtf8"]
assert sha(canonical(execution["material"])) == execution["digest"]
journal = records[execution["name"]]
assert journal["executionDigest"] == execution["digest"]
assert journal["operationId"] == journal["plan"]["operationId"] == journal["fact"]["operationId"]
assert sha(journal["commitBytesUtf8"].encode()) == journal["fact"]["commit"]["digest"]
assert sha(journal["generationBytesUtf8"].encode()) == journal["fact"]["generation"]["digest"]

commits = {r["value"]["operationId"]: r["value"] for r in examples["records"] if r["type"] == "Commit"}


def validate_graph(nodes):
    parents = set()
    for identity, node in nodes.items():
        assert node["operationId"] == identity
        parent_ids = [parent["operationId"] for parent in node["parents"]]
        assert parent_ids == sorted(set(parent_ids))
        for parent in node["parents"]:
            source = nodes[parent["operationId"]]
            assert source["generationId"] == node["generationId"]
            assert parent["digest"] == sha(wire(source))
            parents.add(parent["operationId"])
        if node["intent"] == "resolve" and node["resolution"]["selectionMode"] == "head":
            assert node["resolution"]["selectedHead"] in node["parents"]
        visiting, visited = set(), set()

        def visit(current):
            assert current not in visiting
            if current in visited:
                return
            visiting.add(current)
            for parent in nodes[current]["parents"]:
                visit(parent["operationId"])
            visiting.remove(current)
            visited.add(current)

        visit(identity)
    return set(nodes) - parents


ids = list(commits)
assert len(validate_graph({key: commits[key] for key in ids[:2]})) == 2
assert validate_graph(commits) == {ids[2]}
for node in commits.values():
    prefix = "config-v2-data-" + node["generationId"] + "-" + node["operationId"]
    body = vectors[prefix + ".ocxdconf"]["utf8"].encode()
    assert sha(body) == node["payloadDigest"] and len(body) == node["payloadBytes"]

codes = schema["$defs"]["SyncFailure"]["properties"]["code"]["enum"]
actions = schema["$defs"]["RecoveryAction"]["properties"]["kind"]["enum"]
assert set(codes) == set(policy["policies"])
for name, rule in policy["policies"].items():
    assert set(rule["allowedRecoveryKinds"]).issubset(actions)
    assert len(rule["allowedRecoveryKinds"]) <= budgets["recoveryActionsMaxCount"]
assert not set(policy["forbiddenRecoveryKinds"]).intersection(actions)
assert budgets["payloadFileMaxBytes"] == schema["$defs"]["Commit"]["properties"]["payloadBytes"]["maximum"]
assert budgets["parentsMaxCount"] == schema["$defs"]["Commit"]["properties"]["parents"]["maxItems"]
for record in examples["records"]:
    value = record["value"]
    if record["type"] in ("UploadPlan", "RecoveryPlan", "DownloadPlan"):
        duration = datetime.fromisoformat(value["expiresAt"]) - datetime.fromisoformat(value["preparedAt"])
        assert duration.total_seconds() * 1000 == budgets["unattemptedPlanMaxAgeMs"]
    if record["type"] == "OperationFact" and "commit" in value and value["kind"] == "upload":
        assert value["commit"]["operationId"] == value["operationId"]

negative_checks = []


def reject(name, kind, original, **changes):
    value = copy.deepcopy(original)
    value.update(changes)
    assert not validators[kind].is_valid(value), name
    negative_checks.append(name)


generation = records["generation"]
first = records["commit-101"]
resolution = records["commit-103"]
plan = records["read-only-prepare-no-generation"]
stored = records["stored-but-local-changed-and-remote-conflict"]
reject("ETag is not a wire field", "Generation", generation, etag='"fake"')
reject("unknown wire version", "Generation", generation, formatVersion=2)
reject("invalid UUID version", "Generation", generation, generationId="00000000-0000-1000-8000-000000000001")
reject("uppercase digest", "Commit", first, payloadDigest="sha256:" + "A" * 64)
reject("oversized payload", "Commit", first, payloadBytes=budgets["payloadFileMaxBytes"] + 1)
reject("duplicate parent", "Commit", resolution, parents=[resolution["parents"][0]] * 2)
reject("resolve needs two parents", "Commit", resolution, parents=resolution["parents"][:1])
reject("update cannot masquerade as resolve", "Commit", first, resolution=resolution["resolution"])
reject("head resolution needs explicit selected head", "Resolution", {k: v for k, v in resolution["resolution"].items() if k != "selectedHead"})
reject("prepare must not preallocate generation", "UploadPlan", plan, generation=stored["generation"])
reject("prepare does not contain secret bytes", "UploadPlan", plan, password="synthetic-rejected")
reject("stored needs commit evidence", "OperationFact", {k: v for k, v in stored.items() if k != "commit"})
reject("stored needs observation time", "OperationFact", {k: v for k, v in stored.items() if k != "storedAt"})
reject("possible commit send cannot be cancelled-before-write", "OperationFact", stored, phase="cancelled_before_write")
reject("relation must retain separate axes", "OperationFact", stored, relation="synced")
reject("no generic success storage state", "OperationFact", stored, storageStatus="success")
reject("upload cannot claim local download application", "OperationFact", stored, localApplyStatus="applied")
reject("no clear-remote recovery action", "RecoveryAction", {"kind": "clear_remote"})
reject("original verify needs original operation ID", "RecoveryAction", {"kind": "verify_original_upload"})
reject("ambiguous generations cannot expose ordinary selected generation", "RemoteObservation", records["ambiguous-generations"], generation=stored["generation"])
reject("empty cannot hide existing generations", "RemoteObservation", records["empty-observation"], generations=[stored["generation"]])
reject("verify does not accept current MCP choice", "OperationRequest", records["verify-original-call"], mcpSourcesDigest=plan["sourceGuard"]["mcpSourcesDigest"])
reject("confirm cannot silently change initialization consent", "ConfirmUploadRequest", records["confirm-upload-call"], initializeOnConfirm=False)
reject("recovery download requires local target protection", "RecoveryPlan", {key: value for key, value in records["recovery-download-plan"].items() if key != "targetDigest"})
reject("private frozen commit requires execution digest", "Journal", {key: value for key, value in journal.items() if key != "executionDigest"})

bad_graph = copy.deepcopy(commits)
bad_graph[ids[2]]["resolution"]["selectedHead"] = {"operationId": ids[2], "digest": "sha256:" + "0" * 64}
try:
    validate_graph(bad_graph)
except (AssertionError, KeyError):
    negative_checks.append("selected resolution head must be a parent")
else:
    raise AssertionError("resolution accepted a non-parent")
bad_graph = copy.deepcopy(commits)
bad_graph[ids[0]]["generationId"] = records["other-generation"]["generationId"]
try:
    validate_graph(bad_graph)
except (AssertionError, KeyError):
    negative_checks.append("cross-generation ancestry must fail")
else:
    raise AssertionError("cross-generation graph accepted")

result = {
    "status": "passed",
    "scope": "contract schema and synthetic serialization only; not application or real-WebDAV validation",
    "dtoDefinitions": len(schema["$defs"]),
    "validExamples": len(examples["records"]),
    "exactByteVectors": len(vectors),
    "structuredHashVectors": len(examples["hashMaterials"]) + 1,
    "graphChecks": 2,
    "errorPolicies": len(codes),
    "negativeChecks": negative_checks,
    "networkUsed": False,
    "productionCodeChanged": False,
}
(ROOT / "validation.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10))
print(json.dumps({key: value if key != "negativeChecks" else len(value) for key, value in result.items()}, ensure_ascii=False))
