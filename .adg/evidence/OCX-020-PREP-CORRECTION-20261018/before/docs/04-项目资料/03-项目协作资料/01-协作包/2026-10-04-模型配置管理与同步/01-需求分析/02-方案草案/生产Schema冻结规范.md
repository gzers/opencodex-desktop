# 生产Schema冻结规范（0.2.0）

日期：2026-10-18（Asia/Shanghai）。状态：**冻结待实施**。本文明确WP-01/02/03的生产数据契约、验证规则和持久化边界，供首个切片实现时作为唯一依据。与[模型模版与匹配规则方案](模型模版与匹配规则方案.md)及[Agent制模与版本恢复方案](Agent制模与版本恢复方案.md)协同，不替代原型演示与后端架构。

本文定义生产就绪的字段语义、生成/校验规则及持久化约束，实施时不再猜测或引入新规则。协议细节、保存流程及同步扩展另见对应方案和核心规范。

---

## 1. templateId：稳定身份契约

`templateId` 是模板的全局唯一稳定标识符，用于跨版本引用、渠道策略绑定及本地/远程库一致性维护。

### 1.1 格式

- **类型**：UUIDv4（符合RFC 4122）
- **表示**：小写、带连字符的标准形式（例：`a3f2c4e1-b5d9-4c8a-9e7f-1a2b3c4d5e6f`）
- **长度**：固定36字符
- **字符集**：`[0-9a-f-]`

### 1.2 生成规则

- **责任方**：后端在首次保存新模板时生成
- **时机**：用户点击"保存"且校验通过后、写入持久化存储前
- **算法**：使用密码学安全的随机数生成器（如Rust `uuid::Uuid::new_v4()`或Node.js `crypto.randomUUID()`）
- **前端职责**：新建编辑时不预生成；提交保存请求后接收后端返回的 `templateId`

### 1.3 碰撞检测

- **保存时检测**：后端在生成 `templateId` 后、写入前，检查当前库是否已存在同ID记录
- **碰撞响应**：若冲突，重新生成新UUIDv4并再次检测；最多重试3次
- **拒绝持久化**：若3次重试仍冲突，返回错误 `TemplateIdCollision`，要求用户稍后重试（实际碰撞概率<1e-15）
- **导入时检测**：导入外部模板时，若 `templateId` 已存在且内容不同，拒绝导入并提示用户；若内容相同视为幂等

### 1.4 不可变性

- 一旦保存成功，`templateId` 永不更改
- 修改模板名称、匹配规则、配置等操作不影响 `templateId`
- 删除后再重建模板会获得新的 `templateId`

### 1.5 引用完整性

- 渠道策略（`policy.json`）中存储的 `adoptedTemplates[].templateId` 必须对应本地库中已存在的模板
- 删除模板前检查：若有渠道策略引用（任一版本），拒绝删除并提示"该模板已被X个渠道采用"
- 级联策略：用户需先解除渠道采用，才能删除模板；或标记归档但保留数据

---

## 2. revisionId：版本自增契约

`revisionId` 标识模板的特定版本快照，每次保存生成新版本时递增。

### 2.1 格式

- **类型**：正整数（从1开始）
- **作用域**：每个 `templateId` 独立维护自己的 `revisionId` 序列
- **表示**：`1, 2, 3, ...`

### 2.2 生成规则

- **首次保存**：新建模板首次保存时，`revisionId = 1`
- **修改保存**：基于已有版本编辑后保存，`revisionId = max(existing_revisionIds) + 1`
- **无内容变更**：若编辑后所有字段与当前选中版本完全一致（字段值、匹配规则、配置均无变化），不生成新版本，提示"无变更"

### 2.3 版本不可变性

- **快照语义**：每个 `revisionId` 记录的正文内容（匹配规则、配置、描述、后缀变体等）在保存后不可修改
- **只读历史**：历史版本仅可查看、复制为新版本、设为默认或导出，不可原地编辑
- **删除策略**：
  - 若某 `revisionId` 被渠道策略采用，不可删除（引用完整性）
  - 若无引用且不是唯一版本，允许删除（归档）
  - 删除后 `revisionId` 序号不回收；下次新版本继续递增

### 2.4 当前版本与默认版本

- **当前版本**：用户在界面中选中查看/编辑的版本，不影响渠道采用
- **默认版本**：用户标记为"默认"的版本，用于新渠道初次采用时的推荐
- **渠道采用**：渠道策略中明确记录 `{templateId, revisionId}`，独立于默认版本

