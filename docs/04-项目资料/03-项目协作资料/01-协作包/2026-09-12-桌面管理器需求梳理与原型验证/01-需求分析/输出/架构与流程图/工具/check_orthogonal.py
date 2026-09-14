#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""校验 drawio 连线规则。

规则：
  1. 每条连线必须是**一条直线段**：水平或竖直，没有折角（无控制点）、没有斜线；
  2. 直线段不得穿过任何节点 / 注释框。

用法： python3 check_orthogonal.py        （检查上级目录下所有 .drawio）
退出码 0 表示全部通过。
"""
import glob
import os
import sys
import xml.etree.ElementTree as ET

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)
TOL = 1.0     # 允许的贴合容差（沿节点边界走不算穿过）


def num(v):
    return float(v)


def check(path):
    root = ET.parse(path).getroot()
    bad = []
    for diag in root.iter("diagram"):
        for model in diag.iter("mxGraphModel"):
            cells = list(model.iter("mxCell"))
            geo = {}
            for c in cells:
                g = c.find("mxGeometry")
                if g is not None and c.get("vertex") == "1":
                    geo[c.get("id")] = (num(g.get("x")), num(g.get("y")),
                                        num(g.get("width")), num(g.get("height")))
            for c in cells:
                if c.get("edge") != "1":
                    continue
                eid, name = c.get("id"), diag.get("name")
                g = c.find("mxGeometry")
                pts = [(num(p.get("x")), num(p.get("y"))) for p in g.iter("mxPoint")] if g is not None else []
                if pts or "exitX=" not in (c.get("style") or ""):
                    bad.append(f"{name}/{eid}: 存在折角（控制点 {len(pts)} 个），不是一条直线段")
                    continue
                st = c.get("style")
                ex = num(st.split("exitX=")[1].split(";")[0])
                ey = num(st.split("exitY=")[1].split(";")[0])
                nx = num(st.split("entryX=")[1].split(";")[0])
                ny = num(st.split("entryY=")[1].split(";")[0])
                sx, sy, sw, sh = geo[c.get("source")]
                tx, ty, tw, th = geo[c.get("target")]
                a = (sx + ex * sw, sy + ey * sh)
                b = (tx + nx * tw, ty + ny * th)
                if abs(a[0] - b[0]) > TOL and abs(a[1] - b[1]) > TOL:
                    bad.append(f"{name}/{eid}: 斜线 {a} -> {b}")
                    continue
                for nid, (rx, ry, rw, rh) in geo.items():
                    if nid in ("0", "1", c.get("source"), c.get("target")):
                        continue
                    if abs(a[1] - b[1]) <= TOL:                    # 水平
                        lo, hi = sorted((a[0], b[0]))
                        if ry + TOL < a[1] < ry + rh - TOL and max(lo, rx) < min(hi, rx + rw) - TOL:
                            bad.append(f"{name}/{eid}: 穿过 {nid}")
                    else:                                          # 竖直
                        lo, hi = sorted((a[1], b[1]))
                        if rx + TOL < a[0] < rx + rw - TOL and max(lo, ry) < min(hi, ry + rh) - TOL:
                            bad.append(f"{name}/{eid}: 穿过 {nid}")
    return bad


def main():
    total = 0
    for f in sorted(glob.glob(os.path.join(PKG, "*.drawio"))):
        bad = check(f)
        total += len(bad)
        print(f"{os.path.basename(f)}: {'OK' if not bad else str(len(bad)) + ' 处问题'}")
        for b in bad:
            print("   -", b)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
