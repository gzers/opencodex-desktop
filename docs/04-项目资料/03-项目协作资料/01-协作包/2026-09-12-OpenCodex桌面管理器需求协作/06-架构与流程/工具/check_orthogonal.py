#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""校验 drawio 连线规则：只允许横竖段，且不穿过任何节点 / 注释框。

用法： python3 check_orthogonal.py        （检查上级目录下所有 .drawio）
退出码 0 表示全部通过。
"""
import glob
import os
import sys
import xml.etree.ElementTree as ET

HERE = os.path.dirname(os.path.abspath(__file__))
PKG = os.path.dirname(HERE)


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
                g = c.find("mxGeometry")
                pts = [(num(p.get("x")), num(p.get("y"))) for p in g.iter("mxPoint")] if g is not None else []
                if not pts:
                    bad.append(f"{diag.get('name')}/{c.get('id')}: 无控制点，无法保证正交")
                    continue
                st = c.get("style") or ""

                def anchor(nid, ex, ey):
                    x, y, w, h = geo[nid]
                    return (x + ex * w, y + ey * h)

                full = list(pts)
                if "exitX=" in st and "entryX=" in st:
                    ex = num(st.split("exitX=")[1].split(";")[0])
                    ey = num(st.split("exitY=")[1].split(";")[0])
                    nx = num(st.split("entryX=")[1].split(";")[0])
                    ny = num(st.split("entryY=")[1].split(";")[0])
                    full = [anchor(c.get("source"), ex, ey)] + pts + [anchor(c.get("target"), nx, ny)]
                for i in range(len(full) - 1):
                    (x1, y1), (x2, y2) = full[i], full[i + 1]
                    if abs(x1 - x2) > 1e-6 and abs(y1 - y2) > 1e-6:
                        bad.append(f"{diag.get('name')}/{c.get('id')}: 斜线段 {full[i]} -> {full[i+1]}")
                for nid, (rx, ry, rw, rh) in geo.items():
                    if nid in ("0", "1"):
                        continue
                    for i in range(len(full) - 1):
                        (x1, y1), (x2, y2) = full[i], full[i + 1]
                        if abs(y1 - y2) < 1e-6:
                            lo, hi = sorted((x1, x2))
                            if ry + 1 < y1 < ry + rh - 1 and max(lo, rx) < min(hi, rx + rw) - 1:
                                bad.append(f"{diag.get('name')}/{c.get('id')}: 穿过 {nid}")
                        else:
                            lo, hi = sorted((y1, y2))
                            if rx + 1 < x1 < rx + rw - 1 and max(lo, ry) < min(hi, ry + rh) - 1:
                                bad.append(f"{diag.get('name')}/{c.get('id')}: 穿过 {nid}")
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
