#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成 OpenCodeX-Desktop 协作包的 drawio 架构图与流程图。

用法： python3 build_diagrams.py
输出： 上级目录（06-架构与流程/）下的三个 .drawio 文件。

连线规则（本脚本强制）：
  1. 每条连线只用**一条直线段**：要么水平、要么竖直，没有折角、没有斜线；
     因此端点节点必须在同一水平线（E→W / W→E）或同一竖直线（S→N / N→S）上。
  2. 直线段不得穿过任何节点 / 注释框；不满足时不画折线，而是调整版面使直连成立。
  3. 只有极少数无法直连的情况才会退回正交折线，并在生成时打印 WARN 提示。
"""
import os
import heapq
from xml.sax.saxutils import escape

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)

BG = "#FBFBFC"           # 画布底色（不用纯白）
INK = "#262626"          # 正文（不用纯黑）
INK_STRONG = "#0D0D0D"   # 标题 / 强调（官方 accent 近黑）
MUTED = "#6E6E6E"        # 次要文字
EDGE_C = "#8A8A8E"       # 连线
FONT = "PingFang SC,Helvetica Neue,Arial,sans-serif"

# kind -> (填充, 描边, 字色, 圆角)
BOX_THEME = {
    "proc":    ("#FAFAFB", "#DEDEE1", INK, 10),
    "dec":     ("#F3F4F7", "#CFD3DA", INK, 10),
    "state":   ("#F1F3F7", "#C7CDD9", INK, 24),
    "plain":   ("#FAFAFB", "#E6E6E8", INK, 8),
    "ok":      ("#E8F4EF", "#ABD7C7", INK, 10),
    "danger":  ("#FBECEC", "#E4B1B0", INK, 10),
    "note":    ("#F6F6F7", "#D9D9DB", MUTED, 10),
    "front":   ("#EDF1FD", "#B7C6EE", INK, 10),
    "backend": ("#E8F4EF", "#ABD7C7", INK, 10),
    "ext":     ("#FBF2E8", "#E8C89E", INK, 10),
    "data":    ("#F1F3F7", "#C7CDD9", INK, 10),
    "chip_front":   ("#F3F6FE", "#DCE4F8", INK, 8),
    "chip_backend": ("#F1F8F5", "#D6EAE1", INK, 8),
    "chip_ext":     ("#FDF6EF", "#F2E2CE", INK, 8),
    "start":   ("#2E2E2E", "#232323", "#F4F4F5", 40),
    "end":     ("#2E2E2E", "#232323", "#F4F4F5", 40),
}
CONTAINER_THEME = {
    "front":   ("#F8FAFE", "#CFDAF4"),
    "backend": ("#F6FBF9", "#C6E2D8"),
    "ext":     ("#FDF9F4", "#F0DBBF"),
    "data":    ("#F9FAFC", "#D7DCE6"),
    "state":   ("#F9FAFC", "#D7DCE6"),
    "danger":  ("#FDF7F7", "#EFD0CF"),
    "ok":      ("#F6FBF9", "#C6E2D8"),
    "plain":   ("#FAFAFB", "#E6E6E8"),
}
EDGE_STYLE = ("edgeStyle=none;html=1;rounded=0;endArrow=block;endFill=1;"
              "strokeColor=" + EDGE_C + ";strokeWidth=1.4;fontSize=11;fontColor=" + MUTED + ";"
              "labelBackgroundColor=" + BG + ";fontFamily=" + FONT + ";")


def _box_style(kind, fs, bold, valign):
    f, st, fc, arc = BOX_THEME.get(kind, BOX_THEME["plain"])
    style = (f"rounded=1;arcSize={arc};whiteSpace=wrap;html=1;shadow=0;"
             f"fillColor={f};strokeColor={st};strokeWidth=1;fontColor={fc};"
             f"fontSize={fs};fontFamily={FONT};")
    if kind == "note":
        style += "dashed=1;align=left;"
    if bold:
        style += "fontStyle=1;"
    if valign != "middle":
        style += f"verticalAlign={valign};"
    return style


def _container_style(kind):
    f, st = CONTAINER_THEME.get(kind, CONTAINER_THEME["plain"])
    return (f"rounded=1;arcSize=3;whiteSpace=wrap;html=1;verticalAlign=top;align=left;"
            f"fontSize=15;fontStyle=1;fontColor={INK_STRONG};fontFamily={FONT};"
            f"fillColor={f};strokeColor={st};strokeWidth=1;shadow=0;"
            f"spacingLeft=14;spacingTop=10;")


DIM = {
    "start": (260, 46), "end": (280, 46), "proc": (320, 58),
    "dec": (300, 100), "state": (280, 46), "ok": (300, 56),
    "danger": (340, 72), "note": (360, 80),
}

INFLATE = 8       # 障碍物外扩，保证直线段与节点留出间距
TURN = 22         # 折线回退时的拐弯代价
STUB = 14         # 折线回退时端点引出的短段
ANCHOR = {"E": (1, 0.5), "W": (0, 0.5), "S": (0.5, 1), "N": (0.5, 0)}
STRAIGHT = [("E", "W"), ("S", "N"), ("W", "E"), ("N", "S")]
WARNINGS = []


# ---------------------------------------------------------------- 几何工具
def _anchor(rect, side, stub=0):
    x, y, w, h = rect
    if side == "E":
        return (x + w, y + h / 2), (x + w + stub, y + h / 2)
    if side == "W":
        return (x, y + h / 2), (x - stub, y + h / 2)
    if side == "S":
        return (x + w / 2, y + h), (x + w / 2, y + h + stub)
    return (x + w / 2, y), (x + w / 2, y - stub)


def _clear(p1, p2, obs):
    (x1, y1), (x2, y2) = p1, p2
    if abs(y1 - y2) < 1e-6:
        lo, hi = sorted((x1, x2))
        for (rx, ry, rw, rh) in obs:
            if ry < y1 < ry + rh and max(lo, rx) < min(hi, rx + rw) - 1e-6:
                return False
        return True
    lo, hi = sorted((y1, y2))
    for (rx, ry, rw, rh) in obs:
        if rx < x1 < rx + rw and max(lo, ry) < min(hi, ry + rh) - 1e-6:
            return False
    return True


# 折线回退用（仅当无法直连时）
def _inside(p, obs):
    x, y = p
    return any(rx < x < rx + rw and ry < y < ry + rh for (rx, ry, rw, rh) in obs)


def _compress(pts):
    out = []
    for p in pts:
        if out and abs(p[0] - out[-1][0]) < 1e-6 and abs(p[1] - out[-1][1]) < 1e-6:
            continue
        out.append(p)
    i = 1
    while i < len(out) - 1:
        a, b, c = out[i - 1], out[i], out[i + 1]
        if (abs(a[0] - b[0]) < 1e-6 and abs(b[0] - c[0]) < 1e-6) or \
           (abs(a[1] - b[1]) < 1e-6 and abs(b[1] - c[1]) < 1e-6):
            out.pop(i)
        else:
            i += 1
    return out


def _turns(pts):
    n = 0
    for i in range(1, len(pts) - 1):
        a, b, c = pts[i - 1], pts[i], pts[i + 1]
        if (b[0] - a[0] != 0) != (c[0] - b[0] != 0):
            n += 1
    return n


def _plen(pts):
    return sum(abs(pts[i + 1][0] - pts[i][0]) + abs(pts[i + 1][1] - pts[i][1])
               for i in range(len(pts) - 1))


def _route(start, end, rects, page_w, page_h, inflate=INFLATE):
    obs = [(x - inflate, y - inflate, w + 2 * inflate, h + 2 * inflate)
           for (x, y, w, h) in rects]
    xs = {start[0], end[0], 20.0, float(page_w - 20)}
    ys = {start[1], end[1], 20.0, float(page_h - 20)}
    for (rx, ry, rw, rh) in obs:
        xs.update((rx, rx + rw))
        ys.update((ry, ry + rh))
    xs, ys = sorted(xs), sorted(ys)
    xi = {v: i for i, v in enumerate(xs)}
    yi = {v: i for i, v in enumerate(ys)}
    nx, ny = len(xs), len(ys)
    free = [[not _inside((xs[i], ys[j]), obs) for j in range(ny)] for i in range(nx)]
    si, sj = xi[start[0]], yi[start[1]]
    ei, ej = xi[end[0]], yi[end[1]]
    if not free[si][sj] or not free[ei][ej]:
        return None
    INF = float("inf")
    dist = {(si, sj, 0): 0.0}
    prev = {}
    pq = [(0.0, si, sj, 0)]
    goal = None
    while pq:
        d, i, j, dr = heapq.heappop(pq)
        if d > dist.get((i, j, dr), INF):
            continue
        if (i, j) == (ei, ej):
            goal = (i, j, dr)
            break
        for di, dj, nd in ((1, 0, 1), (-1, 0, 1), (0, 1, 2), (0, -1, 2)):
            ni, nj = i + di, j + dj
            if not (0 <= ni < nx and 0 <= nj < ny) or not free[ni][nj]:
                continue
            if not _clear((xs[i], ys[j]), (xs[ni], ys[nj]), obs):
                continue
            step = abs(xs[ni] - xs[i]) + abs(ys[nj] - ys[j])
            cost = d + step + (TURN if dr not in (0, nd) else 0)
            key = (ni, nj, nd)
            if cost < dist.get(key, INF):
                dist[key] = cost
                prev[key] = (i, j, dr)
                heapq.heappush(pq, (cost, ni, nj, nd))
    if goal is None:
        return None
    path, cur = [], goal
    while cur is not None:
        path.append((xs[cur[0]], ys[cur[1]]))
        cur = prev.get(cur)
    path.reverse()
    return _compress(path)


class Page:
    def __init__(self, did, name, w, h):
        self.did, self.name, self.w, self.h = did, name, w, h
        self.cells = []
        self.rects = []        # [(cid, x, y, w, h)]
        self.specs = []
        self.node_rect = {}
        self._n = 0

    def _reg(self, cid, x, y, w, h, routable=True):
        self.rects.append((cid, x, y, w, h))
        if routable:
            self.node_rect[cid] = (x, y, w, h)

    def box(self, cid, label, x, y, w, h, kind="proc", fs=12, bold=False,
            valign="middle", routable=True):
        style = _box_style(kind, fs, bold, valign)
        label = escape(label).replace("\n", "&lt;br&gt;")
        self.cells.append(
            f'        <mxCell id="{cid}" value="{label}" style="{style}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="{h}" as="geometry"/>\n'
            f'        </mxCell>')
        self._reg(cid, x, y, w, h, routable)

    def container(self, cid, label, x, y, w, h, kind="front"):
        label = escape(label).replace("\n", "&lt;br&gt;")
        self.cells.append(
            f'        <mxCell id="{cid}" value="{label}" style="{_container_style(kind)}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="{h}" as="geometry"/>\n'
            f'        </mxCell>')
        self._reg(cid, x, y, w, h, True)

    def title(self, cid, text, x, y, w, fs=22):
        style = (f"html=1;whiteSpace=wrap;align=left;fontSize={fs};fontStyle=1;"
                 f"fontColor={INK_STRONG};fontFamily={FONT};")
        self.cells.append(
            f'        <mxCell id="{cid}" value="{escape(text)}" style="{style}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="34" as="geometry"/>\n'
            f'        </mxCell>')
        self._reg(cid, x, y, w, 34, False)

    def edge(self, cid, src, tgt, label="", dashed=False):
        self.specs.append((cid, src, tgt, label, dashed))

    def _emit_edge(self, cid, src, tgt, label, dashed):
        rs, rt = self.node_rect[src], self.node_rect[tgt]
        obs = [(x - INFLATE, y - INFLATE, w + 2 * INFLATE, h + 2 * INFLATE)
               for (rid, x, y, w, h) in self.rects if rid not in (src, tgt)]
        direct = None
        for ss, ts in STRAIGHT:
            a_s, _ = _anchor(rs, ss)
            a_t, _ = _anchor(rt, ts)
            if ss in ("E", "W"):
                if abs(a_s[1] - a_t[1]) > 0.6:
                    continue
            else:
                if abs(a_s[0] - a_t[0]) > 0.6:
                    continue
            if _clear(a_s, a_t, obs):
                direct = (ss, ts, a_s, a_t)
                break

        if direct is not None:
            ss, ts, a_s, a_t = direct
            ex, ey = ANCHOR[ss]
            nx_, ny_ = ANCHOR[ts]
            style = (f"{EDGE_STYLE}"
                     f"exitX={ex};exitY={ey};exitDx=0;exitDy=0;"
                     f"entryX={nx_};entryY={ny_};entryDx=0;entryDy=0;")
            if dashed:
                style += "dashed=1;"
            body = '          <mxGeometry relative="1" as="geometry"/>\n'
        else:
            WARNINGS.append(f"[{self.name}] {cid}: {src}->{tgt} 无法直连，退回正交折线")
            best = None
            for ss, ts in STRAIGHT + [("E", "N"), ("N", "E"), ("W", "S"), ("S", "W"),
                                      ("E", "S"), ("S", "E"), ("W", "N"), ("N", "W"),
                                      ("E", "E"), ("W", "W"), ("S", "S"), ("N", "N")]:
                a_s, s_s = _anchor(rs, ss, STUB)
                a_t, s_t = _anchor(rt, ts, STUB)
                pts = _route(s_s, s_t, [r[1:] for r in self.rects], self.w, self.h)
                if pts is None:
                    continue
                cost = _plen(pts) + TURN * _turns(pts)
                if best is None or cost < best[0]:
                    best = (cost, ss, ts, pts)
            if best is None:
                style = EDGE_STYLE
                body = '          <mxGeometry relative="1" as="geometry"/>\n'
            else:
                _, ss, ts, pts = best
                ex, ey = ANCHOR[ss]
                nx_, ny_ = ANCHOR[ts]
                style = (f"{EDGE_STYLE}"
                         f"exitX={ex};exitY={ey};exitDx=0;exitDy=0;"
                         f"entryX={nx_};entryY={ny_};entryDx=0;entryDy=0;")
                if dashed:
                    style += "dashed=1;"
                pts_xml = "".join(f'            <mxPoint x="{round(px, 1)}" y="{round(py, 1)}"/>\n'
                                  for px, py in pts)
                body = ('          <mxGeometry relative="1" as="geometry">\n'
                        '            <Array as="points">\n' + pts_xml + '            </Array>\n'
                        '          </mxGeometry>\n')
        self.cells.append(
            f'        <mxCell id="{cid}" value="{escape(label)}" style="{style}" edge="1" parent="1" '
            f'source="{src}" target="{tgt}">\n{body}        </mxCell>')

    def xml(self):
        for spec in self.specs:
            self._emit_edge(*spec)
        return "\n".join([
            f'  <diagram id="{self.did}" name="{escape(self.name)}">',
            f'    <mxGraphModel dx="1400" dy="1000" grid="0" gridSize="10" guides="0" tooltips="1" '
            f'connect="1" arrows="1" fold="1" background="{BG}" page="1" pageScale="1" pageWidth="{self.w}" '
            f'pageHeight="{self.h}" math="0" shadow="0">',
            '      <root>',
            '        <mxCell id="0"/>',
            '        <mxCell id="1" parent="0"/>',
            *self.cells,
            '      </root>',
            '    </mxGraphModel>',
            '  </diagram>',
        ])


def write_mxfile(path, pages):
    body = "\n".join(p.xml() for p in pages)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write('<mxfile host="app.diagrams.net" type="device">\n'
                 f'{body}\n</mxfile>\n')
    print("written:", os.path.relpath(path, PKG))


def vchain(page, specs, cx=420, y0=96, gap=46, prefix="m"):
    """主链：所有节点共用同一中心 x，保证相邻节点可用一条竖直直线连接。"""
    pos, y = {}, y0
    for sid, label, kind in specs:
        w, h = DIM[kind]
        x = cx - w // 2
        page.box(f"{prefix}_{sid}", label, x, y, w, h, kind=kind)
        pos[sid] = (x, y, w, h)
        y += h + gap
    for i in range(len(specs) - 1):
        page.edge(f"{prefix}_e{i}", f"{prefix}_{specs[i][0]}", f"{prefix}_{specs[i+1][0]}")
    return pos, y


def _side(page, cid, label, anchor, kind="danger", cx=980, w=None, h=None):
    """右侧结果框：垂直中心与 anchor 节点严格对齐，保证一条水平直线连接。"""
    ax, ay, aw, ah = anchor
    w = w or DIM[kind][0]
    h = h or DIM[kind][1]
    y = ay + (ah - h) / 2
    page.box(cid, label, cx - w // 2, y, w, h, kind=kind)
    return cid


def _note(page, cid, text, x, y, w, h, fs=11):
    page.box(cid, text, x, y, w, h, kind="note", fs=fs, valign="top")


# ================================================================ 图一：整体架构
def arch_page():
    p = Page("arch", "整体架构", 1560, 1160)
    p.title("t", "OpenCodeX-Desktop 整体架构（分层 / 模块 / 外部依赖 / 写入边界）", 60, 16, 1440)

    p.container("L1", "前端界面层（渲染在 Tauri v2 WebView 内）", 60, 70, 1400, 122, "front")
    for i, name in enumerate(["概览", "面板", "拓展", "日志与诊断", "设置", "托盘与原生菜单"]):
        p.box(f"f{i}", name, 78 + i * (214 + 12), 110, 214, 66, kind="plain", bold=True)

    p.edge("e_ipc", "L1", "L2", "命令调用（Tauri IPC）· 前端不直接访问文件系统与网络")

    p.container("L2", "管理器后端（Rust）：全部业务逻辑与外部交互的唯一入口", 60, 262, 1400, 306, "backend")
    mods = [
        ("发现模块", "安装发现 / 路径 / 版本 / 可执行校验 / 前置门禁"),
        ("进程托管模块", "ocx start · stop · restart 子进程与超时 / 退出码"),
        ("状态模块", "官方 status 采集、映射与三维状态维护"),
        ("日志与诊断模块", "日志读取 / 脱敏 / 目录打开 / Doctor 只读摘要"),
        ("面板模块", "官方 Web 面板源码级快照承载与失败回退"),
        ("数据根模块", "数据根选择、检查、引用或迁移、备份"),
        ("迁移模块", "加密容器导出、校验、导入与回滚"),
        ("同步模块", "WebDAV 加密、清单、冲突与覆盖前备份"),
        ("扩展管理模块", "Skills 与 MCP 读取、投影与受控写入"),
        ("更新模块", "官方升级引导 + 应用自更新（两条独立通道）"),
        ("通知模块", "通知产生、存储、聚合与清理"),
        ("CLI / IPC 模块", "命令解析、本机 IPC 委托、机读输出与访问控制"),
        ("托盘与菜单模块", "托盘菜单与 macOS 原生菜单入口"),
    ]
    mw, mg = 263, 12
    for i, (name, sub) in enumerate(mods):
        r, c = divmod(i, 5)
        p.box(f"m{i}", f"{name}\n{sub}", 78 + c * (mw + mg), 300 + r * 84, mw, 72, kind="plain")

    p.edge("e_ext", "L2", "L3", "子进程 / 文件 / 网络")

    p.container("L3", "外部依赖（权威归属各不相同）", 60, 616, 1400, 180, "ext")
    exts = [
        "官方 ocx CLI\n启停 / 状态 / 诊断 / 升级｜权威：官方",
        "官方 Web 面板\n只承载视图，不重绘不注入",
        "各客户端配置文件\nMCP / Skills 落点｜共享节点",
        "WebDAV 远端\n视为不可信｜上传前客户端加密",
        "应用自身更新通道\n本产品自有｜签名校验后安装",
    ]
    for i, name in enumerate(exts):
        p.box(f"x{i}", name, 78 + i * (mw + mg), 656, mw, 118, kind="plain")

    p.container("L4", "数据根（用户可自定义）", 60, 856, 690, 200, "data")
    p.box("d4", "manager state｜桌面壳设置、数据根元信息、UI 偏好\n"
                "opencodex home｜可选承载 OPENCODEX_HOME\n"
                "backups｜升级 / 导入 / 同步覆盖前备份\n"
                "logs｜桌面壳操作日志与旋转日志\n"
                "exports｜加密配置导出容器\n"
                "cache｜可丢弃缓存\n"
                "sync state｜WebDAV 冲突记录、远端索引与历史摘要",
          78, 896, 654, 142, kind="plain", valign="top")

    p.container("L5", "写入边界与架构级约束", 790, 856, 670, 200, "danger")
    p.box("b1", "允许写入\nSkills 目录同步；各客户端 MCP 服务器节点（唯一例外）",
          806, 896, 638, 58, kind="ok", valign="top")
    p.box("b2", "禁止写入\nprovider、路由、模型映射与 OpenCodex 核心运行配置\n"
                "（一律走官方 CLI，本产品不提供写入通道）",
          806, 964, 638, 74, kind="danger", valign="top")

    p.box("src", "单一真相源 + 投影：管理器维护统一事实，客户端配置是投影；\n"
                 "原子写（同目录临时文件 + 原子替换）、备份先行、跨进程文件锁、凭据不出域。\n"
                 "来源：docs/02-项目核心/系统架构.md、集成与安全边界.md、数据与状态.md —— 本图为派生示意，不作为事实源。",
          60, 1076, 1400, 46, kind="note", fs=11, valign="top")
    return p


# ================================================================ 图二：状态模型
def state_page():
    p = Page("state", "状态模型", 1320, 990)
    p.title("t", "三维状态模型（互相独立、可同时成立）", 60, 16, 1200)
    cols = [
        ("C1", "运行状态（代理进程）", "front", 60,
         ["not_found 未发现安装", "loading 正在探测", "stopped 已发现未运行", "starting 正在启动",
          "pending 端口可达未就绪", "running 运行且就绪", "stopping 正在停止",
          "starting_failed 启动失败", "at_risk 官方报告 startup at-risk",
          "external_takeover 外部 provider 接管", "unreachable 进程或端口不可达"]),
        ("C2", "连接状态（外部依赖）", "ext", 500,
         ["unconfigured 未配置端点", "disconnected 已配置未连接", "connecting 正在连接",
          "syncing 正在同步", "synced 已同步", "conflict 冲突待处理", "failed 连接或同步失败"]),
        ("C3", "操作状态（单次动作）", "backend", 940,
         ["idle 空闲", "validating 校验中", "backing_up 备份中", "applying 应用中",
          "rolling_back 回滚中", "cancelled 已取消", "succeeded 成功", "failed 失败"]),
    ]
    for cid, label, kind, x, items in cols:
        p.container(cid, label, x, 70, 320, 60 + len(items) * 46 + 10, kind)
        for i, it in enumerate(items):
            p.box(f"{cid}_{i}", it, x + 14, 110 + i * 46, 292, 38, kind=f"chip_{kind}", fs=11)
    p.container("C4", "环境前置门禁（安装发现的前置条件，按 node → npm → ocx 短路）", 60, 668, 1200, 120, "state")
    for i, it in enumerate(["checking 检查中", "missing_node 缺 Node.js",
                            "missing_npm 缺 npm", "missing_ocx 缺 ocx"]):
        p.box(f"G{i}", it, 78 + i * 300, 708, 280, 60, kind="plain", fs=11)
    p.box("cx", "可同时成立的组合示例\nrunning ＋ syncing ＋ backing_up\n\n"
                "渲染规则：动作按状态渲染，不渲染无解释的禁用按钮；用状态说明文字替代灰按钮传达原因。\n"
                "阶段编号（P0/P1/P2）不出现在应用界面，界面只用功能名与状态（规划中 / 可用）。\n"
                "字段命名与 UI 映射留 IMP 冻结。",
          60, 806, 1200, 130, kind="note", fs=11, valign="top")
    return p


# ---------------------------------------------------------------- 流程 01
def flow_discovery():
    p = Page("f01", "01-首次发现与前置门禁", 1320, 1260)
    p.title("t", "流程 01｜首次发现与环境前置门禁", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "应用进入概览", "start"),
        ("p", "环境前置检查（checking）", "proc"),
        ("d1", "node 可用?", "dec"),
        ("d2", "npm 可用?", "dec"),
        ("d3", "ocx 可用?", "dec"),
        ("p2", "隐藏门禁 → 安装发现", "proc"),
        ("d4", "发现安装?", "dec"),
        ("e1", "stopped：进入运行状态机", "end"),
    ], cx=430, prefix="a")
    _side(p, "n1", "missing_node：展示缺失项与最短命令\n有 Homebrew → brew install node\n否则 → Node.js LTS 官方安装渠道", pos["d1"], w=360, h=96)
    _side(p, "n2", "missing_npm：提示重装 Node.js LTS 或检查 PATH", pos["d2"], w=360, h=76)
    _side(p, "n3", "missing_ocx：展示官方 npm 包安装命令\n（示例命令非最终契约）", pos["d3"], w=360, h=76)
    _side(p, "n4", "not_found：只读说明，不引导 npm 安装", pos["d4"], w=360, h=60)
    p.edge("b_e1", "a_d1", "n1", "否")
    p.edge("b_e2", "a_d2", "n2", "否")
    p.edge("b_e3", "a_d3", "n3", "否")
    p.edge("b_e4", "a_d4", "n4", "否")
    _note(p, "nt", "应用只做检测与引导，不执行 brew / npm / ocx 安装，也不自动监听系统变化；前一项未通过时不继续检查后一项（node → npm → ocx 短路）。\n"
                   "缺失项由用户在「设置 → 安装配置」按指引安装后，手动触发重新检查，重新进入本流程；完整安装指引收敛在该分区。",
          60, y + 10, 1200, 74)
    p.h = int(y) + 130
    return p


# ---------------------------------------------------------------- 流程 02
def flow_run():
    p = Page("f02", "02-正常启停", 1320, 1180)
    p.title("t", "流程 02｜正常启停与运行状态迁移", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "stopped（已发现未运行）", "start"),
        ("p1", "点击启动", "proc"),
        ("p2", "托管官方 ocx start 子进程", "proc"),
        ("st", "starting", "state"),
        ("pd", "pending（端口可达未 ready）", "state"),
        ("d", "ready 通过?", "dec"),
        ("ok", "running（健康 / 端口 / PID / 日志入口）\n可用动作：打开面板 / 查看日志", "ok"),
        ("p3", "停止（需确认）", "proc"),
        ("p4", "stopping", "state"),
        ("e1", "stopped", "end"),
    ], cx=430, prefix="a")
    _side(p, "fail", "starting_failed\n查看错误 / 重试启动", pos["d"], w=340, h=86)
    _side(p, "rst", "重启（需确认）", pos["ok"], kind="proc", w=300, h=58)
    p.edge("b_e1", "a_d", "fail", "否")
    p.edge("b_e2", "a_ok", "rst", "重启")
    _note(p, "nt", "状态源优先使用官方 ocx status --json / health / ready / doctor；停止、重启前均需确认；动作按状态渲染，不渲染无解释的禁用按钮。\n"
                   "分支语义：启动失败可查看错误并「重试启动」（回到 starting）；重启（需确认）后回到 starting；面板不可用时回退概览。",
          60, y + 10, 1200, 74)
    p.h = int(y) + 130
    return p


# ---------------------------------------------------------------- 流程 03
def flow_collect():
    p = Page("f03", "03-状态采集数据流", 1320, 520)
    p.title("t", "流程 03｜状态采集数据流与渲染", 60, 16, 1200)
    hx, hy, hw, hh, gap = 60, 130, 268, 96, 32
    p.box("h0", "托管子进程 / 官方 ocx status\nhealth · ready · doctor", hx, hy, hw, hh, kind="ext")
    p.box("h1", "后端采集与字段映射\n（事件与快照）", hx + (hw + gap), hy, hw, hh, kind="backend")
    p.box("h2", "三维状态\n运行 / 连接 / 操作", hx + 2 * (hw + gap), hy, hw, hh, kind="state")
    p.box("h3", "前端按状态渲染动作与说明\n（状态说明替代灰按钮）", hx + 3 * (hw + gap), hy, hw, hh, kind="front")
    p.edge("he0", "h0", "h1")
    p.edge("he1", "h1", "h2")
    p.edge("he2", "h2", "h3")
    p.box("h4", "持久化\n· manager state：设置与偏好（无明文凭据）\n· logs：只读、脱敏、可轮转\n· backups：必须可定位、可校验、可恢复\n· cache：可随时清空",
          60, 290, 560, 130, kind="data", fs=11, valign="top")
    p.box("h5", "渲染约束\n· 动作按状态渲染，不渲染无解释的禁用按钮\n· 界面只用功能名与状态（规划中 / 可用），不出现 P0/P1/P2\n· WebDAV 卡按连接状态渲染动作组合\n· 运行 / 连接 / 操作三个维度可同时成立",
          660, 290, 600, 130, kind="note", fs=11, valign="top")
    return p


# ---------------------------------------------------------------- 流程 04
def flow_write():
    p = Page("f04", "04-受控写入", 1320, 1120)
    p.title("t", "流程 04｜受控写入（扩展配置：Skills / MCP）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "用户动作（扩展管理 / 迁移 / 同步）", "start"),
        ("p1", "前置校验：目标为普通文件、合法路径、\n冲突比对、跨进程文件锁", "proc"),
        ("p2", "生成可恢复备份", "proc"),
        ("p3", "临时区写入 + 原子替换（保留原权限）", "proc"),
        ("p4", "重新校验", "proc"),
        ("d", "成功?", "dec"),
        ("ok", "succeeded", "ok"),
    ], cx=430, prefix="a")
    _side(p, "bf", "备份失败 → 阻断，不写入", pos["p2"], w=320, h=58)
    _side(p, "rb", "rolling_back → failed\n保留备份，可定位 / 可校验 / 可恢复", pos["d"], w=360, h=76)
    p.edge("b_e1", "a_p2", "bf", "失败")
    p.edge("b_e2", "a_d", "rb", "否")
    _note(p, "nt", "· 只改目标节点，保留未知字段与注释；解析失败即拒绝，不做「尽力而为」的部分写回\n"
                   "· 不改写同目录的凭据、认证等非目标文件；检测到外部改写或客户端间不一致进入待处理，不静默覆盖\n"
                   "· 卸载 Skills、删除 MCP 等删除类操作进入回收区，可恢复；写入由单一状态锁串行化\n"
                   "· 分支语义：备份失败即阻断、不写入；写入失败自动回滚，条件修好后可重新发起动作",
          60, y + 10, 1200, 100)
    p.h = int(y) + 160
    return p


# ---------------------------------------------------------------- 流程 05
def flow_dataroot():
    p = Page("f05", "05-数据目录切换", 1320, 1260)
    p.title("t", "流程 05｜数据目录切换（引用优先、迁移可选）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "选择目录", "start"),
        ("p1", "检查目录（可写 / 空间 / 是否已有数据）", "proc"),
        ("d1", "检查通过?", "dec"),
        ("p2", "检测已有数据 → 选择「引用」或「迁移」\n引用：不搬动数据，直接更新配置并恢复代理", "proc"),
        ("p3", "运行前备份", "proc"),
        ("p4", "停止代理", "proc"),
        ("p5", "执行迁移", "proc"),
        ("p6", "校验", "proc"),
        ("d2", "校验通过?", "dec"),
        ("p7", "更新配置 → 恢复代理", "proc"),
        ("ok", "成功", "ok"),
    ], cx=430, prefix="a")
    _side(p, "blk", "目录不可写 / 空间不足 / 数据结构异常\n→ 检查阶段阻断，保持原数据目录", pos["d1"], w=360, h=86)
    _side(p, "rbk", "回滚到原数据目录\n备份可定位 / 可校验 / 可恢复", pos["d2"], w=360, h=76)
    p.edge("b_e1", "a_d1", "blk", "否")
    p.edge("b_e2", "a_d2", "rbk", "否")
    _note(p, "nt", "· 失败分支：目录不可写 / 空间不足 / 迁移中断 / 校验失败（自动回滚并保留原数据目录，保持原数据目录）\n"
                   "· 取消分支：确认阶段取消，保持原数据目录；引用分支不搬动数据\n"
                   "· 关联：REQ-04，OQ-02",
          60, y + 10, 1200, 84)
    p.h = int(y) + 140
    return p


# ---------------------------------------------------------------- 流程 06
def flow_restore():
    p = Page("f06", "06-Restore 引导", 1320, 1020)
    p.title("t", "流程 06｜Restore 引导（外部接管 / at-risk）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "风险状态：at_risk / external_takeover", "start"),
        ("p1", "查看风险与影响摘要", "proc"),
        ("p2", "生成前置备份", "proc"),
        ("p3", "用户显式确认", "proc"),
        ("p4", "官方 restore 引导\n（桌面壳不做私有覆盖逻辑）", "proc"),
        ("p5", "执行中", "proc"),
        ("d", "结果?", "dec"),
        ("ok", "成功", "ok"),
    ], cx=430, prefix="a")
    _side(p, "fail", "失败 / 取消\n展示恢复建议", pos["d"], w=340, h=76)
    _side(p, "rf", "刷新状态", pos["ok"], kind="proc", w=300, h=58)
    p.edge("b_e1", "a_d", "fail", "否")
    p.edge("b_e2", "a_ok", "rf", "刷新")
    _note(p, "nt", "restore 一律走官方路径，桌面壳不自动修改配置、不做私有覆盖逻辑；过程展示前置备份、执行中与结果状态。\n"
                   "分支语义：成功、失败或取消后均可「刷新状态」，回到风险观测。关联：REQ-10，OQ-03。",
          60, y + 10, 1200, 68)
    p.h = int(y) + 130
    return p


# ---------------------------------------------------------------- 流程 07
def flow_import():
    p = Page("f07", "07-导入配置", 1320, 1340)
    p.title("t", "流程 07｜导入加密配置（含异常分支）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "选择加密容器", "start"),
        ("p1", "口令校验", "proc"),
        ("p2", "格式 / 版本 / 完整性校验", "proc"),
        ("d1", "校验通过?", "dec"),
        ("p3", "自动备份当前配置", "proc"),
        ("d2", "备份成功?", "dec"),
        ("p4", "展示掩码摘要 + 用户确认", "proc"),
        ("p5", "原子写入", "proc"),
        ("d3", "写入成功?", "dec"),
        ("ok", "succeeded", "ok"),
    ], cx=430, prefix="a")
    _side(p, "rej", "口令错误 / 容器损坏 / 版本不兼容\n→ 拒绝并说明支持范围，不部分写入", pos["d1"], w=360, h=86)
    _side(p, "bf", "备份失败 → 阻断", pos["d2"], w=320, h=58)
    _side(p, "rb", "自动回滚\n回滚失败 → 保留上一版本 + 原因", pos["d3"], w=340, h=76)
    p.edge("b_e1", "a_d1", "rej", "否")
    p.edge("b_e2", "a_d2", "bf", "否")
    p.edge("b_e3", "a_d3", "rb", "否")
    _note(p, "nt", "· 认证加密自带完整性校验；解密或校验失败即拒绝，不部分导入；只接受白名单字段与合法路径，拒绝未知字段与越界路径\n"
                   "· 导出为全量内容（含敏感内容），必须进入口令保护的加密容器；容器含格式版本与派生参数\n"
                   "· 分支语义：口令错误 / 容器损坏 / 版本不兼容即拒绝；备份失败即阻断；用户取消或校验失败后重新选择容器",
          60, y + 10, 1200, 100)
    p.h = int(y) + 160
    return p


# ---------------------------------------------------------------- 流程 08
def flow_sync():
    p = Page("f08", "08-WebDAV 同步", 1320, 1120)
    p.title("t", "流程 08｜WebDAV 加密同步与冲突处理", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "未配置", "start"),
        ("p1", "配置端点 / 目录 / 凭据（掩码）", "proc"),
        ("c1", "disconnected 未连接", "state"),
        ("p2", "立即同步 / 测试连接", "proc"),
        ("c2", "connecting → syncing", "state"),
        ("p3", "客户端加密 → 远端清单比对", "proc"),
        ("d", "存在冲突?", "dec"),
        ("p4", "覆盖前生成备份 → 上传", "proc"),
        ("ok", "synced 已同步", "ok"),
    ], cx=430, prefix="a")
    _side(p, "fl", "failed 连接或同步失败\n重试连接", pos["c2"], w=340, h=76)
    _side(p, "cf", "conflict 冲突待处理\n不静默覆盖，先入待处理", pos["d"], w=340, h=76)
    p.edge("b_e1", "a_c2", "fl", "失败")
    p.edge("b_e2", "a_d", "cf", "是")
    _note(p, "nt", "冷同步；上传前客户端加密；证书校验失败默认拒绝，且不提供「忽略证书」选项。哈希只用于检测传输损坏与回放，真实性由认证加密覆盖。\n"
                   "分支语义：连接失败可「重试连接」；冲突先入待处理，用户处理后重新发起同步。与官方 ocx connect remote hub 是并列能力，不自建、不替代官方 hub。",
          60, y + 10, 1200, 80)
    p.h = int(y) + 140
    return p


# ---------------------------------------------------------------- 流程 09
def flow_selfupdate():
    p = Page("f09", "09-应用自身更新", 1320, 1220)
    p.title("t", "流程 09｜应用自身更新（套壳应用自更新）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "检查中", "start"),
        ("p1", "有更新", "proc"),
        ("p2", "下载中", "proc"),
        ("p3", "校验中（签名）", "proc"),
        ("d1", "校验通过?", "dec"),
        ("p4", "安装中", "proc"),
        ("p5", "等待重启", "proc"),
        ("p6", "重启接管中\n（不停止 OpenCodex 代理）", "proc"),
        ("d2", "成功?", "dec"),
        ("ok", "成功", "ok"),
    ], cx=430, prefix="a")
    _side(p, "sig", "保留上一版本 + 原因", pos["d1"], w=320, h=58)
    _side(p, "rb", "回滚到上一版本", pos["d2"], w=320, h=58)
    p.edge("b_e1", "a_d1", "sig", "否")
    p.edge("b_e2", "a_d2", "rb", "否")
    _note(p, "nt", "· 必须签名校验通过后才安装；失败保留可回滚的上一版本\n"
                   "· 重启应用会重启应用本体，但不得停止由桌面壳托管的 OpenCodex 代理；重启后重新接管并恢复状态展示\n"
                   "· 检查结果以非阻塞通知呈现；外部提示只走通知中心，不打断当前操作；通道（stable / beta）与官方 npm 通道彼此独立\n"
                   "· 分支语义：校验失败或回滚后可「重新检查」。关联：REQ-17，OQ-05",
          60, y + 10, 1200, 118)
    p.h = int(y) + 140
    return p


# ---------------------------------------------------------------- 流程 10
def flow_upgrade():
    p = Page("f10", "10-官方升级引导", 1320, 1020)
    p.title("t", "流程 10｜OpenCodex 本体升级引导（不接管官方事务）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "检查版本偏差", "start"),
        ("d", "有可用更新?", "dec"),
        ("p1", "生成升级前备份", "proc"),
        ("d2", "备份成功?", "dec"),
        ("p2", "用户确认", "proc"),
        ("p3", "引导官方 ocx update\n（桌面壳不接管更新事务）", "proc"),
        ("p4", "结果展示：成功 / 失败 / 备份位置 / 恢复建议", "proc"),
        ("ok", "结束", "end"),
    ], cx=430, prefix="a")
    _side(p, "none", "已是最新 → 结束", pos["d"], kind="note", w=300, h=58)
    _side(p, "blk", "备份失败 → 阻断升级引导", pos["d2"], w=320, h=58)
    p.edge("b_e1", "a_d", "none", "否")
    p.edge("b_e2", "a_d2", "blk", "否")
    _note(p, "nt", "版本对象不得混用：本流程只针对 OpenCodex 本体（例如 v2.50.0，npm @bitkyc08/opencodex）；套壳应用自身的更新见流程 09。\n"
                   "分支语义：无可用更新即结束；升级前备份失败则先修复备份，再重新检查。",
          60, y + 10, 1200, 68)
    p.h = int(y) + 130
    return p


# ================================================================ main
def main():
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-整体架构.drawio"), [arch_page()])
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-状态模型.drawio"), [state_page()])
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-核心流程.drawio"), [
        flow_discovery(), flow_run(), flow_collect(), flow_write(), flow_dataroot(),
        flow_restore(), flow_import(), flow_sync(), flow_selfupdate(), flow_upgrade(),
    ])
    if WARNINGS:
        print("\nWARN 无法直连（已退回折线）：")
        for w in WARNINGS:
            print("  -", w)
    else:
        print("\nOK 所有连线均为单条直线段（无折角、无斜线）")


if __name__ == "__main__":
    main()
