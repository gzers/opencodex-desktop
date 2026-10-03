// 原型「扩展管理 → Skills」点击「名称 + 描述」单元格查看完整信息：行为测试（jsdom）。
// 运行：node "<本文件>"
// 覆盖：单元格可点/可聚焦、弹窗内容完整（描述不截断 + 元数据 + 6 个同步目标）、
// 只读（只有关闭）、Esc / 关闭按钮 / Enter / 空格、行内按钮不误触、搜索过滤不受影响。
import { createRequire } from 'module';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// 从脚本位置向上找到仓库根（含 .git），避免写死绝对路径。
function findRepo(start){
  let d=start;
  for(let i=0;i<12;i++){
    if(fs.existsSync(path.join(d,'.git'))) return d;
    const up=path.dirname(d);
    if(up===d) break;
    d=up;
  }
  return null;
}
const here=path.dirname(fileURLToPath(import.meta.url));
const repo=findRepo(here);
if(!repo){ console.log('BLOCKED: 未找到仓库根（.git）。'); process.exit(2); }
const protoDir=path.join(here,'..');
const proto=path.join(protoDir,'index.html');
// jsdom 装在 apps/desktop/ui 下；ESM 的 NODE_PATH 不生效，故显式从该目录解析。
const require=createRequire(path.join(repo,'apps/desktop/ui/'));
const { JSDOM }=require('jsdom');

const html=fs.readFileSync(proto,'utf8');

const errors=[];
const dom=new JSDOM(html,{runScripts:'dangerously',pretendToBeVisual:true,url:'file:///proto/index.html#extensions',beforeParse(win){
  win.matchMedia=win.matchMedia||(()=>({matches:false,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}}));
  win.addEventListener('error',e=>errors.push('window error: '+(e.error&&e.error.stack||e.message)));
  // jsdom 对 file:// 的 history.replaceState 抛 SecurityError（原型初始化与路由都用它）。
  win.__nav=[];
  for(const fn of ['replaceState','pushState']){
    const orig=win.history[fn].bind(win.history);
    win.history[fn]=(state,title,url)=>{ win.__nav.push(String(url||'')); try{ orig(state,title,url); }catch{ /* file:// 下被 jsdom 拒绝 */ } };
  }
}});
const {window}=dom;
const doc=window.document;
window.addEventListener('error',e=>errors.push('err: '+(e.error&&e.error.stack||e.message)));

const log=[];const assert=(c,m)=>log.push((c?'PASS':'FAIL')+' '+m);

const rows=()=>[...doc.querySelectorAll('#skillsList .skill-row')];
const infos=()=>[...doc.querySelectorAll('#skillsList .skill-info[data-ext-detail]')];
const maskShown=()=>doc.getElementById('modalMask').style.display==='flex';
const mTitle=()=>doc.getElementById('modalTitle').textContent.trim();
const mBody=()=>doc.getElementById('modalBody').textContent;
const targets=()=>[...doc.querySelectorAll('#modalBody .skill-detail-targets li')];
const md=()=>doc.querySelector('#modalBody .skill-detail-md');
const click=el=>el.dispatchEvent(new window.MouseEvent('click',{bubbles:true}));
const press=(el,key)=>el.dispatchEvent(new window.KeyboardEvent('keydown',{key,bubbles:true}));
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
// 键盘事件派发到元素上（真实浏览器里 keydown 的 target 始终是获得焦点的元素），
// 由它冒泡到 document，与真实交互一致。
const esc=()=>press(doc.body,'Escape');

