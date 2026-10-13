# Windows 焦点草稿PR与中断后说明

焦点修复及测试期探针诊断已整理为PR #1，标题“fix(updates): 切换详情视图后保留弹窗焦点（待Windows原生复验）”，目标main/eae8532，head为feature/0.1.10-update-center-focus/be7395a。PR保持open/draft，merged=false、auto_merge=null；已附加到本任务，没有请求合并或转正式评审。

修改范围为5份代码/CI文件：UpdateCenter.vue、对应测试、版本来源测试期诊断、Windows查询夹具及ci.yml。文档/治理事实仍在docs/governance-main，不复制代码进入文档分支。

创建前认证核对同分支不存在PR、远端head/base未变、同源push CI及候选成功。GitHub连接器创建返回403“Resource not accessible by integration”，未创建；随后使用已授权本机Git凭据在内存请求仓库API，再次核对无重复后创建成功，凭据未存盘/打印。首次列表包装把空JSON数组作为单一对象计数，独立HTTP200/NoEnumerate读回确认0后才创建，没有依据错误计数操作。

2026-10-13 16:05:19（北京时间）真实读回仍为同head/base的草稿，PR CI38361436746/push CI38352525092均completed/success。PR CI头SHA用于关联结果，不宣称其PR合并检出的运行二进制等同候选字节；候选身份仍由38355803684的单独制品证据拥有。

16:34:21只更新PR描述，纠正“原应用/周期仍活”的陈旧说法：旧eae应用、监督器、监测均已不存在、退出时间和原因未知；201份旧前缀与原持久截止保持，新候选尚未启动。描述明确下一步验证原截止的跨启动真实自动行为，不能把定时记录或两段进程计时拼为连续运行；仍不请求合并/发布。创建时原描述和后续描述分别保留，不改旧原件。

原件归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-DRAFT-PR/`，14份全集合/字节摘要冻结重验通过。这个passed只证明PR事实与归档一致，不代表原生焦点修复、原CI失败定位、完整版本门禁或发布通过。

TASK182原件、FZ50许可、安全环境、双令牌/物理DPI/安装与数据保留、实际更新/备份/事件矩阵、原预算同机配对性能及真实周期仍开放，历史failed保持。IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。
