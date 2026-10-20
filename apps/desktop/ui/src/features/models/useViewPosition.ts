import {nextTick,onBeforeUnmount,onMounted,watch,type Ref} from 'vue'
import {useModelsStore} from './store'

// Session-only visual context. Never restores secrets or writes business state.
export function useModelViewPosition(root:Ref<HTMLElement|null>,key:()=>string){
 const models=useModelsStore();let generation=0;let current=key()
 function capture(id:string){
  const main=root.value?.closest<HTMLElement>('.main');if(!main)return
  const active=document.activeElement instanceof HTMLElement?document.activeElement:null
  models.view.positions[id]={top:main.scrollTop,focus:active&&root.value?.contains(active)?active.dataset.modelFocus??null:null}
 }
 async function restore(id:string){
  const sequence=++generation;await nextTick();if(sequence!==generation)return
  const main=root.value?.closest<HTMLElement>('.main');if(!main)return
  const saved=models.view.positions[id];main.scrollTop=saved?.top??0
  if(saved?.focus){const element=[...root.value!.querySelectorAll<HTMLElement>('[data-model-focus]')].find(e=>e.dataset.modelFocus===saved.focus);element?.focus({preventScroll:true})}
 }
 watch(key,next=>{capture(current);current=next;void restore(next)},{flush:'pre'})
 onMounted(()=>{current=key();void restore(current)})
 onBeforeUnmount(()=>{capture(current);generation++})
}