// 1. 结构：6 行 Skills 都可点开，MCP 不受影响
assert(rows().length===6,'Skills 列表为 6 行');
assert(infos().length===6,'每行「名称 + 描述」单元格都能打开详情');
assert(infos().every(el=>el.getAttribute('role')==='button'),'单元格 role=button');
assert(infos().every(el=>el.getAttribute('tabindex')==='0'),'单元格可键盘聚焦');
assert(infos().every(el=>/^查看 .+ 完整信息$/.test(el.getAttribute('aria-label')||'')),'单元格具有无障碍名称');
assert(infos().every(el=>!el.textContent.includes('查看')),'单元格不加「查看…」之类的冗余提示文字');
assert(doc.querySelectorAll('#mcpList .skill-info[data-ext-detail]').length===1,'MCP 行用同一套机制（node_repl 可点开详情）');
assert(html.includes('.skill-info[data-ext-detail]{cursor:pointer}'),'悬停指针提示已定义');
assert(html.includes('.skill-info[data-ext-detail]:focus-visible{outline:none}'),'连浏览器默认聚焦环也去掉（不留任何外框）');
assert(!html.includes('.skill-info[data-ext-detail]:hover{'),'悬停不加边框 / 底色之类的块级特效');
assert(html.includes('.skill-info[data-ext-detail]:hover p,'),'悬停时提亮描述文字');
assert(html.includes('.skill-info[data-ext-detail]:hover .skill-meta,'),'悬停时提亮版本 / 更新时间的文字');
assert(html.includes('.skill-info[data-ext-detail]:focus-visible p,'),'键盘聚焦用同一套提亮（不额外加外框）');
assert(!html.includes('text-decoration:underline'),'不使用标题下划线');
assert(html.includes('.skill-info p.is-clamped{'),'截断描述用底部渐隐作提示');
assert(html.includes('mask-image:linear-gradient(to bottom'),'渐隐实现为遮罩而非叠色块');
assert(typeof window.markClampedDescriptions==='function','有按需标记截断描述的入口');
assert((doc.querySelectorAll('#skillsList .skill-info p').length)===6,'只在 Skills 行内计算截断（6 条描述）');
// 「截断才渐隐 / 短描述不渐隐」依赖真实排版，jsdom 无布局，放到浏览器脚本里验。

// 2. 点击打开
assert(!maskShown(),'初始未打开弹窗');
click(infos()[0]);
assert(maskShown(),'点击单元格打开弹窗');
assert(mTitle()==='design-studio','弹窗标题为 Skill 名称');
assert(doc.querySelector('#modalBody .skill-detail-name')===null,'名称由弹窗标题承载，正文不重复');

