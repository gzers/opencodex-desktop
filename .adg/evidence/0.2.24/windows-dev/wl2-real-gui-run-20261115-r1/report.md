# 真实普通桌面GUI首次失败现场

用户在marenben实际桌面执行原不可变入口，源0c618cdfde4da37da0ca90e1b0913d7d81f63a29，沿用准备轮三份准确制品，不拿03d8的新编译替换。原SID1003/session7/elevated=false，bootstrap与原挂起broker均noJob，原创建句柄仍由observer持有时写出suspended-original。observer pid79512，原broker pid40588/FILETIME134392249926679744。

observer原退出101：7.94秒，0过1败；首错为D/gui-normal.json未生成。真实GUI stderr先报告admission refused raw=None，随后Chromium unregister1412；不把后者当首错。未取得main窗口receipt/manager gen/正常退出/故障续接等成功证据，actualGUIComponentPassed=false；并非fixture manager，也不能签真实H或生产闭环。

同SID自动采集complete=true，14份独立日志逐份base64解码/长度/SHA256核验，原export和启动脚本/输入清单保留。controller读取部分原私有JSON被拒绝，未改ACL，消费既有同SID导出即可。原任务根C:/Users/marenben/ocx024-h-5d09945d-8184-4ccb-b63c-b8a7ba280d7b保留；broker是否最终退出未知，不填raw0，不凭PID杀/收养；无删除/权限修复/日常配置改动。

静态定位到Windows gui_probe使用platform::home_dir；GUI启动刻意启用sandbox，platform::home_dir返回GUI-normal/home而非OS Profile，因此base.parent与profile断言不成立。后继先置假设，改Windows探针读取现有原Token Profile接口，保持完整路径、sandbox、令牌、来源、窗口和预算断言；不改shared platform或取消sandbox，不重复原条件空跑。
