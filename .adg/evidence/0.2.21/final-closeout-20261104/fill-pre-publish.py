#!/usr/bin/env python3
"""Merge three real final reports into ready closure INPUTS only; never close anything.

CI input: gh run view41817249379 --json databaseId,headSha,status,conclusion,jobs,url.
Public assets: parent's verify-uploaded-assets.py --stage public result.
Anonymous input: parent's verify-anonymous-public.py result, downloaded files alongside.
No network, Git mutation, Markdown edit, kernel import or runtime write.
"""
import argparse
import copy
import datetime as dt
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ocx021_closeout", HERE / "formal-closeout.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
SOURCE = "6e2197fd0b8a36dcb8d893b62693c5a2a77151cc"
CI_SOURCE = "20d13334ab9127e013ba6adca8c19023472d015f"
RELEASE_ID, CI_RUN = 433749817, 41817249379
REQUIRED_JOBS = {"frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"}


def original(path):
    path = path.resolve()
    rel = path.relative_to(m.DOCS).as_posix()
    m.need(rel.startswith(".adg/evidence/0.2.21/"), "only actual0.2.21 proof paths accepted")
    path = m.file_at(m.DOCS, rel)
    return m.json_read(path), {"path": rel, "sha256": m.sha(path.read_bytes())}