// 3. 完整信息（描述不截断 + 元数据 + 目标）
const body=mBody();
assert(body.includes('创作表达与自然度校准（去 AI 味、说人话、自然一点、别像模板）'),'展示完整描述（列表里只露两行，弹窗里全都在）');
assert(body.includes('跨项目、跨 agent 使用的全局设计 skill'),'超长描述的第二段也在弹窗里');
// 描述面板：外框 + 加深底色 + 独立滚动 + Markdown 渲染
assert(html.includes('.skill-detail-md{flex:1 1 auto;min-height:0;overflow:auto;'),'描述面板自带滚动并由它吸走剩余高度');
assert(html.includes('background:var(--rail)')&&html.includes('border:1px solid var(--border);border-radius:var(--radius-sm);background:var(--rail)'),'描述面板有外框与加深底色');
assert(!!md(),'描述用独立面板承载');
assert(doc.querySelectorAll('#modalBody .skill-detail-md h4').length>=3,'Markdown 标题已渲染');
assert(doc.querySelectorAll('#modalBody .skill-detail-md ol>li').length===4,'有序列表已渲染（先做什么 4 条）');
assert(doc.querySelectorAll('#modalBody .skill-detail-md ul>li').length>=5,'无序列表已渲染');
assert(!!doc.querySelector('#modalBody .skill-detail-md blockquote'),'引用块已渲染');
assert(!!doc.querySelector('#modalBody .skill-detail-md pre code'),'围栏代码块已渲染');
assert(doc.querySelectorAll('#modalBody .skill-detail-md code').length>=4,'行内代码已渲染');
assert(!!doc.querySelector('#modalBody .skill-detail-md strong'),'加粗已渲染');
assert(md().querySelectorAll('script,img,iframe').length===0,'Markdown 渲染不产生可执行标签');
assert(doc.querySelector('#modalBody .skill-detail-desc')===null,'旧的纯文本描述节点已被面板替换');
assert(doc.querySelector('#modalBody .modal-prototype-note')===null,'详情弹窗不再显示「原型演示」脚注');
// 渲染器安全：HTML 先被转义；链接只留文字（原型不外链、不注入 href）
const hostile=window.renderSkillMarkdown('<img src=x onerror="alert(1)"> **加粗**');
assert(!hostile.includes('<img')&&hostile.includes('&lt;img'),'描述里的 HTML 被转义，不会当作标签执行');
assert(hostile.includes('<strong>加粗</strong>'),'转义之后仍能套 Markdown 标记');
const linked=window.renderSkillMarkdown('[点我](javascript:alert(1)) 与 [官网](https://example.com)');
assert(linked.includes('点我')&&linked.includes('官网'),'链接保留文字');
assert(!linked.includes('javascript:')&&!linked.includes('href'),'不外链、不生成 href');
assert(window.renderSkillMarkdown('')==='' ,'空描述渲染为空，不报错');
assert(doc.querySelectorAll('#modalBody .skill-detail-label').length===0,'详情弹窗不再有任何区块小标题');
assert(!html.includes('skill-detail-label'),'小标题的样式与用法已一并删除');
assert(doc.querySelector('#modalBody').firstElementChild.classList.contains('skill-detail-doc'),'描述面板是正文第一块（说明排在最前）');
assert(doc.querySelector('#modal').classList.contains('modal-wide'),'长描述使用加宽弹窗');
assert(body.includes('本地'),'展示来源标记');
assert(body.includes('v1.8.0'),'展示版本');
assert(body.includes('更新时间')&&body.includes('2026-09-13'),'展示更新时间');
assert(body.includes('~/.agents/skills/design-studio'),'展示源目录');
assert(body.includes('SKILL.md'),'展示入口文件');
assert(body.includes('28 个文件')&&body.includes('412 KB'),'展示文件数与体量');
assert(doc.querySelector('#modalBody .skill-detail-head')===null,'来源 / 版本 / 更新时间已并入信息区，不再单占一行');
assert(!body.includes('安装来源'),'不再单列与「来源」重复的安装来源');
const factRows=[...doc.querySelectorAll('#modalBody .skill-detail-facts>div')];
assert(factRows.length===7,'信息区恰为 7 项（来源 / 版本 / 更新时间 / 入口文件 / 文件数 / 体量 / 源目录）');
assert(factRows.filter(el=>el.classList.contains('span-2')).length===1,'只有源目录占整行（路径较长）');
assert(doc.querySelector('#modalBody .modal-facts')===null,'信息区不套外框（不再用带边框的 .modal-facts）');
assert(targets().length===6,'展示 6 个同步目标');
assert(targets().filter(li=>li.classList.contains('on')).length===3,'已同步数量与行内开关一致（design-studio 为 3）');
assert(targets().filter(li=>li.classList.contains('on')).every(li=>li.textContent.includes('已同步')),'已同步目标用中文标注');
assert(targets().filter(li=>!li.classList.contains('on')).every(li=>li.textContent.includes('未同步')),'未同步目标用中文标注');
assert(targets().every(li=>li.querySelector('svg use')),'每个目标带客户端图标');

// 3b. 标题右侧的动作按钮
const headActions=()=>doc.getElementById('modalHeadActions');
const headButtons=()=>[...headActions().querySelectorAll('button')];
assert(!headActions().hidden&&headButtons().length===2,'详情弹窗标题右侧有 2 个图标动作');
assert(headButtons()[0].getAttribute('title')==='检查 / 更新','第一个是「检查 / 更新」');
assert(headButtons()[1].getAttribute('title')==='卸载'&&headButtons()[1].classList.contains('danger'),'第二个是「卸载」，用危险色');
assert(headButtons().every(b=>/design-studio/.test(b.getAttribute('aria-label')||'')),'动作按钮的无障碍名称指向当前 Skill');

