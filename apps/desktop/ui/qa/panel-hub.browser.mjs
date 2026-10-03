// ③ 层契约验证：面板右下角「品牌悬浮球」（Rust 注入脚本 apps/desktop/tauri/assets/panel-hub.js）。
// 为什么单独验证：悬浮球注入到官方子 WebView，本机无法运行 OpenCodex 时官方面板不加载，
// 制品内看不到它；这里把同一份脚本注入空白页，验证 DOM/材质/交互契约（IMP-07）。
import fs from 'fs'; import path from 'path'; import os from 'os';
import { createRequire } from 'module';
const REPO = path.resolve(path.dirname(new URL(import.meta.url).pathname), '../../../..');
const HUB = path.join(REPO, 'apps/desktop/tauri/assets/panel-hub.js');
function rp() {
  const c = [process.env.PLAYWRIGHT_CORE];
  const n = path.join(os.homedir(), '.npm/_npx');
  if (fs.existsSync(n)) for (const d of fs.readdirSync(n)) c.push(path.join(n, d, 'node_modules/playwright-core'));
  for (const x of c) if (x && fs.existsSync(path.join(x, 'package.json'))) return x;
  return null;
}
function cs() {
  const b = path.join(os.homedir(), 'Library/Caches/ms-playwright');
  for (const d of fs.readdirSync(b).filter(x => x.startsWith('chromium_headless_shell-')).sort().reverse()) {
    const p = path.join(b, d, 'chrome-headless-shell-mac-arm64/chrome-headless-shell');
    if (fs.existsSync(p)) return p;
  }
  return null;
}
let fail = 0, total = 0;
const check = (name, ok, detail = '') => { total += 1; if (!ok) { fail += 1; console.log(`  FAIL ${name}${detail ? ' — ' + detail : ''}`); } };
const require = createRequire(rp() + '/');
const { chromium } = require('playwright-core');
const browser = await chromium.launch({ executablePath: cs() });
const page = await browser.newPage({ viewport: { width: 1180, height: 760 } });
await page.setContent('<!doctype html><html><body><h1>official panel mock</h1></body></html>');
await page.addScriptTag({ content: fs.readFileSync(HUB, 'utf8') });
await page.waitForSelector('#ocxd-panel-hub .hub-toggle', { timeout: 5000 });
const info = await page.evaluate(() => {
  const root = document.getElementById('ocxd-panel-hub');
  const t = root.querySelector('.hub-toggle'); const s = getComputedStyle(t);
  const svg = t.querySelector('svg.hub-logo');
  return {
    hasLogo: !!svg, logoW: svg ? svg.getAttribute('width') : null,
    gradId: !!root.querySelector('defs linearGradient#ocxdHubGrad'),
    maskId: !!root.querySelector('defs mask#ocxdHubKnock'),
    border: s.borderTopColor, shadow: s.boxShadow, blur: s.backdropFilter || s.webkitBackdropFilter || 'none',
    cursor: s.cursor, radius: s.borderRadius, w: s.width, h: s.height,
    menuDisplay: getComputedStyle(root.querySelector('.hub-menu')).display,
    open: root.classList.contains('open'),
  };
});
check('悬浮球挂载', true);
check('按钮为品牌 logo（svg.hub-logo 22×22）', info.hasLogo && info.logoW === '22', JSON.stringify(info.logoW));
check('品牌渐变/镂空定义存在', info.gradId && info.maskId);
check('圆形按钮 38×38', info.radius.startsWith('50%') && info.w === '38px' && info.h === '38px', `${info.w}x${info.h} r=${info.radius}`);
check('玻璃边框非透明', info.border !== 'rgba(0, 0, 0, 0)', info.border);
check('分层阴影含顶部内高光', /inset/.test(info.shadow), info.shadow);
check('保留毛玻璃模糊', /blur/.test(info.blur), info.blur);
check('可拖拽光标 grab', info.cursor === 'grab', info.cursor);
check('初始菜单收起', info.menuDisplay === 'none' && info.open === false, `display=${info.menuDisplay}`);
// 悬浮不展开（IMP-07：改为点击展开）
await page.hover('#ocxd-panel-hub .hub-toggle');
await page.waitForTimeout(400);
check('鼠标悬浮不自动展开', (await page.evaluate(() => document.getElementById('ocxd-panel-hub').classList.contains('open'))) === false);
// 点击展开
await page.click('#ocxd-panel-hub .hub-toggle');
await page.waitForTimeout(120);
check('点击展开菜单', await page.evaluate(() => document.getElementById('ocxd-panel-hub').classList.contains('open')));
// 悬浮球只管面板自身操作；跨路由「概览」由常驻左侧图标栏承担，不在此重复（IMP-09）。
check('菜单含缩放/重载/浏览器', (await page.evaluate(() => [...document.querySelectorAll('#ocxd-panel-hub [data-action]')].map(b => b.dataset.action))).join(',') === 'zoom-out,zoom-in,reload,browser');
// 鼠标收起**不把焦点留在悬浮球**：WKWebView 会把程序化 focus 判为 :focus-visible，
// 点完就在按钮外面留一圈焦点环（2026-10-03 用户报告「悬浮按钮点击有一圈黑色边」）。
const toggleFocused = () => page.evaluate(() => document.activeElement === document.querySelector('#ocxd-panel-hub .hub-toggle'));
await page.click('#ocxd-panel-hub .hub-toggle');
await page.waitForTimeout(100);
check('再次点击可收起菜单', (await page.evaluate(() => document.getElementById('ocxd-panel-hub').classList.contains('open'))) === false);
check('鼠标收起不把焦点留在悬浮球（无焦点环）', (await toggleFocused()) === false);
await page.click('#ocxd-panel-hub .hub-toggle');
await page.waitForTimeout(80);
await page.mouse.click(320, 320);
await page.waitForTimeout(80);
check('点击空白收起菜单', (await page.evaluate(() => document.getElementById('ocxd-panel-hub').classList.contains('open'))) === false);
check('点击空白收起不把焦点留在悬浮球', (await toggleFocused()) === false);
// Esc 收起（键盘路径）：应把焦点送回悬浮球，方便继续用键盘操作。
await page.click('#ocxd-panel-hub .hub-toggle');
await page.waitForTimeout(80);
await page.keyboard.press('Escape');
await page.waitForTimeout(80);
check('Esc 收起菜单', (await page.evaluate(() => document.getElementById('ocxd-panel-hub').classList.contains('open'))) === false);
check('Esc 收起后焦点回到悬浮球（键盘可继续）', await toggleFocused());
await browser.close();
console.log(`\n面板悬浮球契约：${total - fail}/${total} 通过，失败 ${fail}`);
process.exit(fail ? 1 : 0);
