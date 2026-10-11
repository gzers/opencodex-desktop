/*
 * macOS sandbox-only DYLD observer for update scheduler acceptance.
 * Build: clang -dynamiclib -Wall -Wextra -Werror macos-update-probe.c -o probe.dylib
 * Inject only into the isolated, ad-hoc-signed candidate. Never into daily apps.
 * Emits boundary attempts, not successful requests; child env_clear intentionally
 * removes injection. Positive controls must cover manager networking and npm view.
 * This changes the measured process: do NOT use it for CPU/startup comparisons.
 */
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <spawn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/uio.h>
#include <time.h>
#include <unistd.h>

static int probe_fd = -1;

static void record_attempt(const char *event, const char *kind) {
  if (probe_fd < 0) return;
  int saved_errno = errno;
  struct timespec mono, wall;
  clock_gettime(CLOCK_MONOTONIC, &mono);
  clock_gettime(CLOCK_REALTIME, &wall);
  char line[256];
  int n = snprintf(line, sizeof(line),
      "{\"pid\":%d,\"mono_ns\":%llu,\"wall_ns\":%llu,\"event\":\"%s\",\"kind\":\"%s\"}\n",
      getpid(), (unsigned long long)mono.tv_sec * 1000000000ULL + mono.tv_nsec,
      (unsigned long long)wall.tv_sec * 1000000000ULL + wall.tv_nsec, event, kind);
  if (n > 0 && n < (int)sizeof(line)) {
    ssize_t written;
    do { written = write(probe_fd, line, (size_t)n); } while (written < 0 && errno == EINTR);
  }
  errno = saved_errno;
}

__attribute__((constructor)) static void load_probe(void) {
  const char *sandbox = getenv("OPENCODEX_SANDBOX");
  const char *path = getenv("OPENCODEX_UPDATE_PROBE_LOG");
  if (!sandbox || strcmp(sandbox, "1") || !path || path[0] != '/') return;
  probe_fd = open(path, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC | O_NOFOLLOW, 0600);
  record_attempt("probe_loaded", "sandbox");
}

static int is_inet(const struct sockaddr *address) {
  return address && (address->sa_family == AF_INET || address->sa_family == AF_INET6);
}

/* No path, arguments, host, address, or environment are exported. */
static const char *spawn_kind(const char *path, char *const argv[]) {
  const char *name = path ? strrchr(path, '/') : NULL;
  name = name ? name + 1 : path;
  if (name && (!strcmp(name, "npm") || !strcmp(name, "npm-cli.js"))) {
    for (int i = 1; argv && argv[i] && i < 32; ++i)
      if (!strcmp(argv[i], "view")) return "npm_view";
  }
  return "other";
}

static int probe_connect(int fd, const struct sockaddr *address, socklen_t length) {
  if (is_inet(address)) record_attempt("connect", "inet");
  return connect(fd, address, length);
}

static int probe_connectx(int fd, const sa_endpoints_t *endpoints, sae_associd_t association,
    unsigned int flags, const struct iovec *iov, unsigned int count, size_t *length,
    sae_connid_t *connection) {
  if (endpoints && is_inet(endpoints->sae_dstaddr)) record_attempt("connectx", "inet");
  return connectx(fd, endpoints, association, flags, iov, count, length, connection);
}

static int probe_getaddrinfo(const char *node, const char *service,
    const struct addrinfo *hints, struct addrinfo **result) {
  if (node) record_attempt("getaddrinfo", "lookup");
  return getaddrinfo(node, service, hints, result);
}

static int probe_posix_spawn(pid_t *pid, const char *path,
    const posix_spawn_file_actions_t *actions, const posix_spawnattr_t *attributes,
    char *const argv[], char *const env[]) {
  record_attempt("posix_spawn", spawn_kind(path, argv));
  return posix_spawn(pid, path, actions, attributes, argv, env);
}

static int probe_posix_spawnp(pid_t *pid, const char *path,
    const posix_spawn_file_actions_t *actions, const posix_spawnattr_t *attributes,
    char *const argv[], char *const env[]) {
  record_attempt("posix_spawnp", spawn_kind(path, argv));
  return posix_spawnp(pid, path, actions, attributes, argv, env);
}

static int probe_execve(const char *path, char *const argv[], char *const env[]) {
  record_attempt("execve", spawn_kind(path, argv));
  return execve(path, argv, env);
}

static int probe_execv(const char *path, char *const argv[]) {
  record_attempt("execv", spawn_kind(path, argv));
  return execv(path, argv);
}

static int probe_execvp(const char *path, char *const argv[]) {
  record_attempt("execvp", spawn_kind(path, argv));
  return execvp(path, argv);
}

#define INTERPOSE(replacement, original) \
  __attribute__((used)) static struct { const void *replace; const void *original; } \
  interpose_##original __attribute__((section("__DATA,__interpose"))) = \
      { (const void *)(uintptr_t)&replacement, (const void *)(uintptr_t)&original }

INTERPOSE(probe_connect, connect);
INTERPOSE(probe_connectx, connectx);
INTERPOSE(probe_getaddrinfo, getaddrinfo);
INTERPOSE(probe_posix_spawn, posix_spawn);
INTERPOSE(probe_posix_spawnp, posix_spawnp);
INTERPOSE(probe_execve, execve);
INTERPOSE(probe_execv, execv);
INTERPOSE(probe_execvp, execvp);
