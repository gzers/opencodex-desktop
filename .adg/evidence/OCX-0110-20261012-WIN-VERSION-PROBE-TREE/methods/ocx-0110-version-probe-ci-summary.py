"""Recalculate only completed Windows test-log summaries."""
import json
import re
from pathlib import Path

evidence = Path(r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-VERSION-PROBE-TREE")
result = {}
for label in ["probe-red", "probe-fixed", "probe-atomic-red", "probe-atomic-fixed"]:
    path = evidence / (label + "-ci-windows.log")
    if not path.is_file():
        continue
    text = path.read_text(encoding="utf-8-sig")
    rows = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    result[label] = {
        "groups": len(rows), "passed": sum(int(row[1]) for row in rows),
        "failed": sum(int(row[2]) for row in rows), "ignored": sum(int(row[3]) for row in rows),
        "scope": "recorded exact Windows job test summaries only; ignored and unexecuted groups not passed",
    }
(evidence / "windows-test-summary-recalculation.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
