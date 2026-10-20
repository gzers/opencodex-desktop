import {describe,expect,it} from 'vitest'
import {mount} from '@vue/test-utils'
import {emptyContent,emptyRules,type Revision,type Template,type TemplateDraft} from '@/features/models/api'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import TemplateComparisonDialog from '@/features/models/TemplateComparisonDialog.vue'
import {templateDifferences} from '@/features/models/templateChanges'
const revision=(id:string):Revision=>({id,createdAt:'2026-10-20T00:00:00Z',savedLabel:'旧名称',content:emptyContent(),digest:id,baseRevision:null,changes:[]})
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
describe('read-only saved version comparison',()=>{
 it('includes the name, full matching/advanced fields, variants, locks and evidence without changing either side',()=>{
  const before=revision('old'),draft:TemplateDraft={templateId:'template',baseRevision:'old',label:'新名称',content:emptyContent()};draft.content.description='说明';draft.content.matching.priority=10;draft.content.rules.effortMap={mode:'custom',value:{high:'h'}};draft.content.variants.push({id:'variant',name:'具体型号',suffix:'exact',rules:emptyRules(),lockedFields:['contextWindow']});draft.content.lockedFields=['description'];draft.content.sources=[{field:'description',url:'https://example.test/source',recordedAt:'2026-10-20',note:'文档'}]
  const snapshot=JSON.stringify([before,draft]);const fields=templateDifferences(before,draft).map(r=>r.field);expect(fields).toEqual(expect.arrayContaining(['名称','说明','匹配规则 · priority','通用配置 · effortMap','具体型号 · 具体型号','人工锁','来源']));expect(JSON.stringify([before,draft])).toBe(snapshot)
 })
 it('preselects the default and freezes the full comparison while the live editor/library changes',async()=>{
  const first=revision('first'),second=revision('second');second.content.description='默认说明';const template:Template={id:'template',source:'user',label:'模板',defaultRevision:'second',revisions:[first,second]};const draft:TemplateDraft={templateId:'template',baseRevision:'first',label:'旧名称',content:emptyContent()}
  const wrapper=mount(TemplateComparisonDialog,{props:{open:true,template,draft,libraryDigest:'library'},global:{stubs:{UiDialog:dialog,UiSelectField:true}}})
  expect(wrapper.getComponent(UiSelectField).props('modelValue')).toBe('second');expect(wrapper.text()).toContain('默认说明');expect(wrapper.text()).toContain('1 项差异')
  draft.content.description='后来人工值';await wrapper.setProps({draft,libraryDigest:'new-library'});expect(wrapper.text()).toContain('保留打开时');expect(wrapper.text()).not.toContain('后来人工值')
  wrapper.getComponent(UiSelectField).vm.$emit('update:modelValue','first');await wrapper.vm.$nextTick();expect(wrapper.text()).toContain('与所选保存版本一致');expect(wrapper.emitted('apply')).toBeUndefined();expect(template.defaultRevision).toBe('second');expect(draft.content.description).toBe('后来人工值');wrapper.unmount()
 })
 it('treats omitted advanced inheritance as unchanged and keeps adversarial text as text',()=>{
  const before=revision('old'),draft:TemplateDraft={templateId:'template',baseRevision:'old',label:'<img src=x onerror=alert(1)>',content:emptyContent()};delete before.content.rules.effortMap
  expect(templateDifferences(before,draft).map(d=>d.field)).toEqual(['名称']);const template:Template={id:'template',source:'user',label:'template',defaultRevision:null,revisions:[before]};const wrapper=mount(TemplateComparisonDialog,{props:{open:true,template,draft,libraryDigest:'l'},global:{stubs:{UiDialog:dialog,UiSelectField:true}}});expect(wrapper.text()).toContain(draft.label);expect(wrapper.find('img').exists()).toBe(false);wrapper.unmount()
 })
})