---

## 3. MatchRules：匹配规则契约

### 3.1 结构

```json
{
  "include": [
    {"type": "prefix", "value": "gpt-"},
    {"type": "glob", "value": "claude-*"},
    {"type": "exact", "value": "gemini-2.0-flash-exp"}
  ],
  "exclude": [
    {"type": "glob", "value": "*-embedding"}
  ],
  "caseSensitive": false,
  "channelMode": "all",
  "channels": ["ch-uuid-1", "ch-uuid-2"],
  "priority": 10
}
```

### 3.2 字段约束

| 字段 | 类型 | 必填 | 默认值 | 约束 |
|---|---|---|---|---|
| `include` | `Rule[]` | 是 | - | 至少1条；每条含 `type` 和 `value` |
| `exclude` | `Rule[]` | 否 | `[]` | 可空；与 `include` 类型相同 |
| `caseSensitive` | `boolean` | 否 | `false` | - |
| `channelMode` | `enum` | 否 | `"all"` | `"all"` / `"whitelist"` / `"blacklist"` |
| `channels` | `string[]` | 否 | `[]` | 仅存储稳定 `channelId`（UUIDv4），不存储URL |
| `priority` | `integer` | 否 | `0` | 任意整数；越大越优先 |

### 3.3 Rule类型

- **`prefix`**：前缀匹配，`value` 为字面前缀字符串
- **`glob`**：通配符匹配，`*` 匹配任意长度，`?` 匹配单字符
- **`exact`**：精确匹配，`value` 必须与模型ID完全一致

### 3.4 匹配优先级

1. **单模型精确 `templateId` 覆盖**：若渠道策略中有 `overrides` 明确指定某模型的 `templateId`，直接采用，不经过规则匹配
2. **规则匹配**：依次对每个已采用模板的规则计算优先级
3. **最高优先级唯一**：选中 `priority` 最大的模板
4. **同优先级冲突检测**：若多个模板的 `priority` 并列最高且都匹配，**拒绝保存**，提示用户调整优先级或规则

### 3.5 渠道范围语义

| `channelMode` | `channels` 为空 | `channels` 非空 | 新增渠道时 |
|---|---|---|---|
| `all` | 匹配所有渠道 | 忽略 `channels` | 自动匹配 |
| `whitelist` | 不匹配任何渠道 | 仅匹配列表内渠道 | 默认不匹配，需手动加入 |
| `blacklist` | 匹配所有渠道 | 排除列表内渠道 | 默认匹配，除非手动排除 |

### 3.6 零命中规则

- 允许保存当前发现目录中零命中的规则（用于未来模型或未启用渠道）
- 保存时提示警告："当前规则无匹配模型，但仍可保存"
- 不强制要求每个模板都有匹配对象

---

## 4. 校验与冲突解决

### 4.1 保存时校验

1. **格式校验**：
   - `templateId` 格式符合UUIDv4
   - `revisionId` 为正整数
   - `include` 非空且每条规则有效
   - `channelMode` 为合法枚举值

2. **碰撞检测**：
   - `templateId` 冲突检测（见§1.3）
   - 同 `templateId` 下 `revisionId` 递增一致性

3. **引用完整性**：
   - 渠道策略中采用的 `{templateId, revisionId}` 必须存在于本地库
   - 删除模板/版本前检查引用计数

4. **同优先级冲突**：
   - 对所有已采用模板及待保存模板，计算每个模型的匹配结果
   - 若任一模型被多个同最高优先级模板匹配，**拒绝保存**
   - 提示："模型 `gpt-4o` 同时匹配模板A（优先级10）和模板B（优先级10），请调整优先级或规则"

### 4.2 导入时校验

- 检查导入文件中的 `templateId` 是否与本地冲突
- 若冲突且内容不同，拒绝导入；若内容相同，视为幂等跳过
- 检查 `revisionId` 序列完整性（无跳号或重复）

### 4.3 运行时校验

- 编译渠道策略时，若某采用模板的 `revisionId` 不存在，记录错误并跳过该模板
- 若发现同优先级冲突，记录警告，优先级最高的多个中随机选一个（不推荐，应在保存时阻止）

---

## 5. digest：摘要占位与WP-01责任

### 5.1 字段定义

```json
{
  "templateId": "uuid",
  "revisionId": 3,
  "digest": "sha256:abcdef1234567890...",
  "...": "..."
}
```

### 5.2 职责划分