// 3c. 目标格可点击切换「同步 / 关闭同步」
const cell=()=>targets()[0].querySelector('.skill-detail-target');
assert(cell().tagName==='BUTTON'&&cell().getAttribute('aria-pressed')==='true','目标格是可切换按钮，初始为已同步');
click(cell());
assert(cell().getAttribute('aria-pressed')==='false'&&!targets()[0].classList.contains('on'),'点击后切成未同步');
assert(cell().querySelector('em').textContent.trim()==='未同步','状态文案随之更新');
assert(rows()[0].querySelectorAll('.skill-targets .target-icon')[0].getAttribute('aria-checked')==='false','列表那一行的开关一起写回');
assert((cell().getAttribute('aria-label')||'').includes('点击同步'),'无障碍名称随状态更新');
click(cell());
assert(cell().getAttribute('aria-pressed')==='true'&&cell().querySelector('em').textContent.trim()==='已同步','再点一次切回已同步');
assert(rows()[0].querySelectorAll('.skill-targets .target-icon')[0].getAttribute('aria-checked')==='true','列表开关同样切回');
assert(targets().filter(li=>li.classList.contains('on')).length===3,'切换两下后回到 3 个已同步');

// 3d. 标题动作按当前 Skill 生效：更新就地执行（不换窗），卸载点名当前 Skill
const metaOf=row=>[...row.querySelectorAll('.skill-meta span')].find(item=>item.textContent.trim().startsWith('更新'))?.textContent.trim();
const modalUpdated=()=>{
  const cell=[...doc.querySelectorAll('#modalBody .skill-detail-facts>div')].find(item=>(item.querySelector('b')||{}).textContent==='更新时间');
  return cell?cell.querySelector('span').textContent.trim():'';
};
const updInfos=infos();
click(updInfos[1]);
assert(mTitle()==='development-knowledge'&&modalUpdated()==='2026-09-12','打开可更新 Skill 的详情');
click(headButtons()[0]);
assert(maskShown()&&mTitle()==='development-knowledge'&&!headActions().hidden,'标题「检查 / 更新」不换窗，就地执行当前 Skill');
assert(rows()[1].querySelector('[data-prototype-action="skills-update"]').classList.contains('is-busy'),'进行中状态表现在按钮上');
await sleep(1100);
assert(modalUpdated()==='2026-09-23','详情弹窗的更新时间随更新一起刷新');
assert(metaOf(rows()[1])==='更新 2026-09-23','列表那一行也刷新了更新时间');
assert(!rows()[1].querySelector('[data-prototype-action="skills-update"]').classList.contains('is-busy'),'结束后按钮恢复');
click(headButtons()[1]);
assert(mTitle()==='卸载 Skill'&&mBody().includes('development-knowledge')&&mBody().includes('警告'),'标题「卸载」点名当前 Skill 并保留警告样式');
assert(headActions().hidden,'卸载弹窗标题右侧没有详情动作按钮');
assert(doc.querySelector('#modalBody .modal-prototype-note')!==null,'卸载弹窗保留「原型演示」脚注');
doc.getElementById('modalCancel').click();
assert(!maskShown(),'关闭卸载弹窗后回到列表');
// 后面的「只读」「关闭方式」等断言仍针对 design-studio 的详情弹窗。
click(infos()[0]);

// 4. 只读：只有一个关闭动作
assert(doc.getElementById('modalConfirm').hidden,'底部不提供确认按钮（只读弹窗）');
assert(doc.getElementById('modalCancel').textContent.trim()==='关闭','底部动作为「关闭」');
assert(!mBody().includes('当前为原型演示'),'详情弹窗不显示「原型演示」脚注（用户 2026-09-23 要求去掉）');

// 5. 关闭方式
esc();
assert(!maskShown(),'Esc 关闭弹窗');
click(infos()[2]);
assert(maskShown()&&mTitle()==='hatch-pet','点击第 3 行打开对应 Skill');
doc.getElementById('modalCancel').click();
assert(!maskShown(),'「关闭」按钮关闭弹窗');
assert(!doc.querySelector('#modal').classList.contains('modal-wide'),'关闭后收起加宽样式，不影响其它弹窗');

// 6. 键盘等价（role=button 需 Enter / 空格）
press(infos()[5],'Enter');
assert(maskShown()&&mTitle()==='skill-audit','Enter 打开详情');
esc();
press(infos()[1],' ');
assert(maskShown()&&mTitle()==='development-knowledge','空格打开详情');
esc();
assert(!maskShown(),'键盘打开后仍可 Esc 关闭');

