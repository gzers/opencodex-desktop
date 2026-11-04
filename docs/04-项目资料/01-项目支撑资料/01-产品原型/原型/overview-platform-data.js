/* 概览平台用量：独立的纯数据样例，不读取 DOM、网络、时钟或存储。 */
(function (root, factory) {
  'use strict';
  var api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.OverviewPlatformData = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  var PLATFORMS = [
    { k: 'app', n: 'Codex App', c: 'codex' },
    { k: 'opencode', n: 'OpenCode', c: 'opencode' },
    { k: 'cli', n: 'Codex CLI', c: 'cli' },
    { k: 'claude', n: 'Claude Code', c: 'claude' }
  ];

  /* 明确的静态 mock 配额，顺序为 app / opencode / cli / claude。
     这些数仅是演示分配权重，不由 cx、cl、渠道总量或真实历史反推。
     只认以下完整渠道键 × 模型名；未知组合不归属，不默认塞给已知平台。
     因此四平台总量只与已知组合的输入模型明细守恒；未知及缺失明细不计入。 */
  var QUOTAS = new Map([
    ['krill', new Map([
      ['gpt-6.1-sol', [200, 80, 40, 0]],
      ['gpt-6.1-mini', [118, 40, 20, 0]]
    ])],
    ['newapi', new Map([
      ['gpt-6.1-sol', [420, 180, 100, 0]],
      ['claude-sonnet-4.5', [0, 369, 0, 0]]
    ])],
    ['a6', new Map([
      ['claude-sonnet-4.5', [0, 0, 0, 496]],
      ['gpt-6.1-sol', [81, 0, 40, 0]]
    ])],
    ['or', new Map([
      ['deepseek-v3.2', [0, 132, 0, 0]],
      ['qwen3-max', [0, 82, 0, 0]]
    ])],
    ['ds', new Map([['deepseek-v3.2', [0, 156, 0, 0]]])],
    ['sf', new Map([['qwen3-max', [0, 58, 0, 0]]])],
    ['ol', new Map([['llama4-scout', [0, 19, 0, 0]]])]
  ]);

  function participants(channelKey, modelName) {
    var models = QUOTAS.get(channelKey), weights = models && models.get(modelName);
    if (!weights) return [];
    return weights.map(function (weight, index) {
      return { weight: weight, index: index };
    }).filter(function (part) { return part.weight > 0; });
  }

  /* 最大余数法；余数相等时按固定平台顺序分配。BigInt 避免安全整数请求量
     乘以权重后超出 Number 的精确范围，窗口缩放后也不改变分配规则。 */
  function splitRequests(total, parts) {
    var weight = BigInt(parts.reduce(function (sum, part) { return sum + part.weight; }, 0));
    var shares = parts.map(function (part, index) {
      var numerator = BigInt(total) * BigInt(part.weight);
      return { index: index, req: Number(numerator / weight), remainder: numerator % weight };
    });
    var left = total - shares.reduce(function (sum, share) { return sum + share.req; }, 0);
    var ranked = shares.slice().sort(function (a, b) {
      return a.remainder === b.remainder ? a.index - b.index : a.remainder > b.remainder ? -1 : 1;
    });
    for (var i = 0; i < left; i++) ranked[i].req++;
    return shares.map(function (share) { return share.req; });
  }

  function metric(value) {
    return Number.isFinite(value) && value >= 0 ? value : null;
  }

  /**
   * build(channels) -> 固定顺序的四平台数组；每次返回全新对象，不改变输入。
   * 仅从 models[] 汇总，不用渠道级 req/tok/fail 补造模型；缺少 models 时跳过。
   * 已知模型的 req 必须是非负安全整数，tok 必须是非负有限 Number，单位沿用输入。
   * Token 按同一静态配额连续分配，末分支接收浮点余数；不做显示精度截断。
   * 守恒容许 Number 的浮点精度误差。很小的窗口可能有 req=0、tok>0 的分支，
   * 因为 Token 配额独立于请求取整。两者皆零的分支省略，空平台保留。
   * 速度只来自 model.tps 并保留在渠道分支；缺失性能指标为 null，不回退或混合。
   */
  function build(channels) {
    if (!Array.isArray(channels)) throw new TypeError('channels must be an array');
    var platforms = PLATFORMS.map(function (platform) {
      return { k: platform.k, n: platform.n, c: platform.c, req: 0, tok: 0, models: [] };
    });
    var modelMaps = platforms.map(function () { return new Map(); });

    channels.forEach(function (channel) {
      if (!channel || !Array.isArray(channel.models)) return;
      channel.models.forEach(function (model) {
        if (!model) return;
        var parts = participants(channel.k, model.n);
        if (!parts.length) return;
        if (!Number.isSafeInteger(model.req) || model.req < 0) {
          throw new RangeError('model.req must be a non-negative safe integer');
        }
        if (metric(model.tok) === null) {
          throw new RangeError('model.tok must be a non-negative finite number');
        }
        var requests = splitRequests(model.req, parts);
        var weight = parts.reduce(function (sum, part) { return sum + part.weight; }, 0);
        var remainingTokens = model.tok;
        parts.forEach(function (part, index) {
          var tok = index === parts.length - 1 ? remainingTokens : model.tok * (part.weight / weight);
          remainingTokens -= tok;
          var req = requests[index];
          if (!req && !tok) return;
          var platform = platforms[part.index], models = modelMaps[part.index];
          var aggregate = models.get(model.n);
          if (!aggregate) {
            aggregate = { n: model.n, req: 0, tok: 0, channels: [] };
            models.set(model.n, aggregate);
            platform.models.push(aggregate);
          }
          aggregate.channels.push({
            k: channel.k, n: channel.n, req: req, tok: tok, tps: metric(model.tps),
            via: !!channel.via, adapterMs: channel.via ? metric(channel.adapterMs) : null,
            ttft: metric(channel.ttft)
          });
          aggregate.req += req;
          aggregate.tok += tok;
          platform.req += req;
          platform.tok += tok;
        });
      });
    });
    return platforms;
  }

  /* canonical 样例索引从 0 起，在静态参与平台中轮转；不是按取整后请求量
     抽样，也不读取最近一次 build。故同一组合和索引跨窗口、调用顺序稳定。 */
  function owner(channelKey, modelName, index) {
    if (!Number.isSafeInteger(index) || index < 0) return null;
    var parts = participants(channelKey, modelName);
    return parts.length ? PLATFORMS[parts[index % parts.length].index].k : null;
  }

  return { build: build, owner: owner };
});
