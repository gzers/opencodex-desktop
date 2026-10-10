# OCX-0110-20261011-RUNTIME-OBSERVATION

## 证据对象

- 版本范围：OpenCodex Desktop 0.1.10
- 代码分支：feature/0.1.10-maintenance
- 代码提交：c654287c06da3e621091c1051ebc19ce1c0b1077
- 提交主题：隔离运行态观测事件并精确解除异常
- 证据性质：生产源码局部回归，不是平台原生验收或发布授权

## 已验证内容

运行态观测异常与 start / stop / restart 的 execution 生命周期异常已分离。runtime-observation-recovered 只有在可执行文件、工作目录、OPENCODEX_HOME、对象、动作、阶段、通道和候选身份全部匹配时，才能解除同一运行上下文的观测异常；解除范围限定为：

- runtime-starting-failed
- run-unreachable
- run-at-risk
- external-takeover

观测恢复不能解除 start / stop / restart 的 execution 生命周期失败。提交同时更新注册表定义、注册校验和发出点，并新增 3 组运行态恢复测试。

## 可复核命令与结果

    cargo test --locked \
      --manifest-path apps/desktop/tauri/Cargo.toml \
      --features integration-test \
      --test event_registry -- --nocapture

结果：33 passed / 0 failed。

提交后的生产源码注册表统计：

| 类别 | 数量 |
|---|---:|
| triggers | 8 |
| jobs | 39 |
| events | 81 |
| notification policies | 27 |
| emission sites | 87 |
| scheduling sites | 8 |
| UI feedback sites | 8 |
| paths | 12 |
| cleanup policies | 4 |

## 边界

本证据不证明全域事件调用点已完整审阅，不证明真实 AppHandle / 原生通知投递，不证明 macOS 或 Windows 原生 UI、安装、权限、路径、外观和 DPI，不证明自动检查性能、stable 24 小时 / beta 6 小时真实周期、真实更新 / 重启、签名、公证 / Authenticode、制品版本或发布门禁。0.1.10 仍保持 in_progress，不得据此合并、替换日常安装、公开发布或晋升 stable。
