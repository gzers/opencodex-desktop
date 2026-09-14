#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成 OpenCodeX-Desktop 协作包的 drawio 架构图与流程图。

用法： python3 build_diagrams.py
输出： 上级目录（06-架构与流程/）下的三个 .drawio 文件。
说明： .drawio 是后续手工编辑与评审的载体；本脚本只负责首次一致生成。
"""
import os
from xml.sax.saxutils import escape

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)

# ---------------------------------------------------------------- 基础样式
BORDER = "html=1;whiteSpace=wrap;rounded=1;arcSize=8;fontSize=%d;%s"
PAL = {
    "front":   "fillColor=#dae8fc;strokeColor=#6c8ebf",
    "backend": "fillColor=#d5e8d4;strokeColor=#82b366",
    "ext":     "fillColor=#ffe6cc;strokeColor=#d79b00",
    "data":    "fillColor=#fff2cc;strokeColor=#d6b656",
    "state":   "fillColor=#e1d5e7;strokeColor=#9673a6",
    "ok":      "fillColor=#d5e8d4;strokeColor=#82b366",
    "danger":  "fillColor=#f8cecc;strokeColor=#b85450",
    "note":    "fillColor=#f5f5f5;strokeColor=#999999;dashed=1",
    "plain":   "fillColor=#ffffff;strokeColor=#666666",
}
CONTAINER = ("html=1;whiteSpace=wrap;rounded=1;arcSize=3;verticalAlign=top;align=left;"
             "fontSize=15;fontStyle=1;spacingLeft=12;spacingTop=8;")

DIM = {
    "start": (260, 46), "end": (280, 46), "proc": (320, 58),
    "dec": (300, 100), "state": (280, 46), "ok": (300, 56),
    "danger": (340, 72), "note": (360, 80),
}


class Page:
    def __init__(self, did, name, w, h):
        self.did, self.name, self.w, self.h = did, name, w, h
        self.cells = []
        self._n = 0

    def _id(self, prefix):
        self._n += 1
        return f"{prefix}{self._n}"

    def box(self, cid, label, x, y, w, h, kind="proc", fs=12, bold=False, valign="middle"):
        style = BORDER % (fs, PAL.get(kind, PAL["plain"]))
        if bold:
            style += ";fontStyle=1"
        if valign != "middle":
            style += f";verticalAlign={valign}"
        label = escape(label).replace("\n", "&lt;br&gt;")
        self.cells.append(
            f'        <mxCell id="{cid}" value="{label}" style="{style}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="{h}" as="geometry"/>\n'
            f'        </mxCell>')

    def container(self, cid, label, x, y, w, h, kind="front"):
        style = CONTAINER + PAL[kind]
        label = escape(label).replace("\n", "&lt;br&gt;")
        self.cells.append(
            f'        <mxCell id="{cid}" value="{label}" style="{style}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="{h}" as="geometry"/>\n'
            f'        </mxCell>')

    def title(self, cid, text, x, y, w, fs=22):
        style = f"html=1;whiteSpace=wrap;align=left;fontSize={fs};fontStyle=1;"
        self.cells.append(
            f'        <mxCell id="{cid}" value="{escape(text)}" style="{style}" vertex="1" parent="1">\n'
            f'          <mxGeometry x="{x}" y="{y}" width="{w}" height="34" as="geometry"/>\n'
            f'        </mxCell>')

    def edge(self, cid, src, tgt, label="", dashed=False, exit_xy=None, entry_xy=None):
        style = "html=1;endArrow=block;endFill=1;rounded=1;fontSize=11;"
        if dashed:
            style += "dashed=1;"
        if exit_xy:
            style += f"exitX={exit_xy[0]};exitY={exit_xy[1]};exitDx=0;exitDy=0;"
        if entry_xy:
            style += f"entryX={entry_xy[0]};entryY={entry_xy[1]};entryDx=0;entryDy=0;"
        label = escape(label)
        self.cells.append(
            f'        <mxCell id="{cid}" value="{label}" style="{style}" edge="1" parent="1" '
            f'source="{src}" target="{tgt}">\n'
            f'          <mxGeometry relative="1" as="geometry"/>\n'
            f'        </mxCell>')

    def xml(self):
        return "\n".join([
            f'  <diagram id="{self.did}" name="{escape(self.name)}">',
            f'    <mxGraphModel dx="1400" dy="1000" grid="1" gridSize="10" guides="1" tooltips="1" '
            f'connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="{self.w}" '
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
    content = ('<mxfile host="app.diagrams.net" type="device">\n'
               f'{body}\n'
               '</mxfile>\n')
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(content)
    print("written:", os.path.relpath(path, PKG))


def vchain(page, specs, cx=420, y0=96, gap=34, prefix="m"):
    """纵向主链；返回 (位置字典, 末尾 y)。specs: (id,label,kind)"""
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


# ================================================================ 图一：整体架构
def arch_page():
    p = Page("arch", "整体架构", 1560, 1120)
    p.title("t", "OpenCodeX-Desktop 整体架构（分层 / 模块 / 外部依赖 / 写入边界）", 60, 16, 1440)

    p.container("L1", "前端界面层（渲染在 Tauri v2 WebView 内）", 60, 70, 1400, 122, "front")
    fronts = ["概览", "面板", "拓展", "日志与诊断", "设置", "托盘与原生菜单"]
    fw, fg = 214, 12
    for i, name in enumerate(fronts):
        p.box(f"f{i}", name, 78 + i * (fw + fg), 110, fw, 66, kind="plain", bold=True)

    p.edge("e_ipc", "L1", "L2", "命令调用（Tauri IPC）· 前端不直接访问文件系统与网络",
           exit_xy=(0.5, 1), entry_xy=(0.5, 0))

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

    p.edge("e_ext", "L2", "L3", "子进程 / 文件 / 网络", exit_xy=(0.5, 1), entry_xy=(0.5, 0))

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

    # 数据根分区
    p.container("L4", "数据根（用户可自定义）", 60, 856, 690, 200, "data")
    p.box("d4", "manager state｜桌面壳设置、数据根元信息、UI 偏好\n"
                "opencodex home｜可选承载 OPENCODEX_HOME\n"
                "backups｜升级 / 导入 / 同步覆盖前备份\n"
                "logs｜桌面壳操作日志与旋转日志\n"
                "exports｜加密配置导出容器\n"
                "cache｜可丢弃缓存\n"
                "sync state｜WebDAV 冲突记录、远端索引与历史摘要",
          78, 896, 654, 142, kind="plain", valign="top")

    # 写入边界
    p.container("L5", "写入边界与架构级约束", 790, 856, 670, 200, "danger")
    p.box("b1", "允许写入\nSkills 目录同步；各客户端 MCP 服务器节点（唯一例外）",
          806, 896, 638, 58, kind="ok", valign="top")
    p.box("b2", "禁止写入\nprovider、路由、模型映射与 OpenCodex 核心运行配置\n"
                "（一律走官方 CLI，本产品不提供写入通道）",
          806, 964, 638, 74, kind="danger", valign="top")

    p.box("src", "单一真相源 + 投影：管理器维护统一事实，客户端配置是投影；\n"
                 "原子写（同目录临时文件 + 原子替换）、备份先行、跨进程文件锁、凭据不出域。\n"
                 "来源：docs/02-项目核心/系统架构.md、集成与安全边界.md、数据与状态.md —— 本图为派生示意，不作为事实源。",
          60, 1070, 1400, 40, kind="note", fs=11, valign="top")
    return p


# ================================================================ 图二：状态模型
def state_page():
    p = Page("state", "状态模型", 1320, 800)
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
            p.box(f"{cid}_{i}", it, x + 14, 110 + i * 46, 292, 38, kind="plain", fs=11)

    p.container("C4", "环境前置门禁（安装发现的前置条件，按 node → npm → ocx 短路）", 60, 668, 1200, 120, "state")
    pre = ["checking 检查中", "missing_node 缺 Node.js", "missing_npm 缺 npm", "missing_ocx 缺 ocx"]
    for i, it in enumerate(pre):
        p.box(f"G{i}", it, 78 + i * 300, 708, 280, 60, kind="plain", fs=11)

    p.box("cx", "可同时成立的组合示例\nrunning ＋ syncing ＋ backing_up\n\n"
                "渲染规则：动作按状态渲染，不渲染无解释的禁用按钮；用状态说明文字替代灰按钮传达原因。\n"
                "阶段编号（P0/P1/P2）不出现在应用界面，界面只用功能名与状态（规划中 / 可用）。\n"
                "字段命名与 UI 映射留 IMP 冻结。",
          60, 806, 1200, 130, kind="note", fs=11, valign="top")
    p.h = 970
    return p


def _b(page, cid, label, y, kind="danger", cx=980, w=None, h=None):
    w = w or DIM[kind][0]
    h = h or DIM[kind][1]
    page.box(cid, label, cx - w // 2, y, w, h, kind=kind)
    return cid


def _note(page, cid, text, x, y, w, h, fs=11):
    page.box(cid, text, x, y, w, h, kind="note", fs=fs, valign="top")


# ---------------------------------------------------------------- 流程 01
def flow_discovery():
    p = Page("f01", "01-首次发现与前置门禁", 1320, 1240)
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
    _b(p, "n1", "missing_node：展示缺失项与最短命令\n有 Homebrew → brew install node\n否则 → Node.js LTS 官方安装渠道", pos["d1"][1], cx=980, w=360, h=96)
    _b(p, "n2", "missing_npm：提示重装 Node.js LTS\n或检查 PATH", pos["d2"][1], cx=980, w=360, h=76)
    _b(p, "n3", "missing_ocx：展示官方 npm 包安装命令\n（示例命令非最终契约）", pos["d3"][1], cx=980, w=360, h=76)
    _b(p, "n4", "not_found：只读说明，不引导 npm 安装", pos["d4"][1], cx=980, w=360, h=60)
    p.edge("b_e1", "a_d1", "n1", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d2", "n2", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "a_d3", "n3", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e4", "a_d4", "n4", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e5", "n1", "a_p", "安装后手动重新检查", dashed=True)
    p.edge("b_e6", "n2", "a_p", "手动重新检查", dashed=True)
    p.edge("b_e7", "n3", "a_p", "手动重新检查", dashed=True)
    p.edge("b_e8", "n4", "a_p2", "手动重新检查", dashed=True)
    _note(p, "nt", "应用只做检测与引导，不执行 brew / npm / ocx 安装，也不自动监听系统变化；\n"
                   "完整安装指引收敛在「设置 → 安装配置」。前一项未通过时不继续检查后一项（node → npm → ocx 短路）。",
          60, y + 6, 1200, 60)
    p.h = int(y) + 110
    return p


# ---------------------------------------------------------------- 流程 02
def flow_run():
    p = Page("f02", "02-正常启停", 1320, 1160)
    p.title("t", "流程 02｜正常启停与运行状态迁移", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "stopped（已发现未运行）", "start"),
        ("p1", "点击启动", "proc"),
        ("p2", "托管官方 ocx start 子进程", "proc"),
        ("st", "starting", "state"),
        ("pd", "pending（端口可达未 ready）", "state"),
        ("d", "ready 通过?", "dec"),
        ("ok", "running（健康 / 端口 / PID / 日志入口）", "ok"),
        ("p3", "停止（需确认）", "proc"),
        ("p4", "stopping", "state"),
        ("e1", "stopped", "end"),
    ], cx=430, prefix="a")
    _b(p, "fail", "starting_failed\n查看错误 / 重试启动", pos["d"][1], cx=980, w=340, h=86)
    _b(p, "rst", "重启（需确认）", pos["ok"][1], cx=980, w=300, h=58, kind="proc")
    _b(p, "act", "打开面板 / 查看日志\n（面板不可用时回退概览）", pos["ok"][1] + 76, cx=980, w=300, h=76, kind="note")
    p.edge("b_e1", "a_d", "fail", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "fail", "a_p1", "重试启动", dashed=True)
    p.edge("b_e3", "a_ok", "rst", "重启", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e4", "rst", "a_st", "restart", dashed=True)
    p.edge("b_e5", "a_ok", "act", "查看", exit_xy=(1, 1), entry_xy=(0, 0), dashed=True)
    _note(p, "nt", "状态源优先使用官方 ocx status --json / health / ready / doctor；停止与重启前均需确认；\n"
                   "动作按状态渲染，不渲染无解释的禁用按钮。",
          60, y + 6, 1200, 58)
    p.h = int(y) + 110
    return p


# ---------------------------------------------------------------- 流程 03
def flow_collect():
    p = Page("f03", "03-状态采集数据流", 1320, 560)
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
    p.box("h5", "渲染约束\n· 动作按状态渲染，不渲染无解释的禁用按钮\n· 界面只用功能名与状态（规划中 / 可用），不出现 P0/P1/P2\n· WebDAV 卡按连接状态渲染动作组合\n· 阶段状态与连接状态可同时成立",
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
    _b(p, "bf", "备份失败 → 阻断，不写入", pos["p2"][1], cx=980, w=320, h=58)
    _b(p, "rb", "rolling_back → failed\n保留备份，可定位 / 可校验 / 可恢复", pos["d"][1], cx=980, w=360, h=76)
    p.edge("b_e1", "a_p2", "bf", "失败", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d", "rb", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    _note(p, "nt", "· 只改目标节点，保留未知字段与注释；解析失败即拒绝，不做「尽力而为」的部分写回\n"
                   "· 不改写同目录的凭据、认证等非目标文件\n"
                   "· 检测到外部改写或客户端间不一致进入待处理，不静默覆盖\n"
                   "· 卸载 Skills、删除 MCP 等删除类操作进入回收区，可恢复\n"
                   "· 扩展配置写入由单一状态锁串行化，避免并发动作互相覆盖",
          60, y + 6, 1200, 118)
    p.h = int(y) + 166
    return p


# ---------------------------------------------------------------- 流程 05
def flow_dataroot():
    p = Page("f05", "05-数据目录切换", 1320, 1260)
    p.title("t", "流程 05｜数据目录切换（引用优先、迁移可选）", 60, 16, 1200)
    pos, y = vchain(p, [
        ("s", "选择目录", "start"),
        ("p1", "检查目录（可写 / 空间 / 是否已有数据）", "proc"),
        ("d1", "检查通过?", "dec"),
        ("p2", "检测已有数据 → 选择「引用」或「迁移」", "proc"),
        ("p3", "运行前备份", "proc"),
        ("p4", "停止代理", "proc"),
        ("p5", "执行迁移", "proc"),
        ("p6", "校验", "proc"),
        ("d2", "校验通过?", "dec"),
        ("p7", "更新配置 → 恢复代理", "proc"),
        ("ok", "成功", "ok"),
    ], cx=430, prefix="a")
    _b(p, "blk", "目录不可写 / 空间不足 / 数据结构异常\n→ 检查阶段阻断，保持原数据目录", pos["d1"][1], cx=980, w=360, h=86)
    _b(p, "rbk", "回滚到原数据目录\n备份可定位 / 可校验 / 可恢复", pos["d2"][1], cx=980, w=360, h=76)
    p.edge("b_e1", "a_d1", "blk", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d2", "rbk", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "blk", "a_s", "保持原数据目录", dashed=True)
    p.edge("b_e4", "rbk", "a_s", "保持原数据目录", dashed=True)
    p.edge("b_e5", "a_p2", "a_p7", "引用（默认，不搬动数据）", dashed=True, exit_xy=(1, 0.5), entry_xy=(1, 0.5))
    _note(p, "nt", "失败分支：目录不可写 / 空间不足 / 迁移中断 / 校验失败（自动回滚并保留原数据目录）。\n"
                   "取消分支：确认阶段取消，保持原数据目录。关联：REQ-04，OQ-02。",
          60, y + 6, 1200, 60)
    p.h = int(y) + 110
    return p


# ---------------------------------------------------------------- 流程 06
def flow_restore():
    p = Page("f06", "06-Restore 引导", 1320, 960)
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
    _b(p, "fail", "失败 / 取消\n展示恢复建议", pos["d"][1], cx=980, w=340, h=76)
    _b(p, "rf", "刷新状态", pos["d"][1] + 96, cx=980, w=300, h=58, kind="proc")
    p.edge("b_e1", "a_d", "fail", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_ok", "rf", "刷新", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "fail", "rf")
    p.edge("b_e4", "rf", "a_s", "回到风险观测", dashed=True)
    _note(p, "nt", "restore 一律走官方路径，桌面壳不自动修改配置、不做私有覆盖逻辑；\n"
                   "过程展示前置备份、执行中与结果状态。关联：REQ-10，OQ-03。",
          60, y + 6, 1200, 58)
    p.h = int(y) + 110
    return p


# ---------------------------------------------------------------- 流程 07
def flow_import():
    p = Page("f07", "07-导入配置", 1320, 1320)
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
    _b(p, "rej", "口令错误 / 容器损坏 / 版本不兼容\n→ 拒绝并说明支持范围，不部分写入", pos["d1"][1], cx=980, w=360, h=86)
    _b(p, "bf", "备份失败 → 阻断", pos["d2"][1], cx=980, w=320, h=58)
    _b(p, "rb", "自动回滚\n回滚失败 → 保留上一版本 + 原因", pos["d3"][1], cx=980, w=340, h=76)
    p.edge("b_e1", "a_d1", "rej", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d2", "bf", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "a_d3", "rb", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e4", "rej", "a_s", "用户取消 / 重新选择", dashed=True)
    _note(p, "nt", "· 认证加密自带完整性校验；解密或校验失败即拒绝，不部分导入\n"
                   "· 只接受白名单字段与合法路径，拒绝未知字段与越界路径\n"
                   "· 导出为全量内容（含敏感内容），必须进入口令保护的加密容器；容器含格式版本与派生参数",
          60, y + 6, 1200, 92)
    p.h = int(y) + 140
    return p


# ---------------------------------------------------------------- 流程 08
def flow_sync():
    p = Page("f08", "08-WebDAV 同步", 1320, 1080)
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
    _b(p, "cf", "conflict 冲突待处理\n不静默覆盖，先入待处理", pos["d"][1], cx=980, w=340, h=76)
    _b(p, "fl", "failed 连接或同步失败\n重试连接", pos["d"][1] + 96, cx=980, w=340, h=76)
    p.edge("b_e1", "a_d", "cf", "是", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "cf", "a_p4", "用户处理后", dashed=True)
    p.edge("b_e3", "a_c2", "fl", "失败", dashed=True, exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e4", "fl", "a_p2", "重试", dashed=True)
    _note(p, "nt", "冷同步；上传前客户端加密；证书校验失败默认拒绝，且不提供「忽略证书」选项。\n"
                   "哈希只用于检测传输损坏与回放，真实性由认证加密覆盖；与官方 ocx connect remote hub 是并列能力，不自建、不替代官方 hub。",
          60, y + 6, 1200, 66)
    p.h = int(y) + 120
    return p


# ---------------------------------------------------------------- 流程 09
def flow_selfupdate():
    p = Page("f09", "09-应用自身更新", 1320, 1200)
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
    _b(p, "sig", "保留上一版本 + 原因", pos["d1"][1], cx=980, w=320, h=58)
    _b(p, "rb", "回滚到上一版本", pos["d2"][1], cx=980, w=320, h=58)
    p.edge("b_e1", "a_d1", "sig", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d2", "rb", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "sig", "a_p1", "重新检查", dashed=True)
    p.edge("b_e4", "rb", "a_p1", "重新检查", dashed=True)
    _note(p, "nt", "· 必须签名校验通过后才安装；失败保留可回滚的上一版本\n"
                   "· 重启应用会重启应用本体，但不得停止由桌面壳托管的 OpenCodex 代理；重启后重新接管并恢复状态展示\n"
                   "· 检查结果以非阻塞通知呈现；外部提示只走通知中心，不打断当前操作；通道（stable / beta）与官方 npm 通道彼此独立\n"
                   "· 关联：REQ-17，OQ-05",
          60, y + 6, 1200, 118)
    p.h = int(y) + 166
    return p


# ---------------------------------------------------------------- 流程 10
def flow_upgrade():
    p = Page("f10", "10-官方升级引导", 1320, 980)
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
    _b(p, "none", "已是最新 → 结束", pos["d"][1], cx=980, w=300, h=58, kind="note")
    _b(p, "blk", "备份失败 → 阻断升级引导", pos["d2"][1], cx=980, w=320, h=58)
    p.edge("b_e1", "a_d", "none", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e2", "a_d2", "blk", "否", exit_xy=(1, 0.5), entry_xy=(0, 0.5))
    p.edge("b_e3", "blk", "a_s", "升级前备份失败则先修复备份", dashed=True)
    _note(p, "nt", "版本对象不得混用：本流程只针对 OpenCodex 本体（例如 v2.50.0，npm @bitkyc08/opencodex）；\n"
                   "套壳应用自身的更新见流程 09，两者按钮与文案必须能区分当前更新对象。",
          60, y + 6, 1200, 60)
    p.h = int(y) + 110
    return p


# ================================================================ main
def main():
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-整体架构.drawio"), [arch_page()])
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-状态模型.drawio"), [state_page()])
    write_mxfile(os.path.join(PKG, "OpenCodex-Desktop-核心流程.drawio"), [
        flow_discovery(), flow_run(), flow_collect(), flow_write(), flow_dataroot(),
        flow_restore(), flow_import(), flow_sync(), flow_selfupdate(), flow_upgrade(),
    ])


if __name__ == "__main__":
    main()
