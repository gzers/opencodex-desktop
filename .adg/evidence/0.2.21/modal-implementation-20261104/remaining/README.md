# POST-09 剩余模态消费者迁移完成证据

TASK-OPENCODEX-UI021-MODALS-20261104 / AC-021-07 的追加独占写集。8 个消费者文件与 UiDialog 可选 alertdialog 合同已完成，12 个对应测试文件新增/迁移；完整路径/摘要见 owned-files.json，源码增量见 changes.patch。仅改对应模态部分，未改 sync、其它 worker 的 models、stores/app.ts、types/ui.ts 或全局 CSS。

## 先代表再扩

先完成异常提示、受限卸载、版本删除/默认确认三个代表类别。19 项代表检查通过后扩概览、行明细、执行模型配置/选择与官方恢复引导。before/、baseline.json、before-representative.log、before-model-hosts.log 保留原实现与反例，representative-tests.log 保留扩迁移前通过结果。

版本删除从内嵌确认变为公共子 dialog：取消/X/Esc/遮罩都只退出确认，保留父版本窗口与选择；不删除版本。默认版本确认也保留父窗口与实际触发点。匹配/外部任务结果关闭只终止本窗口观察，旧响应不能污染重开窗口或丢弃草稿。模型明细既有行输入及执行模型已保存偏好在取消时保持。

异常窗保留 role=alertdialog、重新加载界面，去掉自己的全局 Esc 与关闭提示按钮。概览两窗保留目录/指引/重新检查业务动作，去掉全局 Esc 和底部关闭，嵌套先退子层。卸载执行阶段所有退出同守卫，无虚构取消能力；完成只从X退出。窄窗确认区现位于固定footer，原同意文案与所有约束保持，避免旧sticky覆盖选项。

## 验证

- targeted.log：15 文件 / 135 项定向测试通过，0 失败、0 未处理异常。未重复全 UI。
- build.log：vue-tsc 与 Vite build 通过。typecheck.log 另有单独通过记录。
- browser-results.json：Chromium/WebKit 各15组（共30）流程/布局，40项几何检查通过；无pageerror及控制台warning/error。宽亮高100%与窄暗低125%两种组合，实际theme/effects也记录校验；截图在screenshots/。
- 已目检窄窗卸载固定同意区、明细滚动/页脚、嵌套版本确认及公共X。浏览器是软件Vue消费者与内存原生传输夹具，壳层无关组件/控制器隔离，未访问日常配置/执行真实安装卸载重启。
- overview-preservation.json：排除两模态块、相关Esc监听及局部宽度规则后，概览其余内容前后SHA-256相等；主线WebDAV投影/线上版本模板保持。
- settings-uninstall与update-release-details-flow按主线要求迁移nativeDialog桩/Teleport断言，原业务断言全部保留。浏览器还暴露未打开批量表单的undefined默认值告警，已在所属模态段补回原有keep值；未删除或忽略告警。

主线已报告sync/WebDAV 128/128、每引擎56通过，此处不重复跑。workspace失败仍由主线负责。

## CI 与原生接入

命令见ci-targeted.txt，逐宿主状态见consumer-matrix.json，原生选择器变化见ci-native-handoff.md。原生macOS/Windows及统一全UI交由主线；本目录不发TASK关闭或整版验收。

ChannelEditor.vue:156 当前已由主线接好 actions.close 和 disabled=busy，行级复核通过，不需重复修改。本贡献未编辑ChannelEditor、ChannelsView、SaveResultNotice或Node worker文件。

追加写集内部没有未完实现。未执行git add/commit/push，未改日常配置或重启日常Codex。
