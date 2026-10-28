"""只生成静态 SVG；使用既有品牌几何，不接入原型或业务。"""
from pathlib import Path
import copy
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[6]
logo = root/'docs/04-项目资料/04-品牌与图标素材/04-品牌组件/Logo/logo.svg'
ns = {'s':'http://www.w3.org/2000/svg'}
tree = ET.fromstring(logo.read_text())
inner = copy.deepcopy(next(n for n in tree.iter() if n.tag.endswith('svg') and n.get('viewBox')=='0 0 24 24'))
inner.set('x','412'); inner.set('y','166'); inner.set('width','136'); inner.set('height','136')
# 示例为运行中，沿已有运行形象的薄荷青至靛蓝配色；品牌原件不改动。
for stop,color in zip(inner.findall('.//s:linearGradient/s:stop',ns),['#80E9CD','#3BAED1','#5861E8']):
    stop.set('stop-color',color)
logo_svg = ET.tostring(inner,encoding='unicode')
svg = '''<svg xmlns="http://www.w3.org/2000/svg" width="960" height="640" viewBox="0 0 960 640">
<title>OpenCodex 中央节点静态草图</title>
<desc>无包围卡片的中央节点：既有品牌形象、管理虚线、请求连接、运行状态、版本、更新提示和展开操作。版本及更新均为示例。</desc>
<defs>
 <linearGradient id="back" x2="0.65" y2="1"><stop stop-color="#F7F8FB"/><stop offset="1" stop-color="#EEF1F6"/></linearGradient>
 <radialGradient id="mint"><stop stop-color="#98E8D1" stop-opacity=".30"/><stop offset="1" stop-color="#98E8D1" stop-opacity="0"/></radialGradient>
 <radialGradient id="blue"><stop stop-color="#92A9E9" stop-opacity=".28"/><stop offset="1" stop-color="#92A9E9" stop-opacity="0"/></radialGradient>
 <linearGradient id="glass" x2=".2" y2="1"><stop stop-color="#FFFFFF" stop-opacity=".89"/><stop offset="1" stop-color="#FFFFFF" stop-opacity=".50"/></linearGradient>
 <filter id="blur"><feGaussianBlur stdDeviation="20"/></filter>
 <filter id="shadow" x="-30%" y="-70%" width="160%" height="250%"><feDropShadow dx="0" dy="7" stdDeviation="10" flood-color="#3E4964" flood-opacity=".10"/></filter>
 <filter id="logoShadow" x="-50%" y="-50%" width="200%" height="220%"><feDropShadow dx="0" dy="12" stdDeviation="12" flood-color="#687CC5" flood-opacity=".12"/></filter>
</defs>
<rect width="960" height="640" fill="url(#back)"/>
<ellipse cx="422" cy="266" rx="270" ry="230" fill="url(#mint)"/>
<ellipse cx="588" cy="320" rx="265" ry="250" fill="url(#blue)"/>
<g font-family="Hiragino Sans GB, PingFang SC, Helvetica Neue, sans-serif">
 <text x="46" y="43" font-size="14" fill="#343D4D" font-weight="600">OpenCodex · 中央节点</text>
 <text x="46" y="65" font-size="11" fill="#8B92A0">选中态 / 常用操作展开 · 单节点放大示意</text>
 <text x="914" y="44" text-anchor="end" font-size="10" fill="#9299A6">静态草图 · 示例数据</text>
 <text x="480" y="105" text-anchor="middle" font-size="11" fill="#9299A3">Desktop 管理</text>
 <path d="M480 117V159" fill="none" stroke="#AAB1BF" stroke-width="1.1" stroke-dasharray="2 5"/>
 <circle cx="480" cy="160" r="2.3" fill="#A5AFBC"/>
 <path d="M92 246C208 246 271 234 387 234" fill="none" stroke="#7394CC" stroke-width="1.6"/>
 <path d="m382 230 5 4-5 4" fill="none" stroke="#7394CC" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
 <path d="M573 234C689 234 752 246 868 246" fill="none" stroke="#7394CC" stroke-width="1.6"/>
 <path d="m863 242 5 4-5 4" fill="none" stroke="#7394CC" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
 <text x="240" y="215" text-anchor="middle" font-size="11" fill="#7F8EA7">客户端请求</text>
 <text x="721" y="215" text-anchor="middle" font-size="11" fill="#7F8EA7">转发到已选渠道</text>
 <ellipse cx="480" cy="317" rx="49" ry="7" fill="#657BC3" opacity=".10" filter="url(#blur)"/>
 <g filter="url(#logoShadow)">__LOGO__</g>
 <text x="480" y="353" text-anchor="middle" font-size="26" font-family="Helvetica Neue, Hiragino Sans GB, sans-serif" font-weight="600" fill="#263244">OpenCodex</text>
 <circle cx="454" cy="380" r="3" fill="#419D89"/>
 <text x="466" y="385" font-size="13" fill="#5A6877">运行中</text>
 <text x="351" y="425" font-size="13" fill="#788394">v2.84.0</text>
 <circle cx="425" cy="421" r="1.25" fill="#A8AFBA"/>
 <text x="440" y="425" font-size="13" fill="#788394">端口 10100</text>
 <rect x="544" y="408" width="76" height="27" rx="13.5" fill="#E9E7F8" fill-opacity=".83" stroke="#AAA2D9" stroke-opacity=".30"/>
 <text x="582" y="425" font-size="11" fill="#7768AB" text-anchor="middle">有更新 ↗</text>
 <g filter="url(#shadow)">
  <rect x="314" y="463" width="332" height="54" rx="15" fill="url(#glass)" stroke="#FFFFFF" stroke-opacity=".95"/>
  <rect x="314.5" y="463.5" width="331" height="53" rx="14.5" fill="none" stroke="#CCD3E0" stroke-opacity=".60"/>
 </g>
 <rect x="322" y="471" width="126" height="38" rx="9" fill="#303A4C"/>
 <g fill="none" stroke="#F9FAFC" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round">
  <rect x="336" y="484" width="15" height="11" rx="2"/><path d="M340 498h7m-3-3v3"/>
 </g>
 <text x="362" y="495" font-size="12" fill="#FFFFFF" font-weight="500">打开面板</text>
 <g fill="none" stroke="#6D798E" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
  <path d="M476 486a6 6 0 1 0 1 8m-1-12v4h-4"/>
 </g>
 <text x="491" y="495" font-size="12" fill="#536176">重启</text>
 <path d="M554 479v22" stroke="#CFD6E0" stroke-width=".8"/>
 <g fill="#728097"><circle cx="590" cy="490" r="1.6"/><circle cx="597" cy="490" r="1.6"/><circle cx="604" cy="490" r="1.6"/></g>
 <text x="480" y="575" text-anchor="middle" font-size="11" fill="#929BAA">图标作为节点，信息与操作围绕同一对象排列</text>
</g>
</svg>'''.replace('__LOGO__',logo_svg)
Path(__file__).with_name('OpenCodex中央节点.svg').write_text(svg)
print('SVG static node written')
