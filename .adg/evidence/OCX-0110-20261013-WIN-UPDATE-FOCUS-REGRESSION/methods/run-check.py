"""Local mocked UI regressions, not real native progress or cycle evidence."""
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[1]
workspace = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
ui = workspace / "apps/desktop/ui"
node = r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe"
phase = sys.argv[1]
assert phase in {"red", "red-final", "green", "typecheck", "frontend-all", "build"}
arguments = {
    "red": [node, "node_modules/vitest/vitest.mjs", "run", "tests/update-center.test.ts"],
    "red-final": [node, "node_modules/vitest/vitest.mjs", "run", "tests/update-center.test.ts"],
    "green": [node, "node_modules/vitest/vitest.mjs", "run", "tests/update-center.test.ts"],
    "typecheck": [node, "node_modules/vue-tsc/bin/vue-tsc.js", "--noEmit"],
    "frontend-all": [node, "node_modules/vitest/vitest.mjs", "run"],
    "build": [node, "node_modules/vite/bin/vite.js", "build"],
}[phase]
assert not (root / (phase + ".json")).exists()
git = lambda args: subprocess.check_output(["git", *args], cwd=workspace).decode().strip()
before = {"componentWorktreeBlob": git(["hash-object", "apps/desktop/ui/src/features/updates/UpdateCenter.vue"]),
          "testsWorktreeBlob": git(["hash-object", "apps/desktop/ui/tests/update-center.test.ts"]),
          "head": git(["rev-parse", "HEAD"]), "branch": git(["branch", "--show-current"])}
started = datetime.now(timezone.utc).isoformat()
clock = time.perf_counter()
with (root / (phase + ".stdout.txt")).open("xb") as output, (root / (phase + ".stderr.txt")).open("xb") as error:
    result = subprocess.run(arguments, cwd=ui, stdout=output, stderr=error, timeout=300,
                            creationflags=subprocess.CREATE_NO_WINDOW)
value = {"phase": phase, "startedAtUtc": started, "finishedAtUtc": datetime.now(timezone.utc).isoformat(),
         "elapsedSeconds": time.perf_counter() - clock, "argv": arguments, "workingDirectory": str(ui),
         **before, "exitCode": result.returncode, "actualNativeUiAccepted": False,
         "nativeCycleOrPerformanceAccepted": False, "productRestarted": False}
(root / (phase + ".json")).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
print(json.dumps({"phase": phase, "exitCode": result.returncode, "elapsedSeconds": value["elapsedSeconds"]}))
