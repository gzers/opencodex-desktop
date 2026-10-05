import {sharedRoot,sharedPrototype} from './prototype-location.mjs';
// 双形态概览行为回归。jsdom 不验证排版、像素尺寸或动效画面。
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {fileURLToPath, pathToFileURL} from 'node:url';
const here = path.dirname(fileURLToPath(import.meta.url));
const base = path.resolve(here,'../..');
let repo = here;
while (!fs.existsSync(path.join(repo,'.git'))) {
  const parent = path.dirname(repo); if (parent === repo) throw Error('未找到仓库'); repo = parent;
}
const {JSDOM,VirtualConsole} = createRequire(path.join(repo,'apps/desktop/ui/package.json'))('jsdom');
const integration = fs.readFileSync(path.join(sharedRoot,'overview-dual.js'),'utf8');
const geometry = fs.readFileSync(path.join(base,'候选/2026-09-28-Logo本体形变/logo-geometry.js'),'utf8');
let combinations = 0;
for (const entry of ['原型/index.html','候选/2026-09-28-Logo本体形变/overview.html']) {
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on('jsdomError',error => { if (!error.message.startsWith('Not implemented:')) errors.push(error.message); });
  const entryPath=entry==='原型/index.html'?sharedPrototype:path.join(base,entry);
  let html = fs.readFileSync(entryPath,'utf8');
  html = html.replace('<script src="./logo-geometry.js"></script>','<script>' + geometry + '</script>');
  const dom = new JSDOM(html,{runScripts:'dangerously',pretendToBeVisual:true,url:pathToFileURL(entryPath).href,virtualConsole,beforeParse(window) {
    window.matchMedia = () => ({matches:false,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}});
    for (const key of ['replaceState','pushState']) {
      const original = window.history[key].bind(window.history);
      window.history[key] = (...args) => { try { return original(...args); } catch { /* file URL in jsdom */ } };
    }
    window.addEventListener('error',event => errors.push(event.error?.stack || event.message));
  }});
  const {window} = dom;
  const document = window.document;
  const script = document.createElement('script'); script.src = pathToFileURL(path.join(sharedRoot,'overview-dual.js')).href;
  document.body.append(script);
  Object.defineProperty(document,'currentScript',{configurable:true,value:script});
  window.eval(integration);
  await new Promise(resolve => setTimeout(resolve,1000));
  const $ = id => document.getElementById(id);
  try {
    assert.equal(document.querySelectorAll('.motion-env-row').length,0,'就绪态没有孤儿环境行');
    assert.equal(document.querySelectorAll('.motion-identity').length,1,'只创建一个状态主体');
    const iframe = document.querySelector('.motion-backdrop');
    assert(fs.existsSync(fileURLToPath(new URL(iframe.src))),'Logo 动效引用有效');
    for (const [width,height,mode] of [[1180,760,'standard'],[900,600,'compact']]) {
      window.setWinSize(width,height);
      assert.equal(document.documentElement.dataset.overviewSize,mode,'按应用窗口高度切换布局');
      for (const environment of ['ready','missing_node_brew','missing_node_nobrew','missing_npm','missing_ocx','checking']) {
        window.setEnv(environment);
        const blocked = environment !== 'ready';
        assert.equal($('route-overview').dataset.mode,blocked ? 'setup' : 'ready');
        assert.equal($('envGate').hidden,!blocked);
        assert.equal($('mods').hidden,blocked);
        assert.equal($('card-overview-events').hidden,blocked);
        assert.equal(document.querySelector('.motion-detail-icon').disabled,blocked);
        if (blocked) {
          const values = [...document.querySelectorAll('#envGate .overview-check-value')].map(el => el.textContent);
          assert.equal(values.length,3);
          if (environment.startsWith('missing_node')) assert.deepEqual(values,['未发现','待检查','待检查']);
          if (environment === 'missing_npm') assert.deepEqual(values,['已发现','未发现','待检查']);
          if (environment === 'checking') assert.deepEqual(values,['检查中','待检查','待检查']);
          if (environment === 'missing_ocx') {
            const install = document.querySelector('#envGate [data-prototype-action="runtime-install"]');
            const offline = document.querySelector('#envGate [data-prototype-action="runtime-import"]');
            assert(install); assert(offline);
            install.click();
            assert($('modalTitle').textContent.includes('安装'),'门禁安装入口接入既有安装弹窗');
            window.closeModal();
            offline.click();
            assert($('modalTitle').textContent.includes('安装') || $('modalTitle').textContent.includes('离线'),'离线入口接入既有安装弹窗');
            window.closeModal();
          }
          window.setEnv(environment);
          assert.equal(document.querySelectorAll('#envGate .overview-check-value').length,3,'重复选择同一状态不恢复旧门禁');
          document.querySelector('#envGate [data-overview-env-detail]').click();
          assert.equal($('modalTitle').textContent,'运行环境');
          assert.equal($('modal').classList.contains('motion-runtime-modal'),false,'运行详情布局不影响环境指引弹窗');
          assert.equal(document.querySelectorAll('#modalBody .motion-check-list li').length,3);
          window.closeModal();
        } else {
          document.querySelector('.motion-detail-icon').click();
          assert.equal($('modalTitle').textContent,'运行详情');
          assert.equal(document.querySelectorAll('#modalBody .motion-check-list li').length,3,'运行详情包含环境明细');
          for (const field of ['运行状态','健康','PID / 进程','端口','当前操作','版本','运行时','数据目录','OPENCODEX_HOME']) {
            assert($('modalBody').textContent.includes(field),'紧凑运行详情保留事实：' + field);
          }
          window.closeModal();
        }
        combinations++;
      }
    }
    window.setEnv('ready');
    for (const state of ['not_found','stopped','running','loading','starting','pending','starting_failed','at_risk','external_takeover','unreachable']) {
      document.querySelector(`#scenarios [data-state="${state}"]`).click();
      assert.equal($('mods').hidden,false,'环境就绪时运行枚举不切换到环境门禁');
      if (state === 'at_risk') assert.equal(document.querySelector('.motion-mainline').textContent,'未运行');
    }
    const sourceRoot = document.querySelector('#card-overview-details [data-b="root"]');
    const originalRoot = sourceRoot.textContent;
    const longRoot = '/Volumes/项目 & 资料/含"引号"的目录/' + '长目录/'.repeat(50);
    sourceRoot.textContent = longRoot;
    document.querySelector('.motion-detail-icon').click();
    const shownRoot = [...document.querySelectorAll('.motion-runtime-paths>div')].find(row => row.querySelector('dt').textContent === '数据目录').querySelector('.motion-runtime-value');
    assert.equal(shownRoot.textContent,longRoot,'长路径的完整事实仍保留');
    assert.equal(shownRoot.title,longRoot,'省略显示的路径可以悬停查看全文');
    assert.equal(shownRoot.children.length,0,'路径中的特殊字符不变成 HTML');
    window.closeModal();
    sourceRoot.textContent = originalRoot;
    const ids = [...document.querySelectorAll('[id]')].map(el => el.id);
    assert.equal(new Set(ids).size,ids.length);
    assert.equal(errors.length,0,errors.join('\n'));
    console.log('PASS ' + entry + '：双形态、明细、安装入口、重复切态、运行主线、尺寸投影。');
  } finally { window.close(); }
}
console.log('TOTAL: ' + combinations + ' 环境/尺寸/入口组合；0 JS 错误。排版和视觉需另验收。');