// 7. 不误触：行内同步开关与行内操作按钮不打开详情
click(rows()[0].querySelector('.skill-targets .target-icon'));
assert(!maskShown(),'点击行内同步开关不打开详情弹窗');
click(rows()[0].querySelector('.row-actions .icon-btn'));
// 行内「检查 / 更新」是就地执行，既不弹详情也不弹全量窗。
assert(!maskShown(),'行内「检查 / 更新」不打开任何弹窗');
assert(rows()[0].querySelector('[data-prototype-action="skills-update"]').classList.contains('is-busy'),'行内更新进入进行中状态');
await sleep(1100);
assert(!maskShown(),'行内更新结束后仍无弹窗');
assert(doc.querySelector('#modalHeadActions').hidden,'列表状态下标题动作区为空');

// 7b. 顶部「检查更新」仍是全部 Skill 的检查（保留列明细弹窗）
click(doc.querySelector('[data-prototype-action="skills-rescan"]'));
assert(maskShown()&&mTitle()==='检查更新','顶部检查更新仍打开全量弹窗');
assert(mBody().includes('3 个 Skill 可更新')&&mBody().includes('development-knowledge'),'弹窗列出可更新的 Skill');
assert(doc.querySelector('#modalBody .modal-prototype-note')!==null,'全量弹窗保留「原型演示」脚注（hideNote 只作用于详情）');
assert(doc.getElementById('modalHeadActions').hidden,'全量弹窗标题右侧没有详情动作按钮');
doc.getElementById('modalCancel').click();
assert(!maskShown(),'关闭行内弹窗后无残留');

// 8. 6 个 Skill 都能取到元数据（无「—」回退）
const empty=[];
for(const info of infos()){
  const name=info.querySelector('.skill-title h3').textContent.trim();
  click(info);
  const text=mBody();
  if(mTitle()!==name||!text.includes('个文件')||text.includes('—')) empty.push(name);
  doc.getElementById('modalCancel').click();
}
assert(empty.length===0,'6 个 Skill 的详情都有元数据'+(empty.length?'（缺失：'+empty.join('、')+'）':''));

// 9. 搜索与分类筛选必须真正生效：未命中行隐藏，计数收敛（此前的「只改计数不隐藏行」缺陷已修）
const search=doc.getElementById('skillsSearch');
search.value='hatch';search.dispatchEvent(new window.Event('input',{bubbles:true}));
assert(doc.querySelector('[data-pagination="skills"] [data-page-summary]').textContent.includes('1 / 6 条'),'搜索后分页计数收敛为 1 / 6 条');
assert([...doc.querySelectorAll('#skillsList .skill-row')].filter(row=>!row.hidden).length===1,'搜索后只显示命中的 1 行，其余行隐藏');
assert(infos()[0].getAttribute('data-ext-detail')===''||infos()[0].hasAttribute('data-ext-detail'),'过滤后单元格仍保留详情入口');
// 分类 chip 与搜索共用同一条过滤路径：切到 Claude 后只显示已同步到 Claude 的行。
search.value='';search.dispatchEvent(new window.Event('input',{bubbles:true}));
assert([...doc.querySelectorAll('#skillsList .skill-row')].filter(row=>!row.hidden).length===6,'清空搜索后 6 行全部显示');
search.value='';search.dispatchEvent(new window.Event('input',{bubbles:true}));
assert(doc.querySelector('[data-pagination="skills"] [data-page-summary]').textContent.includes('6 / 6 条'),'清空搜索后分页计数恢复 6 条');

