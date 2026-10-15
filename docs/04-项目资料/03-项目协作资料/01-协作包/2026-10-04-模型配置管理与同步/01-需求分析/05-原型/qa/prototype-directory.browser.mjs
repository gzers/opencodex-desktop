// 目录纠错复验：只访问本机静态原型；默认不写报告，--report 只创建新文件。
import {repoRoot, prototypeRoot, prototypeEntry} from '../../../../2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/原型/qa/prototype-location.mjs';
import {createRequire} from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath, pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';

const args = process.argv.slice(2);
let reportPath;
let date = new Date().toISOString().slice(0, 10);
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--report' && args[i + 1]) reportPath = path.resolve(args[++i]);
  else if (args[i] === '--date' && args[i + 1]) date = args[++i];
  else throw Error('用法：node 本文件 [--date YYYY-MM-DD] [--report 新报告路径]');
}
if (reportPath && fs.existsSync(reportPath)) throw Error('已有报告拒绝覆盖');
const npx = path.join(os.homedir(), '.npm/_npx');
const cache = path.join(os.homedir(), 'Library/Caches/ms-playwright');
const pw = [process.env.PLAYWRIGHT_CORE, ...(fs.existsSync(npx) ? fs.readdirSync(npx).map(d => path.join(npx, d, 'node_modules/playwright-core')) : [])]
  .find(p => p && fs.existsSync(path.join(p, 'package.json')));
const exe = [process.env.CHROMIUM_EXECUTABLE, ...(fs.existsSync(cache) ? fs.readdirSync(cache).filter(d => d.startsWith('chromium_headless_shell-')).sort().reverse().map(d => path.join(cache, d, 'chrome-headless-shell-mac-arm64/chrome-headless-shell')) : [])]
  .find(p => p && fs.existsSync(p));
