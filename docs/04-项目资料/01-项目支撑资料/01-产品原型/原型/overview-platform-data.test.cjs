'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const data = require('./overview-platform-data.js');

// 独立的当前 CH 样例，不执行任何 UI 脚本，也不依赖 DOM、网络或存储。
const CH = [
  { k: 'krill', n: 'Krill', req: 498, fail: 3, tok: .58, tps: 58, via: 1, adapterMs: 18, ttft: .71,
    models: [{ n: 'gpt-6.1-sol', req: 320, tok: .37, tps: 63 }, { n: 'gpt-6.1-mini', req: 178, tok: .21, tps: 51 }] },
  { k: 'newapi', n: 'NewApi', req: 1069, fail: 9, tok: 1.18, tps: 74, ttft: .64,
    models: [{ n: 'gpt-6.1-sol', req: 700, tok: .78, tps: 78 }, { n: 'claude-sonnet-4.5', req: 369, tok: .40, tps: 68 }] },
  { k: 'a6', n: 'A6api', req: 617, fail: 19, tok: .79, ttft: 1.08,
    models: [{ n: 'claude-sonnet-4.5', req: 496, tok: .66, tps: 38 }, { n: 'gpt-6.1-sol', req: 121, tok: .13, tps: 47 }] },
  { k: 'or', n: 'OpenRouter', req: 214, fail: 2, tok: .31, ttft: .88,
    models: [{ n: 'deepseek-v3.2', req: 132, tok: .19, tps: 55 }, { n: 'qwen3-max', req: 82, tok: .12, tps: 47 }] },
  { k: 'ds', n: 'DeepSeek', req: 156, fail: 1, tok: .21, ttft: .93,
    models: [{ n: 'deepseek-v3.2', req: 156, tok: .21, tps: 66 }] },
  { k: 'sf', n: 'SiliconFlow', req: 58, fail: 5, tok: .07, ttft: 1.3,
    models: [{ n: 'qwen3-max', req: 58, tok: .07, tps: 88 }] },
  { k: 'ol', n: 'Ollama', req: 19, fail: 0, tok: .05, ttft: .4,
    models: [{ n: 'llama4-scout', req: 19, tok: .05, tps: 120 }] }
];
const sum = (rows, key) => rows.reduce((total, row) => total + row[key], 0);
const close = (actual, expected, message = '') => assert.ok(
  Math.abs(actual - expected) <= 1e-12 * Math.max(Math.abs(expected), Number.MIN_VALUE),
  `${message}: ${actual} ≈ ${expected}`
);
const branches = platforms => platforms.flatMap(platform => platform.models.flatMap(model =>
  model.channels.map(channel => ({ platform: platform.k, model: model.n, ...channel }))
));
const model = (platforms, platform, name) => platforms.find(row => row.k === platform).models.find(row => row.n === name);
const scaled = factor => CH.map(channel => ({ ...channel,
  req: factor ? Math.max(1, Math.round(channel.req * factor)) : 0,
  tok: +(channel.tok * factor).toFixed(3),
  models: channel.models.map(row => ({ ...row,
    req: factor ? Math.max(1, Math.round(row.req * factor)) : 0,
    tok: +(row.tok * factor).toFixed(3)
  }))
}));

function conserve(channels, platforms) {
  const inputs = channels.flatMap(channel => channel.models);
  const output = branches(platforms);
  assert.equal(sum(platforms, 'req'), sum(inputs, 'req'));
  close(sum(platforms, 'tok'), sum(inputs, 'tok'), '四平台 Token 合计');
  for (const platform of platforms) {
    assert.equal(platform.req, sum(platform.models, 'req'));
    close(platform.tok, sum(platform.models, 'tok'), `${platform.k} Token`);
    assert.ok(!Object.hasOwn(platform, 'tps'));
    for (const row of platform.models) {
      assert.equal(row.req, sum(row.channels, 'req'));
      close(row.tok, sum(row.channels, 'tok'), `${platform.k}/${row.n} Token`);
      assert.ok(!Object.hasOwn(row, 'tps'));
    }
  }
  for (const channel of channels) for (const row of channel.models) {
    const shares = output.filter(branch => branch.k === channel.k && branch.model === row.n);
    assert.equal(sum(shares, 'req'), row.req, `${channel.k}/${row.n} req`);
    close(sum(shares, 'tok'), row.tok, `${channel.k}/${row.n} tok`);
  }
  for (const branch of output) {
    assert.ok(Number.isSafeInteger(branch.req) && branch.req >= 0);
    assert.ok(Number.isFinite(branch.tok) && branch.tok >= 0);
  }
}

test('CommonJS 和无 DOM 的浏览器全局暴露相同的两项 API', () => {
  assert.deepEqual(Object.keys(data).sort(), ['build', 'owner']);
  const browser = {};
  const source = fs.readFileSync(require.resolve('./overview-platform-data.js'), 'utf8');
  vm.runInNewContext(source, browser, { filename: 'overview-platform-data.js' });
  assert.equal(typeof browser.OverviewPlatformData.build, 'function');
  assert.equal(typeof browser.OverviewPlatformData.owner, 'function');
  assert.deepEqual(JSON.parse(JSON.stringify(browser.OverviewPlatformData.build(CH))), data.build(CH));
  assert.equal(browser.OverviewPlatformData.owner('krill', 'gpt-6.1-sol', 1), 'opencode');
  assert.deepEqual(Object.keys(browser), ['OverviewPlatformData']);
});

