#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""由 Markdown 源重建实施与工作包报告 HTML 与元数据。

用法：
    python3 build.py

- 源：   实施与工作包报告.md
- 产物： index.html 与 assets/report-metadata.json（均可重建，不要手改）
- 侧栏目录由源文件里的 H2 自动生成，标题与锚点不会和正文脱节

依赖：Python-Markdown。本机系统 Python 暂缺该包时，可用 Codex 桌面自带 Python 运行：`/Users/ezio/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3 build.py`。
"""
import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone, timedelta

import markdown

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "实施与工作包报告.md")
OUT = os.path.join(HERE, "index.html")
META = os.path.join(HERE, "assets", "report-metadata.json")
ASSETS = ["assets/shared-style.css", "assets/side-nav.css", "assets/side-nav.js"]
TZ = timezone(timedelta(hours=8))

# 报告自身的编号与版本，跟 06-版本记录 的版本线对齐
REPORT_ID = "RPT-LOCAL-20260914-02"
REPORT_VERSION = "V0.1 · 开发实施工作包报告"


def split_front_matter(text):
    """极简 front matter：只支持 `key: value` 与 `key: |` 多行块。"""
    if not text.startswith("---\n"):
        return {}, text
    end = text.index("\n---", 4)
    block, body = text[4:end], text[end + 4:].lstrip("\n")
    meta, key = {}, None
    for raw in block.split("\n"):
        if key and (raw.startswith("  ") or raw.strip() == ""):
            meta[key] += raw.strip() + "\n"
            continue
        key = None
        if ":" in raw:
            k, v = raw.split(":", 1)
            k, v = k.strip(), v.strip()
            if v in ("|", ""):
                meta[k] = ""
                key = k
            else:
                meta[k] = v
    return {k: v.rstrip("\n") for k, v in meta.items()}, body


def collect_nav(html):
    """导航来源：带 data-nav 的元素（首屏）与全部 H2（章节）。顺序即正文顺序。"""
    items = []
    for m in re.finditer(r'<div class="hero" id="([^"]+)" data-nav="([^"]+)"', html):
        items.append((m.group(1), m.group(2)))
    for m in re.finditer(r'<h2 id="([^"]+)">(.*?)</h2>', html):
        items.append((m.group(1), re.sub(r"<[^>]+>", "", m.group(2)).strip()))
    out = []
    for i, (hid, label) in enumerate(items):
        if "·" in label:
            no, text = label.split("·", 1)
            out.append({"no": no.strip(), "id": hid, "label": text.strip()})
        else:
            out.append({"no": f"{i + 1:02d}", "id": hid, "label": label})
    return out


SHELL = """<!doctype html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
<meta name="generator" content="build.py（由实施与工作包报告.md 重建）">
<link rel="stylesheet" href="assets/shared-style.css">
<link rel="stylesheet" href="assets/side-nav.css">
</head>
<body>

<nav class="side-nav">
<div class="side-brand">{brand}<small>{brand_sub}</small></div>
<div class="side-group">目录</div>
{nav}
<div class="side-foot">{side_foot}</div>
</nav>

<div class="page-shell">
<div class="page-wrap">

{body}
<div class="footer">
{footer}
</div>

</div>
</div>
<script src="assets/side-nav.js"></script>
</body>
</html>
"""

SIDE_FOOT = ("依据 <code>COL-LOCAL-20260912-01</code>、<code>docs/02-项目核心</code> 与 <code>IMP-OPENCODEX-DESKTOP-01</code> 整理。<br>"
             "派生产物，不作为事实源；由 <code>实施与工作包报告.md</code> 重建。")


def main():
    raw = open(SRC, encoding="utf-8").read()
    fm, body_md = split_front_matter(raw)

    md = markdown.Markdown(extensions=["tables", "attr_list", "fenced_code", "sane_lists", "md_in_html"])
    body_html = md.convert(body_md)
    nav = collect_nav(body_html)

    nav_html = "\n".join(
        f'<a href="#{n["id"]}"><span class="n">{n["no"]}</span>{n["label"]}</a>' for n in nav)

    html = SHELL.format(
        title=fm.get("title", "实施与工作包报告"),
        brand=fm.get("brand", "项目报告"),
        brand_sub=fm.get("brand_sub", ""),
        nav=nav_html,
        side_foot=SIDE_FOOT,
        body=body_html,
        footer=fm.get("footer", ""),
    )
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write(html)

    # 统计（从源文件里数，避免依赖渲染结果）
    counts = {
        "requirements": len(set(re.findall(r"REQ-\d{2}", raw))),
        "acceptance": len(set(re.findall(r"AC-\d{2}", raw))),
        "work_packages": len(set(re.findall(r"WP-\d{2}", raw))),
        "sections": len(nav),
        "entities": len(set(re.findall(r"`(EnvironmentGate|Installation|AgentProcess|RuntimeFacts|DataRoot|BackupRecord|SyncEndpoint|CredentialRef|SyncRun|Conflict|ExtensionConfig|Skill|McpServer|McpServerProjection|ClientTarget|WriteTransaction|UpdateTarget|Notification|IpcEndpoint)`", raw))),
    }
    meta = {
        "format": "ocx-report-v1",
        "report_id": REPORT_ID,
        "version": REPORT_VERSION,
        "title": fm.get("title"),
        "source": os.path.basename(SRC),
        "source_bytes": len(raw.encode("utf-8")),
        "source_sha256": hashlib.sha256(raw.encode("utf-8")).hexdigest(),
        "output": os.path.basename(OUT),
        "output_bytes": len(html.encode("utf-8")),
        "generated_at": datetime.now(TZ).isoformat(timespec="seconds"),
        "generator": "build.py",
        "renderer": "Python-Markdown 3.10.3",
        "assets": ASSETS,
        "sections": nav,
        "counts": counts,
        "note": "index.html 与 assets/report-metadata.json 均可重建；不要手改 index.html，改内容请改 实施与工作包报告.md 后重新运行 build.py。",
    }
    with open(META, "w", encoding="utf-8") as fh:
        json.dump(meta, fh, ensure_ascii=False, indent=2)
        fh.write("\n")

    print(f"已重建 {os.path.basename(OUT)}（{meta['output_bytes']} 字节）")
    print(f"已写出 {os.path.basename(META)}；章节 {counts['sections']} 个"
          f"（REQ {counts['requirements']} / AC {counts['acceptance']} / WP {counts['work_packages']} / 实体 {counts['entities']}）")


if __name__ == "__main__":
    sys.exit(main())
