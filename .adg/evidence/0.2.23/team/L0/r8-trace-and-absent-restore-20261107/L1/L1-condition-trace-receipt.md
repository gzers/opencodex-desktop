L1 / AC-MATCH-05 真条件/逐原子只读预览后端回交

正式源码：9c6da8e2c1ee38992f42b83d4428079dccb4afd3，基线62d698b2c9016970759f5f8283aee902c49f923e，父c002589f7d61935a7da238bf385cc3a0e591e066。分支codex/0.2.23-model-matching已推origin；ls-remote确认同一SHA。工作区干净。

写集仅matching_v2.rs、matching_v2_trace_tests.rs、commands/model_config_v2.rs、model_config_v2_trace_tests.rs四授权文件。未改Schema-v2配置字段、proto/docs/UI、版本/锁/日常配置。旧只读审计原文件原结论保持，能力补齐是本批新源码与新证据，不覆盖旧事实。

实际实现：把原family范围和selector逐atom判断抽为共用Check过程；正常匹配仍同一顺序短路、不分配trace，预览在前置通过后评估全部AND原子。parse_name_v2/glob/pattern底层语义、浮点禁用/完整数字槽位、冲突/协议来源/覆盖/不可变revision均未改。trace读取actual matching_view_v2，rawModelId保持原文。当前纯内容预览没有namespace输入，因此真实matchingView就是raw；核心API有explicit namespace时亦有同源视图测试，不由前端猜。

原model_v2_preview_conditions请求/envelope不变，Row新增matchingView、family(matched/reasonCode/reason)、汇总reasonCode/reason和完整conditions。Condition含conditionId/applicable/matched(bool|null)/applied/reasonCode/reason/preconditions/atoms。family拒绝时condition/atoms都是not-applicable(null)，不报已采用；layout/tail前置失败保false前置与null原子；前置通过时每个AND原子都给真实true/false/理由，即使前一原子失败。applied只对应实际唯一matchedConditionId。族范围、排除、重复前缀、无包含、布局解析、精确尾、数字段数/前导零、完整名称片段边界/位置及AND失败有稳定typed reasonEnum。

保留原验证/overlap错误：坏layout reference在content validation仍Err；真实同时命中两个非重复selector仍model_condition_overlap+真实conditionIds，不返回假green。preview批量trace分配前限65536检查项，超限model_limit；正常resolver预算/结果不变。所有输出仅Serialize，不能成为保存或采用授权。

正式机器合同L1-condition-trace-contract.json；成功schema condition-preview.trace.schema.json SHA256 8a12967cc1ed3d52e328796952227c1f399b7aacee8585370607e71480a089a4；错误schema condition-preview.error.schema.json SHA256 6e8a71b4c19e5e78009d256d2e4872ba17c32249793a756d823414ff4d7e1fe2。schema由当前Rust类型schemars生成；仅测试wire_output_schema薄层修正schemars0.8.22不识别既有serde rename_all_fields的FieldOrigin condition_id/template_id schema属性及required名，跟随实际未变conditionId/templateId serializer；没有删改reason枚举/oneOf/ref，不改变生产字段/输出字节。

真实函数I/O：condition-preview.real-cases.json SHA256 974c837759b7c932456b2a9288b51b9af08ac27df3439da8dde7518da7bee01c，共14组（12成功23行+2结构化Err）。数字./-、完整数字段/前导零/构建/日期边界、any/before/after/错位、全部AND、layout/tail前置失败、完整片段/子串、族scope/exclude/repeat/noinclude、无效layout引用及真正运行时overlap。全部由commands::model_config_v2::preview_conditions实际执行产生，没有renderer声明fixture、第二matcher或模拟成功。

复现只读Rust入口：

~~~sh
cargo test --offline --locked --manifest-path apps/desktop/tauri/Cargo.toml --lib commands::model_config_v2::trace_tests -- --nocapture
~~~

stdout marker CONDITION_PREVIEW_CASE给完整request/result/error，CONDITION_PREVIEW_SCHEMA及CONDITION_PREVIEW_ERROR_SCHEMA给生成schema。原始日志condition-trace-cases-schema-final.log可重提取。L3可直接用producerCommit+sourcehash/schemahash/14回执生成浏览器后端响应，不要自行填matchingView或原子结论。

真实独立严格Draft7验证使用自有artifact venv的jsonschema4.25.1：12成功envelope和2错误全部通过，并核23行原文/条件数组/真实appliedID；6负例严格拒unknown reason、非bool/null值、旧蛇形origin、origin oneOf多字段、缺matchingView和错误conditionIds类型。验证脚本validate-condition-trace-schema.py、报告condition-preview.schema-validation.json。仅工具环境新增包，生产Cargo/npm依赖或锁文件无变更。

Rust验证合计177：matcher trace7；public commands9（含既有预览1）；冻结core37（原246/152、133/28、58参数锁、57边界、19保留均继续过）；真实消费者26；实际config_file::v2边界98。不是此前口头估算28消费者，实际日志计数26。全部0失败/0忽略。严格cargo clippy --lib --tests --no-deps -- -D warnings、全cargo fmt --all --check及git diff --check通过。最终命令日志condition-trace-command-final.log，其余见manifest。

schema校验复现：

~~~sh
python3 -m venv --symlinks /absolute/artifacts/condition-trace-schema-env
/absolute/artifacts/condition-trace-schema-env/bin/python -m pip --isolated --no-cache-dir install jsonschema==4.25.1
/absolute/artifacts/condition-trace-schema-env/bin/python /absolute/artifacts/validate-condition-trace-schema.py
~~~

脚本仅读同目录Rust原始日志并写同目录schema/回执/验证报告，不运行服务或日常文件。首次工具venv复制模式与下载TLS/timeout失败工具输出保留；symlink venv/第二次下载成功。Rust产品测试无原始失败，未用allow/假引用/改断言求绿。source/log/remote SHA全部见L1-condition-trace-manifest.json。

后续：L0集成，L3消费完整DTO/null和preconditions后实际双引擎组件对照，才能在新来源上重新裁决AC05/MATCH05；本批后端通过不冒软件视觉或整版发布通过。未触日常配置、真实推理或发布；产物本地，贡献代码已推远端。