// 10. MCP 列表用同一套详情机制（结构与 Skills 一致，只是信息字段与标题动作不同）
const mcpRow=()=>doc.querySelector('#mcpList .skill-row');
const mcpInfo=()=>doc.querySelector('#mcpList .skill-info[data-ext-detail]');
assert(!!mcpInfo()&&mcpInfo().getAttribute('role')==='button'&&mcpInfo().getAttribute('tabindex')==='0','MCP 条目的名称 + 描述单元格也可点开详情');
assert(/^查看 node_repl 完整信息$/.test(mcpInfo().getAttribute('aria-label')||''),'MCP 单元格有无障碍名称');
click(mcpInfo());
assert(maskShown()&&mTitle()==='node_repl','MCP 详情弹窗标题为条目名');
assert(doc.querySelectorAll('#modalBody .skill-detail-label').length===0,'MCP 详情同样没有任何区块小标题');
assert(doc.querySelector('#modalBody .modal-prototype-note')===null,'MCP 详情同样不显示原型脚注');
const mcpBody=mBody();
assert(mcpBody.includes('类型')&&mcpBody.includes('stdio')&&mcpBody.includes('命令')&&mcpBody.includes('node'),'信息区按 MCP 语义给字段（类型 / 命令）');
assert(mcpBody.includes('--input-type=module')&&mcpBody.includes('mcp_servers'),'展示参数与配置落点');
assert(!mcpBody.includes('源目录')&&!mcpBody.includes('文件数'),'不再套用 Skills 的源目录 / 文件数字段');
assert(doc.querySelectorAll('#modalBody .skill-detail-facts>div').length===7,'MCP 信息区恰为 7 项');
const envRow=[...doc.querySelectorAll('#modalBody .skill-detail-facts>div')].find(item=>item.querySelector('b').textContent==='环境变量');
assert(!!envRow&&envRow.classList.contains('span-2'),'环境变量单独占整行');
assert(!!envRow.querySelector('span.scrollable'),'环境变量取值区自带内部滚动');
assert(envRow.textContent.includes('CODEX_HOME'),'环境变量只列键名');
assert(!!doc.querySelector('#modalBody .skill-detail-md pre code'),'MCP 详情用 Markdown 渲染出配置代码块');
assert((doc.querySelector('#modalBody .skill-detail-md pre code').textContent||'').includes('"type": "stdio"'),'代码块是完整 JSON 配置');
assert(doc.querySelectorAll('#modalBody .skill-detail-md ul>li').length>=2,'配置说明渲染为列表');
// MCP 的标题动作是行内已有的「编辑 / 删除」
const mcpHead=()=>[...headActions().querySelectorAll('button')];
assert(!headActions().hidden&&mcpHead().length===2,'MCP 详情标题右侧有 2 个图标动作');
assert(mcpHead().map(b=>b.getAttribute('title')).join(',')==='编辑,删除','动作是「编辑」与「删除」');
assert(mcpHead()[1].classList.contains('danger'),'「删除」用危险色');
// 目标格切换对 MCP 同样生效，并写回列表那一行
assert((await Promise.resolve(),doc.querySelectorAll('#modalBody .skill-detail-targets li')).length===6,'MCP 详情展示 6 个同步目标');
const mcpCell=()=>doc.querySelector('#modalBody .skill-detail-target');
assert(mcpCell().getAttribute('aria-pressed')==='false','node_repl 初始只在 Codex 上同步（第 1 格为未同步）');
click(mcpCell());
assert(mcpCell().getAttribute('aria-pressed')==='true','点击后切成已同步');
assert(mcpRow().querySelectorAll('.skill-targets .target-icon')[0].getAttribute('aria-checked')==='true','写回 MCP 列表那一行的开关');
// 删除动作点名当前条目
click(mcpHead()[1]);
assert(mTitle()==='删除 MCP'&&mBody().includes('node_repl')&&mBody().includes('警告'),'「删除」打开点名当前条目的警告弹窗');
doc.getElementById('modalCancel').click();
assert(!maskShown(),'关闭 MCP 删除弹窗');
// 行内删除同样点名当前条目
click(mcpRow().querySelector('.row-actions .icon-btn.danger'));
assert(mTitle()==='删除 MCP'&&mBody().includes('node_repl'),'行内「删除」也点名当前条目（不再写死）');
doc.getElementById('modalCancel').click();

console.log(log.join('\n'));
const fails=log.filter(l=>l.startsWith('FAIL'));
console.log('\n== 捕获的 JS 错误 ==\n'+(errors.join('\n')||'(none)'));
console.log('\nTOTAL: '+log.length+'  FAIL: '+fails.length+'  JS_ERRORS: '+errors.length);
process.exit(fails.length||errors.length?1:0);