- **0.2.0 框架责任**：
  - 在数据结构中预留 `digest` 字段（字符串类型）
  - 持久化时保存 `digest` 值（若有）
  - 校验时不强制 `digest` 存在或格式

- **WP-01 责任**：
  - 确定哈希算法（候选：SHA256）
  - 定义摘要计算范围（哪些字段参与、顺序、序列化格式）
  - 实现生成器和校验器
  - 定义摘要失配时的处理策略（警告/拒绝）

### 5.3 当前状态

- **冻结声明**："WP-01负责哈希算法（SHA256候选，待定）"
- 首个切片实现时，`digest` 字段可为空或占位值
- 不阻止功能实现，但框架必须预留字段

---

## 6. 版本不可变性与内置基线保护

### 6.1 版本不可变性

- **原则**：一旦保存，任何版本的正文内容（匹配规则、配置、描述、后缀变体等）不可修改
- **编辑流程**：
  1. 用户选择某版本 → 点击"编辑" → 进入编辑器（显示"基于版本X编辑"）
  2. 修改后点击"保存" → 后端生成新 `revisionId = X+1` → 旧版本X保持不变

### 6.2 内置基线保护

- **来源标记**：每个模板/版本有 `source` 字段：
  - `"builtin"`：软件预置的官方基线（如GPT、Claude等）
  - `"user"`：用户自建或导入
  - `"imported"`：从外部导入但非官方来源

- **系统基线标志**：内置模板有 `isSystemBaseline: true` 标记
- **保护规则**：
  - 内置基线模板不可原地编辑或删除
  - 用户可"复制为新模板"或"基于此版本创建新版本"
  - 软件升级时可添加新内置模板，但不修改已有内置模板的 `templateId` 或历史版本

### 6.3 删除与归档

- **删除检查**：
  1. 若模板任一版本被渠道策略采用，拒绝删除，提示"该模板已被X个渠道采用，请先解除采用"
  2. 若模板标记为 `isSystemBaseline`，拒绝删除
  3. 若模板是用户默认模板，警告"这是你的默认模板，删除后将无默认"

- **级联策略**：
  - 不实现自动级联删除渠道引用
  - 用户需手动切换渠道采用，然后才能删除模板

- **归档（可选）**：
  - 可标记模板为"已归档"，不显示在常用列表，但保留数据
  - 已归档模板仍可被渠道引用（历史兼容）

---

## 7. Variants/Suffix：后缀变体配置

### 7.1 结构

```json
{
  "templateId": "uuid",
  "revisionId": 5,
  "variants": [
    {
      "id": "variant-uuid-1",
      "name": "高推理档",
      "suffix": "-thinking",
      "inheritConfig": true,
      "overrides": {
        "temperature": 1.0
      }
    }
  ]
}
```

### 7.2 存储规则

- **版本绑定**：`variants` 随父版本一同保存，修改后缀配置会生成新 `revisionId`
- **变体ID**：每个变体有独立的 `id`（UUIDv4），用于跨版本跟踪同一变体
- **后缀字面值**：`suffix` 为字符串字面量，匹配时直接拼接到模型ID后（如 `gpt-4o + "-thinking" = gpt-4o-thinking`）

### 7.3 匹配优先级

1. **带后缀模型优先**：若发现目录中同时有 `gpt-4o` 和 `gpt-4o-thinking`，优先匹配最长后缀
2. **变体继承**：变体可选择继承父模板配置，或覆盖部分字段
3. **独立匹配**：变体不单独参与规则匹配，只作为父模板的附加选项

### 7.4 持久化

- 变体随父版本一同序列化到 `templates.json` 中
- 修改变体配置视为修改模板，需生成新版本

---
### 7.5 完整示例

#### 示例1：OpenAI官方模型（无冲突）

```json
{
  "templateId": "openai-gpt-4o-2024-08-06",
  "revisionId": 1,
  "source": "builtin",
  "digest": "sha256:a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6",
  "label": "GPT-4o (Aug 2024)",
  "modelName": "gpt-4o",
  "version": "2024-08-06",
  "capabilities": {
    "vision": true,
    "function_calling": true,
    "streaming": true,
    "batch_processing": true
  },
  "variants": [],
  "metadata": {
    "created": "2024-08-06T00:00:00Z",
    "updatedAt": "2024-08-06T00:00:00Z",
    "status": "active",
    "tags": ["official", "latest"]
  }
}
```

