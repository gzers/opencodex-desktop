"""Preparation/denial tests only: never invokes any closure or store mutation."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("ocx021_closeout", HERE / "formal-closeout.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
sys.path.insert(0, str(m.LIB))
from runtime import kernel as k


class PreparationOnlyTests(unittest.TestCase):
    def test_actual_frozen_tasks_and_original_baselines(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        m.load_contract(m.DOCS, contract, k)
        store = k.RuntimeStore(m.DOCS)
        for tid in m.IDS:
            self.assertEqual(contract["tasks"][tid]["task"], store.load("TASK", tid))
        self.assertEqual(sum(len(v["task"]["required_checks"]) for v in contract["tasks"].values()), 17)

    def test_criteria_cannot_be_rewritten(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        contract["criteria"][0]["text"] += " weakened"
        with self.assertRaisesRegex(m.Refusal, "criteria changed"):
            m.load_contract(m.DOCS, contract, k)

    def test_original_task_baseline_cannot_be_replaced(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        contract["tasks"][m.IDS[0]]["baseline_path"] = ".adg/evidence/0.2.21/implementation-20261104/sync-baseline.json"
        with self.assertRaisesRegex(m.Refusal, "baseline drift"):
            m.load_contract(m.DOCS, contract, k)

    def test_actual_scope_rejects_product_and_fact_leaks(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        modal = contract["tasks"][m.IDS[0]]["task"]
        sync = contract["tasks"][m.IDS[1]]["task"]
        self.assertTrue(k.scope_violations(["apps/desktop/tauri/src/commands/model_config.rs"], modal["write_scope"]))
        self.assertTrue(k.scope_violations(["docs/02-项目核心/UI规范.md"], sync["write_scope"]))
        self.assertEqual(k.scope_violations([".adg/evidence/0.2.21/final-closeout-20261104/README.md"], sync["write_scope"]), [])

    def test_default_actual_plan_blocks_and_does_not_write_runtime(self):
        paths = [m.DOCS / ".adg/work/objects/task" / (tid + ".json") for tid in m.IDS]
        paths += [m.DOCS / ".adg/work/objects" / kind.lower() / (oid + ".json") for kind, oid in (("DMD", m.DMD), ("IMP", m.IMP))]
        before = {p: p.read_bytes() for p in paths}
        stream = io.StringIO()
        with patch.object(k.RuntimeStore, "commit_task_closure", side_effect=AssertionError("must never be called")), patch.object(k.RuntimeStore, "put", side_effect=AssertionError("must never be called")), patch.object(k, "close_task", side_effect=AssertionError("must never be called")), redirect_stdout(stream):
            self.assertEqual(m.main([]), 2)
        report = json.loads(stream.getvalue())
        self.assertFalse(report["runtime_written"])
        self.assertEqual(len(report["pending_final_gates"]), 13)
        self.assertEqual(len(report["pending_acceptance"]), 8)
        self.assertEqual(len(report["pending_required_checks"]), 17)
        self.assertEqual(before, {p: p.read_bytes() for p in paths})

    def test_all_status_strings_passed_do_not_replace_real_final_identity(self):
        gates = m.json_read(HERE / "final-gates.template.json")
        for row in gates["gates"].values():
            row["status"] = "passed"
        gates["source_commit"] = None
        with self.assertRaisesRegex(m.Refusal, "source commit/tree missing"):
            m.validate_gates(m.DOCS, m.CODE, gates, {})

    def test_every_ac_and_required_check_is_explicit_and_pending(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        mapping = m.json_read(HERE / "evidence-map.template.json")
        for tid in m.IDS:
            task = contract["tasks"][tid]["task"]
            for field, source in (("acceptance", "acceptance_ids"), ("checks", "required_checks")):
                self.assertEqual(set(mapping["tasks"][tid][field]), set(task[source]))
                for row in mapping["tasks"][tid][field].values():
                    self.assertEqual(row["status"], "pending")
                    self.assertEqual(row["evidence"], [])
                    for ref in row["candidateEvidence"]:
                        self.assertEqual(m.sha(m.file_at(m.DOCS, ref["path"]).read_bytes()), ref["sha256"])

    def test_candidates_cannot_implicitly_pass_an_ac(self):
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        mapping = m.json_read(HERE / "evidence-map.template.json")
        mapping["source_commit"] = "a" * 40
        with self.assertRaisesRegex(m.Refusal, "not accepted"):
            m.validate_map(m.DOCS, contract, mapping, {"source_commit": "a" * 40}, {})

    def test_old_release_certificate_is_not_accepted_evidence(self):
        with self.assertRaisesRegex(m.Refusal, "not an old certificate"):
            m.evidence(m.DOCS, {"path": ".adg/work/objects/cert/CERT-OPENCODEX-UI020-CLOSEOUT-20261104.json", "sha256": "a" * 64}, {})

    def test_actual86_provider_claims_and_nonrestart_boundaries(self):
        gates = m.json_read(HERE / "final-gates.template.json")
        row = gates["gates"]["provider-chain"]
        claims = {"tested_commit": row["tested_commit"]}
        claims.update({"facts/" + key: value for key, value in row["facts"].items()})
        proven = set()
        for ref in row["evidence"]:
            proven.update(m.evidence(m.DOCS, ref, {}, claims=claims))
        self.assertEqual(set(claims), proven)
        self.assertEqual(row["facts"], m.GATE_FACTS["provider-chain"])
        actual = m.json_read(m.file_at(m.DOCS, row["evidence"][0]["path"]))
        self.assertEqual(actual["sourceFreeze"], "86eee1dbff23823eead733cb65308cf02fd95f9e")
        self.assertFalse(actual["globalRestartCalled"])
        self.assertTrue(actual["actualPendingRetained"])
        self.assertFalse(actual["actualGuiRestartVerified"])
        self.assertFalse(actual["actualPendingAcknowledged"])
        self.assertEqual(row["status"], "pending-final-source-binding")

    def test_progress_accepts_only_four_bounded_gates(self):
        gates = m.json_read(HERE / "final-gates.progress.json")
        passed = {name for name, row in gates["gates"].items() if row["status"] == "passed"}
        self.assertEqual(passed, {"provider-chain", "artifacts-signatures", "windows-msi-custom", "windows-nsis-custom"})
        for name in passed:
            row = gates["gates"][name]
            self.assertEqual(row["facts"], m.GATE_FACTS[name])
            claims = {"tested_commit": row["tested_commit"], "release_id": gates["release_id"]}
            claims.update({"facts/" + key: value for key, value in row["facts"].items()})
            if name in m.GATE_ASSETS:
                claims["executable_sha256"] = row["executable_sha256"]
            proven = set()
            for ref in row["evidence"]:
                proven.update(m.evidence(m.DOCS, ref, {}, claims=claims))
            required = set(claims) - ({"release_id"} if name == "provider-chain" else set())
            self.assertTrue(required <= proven)
        self.assertEqual(gates["gates"]["native-windows"]["subchecks"]["officialDefaultBinding"], "passed")
        self.assertEqual(gates["gates"]["native-windows"]["status"], "pending")
        self.assertEqual(gates["gates"]["public-stable"]["status"], "pending")

    def test_progress_is_not_overall_acceptance(self):
        mapping = m.json_read(HERE / "evidence-map.progress.json")
        contract = m.json_read(HERE / "frozen-task-contracts.json")
        count = {"acceptance": {"passed": 0, "pending": 0}, "checks": {"passed": 0, "pending": 0}}
        for tid in m.IDS:
            task = contract["tasks"][tid]["task"]
            for field, key in (("acceptance", "acceptance_ids"), ("checks", "required_checks")):
                self.assertEqual(set(mapping["tasks"][tid][field]), set(task[key]))
                for row in mapping["tasks"][tid][field].values():
                    count[field][row["status"]] += 1
                    if row["status"] == "pending":
                        self.assertTrue(row["remaining_gate_keys"])
        self.assertEqual(count, {"acceptance": {"passed": 1, "pending": 7}, "checks": {"passed": 10, "pending": 7}})
        self.assertEqual(len(mapping["retainedFailures"]), 5)

    def test_output_cannot_escape_write_set_or_overwrite(self):
        with self.assertRaisesRegex(m.Refusal, "output must be"):
            m.owned_destination(m.DOCS / ".adg/work/unsafe-result.json")
        with self.assertRaisesRegex(m.Refusal, "overwriting existing evidence"):
            m.owned_destination(HERE / "formal-closeout.py")

    def test_evidence_hash_assertions_and_proven_types(self):
        with tempfile.TemporaryDirectory(prefix="proof-test-", dir=HERE) as folder:
            root = Path(folder)
            rel = ".adg/evidence/0.2.21/synthetic-validator-only.json"
            path = root / rel
            path.parent.mkdir(parents=True)
            raw = {"scope": "synthetic validator fixture; not project acceptance", "result": "pass", "source": "a" * 40, "checks": {"restarted": False}, "assets": [1, 2, 3]}
            path.write_text(json.dumps(raw))
            ref = {"path": rel, "sha256": m.sha(path.read_bytes()), "claims": {"restarted": "/checks/restarted"}, "assertions": [{"pointer": "/result", "equals": "pass"}]}
            self.assertEqual(m.evidence(root, ref, {}, claims={"restarted": False}), {"restarted"})
            with self.assertRaisesRegex(m.Refusal, "proof claim mismatch"):
                m.evidence(root, ref, {}, claims={"restarted": True})
            with self.assertRaisesRegex(m.Refusal, "proof claim mismatch"):
                m.evidence(root, ref, {}, claims={"restarted": 0})
            bad = copy.deepcopy(ref); bad["sha256"] = "b" * 64
            with self.assertRaisesRegex(m.Refusal, "digest drift"):
                m.evidence(root, bad, {}, claims={"restarted": False})
            path.write_text(json.dumps({**raw, "result": "failed"}))
            ref["sha256"] = m.sha(path.read_bytes())
            with self.assertRaisesRegex(m.Refusal, "assertion failed"):
                m.evidence(root, ref, {}, claims={"restarted": False})

    def test_collection_and_status_transforms_do_not_invent_success(self):
        raw = {"assets": [1, 2, 3], "result": "skipped"}
        self.assertEqual(m.claim_value(raw, {"pointer": "/assets", "length": True}), 3)
        self.assertIs(m.claim_value(raw, {"pointer": "/result", "equals": "pass"}), False)
        with self.assertRaises(m.Refusal):
            m.claim_value(raw, {"pointer": "/assets", "length": False})

    def test_unsafe_paths_and_duplicate_json_are_refused(self):
        with self.assertRaisesRegex(m.Refusal, "unsafe path"):
            m.file_at(m.DOCS, "../ocx-0219-fixes/CHANGELOG.md")
        with tempfile.TemporaryDirectory(prefix="json-test-", dir=HERE) as folder:
            path = Path(folder) / "duplicate.json"
            path.write_text('{"status":"failed","status":"passed"}')
            with self.assertRaisesRegex(m.Refusal, "duplicate JSON key"):
                m.json_read(path)


if __name__ == "__main__":
    unittest.main(verbosity=2)