if (!pw || !exe) throw Error('需要既有 playwright-core / Chromium；可通过 PLAYWRIGHT_CORE / CHROMIUM_EXECUTABLE 指定，本脚本不安装依赖。');
const {chromium} = createRequire(pw + '/')('playwright-core');
const browser = await chromium.launch({executablePath: exe});
const context = await browser.newContext({viewport: {width: 1692, height: 916}});
const results = [], errors = [], retiredRequests = [], externalRequests = [];
const retired = path.join(repoRoot, 'docs/04-项目资料/03-项目协作资料/03-共享原型') + path.sep;
await context.route('**/*', route => {
  const url = route.request().url();
  if (/^(file|data|about):/.test(url)) return route.continue();
  externalRequests.push(url);
  return route.abort();
});
context.on('page', page => {
  page.setDefaultTimeout(6000);
  page.on('pageerror', error => errors.push({page: page.url(), type: 'pageerror', message: error.message}));
  page.on('console', message => {
    if (message.type() === 'error') errors.push({page: page.url(), type: 'console', message: message.text()});
  });
  page.on('request', request => {
    if (request.url().startsWith('file:') && fileURLToPath(request.url()).startsWith(retired)) retiredRequests.push(request.url());
  });
  page.on('requestfailed', request => {
    if (request.isNavigationRequest() && request.failure()?.errorText === 'net::ERR_ABORTED') return;
    errors.push({page: page.url(), type: 'resource', url: request.url(), message: request.failure()?.errorText});
  });
});
async function check(name, fn) {
  try {
    await fn();
    results.push({name, status: 'PASS'});
  } catch (error) {
    results.push({name, status: 'FAIL', error: error.message});
  }
  console.log(results.at(-1).status + ' ' + name);
}
const source = path.join(repoRoot, 'docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型');
const entry = pathToFileURL(prototypeEntry).href;
try {
  const page = await context.newPage();
  const go = async (route, selector) => {
    await page.locator('.nav button[data-route="' + route + '"]').click();
    await page.waitForFunction(route => document.documentElement.dataset.route === route, route);
    assert(await page.locator(selector).isVisible());
  };
  await check('产品入口概览及工具板可用，共用 CSS / JS 已加载', async () => {
    await page.goto(entry);
    assert.equal(await page.locator('html').getAttribute('data-route'), 'overview');
    assert(await page.locator('#route-overview').isVisible());
    assert(await page.locator('.prototype-side').isVisible());
    assert(await page.evaluate(() => typeof modelPrototype === 'object' && typeof syncPrototype === 'object' && typeof setWinSize === 'function'));
    assert(await page.locator('link[rel="stylesheet"]').evaluateAll(links => links.length > 0 && links.every(link => !!link.sheet)));
  });
  await check('模型渠道列表及模型模版 Tab 可达', async () => {
    await go('models', '#route-models');
    assert(await page.locator('#mpBudget').isVisible());
    await page.locator('[data-mp="tab"][data-tab="templates"]').click();
    assert.equal(await page.locator('[data-mp="tab"][data-tab="templates"]').getAttribute('aria-selected'), 'true');
    await page.locator('[data-mp="tab"][data-tab="channels"]').click();
    assert.equal(await page.locator('[data-mp="tab"][data-tab="channels"]').getAttribute('aria-selected'), 'true');
  });
  await check('文件同步及 WebDAV Tab 可达，工具板随上下文切换', async () => {
    await go('sync', '#route-sync');
    assert.equal(await page.locator('#sxTab-file').getAttribute('aria-selected'), 'true');
    await page.locator('#sxTab-webdav').click();
    assert.equal(await page.locator('#sxTab-webdav').getAttribute('aria-selected'), 'true');
    await page.waitForFunction(() => document.querySelector('.prototype-side').dataset.context === 'sync:webdav');
    await page.locator('#sxTab-file').click();
    await page.waitForFunction(() => document.querySelector('.prototype-side').dataset.context === 'sync:file');
  });
  await check('设置入口可达', () => go('settings', '#route-settings'));
  await check('托盘入口可达', async () => {
    await page.locator('#protoCommon > summary').click();
    await page.locator('#launchGrid [data-launch="tray"]').click();
    await page.waitForFunction(() => document.documentElement.dataset.route === 'tray');
    assert(await page.locator('.tray-stage').isVisible());
  });
  await check('从概览进入 Windows 专题，模拟最大化与退出可用', async () => {
    await page.locator('#launchGrid [data-launch="overview"]').click();
    await page.waitForFunction(() => document.documentElement.dataset.route === 'overview');
    await page.locator('#launchGrid [data-launch="windows"]').click();
    await page.waitForFunction(() => document.body.classList.contains('windows-mode'));
    await page.waitForFunction(() => document.querySelector('.prototype-side').dataset.context === 'windows');
    assert(await page.locator('#windowsDemoCard').isVisible());
    assert(await page.locator('.windows-caption').isVisible());
    await page.locator('[data-win-action="maximize"]').click();
    assert.equal(await page.locator('[data-win-action="maximize"]').getAttribute('aria-label'), '还原');
    await page.locator('[data-win-action="leave"]').click();
    assert.equal(await page.evaluate(() => document.body.classList.contains('windows-mode')), false);
  });
  for (const query of ['?theme=dark&probe=directory#models', '?topic=windows&probe=directory#models']) {
    await check('来源兼容跳转保留 query / hash ' + query, async () => {
      await page.goto(pathToFileURL(path.join(source, '原型/index.html')).href + query);
      await page.waitForURL(url => fileURLToPath(url) === prototypeEntry);
      const actual = new URL(page.url()), expected = new URL(entry + query);
      assert.equal(actual.search, expected.search);
      assert.equal(actual.hash, expected.hash);
      assert(await page.locator('#mpBudget').isVisible());
      if (expected.searchParams.has('topic')) assert(await page.evaluate(() => document.body.classList.contains('windows-mode')));
    });
  }
  await page.close();
  const candidates = [
    '2026-09-13-前置条件引导/index.html',
    '2026-09-18-通知与反馈专题/index.html',
    '2026-09-28-Logo本体形变/glass.html',
    '2026-09-28-Logo本体形变/index.html',
    '2026-09-28-Logo本体形变/layout-options.html',
    '2026-09-28-Logo本体形变/overview.html',
    '2026-09-28-状态形象动画/index.html',
    '2026-09-28-状态形象动画/logo.html'
  ];
  for (const candidate of candidates) {
    await check('历史候选共用样式可加载 ' + candidate, async () => {
      const preview = await context.newPage();
      try {
        await preview.goto(pathToFileURL(path.join(source, '候选', candidate)).href);
        const sharedStyles = await preview.locator('link[rel="stylesheet"]').evaluateAll(links => links.map(link => ({href: link.href, loaded: !!link.sheet})));
        const fromProduct = sharedStyles.filter(link => fileURLToPath(link.href).startsWith(prototypeRoot + path.sep));
        assert(fromProduct.length > 0);
        assert(fromProduct.every(link => link.loaded));
      } finally {
        await preview.close();
      }
    });
  }
  await check('无脚本 / 资源错误、旧目录请求或外部网络请求', async () => {
    assert.deepEqual(errors, []);
    assert.deepEqual(retiredRequests, []);
    assert.deepEqual(externalRequests, []);
  });
} finally {
  await browser.close();
}
const report = {date, prototypeRoot: path.relative(repoRoot, prototypeRoot), runtime: {playwright: pw, chromium: exe}, boundary: '目录与入口静态浏览器复验；无生产 API / 配置写入；Windows 仅网页模拟，不是原生验收；不覆盖历史证据', results, errors, retiredRequests, externalRequests};
if (reportPath) {
  fs.mkdirSync(path.dirname(reportPath), {recursive: true});
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n', {flag: 'wx'});
}
console.log('TOTAL ' + results.length + ' / FAIL ' + results.filter(result => result.status === 'FAIL').length);
process.exitCode = results.some(result => result.status === 'FAIL') ? 1 : 0;