**说明**：
- 官方模型无需冲突检测或变体管理，digest基于完整配置SHA-256哈希计算
- source为 `builtin` 表示随Codex发布的内置模型
- revisionId随每次配置变更递增（用于版本追踪）
- 变体列表为空，表示不支持基于此模型的用户变体

---

#### 示例2：用户自定义模型（含变体与字段映射）

```json
{
  "templateId": "user-deepseek-r1",
  "revisionId": 1,
  "source": "user",
  "digest": "sha256:b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8",
  "label": "DeepSeek R1",
  "modelName": "deepseek-reasoner",
  "version": "local-v1",
  "baseModel": "openai-gpt-4o-2024-08-06",
  "endpointMapping": {
    "compatibleEndpoints": [
      "azure-openai",
      "cloudflare-workers-ai"
    ],
    "fieldMappings": {
      "temperature": "creativeness",
      "top_p": "diversity_level",
      "frequency_penalty": "repetition_control"
    }
  },
  "variants": [
    {
      "variantId": "deepseek-r1-reasoning",
      "label": "DeepSeek R1 (深度推理)",
      "overrides": {
        "temperature": 0.3,
        "system_prompt": "你是一个具有深度推理能力的AI助手。在回答前进行充分的思考。"
      }
    },
    {
      "variantId": "deepseek-r1-fast",
      "label": "DeepSeek R1 (快速响应)",
      "overrides": {
        "temperature": 0.7,
        "timeout": 30000
      }
    }
  ],
  "metadata": {
    "created": "2026-09-15T10:30:00Z",
    "updatedAt": "2026-10-18T14:25:00Z",
    "status": "active",
    "tags": ["user-custom", "reasoning-capable"],
    "source_url": "https://api.deepseek.com/v1"
  }
}
```

**说明**：
- source为 `user` 表示用户通过导入或手动创建的模型
- baseModel指向兼容的官方模型，用于继承部分配置和特性
- endpointMapping定义了字段名称映射（例如temperature→creativeness），支持跨平台模型的参数兼容
- variants包含两个预设配置，用户可在运行时选择不同的行为模式
- revisionId初始为1，修改配置（包括变体）时递增

---

#### 示例3：冲突检测与版本迁移场景

```json
{
  "templateId": "openai-gpt-4o",
  "revisionId": 2,
  "source": "builtin",
  "digest": "sha256:c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a7b8c9",
  "label": "GPT-4o (Latest)",
  "modelName": "gpt-4o",
  "version": "2024-11-20",
  "conflictHistory": [
    {
      "previousVersion": "2024-08-06",
      "previousDigest": "sha256:a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6",
      "reason": "API参数扩展（新增vision_detail字段）",
      "migration": {
        "strategy": "auto-update",
        "breaking_changes": false,
        "user_action_required": false
      }
    }
  ],
  "capabilities": {
    "vision": true,
    "function_calling": true,
    "streaming": true,
    "batch_processing": true,
    "vision_detail": "high"
  },
  "variants": [],
  "metadata": {
    "created": "2024-08-06T00:00:00Z",
    "updatedAt": "2026-10-18T00:00:00Z",
    "status": "active",
    "tags": ["official", "latest", "upgraded"]
  }
}
```

**说明**：
- revisionId=2表示曾发生过版本变更（从2024-08-06升级至2024-11-20）
- conflictHistory记录历次breaking changes及其处理方式
- migration.strategy为 `auto-update` 表示Codex可自动应用此变更，无需用户干预
- breaking_changes=false表示旧配置与新版本兼容，现存工作流无需调整

---

## 8. 实施检查清单

实施首个切片时，必须完成以下检查：

- [ ] `templateId` 使用UUIDv4格式，后端生成并检测碰撞
- [ ] `revisionId` 从1开始递增，无跳号
- [ ] 保存时校验 `include` 非空、`channelMode` 合法
- [ ] 同优先级冲突检测：若多个模板同最高优先级匹配同一模型，拒绝保存
- [ ] 引用完整性：删除模板前检查渠道策略引用
- [ ] 版本不可变：旧版本只读，修改生成新版本
- [ ] 内置基线标记：预置模板标记 `source: "builtin"` 且不可删除
- [ ] `digest` 字段预留但不强制，WP-01负责算法
- [ ] 零命中规则允许保存，但给出警告
- [ ] 导入时检测 `templateId` 冲突，相同内容幂等跳过

---

## 9. 与其他方案的协同

