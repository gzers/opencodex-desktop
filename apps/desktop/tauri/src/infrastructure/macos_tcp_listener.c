/* Use the target Darwin SDK layout, not hand-maintained Rust offsets. */
#include <libproc.h>
#include <sys/proc_info.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <errno.h>
#include <stdint.h>
#include <string.h>

struct ocxd_tcp_row { uint64_t object; uint32_t family; uint32_t port; };
_Static_assert(sizeof(struct ocxd_tcp_row) == 16, "unexpected listener wire layout");

int ocxd_macos_tcp_listener(int pid, int fd, struct ocxd_tcp_row *out) {
    if (pid <= 0 || fd < 0 || out == NULL) return -EINVAL;
    struct socket_fdinfo info;
    memset(&info, 0, sizeof(info));
    memset(out, 0, sizeof(*out));
    errno = 0;
    int result = proc_pidfdinfo(pid, fd, PROC_PIDFDSOCKETINFO, &info, sizeof(info));
    if (result != (int)sizeof(info)) return -(errno != 0 ? errno : EPROTO);
    if (info.psi.soi_protocol != IPPROTO_TCP ||
        (info.psi.soi_family != AF_INET && info.psi.soi_family != AF_INET6)) return 0;
    if (info.psi.soi_kind != SOCKINFO_TCP || info.psi.soi_type != SOCK_STREAM) return -EPROTO;
    if (info.psi.soi_proto.pri_tcp.tcpsi_state != TSI_S_LISTEN) return 0;
    int raw_port = info.psi.soi_proto.pri_tcp.tcpsi_ini.insi_lport;
    if (raw_port < 0 || raw_port > UINT16_MAX || info.psi.soi_so == 0) return -EPROTO;
    out->object = info.psi.soi_so;
    out->family = info.psi.soi_family;
    out->port = ntohs((uint16_t)raw_port);
    if (out->port == 0) return -EPROTO;
    return 1;
}