def asset_identities(rows):
    m.need(len(rows) == 8 and {x["name"] for x in rows} == m.ASSETS, "exact eight assets required")
    return sorted([{key: row[key] for key in ("name", "id", "bytes", "sha256")} for row in rows], key=lambda x: x["name"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ci", required=True, type=Path)
    parser.add_argument("--public-assets", required=True, type=Path)
    parser.add_argument("--anonymous-public", required=True, type=Path)
    args = parser.parse_args()
    # Refuse overwriting previous inputs even if a later check fails.
    for filename in ("final-gates.json", "evidence-map.json", "final-input-merge-receipt.json"):
        m.owned_destination(HERE / filename)
    ci, ci_ref = original(args.ci)
    public, public_ref = original(args.public_assets)
    anon, anon_ref = original(args.anonymous_public)
    gates = m.json_read(HERE / "final-gates.pre-publish.json")
    mapping = m.json_read(HERE / "evidence-map.pre-publish.json")
    previous = m.json_read(m.file_at(m.DOCS, gates["asset_report"]["path"]))
    m.need(ci["databaseId"] == CI_RUN and ci["headSha"] == CI_SOURCE, "wrong final CI run/source")
    m.need(ci["status"] == "completed" and ci["conclusion"] == "success", "final CI still running/failed")
    jobs = {row["name"]: row for row in ci["jobs"]}
    m.need(len(jobs) == len(ci["jobs"]) and REQUIRED_JOBS <= set(jobs), "missing/duplicate required CI jobs")
    for name in REQUIRED_JOBS:
        m.need(jobs[name]["status"] == "completed" and jobs[name]["conclusion"] == "success", "CI job not passed: " + name)
    excluded = []
    for name in set(jobs) - REQUIRED_JOBS:
        m.need(name == "build" and jobs[name]["conclusion"] in {"success", "skipped"}, "unexpected incomplete CI job: " + name)
        if jobs[name]["conclusion"] == "skipped":
            excluded.append("main-only build job skipped on feature ref; excluded, never counted as passed; exact package gates separate")
    m.need(not m.git(m.CODE, "diff", "--name-only", CI_SOURCE, SOURCE, "--", *m.PRODUCT), "CI/product inputs differ")
    for report in (public, anon):
        m.need(report["result"] == "pass" and report["sourceCommit"] == SOURCE and report["releaseId"] == RELEASE_ID, "wrong or failed public proof")
    m.need(public["draft"] is False and public["prerelease"] is False, "public assets are not stable yet")
    expected = asset_identities(previous["assets"])
    m.need(asset_identities(public["assets"]) == expected == asset_identities(anon["assets"]), "same8asset ID/bytes/SHA changed")
    m.need(public["macBundle"] == previous["macBundle"] and set(public["signedArtifacts"]) == set(previous["signedArtifacts"]) and set(public["platforms"]) == m.PLATFORMS, "final original signature/bundle/platform differs")
    m.need(anon["tag"] == anon["latest"] == m.TAG and anon["anonymousTagReadback"] is True and anon["anonymousLatestReadback"] is True and anon["sameAssetIdentity"] is True and anon["credentialHeadersUsed"] is False, "anonymous stable/Latest proof incomplete")
    latest_hash = next(row["sha256"] for row in expected if row["name"] == "latest.json")
    m.need(anon["manifestSha256"] == latest_hash, "anonymous manifest differs")
    # The parent's verifier saves every anonymous download. Verify exact retained bytes again.
    for row in anon["assets"]:
        m.need(row["anonymousDownloadPassed"] is True, "anonymous asset not passed")
        path = args.anonymous_public.resolve().parent / row["name"]
        m.need(path.is_file() and path.stat().st_size == row["bytes"] and m.sha(path.read_bytes()) == row["sha256"], "retained anonymous asset bytes differ: " + row["name"])
    m.need({name for name, row in gates["gates"].items() if row["status"] != "passed"} == {"final-ci", "public-stable", "artifacts-signatures"}, "unexpected pre-publish pending gates")
    now = dt.datetime.now(dt.timezone.utc).isoformat()
    record = {"recordedAt": now, "sourceCommit": SOURCE, "releaseId": RELEASE_ID, "result": "verified-final-inputs-only-no-closure", "rawInputs": [ci_ref, public_ref, anon_ref], "finalGateFacts": {}, "excludedCiJobs": excluded, "ciRequiredJobs": sorted(REQUIRED_JOBS), "sourceReuse": "CI20d test-only source has exact same product inputs as release6e; package bytes unchanged", "closureExecuted": False}
    for name in ("final-ci", "public-stable", "artifacts-signatures"):
        record["finalGateFacts"][name] = {"testedCommit": CI_SOURCE if name == "final-ci" else SOURCE, "releaseId": RELEASE_ID, "facts": copy.deepcopy(m.GATE_FACTS[name])}
    receipt_path = HERE / "final-input-merge-receipt.json"
    m.write_owned(receipt_path, record)
    receipt_ref = {"path": receipt_path.relative_to(m.DOCS).as_posix(), "sha256": m.sha(receipt_path.read_bytes())}
    for name, final in record["finalGateFacts"].items():
        row = gates["gates"][name]
        row.update(status="passed", tested_commit=final["testedCommit"], facts=final["facts"], missing_inputs=[])
        row.pop("scope_status", None)
        claims = {"tested_commit": "/finalGateFacts/" + name + "/testedCommit", "release_id": "/releaseId"}
        claims.update({"facts/" + key: "/finalGateFacts/" + name + "/facts/" + key for key in final["facts"]})
        row["evidence"] = [{**receipt_ref, "claims": claims}, ci_ref if name == "final-ci" else public_ref, anon_ref]
        if name == "final-ci":
            row["reuse_note"] = record["sourceReuse"]
            row["test_source_commit"] = CI_SOURCE
    gates.update(completed_at=now, asset_report=public_ref, asset_report_stage="public-stable", stage="final-inputs-only; parent-human-facts-before-prepare", summary="Same original6e release433749817 has all13 final gates verified: CI20d same product, two native platforms, four applicable installation cells, three actual adjacent OTA, stable/anonymous8assets/3signatures/5platforms. Historical failures and scope limits retained; no task closure by this tool.", progress_counts={"passed": 13, "pending": 0})
    gates["scope_boundaries"].update(public_beta_only=False, stable_promotion_executed=True)
    # Resolve only the three known common-acceptance rows after raw final checks pass.
    for tid, task in mapping["tasks"].items():
        for field in ("acceptance", "checks"):
            for name, row in task[field].items():
                if row["status"] == "passed":
                    continue
                m.need(name in {"AC-021-08", "review:full-regression", "review:release-and-ota"}, "unexpected pending acceptance: " + name)
                row.update(status="passed", remaining_gate_keys=[], remaining="None; all final input gates verified, runtime closure still separate.", summary="Actual final CI completed successfully on20d same-product6e; same8 original assets/signatures and anonymoustag/Latest stable verified. Two native platforms/four applicable installs/three actual020→021OTA and all historical boundaries remain bound; no old certificate or skipped test counted as pass.")
                row["source_originals"] += row["evidence"]
                row["evidence"] = [{**receipt_ref, "assertions": [{"pointer": "/sourceCommit", "equals": SOURCE}, {"pointer": "/releaseId", "equals": RELEASE_ID}, {"pointer": "/result", "equals": "verified-final-inputs-only-no-closure"}]}, ci_ref, public_ref, anon_ref]
    mapping.update(stage="final-inputs-only; not-runtime-closed", final_input_receipt=receipt_ref, counts={"acceptance": {"total": 8, "passed": 8, "pending": 0}, "checks": {"total": 17, "passed": 17, "pending": 0}})
    # Read-only verification with the exact prepared tool; no RuntimeStore/kernel lifecycle call.
    m.validate_gates(m.DOCS, m.CODE, gates, {})
    m.validate_map(m.DOCS, m.json_read(HERE / "frozen-task-contracts.json"), mapping, gates, {})
    m.write_owned(HERE / "final-gates.json", gates)
    mapping["gates_progress"] = {"path": (HERE / "final-gates.json").relative_to(m.DOCS).as_posix(), "sha256": m.sha((HERE / "final-gates.json").read_bytes())}
    m.write_owned(HERE / "evidence-map.json", mapping)
    print(json.dumps({"result": "final-inputs-verified-only", "gates": "final-gates.json", "map": "evidence-map.json", "runtime_written": False, "next": "Parent writes final human facts in two stages, then explicit prepare/plan/commit."}, ensure_ascii=False))


if __name__ == "__main__":
    try:
        main()
    except (m.Refusal, OSError, KeyError, ValueError, RuntimeError) as exc:
        print(json.dumps({"result": "refused-no-closure", "reason": str(exc)}, ensure_ascii=False), file=sys.stderr)
        raise SystemExit(2)