- **[模型模版与匹配规则方案](模型模版与匹配规则方案.md)**：本文是其生产实现细节的冻结
- **[Agent制模与版本恢复方案](Agent制模与版本恢复方案.md)**：版本身份和快照语义由本文定义
- **[模型协议与端点方案](模型协议与端点方案.md)**：协议字段继承规则参考本文的变体机制
- **[数据与状态](../../../../../02-项目核心/数据与状态.md)§6**：本文实现其版本不可变性原则

---

## 10. 版本履历

| 日期 | 变更 |
|---|---|
| 2026-10-18 | 首次冻结，明确templateId/revisionId契约、匹配规则、校验策略及内置基线保护 |


## 3.4 digest生成与验证规则

> **定位**：定义模板内容摘要（digest）的生成算法、验证时机及错误容忍策略，支持0.2.0版本前瞻性演进。

### 3.4.1 null容忍阶段

**适用范围**：0.2.0首发版本  
**规则**：

| 字段 | 生成策略 | 验证策略 | 理由 |
|------|----------|----------|------|
| `digest` | 允许生成端不填 | 验证端跳过null检查 | 首发版不阻塞发布，避免未知边界case |
| `digestAlgorithm` | 填写时必须为`"sha256"` | 识别算法，为null时跳过 | 为后续多算法演进预留 |

**代码示例**（生成端）：

```typescript
// packages/shared/src/types/notification.ts
export interface NotificationTemplate {
  templateId: string;
  revisionId: string;
  digest?: string | null;           // 0.2.0允许为null
  digestAlgorithm?: 'sha256' | null; // 默认sha256
  // ...
}
```

**验证端行为**：

```typescript
// packages/main/src/services/NotificationTemplateRegistry.ts
private validateDigest(template: NotificationTemplate): void {
  if (template.digest == null) {
    this.logger.debug(`Template ${template.templateId} has no digest, skip validation`);
    return; // 0.2.0容忍null
  }
  // 有digest时必须验证
  const computed = this.computeDigest(template);
  if (computed !== template.digest) {
    throw new Error(`Digest mismatch for ${template.templateId}`);
  }
}
```

### 3.4.2 修改补全阶段

**触发时机**：0.2.x后续小版本或0.3.0  
**规则变更**：

| 动作 | 执行点 | 行为 |
|------|--------|------|
| 自动补全 | `registerTemplate`时检测到`digest == null` | 立即计算并回填 |
| 日志记录 | 补全成功后 | 记录`info`级别：`Auto-filled digest for ${templateId}@${revisionId}` |
| 错误处理 | 计算失败 | 拒绝注册，抛出`DigestComputeError` |

**测试验证**：

```typescript
// packages/main/__tests__/NotificationTemplateRegistry.test.ts
describe('digest auto-fill', () => {
  it('should compute and fill null digest', () => {
    const template: NotificationTemplate = {
      templateId: 'test.template',
      revisionId: 'v1',
      digest: null, // 测试null场景
      title: 'Test',
      body: 'Body'
    };
    registry.registerTemplate(template);
    const stored = registry.getTemplate('test.template');
    expect(stored.digest).toMatch(/^[a-f0-9]{64}$/); // SHA-256格式
  });
});
```

### 3.4.3 传输校验规范

**WebDAV下载场景**：

1. **计算时机**：下载完成后、反序列化前
2. **校验逻辑**：
   - 读取JSON中的`digest`字段
   - 对**去除digest字段后的JSON**计算SHA-256
   - 比对结果，不一致则拒绝应用
3. **错误恢复**：记录到`sync-errors.log`，保留本地副本

**Backup恢复场景**：

- 同WebDAV校验逻辑
- 校验失败时提示用户"备份文件可能损坏"，不自动应用

### 3.4.4 测试场景覆盖

**必测场景**（4个）：

| 场景ID | 描述 | 预期结果 |
|--------|------|----------|
| DIG-01 | 0.2.0注册null digest模板 | 成功注册，不抛错 |
| DIG-02 | 0.2.0注册有digest模板但计算不匹配 | 抛`DigestMismatchError` |
| DIG-03 | 0.3.0注册null digest模板 | 自动补全，日志记录 |
| DIG-04 | WebDAV下载篡改digest的JSON | 拒绝应用，记录错误日志 |

**测试实现位置**：

- 单元测试：`packages/main/__tests__/NotificationTemplateRegistry.test.ts`
- 集成测试：`packages/main/__tests__/integration/webdav-digest-validation.test.ts`

---

**变更记录**：
- 2026-10-18：增补§3.4，定义digest生成与验证的三阶段演进策略及4个测试场景
