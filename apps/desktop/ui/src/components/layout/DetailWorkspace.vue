<script setup lang="ts">
import { layoutObserver } from '@/lib/layoutObserver'
import {nextTick,onBeforeUnmount,onMounted,ref,watch} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
const props=withDefaults(defineProps<{label:string;sections:{id:string;label:string}[];collapsed?:boolean;activeId?:string}>(),{collapsed:false,activeId:''})
const emit=defineEmits<{'update:collapsed':[boolean];'update:activeId':[string]}>()
const root=ref<HTMLElement|null>(null),head=ref<HTMLElement|null>(null)
let main:HTMLElement|null=null,observer:ReturnType<typeof layoutObserver>,frame=0,reflow=false,disposed=false
function targets(){return props.sections.flatMap(item=>{const element=[...root.value?.querySelectorAll<HTMLElement>('[id]')??[]].find(e=>e.id===item.id);return element?[{...item,element}]:[]})}
function scale(){return main&&main.offsetHeight?main.getBoundingClientRect().height/main.offsetHeight:1}
function offset(){if(!root.value)return 12;const style=getComputedStyle(root.value);return (Number.parseFloat(style.getPropertyValue('--page-tabs-height'))||0)+(head.value?.offsetHeight??0)+12+(main?.clientTop??0)+(main?Number.parseFloat(getComputedStyle(main).paddingTop)||0:0)-(Number.parseFloat(style.getPropertyValue('--page-tabs-inset-top'))||0)}
function fitNavigation(){
 if(!main||!root.value)return
 const navigation=root.value.querySelector<HTMLElement>('.detail-navigation');if(!navigation)return
 const zoom=scale()||1,viewport=main.getBoundingClientRect(),position=navigation.getBoundingClientRect()
 const available=(viewport.bottom-position.top)/zoom-12
 const collapse=navigation.querySelector<HTMLElement>('.detail-collapse'),primary=navigation.querySelector<HTMLElement>('.detail-navigation-primary')
 // At high UI zoom the non-sticky page introduction can consume the initial
 // viewport. Reveal the workspace only as far as its sticky boundary permits;
 // never keep scrolling a form once that boundary has been reached.
 const minimum=(collapse?.offsetHeight??0)+(primary?.offsetHeight??0)+56
 const stickyTop=Number.parseFloat(getComputedStyle(navigation).top)||0
 const lift=Math.min(minimum-available,(position.top-viewport.top)/zoom-stickyTop,main.scrollHeight-main.clientHeight-main.scrollTop)
 if(lift>1){main.scrollTop+=lift;schedule();return}
 const height=Math.max(0,available)+'px'
 if(root.value.style.getPropertyValue('--detail-navigation-height')!==height)root.value.style.setProperty('--detail-navigation-height',height)
}
function updateActive(){
 frame=0;fitNavigation();if(reflow||!main||!root.value?.getClientRects().length)return
 const top=main.getBoundingClientRect().top+offset()*scale()
 const visible=targets();const active=main.scrollTop+main.clientHeight>=main.scrollHeight-2?visible.at(-1):[...visible].reverse().find(item=>item.element.getBoundingClientRect().top<=top+2)??visible[0]
 if(active&&active.id!==props.activeId)emit('update:activeId',active.id)
}
function schedule(){if(!frame)frame=requestAnimationFrame(updateActive)}
function measure(){
 if(!root.value||!main)return
 const set=(key:string,value:string)=>{if(root.value!.style.getPropertyValue(key)!==value)root.value!.style.setProperty(key,value)}
 set('--detail-header-height',String(head.value?.offsetHeight??0)+'px')
 set('--detail-viewport-height',String(main.clientHeight)+'px')
 fitNavigation();schedule()
}
function go(id:string,focus=true){
 const item=targets().find(item=>item.id===id);if(!item||!main)return
 const zoom=scale()||1
 main.scrollTop=Math.max(0,main.scrollTop+(item.element.getBoundingClientRect().top-main.getBoundingClientRect().top)/zoom-offset())
 emit('update:activeId',id)
 if(focus)item.element.focus({preventScroll:true})
 schedule()
}
async function collapse(){
 const current=props.activeId
 const target=targets().find(item=>item.id===current)
 const relative=target&&main?(target.element.getBoundingClientRect().top-main.getBoundingClientRect().top)/(scale()||1):undefined
 reflow=true;emit('update:collapsed',!props.collapsed);await nextTick();if(disposed)return;measure()
 if(frame)cancelAnimationFrame(frame)
 // Preserve the position within the section, including a partly edited row far below its heading.
 frame=requestAnimationFrame(()=>{
  frame=0
  const fresh=targets().find(item=>item.id===current)
  if(fresh&&main&&relative!==undefined)main.scrollTop=Math.max(0,main.scrollTop+(fresh.element.getBoundingClientRect().top-main.getBoundingClientRect().top)/(scale()||1)-relative)
  reflow=false;schedule()
 })
}
onMounted(()=>{
 main=root.value?.closest<HTMLElement>('.main')??root.value?.parentElement??null
 if(typeof ResizeObserver!=='undefined'){observer=layoutObserver(measure)!;if(main)observer.observe(main);if(head.value)observer.observe(head.value)}
 main?.addEventListener('scroll',schedule,{passive:true});window.addEventListener('resize',measure);measure()
})
watch(()=>props.sections.map(s=>s.id).join('\0'),()=>void nextTick().then(measure))
onBeforeUnmount(()=>{disposed=true;observer?.disconnect();main?.removeEventListener('scroll',schedule);window.removeEventListener('resize',measure);if(frame)cancelAnimationFrame(frame)})
</script>
<template>
 <div ref="root" class="detail-workspace" :class="{'detail-collapsed':collapsed}">
  <header ref="head" class="detail-workspace-head"><slot name="header" /></header>
  <div class="detail-workspace-body">
   <nav class="detail-navigation" :aria-label="label">
    <UiButton size="sm" variant="ghost" class="detail-collapse" :aria-label="(collapsed?'展开':'收起')+label" :title="(collapsed?'展开':'收起')+label" :aria-expanded="!collapsed" @click="collapse"><svg class="detail-nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16"/></svg><span v-if="!collapsed">收起导航</span></UiButton>
    <div class="detail-navigation-links"><a v-for="(item,index) in sections" :key="item.id" :href="'#'+item.id" :title="item.label" :aria-label="item.label" :aria-current="activeId===item.id?'location':undefined" @click.prevent="go(item.id)"><span class="detail-nav-number" aria-hidden="true">{{ String(index+1).padStart(2,'0') }}</span><span v-if="!collapsed">{{ item.label }}</span></a></div>
    <div v-if="$slots.actions" class="detail-navigation-actions"><slot name="actions" :collapsed="collapsed" /></div>
    <div v-if="$slots.primary" class="detail-navigation-primary"><slot name="primary" :collapsed="collapsed" /></div>
   </nav>
   <div class="detail-workspace-content"><slot /></div>
  </div>
 </div>
