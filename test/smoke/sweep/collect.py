"""Adapt actual controlled receipts; missing/not-ready is a failed diagnostic."""
import argparse
import json
from pathlib import Path


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def require(value, message):
    if not value:
        raise ValueError(message)


def collect(root, selected=None):
    root = Path(root)
    sweep, evidence = root / "sweep", root / "evidence"
    sweep.mkdir(parents=True, exist_ok=True)
    users = evidence / "two-user"

    def write(step, name, check, dependencies=()):
        if selected is not None and step != selected:
            return
        try:
            row = dict(status="pass", firstError="", evidence=check())
        except Exception as e:
            row = dict(status="fail", firstError=str(e).splitlines()[0], evidence=None)
        row.update(schemaVersion=1, step=step, name=name, accepted=False, diagnosticOnly=True,
                   dependencies=list(dependencies))
        with (sweep / f"{step}-{name}.json").open("x", encoding="utf-8") as f:
            json.dump(row, f, ensure_ascii=False, indent=2)

    def both(stage):
        result = []
        for i in (1, 2):
            p = users / f"user{i}/sweep-{stage}.json"
            require(p.exists(), f"user{i}: no actual {stage} receipt; upstream identity/startup failed")
            r = read(p)
            require(r.get("status") == "pass", f"user{i}: {r.get('firstError') or stage + ' failed'}")
            require(r.get("accepted") is False and r.get("diagnosticOnly") is True, "Not a diagnostic receipt")
            result.append(r)
        return result

    write("12", "private-bundle-supply", lambda: both("private-bundle-supply"), ["11"])
    write("13", "runtime-readiness", lambda: both("runtime-readiness"), ["12"])

    def ports():
        d = read(users / "two-user-native.json")
        if d.get("runtimeProxyPortIsolation") != "verified-runtime-ports":
            for index in (1, 2):
                for path in sorted((users / f"user{index}").glob("runtime-*/runtime-step.json")):
                    failure = read(path)
                    if failure.get("result") == "fail":
                        raise ValueError(f"user{index}: {failure.get('failureCode', 'runtime-step-failed')}; stage={failure.get('stage', 'unknown')}")
            raise ValueError("Actual runtime port protocol not verified; actual readiness/identity or protocol prerequisite remains unproved")
        protocol = d.get("runtimeProtocol") or {}
        require(protocol.get("servicesStopped") is True, "Actual owned services stop receipt is missing")
        return protocol

    write("14", "runtime-ports", ports, ["12", "13"])
    write("15", "two-user-paths", lambda: both("path-snapshot"), ["11"])
    write("16", "same-user-ipc", lambda: both("same-user-ipc"), ["11", "15"])
    write("17", "cross-user-ipc", lambda: {k: both(k) for k in
          ("peer-business-data", "cross-user-ipc", "cross-user-dpapi", "same-sid-single-instance")}, ["11", "15"])

    def query_phases(names):
        d = read(evidence / "native-query/native-query-recovery.json")
        result = []
        for name in names:
            rows = [r for r in d.get("sweep", []) if r["name"] == name]
            require(len(rows) == 1 and rows[0].get("status") == "pass",
                    name + ": " + (rows[0].get("firstError", "failed") if rows else "No real execution receipt"))
            result += rows
        require(d.get("sandboxNpmrcRestored") is True, "Actual sandbox npmrc restoration not proved")
        return result

    write("21", "query-failure", lambda: query_phases(["query-failure-and-reopen"]), ["10"])
    write("22", "update-failure-recovery", lambda: query_phases([
        "update-failure-and-reopen", "query-recovery-and-source-preservation"]), ["10"])

    def native_cleanup():
        result = []
        for probe in ("paths", "install", "query"):
            d = read(evidence / f"native-{probe}/launcher.json")
            require(d.get("cleanup", {}).get("handleHasExited") is True,
                    probe + ": owned native Process handle exit not proved")
            result.append(d["cleanup"])
        return result

    write("23", "native-process-cleanup", native_cleanup, ["08", "09", "10"])

    def cleanup():
        d = read(users / "two-user-native.json")
        rows = d.get("cleanup") or []
        removed = [r for r in rows if r.get("result") == "account-and-profile-removed"]
        require(len(removed) == 2 and len({r["sid"] for r in removed}) == 2,
                "Two exact real account/Profile removals not proved")
        require(not any(r.get("result") in {"retained-unproven", "account-or-profile-retained"} for r in rows),
                "Unproven live generations or retained account/Profile; cleanup failed")
        require(d.get("desktopAclRestored") is True and d.get("evidenceAclRestored") is True and
                d.get("sealedCandidateUnchanged") is True, "Original ACL/sealed candidate restoration failed")
        for kind in ("msi", "nsis"):
            for mode in ("default", "custom"):
                receipt = read(evidence / f"{kind}-{mode}/installation-owner-cleanup.json")
                require(receipt.get("removed") is True, f"{kind}/{mode}: installer owner cleanup not proved")
        return dict(twoUser=rows, method="retained PID/SID/image/creation and exact account/Profile/ACL readback")

    write("18", "account-and-process-cleanup", cleanup, ["04", "05", "06", "07", "11"])

    def uninstalls():
        result = []
        for kind in ("msi", "nsis"):
            for mode in ("default", "custom"):
                d = read(evidence / f"{kind}-{mode}/installer-verification.json")
                require(d.get("result") == "pass", f"{kind}/{mode}: failed installer receipt; uninstall not accepted")
                if mode == "default":
                    rows = d.get("defaultUninstallChecks") or []
                    require(len(rows) == 1 and rows[0].get("result") == "pass" and
                            rows[0].get("exitCode") == 0 and rows[0].get("executableRemoved") is True and
                            rows[0].get("userDataRetained") is True,
                            f"{kind}/{mode}: exact default uninstall/user-data retention receipt incomplete")
                if mode == "custom":
                    rows = d.get("installers") or []
                    require(len(rows) == 1 and rows[0].get("uninstallExitCode") == 0 and
                            rows[0].get("uninstallRemovedExecutable") is True and
                            rows[0].get("defaultUninstallRetainedData") is True,
                            f"{kind}/{mode}: exact uninstall/data retention receipt incomplete")
                result.append(dict(kind=kind, mode=mode, receipt=d))
        return result

    write("19", "uninstall-and-retention", uninstalls, ["04", "05", "06", "07"])


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("root", type=Path)
    p.add_argument("--step")
    a = p.parse_args()
    collect(a.root, a.step)
    if a.step:
        rows = list((a.root / "sweep").glob(a.step + "-*.json"))
        if len(rows) != 1 or read(rows[0])["status"] != "pass":
            raise SystemExit(1)