test('基准配额与平台身份固定，平台数组直接返回', () => {
  const platforms = data.build(CH);
  assert.ok(Array.isArray(platforms));
  assert.deepEqual(platforms.map(({ k, n, c, req }) => ({ k, n, c, req })), [
    { k: 'app', n: 'Codex App', c: 'codex', req: 819 },
    { k: 'opencode', n: 'OpenCode', c: 'opencode', req: 1116 },
    { k: 'cli', n: 'Codex CLI', c: 'cli', req: 200 },
    { k: 'claude', n: 'Claude Code', c: 'claude', req: 496 }
  ]);
  const output = branches(platforms);
  for (const [channel, name, expected] of [
    ['krill', 'gpt-6.1-sol', [200, 80, 40, 0]],
    ['krill', 'gpt-6.1-mini', [118, 40, 20, 0]],
    ['newapi', 'gpt-6.1-sol', [420, 180, 100, 0]],
    ['newapi', 'claude-sonnet-4.5', [0, 369, 0, 0]],
    ['a6', 'gpt-6.1-sol', [81, 0, 40, 0]],
    ['a6', 'claude-sonnet-4.5', [0, 0, 0, 496]],
    ['or', 'deepseek-v3.2', [0, 132, 0, 0]],
    ['or', 'qwen3-max', [0, 82, 0, 0]],
    ['ds', 'deepseek-v3.2', [0, 156, 0, 0]],
    ['sf', 'qwen3-max', [0, 58, 0, 0]],
    ['ol', 'llama4-scout', [0, 19, 0, 0]]
  ]) {
    assert.deepEqual(platforms.map(platform => sum(output.filter(branch =>
      branch.platform === platform.k && branch.k === channel && branch.model === name), 'req')), expected);
  }
  conserve(CH, platforms);
});

for (const factor of [0, 1 / 24, .1, 1, 1.2345, 30, 365]) {
  test(`窗口尺度 ${factor}：每个渠道×模型及各级汇总的整数请求与 Token 守恒`, () => {
    const channels = scaled(factor);
    const platforms = data.build(channels);
    conserve(channels, platforms);
    assert.deepEqual(data.build(channels), platforms);
  });
}

test('最大余数法使用固定顺序破同余，不各自四舍五入，也不强行每平台至少一个请求', () => {
  const build = req => data.build([{ k: 'krill', n: 'Krill', models: [{ n: 'gpt-6.1-sol', req, tok: .37 }] }]);
  // 4 × (200,80,40)/320 = (2.5,1,.5)，相同的 .5 余数优先 app。
  assert.deepEqual(build(4).map(row => row.req), [3, 1, 0, 0]);
  assert.deepEqual(build(1).map(row => row.req), [1, 0, 0, 0]);
  const tiny = branches(build(1));
  close(tiny.find(row => row.platform === 'app').tok, .23125);
  close(tiny.find(row => row.platform === 'opencode').tok, .0925);
  close(tiny.find(row => row.platform === 'cli').tok, .04625);
  close(sum(tiny, 'tok'), .37);
});

test('请求乘配额超出安全整数乘积时仍精确取整并守恒', () => {
  const platforms = data.build([{ k: 'krill', models: [{ n: 'gpt-6.1-sol', req: Number.MAX_SAFE_INTEGER, tok: 1 }] }]);
  assert.deepEqual(platforms.map(row => row.req), [5629499534213119, 2251799813685248, 1125899906842624, 0]);
  assert.equal(sum(platforms, 'req'), Number.MAX_SAFE_INTEGER);
  close(sum(platforms, 'tok'), 1);
});

test('同名模型的渠道速度分开，适配与 TTFT 元数据来自各自渠道', () => {
  const platforms = data.build(CH);
  const sol = model(platforms, 'app', 'gpt-6.1-sol');
  assert.deepEqual(sol.channels.map(({ k, tps }) => [k, tps]), [['krill', 63], ['newapi', 78], ['a6', 47]]);
  assert.deepEqual(Object.keys(sol).sort(), ['channels', 'n', 'req', 'tok']);
  assert.deepEqual(sol.channels[0], {
    k: 'krill', n: 'Krill', req: 200, tok: .23125, tps: 63, via: true, adapterMs: 18, ttft: .71
  });
  assert.equal(sol.channels[1].via, false);
  assert.equal(sol.channels[1].adapterMs, null);
  assert.equal(sol.channels[1].ttft, .64);
  assert.deepEqual(model(platforms, 'opencode', 'deepseek-v3.2').channels.map(row => row.tps), [55, 66]);
  assert.deepEqual(model(platforms, 'opencode', 'qwen3-max').channels.map(row => row.tps), [47, 88]);
});