</template>
<style scoped>
.detail-workspace{overflow-anchor:none;min-width:0;--detail-nav-width:148px;--detail-header-height:48px}.detail-workspace.detail-collapsed{--detail-nav-width:42px}
.detail-workspace-head{position:sticky;top:calc(var(--page-tabs-height,0px) - var(--page-tabs-inset-top,0px));margin-inline-start:calc(-1 * var(--page-tabs-inset-left,0px));margin-inline-end:calc(-1 * var(--page-tabs-inset-right,0px));padding-inline-start:var(--page-tabs-inset-left,0px);padding-inline-end:var(--page-tabs-inset-right,0px);z-index:calc(var(--z-sticky,20) + 1);display:flex;flex-wrap:wrap;align-items:center;gap:var(--space-2);min-width:0;padding:10px var(--page-tabs-inset-right,0px) 10px var(--page-tabs-inset-left,0px);border-bottom:1px solid var(--border);background:var(--surface-sticky-bg,var(--surface-panel-bg));backdrop-filter:var(--surface-sticky-blur);-webkit-backdrop-filter:var(--surface-sticky-blur)}
.detail-workspace-body{display:grid;grid-template-columns:var(--detail-nav-width) minmax(0,1fr);gap:24px;margin-top:20px;align-items:start;min-width:0}
.detail-navigation{position:sticky;top:calc(var(--page-tabs-height,0px) + var(--detail-header-height) + 12px - var(--page-tabs-inset-top,0px));max-height:var(--detail-navigation-height,calc(var(--detail-viewport-height,100dvh) - var(--page-tabs-height,0px) - var(--detail-header-height) - 24px));overflow:hidden;display:flex;flex-direction:column;gap:4px;min-width:0;padding:2px}
.detail-navigation a{display:flex;align-items:center;gap:8px;flex:none;min-height:34px;padding:6px;color:var(--muted);border-radius:var(--radius-sm);text-decoration:none;font-size:var(--text-control);overflow-wrap:anywhere}.detail-navigation a:hover,.detail-navigation a[aria-current]{color:var(--text);background:var(--hover)}.detail-navigation a:focus-visible{outline:2px solid var(--accent);outline-offset:-2px}
.detail-nav-icon{width:16px;height:16px;flex:none}.detail-nav-number{width:20px;text-align:center;flex:none;font-size:var(--text-caption)}.detail-collapse{position:sticky;top:0;z-index:var(--z-sticky);flex:none;justify-content:flex-start;background:var(--surface-sticky-bg,var(--surface-panel-bg));backdrop-filter:var(--surface-sticky-blur);-webkit-backdrop-filter:var(--surface-sticky-blur)}.detail-navigation-actions{display:grid;gap:8px;margin-top:16px;padding-top:12px;border-top:1px solid var(--border)}.detail-workspace-content{min-width:0;container-type:inline-size;container-name:detail-content}.detail-workspace-content :deep(section[id]){scroll-margin-top:calc(var(--page-tabs-height,0px) + var(--detail-header-height) + 12px)}.detail-workspace-content :deep(section[id]:focus){outline:none}
@supports not ((backdrop-filter:blur(1px)) or (-webkit-backdrop-filter:blur(1px))){.detail-workspace-head{background:var(--panel)}}
@media(max-width:680px){.detail-workspace-body{gap:12px}}
</style>

<style scoped>
.detail-navigation-links{display:grid;gap:var(--space-1);min-height:0;overflow:auto;flex:0 1 auto}
.detail-navigation-actions{min-height:0;overflow:auto;flex:0 1 auto}
.detail-navigation-primary{flex:none;display:grid;gap:var(--space-2);padding-top:var(--space-2);border-top:1px solid var(--border);margin-top:auto}
.detail-navigation-primary :deep(.btn){width:100%}
</style>
