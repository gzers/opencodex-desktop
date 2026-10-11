#!/usr/bin/env python3
"""Build and self-check the sandbox observer; no app or external network is used."""
import collections
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

CONTROL = r'''
#include <arpa/inet.h>
#include <netdb.h>
#include <spawn.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>
extern char **environ;
int main(int argc, char **argv) {
  if (argc != 2) return 2;
  struct addrinfo *result = NULL;
  if (getaddrinfo("localhost", "80", NULL, &result)) return 3;
  freeaddrinfo(result);
  struct sockaddr_in address = {.sin_len = sizeof(address), .sin_family = AF_INET,
      .sin_port = htons(1), .sin_addr.s_addr = htonl(INADDR_LOOPBACK)};
  int fd = socket(AF_INET, SOCK_STREAM, 0);
  connect(fd, (struct sockaddr *)&address, sizeof(address));
  close(fd);
  fd = socket(AF_INET, SOCK_STREAM, 0);
  sa_endpoints_t endpoints = {.sae_dstaddr = (struct sockaddr *)&address,
      .sae_dstaddrlen = sizeof(address)};
  connectx(fd, &endpoints, SAE_ASSOCID_ANY, 0, NULL, 0, NULL, NULL);
  close(fd);
  char *args[] = {argv[1], "view", "private-sentinel", NULL};
  pid_t child;
  if (posix_spawn(&child, argv[1], NULL, NULL, args, environ)) return 4;
  waitpid(child, NULL, 0);
  if (posix_spawnp(&child, argv[1], NULL, NULL, args, environ)) return 5;
  waitpid(child, NULL, 0);
  for (int i = 0; i < 3; i++) {
    child = fork();
    if (!child) {
      if (i == 0) execv(argv[1], args);
      if (i == 1) execvp(argv[1], args);
      if (i == 2) execve(argv[1], args, environ);
      _exit(6);
    }
    int status;
    if (child < 0 || waitpid(child, &status, 0) < 0 || status != 0) return 7;
  }
  return 0;
}
'''


def main():
    if sys.platform != "darwin":
        raise SystemExit("This observer requires macOS / dyld.")
    with tempfile.TemporaryDirectory(prefix="ocx-update-probe-check-") as directory:
        root = Path(directory)
        library = root / "probe.dylib"
        source = Path(__file__).with_name("macos-update-probe.c")
        subprocess.run(["clang", "-dynamiclib", "-Wall", "-Wextra", "-Werror",
                        str(source), "-o", str(library)], check=True)
        control_source = root / "control.c"
        control_source.write_text(CONTROL)
        control = root / "control"
        subprocess.run(["clang", "-Wall", "-Wextra", "-Werror",
                        str(control_source), "-o", str(control)], check=True)
        fixture = root / "npm"
        fixture.write_text("#!/bin/sh\nexit 0\n")
        fixture.chmod(0o700)
        log = root / "attempts.jsonl"
        env = dict(os.environ, DYLD_INSERT_LIBRARIES=str(library),
                   OPENCODEX_UPDATE_PROBE_LOG=str(log), OPENCODEX_SANDBOX="1")
        subprocess.run([str(control), str(fixture)], env=env, check=True)
        rows = [json.loads(line) for line in log.read_text().splitlines()]
        counts = collections.Counter((r["event"], r["kind"]) for r in rows)
        expected = [("probe_loaded", "sandbox"), ("connect", "inet"),
                    ("connectx", "inet"), ("getaddrinfo", "lookup")]
        expected += [(event, "npm_view") for event in
                     ("posix_spawn", "posix_spawnp", "execv", "execvp", "execve")]
        for key in expected:
            assert counts[key] > 0, (key, counts)
        assert log.stat().st_mode & 0o777 == 0o600
        assert all(set(row) == {"pid", "mono_ns", "wall_ns", "event", "kind"} for row in rows)
        assert "private-sentinel" not in log.read_text()
        assert str(root) not in log.read_text()
        guarded = root / "guarded.jsonl"
        env.update(OPENCODEX_UPDATE_PROBE_LOG=str(guarded), OPENCODEX_SANDBOX="0")
        subprocess.run([str(control), str(fixture)], env=env, check=True)
        assert not guarded.exists(), "Non-sandbox injection must not log"
        target = root / "untouched"
        target.write_text("sentinel\n")
        guarded.symlink_to(target)
        env.update(OPENCODEX_SANDBOX="1")
        subprocess.run([str(control), str(fixture)], env=env, check=True)
        assert target.read_text() == "sentinel\n", "Refuse a symlink log"
        print(json.dumps({"result": "passed", "observed":
                          {f"{event}:{kind}": counts[(event, kind)] for event, kind in expected},
                          "sandbox_guard": True, "symlink_guard": True,
                          "redacted_schema": True}, indent=2))


if __name__ == "__main__":
    main()
