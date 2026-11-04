#!/usr/bin/env python3
"""0.2.21 only: prepare/plan by default; actual RuntimeStore writes need --commit.

No network, Git mutation, publication, source Markdown edit, or YAML dependency.
The caller supplies truthful final receipts; this tool verifies their bytes,
claims, source identity, complete coverage and the real kernel closure contract.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
DOCS = HERE.parents[3]
CODE = DOCS.parent / "ocx-0219-fixes"
LIB = Path("/Users/ezio/.codex/skills/project-governance/scripts/lib")
TAG, VERSION = "v0.2.21", "0.2.21"
DMD, IMP, REL = "DMD-OPENCODEX-UI-021", "IMP-OPENCODEX-DESKTOP-29", "REL-OPENCODEX-DESKTOP-24"
BASE = "28d7f62e994beed57d291c904ab2c775491bc928"
IDS = ["TASK-OPENCODEX-UI021-" + x + "-20261104" for x in ("MODALS", "SYNC", "INTEGRATION")]
MAC = "OpenCodeX.Desktop_aarch64.app.tar.gz"
MSI = "OpenCodeX.Desktop_0.2.21_x64_en-US.msi"
NSIS = "OpenCodeX.Desktop_0.2.21_x64-setup.exe"
ASSETS = {"latest.json", "OpenCodeX.Desktop_0.2.21_aarch64.dmg", MAC, MAC + ".sig", MSI, MSI + ".sig", NSIS, NSIS + ".sig"}
PLATFORMS = {"darwin-aarch64", "darwin-aarch64-app", "windows-x86_64", "windows-x86_64-msi", "windows-x86_64-nsis"}
# A provider proof may predate test-only commits, only if these inputs are identical.
PRODUCT = ("apps/desktop/tauri/src", "apps/desktop/tauri/Cargo.toml", "apps/desktop/tauri/Cargo.lock", "apps/desktop/tauri/build.rs", "apps/desktop/tauri/tauri.conf.json", "apps/desktop/tauri/config", "apps/desktop/ui/src", "apps/desktop/ui/index.html", "apps/desktop/ui/vite.config.ts", "apps/desktop/ui/tsconfig.json", "apps/desktop/ui/package.json", "apps/desktop/ui/package-lock.json")
GATE_FACTS = {
    "public-stable": {"tag": TAG, "latest": TAG, "draft": False, "prerelease": False, "anonymousTagReadback": True, "anonymousLatestReadback": True, "sameAssetIdentity": True},
    "artifacts-signatures": {"uploadedAssets": 8, "signedArtifacts": 3, "manifestPlatforms": 5, "macDeepStrictCodesign": True, "macEmbeddedSource": True, "licenseAndNotices": True, "dmgVerified": True, "sameFrozenBytes": True},
    "native-macos": {"platform": "darwin-aarch64", "originalReleaseBundle": True, "allRequiredChecksPassed": True, "savedFileReadback": True, "cancelAndFailureSideEffects": True},
    "native-windows": {"platform": "windows-x86_64", "originalFinalExe": True, "allRequiredChecksPassed": True, "savedFileReadback": True, "cancelAndFailureSideEffects": True, "actualOfficialDefaultBindingCaptured": True, "defaultBindingNotOverridden": True, "nativeOwnerAdmissionPassed": True},
    "final-ci": {"conclusion": "success", "requiredJobsPassed": True, "sameFinalSource": True},
    "provider-chain": {"successfulRawProductionRun": True, "exactFreezeAcrossFiveSnapshots": True, "noProductBuildOrRunDrift": True, "standardNpmSymlinkPreserved": True, "noForcedNativeRuntimeEnvironment": True, "actualInitialPathDiscoveryUnpinned": True, "actualSettingsAgreeWithResolver": True, "nativeIntegrationBoundToOwnedHome": True, "realSaveAndStableIdentity": True, "realPublicProductionPrepare": True, "liveRosterAndDefaultFullyAdopted": True, "savedDiskIdentityUnchanged": True, "producerGeneratedNewCatalogNamespace": True, "freshOfficialClientReadNewDirectory": True, "currentConfigAndFreshDefaultAgree": True, "threadStartWithoutModelConfigOverride": True, "execUsesNpmWithoutModelConfigOverride": True, "localResponsesRawModelAndSyntheticAuth": True, "actualProviderRawModelAndRouteKind": True, "clientConsumedLocalResponse": True, "realPendingRetainedAndMapped": True, "noPostSaveManualConfigCatalogOwnerRepair": True, "noDailyGlobalRestartOrRealSupplierRequest": True},
}
GATE_ASSETS = {"native-macos": [MAC], "native-windows": [MSI, NSIS]}
for kind, asset in (("msi", MSI), ("nsis", NSIS)):
    for location in ("default", "custom"):
        name = "windows-{}-{}".format(kind, location)
        GATE_FACTS[name] = {"installer": kind, "pathType": location, "installed": True, "startupReadback": True, "uninstalled": True, "dataPreserved": True, "sameFrozenBytes": True}
        # REL24 adds four installation-path cells. The existing custom cells
        # own explicit removal/data-retention assertions for both installers;
        # default cells own default-path installation/startup/resource checks.
        if location == "default":
            GATE_FACTS[name].pop("uninstalled")
            GATE_FACTS[name].pop("dataPreserved")
        GATE_ASSETS[name] = [asset]
for kind, asset in (("macos", MAC), ("msi", MSI), ("nsis", NSIS)):
    name = "ota-{}-020-to-021".format(kind)
    GATE_FACTS[name] = {"fromVersion": "0.2.20", "toVersion": VERSION, "actualDownloaded": True, "signatureVerified": True, "explicitInstall": True, "automaticNewProcessObserved": True, "installedIdentityVerified": True, "configReadback": True, "pendingCleared": True, "sameFrozenBytes": True}
    GATE_ASSETS[name] = [asset]


class Refusal(RuntimeError):
    pass


def need(condition, message):
    if not condition:
        raise Refusal(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def hex_digest(value, size=64):
    return isinstance(value, str) and re.fullmatch("[0-9a-f]{%d}" % size, value) is not None


def exact(a, b):
    return type(a) is type(b) and a == b


def file_at(root, relative):
    need(isinstance(relative, str), "path must be a string")
    p = PurePosixPath(relative)
    need(not p.is_absolute() and relative == p.as_posix() and not ({"..", "."} & set(p.parts)), "unsafe path: " + relative)
    current = root
    for part in p.parts:
        current = current / part
        need(not current.is_symlink(), "symlink is not accepted: " + relative)
    need(current.is_file(), "missing regular file: " + relative)
    return current


def json_read(path):
    def pairs(items):
        result = {}
        for key, value in items:
            need(key not in result, "duplicate JSON key: " + key)
            result[key] = value
        return result
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=pairs)


def git(code, *args):
    return subprocess.check_output(["git", *args], cwd=code, text=True).strip()


def pointer(value, path):
    need(isinstance(path, str) and (not path or path.startswith("/")), "invalid JSON pointer")
    for token in path.split("/")[1:]:
        token = token.replace("~1", "/").replace("~0", "~")
        value = value[int(token)] if isinstance(value, list) else value[token]
    return value


def claim_value(report, location):
    if isinstance(location, str):
        return pointer(report, location)
    need(isinstance(location, dict) and set(location) in ({"pointer", "length"}, {"pointer", "equals"}), "unsupported claim transform")
    value = pointer(report, location["pointer"])
    if "length" in location:
        need(location["length"] is True and isinstance(value, (dict, list)), "length proof requires a JSON collection")
        return len(value)
    return exact(value, location["equals"])


def evidence(root, ref, watched, *, claims=None):
    need(isinstance(ref, dict), "evidence reference must be an object")
    path = ref.get("path")
    need(isinstance(path, str) and path.startswith(".adg/evidence/0.2.21/"), "accepted evidence must be a 0.2.21 file, not an old certificate")
    target = file_at(root, path)
    digest = sha(target.read_bytes())
    need(hex_digest(ref.get("sha256")) and digest == ref["sha256"], "evidence digest drift: " + path)
    watched[path] = digest
    assertions = ref.get("assertions", [])
    bindings = ref.get("claims", {})
    if assertions or bindings:
        report = json_read(target)
        for row in assertions:
            need(exact(pointer(report, row["pointer"]), row["equals"]), "evidence assertion failed: " + path + row["pointer"])
        for name, location in bindings.items():
            need(claims is not None and name in claims, "unknown proof claim: " + name)
            need(exact(claim_value(report, location), claims[name]), "proof claim mismatch: " + name + " in " + path)
    return set(bindings)


def load_contract(root, contract, k):
    need(contract["schema"] == "ocx021-frozen-closure-contract-v1", "wrong frozen contract")
    need(contract["task_ids"] == IDS, "wrong TASK set/order")
    demand = k.RuntimeStore(root).load("DMD", DMD)
    need(demand["acceptance_criteria"] == contract["criteria"], "frozen criteria changed")
    for tid in IDS:
        row = contract["tasks"][tid]
        task = row["task"]
        k.validate_object(task)
        need(task["id"] == tid and task["state"] == "in_progress" and task["result"] == "pending", "bad frozen TASK")
        need(task["implementation_id"] == IMP and task["demand_ids"] == [DMD], "TASK carrier drift")
        need(k.build_acceptance_bindings([demand], task["acceptance_ids"]) == task["acceptance_bindings"], "acceptance binding drift: " + tid)
        path = file_at(root, row["baseline_path"])
        need(sha(path.read_bytes()) == task["baseline_digest"], "original baseline drift: " + tid)
    flat = [aid for tid in IDS for aid in contract["tasks"][tid]["task"]["acceptance_ids"]]
    need(len(flat) == len(set(flat)) == 8 and set(flat) == {r["id"] for r in demand["acceptance_criteria"]}, "8AC must have unique complete TASK coverage")


def prepare(root, code, contract, gates, mapping, paths, k):
    source = gates.get("source_commit") or git(code, "rev-parse", "HEAD")
    need(hex_digest(source, 40), "source commit must be a full SHA")
    tip = git(code, "rev-parse", "HEAD")
    changes = sorted(filter(None, git(code, "diff", "--no-renames", "--name-only", BASE, tip).splitlines()))
    store = k.RuntimeStore(root)
    facts = {target["path"] for tid in IDS for target in contract["tasks"][tid]["task"]["fact_writeback"]["targets"]}
    result = {"schema": "ocx021-final-bindings-v1", "prepared_at": dt.datetime.now(dt.timezone.utc).isoformat(), "docs_root": str(root), "code_root": str(code), "source_commit": source, "source_tree": git(code, "rev-parse", source + "^{tree}"), "input_sha256": {name: sha(path.read_bytes()) for name, path in paths.items()}, "kernel_sha256": sha(Path(k.__file__).read_bytes()), "kernel_contract_digest": k.contract_digest(), "runtime_before": {kind: store.load(kind, oid) for kind, oid in (("DMD", DMD), ("IMP", IMP))}, "fact_after_digests": {p: sha(file_at(root, p).read_bytes()) for p in sorted(facts)}, "code_changes": changes, "code_change_tip": tip, "tasks": {}}
    for tid in IDS:
        task = contract["tasks"][tid]["task"]
        # Conservative scope candidates; caller reviews actual ownership before plan.
        chosen = [p for p in changes if not k.scope_violations([p], task["write_scope"])]
        chosen += [t["path"] for t in task["fact_writeback"]["targets"]]
        result["tasks"][tid] = {"receipt_id": tid.replace("TASK-", "RCP-", 1), "certificate_id": tid.replace("TASK-", "CERT-", 1), "changed_paths": sorted(set(chosen))}
    return result


def validate_gates(root, code, gates, watched):
    need(gates["schema"] == "ocx021-final-gates-v1" and gates["version"] == VERSION and gates["tag"] == TAG, "wrong final version")
    source = gates.get("source_commit")
    need(hex_digest(source, 40) and hex_digest(gates.get("source_tree"), 40), "final source commit/tree missing")
    need(git(code, "rev-parse", source + "^{tree}") == gates["source_tree"], "final tree mismatch")
    need(git(code, "rev-parse", "refs/tags/" + TAG + "^{commit}") == source, "local release tag differs from final source")
    need(not git(code, "diff", "--name-only", source, "HEAD", "--", *PRODUCT), "checkout product inputs differ from final packet")
    need(not git(code, "status", "--porcelain", "--untracked-files=all"), "code checkout has uncommitted files")
    need(set(gates["gates"]) == set(GATE_FACTS), "final gate set incomplete or unexpected")
    evidence(root, gates["asset_report"], watched)
    assets = json_read(file_at(root, gates["asset_report"]["path"]))
    need(assets["result"] == "pass" and assets["sourceCommit"] == source and assets["draft"] is False and assets["prerelease"] is False, "asset report must be final public stable bytes")
    need(type(assets["releaseId"]) is int and assets["releaseId"] > 0 and assets["releaseId"] == gates["release_id"], "release identity mismatch")
    need(len(assets["assets"]) == 8 and {r["name"] for r in assets["assets"]} == ASSETS, "expected exact eight assets")
    by_name = {r["name"]: r for r in assets["assets"]}
    need(len({r["id"] for r in by_name.values()}) == 8, "asset IDs not unique")
    for row in by_name.values():
        need(type(row["id"]) is int and row["id"] > 0 and type(row["bytes"]) is int and row["bytes"] > 0 and hex_digest(row["sha256"]), "invalid asset ID/bytes/hash")
    need(len(assets["signedArtifacts"]) == 3 and set(assets["signedArtifacts"]) == {MAC, MSI, NSIS} and len(assets["platforms"]) == 5 and set(assets["platforms"]) == PLATFORMS, "signature/platform coverage incomplete")
    mac = assets["macBundle"]
    need(mac["id"] == "com.gzers.opencodex.desktop" and mac["version"] == VERSION and mac["embeddedSourceCommit"] == source and mac["deepStrictSignature"] == "pass" and mac["licenseAndNotices"] == "present" and hex_digest(mac["executableSha256"]), "Mac original bundle evidence incomplete")
    need(hex_digest(gates.get("windows_built_exe_sha256")), "final original Windows EXE identity missing")
    for name, expected in GATE_FACTS.items():
        gate = gates["gates"][name]
        need(gate.get("status") == "passed", "pending/failed final gate: " + name)
        need(set(gate["facts"]) == set(expected) and all(exact(gate["facts"][f], v) for f, v in expected.items()), "partial/wrong facts in gate: " + name)
        tested = gate.get("tested_commit")
        need(hex_digest(tested, 40), "gate missing tested source: " + name)
        if name in {"provider-chain", "final-ci"} and tested != source:
            need(bool(gate.get("reuse_note")), "source reuse requires explicit limits: " + name)
            need(not git(code, "diff", "--name-only", tested, source, "--", *PRODUCT), "product source changed; rerun " + name)
        else:
            need(git(code, "rev-parse", tested + "^{tree}") == gates["source_tree"], "gate is not from final source tree: " + name)
        claims = {"tested_commit": tested, "release_id": gates["release_id"]}
        claims.update({"facts/" + key: value for key, value in expected.items()})
        if name in GATE_ASSETS:
            claims["executable_sha256"] = mac["executableSha256"] if name in {"native-macos", "ota-macos-020-to-021"} else gates["windows_built_exe_sha256"]
            need(gate.get("executable_sha256") == claims["executable_sha256"], "original executable identity differs: " + name)
        proven = set()
        need(bool(gate.get("evidence")), "gate lacks evidence: " + name)
        for ref in gate["evidence"]:
            proven.update(evidence(root, ref, watched, claims=claims))
        required = set(claims) - ({"release_id"} if name == "provider-chain" else set())
        need(required <= proven, "unproven gate fields {}: {}".format(name, sorted(required - proven)))
        names = GATE_ASSETS.get(name, [])
        need(set(gate.get("bound_artifacts", {})) == set(names), "wrong package binding: " + name)
        for asset in names:
            need(gate["bound_artifacts"][asset] == by_name[asset]["sha256"], "gate used different package bytes: " + name)
    need(isinstance(gates["completed_at"], str) and dt.datetime.fromisoformat(gates["completed_at"]).utcoffset() is not None, "completed_at must include timezone")
    need(dt.datetime.fromisoformat(gates["completed_at"]) <= dt.datetime.now(dt.timezone.utc), "completed_at cannot be in the future")
    need(bool(gates.get("summary")), "final truthful summary missing")


def validate_map(root, contract, mapping, gates, watched):
    need(mapping["schema"] == "ocx021-evidence-map-v1" and mapping["source_commit"] == gates["source_commit"], "map final source mismatch")
    need(set(mapping["tasks"]) == set(IDS), "map TASK set mismatch")
    for tid in IDS:
        task = contract["tasks"][tid]["task"]
        row = mapping["tasks"][tid]
        for field, expected in (("acceptance", task["acceptance_ids"]), ("checks", task["required_checks"])):
            need(set(row[field]) == set(expected), "missing/extra " + field + " in " + tid)
            for name in expected:
                decision = row[field][name]
                need(decision.get("status") == "passed" and bool(decision.get("summary")), "not accepted: " + tid + " / " + name)
                need(bool(decision.get("evidence")), "accepted row lacks evidence: " + name)
                for ref in decision["evidence"]:
                    evidence(root, ref, watched)
                # Candidate/old failure arrays are deliberately not converted to passed proofs.


def source_candidates(root, binding, contract, k):
    before = binding["runtime_before"]
    need(set(before) == {"DMD", "IMP"}, "wrong source runtime baseline")
    for kind, oid in (("DMD", DMD), ("IMP", IMP)):
        obj = before[kind]
        k.validate_object(obj)
        need(obj["id"] == oid and obj["state"] == "in_progress", "capture projections while still open")
    need(before["DMD"]["acceptance_criteria"] == contract["criteria"], "DMD criteria changed")
    need(before["IMP"]["task_ids"] == IDS and before["IMP"]["demand_ids"] == [DMD] and before["DMD"]["implementation_ids"] == [IMP], "source carriers drifted")
    result = {}
    for kind in ("IMP", "DMD"):
        old = before[kind]
        new = k.sync_source_bound_object(old, root, expected_digest=old["content_digest"])
        need(new["state"] == "completed", "parent must stage final completed source: " + kind)
        allowed = {"state", "revision", "source_digest", "source_revision", "content_digest", "summary_digest"}
        allowed.update({"completion_summary"} if kind == "IMP" else {"resolution", "next_action"})
        need({f for f in set(old) | set(new) if old.get(f) != new.get(f)} <= allowed, "sync final human summaries while in_progress before staging completed source: " + kind)
        result[kind] = new
    return result


def make_plan(root, code, contract, mapping, gates, binding, paths, k):
    watched = {}
    load_contract(root, contract, k)
    validate_gates(root, code, gates, watched)
    validate_map(root, contract, mapping, gates, watched)
    need(binding["schema"] == "ocx021-final-bindings-v1" and binding["docs_root"] == str(root) and binding["code_root"] == str(code), "bindings point at wrong roots")
    need(binding["kernel_sha256"] == sha(Path(k.__file__).read_bytes()) and binding["kernel_contract_digest"] == k.contract_digest(), "kernel changed since preparation")
    need(binding["source_commit"] == gates["source_commit"] and binding["source_tree"] == gates["source_tree"], "bindings final source drift")
    for name, path in paths.items():
        need(binding["input_sha256"][name] == sha(path.read_bytes()), "input changed after --prepare: " + name)
        watched[path.relative_to(root).as_posix()] = sha(path.read_bytes())
    targets = {x["path"] for tid in IDS for x in contract["tasks"][tid]["task"]["fact_writeback"]["targets"]}
    need(set(binding["fact_after_digests"]) == targets, "fact targets mismatch")
    for p in targets:
        digest = sha(file_at(root, p).read_bytes())
        need(digest == binding["fact_after_digests"][p], "final fact changed since --prepare: " + p)
        watched[p] = digest
    relpath = "docs/03-开发实施/" + REL + ".md"
    need(k._source_frontmatter(file_at(root, relpath)).get("state") == "verified", "parent REL24 must record final verified stable facts")
    source = source_candidates(root, binding, contract, k)
    need(git(code, "rev-parse", "HEAD") == binding["code_change_tip"], "QA change tip drifted since prepare")
    actual = sorted(filter(None, git(code, "diff", "--no-renames", "--name-only", BASE, binding["code_change_tip"]).splitlines()))
    need(binding["code_changes"] == actual, "declared product change set drifted")
    need(set(binding["tasks"]) == set(IDS), "binding TASK set mismatch")
    closures, covered = {}, set()
    for tid in IDS:
        task = contract["tasks"][tid]["task"]
        settings = binding["tasks"][tid]
        need(settings["receipt_id"] == tid.replace("TASK-", "RCP-", 1) and settings["certificate_id"] == tid.replace("TASK-", "CERT-", 1), "wrong/old receipt or certificate identity")
        changed = settings["changed_paths"]
        need(isinstance(changed, list) and changed == sorted(set(changed)), "changed_paths must be unique sorted paths")
        need(not k.scope_violations(changed, task["write_scope"]), "TASK changed path outside frozen scope: " + tid)
        fact_paths = {t["path"] for t in task["fact_writeback"]["targets"]}
        for p in changed:
            if p in actual:
                covered.add(p)
            else:
                need(p in fact_paths or p.startswith(".adg/evidence/0.2.21/"), "not a real product change or owned fact/evidence: " + p)
                watched[p] = sha(file_at(root, p).read_bytes())
        need(fact_paths <= set(changed), "all fact impacts must appear in changed_paths")
        impacts = [{"target": t["id"], "path": t["path"], "after_digest": binding["fact_after_digests"][t["path"]], "summary": "Parent's final 0.2.21 facts, exact bytes bound after stable gates."} for t in task["fact_writeback"]["targets"]]
        location = paths["map"].relative_to(root).as_posix()
        checkrows = [{"name": name, "status": "passed", "evidence": location + "#/tasks/" + tid + "/checks/" + name.replace("/", "~1")} for name in task["required_checks"]]
        acrows = [{"acceptance_id": aid, "status": "passed", "evidence": location + "#/tasks/" + tid + "/acceptance/" + aid} for aid in task["acceptance_ids"]]
        receipt = k.build_receipt(task, receipt_id=settings["receipt_id"], result="succeeded", changed_paths=changed, checks=checkrows, acceptance_results=acrows, fact_impacts=impacts, completed_at=gates["completed_at"], summary=gates["summary"] + "; final-input-sha256=" + binding["input_sha256"]["gates"])
        closed, cert = k.close_task(task, receipt, expected_task_digest=task["content_digest"], certificate_id=settings["certificate_id"], verified_at=gates["completed_at"])
        k.verify_closed_task(closed, receipt, cert)
        closures[tid] = (closed, receipt, cert)
    need(covered == set(actual), "unassigned actual product changes: " + repr(sorted(set(actual) - covered)))
    # Pure derived-lifecycle validation; does not persist or change Markdown.
    imp = k.complete_implementation(binding["runtime_before"]["IMP"], list(closures.values()), expected_digest=binding["runtime_before"]["IMP"]["content_digest"], completion_summary=source["IMP"]["completion_summary"])
    dmd = k.complete_demand(binding["runtime_before"]["DMD"], [imp], expected_digest=binding["runtime_before"]["DMD"]["content_digest"], resolution=source["DMD"]["resolution"])
    for kind, expected in (("IMP", imp), ("DMD", dmd)):
        ignore = {"source_digest", "source_revision", "content_digest", "summary_digest"}
        need(all(source[kind].get(key) == expected.get(key) for key in set(source[kind]) | set(expected) if key not in ignore), "derived source close differs: " + kind)
    return {"source": source, "closures": closures, "watched": watched}


def verify_current(root, code, plan, binding, gates, k):
    store = k.RuntimeStore(root)
    need(git(code, "rev-parse", "HEAD") == binding["code_change_tip"], "QA change tip changed during close")
    need(binding["kernel_sha256"] == sha(Path(k.__file__).read_bytes()) and binding["kernel_contract_digest"] == k.contract_digest(), "kernel changed during close")
    for p, digest in plan["watched"].items():
        need(sha(file_at(root, p).read_bytes()) == digest, "source/evidence changed during close: " + p)
    need(not git(code, "diff", "--name-only", gates["source_commit"], "HEAD", "--", *PRODUCT) and not git(code, "status", "--porcelain", "--untracked-files=all"), "product source changed during close")
    need(git(code, "rev-parse", "refs/tags/" + TAG + "^{commit}") == gates["source_commit"], "release tag changed during close")
    for tid, (closed, receipt, cert) in plan["closures"].items():
        current = store.load("TASK", tid)
        if current["state"] == "completed":
            actual_receipt = store.load("RCP", receipt["id"])
            actual_cert = store.load("CERT", cert["id"])
            k.verify_closed_task(current, actual_receipt, actual_cert)
            need((current, actual_receipt, actual_cert) == (closed, receipt, cert), "existing closure is not this exact final plan: " + tid)
        else:
            need(current["content_digest"] == receipt["task_digest"], "frozen TASK drifted before close: " + tid)
    for kind, oid in (("IMP", IMP), ("DMD", DMD)):
        current = store.load(kind, oid)
        need(current in (binding["runtime_before"][kind], plan["source"][kind]), "source projection changed after --prepare: " + kind)


def owned_destination(path):
    destination = Path(path).absolute()
    need(destination.parent.resolve() == HERE and not destination.is_symlink(), "output must be directly in final-closeout-20261104")
    need(not destination.exists(), "refuse overwriting existing evidence: " + str(destination))
    return destination


def write_owned(path, payload):
    destination = owned_destination(path)
    with destination.open("x", encoding="utf-8") as handle:
        handle.write(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docs-root", type=Path, default=DOCS)
    parser.add_argument("--code-root", type=Path, default=CODE)
    parser.add_argument("--kernel-lib", type=Path, default=LIB)
    parser.add_argument("--contracts", type=Path, default=HERE / "frozen-task-contracts.json")
    parser.add_argument("--map", type=Path, default=HERE / "evidence-map.template.json")
    parser.add_argument("--gates", type=Path, default=HERE / "final-gates.template.json")
    parser.add_argument("--bindings", type=Path, default=HERE / "final-bindings.json")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--prepare", type=Path, metavar="NEW_BINDINGS_JSON", help="capture only; never modifies runtime or facts")
    mode.add_argument("--commit", action="store_true", help="after all gates pass, commit three TASK closures and sync prepared source projections")
    parser.add_argument("--output", type=Path, help="new plan/result JSON within this directory")
    args = parser.parse_args(argv)
    root, code = args.docs_root.resolve(), args.code_root.resolve()
    need(root == DOCS and code == CODE.resolve(), "only the actual 0.2.21 roots are admitted")
    need(args.kernel_lib.resolve() == LIB.resolve(), "only the actual installed governance kernel is admitted")
    need(not (args.prepare and args.output), "--prepare already names its output; do not add --output")
    if args.output:
        owned_destination(args.output)
    sys.path.insert(0, str(args.kernel_lib.resolve()))
    from runtime import kernel as k
    paths = {"contracts": args.contracts.resolve(), "map": args.map.resolve(), "gates": args.gates.resolve()}
    for p in paths.values():
        need(p.parent == HERE and p.is_file(), "input must be in the owned final-closeout directory")
    contract, mapping, gates = (json_read(paths[name]) for name in ("contracts", "map", "gates"))
    load_contract(root, contract, k)
    if args.prepare:
        result = prepare(root, code, contract, gates, mapping, paths, k)
        write_owned(args.prepare, result)
        print(json.dumps({"mode": "prepare-only", "runtime_written": False, "bindings": str(args.prepare), "warning": "Captured values are not acceptance; all final gates and source stages still require validation."}, ensure_ascii=False))
        return 0
    pending = [name for name in GATE_FACTS if gates.get("gates", {}).get(name, {}).get("status") != "passed"]
    if pending:
        report = {"mode": "blocked-plan-only", "runtime_written": False, "pending_final_gates": pending, "pending_acceptance": [], "pending_required_checks": []}
        for tid in IDS:
            task = contract["tasks"][tid]["task"]
            for name, source_field, report_field in (("acceptance_ids", "acceptance", "pending_acceptance"), ("required_checks", "checks", "pending_required_checks")):
                for item in task[name]:
                    if mapping.get("tasks", {}).get(tid, {}).get(source_field, {}).get(item, {}).get("status") != "passed":
                        report[report_field].append({"task_id": tid, "id": item})
        if args.output:
            write_owned(args.output, report)
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 2
    binding_path = args.bindings.resolve()
    need(binding_path.parent == HERE and not args.bindings.is_symlink(), "bindings must be an owned regular input")
    binding = json_read(binding_path)
    plan = make_plan(root, code, contract, mapping, gates, binding, paths, k)
    plan["watched"][binding_path.relative_to(root).as_posix()] = sha(binding_path.read_bytes())
    verify_current(root, code, plan, binding, gates, k)
    if args.commit:
        store = k.RuntimeStore(root)
        # The real kernel provides atomicity per TASK, not across all three TASKs.
        # Exact deterministic receipts permit a safe continuation after interruption.
        for tid in IDS:
            verify_current(root, code, plan, binding, gates, k)
            closed, receipt, cert = plan["closures"][tid]
            if store.load("TASK", tid)["state"] != "completed":
                original = contract["tasks"][tid]["task"]
                stored = store.commit_task_closure(original, receipt, expected_task_digest=original["content_digest"], certificate_id=cert["id"], verified_at=gates["completed_at"])
                need(stored == (closed, receipt, cert), "kernel persisted unexpected closure")
            verify_current(root, code, plan, binding, gates, k)
            print(json.dumps({"committed_task": tid, "certificate": cert["id"]}), flush=True)
        for kind, oid in (("IMP", IMP), ("DMD", DMD)):
            verify_current(root, code, plan, binding, gates, k)
            old = store.load(kind, oid)
            if old != plan["source"][kind]:
                store.put(plan["source"][kind], expected_digest=old["content_digest"])
            k.verify_source_binding(store.load(kind, oid), root)
        verify_current(root, code, plan, binding, gates, k)
    result = {"mode": "committed" if args.commit else "validated-plan-only", "runtime_written": args.commit, "source_commit": gates["source_commit"], "release_id": gates["release_id"], "task_count": 3, "acceptance_count": 8, "required_check_count": 17, "tasks": [{"id": tid, "receipt": r["id"], "receipt_digest": r["content_digest"], "certificate": c["id"], "certificate_digest": c["content_digest"]} for tid, (_, r, c) in plan["closures"].items()]}
    if args.output:
        write_owned(args.output, result)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (Refusal, OSError, KeyError, TypeError, ValueError, subprocess.CalledProcessError, RuntimeError) as exc:
        print(json.dumps({"result": "refused", "reason": str(exc), "note": "No all-task success is claimed. If an explicit commit was interrupted, inspect exact stored TASK/RCP/CERT before continuing."}, ensure_ascii=False), file=sys.stderr)
        raise SystemExit(2)
