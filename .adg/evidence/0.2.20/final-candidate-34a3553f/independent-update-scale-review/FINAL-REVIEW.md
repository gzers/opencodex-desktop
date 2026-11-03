UPDATE-SCALE独立复核通过，原阻塞在提交34a3553f9f014c0d553e16e8c509669a3e64fcd9上已解除。

实际UpdateCenter SHA-256：1b3bb05e8e63a9251baeb3f93fe4aaf19479e865847c683df95d8cfd8d398a7f。开始时工作源码、结束时提交源码、Vite实际返回逐字节一致；其他生产源码本轮无变化。

Chromium/WebKit各通过：原始缩放合同2项，原update-hosts156项，扩展几何/比例/操作/滚动372项，仓库永久qa/update-dialog-scale.mjs实际7项。合计1074个断言。

扩展矩阵覆盖Overview/Settings、900×600/1180×760、100/150/200%。实际按钮高度34/51/68px，原200%>=1.8阈值未改；新增精确比例误差<0.03。四边、正文无横向裁切、头尾不收缩、关闭和确认真实鼠标命中、长说明与受限正文内部滚动、Escape/原页返回均通过。两引擎页面/资源/外网错误为0。900×600/200%长说明截图已目检，按钮持续可达。

新增驱动首轮因解析raw响应的JS正则语法错误，浏览器启动前退出1；日志/原harness保留，第二轮修正解析后通过，产品断言未放宽。旧bda真实失败不覆盖，修复结果以新SHA追加。

永久脚本本身已分别实际执行7/7；静态另有非产品阻塞P2：finally无条件completed:true，早期异常时空checks.every可使JSON误写pass，虽然进程仍exit1。建议后续显式记录failure/completed并要求非空checks；本轮实际通过结果不受影响。精确位置、命令、全部报告哈希见repo-regression-review.json、exact-commands.json及handoff.json。

仅本子目录有新增文件，未改产品/共享docs/原型。只做隔离浏览器内存IPC，不代表新正式包/native/OTA验收；主线程继续负责新候选完整CI、包与发布门。
