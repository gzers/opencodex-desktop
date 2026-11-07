# 待单独授权的产品补丁，尚未应用

第三轮全量诊断42392165444，源码a9a230ac9，18 pass / 5 fail，accepted:false。

两个真实普通账户的filesystem-observations.log均证实：完整普通路径及目录访问正常；系统canonicalize返回的路径逐段检查在Windows extended drive prefix处返回Incorrect function (os error 1)，因其尚未拼入RootDir。同一已存在安装目录和Desktop根均复现。

产品bundled::absolute()在拼入Component::Prefix后立即读取元数据，尚未拼入RootDir；这不是完整根路径。建议仅在Windows Prefix阶段延后元数据访问，到RootDir后继续原检查。不忽略错误、不降级供应、不扫描旧路径、不跳过reparse检查。

需要修改apps/desktop/tauri/src/modules/runtime/bundled.rs并补真实Windows普通/extended合法路径和reparse拒绝回归。本文件和旁边patch只是可审查提案，未修改产品。

2026-11-08：用户已明确允许此最小产品修复和真实Windows回归；第四轮42394428304全表已先展示，再应用此补丁。新增第24项独立回执，普通/扩展路径与junction拒绝均须实际执行；本轮仍accepted:false。
