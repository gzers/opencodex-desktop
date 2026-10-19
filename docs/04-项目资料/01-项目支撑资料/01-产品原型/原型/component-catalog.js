/* 原型组件目录：ID、章节、样例与代码映射同源；登记不等于生产已实现。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.PrototypeComponentCatalog=api;})(typeof window==='undefined'?globalThis:window,function(){
 'use strict';
 const groups=[['foundations','设计基础'],['layout','布局与组合'],['controls','基础控件'],['data','数据展示'],['overlays','浮层与反馈'],['compound','复合选择器'],['platform','平台外壳']].map(x=>({id:x[0],label:x[1]}));
 // 使用场景随章节登记；已有规格、候选方案及新增建议分别标记，不代表生产采用。
 const ui='../../../../02-项目核心/UI规范.md';
 const sources={
  tokens:{label:'UI规范 §2.4',href:ui+'#24-尺寸圆角字号动效'},
  controls:{label:'UI规范 §3',href:ui+'#3-基础控件'},
  buttons:{label:'UI规范 §3.1',href:ui+'#31-按钮尺寸使用规则2026-10-15确认'},
  accessibility:{label:'UI规范 §6',href:ui+'#6-键盘与可访问性'},
  material:{label:'UI规范 §26',href:ui+'#26-页面还原与三档材质组件体系2026-10-01-已确认待实施'},
  tables:{label:'UI规范 §16',href:ui+'#16-表格列宽分配与短内容不换行2026-09-24'},
  text:{label:'UI规范 §20',href:ui+'#20-短文本不断行长值行内截断全局2026-09-25'},
  tabs:{label:'UI规范 §28',href:ui+'#28-页面主-tab-到顶固定2026-10-14-确认020-开发调整'},
  notifications:{label:'UI规范 §32',href:ui+'#32-统一通知注册与原型制作约束2026-10-14确认020'},
  windows:{label:'UI规范 §27',href:ui+'#27-windows-原生窗口与菜单2026-10-08'},
  tray:{label:'UI规范 §15',href:ui+'#15-托盘不承载风险结论2026-09-24'},
  brand:{label:'品牌素材规格',href:'../../../04-品牌与图标素材/README.md'},
  picker:{label:'公共模型选择器方案',href:'../公共渠道模型选择器交互方案.md'},
  nodes:{label:'模型协议与服务节点方案',href:'../../../03-项目协作资料/01-协作包/2026-10-04-模型配置管理与同步/01-需求分析/02-方案草案/模型协议与端点方案.md'},
  topic:{label:'组件专题规划',href:'../组件与样式专题规划.md'}
 };
 const rule=(text,source)=>({text,source:sources[source],status:'existing'});
 const proposal=(text,source)=>({text,source:sources[source],status:'proposal'});
 const suggest=(text,source)=>({text,source:sources[source],status:'suggestion'});
 const usageBySection={
  brand:[
   rule('使用现有原色Logo；浅底配深色单色，深底配白色单色，保持图形比例。','brand'),
   suggest('软件标识、关于页与文档页使用Logo加完整名称；32px适合紧凑标识，64px适合常规展示，128px用于品牌核对。','brand'),
   suggest('空间不足时优先调整排列或允许名称换行；不为挤入单行缩到难读。最终展示尺寸按真实场景校准。','topic'),
   rule('Logo、平台应用图标和托盘图标分别取对应素材，不把带底板AppIcon当托盘图标。','brand')
  ],
  'app-icons':[
   rule('macOS使用白色圆角底板AppIcon；Windows使用已确认的透明90%主体版本。','brand'),
   rule('应用启动器、Dock、任务栏和安装包使用所属平台AppIcon；交付格式分别沿ICNS与ICO清单。','brand'),
   suggest('16/32px检查小尺寸轮廓，64/128px核对较大预览；实际导出尺寸以平台交付清单为准。','brand'),
   rule('Windows不附加macOS底板；放大网页预览不等于小尺寸或高DPI原生验收。','brand')
  ],
  'tray-icons':[
   rule('macOS用Template黑/白前景；Windows浅背景配tray-dark，深背景配tray-light。','brand'),
   rule('用于macOS菜单栏和Windows通知区；只使用各自托盘目录中的纯色透明素材。','brand'),
   rule('macOS以18pt及其2×素材核对；Windows重点核对16/20px，并按DPI选择交付档位。','brand'),
   rule('不加底板、渐变或阴影；不可用应用图标缩小替代，状态也不能只靠图标颜色说明。','brand')
  ],
  'icons-spacing':[
   rule('普通图标按钮用32×32点击区，通知内24×24；图形沿14/16/20px令牌，不把图形尺寸当点击区。','buttons'),
   suggest('紧凑行内用14px图形，常规控件用16px，较大操作用20px；间距从已有4/8/12/16/24/32档选择。','tokens'),
   suggest('同组图标统一视觉尺寸；先用间距和排列解决拥挤，触控场景另行核对点击面积。','tokens'),
   rule('图标动作必须有可读名称与焦点；触控44px令牌不等于已有按钮touch规格。','buttons')
  ],
  typography:[
   rule('沿现有文字令牌：正文14、控件13、标签12、辅助11、小字10、分组16、标题20、展示24px。','tokens'),
   suggest('设置说明用正文，操作文案用控件字号；10/11px只承载辅助信息，重要错误使用清晰正文。代码和路径使用代码字体。','tokens'),
   rule('缩放或长内容时允许布局增长；短按钮/状态不拆字，长路径按所在场景提供截断、换行或完整查看。','text'),
   suggest('避免用一串不同字号制造层级；先用标题、字重和间距组织信息，不靠浅色小字承载关键判断。','topic')
  ],
  materials:[
   rule('按shell/page/panel/control/emphasis/overlay表面分类取配方；亮暗主题与高中低画质独立。','material'),
   rule('页面、卡片、控件、强调动作和浮层分别使用对应语义表面；高/中档玻璃，低档统一实底。','material'),
   rule('高档允许装饰运动，中档保留玻璃并静止，低档关闭环境光与玻璃；不改变布局、文字和功能。','material'),
   rule('不逐页调透明度或叠加多层相同模糊；软件.main不加backdrop-filter，不暗中改用户画质。','material')
  ],
  layout:[
   suggest('同一任务连续阅读用纵向排列；同组短操作横向排列并允许整组换行，有目录时使用左导航右正文。','topic'),
   rule('除概览/面板外，有页面主Tab时随滚动到主容器顶边固定；详情标题继续在其下避让。','tabs'),
   rule('按固定条实际高度与正文内距更新锚点位置；窄窗和文字放大后仍保留草稿、范围及滚动语义。','tabs'),
   rule('不把所有筛选/弹窗Tab都固定，也不用全窗口fixed代替容器sticky；不压盖内容或官方内嵌页面。','tabs')
  ],
  buttons:[
   rule('默认md34px；行内sm28px；明确需要更大点击区域才选lg40px。通知内部沿专用24px。','buttons'),
   rule('列表编辑/复制用sm，页面/设置/弹窗常规动作通常md；primary、ghost、danger表达动作语义，独立于尺寸。','buttons'),
   suggest('同一区域通常只强调一个主要动作；有未保存修改时强调保存，取消/辅助动作次级，破坏性动作采用危险样式。','buttons'),
   rule('不把primary自动做成lg，不只缩文字却继承大min-height；状态框与并列按钮等高，放大文字不裁字。','buttons')
  ],
  forms:[
   rule('常规Input/Select使用主控制档与清晰标签，保留帮助、聚焦、只读、禁用和错误状态；Switch沿38×22。','controls'),
   suggest('短文本用Input、长说明用Textarea；独立多选用Checkbox、互斥选项用Radio，即时布尔设置用Switch。','controls'),
   suggest('表单默认34px；高密度行编辑可评估28px，大点击面场景可评估40px。多行字段按内容增长，最终密度随开发验收。','buttons'),
   rule('校验错误留字段附近且能定位焦点，不发送持久通知；不使用禁用外观冒充可读的只读字段。','notifications')
  ],
  'menus-tabs':[
   rule('页面Tab自然宽度、13px文字、36px基准高；选中用底部2px指示。Select/Input沿主控制档。','controls'),
   suggest('选择一个或多个值用Select；执行命令用动作菜单；切同级内容用Tab，不用三个外观相近的控件混做同一件事。','controls'),
   suggest('少量高频动作可直接露出；低频成组动作再用菜单。长选项保留完整可读出口，选项多时核对搜索/滚动需求。','topic'),
   rule('键盘需可聚焦、可选中并能退出；不要为收紧外观把Tab、菜单和树控件套成普通按钮尺寸。','buttons')
  ],
  data:[
   rule('长文本列吃剩余宽度，状态/操作等短列按内容取宽；列表行内动作通常sm28px。','tables'),
   suggest('横向比较多条对象用表格，真实父子关系用树表，简单摘要用列表；状态文字与计数分开呈现。','topic'),
   rule('空间不足时先压缩/截断长内容；状态与动作不拆行，长值仍须有完整查看方式。','tables'),
   rule('不用平均列宽把按钮挤成两行，不只靠颜色判断状态；展开、选择与编辑保持独立职责。','accessibility')
  ],
  overlays:[
   rule('沿公共Modal与浮层宿主；常规弹窗操作通常md34px。打开移入焦点，关闭归还入口，Esc表示取消。','accessibility'),
   suggest('短确认或短表单用Modal，持续阅读和长配置用页面；字段选值用下拉，不把每个局部动作都升级成弹窗。','topic'),
   suggest('弹窗宽度随内容类型与可用空间决定，表格/选择器可更宽；当前不新增一组未经实屏核对的固定宽度档。','topic'),
   rule('遮罩只能取消不能确认，焦点在弹层内循环；表单错误就地展示，不重复创建通知结果卡。','accessibility')
  ],
  feedback:[
   rule('字段错误、空态、加载和步骤留在对应内容区域；短反馈、任务观察与持久通知按不同职责处理。','notifications'),
   suggest('空态给出可开始的动作，失败给出可恢复的下一步；加载保留范围和既有选择，不用清空页面表达等待。','topic'),
   rule('需要跨页观察的事务走任务宿主；需要后续处理的失败、冲突、待重启等结果才按策略进入通知中心。','notifications'),
   rule('不把字段校验、筛选、选择或每个进度步骤都变成未读通知；不只有红/绿色而没有文字说明。','notifications')
  ],
  'model-picker':[
   proposal('默认弹窗形态，完整显示左渠道、右模型和底部已选；单选/多选及推理/Fast由调用场景决定。','picker'),
   proposal('Agent执行模型或其他模型字段复用同一选择器；页内形态用于需要持续浏览的完整选择区。','picker'),
   proposal('单选替换唯一待选项，多选保持跨搜索选择与优先级；推理/Fast可连续编辑，统一确认才返回。','picker'),
   proposal('未知能力不猜测，失效引用解释原因；选择器不自动保存、启用渠道或改模型白名单，取消保留原值。','picker')
  ],
  'node-picker':[
   proposal('列表用独立服务节点列和单行选择器；下拉集成当前节点、逐项检测、一键检测及结果。','nodes'),
   proposal('快捷切节点或观察响应耗时用列表浮层；维护URL、当前节点和逐项/全部测速用详情服务节点区。','nodes'),
   proposal('同一渠道节点只换URL，认证与Key共用；切节点进入全渠道草稿，测速结果按会话和配置版本失效。','nodes'),
   proposal('测速不自动选最快节点，不用模型生成做测速；毫秒耗时不代表模型能力/额度或自动故障转移。','nodes')
  ],
  notifications:[
   rule('Toast、任务卡和中心复用既有宿主，内部操作沿通知专用24px，不套常规md/lg按钮。','controls'),
   rule('短操作结果用短反馈/日志，过程用任务观察；需要后续处理的结果按登记策略入中心。','notifications'),
   rule('长任务按operation/task身份更新；收起只改视图，可从铃铛恢复，已读/解决/删除分别记录。','notifications'),
   rule('先登记再发布，不直接拼卡或维护专用宿主；不让每一步产生未读，也不覆盖别的任务。','notifications')
  ],
  windows:[
   rule('Windows采用单行标题栏：左图标/名称，右系统窗口按钮；保留系统拖拽、缩放、Snap与关闭语义。','windows'),
   rule('应用顶层Windows窗口使用平台标题栏；组件专题本身是独立文档工作台，窗口只作为此处样例。','windows'),
   rule('Win11以真实DWM Mica为目标；Win10、透明关闭、高对比、非活动或能力失败回退实色。','windows'),
   rule('系统窗口按钮不套28/34/40普通动作规格；网页blur不等于Mica，模拟最大化不等于原生验证。','windows')
  ],
  tray:[
   rule('沿平台原生托盘菜单，顶部只展示进程事实和适用地址；图标来自对应平台托盘素材。','tray'),
   rule('放启动/停止、打开主界面/面板/诊断等快捷入口；详细风险和恢复依据回到应用内展示。','tray'),
   rule('动作可用性跟随进程事实，忙碌/不可用时解释原因；菜单尺寸由平台决定，浏览器仅作示意。','tray'),
   rule('不在标题拼长风险说明，也不套普通按钮尺寸；托盘不承担完整设置表单或业务详情。','tray')
  ],
  examples:[
   rule('组合沿公共配方：设置字段主控制档、常规动作md34、行内动作sm28；状态框与并列按钮等高。','buttons'),
   suggest('设置表单用来核对标签/帮助/输入/保存关系；模型操作条用来核对预览、状态、还原和保存的优先级。','topic'),
   suggest('内容变长或窗口变窄时先调整排列和可用宽度，控件仍沿公共尺寸；未保存时强调保存，保存后回到稳定状态。','topic'),
   rule('组合容器只负责布局，不重写子组件字号/内距/玻璃；示例完成不代表业务迁移或生产验收。','material')
  ]
 };
 const usageLabels=['默认规格','适用场景','切换条件','避免误用'];
 const sections=[
 ['brand','Logo与标识','foundations','品牌原色、单色与亮暗背景'],['app-icons','应用图标','foundations','macOS底板与Windows透明图标'],['tray-icons','托盘图标','foundations','macOS Template / Windows亮暗单色'],['icons-spacing','图标与间距','foundations','可见图标、点击区域与尺度'],
 ['typography','字体与内容','foundations','中英文、数字、长路径与代码'],['materials','颜色与材质','foundations','语义表面、亮暗与三档效果'],['layout','布局与锚点','layout','分栏、换行与到顶固定'],['buttons','按钮尺寸','controls','小号28 / 标准34 / 大号40'],['forms','输入与选择','controls','字段、校验、只读与禁用'],['menus-tabs','菜单与分区','controls','单多选、键盘和浮层'],['data','表格与树表','data','长列表、勾选与展开'],['overlays','弹窗与注释','overlays','确认、详情、表单与焦点'],['feedback','就地反馈','overlays','空态、加载、错误与步骤'],['model-picker','渠道模型选择器','compound','同一组件的型号、推理与Fast选择'],['node-picker','服务节点选择','compound','选择、逐节点检测与一键检测'],['notifications','通知与任务','overlays','登记事件、收起恢复与生命周期'],['windows','Windows窗口','platform','标题栏与关闭行为示意'],['tray','托盘菜单','platform','macOS / Windows状态菜单'],['examples','完整组合','layout','设置表单与模型操作条']
 ].map(x=>({id:x[0],title:x[1],group:x[2],description:x[3],renderer:x[0],anchor:'component-'+x[0],usage:(usageBySection[x[0]]||[]).map((item,i)=>({...item,label:usageLabels[i]}))}));
 const rows=[
 ['brand.logo','Logo与标识','brand','brand',null],['brand.app-macos','macOS应用图标','app-icons','brand',null],['brand.app-windows','Windows应用图标','app-icons','brand',null],['brand.tray-macos','macOS托盘图标','tray-icons','brand',null],['brand.tray-windows','Windows托盘图标','tray-icons','brand',null],
 ['type.scale','字号与字体','typography','tokens','styles/tokens.css'],['content.markdown','代码与富文本','typography','pattern','components/patterns/MarkdownContent.vue'],['surface.recipe','语义材质','materials','tokens','styles/materials.css'],['appearance.effects','画质与主题','materials','policy','app/appearance/effects.ts'],['icon.scale','图标与点击区域','icons-spacing','tokens',null],['layout.stack','纵向排列','layout','layout','components/layout/Stack.vue'],['layout.inline','横向排列','layout','layout','components/layout/Inline.vue'],['layout.split','分栏与响应式','layout','layout',null],['navigation.anchor','锚点与固定条','layout','layout',null],['button.text','文字按钮','buttons','ui','components/ui/UiButton.vue'],['button.icon','图标按钮','buttons','ui',null],['field.input','输入字段','forms','ui',null],['field.textarea','多行输入','forms','ui',null],['choice.checkbox','复选/半选','forms','ui',null],['choice.radio','单选','forms','ui',null],['choice.switch','开关','forms','ui','components/ui/UiSwitch.vue'],['choice.slider','滑杆','forms','ui',null],['choice.select','选择与下拉','menus-tabs','ui','components/ui/UiSelectMenu.vue'],['menu.actions','动作菜单','menus-tabs','ui',null],['navigation.tabs','Tab分区','menus-tabs','ui',null],['data.table','数据表','data','pattern',null],['data.tree','树表','data','ui','components/ui/UiTreeTable.vue'],['data.status','状态标签','data','ui',null],['overlay.dialog','Dialog/Modal','overlays','ui',null],['overlay.annotation','原型注释','overlays','dev',null],['feedback.local','空态/校验/加载','feedback','pattern',null],['feedback.steps','步骤与进度','feedback','pattern',null],['model.selection','模型选择器','model-picker','pattern',null],['node.selection','节点与检测','node-picker','feature',null],['feedback.toast','短结果反馈','notifications','host','components/ToastHost.vue'],['feedback.task','任务卡','notifications','host','components/TaskProgressHost.vue'],['feedback.notification','通知中心条目','notifications','feature','features/notifications/components/NotificationItem.vue'],['platform.caption','Windows标题栏','windows','platform',null],['platform.tray','托盘菜单','tray','platform',null],['pattern.setting','设置行','examples','pattern','components/patterns/SettingRow.vue'],['pattern.card','卡片','examples','ui','components/ui/UiCard.vue'],['pattern.card-header','卡片标题','examples','ui','components/ui/UiCardHeader.vue'],['overlay.modal-host','应用Modal宿主','overlays','host','components/AppModal.vue']
 ];
 const special={ 'choice.switch':{sizes:['default'],states:['off','on','focus','disabled','busy'],productionRevision:null,status:{registered:true,prototype:true,production:true,adopted:'0.2.0设置页14处共用UiSwitch',verified:'浏览器亮暗×三档尺寸与键盘通过；完整矩阵/原生待验'}}, 'brand.logo':{sizes:[32,64,128],states:['original','mono-light','mono-dark']},'brand.app-macos':{platforms:['macos'],sizes:[16,32,64,128],states:['default']},'brand.app-windows':{platforms:['windows'],sizes:[16,32,64,128],states:['transparent']},'brand.tray-macos':{platforms:['macos'],sizes:[16,18,24,32],states:['template-dark','template-white']},'brand.tray-windows':{platforms:['windows'],sizes:[16,20,24,32],states:['mono-dark','mono-light']},'field.input':{sizes:['sm','md','lg'],states:['default','focus','readonly','disabled','error']},'field.textarea':{sizes:['md'],states:['default','focus','disabled','error']},'choice.select':{variants:['single','multiple'],states:['default','open','selected','disabled']},'menu.actions':{states:['closed','open','disabled','keyboard-focus']},'navigation.tabs':{states:['selected','keyboard-focus']},'data.status':{states:['neutral','success','warning','danger','count']},'button.text':{sizes:['sm','md','lg'],variants:['default','primary','ghost','danger'],states:['default','hover','active','focus','disabled','loading']},'model.selection':{sizes:['dialog','inline'],variants:['single','multiple'],states:['normal','empty','loading','failed','missing','unknown-effort']},'node.selection':{states:['untested','running','success','failed','stopped']},'feedback.notification':{states:['unread','read','resolved','deleted']},'platform.caption':{platforms:['win11','win10'],states:['active','inactive','maximized','minimized','tray']},'platform.tray':{platforms:['macos','windows'],states:['stopped','running','starting','failed']}};
 const assetSources={
  'brand.logo':['04-品牌组件/Logo/logo.svg','04-品牌组件/Logo/logo-light.svg','04-品牌组件/Logo/logo-dark.svg'],
  'brand.app-macos':['03-平台交付/macOS/AppIcon/AppIcon.svg'],
  'brand.app-windows':['03-平台交付/Windows/AppIcon/AppIcon.svg'],
  'brand.tray-macos':['03-平台交付/macOS/Tray/tray-template.svg','03-平台交付/macOS/Tray/tray-white.svg'],
  'brand.tray-windows':['03-平台交付/Windows/Tray/tray-dark.svg','03-平台交付/Windows/Tray/tray-light.svg']
 };
 const components=rows.map(x=>({id:x[0],name:x[1],section:x[2],layer:x[3],spec:'../../../../02-项目核心/UI规范.md',owner:x[3]==='brand'?'OpenCodex品牌素材':'OpenCodex公共UI',assets:(assetSources[x[0]]||[]).map(path=>'../../../04-品牌与图标素材/'+path),version:x[3]==='brand'?'既有品牌素材': ['model.selection','node.selection'].includes(x[0])?'0.2.0':x[3]==='platform'?'既有平台范围':'0.2.0公共界面',prototype:'component-topic.js',renderer:x[2],fixture:'component-topic.js#state',consumers:{prototype:['component-topic.js#'+x[2]],production:'待逐入口核对'},density:x[0]==='button.text'?['default','compact']:[],implementation:x[4],productionRevision:'eae85322ebca3b755c034db97cbebf2ea5bf71fc',status:{registered:true,prototype:true,production:!!x[4],adopted:'待逐消费者核对',verified:'原型源码/DOM检查；实屏待验'},themes:['light','dark'],effects:['high','mid','low'],sizes:[],variants:[],states:['tokens','policy','layout'].includes(x[3])?['default']:['default','disabled','error'],platforms:['macos','windows'],...(special[x[0]]||{})}));
 function validate(){const ids=new Set(),anchors=new Set();for(const s of sections){if(ids.has(s.id)||anchors.has(s.anchor)||!groups.some(g=>g.id===s.group))throw Error('组件章节登记无效');if(s.usage.length!==usageLabels.length||s.usage.some((u,i)=>u.label!==usageLabels[i]||!u.text||!['existing','proposal','suggestion'].includes(u.status)||!u.source?.label||!u.source?.href))throw Error('组件使用建议登记无效：'+s.id);ids.add(s.id);anchors.add(s.anchor);}const componentsIds=new Set();for(const c of components){if(componentsIds.has(c.id)||!ids.has(c.section)||!c.owner||!c.spec)throw Error('组件登记无效');componentsIds.add(c.id);}return true;}
 function section(id){return sections.find(s=>s.id===id);}
 function entries(id){return components.filter(c=>c.section===id);}
 function freeze(v){if(v&&typeof v==='object'){Object.values(v).forEach(freeze);Object.freeze(v);}return v;}validate();for(const collection of [groups,sections,components])freeze(collection);return Object.freeze({groups:Object.freeze(groups),sections:Object.freeze(sections),components:Object.freeze(components),section,entries,validate});
});