test('缺少模型与性能明细时不根据渠道总量或渠道速度补造', () => {
  const empty = data.build([]);
  assert.deepEqual(data.build([
    { k: 'krill', req: 999, tok: 999, tps: 99 },
    { k: 'newapi', req: 999, tok: 999, models: [] },
    { k: 'a6', models: null }
  ]), empty);
  assert.ok(empty.every(row => row.req === 0 && row.tok === 0 && row.models.length === 0));
  const platforms = data.build([{ k: 'krill', n: 'Krill', req: 999, tok: 999, tps: 99, cx: 0, cl: 999,
    models: [{ n: 'gpt-6.1-sol', req: 4, tok: .04 }] }]);
  assert.equal(sum(platforms, 'req'), 4);
  close(sum(platforms, 'tok'), .04);
  for (const branch of branches(platforms)) {
    assert.equal(branch.tps, null);
    assert.equal(branch.ttft, null);
    assert.equal(branch.adapterMs, null);
  }
});

test('未知渠道或模型不进入任何已知平台，owner 返回 null', () => {
  const unknown = [
    ['future', 'gpt-6.1-sol'], ['future', 'deepseek-v3.2'],
    ['krill', 'gpt-future'], ['newapi', 'gpt-6.1-mini'], ['ds', 'qwen3-max'],
    ['__proto__', 'gpt-6.1-sol'], ['krill', 'toString']
  ];
  const channels = unknown.map(([k, n]) => ({ k, n: 'Krill', req: 1000, tok: 1000, models: [{ n, req: 1000, tok: 1000 }] }));
  assert.deepEqual(data.build(channels), data.build([]));
  assert.deepEqual(data.build([...CH, ...channels]), data.build(CH));
  for (const [channel, name] of unknown) for (const index of [0, 1, 2]) {
    assert.equal(data.owner(channel, name, index), null);
  }
});

test('canonical 请求按参与平台轮转，单平台全部归其所有', () => {
  for (const [channel, name, expected] of [
    ['krill', 'gpt-6.1-sol', ['app', 'opencode', 'cli']],
    ['krill', 'gpt-6.1-mini', ['app', 'opencode', 'cli']],
    ['newapi', 'gpt-6.1-sol', ['app', 'opencode', 'cli']],
    ['a6', 'gpt-6.1-sol', ['app', 'cli', 'app']],
    ['a6', 'claude-sonnet-4.5', ['claude', 'claude', 'claude']],
    ['newapi', 'claude-sonnet-4.5', ['opencode', 'opencode', 'opencode']],
    ['or', 'deepseek-v3.2', ['opencode', 'opencode', 'opencode']],
    ['or', 'qwen3-max', ['opencode', 'opencode', 'opencode']],
    ['ds', 'deepseek-v3.2', ['opencode', 'opencode', 'opencode']],
    ['sf', 'qwen3-max', ['opencode', 'opencode', 'opencode']],
    ['ol', 'llama4-scout', ['opencode', 'opencode', 'opencode']]
  ]) {
    assert.deepEqual([0, 1, 2].map(index => data.owner(channel, name, index)), expected);
  }
  assert.equal(data.owner('krill', 'gpt-6.1-sol', 3), 'app');
});

test('owner 跨窗口及 build 调用顺序稳定，build 不改变输入且输出不共享可变引用', () => {
  function freeze(value) {
    if (value && typeof value === 'object') { Object.values(value).forEach(freeze); Object.freeze(value); }
    return value;
  }
  const frozen = freeze(JSON.parse(JSON.stringify(CH)));
  const first = data.build(frozen);
  for (const factor of [1 / 24, 365, 0, 1.2345]) {
    data.build(scaled(factor));
    assert.deepEqual([0, 1, 2].map(index => data.owner('a6', 'gpt-6.1-sol', index)), ['app', 'cli', 'app']);
    assert.deepEqual([0, 1, 2].map(index => data.owner('krill', 'gpt-6.1-sol', index)), ['app', 'opencode', 'cli']);
  }
  assert.deepEqual(data.build(frozen), first);
  first[0].models[0].channels[0].tps = 9999;
  first[0].models[0].channels.length = 0;
  first[0].n = 'changed';
  assert.deepEqual(data.build(frozen), data.build(CH));
  assert.deepEqual(frozen, CH);
});

test('无效请求量或 Token 显式拒绝；无效样例索引不生成归属', () => {
  assert.throws(() => data.build(null), TypeError);
  const build = (req, tok) => data.build([{ k: 'krill', models: [{ n: 'gpt-6.1-sol', req, tok }] }]);
  for (const req of [-1, .5, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1, undefined]) {
    assert.throws(() => build(req, .1), RangeError);
  }
  for (const tok of [-1, NaN, Infinity, undefined, '.1']) assert.throws(() => build(1, tok), RangeError);
  for (const index of [-1, .5, NaN, Infinity, undefined, '0']) {
    assert.equal(data.owner('krill', 'gpt-6.1-sol', index), null);
  }
});
