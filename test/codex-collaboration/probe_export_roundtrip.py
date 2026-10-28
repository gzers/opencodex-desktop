#!/usr/bin/env python3
"""Probe synthetic history transfer between isolated Codex homes with a local stub.

This tests native rollout path forking, not a supported portable export product,
real desktop integration, child-tree restore, or cross-version compatibility.
"""

import sys
sys.dont_write_bytecode = True

import argparse
from datetime import datetime, timezone
import hashlib
from io import BytesIO
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import traceback
from http.server import ThreadingHTTPServer

from probe_native import AppServer, LocalResponses, RpcError


class HistoryResponses(LocalResponses):
    def do_POST(self):
        if self.path != "/v1/responses":
            return super().do_POST()
        body = self.rfile.read(int(self.headers["Content-Length"]))
        self.server.inputs.append(json.loads(body).get("input", []))
        original = self.rfile
        self.rfile = BytesIO(body)
        try:
            return super().do_POST()
        finally:
            self.rfile = original


def initialize(server):
    server.call("initialize", {
        "clientInfo": {"name": "export_roundtrip_audit", "version": "0.1"},
        "capabilities": {"experimentalApi": True},
    })
    server.send({"method": "initialized", "params": {}})


def probe(binary, report):
    def record(name, observed):
        report["checks"].append({"name": name, "status": "passed", "observed": observed})

    with tempfile.TemporaryDirectory(prefix="codex-export-roundtrip-") as temporary:
        root = Path(temporary)
        http = ThreadingHTTPServer(("127.0.0.1", 0), HistoryResponses)
        http.requests, http.inputs = [], []
        runner = threading.Thread(target=http.serve_forever, daemon=True)
        runner.start()
        source = target = None
        try:
            source = AppServer(binary, root / "source", http.server_port)
            initialize(source)
            project = source.call("project/create", {
                "idempotencyKey": "source-project", "name": "Synthetic source project",
                "roots": [{"path": str(source.workspace)}],
            })["project"]
            thread = source.call("thread/start", {
                "cwd": str(source.workspace), "projectId": project["id"],
            })["thread"]
            source.turn(thread["id"])
            source.call("thread/name/set", {"threadId": thread["id"], "name": "Synthetic export thread"})
            stored = source.call("thread/read", {"threadId": thread["id"], "includeTurns": True})["thread"]
            assert stored["turns"], {"read_turn_count": len(stored["turns"]), "status": stored.get("status")}
            assert "Local audit response." in json.dumps(stored["turns"]), "Read history lacks synthetic assistant response"
            source_path = Path(stored["path"]).resolve(strict=True)
            assert source_path.is_relative_to(source.codex_home.resolve()), "Record is outside isolated source home"
            record("read_stored_history", {"turn_count": len(stored["turns"]), "native_record_available": True})
            source.close()
            source = None

            target = AppServer(binary, root / "target", http.server_port)
            initialize(target)
            imported_path = target.codex_home / "import-staging" / "source-rollout.jsonl"
            imported_path.parent.mkdir()
            shutil.copyfile(source_path, imported_path)
            try:
                forked = target.call("thread/fork", {
                    "threadId": thread["id"], "path": str(imported_path),
                    "cwd": str(target.workspace), "deferGoalContinuation": True,
                })["thread"]
                record("fork_from_staging_path", {"accepted": True})
            except RpcError as error:
                report["observations"].append({
                    "name": "fork_from_staging_path", "accepted": False, "error": error.error,
                })
                native_path = target.codex_home / source_path.relative_to((root / "source" / "home" / ".codex").resolve())
                native_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source_path, native_path)
                target.call("thread/read", {"threadId": thread["id"], "includeTurns": True})
                forked = target.call("thread/fork", {
                    "threadId": thread["id"], "cwd": str(target.workspace), "deferGoalContinuation": True,
                })["thread"]
                record("fork_from_native_session_layout", {"accepted": True, "database_copy": False})
            assert forked["id"] != thread["id"]
            fork_history = target.call("thread/read", {"threadId": forked["id"], "includeTurns": True})["thread"]
            assert len(fork_history["turns"]) == len(stored["turns"])
            assert "Local audit response." in json.dumps(fork_history["turns"])
            record("fork_copied_native_record", {"new_thread_id": True, "preserved_turn_count": len(fork_history["turns"])})

            imported_project = target.call("project/import", {
                "idempotencyKey": "target-project", "name": "Synthetic imported project",
                "roots": [{"path": str(target.workspace)}], "threads": [forked["id"]],
            })["project"]
            rebound = target.call("thread/read", {"threadId": forked["id"]})["thread"]
            assert imported_project["id"] != project["id"]
            assert rebound["projectId"] == imported_project["id"]
            record("import_project_with_new_thread", {"new_project_id": True, "new_thread_bound": True})

            target.turn(forked["id"], model="gpt-5.4", effort="medium")
            payload = json.dumps(http.inputs[-1])
            assert "Local audit response." in payload
            assert payload.count("Reply with a short plain-text acknowledgement.") >= 2
            resumed = target.call("thread/read", {"threadId": forked["id"], "includeTurns": True})["thread"]
            assert len(resumed["turns"]) == len(stored["turns"]) + 1
            record("continue_with_prior_context", {"prior_assistant_sent": True, "turn_count": len(resumed["turns"]), "model": http.requests[-1]["model"]})
            report["status"] = "passed"
        except Exception as error:
            report["status"], report["error"] = "failed", repr(error)
            report["traceback"] = traceback.format_exc()
        finally:
            report["local_requests"] = http.requests
            for server in (source, target):
                if server is not None:
                    server.close()
            http.shutdown()
            http.server_close()
            runner.join(timeout=2)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codex", required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    binary = str(Path(args.codex).resolve(strict=True))
    report = {
        "tested_at_utc": datetime.now(timezone.utc).isoformat(),
        "codex_version": subprocess.check_output([binary, "--version"], text=True).strip(),
        "probe_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "helper_sha256": hashlib.sha256(Path(__file__).with_name("probe_native.py").read_bytes()).hexdigest(),
        "experimental_api": True, "isolated_homes": 2, "provider": "loopback Responses stub",
        "checks": [], "observations": [],
        "limitations": [
            "Same binary on both ends; no cross-version, cross-platform, or real desktop import checked.",
            "Native path fork is UNSTABLE; thread/resume.history is reserved for Codex Cloud and is not used.",
            "No production archive format, policy import, child-tree, attachments, or repository transfer checked.",
        ],
    }
    probe(binary, report)
    output = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(output)
    print(output, end="")
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
