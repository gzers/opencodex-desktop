/* 仅在 VM 内暴露真实内部函数；不写产品源码，不模拟已删除的导入弹窗。
 * fixture 用于构造差异/解析场景，不作为用户文件导入流程的验证证据。 */
import fs from 'node:fs';
import assert from 'node:assert/strict';
export function diagnosticSource(source){
 const text=fs.readFileSync(source,'utf8'),marker=' window.modelPrototype={';
 assert.equal(text.split(marker).length,2,'唯一原型导出位置');
 return text.replace(marker,`
 window.__sourceDiagnostics={
  applyPolicyFixture:part=>change(()=>draft=mergePolicy(draft,part)),
  portable:()=>clone(portablePolicy(draft)),
  baseDigest,templateError,
  seedProposalFixture:content=>{proposal={id:templateId,baseDigest:editorDigest,baseContent:clone(templateContent()),rules:clone(content.rules||{}),match:clone(content.match||templateContent().match),variants:clone(content.variants),locked:true};}
 };
`+marker);
}
