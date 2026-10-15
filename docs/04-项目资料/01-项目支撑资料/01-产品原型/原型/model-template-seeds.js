/* 公开调研摘要的静态原型 fixture；非生产初始库，不进行实时检索。 */
window.ModelTemplateSeeds = [
  {
    "id": "glm",
    "name": "智谱 GLM",
    "prefix": "glm-",
    "models": [
      {
        "id": "seed-glm-5.3",
        "name": "glm-5.3",
        "suffix": "glm-5.3",
        "rules": {
          "modalities": [
            "text"
          ],
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "max"
        },
        "source": "https://docs.z.ai/guides/llm/glm-5.3",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": []
      },
      {
        "id": "seed-glm-5.3-flash",
        "name": "glm-5.3-flash",
        "suffix": "glm-5.3-flash",
        "rules": {
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "max"
        },
        "source": "https://docs.z.ai/guides/vlm/glm-5.3-flash",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "FlashX 与 Flash 的产品额度 / 速度不同；FlashX 不据此视为 OpenAI service_tier。"
        ]
      },
      {
        "id": "seed-glm-5.3-flashx",
        "name": "glm-5.3-flashx",
        "suffix": "glm-5.3-flashx",
        "rules": {
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "max"
        },
        "source": "https://docs.z.ai/guides/vlm/glm-5.3-flash",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "FlashX 与 Flash 的产品额度 / 速度不同；FlashX 不据此视为 OpenAI service_tier。"
        ]
      },
      {
        "id": "seed-glm-5.2",
        "name": "glm-5.2",
        "suffix": "glm-5.2",
        "rules": {
          "modalities": [
            "text"
          ]
        },
        "source": "https://docs.z.ai/guides/llm/glm-5.2",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": []
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "hy",
    "name": "腾讯混元 HY",
    "prefix": "hy",
    "models": [
      {
        "id": "seed-hy3",
        "name": "hy3",
        "suffix": "hy3",
        "rules": {
          "modalities": [
            "text"
          ],
          "efforts": [
            "none",
            "low",
            "high"
          ],
          "default": "high"
        },
        "source": "https://intl.cloud.tencent.com/zh/document/product/1300/78934",
        "sourceAccess": "indexed-excerpt",
        "scope": "tencent-tokenhub",
        "lifecycle": "documented",
        "notes": [
          "旧混元平台与 TokenHub 不混用连接；工具请求的 low 映射须按具体型号复核。"
        ]
      },
      {
        "id": "seed-hy4-preview",
        "name": "hy4-preview",
        "suffix": "hy4-preview",
        "rules": {
          "modalities": [
            "text"
          ],
          "efforts": [
            "none",
            "high"
          ],
          "default": "high"
        },
        "source": "https://intl.cloud.tencent.com/zh/document/product/1300/78934",
        "sourceAccess": "indexed-excerpt",
        "scope": "tencent-tokenhub",
        "lifecycle": "preview",
        "notes": [
          "旧混元平台与 TokenHub 不混用连接；工具请求的 low 映射须按具体型号复核。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "doubao",
    "name": "豆包 Seed",
    "prefix": "doubao-",
    "models": [
      {
        "id": "seed-doubao-seed-2-1-pro-260915",
        "name": "doubao-seed-2-1-pro-260915",
        "suffix": "doubao-seed-2-1-pro-260915",
        "rules": {},
        "source": "https://docs.volcengine.com/docs/ark/model-list?lang=zh",
        "sourceAccess": "page",
        "scope": "volcengine-ark",
        "lifecycle": "documented",
        "notes": [
          "官方目录列多模态理解；具体图像 / 视频限制待型号详情复核，模态不直接补满。",
          "最大回答与最大思维链分别核对；不能把输出上限当默认输出值。",
          "保留日期 ID；ep 接入点是渠道绑定，不作为全局型号。"
        ]
      },
      {
        "id": "seed-doubao-seed-2-1-lite-260915",
        "name": "doubao-seed-2-1-lite-260915",
        "suffix": "doubao-seed-2-1-lite-260915",
        "rules": {},
        "source": "https://docs.volcengine.com/docs/ark/model-list?lang=zh",
        "sourceAccess": "page",
        "scope": "volcengine-ark",
        "lifecycle": "documented",
        "notes": [
          "官方目录列多模态理解；具体图像 / 视频限制待型号详情复核，模态不直接补满。",
          "最大回答与最大思维链分别核对；不能把输出上限当默认输出值。",
          "保留日期 ID；ep 接入点是渠道绑定，不作为全局型号。"
        ]
      },
      {
        "id": "seed-doubao-seed-2-1-turbo-260628",
        "name": "doubao-seed-2-1-turbo-260628",
        "suffix": "doubao-seed-2-1-turbo-260628",
        "rules": {},
        "source": "https://docs.volcengine.com/docs/ark/model-list?lang=zh",
        "sourceAccess": "page",
        "scope": "volcengine-ark",
        "lifecycle": "documented",
        "notes": [
          "官方目录列多模态理解；具体图像 / 视频限制待型号详情复核，模态不直接补满。",
          "最大回答与最大思维链分别核对；不能把输出上限当默认输出值。",
          "保留日期 ID；ep 接入点是渠道绑定，不作为全局型号。"
        ]
      },
      {
        "id": "seed-doubao-seed-evolving",
        "name": "doubao-seed-evolving",
        "suffix": "doubao-seed-evolving",
        "rules": {},
        "source": "https://docs.volcengine.com/docs/ark/model-list?lang=zh",
        "sourceAccess": "page",
        "scope": "volcengine-ark",
        "lifecycle": "rolling",
        "notes": [
          "官方目录列多模态理解；具体图像 / 视频限制待型号详情复核，模态不直接补满。",
          "最大回答与最大思维链分别核对；不能把输出上限当默认输出值。",
          "保留日期 ID；ep 接入点是渠道绑定，不作为全局型号。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "qwen",
    "name": "千问 Qwen",
    "prefix": "qwen",
    "models": [
      {
        "id": "seed-qwen3.8-max",
        "name": "qwen3.8-max",
        "suffix": "qwen3.8-max",
        "rules": {},
        "source": "https://help.aliyun.com/zh/model-studio/text-generation-model",
        "sourceAccess": "page",
        "scope": "alibaba-model-studio",
        "lifecycle": "documented",
        "notes": [
          "文档支持多模态输入；本资料未逐一确认模态组合，保持待核实。",
          "地域与快照需绑定后复核；不同型号不统一推理枚举。"
        ]
      },
      {
        "id": "seed-qwen3.8-flash",
        "name": "qwen3.8-flash",
        "suffix": "qwen3.8-flash",
        "rules": {},
        "source": "https://help.aliyun.com/zh/model-studio/text-generation-model",
        "sourceAccess": "page",
        "scope": "alibaba-model-studio",
        "lifecycle": "documented",
        "notes": [
          "文档支持多模态输入；本资料未逐一确认模态组合，保持待核实。",
          "地域与快照需绑定后复核；不同型号不统一推理枚举。"
        ]
      },
      {
        "id": "seed-qwen3.7-plus",
        "name": "qwen3.7-plus",
        "suffix": "qwen3.7-plus",
        "rules": {},
        "source": "https://help.aliyun.com/zh/model-studio/text-generation-model",
        "sourceAccess": "page",
        "scope": "alibaba-model-studio",
        "lifecycle": "documented",
        "notes": [
          "文档支持多模态输入；本资料未逐一确认模态组合，保持待核实。",
          "地域与快照需绑定后复核；不同型号不统一推理枚举。"
        ]
      },
      {
        "id": "seed-qwen3.7-flash",
        "name": "qwen3.7-flash",
        "suffix": "qwen3.7-flash",
        "rules": {},
        "source": "https://help.aliyun.com/zh/model-studio/text-generation-model",
        "sourceAccess": "page",
        "scope": "alibaba-model-studio",
        "lifecycle": "documented",
        "notes": [
          "文档支持多模态输入；本资料未逐一确认模态组合，保持待核实。",
          "地域与快照需绑定后复核；不同型号不统一推理枚举。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "deepseek",
    "name": "DeepSeek",
    "prefix": "deepseek-",
    "models": [
      {
        "id": "seed-deepseek-flash",
        "name": "deepseek-flash",
        "suffix": "deepseek-flash",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "high"
        },
        "source": "https://api-docs.deepseek.com/quick_start/pricing/",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "文档对应 DeepSeek-V4.1-Flash；显示版本与 API model ID 分开。"
        ]
      },
      {
        "id": "seed-deepseek-v4-pro",
        "name": "deepseek-v4-pro",
        "suffix": "deepseek-v4-pro",
        "rules": {
          "modalities": [
            "text"
          ],
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "high"
        },
        "source": "https://api-docs.deepseek.com/quick_start/pricing/",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": []
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "kimi",
    "name": "Kimi / Moonshot",
    "prefix": "kimi-",
    "models": [
      {
        "id": "seed-kimi-k3",
        "name": "kimi-k3",
        "suffix": "kimi-k3",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "low",
            "high",
            "max"
          ],
          "default": "max"
        },
        "source": "https://platform.kimi.ai/docs/guide/kimi-k3-quickstart",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": []
      },
      {
        "id": "seed-kimi-k2.6",
        "name": "kimi-k2.6",
        "suffix": "kimi-k2.6",
        "rules": {},
        "source": "https://platform.kimi.ai/docs/guide/kimi-k2-6-quickstart",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": []
      },
      {
        "id": "seed-kimi-k2.7-code",
        "name": "kimi-k2.7-code",
        "suffix": "kimi-k2.7-code",
        "rules": {},
        "source": "https://platform.kimi.ai/docs/guide/kimi-k2-6-quickstart",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "此来源确认型号与上下文；其余能力等待该型号详情。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "minimax",
    "name": "MiniMax",
    "prefix": "minimax-",
    "models": [
      {
        "id": "seed-minimax-m3.1-flash-preview",
        "name": "MiniMax-M3.1-Flash-Preview",
        "suffix": "MiniMax-M3.1-Flash-Preview",
        "rules": {
          "context": 1000000,
          "efforts": [
            "low",
            "medium",
            "high",
            "xhigh",
            "max"
          ],
          "default": "max"
        },
        "source": "https://platform.minimax.io/docs/guides/text-generation",
        "sourceAccess": "page",
        "scope": "minimax-m-plan-and-code",
        "lifecycle": "preview-restricted",
        "notes": [
          "截至核对日仅 M Plan / MiniMax Code 提供；不等于所有按量 API Key 可用。",
          "原厂称多模态；具体模态与最大输出未在本资料确认。"
        ]
      },
      {
        "id": "seed-minimax-m3",
        "name": "MiniMax-M3",
        "suffix": "MiniMax-M3",
        "rules": {
          "context": 1000000
        },
        "source": "https://platform.minimax.io/docs/guides/text-generation",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "保留原厂大小写；highspeed 是独立 API ID，不推导成所有渠道 Fast。"
        ]
      },
      {
        "id": "seed-minimax-m2.7",
        "name": "MiniMax-M2.7",
        "suffix": "MiniMax-M2.7",
        "rules": {
          "context": 204800
        },
        "source": "https://platform.minimax.io/docs/guides/text-generation",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "保留原厂大小写；highspeed 是独立 API ID，不推导成所有渠道 Fast。"
        ]
      },
      {
        "id": "seed-minimax-m2.7-highspeed",
        "name": "MiniMax-M2.7-highspeed",
        "suffix": "MiniMax-M2.7-highspeed",
        "rules": {
          "context": 204800
        },
        "source": "https://platform.minimax.io/docs/guides/text-generation",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "保留原厂大小写；highspeed 是独立 API ID，不推导成所有渠道 Fast。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "ernie",
    "name": "百度 ERNIE",
    "prefix": "ernie-",
    "models": [
      {
        "id": "seed-ernie-5.0",
        "name": "ernie-5.0",
        "suffix": "ernie-5.0",
        "rules": {},
        "source": "https://intl.cloud.baidu.com/en/doc/qianfan/s/7m95lyy43-intl-en",
        "sourceAccess": "indexed-excerpt",
        "scope": "baidu-qianfan",
        "lifecycle": "documented",
        "notes": [
          "千帆公开目录可核对型号与限制；具体协议、思考枚举和模态仍需对应详情。"
        ]
      },
      {
        "id": "seed-ernie-5.0-thinking-preview",
        "name": "ernie-5.0-thinking-preview",
        "suffix": "ernie-5.0-thinking-preview",
        "rules": {},
        "source": "https://cloud.baidu.com/product-s/qianfan_home",
        "sourceAccess": "indexed-excerpt",
        "scope": "baidu-qianfan",
        "lifecycle": "preview",
        "notes": []
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "step",
    "name": "阶跃 Step",
    "prefix": "step-",
    "models": [
      {
        "id": "seed-step-3.5-flash",
        "name": "step-3.5-flash",
        "suffix": "step-3.5-flash",
        "rules": {},
        "source": "https://www.stepfun.com/step-3.5-flash",
        "sourceAccess": "indexed-excerpt",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "权重与云端部署限制分别验证；其余接口字段待模型详情。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "mimo",
    "name": "小米 MiMo",
    "prefix": "mimo-",
    "models": [
      {
        "id": "seed-mimo-v2.5-pro",
        "name": "mimo-v2.5-pro",
        "suffix": "mimo-v2.5-pro",
        "rules": {},
        "source": "https://mimo.xiaomi.com/mimo-v2-5",
        "sourceAccess": "indexed-excerpt",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "官方示例确认精确 API ID；不能从 V2.5 普通版推导 Pro 模态、上下文或推理枚举。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "gpt",
    "name": "OpenAI GPT",
    "prefix": "gpt-",
    "models": [
      {
        "id": "seed-gpt-6-astra",
        "name": "gpt-6-astra",
        "suffix": "gpt-6-astra",
        "rules": {
          "context": 1050000,
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "low",
            "medium",
            "high",
            "xhigh",
            "max"
          ]
        },
        "source": "https://developers.openai.com/api/docs/models/gpt-6-astra",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "工具调用优先 Responses；Chat Completions 的可用参数另按型号限制。",
          "本机 ultra 属代理执行层选项，不列为原厂原生 effort。"
        ]
      },
      {
        "id": "seed-gpt-6.1-sol",
        "name": "gpt-6.1-sol",
        "suffix": "gpt-6.1-sol",
        "rules": {
          "context": 1050000,
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "low",
            "medium",
            "high",
            "xhigh",
            "max"
          ],
          "default": "medium"
        },
        "source": "https://developers.openai.com/api/docs/models/gpt-6.1-sol",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "工具调用优先 Responses；Chat Completions 的可用参数另按型号限制。",
          "本机 ultra 属代理执行层选项，不列为原厂原生 effort。"
        ]
      },
      {
        "id": "seed-gpt-6-luna",
        "name": "gpt-6-luna",
        "suffix": "gpt-6-luna",
        "rules": {
          "context": 1050000,
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "none",
            "low",
            "medium",
            "high",
            "xhigh",
            "max"
          ],
          "default": "medium"
        },
        "source": "https://developers.openai.com/api/docs/models/gpt-6-luna",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "工具调用优先 Responses；Chat Completions 的可用参数另按型号限制。",
          "本机 ultra 属代理执行层选项，不列为原厂原生 effort。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "claude",
    "name": "Claude",
    "prefix": "claude-",
    "models": [
      {
        "id": "seed-claude-fable-5-1",
        "name": "claude-fable-5-1",
        "suffix": "claude-fable-5-1",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "default": "high"
        },
        "source": "https://platform.claude.com/docs/en/models/overview",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "默认 effort 已确认；是否允许关闭 thinking 与完整枚举须逐型号查 capabilities。"
        ]
      },
      {
        "id": "seed-claude-opus-5-5",
        "name": "claude-opus-5-5",
        "suffix": "claude-opus-5-5",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "default": "medium"
        },
        "source": "https://platform.claude.com/docs/en/models/overview",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "默认 effort 已确认；是否允许关闭 thinking 与完整枚举须逐型号查 capabilities。"
        ]
      },
      {
        "id": "seed-claude-sonnet-5-5",
        "name": "claude-sonnet-5-5",
        "suffix": "claude-sonnet-5-5",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "default": "high"
        },
        "source": "https://platform.claude.com/docs/en/models/overview",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "默认 effort 已确认；是否允许关闭 thinking 与完整枚举须逐型号查 capabilities。"
        ]
      },
      {
        "id": "seed-claude-haiku-5-5",
        "name": "claude-haiku-5-5",
        "suffix": "claude-haiku-5-5",
        "rules": {
          "modalities": [
            "text",
            "image"
          ],
          "default": "medium"
        },
        "source": "https://platform.claude.com/docs/en/models/overview",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "默认 effort 已确认；是否允许关闭 thinking 与完整枚举须逐型号查 capabilities。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "gemini",
    "name": "Gemini",
    "prefix": "gemini-",
    "models": [
      {
        "id": "seed-gemini-3.8-flash",
        "name": "gemini-3.8-flash",
        "suffix": "gemini-3.8-flash",
        "rules": {
          "efforts": [
            "low",
            "medium",
            "high"
          ],
          "default": "medium"
        },
        "source": "https://ai.google.dev/gemini-api/docs/latest-model",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "minimal 不支持；GenerateContent 参数表达与模态仍需按具体接口确认。"
        ]
      },
      {
        "id": "seed-gemini-3.6-flash",
        "name": "gemini-3.6-flash",
        "suffix": "gemini-3.6-flash",
        "rules": {},
        "source": "https://ai.google.dev/gemini-api/docs/latest-model",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "登记已在官方文档出现的名称；本次未逐型号冻结能力。"
        ]
      },
      {
        "id": "seed-gemini-3.7-flash",
        "name": "gemini-3.7-flash",
        "suffix": "gemini-3.7-flash",
        "rules": {},
        "source": "https://ai.google.dev/gemini-api/docs/latest-model",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "登记已在官方文档出现的名称；本次未逐型号冻结能力。"
        ]
      },
      {
        "id": "seed-gemini-3.5-flash-lite",
        "name": "gemini-3.5-flash-lite",
        "suffix": "gemini-3.5-flash-lite",
        "rules": {},
        "source": "https://ai.google.dev/gemini-api/docs/latest-model",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "登记已在官方文档出现的名称；本次未逐型号冻结能力。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "grok",
    "name": "Grok",
    "prefix": "grok-",
    "models": [
      {
        "id": "seed-grok-4.7",
        "name": "grok-4.7",
        "suffix": "grok-4.7",
        "rules": {
          "context": 500000,
          "modalities": [
            "text",
            "image"
          ],
          "efforts": [
            "low",
            "medium",
            "high",
            "xhigh"
          ],
          "default": "high"
        },
        "source": "https://docs.x.ai/developers/models/grok-4.7",
        "sourceAccess": "page",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "Fast 为同模型服务速度档；不自造 grok-4.7-fast model ID。"
        ]
      },
      {
        "id": "seed-grok-4.6",
        "name": "grok-4.6",
        "suffix": "grok-4.6",
        "rules": {},
        "source": "https://docs.x.ai/developers/model-capabilities/text/reasoning",
        "sourceAccess": "indexed-excerpt",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "只确认支持 reasoning_effort，完整枚举与限制待型号页。"
        ]
      },
      {
        "id": "seed-grok-4.5",
        "name": "grok-4.5",
        "suffix": "grok-4.5",
        "rules": {},
        "source": "https://docs.x.ai/developers/model-capabilities/text/reasoning",
        "sourceAccess": "indexed-excerpt",
        "scope": "vendor-api",
        "lifecycle": "documented",
        "notes": [
          "只确认支持 reasoning_effort，完整枚举与限制待型号页。"
        ]
      }
    ],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "mistral",
    "name": "Mistral / Devstral",
    "prefix": "mistral-",
    "models": [],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "gemma",
    "name": "Gemma",
    "prefix": "gemma",
    "models": [],
    "checkedAt": "2026-10-13"
  },
  {
    "id": "llama",
    "name": "Llama",
    "prefix": "llama",
    "models": [],
    "checkedAt": "2026-10-13"
  }
];
