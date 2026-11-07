"""Assemble one immutable diagnostic sweep. Missing/not-ready is a failure."""
import argparse
import json
from pathlib import Path


def summarize(root, expected):
    root = Path(root)
    rows = []
    for step, name in expected:
        path = root / f"{step}-{name}.json"
        if path.exists():
            row = json.loads(path.read_text(encoding="utf-8-sig"))
            if row.get("step") != step or row.get("name") != name or row.get("status") not in {"pass", "fail"}:
                raise ValueError(f"Invalid diagnostic record: {path.name}")
            if row.get("accepted") is not False or row.get("diagnosticOnly") is not True:
                raise ValueError(f"A sweep cannot accept a candidate: {path.name}")
        else:
            row = dict(schemaVersion=1, step=step, name=name, status="fail",
                       firstError="No execution receipt: prerequisite, cancellation, or runner interruption; not accepted",
                       accepted=False, diagnosticOnly=True, evidence=None)
            with path.open("x", encoding="utf-8") as f:
                json.dump(row, f, ensure_ascii=False, indent=2)
        rows.append(row)
    return dict(schemaVersion=1, diagnosticOnly=True, accepted=False,
                status="diagnostic-complete", passed=sum(r["status"] == "pass" for r in rows),
                failed=sum(r["status"] == "fail" for r in rows), steps=rows,
                acceptance="No individual success or all-green sweep is release acceptance")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("root", type=Path)
    p.add_argument("--plan", type=Path, required=True)
    a = p.parse_args()
    a.root.mkdir(parents=True, exist_ok=True)
    expected = json.loads(a.plan.read_text())["steps"]
    result = summarize(a.root, [(r["step"], r["name"]) for r in expected])
    # One output per run/attempt, including a complete unsuccessful round.
    with (a.root / "summary.json").open("x", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print(json.dumps({k: result[k] for k in ("accepted", "passed", "failed")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
