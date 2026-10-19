import {onMounted,onBeforeUnmount,ref} from 'vue'
/** CSS media widths do not follow the app's CSS zoom consistently across
 * WebViews. Layout uses viewport / the same saved scale as all controls. */
export function compactShellFor(width:number,zoom:number):boolean{return width/Math.max(.5,Math.min(2,zoom||1))<=760}
export function useShellLayout(){
 const compact=ref(false);let observer:MutationObserver|undefined
 function update(){const scale=Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-zoom'))||1;compact.value=compactShellFor(window.innerWidth,scale)}
 onMounted(()=>{update();observer=new MutationObserver(update);observer.observe(document.documentElement,{attributes:true,attributeFilter:['style']});window.addEventListener('resize',update)})
 onBeforeUnmount(()=>{observer?.disconnect();window.removeEventListener('resize',update)})
 return compact
}
